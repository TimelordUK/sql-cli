use crate::api_client::{ApiClient, QueryResponse};
use crate::parser::SqlParser;
use crate::cursor_aware_parser::CursorAwareParser;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Cell},
    Frame, Terminal,
};
use regex::Regex;
use serde_json::Value;
use std::io;
use std::cmp::Ordering;
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Clone, PartialEq)]
enum AppMode {
    Command,
    Results,
    Search,
    Filter,
    Help,
}

#[derive(Clone, PartialEq, Copy)]
enum SortOrder {
    Ascending,
    Descending,
    None,
}

#[derive(Clone)]
struct SortState {
    column: Option<usize>,
    order: SortOrder,
}

#[derive(Clone)]
struct FilterState {
    pattern: String,
    regex: Option<Regex>,
    active: bool,
}

#[derive(Clone)]
struct SearchState {
    pattern: String,
    current_match: Option<(usize, usize)>, // (row, col)
    matches: Vec<(usize, usize)>,
    match_index: usize,
}

#[derive(Clone)]
struct CompletionState {
    suggestions: Vec<String>,
    current_index: usize,
    last_query: String,
    last_cursor_pos: usize,
}

#[derive(Clone)]
pub struct EnhancedTuiApp {
    api_client: ApiClient,
    input: Input,
    mode: AppMode,
    results: Option<QueryResponse>,
    table_state: TableState,
    show_help: bool,
    status_message: String,
    sql_parser: SqlParser,
    cursor_parser: CursorAwareParser,
    
    // Enhanced features
    sort_state: SortState,
    filter_state: FilterState,
    search_state: SearchState,
    completion_state: CompletionState,
    filtered_data: Option<Vec<Vec<String>>>,
    column_widths: Vec<u16>,
    scroll_offset: (usize, usize), // (row, col)
}

impl EnhancedTuiApp {
    pub fn new(api_url: &str) -> Self {
        Self {
            api_client: ApiClient::new(api_url),
            input: Input::default(),
            mode: AppMode::Command,
            results: None,
            table_state: TableState::default(),
            show_help: false,
            status_message: "Ready - Type SQL query and press Enter (Enhanced mode with sorting/filtering)".to_string(),
            sql_parser: SqlParser::new(),
            cursor_parser: CursorAwareParser::new(),
            
            sort_state: SortState {
                column: None,
                order: SortOrder::None,
            },
            filter_state: FilterState {
                pattern: String::new(),
                regex: None,
                active: false,
            },
            search_state: SearchState {
                pattern: String::new(),
                current_match: None,
                matches: Vec::new(),
                match_index: 0,
            },
            completion_state: CompletionState {
                suggestions: Vec::new(),
                current_index: 0,
                last_query: String::new(),
                last_cursor_pos: 0,
            },
            filtered_data: None,
            column_widths: Vec::new(),
            scroll_offset: (0, 0),
        }
    }

    pub fn run(mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            self.status_message = format!("Error: {}", err);
        }

