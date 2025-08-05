use crate::api_client::{ApiClient, QueryResponse};
use crate::parser::SqlParser;
use crate::hybrid_parser::HybridParser;
use crate::history::{CommandHistory, HistoryMatch};
use crate::sql_highlighter::SqlHighlighter;
use sql_cli::cache::QueryCache;
use sql_cli::csv_datasource::CsvApiClient;
use sql_cli::where_parser::WhereParser;
use sql_cli::where_ast::format_where_ast;
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
use tui_textarea::{TextArea, CursorMove};

#[derive(Clone, PartialEq)]
enum AppMode {
    Command,
    Results,
    Search,
    Filter,
    Help,
    History,
    Debug,
    PrettyQuery,
    CacheList,
}

#[derive(Clone, PartialEq)]
enum EditMode {
    SingleLine,
    MultiLine,
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

pub struct EnhancedTuiApp {
    api_client: ApiClient,
    input: Input,
    textarea: TextArea<'static>,
    edit_mode: EditMode,
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
    
    // CSV mode
    csv_client: Option<CsvApiClient>,
    csv_mode: bool,
    csv_table_name: String,
    
    // Cache
    query_cache: Option<QueryCache>,
    cache_mode: bool,
    cached_data: Option<Vec<serde_json::Value>>,
    
    // Undo/redo and kill ring
    undo_stack: Vec<(String, usize)>, // (text, cursor_pos)
    redo_stack: Vec<(String, usize)>,
    kill_ring: String,
    
    // Viewport tracking
    last_visible_rows: usize, // Track the last calculated viewport height
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
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        
        Self {
            api_client: ApiClient::new(api_url),
            input: Input::default(),
            textarea,
            edit_mode: EditMode::SingleLine,
            mode: AppMode::Command,
            results: None,
            table_state: TableState::default(),
            show_help: false,
            status_message: "Ready - Type SQL query and press Enter (F3 to toggle multi-line mode)".to_string(),
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
            csv_client: None,
            csv_mode: false,
            csv_table_name: String::new(),
            query_cache: QueryCache::new().ok(),
            cache_mode: false,
            cached_data: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            kill_ring: String::new(),
            last_visible_rows: 30, // Default estimate
        }
    }
    
