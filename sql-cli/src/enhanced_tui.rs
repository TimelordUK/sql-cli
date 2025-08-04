use crate::api_client::{ApiClient, QueryResponse};
use crate::parser::SqlParser;
use crate::hybrid_parser::HybridParser;
use crate::history::{CommandHistory, HistoryMatch};
use crate::sql_highlighter::SqlHighlighter;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Cell, Wrap},
    Frame, Terminal,
};
use regex::Regex;
use serde_json::Value;
use std::io;
use std::io::Write;
use std::fs::File;
use std::cmp::Ordering;
use tui_input::{backend::crossterm::EventHandler, Input};

#[derive(Clone, PartialEq)]
enum AppMode {
    Command,
    Results,
    Search,
    Filter,
    Help,
    History,
    Debug,
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
struct HistoryState {
    search_query: String,
    matches: Vec<HistoryMatch>,
    selected_index: usize,
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
    hybrid_parser: HybridParser,
    
    // Enhanced features
    sort_state: SortState,
    filter_state: FilterState,
    search_state: SearchState,
    completion_state: CompletionState,
    history_state: HistoryState,
    command_history: CommandHistory,
    filtered_data: Option<Vec<Vec<String>>>,
    column_widths: Vec<u16>,
    scroll_offset: (usize, usize), // (row, col)
    current_column: usize, // For column-based operations
    sql_highlighter: SqlHighlighter,
    debug_text: String,
    debug_scroll: u16,
    input_scroll_offset: u16, // Horizontal scroll offset for input
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        // Escape quotes by doubling them and wrap field in quotes
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn is_sql_delimiter(ch: char) -> bool {
    matches!(ch, ',' | '(' | ')' | '=' | '<' | '>' | '.' | '"' | '\'' | ';')
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
            hybrid_parser: HybridParser::new(),
            
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
            history_state: HistoryState {
                search_query: String::new(),
                matches: Vec::new(),
                selected_index: 0,
            },
            command_history: CommandHistory::new().unwrap_or_default(),
            filtered_data: None,
            column_widths: Vec::new(),
            scroll_offset: (0, 0),
            current_column: 0,
            sql_highlighter: SqlHighlighter::new(),
            debug_text: String::new(),
            debug_scroll: 0,
            input_scroll_offset: 0,
        }
    }