        Ok(())
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match self.mode {
                    AppMode::Command => {
                        if self.handle_command_input(key)? {
                            break;
                        }
                    },
                    AppMode::Results => {
                        if self.handle_results_input(key)? {
                            break;
                        }
                    },
                    AppMode::Search => {
                        if self.handle_search_input(key)? {
                            break;
                        }
                    },
                    AppMode::Filter => {
                        if self.handle_filter_input(key)? {
                            break;
                        }
                    },
                    AppMode::Help => {
                        if self.handle_help_input(key)? {
                            break;
                        }
                    },
                }
            }
        }
        Ok(())
    }

    fn handle_command_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                self.mode = if self.show_help { AppMode::Help } else { AppMode::Command };
            },
            KeyCode::Enter => {
                let query = self.input.value().trim().to_string();
                if !query.is_empty() {
                    self.execute_query(&query)?;
                }
            },
            KeyCode::Tab => {
                self.apply_completion();
            },
            KeyCode::Down if self.results.is_some() => {
                self.mode = AppMode::Results;
                self.table_state.select(Some(0));
            },
            _ => {
                self.input.handle_event(&Event::Key(key));
                // Clear completion state when typing other characters
                self.completion_state.suggestions.clear();
                self.completion_state.current_index = 0;
                self.handle_completion();
            }
        }
        Ok(false)
    }

    fn handle_results_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Esc => {
                self.mode = AppMode::Command;
                self.table_state.select(None);
            },
            KeyCode::Up => {
                self.mode = AppMode::Command;
                self.table_state.select(None);
            },
            // Vim-like navigation
            KeyCode::Char('j') | KeyCode::Down => {
                self.next_row();
            },
            KeyCode::Char('k') => {
                self.previous_row();
            },
            KeyCode::Char('h') | KeyCode::Left => {
                self.scroll_left();
            },
            KeyCode::Char('l') | KeyCode::Right => {
                self.scroll_right();
            },
            KeyCode::Char('g') => {
                self.goto_first_row();
            },
            KeyCode::Char('G') => {
                self.goto_last_row();
            },
            KeyCode::PageDown | KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.page_down();
            },
            KeyCode::PageUp | KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.page_up();
            },
            // Search functionality
            KeyCode::Char('/') => {
                self.mode = AppMode::Search;
                self.search_state.pattern.clear();
            },
            KeyCode::Char('n') => {
                self.next_search_match();
            },
            KeyCode::Char('N') => {
                self.previous_search_match();
            },
            // Filter functionality
            KeyCode::Char('F') => {
                self.mode = AppMode::Filter;
                self.filter_state.pattern.clear();
            },
            // Sort functionality
            KeyCode::Char('s') => {
                if let Some(_selected) = self.table_state.selected() {
                    // Sort by current column (approximate)
                    self.sort_by_column(0);
                }
            },
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.show_help = true;
                self.mode = AppMode::Help;
            },
            _ => {}
        }
        Ok(false)
    }

    fn handle_search_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Results;
            },
            KeyCode::Enter => {
                self.perform_search();
                self.mode = AppMode::Results;
            },
            KeyCode::Backspace => {
                self.search_state.pattern.pop();
            },
            KeyCode::Char(c) => {
                self.search_state.pattern.push(c);
            },
            _ => {}
        }
        Ok(false)
    }

    fn handle_filter_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Results;
            },
            KeyCode::Enter => {
                self.apply_filter();
                self.mode = AppMode::Results;
            },
            KeyCode::Backspace => {
                self.filter_state.pattern.pop();
            },
            KeyCode::Char(c) => {
                self.filter_state.pattern.push(c);
            },
            _ => {}
        }
        Ok(false)
    }

    fn handle_help_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::F(1) => {
                self.show_help = false;
                self.mode = if self.results.is_some() { AppMode::Results } else { AppMode::Command };
            },
            _ => {}
        }
        Ok(false)
    }

    fn execute_query(&mut self, query: &str) -> Result<()> {
        self.status_message = "Executing query...".to_string();
        
        match self.api_client.query_trades(query) {
            Ok(response) => {
                self.results = Some(response);
                self.reset_table_state();
                self.status_message = "Query executed successfully - Use ↓ or j/k to navigate results".to_string();
                self.mode = AppMode::Results;
                self.table_state.select(Some(0));
            },
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
        Ok(())
    }

    fn handle_completion(&mut self) {
        let cursor_pos = self.input.cursor();
        let query = self.input.value();
        
        let parse_result = self.cursor_parser.get_completions(query, cursor_pos);
        if !parse_result.suggestions.is_empty() {
            self.status_message = format!("Suggestions: {}", parse_result.suggestions.join(", "));
        }
    }

    fn apply_completion(&mut self) {
        let cursor_pos = self.input.cursor();
        let query = self.input.value();
        
        // Check if this is a continuation of the same completion session
        let is_same_context = query == self.completion_state.last_query && 
                             cursor_pos == self.completion_state.last_cursor_pos;
        
        if !is_same_context {
            // New completion context - get fresh suggestions
            let parse_result = self.cursor_parser.get_completions(query, cursor_pos);
            if parse_result.suggestions.is_empty() {
                self.status_message = "No completions available".to_string();
                return;
            }
            
            self.completion_state.suggestions = parse_result.suggestions;
            self.completion_state.current_index = 0;
        } else if !self.completion_state.suggestions.is_empty() {
            // Cycle to next suggestion
            self.completion_state.current_index = 
                (self.completion_state.current_index + 1) % self.completion_state.suggestions.len();
        } else {
            self.status_message = "No completions available".to_string();
            return;
        }
        
        // Apply the current suggestion
        let suggestion = &self.completion_state.suggestions[self.completion_state.current_index];
        let partial_word = self.extract_partial_word_at_cursor(query, cursor_pos);
        
        if let Some(partial) = partial_word {
            // Replace the partial word with the suggestion
            let before_partial = &query[..cursor_pos - partial.len()];
            let after_cursor = &query[cursor_pos..];
            let new_query = format!("{}{}{}", before_partial, suggestion, after_cursor);
            
            // Update input and cursor position
            let cursor_pos = before_partial.len() + suggestion.len();
            self.input = tui_input::Input::new(new_query.clone()).with_cursor(cursor_pos);
            
            // Update completion state for next tab press
            self.completion_state.last_query = new_query;
            self.completion_state.last_cursor_pos = cursor_pos;
            
            let suggestion_info = if self.completion_state.suggestions.len() > 1 {
                format!("Completed: {} ({}/{} - Tab for next)", 
                    suggestion, 
                    self.completion_state.current_index + 1, 
                    self.completion_state.suggestions.len())
            } else {
                format!("Completed: {}", suggestion)
            };
            self.status_message = suggestion_info;
            
        } else {
            // Just insert the suggestion at cursor position
            let before_cursor = &query[..cursor_pos];
            let after_cursor = &query[cursor_pos..];
            let new_query = format!("{}{}{}", before_cursor, suggestion, after_cursor);
            
            let cursor_pos_new = cursor_pos + suggestion.len();
            self.input = tui_input::Input::new(new_query.clone()).with_cursor(cursor_pos_new);
            
            // Update completion state
            self.completion_state.last_query = new_query;
            self.completion_state.last_cursor_pos = cursor_pos_new;
            
            self.status_message = format!("Inserted: {}", suggestion);
        }
    }

    fn extract_partial_word_at_cursor(&self, query: &str, cursor_pos: usize) -> Option<String> {
        if cursor_pos == 0 || cursor_pos > query.len() {
            return None;
        }

        let chars: Vec<char> = query.chars().collect();
        let mut start = cursor_pos;
        let end = cursor_pos;

        // Find start of word (go backward)
        while start > 0 {
            let prev_char = chars[start - 1];
            if prev_char.is_alphanumeric() || prev_char == '_' {
                start -= 1;
            } else {
                break;
            }
        }

        // Convert back to byte positions
        let start_byte = chars[..start].iter().map(|c| c.len_utf8()).sum();
        let end_byte = chars[..end].iter().map(|c| c.len_utf8()).sum();

        if start_byte < end_byte {
            Some(query[start_byte..end_byte].to_string())
        } else {
            None
        }
    }

    // Navigation functions
    fn next_row(&mut self) {
        if let Some(data) = self.get_current_data() {
            let i = match self.table_state.selected() {
                Some(i) => (i + 1).min(data.len().saturating_sub(1)),
                None => 0,
            };
            self.table_state.select(Some(i));
        }
    }

    fn previous_row(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn scroll_left(&mut self) {
        self.scroll_offset.1 = self.scroll_offset.1.saturating_sub(1);
    }

    fn scroll_right(&mut self) {
        self.scroll_offset.1 += 1;
    }

    fn goto_first_row(&mut self) {
        self.table_state.select(Some(0));
    }

    fn goto_last_row(&mut self) {
        if let Some(data) = self.get_current_data() {
            if !data.is_empty() {
                self.table_state.select(Some(data.len() - 1));
            }
        }
    }

    fn page_down(&mut self) {
        if let Some(data) = self.get_current_data() {
            let i = match self.table_state.selected() {
                Some(i) => (i + 10).min(data.len().saturating_sub(1)),
                None => 0,
            };
            self.table_state.select(Some(i));
        }
    }

    fn page_up(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => i.saturating_sub(10),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    // Search and filter functions
    fn perform_search(&mut self) {
        if let Some(data) = self.get_current_data() {
            self.search_state.matches.clear();
            
            if let Ok(regex) = Regex::new(&self.search_state.pattern) {
                for (row_idx, row) in data.iter().enumerate() {
                    for (col_idx, cell) in row.iter().enumerate() {
                        if regex.is_match(cell) {
                            self.search_state.matches.push((row_idx, col_idx));
                        }
                    }
                }
                
                if !self.search_state.matches.is_empty() {
                    self.search_state.match_index = 0;
                    self.search_state.current_match = Some(self.search_state.matches[0]);
                    let (row, _) = self.search_state.matches[0];
                    self.table_state.select(Some(row));
                    self.status_message = format!("Found {} matches", self.search_state.matches.len());
                } else {
                    self.status_message = "No matches found".to_string();
                }
            } else {
                self.status_message = "Invalid regex pattern".to_string();
            }
        }
    }

    fn next_search_match(&mut self) {
        if !self.search_state.matches.is_empty() {
            self.search_state.match_index = (self.search_state.match_index + 1) % self.search_state.matches.len();
            let (row, _) = self.search_state.matches[self.search_state.match_index];
            self.table_state.select(Some(row));
            self.search_state.current_match = Some(self.search_state.matches[self.search_state.match_index]);
            self.status_message = format!("Match {} of {}", self.search_state.match_index + 1, self.search_state.matches.len());
        }
    }

    fn previous_search_match(&mut self) {
        if !self.search_state.matches.is_empty() {
            self.search_state.match_index = if self.search_state.match_index == 0 {
                self.search_state.matches.len() - 1
            } else {
                self.search_state.match_index - 1
            };
            let (row, _) = self.search_state.matches[self.search_state.match_index];
            self.table_state.select(Some(row));
            self.search_state.current_match = Some(self.search_state.matches[self.search_state.match_index]);
            self.status_message = format!("Match {} of {}", self.search_state.match_index + 1, self.search_state.matches.len());
        }
    }

    fn apply_filter(&mut self) {
        if self.filter_state.pattern.is_empty() {
            self.filtered_data = None;
            self.filter_state.active = false;
            self.status_message = "Filter cleared".to_string();
            return;
        }

        if let Some(results) = &self.results {
            if let Ok(regex) = Regex::new(&self.filter_state.pattern) {
                let mut filtered = Vec::new();
                
                for item in &results.data {
                    let mut row = Vec::new();
                    let mut matches = false;
                    
                    for (_, value) in item.as_object().unwrap() {
                        let cell_str = match value {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "".to_string(),
                            _ => value.to_string(),
                        };
                        
                        if regex.is_match(&cell_str) {
                            matches = true;
                        }
                        row.push(cell_str);
                    }
                    
                    if matches {
                        filtered.push(row);
                    }
                }
                
                self.filtered_data = Some(filtered);
                self.filter_state.regex = Some(regex);
                self.filter_state.active = true;
                self.reset_table_state();
                self.status_message = format!("Filtered to {} rows", self.filtered_data.as_ref().unwrap().len());
            } else {
                self.status_message = "Invalid regex pattern".to_string();
            }
        }
    }

    fn sort_by_column(&mut self, column_index: usize) {
        let new_order = match &self.sort_state {
            SortState { column: Some(col), order } if *col == column_index => {
                match order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::None,
                    SortOrder::None => SortOrder::Ascending,
                }
            },
            _ => SortOrder::Ascending,
        };

        if new_order == SortOrder::None {
            // Reset to original order - would need to store original data
            self.sort_state = SortState { column: None, order: SortOrder::None };
            self.status_message = "Sort cleared".to_string();
            return;
        }

        if let Some(data) = self.get_current_data_mut() {
            data.sort_by(|a, b| {
                if column_index >= a.len() || column_index >= b.len() {
                    return Ordering::Equal;
                }

                let cell_a = &a[column_index];
                let cell_b = &b[column_index];

                // Try numeric comparison first
                if let (Ok(num_a), Ok(num_b)) = (cell_a.parse::<f64>(), cell_b.parse::<f64>()) {
                    let cmp = num_a.partial_cmp(&num_b).unwrap_or(Ordering::Equal);
                    match new_order {
                        SortOrder::Ascending => cmp,
                        SortOrder::Descending => cmp.reverse(),
                        SortOrder::None => Ordering::Equal,
                    }
                } else {
                    // String comparison
                    let cmp = cell_a.cmp(cell_b);
                    match new_order {
                        SortOrder::Ascending => cmp,
                        SortOrder::Descending => cmp.reverse(),
                        SortOrder::None => Ordering::Equal,
                    }
                }
            });

            self.sort_state = SortState { column: Some(column_index), order: new_order };
            self.reset_table_state();
            self.status_message = format!("Sorted by column {} ({})", 
                column_index + 1, 
                match new_order {
                    SortOrder::Ascending => "ascending",
                    SortOrder::Descending => "descending",
                    SortOrder::None => "none",
                }
            );
        }
    }

    fn get_current_data(&self) -> Option<Vec<Vec<String>>> {
        if let Some(filtered) = &self.filtered_data {
            Some(filtered.clone())
        } else if let Some(results) = &self.results {
            Some(self.convert_json_to_strings(results))
        } else {
            None
        }
    }

    fn get_current_data_mut(&mut self) -> Option<&mut Vec<Vec<String>>> {
        if self.filtered_data.is_none() && self.results.is_some() {
            let results = self.results.as_ref().unwrap();
            self.filtered_data = Some(self.convert_json_to_strings(results));
        }
        self.filtered_data.as_mut()
    }

    fn convert_json_to_strings(&self, results: &QueryResponse) -> Vec<Vec<String>> {
        if let Some(first_row) = results.data.first() {
            if let Some(obj) = first_row.as_object() {
                let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                
                results.data.iter().map(|item| {
                    if let Some(obj) = item.as_object() {
                        headers.iter().map(|&header| {
                            match obj.get(header) {
                                Some(Value::String(s)) => s.clone(),
                                Some(Value::Number(n)) => n.to_string(),
                                Some(Value::Bool(b)) => b.to_string(),
                                Some(Value::Null) => "".to_string(),
                                Some(other) => other.to_string(),
                                None => "".to_string(),
                            }
                        }).collect()
                    } else {
                        vec![]
                    }
                }).collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    fn reset_table_state(&mut self) {
        self.table_state = TableState::default();
        self.scroll_offset = (0, 0);
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Command input
                Constraint::Min(0),    // Results
                Constraint::Length(3), // Status bar
            ].as_ref())
            .split(f.area());

        // Command input area
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(match self.mode {
                AppMode::Command => "SQL Query",
                AppMode::Results => "SQL Query (Results Mode - Press ↑ to edit)",
                AppMode::Search => "Search Pattern",
                AppMode::Filter => "Filter Pattern", 
                AppMode::Help => "Help",
            });

        let input_text = match self.mode {
            AppMode::Search => &self.search_state.pattern,
            AppMode::Filter => &self.filter_state.pattern,
            _ => self.input.value(),
        };

        let input_paragraph = Paragraph::new(input_text)
            .block(input_block)
            .style(match self.mode {
                AppMode::Command => Style::default(),
                AppMode::Results => Style::default().fg(Color::DarkGray),
                AppMode::Search => Style::default().fg(Color::Yellow),
                AppMode::Filter => Style::default().fg(Color::Cyan),
                AppMode::Help => Style::default().fg(Color::DarkGray),
            });

        f.render_widget(input_paragraph, chunks[0]);

        // Set cursor position for input modes
        match self.mode {
            AppMode::Command => {
                f.set_cursor_position((
                    chunks[0].x + self.input.visual_cursor() as u16 + 1,
                    chunks[0].y + 1
                ));
            },
            AppMode::Search => {
                f.set_cursor_position((
                    chunks[0].x + self.search_state.pattern.len() as u16 + 1,
                    chunks[0].y + 1
                ));
            },
            AppMode::Filter => {
                f.set_cursor_position((
                    chunks[0].x + self.filter_state.pattern.len() as u16 + 1,
                    chunks[0].y + 1
                ));
            },
            _ => {}
        }

        // Results area
        if self.show_help {
            self.render_help(f, chunks[1]);
        } else if let Some(results) = &self.results {
            self.render_table(f, chunks[1], results);
        } else {
            let placeholder = Paragraph::new("Enter a SQL query above and press Enter to see results.\n\nSupported features:\n• Tab completion with cursor awareness\n• Dynamic LINQ expressions (Contains, IndexOf, etc.)\n• Column sorting (s key)\n• Search (/ key)\n• Filter (F key)\n• Vim-like navigation (j/k/h/l)")
                .block(Block::default().borders(Borders::ALL).title("Results"))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(placeholder, chunks[1]);
        }

        // Status bar
        let status_style = match self.mode {
            AppMode::Command => Style::default().fg(Color::Green),
            AppMode::Results => Style::default().fg(Color::Blue),
            AppMode::Search => Style::default().fg(Color::Yellow),
            AppMode::Filter => Style::default().fg(Color::Cyan),
            AppMode::Help => Style::default().fg(Color::Magenta),
        };

        let mode_indicator = match self.mode {
            AppMode::Command => "CMD",
            AppMode::Results => "NAV",
            AppMode::Search => "SEARCH",
            AppMode::Filter => "FILTER",
            AppMode::Help => "HELP",
        };

        let status_text = format!("[{}] {} | F1:Help q:Quit", mode_indicator, self.status_message);
        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .style(status_style);
        f.render_widget(status, chunks[2]);
    }

    fn render_table(&self, f: &mut Frame, area: Rect, results: &QueryResponse) {
        if results.data.is_empty() {
            let empty = Paragraph::new("No results found")
                .block(Block::default().borders(Borders::ALL).title("Results"))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(empty, area);
            return;
        }

        // Get headers from first row
        let headers: Vec<&str> = if let Some(first_row) = results.data.first() {
            if let Some(obj) = first_row.as_object() {
                obj.keys().map(|k| k.as_str()).collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // Prepare table data
        let data_to_display = if let Some(filtered) = &self.filtered_data {
            filtered.clone()
        } else {
            // Convert JSON data to string matrix
            results.data.iter().map(|item| {
                if let Some(obj) = item.as_object() {
                    headers.iter().map(|&header| {
                        match obj.get(header) {
                            Some(Value::String(s)) => s.clone(),
                            Some(Value::Number(n)) => n.to_string(),
                            Some(Value::Bool(b)) => b.to_string(),
                            Some(Value::Null) => "".to_string(),
                            Some(other) => other.to_string(),
                            None => "".to_string(),
                        }
                    }).collect()
                } else {
                    vec![]
                }
            }).collect::<Vec<Vec<String>>>()
        };

        // Create header row with sort indicators
        let header_cells: Vec<Cell> = headers.iter().enumerate().map(|(i, &header)| {
            let sort_indicator = if let Some(col) = self.sort_state.column {
                if col == i {
                    match self.sort_state.order {
                        SortOrder::Ascending => " ↑",
                        SortOrder::Descending => " ↓",
                        SortOrder::None => "",
                    }
                } else {
                    ""
                }
            } else {
                ""
            };
            
            Cell::from(format!("{}{}", header, sort_indicator))
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        }).collect();

        // Create data rows
        let rows: Vec<Row> = data_to_display.iter().enumerate().map(|(row_idx, row)| {
            let cells: Vec<Cell> = row.iter().enumerate().map(|(col_idx, cell)| {
                let mut style = Style::default();
                
                // Highlight search matches
                if let Some((match_row, match_col)) = self.search_state.current_match {
                    if row_idx == match_row && col_idx == match_col {
                        style = style.bg(Color::Yellow).fg(Color::Black);
                    }
                }
                
                // Highlight filter matches
                if self.filter_state.active {
                    if let Some(ref regex) = self.filter_state.regex {
                        if regex.is_match(cell) {
                            style = style.fg(Color::Cyan);
                        }
                    }
                }
                
                Cell::from(cell.as_str()).style(style)
            }).collect();
            
            Row::new(cells)
        }).collect();

        // Calculate column constraints
        let constraints: Vec<Constraint> = (0..headers.len())
            .map(|_| Constraint::Min(10))
            .collect();

        let table = Table::new(rows, constraints)
            .header(Row::new(header_cells).height(1))
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!("Results ({} rows)", data_to_display.len())))
            .row_highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("► ");

        f.render_stateful_widget(table, area, &mut self.table_state.clone());
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = Text::from(vec![
            Line::from("SQL CLI Help"),
            Line::from(""),
            Line::from("Command Mode:"),
            Line::from("  Enter    - Execute query"),
            Line::from("  Tab      - Auto-complete"),
            Line::from("  ↓        - Enter results mode"),
            Line::from("  F1/?     - Toggle help"),
            Line::from("  Ctrl+C/D - Exit"),
            Line::from(""),
            Line::from("Results Navigation Mode:"),
            Line::from("  j/↓      - Next row"),
            Line::from("  k/↑      - Previous row"), 
            Line::from("  h/←      - Scroll left"),
            Line::from("  l/→      - Scroll right"),
            Line::from("  g        - First row"),
            Line::from("  G        - Last row"),
            Line::from("  Ctrl+F   - Page down"),
            Line::from("  Ctrl+B   - Page up"),
            Line::from("  /        - Search"),
            Line::from("  n        - Next match"),
            Line::from("  N        - Previous match"),
            Line::from("  F        - Filter rows"),
            Line::from("  s        - Sort column"),
            Line::from("  ↑/Esc    - Back to command mode"),
            Line::from("  q        - Quit"),
            Line::from(""),
            Line::from("Search Mode:"),
            Line::from("  Enter    - Execute search"),
            Line::from("  Esc      - Cancel"),
            Line::from(""),
            Line::from("Filter Mode:"),
            Line::from("  Enter    - Apply filter"),
            Line::from("  Esc      - Cancel"),
        ]);

        let help_paragraph = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .style(Style::default());

        f.render_widget(help_paragraph, area);
    }
}

pub fn run_enhanced_tui(api_url: &str) -> Result<()> {
    let app = EnhancedTuiApp::new(api_url);
    app.run()
}