    pub fn new_with_csv(csv_path: &str) -> Result<Self> {
        let mut csv_client = CsvApiClient::new();
        let table_name = std::path::Path::new(csv_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data")
            .to_string();
        
        csv_client.load_csv(csv_path, &table_name)?;
        
        // Get schema from CSV
        let schema = csv_client.get_schema()
            .ok_or_else(|| anyhow::anyhow!("Failed to get CSV schema"))?;
        
        let mut app = Self::new(""); // Empty API URL for CSV mode
        app.csv_client = Some(csv_client);
        app.csv_mode = true;
        app.csv_table_name = table_name.clone();
        
        // Update parser with CSV columns
        if let Some(columns) = schema.get(&table_name) {
            // Update the parser with CSV columns
            app.hybrid_parser.update_single_table(table_name.clone(), columns.clone());
            app.status_message = format!("CSV loaded: table '{}' with {} columns. Use: SELECT * FROM {}", 
                table_name, columns.len(), table_name);
        }
        
        Ok(app)
    }
    
    pub fn new_with_json(json_path: &str) -> Result<Self> {
        let mut csv_client = CsvApiClient::new();
        let table_name = std::path::Path::new(json_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data")
            .to_string();
        
        csv_client.load_json(json_path, &table_name)?;
        
        // Get schema from JSON data
        let schema = csv_client.get_schema()
            .ok_or_else(|| anyhow::anyhow!("Failed to get JSON schema"))?;
        
        let mut app = Self::new(""); // Empty API URL for JSON mode
        app.csv_client = Some(csv_client);
        app.csv_mode = true; // Reuse CSV mode since the data structure is the same
        app.csv_table_name = table_name.clone();
        
        // Update parser with JSON columns
        if let Some(columns) = schema.get(&table_name) {
            app.hybrid_parser.update_single_table(table_name.clone(), columns.clone());
            app.status_message = format!("JSON loaded: table '{}' with {} columns. Use: SELECT * FROM {}", 
                table_name, columns.len(), table_name);
        }
        
        Ok(app)
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
            // Use blocking read for better performance - only process when there's an actual event
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
                            AppMode::PrettyQuery => self.handle_pretty_query_input(key)?,
                            AppMode::CacheList => self.handle_cache_list_input(key)?,
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
            KeyCode::F(3) => {
                // Toggle between single-line and multi-line mode
                match self.edit_mode {
                    EditMode::SingleLine => {
                        self.edit_mode = EditMode::MultiLine;
                        let current_text = self.input.value().to_string();
                        
                        // Pretty format the query for multi-line editing
                        let formatted_lines = if !current_text.trim().is_empty() {
                            crate::recursive_parser::format_sql_pretty_compact(&current_text, 5) // 5 columns per line for compact multi-line
                        } else {
                            vec![current_text]
                        };
                        
                        self.textarea = TextArea::from(formatted_lines);
                        self.textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
                        // Move cursor to the beginning
                        self.textarea.move_cursor(CursorMove::Top);
                        self.textarea.move_cursor(CursorMove::Head);
                        self.status_message = "Multi-line mode (F3 to toggle, Tab for completion, Ctrl+Enter to execute)".to_string();
                    },
                    EditMode::MultiLine => {
                        self.edit_mode = EditMode::SingleLine;
                        // Join lines with single space to create compact query
                        let text = self.textarea.lines()
                            .iter()
                            .map(|line| line.trim())
                            .filter(|line| !line.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        self.input = tui_input::Input::new(text);
                        self.status_message = "Single-line mode enabled (F3 to toggle multi-line)".to_string();
                    }
                }
            },
            KeyCode::F(7) => {
                // F7 - Toggle cache mode or show cache list
                if self.cache_mode {
                    self.mode = AppMode::CacheList;
                } else {
                    self.mode = AppMode::CacheList;
                }
            },
            KeyCode::Enter => {
                let query = match self.edit_mode {
                    EditMode::SingleLine => self.input.value().trim().to_string(),
                    EditMode::MultiLine => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // Ctrl+Enter executes the query in multi-line mode
                            self.textarea.lines().join("\n").trim().to_string()
                        } else {
                            // Regular Enter adds a new line
                            self.textarea.input(key);
                            return Ok(false);
                        }
                    }
                };
                
                if !query.is_empty() {
                    // Check for cache commands
                    if query.starts_with(":cache ") {
                        self.handle_cache_command(&query)?;
                    } else {
                        self.status_message = format!("Processing query: '{}'", query);
                        self.execute_query(&query)?;
                    }
                } else {
                    self.status_message = "Empty query - please enter a SQL command".to_string();
                }
            },
            KeyCode::Tab => {
                // Tab completion works in both modes
                match self.edit_mode {
                    EditMode::SingleLine => self.apply_completion(),
                    EditMode::MultiLine => {
                        // In vim normal mode, Tab should also trigger completion
                        self.apply_completion_multiline();
                    }
                }
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
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Delete word backward (like bash/zsh)
                self.delete_word_backward();
            },
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Delete word forward (like bash/zsh)
                self.delete_word_forward();
            },
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Kill line - delete from cursor to end of line
                self.kill_line();
            },
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Kill line backward - delete from cursor to beginning of line
                self.kill_line_backward();
            },
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Undo
                self.undo();
            },
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Yank - paste from kill ring
                self.yank();
            },
            KeyCode::Char('[') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Jump to previous SQL token
                self.jump_to_prev_token();
            },
            KeyCode::Char(']') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Jump to next SQL token
                self.jump_to_next_token();
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
            KeyCode::Down if self.results.is_some() && self.edit_mode == EditMode::SingleLine => {
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
                
                // Add dataset information
                let dataset_info = if self.csv_mode {
                    if let Some(ref csv_client) = self.csv_client {
                        if let Some(schema) = csv_client.get_schema() {
                            let (table_name, columns) = schema.iter().next()
                                .map(|(t, c)| (t.as_str(), c.clone()))
                                .unwrap_or(("unknown", vec![]));
                            format!(
                                "\n========== DATASET INFO ==========\n\
                                Mode: CSV\n\
                                Table Name: {}\n\
                                Columns ({}): {}\n",
                                table_name,
                                columns.len(),
                                columns.join(", ")
                            )
                        } else {
                            "\n========== DATASET INFO ==========\nMode: CSV\nNo schema available\n".to_string()
                        }
                    } else {
                        "\n========== DATASET INFO ==========\nMode: CSV\nNo CSV client initialized\n".to_string()
                    }
                } else {
                    format!(
                        "\n========== DATASET INFO ==========\n\
                        Mode: API ({})\n\
                        Table: trade_deal\n\
                        Default Columns: {}\n",
                        self.api_client.base_url,
                        "id, platformOrderId, tradeDate, executionSide, quantity, price, counterparty, ..."
                    )
                };
                debug_info.push_str(&dataset_info);
                
                // Add current data statistics
                let data_stats = format!(
                    "\n========== CURRENT DATA ==========\n\
                    Total Rows Loaded: {}\n\
                    Filtered Rows: {}\n\
                    Current Column: {}\n\
                    Sort State: {}\n",
                    self.results.as_ref().map(|r| r.data.len()).unwrap_or(0),
                    self.filtered_data.as_ref().map(|d| d.len()).unwrap_or(0),
                    self.current_column,
                    match &self.sort_state {
                        SortState { column: Some(col), order } => 
                            format!("Column {} - {}", col, match order {
                                SortOrder::Ascending => "Ascending",
                                SortOrder::Descending => "Descending",
                                SortOrder::None => "None"
                            }),
                        _ => "None".to_string()
                    }
                );
                debug_info.push_str(&data_stats);
                
                // Add WHERE clause AST if query contains WHERE
                if query.to_lowercase().contains(" where ") {
                    let where_ast_info = match self.parse_where_clause_ast(query) {
                        Ok(ast_str) => ast_str,
                        Err(e) => format!("\n========== WHERE CLAUSE AST ==========\nError parsing WHERE clause: {}\n", e)
                    };
                    debug_info.push_str(&where_ast_info);
                }
                
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
            KeyCode::F(6) => {
                // Pretty print query view
                let query = self.input.value();
                if !query.trim().is_empty() {
                    self.debug_text = format!("Pretty SQL Query\n{}\n\n{}", "=".repeat(50), 
                        crate::recursive_parser::format_sql_pretty_compact(query, 5).join("\n"));
                    self.debug_scroll = 0;
                    self.mode = AppMode::PrettyQuery;
                    self.status_message = "Pretty query view (press Esc or q to return)".to_string();
                } else {
                    self.status_message = "No query to format".to_string();
                }
            },
            _ => {
                match self.edit_mode {
                    EditMode::SingleLine => {
                        self.input.handle_event(&Event::Key(key));
                        // Clear completion state when typing other characters
                        self.completion_state.suggestions.clear();
                        self.completion_state.current_index = 0;
                        self.handle_completion();
                    },
                    EditMode::MultiLine => {
                        // Pass all keys to textarea
                        self.textarea.input(key);
                        // Clear completion state when typing other characters
                        self.completion_state.suggestions.clear();
                        self.completion_state.current_index = 0;
                        self.handle_completion_multiline();
                    }
                }
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
            KeyCode::Char('s') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.sort_by_column(self.current_column);
            },
            // Export to CSV
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.export_to_csv();
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

    fn handle_pretty_query_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
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
        
        let result = if self.cache_mode {
            // When in cache mode, use CSV client to query cached data
            if let Some(ref cached_data) = self.cached_data {
                let mut csv_client = CsvApiClient::new();
                csv_client.load_from_json(cached_data.clone(), "cached_data")?;
                
                csv_client.query_csv(query)
                    .map(|r| QueryResponse {
                        data: r.data,
                        count: r.count,
                        query: crate::api_client::QueryInfo {
                            select: r.query.select,
                            where_clause: r.query.where_clause,
                            order_by: r.query.order_by,
                        }
                    })
            } else {
                Err(anyhow::anyhow!("No cached data loaded"))
            }
        } else if self.csv_mode {
            if let Some(ref csv_client) = self.csv_client {
                // Convert CSV result to match the expected type
                csv_client.query_csv(query)
                    .map(|r| QueryResponse {
                        data: r.data,
                        count: r.count,
                        query: crate::api_client::QueryInfo {
                            select: r.query.select,
                            where_clause: r.query.where_clause,
                            order_by: r.query.order_by,
                        }
                    })
            } else {
                Err(anyhow::anyhow!("CSV client not initialized"))
            }
        } else {
            self.api_client.query_trades(query)
                .map_err(|e| anyhow::anyhow!("{}", e))
        };
        
        match result {
            Ok(response) => {
                let duration = start_time.elapsed();
                let _ = self.command_history.add_entry(
                    query.to_string(), 
                    true, 
                    Some(duration.as_millis() as u64)
                );
                
                // Add debug info about results
                let row_count = response.data.len();
                self.results = Some(response);
                self.calculate_optimal_column_widths();
                self.reset_table_state();
                
                if row_count == 0 {
                    self.status_message = format!("Query executed successfully but returned 0 rows ({}ms)", duration.as_millis());
                } else {
                    self.status_message = format!("Query executed successfully - {} rows returned ({}ms) - Use ↓ or j/k to navigate", row_count, duration.as_millis());
                }
                
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
    
    fn parse_where_clause_ast(&self, query: &str) -> Result<String> {
        let query_lower = query.to_lowercase();
        if let Some(where_pos) = query_lower.find(" where ") {
            let where_clause = &query[where_pos + 7..]; // Skip " where "
            
            match WhereParser::parse(where_clause) {
                Ok(ast) => {
                    let tree = format_where_ast(&ast, 0);
                    Ok(format!(
                        "\n========== WHERE CLAUSE AST ==========\n\
                        Query: {}\n\
                        WHERE clause: {}\n\n\
                        AST Tree:\n{}\n\n\
                        Note: Parentheses in the query control operator precedence.\n\
                        The parser respects: OR < AND < NOT < comparisons\n\
                        Example: 'a = 1 OR b = 2 AND c = 3' parses as 'a = 1 OR (b = 2 AND c = 3)'\n\
                        Use parentheses to override: '(a = 1 OR b = 2) AND c = 3'\n",
                        query,
                        where_clause,
                        tree
                    ))
                }
                Err(e) => Err(anyhow::anyhow!("Failed to parse WHERE clause: {}", e))
            }
        } else {
            Ok("\n========== WHERE CLAUSE AST ==========\nNo WHERE clause found in query\n".to_string())
        }
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
            
            // Handle quoted identifiers - if both partial and suggestion start with quotes,
            // we need to avoid double quotes
            let suggestion_to_use = if partial.starts_with('"') && suggestion.starts_with('"') {
                // The partial already includes the opening quote, so use suggestion without its quote
                if suggestion.len() > 1 {
                    &suggestion[1..]
                } else {
                    suggestion
                }
            } else {
                suggestion
            };
            
            let new_query = format!("{}{}{}", before_partial, suggestion_to_use, after_cursor);
            
            // Update input and cursor position
            let cursor_pos = before_partial.len() + suggestion_to_use.len();
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

    fn apply_completion_multiline(&mut self) {
        let (cursor_row, cursor_col) = self.textarea.cursor();
        let lines = self.textarea.lines();
        let query = lines.join("\n");
        
        // Calculate cursor position in the full query string
        let mut cursor_pos = 0;
        for (i, line) in lines.iter().enumerate() {
            if i < cursor_row {
                cursor_pos += line.len() + 1; // +1 for newline
            } else if i == cursor_row {
                cursor_pos += cursor_col;
                break;
            }
        }
        
        // Check if this is a continuation of the same completion session
        let is_same_context = query == self.completion_state.last_query && 
                             cursor_pos == self.completion_state.last_cursor_pos;
        
        if !is_same_context {
            // New completion context - get fresh suggestions
            let hybrid_result = self.hybrid_parser.get_completions(&query, cursor_pos);
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
        let partial_word = self.extract_partial_word_at_cursor(&query, cursor_pos);
        
        if let Some(partial) = partial_word {
            // Replace the partial word with the suggestion
            let current_line = lines[cursor_row].clone();
            let line_before = &current_line[..cursor_col.saturating_sub(partial.len())];
            let line_after = &current_line[cursor_col..];
            
            // Handle quoted identifiers - if both partial and suggestion start with quotes,
            // we need to avoid double quotes
            let suggestion_to_use = if partial.starts_with('"') && suggestion.starts_with('"') {
                // The partial already includes the opening quote, so use suggestion without its quote
                if suggestion.len() > 1 {
                    &suggestion[1..]
                } else {
                    suggestion
                }
            } else {
                suggestion
            };
            
            let new_line = format!("{}{}{}", line_before, suggestion_to_use, line_after);
            
            // Update the line in textarea
            self.textarea.delete_line_by_head();
            self.textarea.insert_str(&new_line);
            
            // Move cursor to after the completion
            let new_col = line_before.len() + suggestion_to_use.len();
            for _ in 0..new_col {
                self.textarea.move_cursor(CursorMove::Forward);
            }
            
            // Update completion state
            let new_query = self.textarea.lines().join("\n");
            self.completion_state.last_query = new_query;
            self.completion_state.last_cursor_pos = cursor_pos - partial.len() + suggestion_to_use.len();
            
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
            self.textarea.insert_str(suggestion);
            
            // Update completion state
            let new_query = self.textarea.lines().join("\n");
            self.completion_state.last_query = new_query;
            self.completion_state.last_cursor_pos = cursor_pos + suggestion.len();
            
            self.status_message = format!("Inserted: {}", suggestion);
        }
    }
    
    fn handle_completion_multiline(&mut self) {
        // Similar to handle_completion but for multiline mode
        let (cursor_row, cursor_col) = self.textarea.cursor();
        let lines = self.textarea.lines();
        let query = lines.join("\n");
        
        // Calculate cursor position in the full query string
        let mut cursor_pos = 0;
        for (i, line) in lines.iter().enumerate() {
            if i < cursor_row {
                cursor_pos += line.len() + 1; // +1 for newline
            } else if i == cursor_row {
                cursor_pos += cursor_col;
                break;
            }
        }
        
        // Update completions based on cursor position
        let hybrid_result = self.hybrid_parser.get_completions(&query, cursor_pos);
        self.completion_state.suggestions = hybrid_result.suggestions;
        self.completion_state.current_index = 0;
        self.completion_state.last_query = query;
        self.completion_state.last_cursor_pos = cursor_pos;
    }
    
    fn extract_partial_word_at_cursor(&self, query: &str, cursor_pos: usize) -> Option<String> {
        if cursor_pos == 0 || cursor_pos > query.len() {
            return None;
        }

        let chars: Vec<char> = query.chars().collect();
        let mut start = cursor_pos;
        let end = cursor_pos;

        // Check if we might be in a quoted identifier
        let mut in_quote = false;
        
        // Find start of word (go backward)
        while start > 0 {
            let prev_char = chars[start - 1];
            if prev_char == '"' {
                // Found a quote, include it and stop
                start -= 1;
                in_quote = true;
                break;
            } else if prev_char.is_alphanumeric() || prev_char == '_' || (prev_char == ' ' && in_quote) {
                start -= 1;
            } else {
                break;
            }
        }

        // If we found a quote but are in a quoted identifier, 
        // we need to continue backwards to include the identifier content
        if in_quote && start > 0 {
            // We've already moved past the quote, now get the content before it
            // Actually, we want to include everything from the quote forward
            // The logic above is correct - we stop at the quote
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

    // Helper to get estimated visible rows based on terminal size
    fn get_visible_rows(&self) -> usize {
        // Try to get terminal size, or use stored default
        if let Ok((_, height)) = crossterm::terminal::size() {
            let terminal_height = height as usize;
            let available_height = terminal_height.saturating_sub(4); // Account for header, borders, etc.
            let max_visible_rows = available_height.saturating_sub(1).max(10); // Reserve space for header
            max_visible_rows
        } else {
            self.last_visible_rows // Fallback to stored value
        }
    }
    
    // Navigation functions
    fn next_row(&mut self) {
        if let Some(data) = self.get_current_data() {
            let total_rows = data.len();
            if total_rows == 0 { return; }
            
            // Update viewport size before navigation
            self.update_viewport_size();
            
            let current = self.table_state.selected().unwrap_or(0);
            if current >= total_rows - 1 { return; } // Already at bottom
            
            let new_position = current + 1;
            self.table_state.select(Some(new_position));
            
            // Update viewport if needed
            let visible_rows = self.last_visible_rows;
            
            // Check if cursor would be below the last visible row
            if new_position > self.scroll_offset.0 + visible_rows - 1 {
                // Cursor moved below viewport - scroll down by one
                self.scroll_offset.0 += 1;
            }
        }
    }

    fn previous_row(&mut self) {
        let current = self.table_state.selected().unwrap_or(0);
        if current == 0 { return; } // Already at top
        
        let new_position = current - 1;
        self.table_state.select(Some(new_position));
        
        // Update viewport if needed
        if new_position < self.scroll_offset.0 {
            // Cursor moved above viewport - scroll up
            self.scroll_offset.0 = new_position;
        }
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
        self.scroll_offset.0 = 0; // Reset viewport to top
    }

    fn update_viewport_size(&mut self) {
        // Update the stored viewport size based on current terminal size
        if let Ok((_, height)) = crossterm::terminal::size() {
            let terminal_height = height as usize;
            // Match the actual layout calculation:
            // - Input area: 3 rows (from input_height)
            // - Status bar: 3 rows
            // - Results area gets the rest
            let input_height = 3;
            let status_height = 3;
            let results_area_height = terminal_height.saturating_sub(input_height + status_height);
            
            // Now match EXACTLY what the render function does:
            // - 1 row for top border
            // - 1 row for header
            // - 1 row for bottom border
            self.last_visible_rows = results_area_height.saturating_sub(3).max(10);
        }
    }
    
    fn goto_last_row(&mut self) {
        if let Some(data) = self.get_current_data() {
            if !data.is_empty() {
                let last_row = data.len() - 1;
                self.table_state.select(Some(last_row));
                // Position viewport to show the last row at the bottom
                let visible_rows = self.last_visible_rows;
                self.scroll_offset.0 = last_row.saturating_sub(visible_rows - 1);
            }
        }
    }

    fn page_down(&mut self) {
        if let Some(data) = self.get_current_data() {
            let total_rows = data.len();
            if total_rows == 0 { return; }
            
            let visible_rows = self.last_visible_rows;
            let current = self.table_state.selected().unwrap_or(0);
            let new_position = (current + visible_rows).min(total_rows - 1);
            
            self.table_state.select(Some(new_position));
            
            // Scroll viewport down by a page
            self.scroll_offset.0 = (self.scroll_offset.0 + visible_rows)
                .min(total_rows.saturating_sub(visible_rows));
        }
    }

    fn page_up(&mut self) {
        let visible_rows = self.last_visible_rows;
        let current = self.table_state.selected().unwrap_or(0);
        let new_position = current.saturating_sub(visible_rows);
        
        self.table_state.select(Some(new_position));
        
        // Scroll viewport up by a page
        self.scroll_offset.0 = self.scroll_offset.0.saturating_sub(visible_rows);
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
                    
                    if let Some(obj) = item.as_object() {
                        for (_, value) in obj {
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
                }
                
                let filtered_count = filtered.len();
                self.filtered_data = Some(filtered);
                self.filter_state.regex = Some(regex);
                self.filter_state.active = true;
                
                // Reset table state but preserve filtered data
                self.table_state = TableState::default();
                self.scroll_offset = (0, 0);
                self.current_column = 0;
                
                // Clear search state but keep filter state
                self.search_state = SearchState {
                    pattern: String::new(),
                    current_match: None,
                    matches: Vec::new(),
                    match_index: 0,
                };
                
                self.status_message = format!("Filtered to {} rows", filtered_count);
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

        // Sort using original JSON values for proper type-aware comparison
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                    
                    if column_index < headers.len() {
                        let column_name = headers[column_index];
                        
                        // Create a vector of (original_json_row, row_index) pairs for sorting
                        let mut indexed_rows: Vec<(serde_json::Value, usize)> = results.data.iter()
                            .enumerate()
                            .map(|(i, row)| (row.clone(), i))
                            .collect();
                        
                        // Sort based on the original JSON values
                        indexed_rows.sort_by(|(row_a, _), (row_b, _)| {
                            let val_a = row_a.get(column_name);
                            let val_b = row_b.get(column_name);
                            
                            let cmp = match (val_a, val_b) {
                                (Some(serde_json::Value::Number(a)), Some(serde_json::Value::Number(b))) => {
                                    // Numeric comparison - this handles integers and floats properly
                                    let a_f64 = a.as_f64().unwrap_or(0.0);
                                    let b_f64 = b.as_f64().unwrap_or(0.0);
                                    a_f64.partial_cmp(&b_f64).unwrap_or(Ordering::Equal)
                                },
                                (Some(serde_json::Value::String(a)), Some(serde_json::Value::String(b))) => {
                                    // String comparison
                                    a.cmp(b)
                                },
                                (Some(serde_json::Value::Bool(a)), Some(serde_json::Value::Bool(b))) => {
                                    // Boolean comparison (false < true)
                                    a.cmp(b)
                                },
                                (Some(serde_json::Value::Null), Some(serde_json::Value::Null)) => {
                                    Ordering::Equal
                                },
                                (Some(serde_json::Value::Null), Some(_)) => {
                                    // NULL comes first
                                    Ordering::Less 
                                },
                                (Some(_), Some(serde_json::Value::Null)) => {
                                    // NULL comes first
                                    Ordering::Greater
                                },
                                (None, None) => Ordering::Equal,
                                (None, Some(_)) => Ordering::Less,
                                (Some(_), None) => Ordering::Greater,
                                // Mixed type comparison - fall back to string representation
                                (Some(a), Some(b)) => {
                                    let a_str = match a {
                                        serde_json::Value::String(s) => s.clone(),
                                        other => other.to_string(),
                                    };
                                    let b_str = match b {
                                        serde_json::Value::String(s) => s.clone(),
                                        other => other.to_string(),
                                    };
                                    a_str.cmp(&b_str)
                                }
                            };
                            
                            match new_order {
                                SortOrder::Ascending => cmp,
                                SortOrder::Descending => cmp.reverse(),
                                SortOrder::None => Ordering::Equal,
                            }
                        });
                        
                        // Rebuild the QueryResponse with sorted data
                        let sorted_data: Vec<serde_json::Value> = indexed_rows.into_iter()
                            .map(|(row, _)| row)
                            .collect();
                        
                        // Update both the results and clear filtered_data to force regeneration
                        let mut new_results = results.clone();
                        new_results.data = sorted_data;
                        self.results = Some(new_results);
                        self.filtered_data = None; // Force regeneration of string data
                    }
                }
            }
        } else if let Some(data) = self.get_current_data_mut() {
            // Fallback to string-based sorting if no JSON data available
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
        }

        self.sort_state = SortState { column: Some(column_index), order: new_order };
        
        // Reset table state but preserve current column position
        let current_column = self.current_column;
        self.reset_table_state();
        self.current_column = current_column;
        
        self.status_message = format!("Sorted by column {} ({}) - type-aware", 
            column_index + 1, 
            match new_order {
                SortOrder::Ascending => "ascending",
                SortOrder::Descending => "descending",
                SortOrder::None => "none",
            }
        );
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
        
        // Clear filter state to prevent old filtered data from persisting
        self.filter_state = FilterState {
            pattern: String::new(),
            regex: None,
            active: false,
        };
        
        // Clear search state
        self.search_state = SearchState {
            pattern: String::new(),
            current_match: None,
            matches: Vec::new(),
            match_index: 0,
        };
        
        // Clear filtered data
        self.filtered_data = None;
    }

    fn calculate_optimal_column_widths(&mut self) {
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                    let mut widths = Vec::new();
                    
                    for header in &headers {
                        // Start with header width
                        let mut max_width = header.len();
                        
                        // Check all data rows for this column
                        for row in &results.data {
                            if let Some(obj) = row.as_object() {
                                if let Some(value) = obj.get(*header) {
                                    let display_len = match value {
                                        serde_json::Value::String(s) => s.len(),
                                        serde_json::Value::Number(n) => n.to_string().len(),
                                        serde_json::Value::Bool(b) => b.to_string().len(),
                                        serde_json::Value::Null => 4, // "null".len()
                                        _ => value.to_string().len(),
                                    };
                                    max_width = max_width.max(display_len);
                                }
                            }
                        }
                        
                        // Add some padding and set reasonable limits
                        let optimal_width = (max_width + 2).max(4).min(50); // 4-50 char range with 2 char padding
                        widths.push(optimal_width as u16);
                    }
                    
                    self.column_widths = widths;
                }
            }
        }
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
    
    fn get_token_at_cursor(&self) -> Option<String> {
        let query = self.input.value();
        let cursor_pos = self.input.cursor();
        
        if query.is_empty() {
            return None;
        }
        
        // Use our lexer to tokenize the query
        use crate::recursive_parser::Lexer;
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all_with_positions();
        
        // Find the token at cursor position
        for (start, end, token) in &tokens {
            if cursor_pos >= *start && cursor_pos <= *end {
                // Format token nicely
                use crate::recursive_parser::Token;
                let token_str = match token {
                    Token::Select => "SELECT",
                    Token::From => "FROM", 
                    Token::Where => "WHERE",
                    Token::GroupBy => "GROUP BY",
                    Token::OrderBy => "ORDER BY",
                    Token::Having => "HAVING",
                    Token::And => "AND",
                    Token::Or => "OR",
                    Token::In => "IN",
                    Token::DateTime => "DateTime",
                    Token::Identifier(s) => s,
                    Token::QuotedIdentifier(s) => s,
                    Token::StringLiteral(s) => s,
                    Token::NumberLiteral(s) => s,
                    Token::Star => "*",
                    Token::Comma => ",",
                    Token::Dot => ".",
                    Token::LeftParen => "(",
                    Token::RightParen => ")",
                    Token::Equal => "=",
                    Token::GreaterThan => ">",
                    Token::LessThan => "<",
                    Token::GreaterThanOrEqual => ">=",
                    Token::LessThanOrEqual => "<=",
                    Token::NotEqual => "!=",
                    Token::Not => "NOT",
                    Token::Between => "BETWEEN",
                    Token::Like => "LIKE",
                    Token::Is => "IS",
                    Token::Null => "NULL",
                    Token::Eof => "EOF",
                };
                return Some(token_str.to_string());
            }
        }
        
        None
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

    fn delete_word_backward(&mut self) {
        let query = self.input.value();
        let cursor_pos = self.input.cursor();
        
        if cursor_pos == 0 {
            return;
        }
        
        // Save to undo stack before modifying
        self.undo_stack.push((query.to_string(), cursor_pos));
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        
        // Find the start of the previous word
        let chars: Vec<char> = query.chars().collect();
        let mut word_start = cursor_pos;
        
        // Skip any whitespace before cursor
        while word_start > 0 && chars[word_start - 1].is_whitespace() {
            word_start -= 1;
        }
        
        // Find the beginning of the word
        while word_start > 0 && !chars[word_start - 1].is_whitespace() && !is_sql_delimiter(chars[word_start - 1]) {
            word_start -= 1;
        }
        
        // If we only moved through whitespace, try to delete at least one word
        if word_start == cursor_pos && word_start > 0 {
            word_start -= 1;
            while word_start > 0 && !chars[word_start - 1].is_whitespace() && !is_sql_delimiter(chars[word_start - 1]) {
                word_start -= 1;
            }
        }
        
        // Delete from word_start to cursor_pos
        if word_start < cursor_pos {
            let before = &query[..word_start];
            let after = &query[cursor_pos..];
            let new_query = format!("{}{}", before, after);
            self.input = tui_input::Input::new(new_query).with_cursor(word_start);
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
    
    fn delete_word_forward(&mut self) {
        let query = self.input.value();
        let cursor_pos = self.input.cursor();
        let query_len = query.len();
        
        if cursor_pos >= query_len {
            return;
        }
        
        // Save to undo stack before modifying
        self.undo_stack.push((query.to_string(), cursor_pos));
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        
        // Find the end of the current/next word
        let chars: Vec<char> = query.chars().collect();
        let mut word_end = cursor_pos;
        
        // Skip any non-word characters first
        while word_end < chars.len() && !chars[word_end].is_alphanumeric() && chars[word_end] != '_' {
            word_end += 1;
        }
        
        // Then skip word characters
        while word_end < chars.len() && (chars[word_end].is_alphanumeric() || chars[word_end] == '_') {
            word_end += 1;
        }
        
        // Delete from cursor to word end
        if word_end > cursor_pos {
            let before = query.chars().take(cursor_pos).collect::<String>();
            let after = query.chars().skip(word_end).collect::<String>();
            let new_query = format!("{}{}", before, after);
            self.input = tui_input::Input::new(new_query).with_cursor(cursor_pos);
        }
    }
    
    fn kill_line(&mut self) {
        let query = self.input.value();
        let cursor_pos = self.input.cursor();
        
        if cursor_pos < query.len() {
            // Save to undo stack before modifying
            self.undo_stack.push((query.to_string(), cursor_pos));
            if self.undo_stack.len() > 100 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            
            // Save to kill ring before deleting
            self.kill_ring = query.chars().skip(cursor_pos).collect::<String>();
            let new_query = query.chars().take(cursor_pos).collect::<String>();
            self.input = tui_input::Input::new(new_query).with_cursor(cursor_pos);
        }
    }
    
    fn kill_line_backward(&mut self) {
        let query = self.input.value();
        let cursor_pos = self.input.cursor();
        
        if cursor_pos > 0 {
            // Save to undo stack before modifying
            self.undo_stack.push((query.to_string(), cursor_pos));
            if self.undo_stack.len() > 100 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            
            // Save to kill ring before deleting
            self.kill_ring = query.chars().take(cursor_pos).collect::<String>();
            let new_query = query.chars().skip(cursor_pos).collect::<String>();
            self.input = tui_input::Input::new(new_query).with_cursor(0);
        }
    }
    
    fn undo(&mut self) {
        // Simple undo - restore from undo stack
        if let Some(prev_state) = self.undo_stack.pop() {
            let current_state = (self.input.value().to_string(), self.input.cursor());
            self.redo_stack.push(current_state);
            self.input = tui_input::Input::new(prev_state.0).with_cursor(prev_state.1);
        }
    }
    
    fn yank(&mut self) {
        if !self.kill_ring.is_empty() {
            let query = self.input.value();
            let cursor_pos = self.input.cursor();
            
            // Save to undo stack before modifying
            self.undo_stack.push((query.to_string(), cursor_pos));
            if self.undo_stack.len() > 100 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            
            // Insert kill ring content at cursor
            let before = query.chars().take(cursor_pos).collect::<String>();
            let after = query.chars().skip(cursor_pos).collect::<String>();
            let new_query = format!("{}{}{}", before, self.kill_ring, after);
            let new_cursor = cursor_pos + self.kill_ring.len();
            self.input = tui_input::Input::new(new_query).with_cursor(new_cursor);
        }
    }
    
    fn jump_to_prev_token(&mut self) {
        let query = self.input.value();
        let cursor_pos = self.input.cursor();
        
        if cursor_pos == 0 {
            return;
        }
        
        use crate::recursive_parser::Lexer;
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all_with_positions();
        
        // Find current token position
        let mut in_token = false;
        let mut current_token_start = 0;
        for (start, end, _) in &tokens {
            if cursor_pos > *start && cursor_pos <= *end {
                in_token = true;
                current_token_start = *start;
                break;
            }
        }
        
        // Find the previous token start
        let mut target_pos = 0;
        
        if in_token && cursor_pos > current_token_start {
            // If we're in the middle of a token, go to its start
            target_pos = current_token_start;
        } else {
            // Otherwise, find the previous token
            for (start, _, _) in tokens.iter().rev() {
                if *start < cursor_pos {
                    target_pos = *start;
                    break;
                }
            }
        }
        
        // Move cursor
        if target_pos < cursor_pos {
            let moves = cursor_pos - target_pos;
            for _ in 0..moves {
                self.input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::empty())));
            }
        }
    }
    
    fn jump_to_next_token(&mut self) {
        let query = self.input.value();
        let cursor_pos = self.input.cursor();
        let query_len = query.len();
        
        if cursor_pos >= query_len {
            return;
        }
        
        use crate::recursive_parser::Lexer;
        let mut lexer = Lexer::new(query);
        let tokens = lexer.tokenize_all_with_positions();
        
        // Find the next token start after cursor
        let mut target_pos = query_len;
        let mut in_current_token = false;
        
        for (start, end, _) in &tokens {
            if cursor_pos >= *start && cursor_pos < *end {
                in_current_token = true;
            } else if in_current_token && *start >= *end {
                // Move to the start of the next token after the current one
                target_pos = *start;
                break;
            } else if *start > cursor_pos {
                target_pos = *start;
                break;
            }
        }
        
        // Move cursor
        let moves = target_pos.saturating_sub(cursor_pos);
        for _ in 0..moves {
            self.input.handle_event(&Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::empty())));
        }
    }

    /*
    fn handle_vim_mode(&mut self, key: KeyEvent) -> bool {
        // Returns true if the key was handled by vim mode
        match self.vim_state.mode {
            VimMode::Normal => {
                match key.code {
                    // Mode switching
                    KeyCode::Char('i') => {
                        self.vim_state.mode = VimMode::Insert;
                        self.update_vim_status();
                        true
                    }
                    KeyCode::Char('I') => {
                        self.vim_state.mode = VimMode::Insert;
                        self.textarea.move_cursor(CursorMove::Head);
                        self.update_vim_status();
                        true
                    }
                    KeyCode::Char('a') => {
                        self.vim_state.mode = VimMode::Insert;
                        self.textarea.move_cursor(CursorMove::Forward);
                        self.update_vim_status();
                        true
                    }
                    KeyCode::Char('A') => {
                        self.vim_state.mode = VimMode::Insert;
                        self.textarea.move_cursor(CursorMove::End);
                        self.update_vim_status();
                        true
                    }
                    KeyCode::Char('o') => {
                        self.vim_state.mode = VimMode::Insert;
                        self.textarea.move_cursor(CursorMove::End);
                        self.textarea.insert_newline();
                        self.update_vim_status();
                        true
                    }
                    KeyCode::Char('O') => {
                        self.vim_state.mode = VimMode::Insert;
                        self.textarea.move_cursor(CursorMove::Head);
                        self.textarea.insert_newline();
                        self.textarea.move_cursor(CursorMove::Up);
                        self.update_vim_status();
                        true
                    }
                    KeyCode::Char('v') => {
                        self.vim_state.mode = VimMode::Visual;
                        let cursor = self.textarea.cursor();
                        self.vim_state.visual_start = Some(cursor);
                        self.update_vim_status();
                        true
                    }
                    
                    // Movement
                    KeyCode::Char('h') | KeyCode::Left => {
                        self.textarea.move_cursor(CursorMove::Back);
                        true
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        self.textarea.move_cursor(CursorMove::Down);
                        true
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.textarea.move_cursor(CursorMove::Up);
                        true
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        self.textarea.move_cursor(CursorMove::Forward);
                        true
                    }
                    KeyCode::Char('0') => {
                        self.textarea.move_cursor(CursorMove::Head);
                        true
                    }
                    KeyCode::Char('$') => {
                        self.textarea.move_cursor(CursorMove::End);
                        true
                    }
                    KeyCode::Char('w') => {
                        self.textarea.move_cursor(CursorMove::WordForward);
                        true
                    }
                    KeyCode::Char('b') => {
                        self.textarea.move_cursor(CursorMove::WordBack);
                        true
                    }
                    KeyCode::Char('e') => {
                        // Move to end of word
                        self.textarea.move_cursor(CursorMove::WordForward);
                        self.textarea.move_cursor(CursorMove::Back);
                        true
                    }
                    KeyCode::Char('g') => {
                        // gg - go to first line (need to handle double-g)
                        self.textarea.move_cursor(CursorMove::Top);
                        true
                    }
                    KeyCode::Char('G') => {
                        self.textarea.move_cursor(CursorMove::Bottom);
                        true
                    }
                    
                    // Editing
                    KeyCode::Char('x') => {
                        self.textarea.delete_char();
                        true
                    }
                    KeyCode::Char('d') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            // Ctrl-d - half page down
                            for _ in 0..10 {
                                self.textarea.move_cursor(CursorMove::Down);
                            }
                        } else {
                            // dd - delete line
                            self.textarea.move_cursor(CursorMove::Head);
                            self.textarea.delete_line_by_end();
                            self.textarea.delete_newline();
                        }
                        true
                    }
                    KeyCode::Char('y') => {
                        // yy - yank line
                        let current_line = self.textarea.lines()[self.textarea.cursor().0].clone();
                        self.vim_state.yank_buffer = current_line;
                        self.status_message = "Line yanked".to_string();
                        true
                    }
                    KeyCode::Char('p') => {
                        // Paste after cursor
                        if !self.vim_state.yank_buffer.is_empty() {
                            self.textarea.move_cursor(CursorMove::End);
                            self.textarea.insert_newline();
                            self.textarea.insert_str(&self.vim_state.yank_buffer);
                        }
                        true
                    }
                    KeyCode::Char('P') => {
                        // Paste before cursor
                        if !self.vim_state.yank_buffer.is_empty() {
                            self.textarea.move_cursor(CursorMove::Head);
                            self.textarea.insert_str(&self.vim_state.yank_buffer);
                            self.textarea.insert_newline();
                            self.textarea.move_cursor(CursorMove::Up);
                        }
                        true
                    }
                    KeyCode::Char('u') => {
                        self.textarea.undo();
                        true
                    }
                    KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.textarea.redo();
                        true
                    }
                    
                    _ => false
                }
            }
            VimMode::Insert => {
                match key.code {
                    KeyCode::Esc => {
                        self.vim_state.mode = VimMode::Normal;
                        self.update_vim_status();
                        true
                    }
                    _ => false // Let textarea handle the input
                }
            }
            VimMode::Visual => {
                match key.code {
                    KeyCode::Esc => {
                        self.vim_state.mode = VimMode::Normal;
                        self.vim_state.visual_start = None;
                        self.update_vim_status();
                        true
                    }
                    KeyCode::Char('y') => {
                        // Yank selected text
                        if let Some(start) = self.vim_state.visual_start {
                            let end = self.textarea.cursor();
                            // Simple line-based yanking for now
                            let lines = self.textarea.lines();
                            let start_row = start.0.min(end.0);
                            let end_row = start.0.max(end.0);
                            let yanked: Vec<String> = lines[start_row..=end_row]
                                .iter()
                                .map(|s| s.to_string())
                                .collect();
                            self.vim_state.yank_buffer = yanked.join("\n");
                            self.status_message = format!("{} lines yanked", yanked.len());
                        }
                        self.vim_state.mode = VimMode::Normal;
                        self.vim_state.visual_start = None;
                        self.update_vim_status();
                        true
                    }
                    // Movement in visual mode
                    KeyCode::Char('h') | KeyCode::Left => {
                        self.textarea.move_cursor(CursorMove::Back);
                        true
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        self.textarea.move_cursor(CursorMove::Down);
                        true
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.textarea.move_cursor(CursorMove::Up);
                        true
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        self.textarea.move_cursor(CursorMove::Forward);
                        true
                    }
                    _ => false
                }
            }
        }
    }
    
    */
    
    /*
    fn update_vim_status(&mut self) {
        let mode_str = match self.vim_state.mode {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
            VimMode::Visual => "VISUAL",
        };
        
        // Update cursor style based on mode
        match self.vim_state.mode {
            VimMode::Normal => {
                self.textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            },
            VimMode::Insert => {
                self.textarea.set_cursor_style(Style::default().add_modifier(Modifier::UNDERLINED));
            },
            VimMode::Visual => {
                self.textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED).fg(Color::Yellow));
            },
        }
        
        // Get cursor position
        let (row, col) = self.textarea.cursor();
        self.status_message = format!("-- {} -- L{}:C{} (F3 single-line)", mode_str, row + 1, col + 1);
    }
    */
    
    fn ui(&mut self, f: &mut Frame) {
        // Dynamically adjust layout based on edit mode
        let input_height = match self.edit_mode {
            EditMode::SingleLine => 3,
            EditMode::MultiLine => {
                // Use 1/3 of terminal height or 10 lines, whichever is larger (max 20)
                let dynamic_height = f.area().height / 3;
                std::cmp::min(20, std::cmp::max(10, dynamic_height))
            }
        };
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(input_height), // Command input area
                Constraint::Min(0),              // Results
                Constraint::Length(3),           // Status bar
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
            AppMode::PrettyQuery => "Pretty Query View (F6)".to_string(),
            AppMode::CacheList => "Cache Management (F7)".to_string(),
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
                match self.edit_mode {
                    EditMode::SingleLine => {
                        // Use syntax highlighting for SQL command input with horizontal scrolling
                        let highlighted_line = self.sql_highlighter.simple_sql_highlight(input_text);
                        Paragraph::new(Text::from(vec![highlighted_line]))
                            .block(input_block)
                            .scroll((0, self.get_horizontal_scroll_offset()))
                    },
                    EditMode::MultiLine => {
                        // For multiline mode, we'll render the textarea widget instead
                        // This is a placeholder - actual textarea rendering happens below
                        Paragraph::new("")
                            .block(input_block)
                    }
                }
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
            AppMode::PrettyQuery => Style::default().fg(Color::Green),
                        AppMode::CacheList => Style::default().fg(Color::Cyan),
                        _ => Style::default(),
                    })
                    .scroll((0, self.get_horizontal_scroll_offset()))
            }
        };

        // Determine the actual results area based on edit mode
        let results_area = if self.mode == AppMode::Command && self.edit_mode == EditMode::MultiLine {
            // In multi-line mode, render textarea in the input area
            f.render_widget(&self.textarea, chunks[0]);
            
            // Use the full results area - no preview in multi-line mode anymore
            chunks[1]
        } else {
            // Single-line mode - render the input
            f.render_widget(input_paragraph, chunks[0]);
            // Use the full results area
            chunks[1]
        };

        // Set cursor position for input modes
        match self.mode {
            AppMode::Command => {
                match self.edit_mode {
                    EditMode::SingleLine => {
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
                    EditMode::MultiLine => {
                        // Cursor is handled by the textarea widget
                    }
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
            (_, true) => self.render_help(f, results_area),
            (AppMode::History, false) => self.render_history(f, results_area),
            (AppMode::Debug, false) => self.render_debug(f, results_area),
            (AppMode::PrettyQuery, false) => self.render_pretty_query(f, results_area),
            (AppMode::CacheList, false) => self.render_cache_list(f, results_area),
            (_, false) if self.results.is_some() => {
                self.render_table(f, results_area, self.results.as_ref().unwrap());
            },
            _ => {
                // Simple placeholder - reduced text to improve rendering speed
                let placeholder = Paragraph::new("Enter SQL query and press Enter\n\nTip: Use Tab for completion, Ctrl+R for history")
                    .block(Block::default().borders(Borders::ALL).title("Results"))
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(placeholder, results_area);
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
            AppMode::PrettyQuery => Style::default().fg(Color::Green),
            AppMode::CacheList => Style::default().fg(Color::Cyan),
        };

        let mode_indicator = match self.mode {
            AppMode::Command => "CMD",
            AppMode::Results => "NAV",
            AppMode::Search => "SEARCH",
            AppMode::Filter => "FILTER",
            AppMode::Help => "HELP",
            AppMode::History => "HISTORY",
            AppMode::Debug => "DEBUG",
            AppMode::PrettyQuery => "PRETTY",
            AppMode::CacheList => "CACHE",
        };

        // Add useful status info
        let status_info = if self.mode == AppMode::Command {
            let (token_pos, total_tokens) = self.get_cursor_token_position();
            
            // Get current token at cursor
            let current_token = self.get_token_at_cursor();
            let token_display = if let Some(token) = current_token {
                format!(" [{}]", token)
            } else {
                String::new()
            };
            
            format!(" | Token {}/{}{}", token_pos, total_tokens, token_display)
        } else if self.mode == AppMode::Results {
            let row_info = if let Some(data) = self.get_current_data() {
                let total_rows = data.len();
                let selected = self.table_state.selected().unwrap_or(0) + 1;
                format!(" | Row {}/{}", selected, total_rows)
            } else {
                String::new()
            };
            
            let filter_info = if self.filter_state.active {
                format!(" | FILTERED [{}]", self.filter_state.pattern)
            } else {
                String::new()
            };
            
            format!("{}{}", row_info, filter_info)
        } else {
            String::new()
        };

        // Limit status message length to reduce rendering overhead
        let truncated_status = if self.status_message.len() > 40 {
            format!("{}...", &self.status_message[..37])
        } else {
            self.status_message.clone()
        };
        let mode_info = if self.csv_mode {
            format!(" | CSV: {}", self.csv_table_name)
        } else if self.cache_mode {
            " | CACHE MODE".to_string()
        } else {
            String::new()
        };
        let status_text = format!("[{}] {}{}{} | F1:Help F7:Cache q:Quit", mode_indicator, truncated_status, status_info, mode_info);
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

        // Calculate visible columns for virtual scrolling based on actual widths
        let terminal_width = area.width as usize;
        let available_width = terminal_width.saturating_sub(4); // Account for borders and padding
        
        // Calculate how many columns can fit using actual column widths
        let max_visible_cols = if !self.column_widths.is_empty() {
            let mut width_used = 0;
            let mut cols_that_fit = 0;
            
            for (i, &col_width) in self.column_widths.iter().enumerate() {
                if i >= headers.len() { break; }
                if width_used + col_width as usize <= available_width {
                    width_used += col_width as usize;
                    cols_that_fit += 1;
                } else {
                    break;
                }
            }
            cols_that_fit.max(1).min(headers.len())
        } else {
            // Fallback to old method if no calculated widths
            let avg_col_width = 15;
            (available_width / avg_col_width).max(1).min(headers.len())
        };
        
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
        
        // Calculate viewport dimensions FIRST before processing any data
        let terminal_height = area.height as usize;
        let max_visible_rows = terminal_height.saturating_sub(3).max(10);
        
        let total_rows = if let Some(filtered) = &self.filtered_data {
            filtered.len()
        } else {
            results.data.len()
        };
        
        // Calculate row viewport
        let row_viewport_start = self.scroll_offset.0.min(total_rows.saturating_sub(1));
        let row_viewport_end = (row_viewport_start + max_visible_rows).min(total_rows);
        
        // Prepare table data (only visible rows AND columns)
        let data_to_display = if let Some(filtered) = &self.filtered_data {
            // Apply both row and column viewport to filtered data
            filtered[row_viewport_start..row_viewport_end].iter().map(|row| {
                row[viewport_start..viewport_end].to_vec()
            }).collect()
        } else {
            // Convert JSON data to string matrix (only visible rows AND columns)
            results.data[row_viewport_start..row_viewport_end].iter().map(|item| {
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

        let selected_row = self.table_state.selected().unwrap_or(0);
        
        // Create data rows (already filtered to visible rows and columns)
        let rows: Vec<Row> = data_to_display.iter().enumerate().map(|(visible_row_idx, row)| {
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

        // Calculate column constraints using optimal widths (only for visible columns)
        let constraints: Vec<Constraint> = if !self.column_widths.is_empty() {
            // Use calculated optimal widths for visible columns 
            (viewport_start..viewport_end)
                .map(|col_idx| {
                    if col_idx < self.column_widths.len() {
                        Constraint::Length(self.column_widths[col_idx])
                    } else {
                        Constraint::Min(10) // Fallback
                    }
                })
                .collect()
        } else {
            // Fallback to minimum width if no calculated widths available
            (0..visible_headers.len())
                .map(|_| Constraint::Min(10))
                .collect()
        };

        let table = Table::new(rows, constraints)
            .header(Row::new(header_cells).height(1))
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!("Results ({} rows) - Columns {}-{} of {} | Viewport rows {}-{} (selected: {}) | Use h/l to scroll", 
                    total_rows, 
                    viewport_start + 1, 
                    viewport_end, 
                    headers.len(),
                    row_viewport_start + 1,
                    row_viewport_end,
                    selected_row + 1)))
            .row_highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("► ");

        let mut table_state = self.table_state.clone();
        // Adjust table state to use relative position within the viewport
        if let Some(selected) = table_state.selected() {
            let relative_position = selected.saturating_sub(row_viewport_start);
            table_state.select(Some(relative_position));
        }
        f.render_stateful_widget(table, area, &mut table_state);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = Text::from(vec![
            Line::from("SQL CLI Help"),
            Line::from(""),
            Line::from("Command Mode:"),
            Line::from("  Enter    - Execute query"),
            Line::from("  Tab      - Auto-complete"),
            Line::from("  Ctrl+R   - Search command history"),
            Line::from("  "),
            Line::from("Navigation:"),
            Line::from("  Ctrl+A   - Jump to beginning of line"),
            Line::from("  Ctrl+E   - Jump to end of line"),
            Line::from("  Ctrl+←/Alt+B - Move backward one word"),
            Line::from("  Ctrl+→/Alt+F - Move forward one word"),
            Line::from("  Alt+[    - Jump to previous SQL token"),
            Line::from("  Alt+]    - Jump to next SQL token"),
            Line::from("  "),
            Line::from("Editing:"),
            Line::from("  Ctrl+W   - Delete word backward"),
            Line::from("  Alt+D    - Delete word forward"),
            Line::from("  Ctrl+K   - Kill line (delete to end)"),
            Line::from("  Ctrl+U   - Kill line backward"),
            Line::from("  Ctrl+Y   - Yank (paste from kill ring)"),
            Line::from("  Ctrl+Z   - Undo"),
            Line::from("  "),
            Line::from("Other:"),
            Line::from("  F1/?     - Toggle help"),
            Line::from("  F3       - Toggle multi-line mode"),
            Line::from("  F5       - Debug info"),
            Line::from("  F6       - Pretty query view"),
            Line::from("  F7       - Cache management"),
            Line::from("  ↓        - Enter results mode"),
            Line::from("  Ctrl+C/D - Exit"),
            Line::from(""),
            Line::from("Cache Commands:"),
            Line::from("  :cache save    - Save current results to cache"),
            Line::from("  :cache load ID - Load cached query by ID"),
            Line::from("  :cache list    - Show cached queries (F7)"),
            Line::from("  :cache clear   - Disable cache mode"),
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

    fn render_pretty_query(&self, f: &mut Frame, area: Rect) {
        let pretty_lines: Vec<Line> = self.debug_text
            .lines()
            .map(|line| Line::from(line.to_string()))
            .collect();
        
        let total_lines = pretty_lines.len();
        let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
        
        // Calculate visible range based on scroll
        let start = self.debug_scroll as usize;
        let end = (start + visible_height).min(total_lines);
        
        let visible_lines: Vec<Line> = if start < total_lines {
            pretty_lines[start..end].to_vec()
        } else {
            vec![]
        };
        
        let pretty_text = Text::from(visible_lines);
        
        let pretty_paragraph = Paragraph::new(pretty_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Pretty SQL Query (F6) - ↑↓ to scroll, Esc/q to close")
                .border_style(Style::default().fg(Color::Green)))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        
        f.render_widget(pretty_paragraph, area);
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
                Constraint::Percentage(50), // History list - 50% of space
                Constraint::Percentage(50), // Selected command preview - 50% of space
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
            
            // Pretty format the SQL command - adjust compactness based on available space
            use crate::recursive_parser::format_sql_pretty_compact;
            
            // Calculate how many columns we can fit per line
            let available_width = area.width.saturating_sub(6) as usize; // Account for indentation and borders
            let avg_col_width = 15; // Assume average column name is ~15 chars
            let cols_per_line = (available_width / avg_col_width).max(3).min(12); // Between 3-12 columns per line
            
            let mut pretty_lines = format_sql_pretty_compact(&entry.command, cols_per_line);
            
            // If too many lines for the area, use a more compact format
            let max_lines = area.height.saturating_sub(2) as usize; // Account for borders
            if pretty_lines.len() > max_lines && cols_per_line < 12 {
                // Try with more columns per line
                pretty_lines = format_sql_pretty_compact(&entry.command, 15);
            }
            
            // Convert to Text with syntax highlighting
            let mut highlighted_lines = Vec::new();
            for line in pretty_lines {
                highlighted_lines.push(self.sql_highlighter.simple_sql_highlight(&line));
            }
            
            let preview_text = Text::from(highlighted_lines);
            
            let duration_text = entry.duration_ms
                .map(|d| format!("{}ms", d))
                .unwrap_or_else(|| "?ms".to_string());
            
            let success_text = if entry.success { "✓ Success" } else { "✗ Failed" };
            
            let preview = Paragraph::new(preview_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Pretty SQL Preview: {} | {} | Used {}x", success_text, duration_text, entry.execution_count)))
                .scroll((0, 0)); // Allow scrolling if needed
            
            f.render_widget(preview, area);
        } else {
            let empty_preview = Paragraph::new("No command selected")
                .block(Block::default().borders(Borders::ALL).title("Preview"))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(empty_preview, area);
        }
    }
    
    fn handle_cache_command(&mut self, command: &str) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() < 2 {
            self.status_message = "Invalid cache command. Use :cache save <query> or :cache load <id>".to_string();
            return Ok(());
        }
        
        match parts[1] {
            "save" => {
                // Save last query results to cache
                if let Some(ref results) = self.results {
                    if let Some(ref mut cache) = self.query_cache {
                        let query = if parts.len() > 2 {
                            parts[2..].join(" ")
                        } else if let Some(last_entry) = self.command_history.get_last_entry() {
                            last_entry.command.clone()
                        } else {
                            self.status_message = "No query to cache".to_string();
                            return Ok(());
                        };
                        
                        match cache.save_query(&query, &results.data, None) {
                            Ok(id) => {
                                self.status_message = format!("Query cached with ID: {} ({} rows)", id, results.data.len());
                            }
                            Err(e) => {
                                self.status_message = format!("Failed to cache query: {}", e);
                            }
                        }
                    }
                } else {
                    self.status_message = "No results to cache. Execute a query first.".to_string();
                }
            }
            "load" => {
                if parts.len() < 3 {
                    self.status_message = "Usage: :cache load <id>".to_string();
                    return Ok(());
                }
                
                if let Ok(id) = parts[2].parse::<u64>() {
                    if let Some(ref cache) = self.query_cache {
                        match cache.load_query(id) {
                            Ok((_query, data)) => {
                                self.cached_data = Some(data.clone());
                                self.cache_mode = true;
                                self.status_message = format!("Loaded cache ID {} with {} rows. Cache mode enabled.", id, data.len());
                                
                                // Update parser with cached data schema if available
                                if let Some(first_row) = data.first() {
                                    if let Some(obj) = first_row.as_object() {
                                        let columns: Vec<String> = obj.keys().map(|k| k.to_string()).collect();
                                        self.hybrid_parser.update_single_table("cached_data".to_string(), columns);
                                    }
                                }
                            }
                            Err(e) => {
                                self.status_message = format!("Failed to load cache: {}", e);
                            }
                        }
                    }
                } else {
                    self.status_message = "Invalid cache ID".to_string();
                }
            }
            "list" => {
                self.mode = AppMode::CacheList;
            }
            "clear" => {
                self.cache_mode = false;
                self.cached_data = None;
                self.status_message = "Cache mode disabled".to_string();
            }
            _ => {
                self.status_message = "Unknown cache command. Use save, load, list, or clear.".to_string();
            }
        }
        
        Ok(())
    }
    
    fn handle_cache_list_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.mode = AppMode::Command;
            },
            _ => {}
        }
        Ok(false)
    }
    
    fn render_cache_list(&self, f: &mut Frame, area: Rect) {
        if let Some(ref cache) = self.query_cache {
            let cached_queries = cache.list_cached_queries();
            
            if cached_queries.is_empty() {
                let empty = Paragraph::new("No cached queries found.\n\nUse :cache save after running a query to cache results.")
                    .block(Block::default().borders(Borders::ALL).title("Cached Queries (F7)"))
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(empty, area);
                return;
            }
            
            // Create table of cached queries
            let header_cells = vec!["ID", "Query", "Rows", "Cached At"]
                .into_iter()
                .map(|h| Cell::from(h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
                .collect::<Vec<Cell>>();
            
            let rows: Vec<Row> = cached_queries.iter().map(|query| {
                let cells = vec![
                    Cell::from(query.id.to_string()),
                    Cell::from(if query.query_text.len() > 50 {
                        format!("{}...", &query.query_text[..47])
                    } else {
                        query.query_text.clone()
                    }),
                    Cell::from(query.row_count.to_string()),
                    Cell::from(query.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
                ];
                Row::new(cells)
            }).collect();
            
            let table = Table::new(rows, vec![Constraint::Length(6), Constraint::Percentage(50), Constraint::Length(8), Constraint::Length(20)])
                .header(Row::new(header_cells))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Cached Queries (F7) - Use :cache load <id> to load"))
                .row_highlight_style(Style::default().bg(Color::DarkGray));
            
            f.render_widget(table, area);
        } else {
            let error = Paragraph::new("Cache not available")
                .block(Block::default().borders(Borders::ALL).title("Cache Error"))
                .style(Style::default().fg(Color::Red));
            f.render_widget(error, area);
        }
    }
}

pub fn run_enhanced_tui(api_url: &str, data_file: Option<&str>) -> Result<()> {
    let app = if let Some(file_path) = data_file {
        // Determine file type by extension
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "csv" => EnhancedTuiApp::new_with_csv(file_path)?,
            "json" => EnhancedTuiApp::new_with_json(file_path)?,
            _ => return Err(anyhow::anyhow!("Unsupported file type. Please use .csv or .json files")),
        }
    } else {
        EnhancedTuiApp::new(api_url)
    };
    app.run()
}