    pub fn run(mut self) -> Result<()> {
        // Setup terminal with error handling
        if let Err(e) = enable_raw_mode() {
            return Err(anyhow::anyhow!("Failed to enable raw mode: {}. Try running with --classic flag.", e));
        }
        
        let mut stdout = io::stdout();
        if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            let _ = disable_raw_mode();
            return Err(anyhow::anyhow!("Failed to setup terminal: {}. Try running with --classic flag.", e));
        }
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(e) => {
                let _ = disable_raw_mode();
                return Err(anyhow::anyhow!("Failed to create terminal: {}. Try running with --classic flag.", e));
            }
        };

        let res = self.run_app(&mut terminal);

        // Always restore terminal, even on error
        let _ = disable_raw_mode();
        let _ = execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = terminal.show_cursor();

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("TUI error: {}", e))
        }
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // Initial draw
        terminal.draw(|f| self.ui(f))?;
        
        loop {
            // Use polling with a reasonable timeout to limit frame rate
            if event::poll(std::time::Duration::from_millis(50))? { // ~20 FPS max, much more CPU friendly
                match event::read()? {
                    Event::Key(key) => {
                        let should_exit = match self.mode {
                            AppMode::Command => self.handle_command_input(key)?,
                            AppMode::Results => self.handle_results_input(key)?,
                            AppMode::Search => self.handle_search_input(key)?,
                            AppMode::Filter => self.handle_filter_input(key)?,
                            AppMode::Help => self.handle_help_input(key)?,
                            AppMode::History => self.handle_history_input(key)?,
                            AppMode::Debug => self.handle_debug_input(key)?,
                        };
                        
                        if should_exit {
                            break;
                        }
                        
                        // Only redraw after handling a key event
                        terminal.draw(|f| self.ui(f))?;
                    },
                    _ => {
                        // Ignore other events (mouse, resize, etc.) to reduce CPU
                    }
                }
            }
            // If no events, continue the loop without redrawing
        }
        Ok(())
    }

    fn handle_command_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Store old cursor position
        let old_cursor = self.input.cursor();
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
                    self.status_message = format!("Processing query: '{}'", query);
                    self.execute_query(&query)?;
                } else {
                    self.status_message = "Empty query - please enter a SQL command".to_string();
                }
            },
            KeyCode::Tab => {
                self.apply_completion();
            },
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.mode = AppMode::History;
                self.history_state.search_query.clear();
                self.update_history_matches();
            },
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Jump to beginning of line (like bash/zsh)
                self.input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Home, KeyModifiers::empty())));
            },
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Jump to end of line (like bash/zsh)
                self.input.handle_event(&Event::Key(KeyEvent::new(KeyCode::End, KeyModifiers::empty())));
            },
            KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Move backward one word
                self.move_cursor_word_backward();
            },
            KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Move forward one word
                self.move_cursor_word_forward();
            },
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move backward one word (alt+b like in bash)
                self.move_cursor_word_backward();
            },
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move forward one word (alt+f like in bash)
                self.move_cursor_word_forward();
            },
            KeyCode::Down if self.results.is_some() => {
                self.mode = AppMode::Results;
                self.table_state.select(Some(0));
            },
            KeyCode::F(5) => {
                // Debug command - show detailed parser information
                let cursor_pos = self.input.cursor();
                let visual_cursor = self.input.visual_cursor();
                let query = self.input.value();
                let mut debug_info = self.hybrid_parser.get_detailed_debug_info(query, cursor_pos);
                
                // Add input state information
                let input_state = format!(
                    "\n========== INPUT STATE ==========\n\
                    Input Value Length: {}\n\
                    Cursor Position: {}\n\
                    Visual Cursor: {}\n\
                    Input Mode: Command\n",
                    query.len(),
                    cursor_pos,
                    visual_cursor
                );
                debug_info.push_str(&input_state);
                
                // Store debug info and switch to debug mode
                self.debug_text = debug_info.clone();
                self.debug_scroll = 0;
                self.mode = AppMode::Debug;
                
                // Try to copy to clipboard
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => {
                        match clipboard.set_text(&debug_info) {
                            Ok(_) => {
                                self.status_message = "DEBUG INFO copied to clipboard!".to_string();
                            }
                            Err(e) => {
                                self.status_message = format!("Clipboard error: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.status_message = format!("Can't access clipboard: {}", e);
                    }
                }
            },
            _ => {
                self.input.handle_event(&Event::Key(key));
                // Clear completion state when typing other characters
                self.completion_state.suggestions.clear();
                self.completion_state.current_index = 0;
                self.handle_completion();
            }
        }
        
        // Update horizontal scroll if cursor moved
        if self.input.cursor() != old_cursor {
            self.update_horizontal_scroll(120); // Assume reasonable terminal width, will be adjusted in render
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
                self.move_column_left();
            },
            KeyCode::Char('l') | KeyCode::Right => {
                self.move_column_right();
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
                self.sort_by_column(self.current_column);
            },
            // Number keys for direct column sorting
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(digit) = c.to_digit(10) {
                    let column_index = (digit as usize).saturating_sub(1);
                    self.sort_by_column(column_index);
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

    fn handle_history_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Esc => {
                self.mode = AppMode::Command;
            },
            KeyCode::Enter => {
                if !self.history_state.matches.is_empty() && self.history_state.selected_index < self.history_state.matches.len() {
                    let selected_command = self.history_state.matches[self.history_state.selected_index].entry.command.clone();
                    let cursor_pos = selected_command.len();
                    self.input = tui_input::Input::new(selected_command).with_cursor(cursor_pos);
                    self.mode = AppMode::Command;
                    self.status_message = "Command loaded from history".to_string();
                    // Reset scroll to show end of command
                    self.input_scroll_offset = 0;
                    self.update_horizontal_scroll(120); // Will be properly updated on next render
                }
            },
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.history_state.matches.is_empty() {
                    self.history_state.selected_index = self.history_state.selected_index.saturating_sub(1);
                }
            },
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.history_state.matches.is_empty() && self.history_state.selected_index + 1 < self.history_state.matches.len() {
                    self.history_state.selected_index += 1;
                }
            },
            KeyCode::Backspace => {
                self.history_state.search_query.pop();
                self.update_history_matches();
            },
            KeyCode::Char(c) => {
                self.history_state.search_query.push(c);
                self.update_history_matches();
            },
            _ => {}
        }
        Ok(false)
    }

    fn update_history_matches(&mut self) {
        self.history_state.matches = self.command_history.search(&self.history_state.search_query);
        self.history_state.selected_index = 0;
    }

    fn handle_debug_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.mode = AppMode::Command;
            },
            KeyCode::Up | KeyCode::Char('k') => {
                if self.debug_scroll > 0 {
                    self.debug_scroll = self.debug_scroll.saturating_sub(1);
                }
            },
            KeyCode::Down | KeyCode::Char('j') => {
                self.debug_scroll = self.debug_scroll.saturating_add(1);
            },
            KeyCode::PageUp => {
                self.debug_scroll = self.debug_scroll.saturating_sub(10);
            },
            KeyCode::PageDown => {
                self.debug_scroll = self.debug_scroll.saturating_add(10);
            },
            _ => {}
        }
        Ok(false)
    }

    fn execute_query(&mut self, query: &str) -> Result<()> {
        self.status_message = format!("Executing query: '{}'...", query);
        let start_time = std::time::Instant::now();
        
        match self.api_client.query_trades(query) {
            Ok(response) => {
                let duration = start_time.elapsed();
                let _ = self.command_history.add_entry(
                    query.to_string(), 
                    true, 
                    Some(duration.as_millis() as u64)
                );
                
                self.results = Some(response);
                self.reset_table_state();
                self.status_message = "Query executed successfully - Use ↓ or j/k to navigate results".to_string();
                self.mode = AppMode::Results;
                self.table_state.select(Some(0));
            },
            Err(e) => {
                let duration = start_time.elapsed();
                let _ = self.command_history.add_entry(
                    query.to_string(), 
                    false, 
                    Some(duration.as_millis() as u64)
                );
                self.status_message = format!("Error: {}", e);
            }
        }
        Ok(())
    }

    fn handle_completion(&mut self) {
        let cursor_pos = self.input.cursor();
        let query = self.input.value();
        
        let hybrid_result = self.hybrid_parser.get_completions(query, cursor_pos);
        if !hybrid_result.suggestions.is_empty() {
            self.status_message = format!("Suggestions: {}", hybrid_result.suggestions.join(", "));
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
            let hybrid_result = self.hybrid_parser.get_completions(query, cursor_pos);
            if hybrid_result.suggestions.is_empty() {
                self.status_message = "No completions available".to_string();
                return;
            }
            
            self.completion_state.suggestions = hybrid_result.suggestions;
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

    fn move_column_left(&mut self) {
        self.current_column = self.current_column.saturating_sub(1);
        self.scroll_offset.1 = self.scroll_offset.1.saturating_sub(1);
        self.status_message = format!("Column {} selected", self.current_column + 1);
    }

    fn move_column_right(&mut self) {
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let max_columns = obj.len();
                    if self.current_column + 1 < max_columns {
                        self.current_column += 1;
                        self.scroll_offset.1 += 1;
                        self.status_message = format!("Column {} selected", self.current_column + 1);
                    }
                }
            }
        }
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
        self.current_column = 0;
    }

    fn export_to_csv(&mut self) {
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    // Generate filename with timestamp
                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                    let filename = format!("query_results_{}.csv", timestamp);
                    
                    match File::create(&filename) {
                        Ok(mut file) => {
                            // Write headers
                            let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                            let header_line = headers.join(",");
                            if let Err(e) = writeln!(file, "{}", header_line) {
                                self.status_message = format!("Failed to write headers: {}", e);
                                return;
                            }
                            
                            // Write data rows
                            let mut row_count = 0;
                            for item in &results.data {
                                if let Some(obj) = item.as_object() {
                                    let row: Vec<String> = headers.iter().map(|&header| {
                                        match obj.get(header) {
                                            Some(Value::String(s)) => escape_csv_field(s),
                                            Some(Value::Number(n)) => n.to_string(),
                                            Some(Value::Bool(b)) => b.to_string(),
                                            Some(Value::Null) => String::new(),
                                            Some(other) => escape_csv_field(&other.to_string()),
                                            None => String::new(),
                                        }
                                    }).collect();
                                    
                                    let row_line = row.join(",");
                                    if let Err(e) = writeln!(file, "{}", row_line) {
                                        self.status_message = format!("Failed to write row: {}", e);
                                        return;
                                    }
                                    row_count += 1;
                                }
                            }
                            
                            self.status_message = format!("Exported {} rows to {}", row_count, filename);
                        },
                        Err(e) => {
                            self.status_message = format!("Failed to create file: {}", e);
                        }
                    }
                } else {
                    self.status_message = "No data to export".to_string();
                }
            } else {
                self.status_message = "No data to export".to_string();
            }
        } else {
            self.status_message = "No results to export - run a query first".to_string();
        }
    }

    fn get_horizontal_scroll_offset(&self) -> u16 {
        self.input_scroll_offset
    }

    fn update_horizontal_scroll(&mut self, terminal_width: u16) {
        let inner_width = terminal_width.saturating_sub(3) as usize; // Account for borders + 1 char padding
        let cursor_pos = self.input.visual_cursor();
        
        // If cursor is before the scroll window, scroll left
        if cursor_pos < self.input_scroll_offset as usize {
            self.input_scroll_offset = cursor_pos as u16;
        }
        // If cursor is after the scroll window, scroll right
        else if cursor_pos >= self.input_scroll_offset as usize + inner_width {
            self.input_scroll_offset = (cursor_pos + 1).saturating_sub(inner_width) as u16;
        }
    }

    fn get_cursor_token_position(&self) -> (usize, usize) {
        let query = self.input.value();
        let cursor_pos = self.input.cursor();
        
        if query.is_empty() {
            return (0, 0);
        }
        
        // Use our lexer to tokenize the query
        use crate::recursive_parser::Lexer;
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all_with_positions();
        
        if tokens.is_empty() {
            return (0, 0);
        }
        
        // Find which token the cursor is in
        let mut current_token = 0;
        for (i, (start, end, _)) in tokens.iter().enumerate() {
            if cursor_pos >= *start && cursor_pos <= *end {
                current_token = i + 1;
                break;
            } else if cursor_pos < *start {
                // Cursor is between tokens
                current_token = i;
                break;
            }
        }
        
        // If cursor is after all tokens
        if current_token == 0 && cursor_pos > 0 {
            current_token = tokens.len();
        }
        
        (current_token, tokens.len())
    }

    fn move_cursor_word_backward(&mut self) {
        let query = self.input.value();
        let cursor_pos = self.input.cursor();
        
        if cursor_pos == 0 {
            return;
        }
        
        // Use our lexer to tokenize the query
        use crate::recursive_parser::Lexer;
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all_with_positions();
        
        // Find the token boundary before the cursor
        let mut target_pos = 0;
        for (start, end, _) in tokens.iter().rev() {
            if *end <= cursor_pos {
                // If we're at the start of a token, go to the previous one
                if *end == cursor_pos && start < &cursor_pos {
                    target_pos = *start;
                } else {
                    // Otherwise go to the start of this token
                    for (s, e, _) in tokens.iter().rev() {
                        if *e <= cursor_pos && *s < cursor_pos {
                            target_pos = *s;
                            break;
                        }
                    }
                }
                break;
            }
        }
        
        // Move cursor to new position
        let moves = cursor_pos.saturating_sub(target_pos);
        for _ in 0..moves {
            self.input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::empty())));
        }
    }

    fn move_cursor_word_forward(&mut self) {
        let query = self.input.value();
        let cursor_pos = self.input.cursor();
        let query_len = query.len();
        
        if cursor_pos >= query_len {
            return;
        }
        
        // Use our lexer to tokenize the query
        use crate::recursive_parser::Lexer;
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all_with_positions();
        
        // Find the next token boundary after the cursor
        let mut target_pos = query_len;
        for (start, end, _) in &tokens {
            if *start > cursor_pos {
                target_pos = *start;
                break;
            } else if *end > cursor_pos {
                target_pos = *end;
                break;
            }
        }
        
        // Move cursor to new position
        let moves = target_pos.saturating_sub(cursor_pos);
        for _ in 0..moves {
            self.input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())));
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Command input - single line
                Constraint::Min(0),    // Results
                Constraint::Length(3), // Status bar
            ].as_ref())
            .split(f.area());

        // Update horizontal scroll based on actual terminal width
        self.update_horizontal_scroll(chunks[0].width);

        // Command input area
        let input_title = match self.mode {
            AppMode::Command => "SQL Query".to_string(),
            AppMode::Results => "SQL Query (Results Mode - Press ↑ to edit)".to_string(),
            AppMode::Search => "Search Pattern".to_string(),
            AppMode::Filter => "Filter Pattern".to_string(), 
            AppMode::Help => "Help".to_string(),
            AppMode::History => format!("History Search: '{}' (Esc to cancel)", self.history_state.search_query),
            AppMode::Debug => "Parser Debug (F5)".to_string(),
        };
        
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(input_title);

        let input_text = match self.mode {
            AppMode::Search => &self.search_state.pattern,
            AppMode::Filter => &self.filter_state.pattern,
            AppMode::History => &self.history_state.search_query,
            _ => self.input.value(),
        };

        let input_paragraph = match self.mode {
            AppMode::Command => {
                // Use syntax highlighting for SQL command input with horizontal scrolling
                let highlighted_line = self.sql_highlighter.simple_sql_highlight(input_text);
                Paragraph::new(Text::from(vec![highlighted_line]))
                    .block(input_block)
                    .scroll((0, self.get_horizontal_scroll_offset()))
            },
            _ => {
                // Plain text for other modes
                Paragraph::new(input_text)
                    .block(input_block)
                    .style(match self.mode {
                        AppMode::Results => Style::default().fg(Color::DarkGray),
                        AppMode::Search => Style::default().fg(Color::Yellow),
                        AppMode::Filter => Style::default().fg(Color::Cyan),
                        AppMode::Help => Style::default().fg(Color::DarkGray),
                        AppMode::History => Style::default().fg(Color::Magenta),
                        AppMode::Debug => Style::default().fg(Color::Yellow),
                        _ => Style::default(),
                    })
                    .scroll((0, self.get_horizontal_scroll_offset()))
            }
        };

        f.render_widget(input_paragraph, chunks[0]);

        // Set cursor position for input modes
        match self.mode {
            AppMode::Command => {
                // Calculate cursor position with horizontal scrolling
                let inner_width = chunks[0].width.saturating_sub(2) as usize;
                let cursor_pos = self.input.visual_cursor();
                let scroll_offset = self.get_horizontal_scroll_offset() as usize;
                
                // Calculate visible cursor position
                if cursor_pos >= scroll_offset && cursor_pos < scroll_offset + inner_width {
                    let visible_pos = cursor_pos - scroll_offset;
                    f.set_cursor_position((
                        chunks[0].x + visible_pos as u16 + 1,
                        chunks[0].y + 1
                    ));
                }
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
            AppMode::History => {
                f.set_cursor_position((
                    chunks[0].x + self.history_state.search_query.len() as u16 + 1,
                    chunks[0].y + 1
                ));
            },
            _ => {}
        }

        // Results area - render based on mode to reduce complexity
        match (&self.mode, self.show_help) {
            (_, true) => self.render_help(f, chunks[1]),
            (AppMode::History, false) => self.render_history(f, chunks[1]),
            (AppMode::Debug, false) => self.render_debug(f, chunks[1]),
            (_, false) if self.results.is_some() => {
                self.render_table(f, chunks[1], self.results.as_ref().unwrap());
            },
            _ => {
                // Simple placeholder - reduced text to improve rendering speed
                let placeholder = Paragraph::new("Enter SQL query and press Enter\n\nTip: Use Tab for completion, Ctrl+R for history")
                    .block(Block::default().borders(Borders::ALL).title("Results"))
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(placeholder, chunks[1]);
            }
        }

        // Status bar
        let status_style = match self.mode {
            AppMode::Command => Style::default().fg(Color::Green),
            AppMode::Results => Style::default().fg(Color::Blue),
            AppMode::Search => Style::default().fg(Color::Yellow),
            AppMode::Filter => Style::default().fg(Color::Cyan),
            AppMode::Help => Style::default().fg(Color::Magenta),
            AppMode::History => Style::default().fg(Color::Magenta),
            AppMode::Debug => Style::default().fg(Color::Yellow),
        };

        let mode_indicator = match self.mode {
            AppMode::Command => "CMD",
            AppMode::Results => "NAV",
            AppMode::Search => "SEARCH",
            AppMode::Filter => "FILTER",
            AppMode::Help => "HELP",
            AppMode::History => "HISTORY",
            AppMode::Debug => "DEBUG",
        };

        // Add parser debug info and token position for technical users
        let parser_debug = if self.mode == AppMode::Command {
            let cursor_pos = self.input.cursor();
            let query = self.input.value();
            let hybrid_result = self.hybrid_parser.get_completions(query, cursor_pos);
            let (token_pos, total_tokens) = self.get_cursor_token_position();
            format!(" | Token: {}/{} | {}: {} | Suggestions: {} | Complexity: {}", 
                token_pos,
                total_tokens,
                hybrid_result.parser_used,
                hybrid_result.context,
                if hybrid_result.suggestions.is_empty() { 
                    "none".to_string() 
                } else { 
                    hybrid_result.suggestions.len().to_string() 
                },
                hybrid_result.query_complexity)
        } else {
            String::new()
        };

        // Limit status message length to reduce rendering overhead
        let truncated_status = if self.status_message.len() > 40 {
            format!("{}...", &self.status_message[..37])
        } else {
            self.status_message.clone()
        };
        let status_text = format!("[{}] {}{} | F1:Help q:Quit", mode_indicator, truncated_status, parser_debug);
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

        // Calculate visible columns for virtual scrolling
        let terminal_width = area.width as usize;
        let available_width = terminal_width.saturating_sub(4); // Account for borders and padding
        let avg_col_width = 15; // Assume average column width
        let max_visible_cols = (available_width / avg_col_width).max(1).min(headers.len());
        
        // Calculate column viewport based on current_column
        let viewport_start = if self.current_column < max_visible_cols / 2 {
            0
        } else if self.current_column + max_visible_cols / 2 >= headers.len() {
            headers.len().saturating_sub(max_visible_cols)
        } else {
            self.current_column.saturating_sub(max_visible_cols / 2)
        };
        let viewport_end = (viewport_start + max_visible_cols).min(headers.len());
        
        // Only work with visible headers
        let visible_headers: Vec<&str> = headers[viewport_start..viewport_end].iter().copied().collect();
        
        // Prepare table data (only visible columns)
        let data_to_display = if let Some(filtered) = &self.filtered_data {
            // Apply column viewport to filtered data
            filtered.iter().map(|row| {
                row[viewport_start..viewport_end].to_vec()
            }).collect()
        } else {
            // Convert JSON data to string matrix (only visible columns)
            results.data.iter().map(|item| {
                if let Some(obj) = item.as_object() {
                    visible_headers.iter().map(|&header| {
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

        // Create header row with sort indicators and column selection
        let header_cells: Vec<Cell> = visible_headers.iter().enumerate().map(|(visible_i, &header)| {
            let actual_col_index = viewport_start + visible_i;
            let sort_indicator = if let Some(col) = self.sort_state.column {
                if col == actual_col_index {
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
            
            let column_indicator = if actual_col_index == self.current_column { " [*]" } else { "" };
            
            let header_text = format!("{}{}{}", header, sort_indicator, column_indicator);
            let mut style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
            
            // Highlight the current column
            if actual_col_index == self.current_column {
                style = style.bg(Color::DarkGray);
            }
            
            Cell::from(header_text).style(style)
        }).collect();

        // Calculate visible rows for virtual scrolling
        let terminal_height = area.height as usize;
        let available_height = terminal_height.saturating_sub(4); // Account for header, borders, etc.
        let max_visible_rows = available_height.saturating_sub(1).max(10); // Reserve space for header
        
        let selected_row = self.table_state.selected().unwrap_or(0);
        let total_rows = data_to_display.len();
        
        // Calculate row viewport based on selected row
        let row_viewport_start = if total_rows <= max_visible_rows {
            0 // Show all rows if they fit
        } else if selected_row < max_visible_rows / 2 {
            0 // Near the top
        } else if selected_row + max_visible_rows / 2 >= total_rows {
            total_rows.saturating_sub(max_visible_rows) // Near the bottom
        } else {
            selected_row.saturating_sub(max_visible_rows / 2) // Center the selection
        };
        let row_viewport_end = (row_viewport_start + max_visible_rows).min(total_rows);
        
        // Only render visible rows
        let visible_data = &data_to_display[row_viewport_start..row_viewport_end];
        
        // Create data rows (only visible rows and columns)
        let rows: Vec<Row> = visible_data.iter().enumerate().map(|(visible_row_idx, row)| {
            let actual_row_idx = row_viewport_start + visible_row_idx;
            let cells: Vec<Cell> = row.iter().enumerate().map(|(visible_col_idx, cell)| {
                let actual_col_idx = viewport_start + visible_col_idx;
                let mut style = Style::default();
                
                // Highlight current column
                if actual_col_idx == self.current_column {
                    style = style.bg(Color::DarkGray);
                }
                
                // Highlight search matches (override column highlight)
                if let Some((match_row, match_col)) = self.search_state.current_match {
                    if actual_row_idx == match_row && actual_col_idx == match_col {
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

        // Calculate column constraints (only for visible columns)
        let constraints: Vec<Constraint> = (0..visible_headers.len())
            .map(|_| Constraint::Min(10))
            .collect();

        let table = Table::new(rows, constraints)
            .header(Row::new(header_cells).height(1))
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!("Results ({} rows) - Columns {}-{} of {} | Use h/l to scroll", 
                    data_to_display.len(), 
                    viewport_start + 1, 
                    viewport_end, 
                    headers.len())))
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
            Line::from("  Ctrl+R   - Search command history"),
            Line::from("  Ctrl+A   - Jump to beginning of line"),
            Line::from("  Ctrl+E   - Jump to end of line"),
            Line::from("  Ctrl+←/Alt+B - Move backward one word"),
            Line::from("  Ctrl+→/Alt+F - Move forward one word"),
            Line::from("  ↓        - Enter results mode"),
            Line::from("  F1/?     - Toggle help"),
            Line::from("  Ctrl+C/D - Exit"),
            Line::from(""),
            Line::from("Results Navigation Mode:"),
            Line::from("  j/↓      - Next row"),
            Line::from("  k/↑      - Previous row"), 
            Line::from("  h/←      - Move to previous column"),
            Line::from("  l/→      - Move to next column"),
            Line::from("  g        - First row"),
            Line::from("  G        - Last row"),
            Line::from("  Ctrl+F   - Page down"),
            Line::from("  Ctrl+B   - Page up"),
            Line::from("  /        - Search"),
            Line::from("  n        - Next match"),
            Line::from("  N        - Previous match"),
            Line::from("  F        - Filter rows"),
            Line::from("  s        - Sort by current column"),
            Line::from("  1-9      - Sort by column number (1=first)"),
            Line::from("  Ctrl+S   - Export to CSV"),
            Line::from("  ↑/Esc    - Back to command mode"),
            Line::from("  q        - Quit"),
            Line::from(""),
            Line::from("History Search Mode:"),
            Line::from("  j/k/↓/↑  - Navigate history"),
            Line::from("  Enter    - Select command"),
            Line::from("  Esc      - Cancel"),
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

    fn render_debug(&self, f: &mut Frame, area: Rect) {
        let debug_lines: Vec<Line> = self.debug_text
            .lines()
            .map(|line| Line::from(line.to_string()))
            .collect();
        
        let total_lines = debug_lines.len();
        let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
        
        // Calculate visible range based on scroll
        let start = self.debug_scroll as usize;
        let end = (start + visible_height).min(total_lines);
        
        let visible_lines: Vec<Line> = if start < total_lines {
            debug_lines[start..end].to_vec()
        } else {
            vec![]
        };
        
        let debug_text = Text::from(visible_lines);
        
        let debug_paragraph = Paragraph::new(debug_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!("Parser Debug Info - Lines {}-{} of {} (↑↓ to scroll, Enter/Esc to close)", 
                    start + 1, end, total_lines))
                .border_style(Style::default().fg(Color::Yellow)))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        
        f.render_widget(debug_paragraph, area);
    }

    fn render_history(&self, f: &mut Frame, area: Rect) {
        if self.history_state.matches.is_empty() {
            let no_history = if self.history_state.search_query.is_empty() {
                "No command history found.\nExecute some queries to build history."
            } else {
                "No matches found for your search.\nTry a different search term."
            };
            
            let placeholder = Paragraph::new(no_history)
                .block(Block::default().borders(Borders::ALL).title("Command History"))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(placeholder, area);
            return;
        }

        // Split the area to show selected command details
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),    // History list
                Constraint::Length(3), // Selected command preview
            ])
            .split(area);

        self.render_history_list(f, chunks[0]);
        self.render_selected_command_preview(f, chunks[1]);
    }

    fn render_history_list(&self, f: &mut Frame, area: Rect) {
        // Create more compact history list - just show essential info
        let history_items: Vec<Line> = self.history_state.matches
            .iter()
            .enumerate()
            .map(|(i, history_match)| {
                let entry = &history_match.entry;
                let is_selected = i == self.history_state.selected_index;
                
                let success_indicator = if entry.success { "✓" } else { "✗" };
                let time_ago = {
                    let elapsed = chrono::Utc::now() - entry.timestamp;
                    if elapsed.num_days() > 0 {
                        format!("{}d", elapsed.num_days())
                    } else if elapsed.num_hours() > 0 {
                        format!("{}h", elapsed.num_hours())
                    } else if elapsed.num_minutes() > 0 {
                        format!("{}m", elapsed.num_minutes())
                    } else {
                        "now".to_string()
                    }
                };

                // Use more space for the command, less for metadata
                let terminal_width = area.width as usize;
                let metadata_space = 15; // Reduced metadata: " ✓ 2x 1h"
                let available_for_command = terminal_width.saturating_sub(metadata_space).max(50);
                
                let command_text = if entry.command.len() > available_for_command {
                    format!("{}…", &entry.command[..available_for_command.saturating_sub(1)])
                } else {
                    entry.command.clone()
                };

                let line_text = format!(
                    "{} {} {} {}x {}",
                    if is_selected { "►" } else { " " },
                    command_text,
                    success_indicator,
                    entry.execution_count,
                    time_ago
                );

                let mut style = Style::default();
                if is_selected {
                    style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
                }
                if !entry.success {
                    style = style.fg(Color::Red);
                }

                // Highlight matching characters for fuzzy search
                if !history_match.indices.is_empty() && is_selected {
                    style = style.fg(Color::Yellow);
                }

                Line::from(line_text).style(style)
            })
            .collect();

        let history_paragraph = Paragraph::new(history_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!("History ({} matches) - j/k to navigate, Enter to select", self.history_state.matches.len())))
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(history_paragraph, area);
    }

    fn render_selected_command_preview(&self, f: &mut Frame, area: Rect) {
        if let Some(selected_match) = self.history_state.matches.get(self.history_state.selected_index) {
            let entry = &selected_match.entry;
            
            // Show the full command with syntax highlighting
            let full_command = &entry.command;
            let highlighted_command = self.sql_highlighter.simple_sql_highlight(full_command);
            
            let preview_text = Text::from(vec![highlighted_command]);
            
            let duration_text = entry.duration_ms
                .map(|d| format!("{}ms", d))
                .unwrap_or_else(|| "?ms".to_string());
            
            let success_text = if entry.success { "✓ Success" } else { "✗ Failed" };
            
            let preview = Paragraph::new(preview_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Preview: {} | {} | Used {}x", success_text, duration_text, entry.execution_count)))
                .wrap(ratatui::widgets::Wrap { trim: true });
            
            f.render_widget(preview, area);
        } else {
            let empty_preview = Paragraph::new("No command selected")
                .block(Block::default().borders(Borders::ALL).title("Preview"))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(empty_preview, area);
        }
    }
}

pub fn run_enhanced_tui(api_url: &str) -> Result<()> {
    let app = EnhancedTuiApp::new(api_url);
    app.run()
}