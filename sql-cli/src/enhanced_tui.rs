use crate::api_client::{ApiClient, QueryResponse};
use crate::hybrid_parser::HybridParser;
use crate::parser::SqlParser;
use crate::sql_highlighter::SqlHighlighter;
use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame, Terminal,
};
use regex::Regex;
use serde_json::Value;
use sql_cli::buffer::{BufferAPI, BufferManager};
use sql_cli::cache::QueryCache;
use sql_cli::config::Config;
use sql_cli::csv_datasource::CsvApiClient;
use sql_cli::history::{CommandHistory, HistoryMatch};
use sql_cli::where_ast::format_where_ast;
use sql_cli::where_parser::WhereParser;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::io::Write;
use tui_input::{backend::crossterm::EventHandler, Input};
use tui_textarea::{CursorMove, TextArea};

#[derive(Clone, PartialEq, Debug)]
enum AppMode {
    Command,
    Results,
    Search,
    Filter,
    FuzzyFilter,
    ColumnSearch,
    Help,
    History,
    Debug,
    PrettyQuery,
    CacheList,
    JumpToRow,
    ColumnStats,
}

#[derive(Clone, PartialEq)]
enum EditMode {
    SingleLine,
    MultiLine,
}

#[derive(Clone, PartialEq, Debug)]
enum SelectionMode {
    Row,
    Cell,
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

struct FuzzyFilterState {
    pattern: String,
    active: bool,
    matcher: SkimMatcherV2,
    filtered_indices: Vec<usize>, // Indices of rows that match
}

impl Clone for FuzzyFilterState {
    fn clone(&self) -> Self {
        Self {
            pattern: self.pattern.clone(),
            active: self.active,
            matcher: SkimMatcherV2::default(), // Create new matcher
            filtered_indices: self.filtered_indices.clone(),
        }
    }
}

#[derive(Clone)]
struct ColumnSearchState {
    pattern: String,
    matching_columns: Vec<(usize, String)>, // (index, column_name)
    current_match: usize,                   // Index into matching_columns
}

#[derive(Clone, Debug)]
struct ColumnStatistics {
    column_name: String,
    column_type: ColumnType,
    // For all columns
    total_count: usize,
    null_count: usize,
    unique_count: usize,
    // For categorical/string columns
    frequency_map: Option<BTreeMap<String, usize>>,
    // For numeric columns
    min: Option<f64>,
    max: Option<f64>,
    sum: Option<f64>,
    mean: Option<f64>,
    median: Option<f64>,
}

#[derive(Clone, Debug)]
enum ColumnType {
    String,
    Numeric,
    Mixed,
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
    last_results_row: Option<usize>, // Preserve row position when switching modes
    last_scroll_offset: (usize, usize), // Preserve scroll offset when switching modes
    show_help: bool,
    sql_parser: SqlParser,
    hybrid_parser: HybridParser,

    // Configuration
    config: Config,

    // Enhanced features
    sort_state: SortState,
    filter_state: FilterState,
    fuzzy_filter_state: FuzzyFilterState,
    search_state: SearchState,
    column_search_state: ColumnSearchState,
    completion_state: CompletionState,
    history_state: HistoryState,
    command_history: CommandHistory,
    filtered_data: Option<Vec<Vec<String>>>,
    column_widths: Vec<u16>,
    scroll_offset: (usize, usize),          // (row, col)
    current_column: usize,                  // For column-based operations
    pinned_columns: Vec<usize>,             // Indices of pinned columns
    column_stats: Option<ColumnStatistics>, // Current column statistics
    sql_highlighter: SqlHighlighter,
    debug_text: String,
    debug_scroll: u16,
    help_scroll: u16,         // Scroll offset for help page
    input_scroll_offset: u16, // Horizontal scroll offset for input
    case_insensitive: bool,   // Toggle for case-insensitive string comparisons

    // Selection and clipboard
    selection_mode: SelectionMode,         // Row or Cell mode
    yank_mode: Option<char>,               // Track multi-key yank commands (e.g., 'yy', 'yc')
    last_yanked: Option<(String, String)>, // (description, value) of last yanked item

    // CSV mode
    csv_client: Option<CsvApiClient>,
    csv_mode: bool,
    csv_table_name: String,

    // Buffer management (new - for supporting multiple files)
    buffer_manager: Option<BufferManager>,
    current_buffer_name: Option<String>, // Name of current buffer/table

    // Cache
    query_cache: Option<QueryCache>,
    cache_mode: bool,
    cached_data: Option<Vec<serde_json::Value>>,

    // Data source tracking
    last_query_source: Option<String>,

    // Undo/redo and kill ring
    undo_stack: Vec<(String, usize)>, // (text, cursor_pos)
    redo_stack: Vec<(String, usize)>,
    kill_ring: String,

    // Viewport tracking
    last_visible_rows: usize, // Track the last calculated viewport height

    // Display options
    compact_mode: bool,               // Compact display mode with reduced padding
    viewport_lock: bool,              // Lock viewport position for anchor scrolling
    viewport_lock_row: Option<usize>, // The row position to lock to in viewport
    show_row_numbers: bool,           // Show row numbers in results view
    jump_to_row_input: String,        // Input buffer for jump to row command
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
    matches!(
        ch,
        ',' | '(' | ')' | '=' | '<' | '>' | '.' | '"' | '\'' | ';'
    )
}

impl EnhancedTuiApp {
    // --- Buffer Compatibility Layer ---
    // These methods provide a gradual migration path from direct field access to BufferAPI

    /// Get current buffer if available (for reading)
    fn current_buffer(&self) -> Option<&dyn sql_cli::buffer::BufferAPI> {
        self.buffer_manager
            .as_ref()?
            .current()
            .map(|b| b as &dyn sql_cli::buffer::BufferAPI)
    }

    /// Get current buffer if available (for writing)  
    fn current_buffer_mut(&mut self) -> Option<&mut dyn sql_cli::buffer::BufferAPI> {
        self.buffer_manager
            .as_mut()?
            .current_mut()
            .map(|b| b as &mut dyn sql_cli::buffer::BufferAPI)
    }

    // Compatibility wrapper for edit_mode
    fn get_edit_mode(&self) -> EditMode {
        if let Some(buffer) = self.current_buffer() {
            // Convert from buffer::EditMode to local EditMode
            match buffer.get_edit_mode() {
                sql_cli::buffer::EditMode::SingleLine => EditMode::SingleLine,
                sql_cli::buffer::EditMode::MultiLine => EditMode::MultiLine,
            }
        } else {
            self.edit_mode.clone()
        }
    }

    fn set_edit_mode(&mut self, mode: EditMode) {
        // Update local field (will be removed later)
        self.edit_mode = mode.clone();

        // Also update in buffer if available
        if let Some(buffer) = self.current_buffer_mut() {
            let buffer_mode = match mode {
                EditMode::SingleLine => sql_cli::buffer::EditMode::SingleLine,
                EditMode::MultiLine => sql_cli::buffer::EditMode::MultiLine,
            };
            buffer.set_edit_mode(buffer_mode);
        }
    }

    // Compatibility wrapper for case_insensitive
    fn get_case_insensitive(&self) -> bool {
        if let Some(buffer) = self.current_buffer() {
            buffer.is_case_insensitive()
        } else {
            self.case_insensitive
        }
    }

    fn set_case_insensitive(&mut self, case_insensitive: bool) {
        if let Some(buffer) = self.current_buffer_mut() {
            buffer.set_case_insensitive(case_insensitive);
        } else {
            self.case_insensitive = case_insensitive;
        }
    }

    // Compatibility wrapper for last_results_row
    fn get_last_results_row(&self) -> Option<usize> {
        if let Some(buffer) = self.current_buffer() {
            buffer.get_last_results_row()
        } else {
            self.last_results_row
        }
    }

    fn set_last_results_row(&mut self, row: Option<usize>) {
        if let Some(buffer) = self.current_buffer_mut() {
            buffer.set_last_results_row(row);
        } else {
            self.last_results_row = row;
        }
    }

    // Compatibility wrapper for last_scroll_offset
    fn get_last_scroll_offset(&self) -> (usize, usize) {
        if let Some(buffer) = self.current_buffer() {
            buffer.get_last_scroll_offset()
        } else {
            self.last_scroll_offset
        }
    }

    fn set_last_scroll_offset(&mut self, offset: (usize, usize)) {
        if let Some(buffer) = self.current_buffer_mut() {
            buffer.set_last_scroll_offset(offset);
        } else {
            self.last_scroll_offset = offset;
        }
    }

    // Compatibility wrapper for last_query_source
    fn get_last_query_source(&self) -> Option<String> {
        if let Some(buffer) = self.current_buffer() {
            buffer.get_last_query_source()
        } else {
            self.last_query_source.clone()
        }
    }

    fn set_last_query_source(&mut self, source: Option<String>) {
        if let Some(buffer) = self.current_buffer_mut() {
            buffer.set_last_query_source(source);
        } else {
            self.last_query_source = source;
        }
    }

    // Compatibility wrapper for input
    fn get_input(&self) -> &tui_input::Input {
        if let Some(buffer) = self.current_buffer() {
            // TODO: Need to get input from buffer - for now use TUI field
            &self.input
        } else {
            &self.input
        }
    }

    fn get_input_mut(&mut self) -> &mut tui_input::Input {
        // For now, always use TUI field since Buffer input access is more complex
        &mut self.input
    }

    // Helper functions to convert between buffer AppMode and local AppMode
    fn buffer_mode_to_local(buffer_mode: sql_cli::buffer::AppMode) -> AppMode {
        match buffer_mode {
            sql_cli::buffer::AppMode::Command => AppMode::Command,
            sql_cli::buffer::AppMode::Results => AppMode::Results,
            sql_cli::buffer::AppMode::Search => AppMode::Search,
            sql_cli::buffer::AppMode::Filter => AppMode::Filter,
            sql_cli::buffer::AppMode::FuzzyFilter => AppMode::FuzzyFilter,
            sql_cli::buffer::AppMode::ColumnSearch => AppMode::ColumnSearch,
            sql_cli::buffer::AppMode::Help => AppMode::Help,
            sql_cli::buffer::AppMode::History => AppMode::History,
            sql_cli::buffer::AppMode::Debug => AppMode::Debug,
            sql_cli::buffer::AppMode::PrettyQuery => AppMode::PrettyQuery,
            sql_cli::buffer::AppMode::CacheList => AppMode::CacheList,
            sql_cli::buffer::AppMode::JumpToRow => AppMode::JumpToRow,
            sql_cli::buffer::AppMode::ColumnStats => AppMode::ColumnStats,
        }
    }

    fn local_mode_to_buffer(local_mode: &AppMode) -> sql_cli::buffer::AppMode {
        match local_mode {
            AppMode::Command => sql_cli::buffer::AppMode::Command,
            AppMode::Results => sql_cli::buffer::AppMode::Results,
            AppMode::Search => sql_cli::buffer::AppMode::Search,
            AppMode::Filter => sql_cli::buffer::AppMode::Filter,
            AppMode::FuzzyFilter => sql_cli::buffer::AppMode::FuzzyFilter,
            AppMode::ColumnSearch => sql_cli::buffer::AppMode::ColumnSearch,
            AppMode::Help => sql_cli::buffer::AppMode::Help,
            AppMode::History => sql_cli::buffer::AppMode::History,
            AppMode::Debug => sql_cli::buffer::AppMode::Debug,
            AppMode::PrettyQuery => sql_cli::buffer::AppMode::PrettyQuery,
            AppMode::CacheList => sql_cli::buffer::AppMode::CacheList,
            AppMode::JumpToRow => sql_cli::buffer::AppMode::JumpToRow,
            AppMode::ColumnStats => sql_cli::buffer::AppMode::ColumnStats,
        }
    }

    // Compatibility wrapper for mode
    fn get_mode(&self) -> AppMode {
        if let Some(buffer) = self.current_buffer() {
            Self::buffer_mode_to_local(buffer.get_mode())
        } else {
            self.mode.clone()
        }
    }

    fn set_mode(&mut self, mode: AppMode) {
        // Update local field (will be removed later)
        self.mode = mode.clone();

        // Also update in buffer if available
        if let Some(buffer) = self.current_buffer_mut() {
            buffer.set_mode(Self::local_mode_to_buffer(&mode));
        }
    }

    // Compatibility wrapper for results
    fn get_results(&self) -> Option<&QueryResponse> {
        // For now, always use TUI field due to type conflicts
        // TODO: Resolve QueryResponse type mismatch between crate::api_client and sql_cli::api_client
        self.results.as_ref()
    }

    fn set_results(&mut self, results: Option<QueryResponse>) {
        // Update local field
        self.results = results;

        // TODO: Also update in buffer when type conflicts are resolved
    }

    // Compatibility wrapper for table_state
    fn get_table_state(&self) -> &TableState {
        // For now, always use TUI field since TableState access is complex
        &self.table_state
    }

    fn get_table_state_mut(&mut self) -> &mut TableState {
        &mut self.table_state
    }

    // Wrapper methods for status_message (uses buffer system)
    fn get_status_message(&self) -> String {
        if let Some(buffer) = self.current_buffer() {
            buffer.get_status_message()
        } else {
            "Ready".to_string()
        }
    }

    fn set_status_message(&mut self, message: String) {
        if let Some(buffer) = self.current_buffer_mut() {
            buffer.set_status_message(message);
        }
    }

    fn sanitize_table_name(name: &str) -> String {
        // Replace spaces and other problematic characters with underscores
        // to create SQL-friendly table names
        // Examples: "Business Crime Borough Level" -> "Business_Crime_Borough_Level"
        name.trim()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    pub fn has_results(&self) -> bool {
        self.get_results().is_some()
    }

    pub fn new(api_url: &str) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));

        // Load configuration
        let config = Config::load().unwrap_or_else(|e| {
            eprintln!("Warning: Could not load config: {}. Using defaults.", e);
            Config::default()
        });

        Self {
            api_client: ApiClient::new(api_url),
            input: Input::default(),
            textarea,
            edit_mode: EditMode::SingleLine,
            mode: AppMode::Command,
            results: None,
            table_state: TableState::default(),
            last_results_row: None,
            last_scroll_offset: (0, 0),
            show_help: false,
            sql_parser: SqlParser::new(),
            hybrid_parser: HybridParser::new(),
            config: config.clone(),

            sort_state: SortState {
                column: None,
                order: SortOrder::None,
            },
            filter_state: FilterState {
                pattern: String::new(),
                regex: None,
                active: false,
            },
            fuzzy_filter_state: FuzzyFilterState {
                pattern: String::new(),
                active: false,
                matcher: SkimMatcherV2::default(),
                filtered_indices: Vec::new(),
            },
            search_state: SearchState {
                pattern: String::new(),
                current_match: None,
                matches: Vec::new(),
                match_index: 0,
            },
            column_search_state: ColumnSearchState {
                pattern: String::new(),
                matching_columns: Vec::new(),
                current_match: 0,
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
            pinned_columns: Vec::new(),
            column_stats: None,
            sql_highlighter: SqlHighlighter::new(),
            debug_text: String::new(),
            debug_scroll: 0,
            help_scroll: 0,
            input_scroll_offset: 0,
            case_insensitive: config.behavior.case_insensitive_default,
            selection_mode: SelectionMode::Row, // Default to row mode
            yank_mode: None,
            last_yanked: None,
            csv_client: None,
            csv_mode: false,
            csv_table_name: String::new(),
            buffer_manager: {
                // Initialize buffer manager with a default buffer
                let mut manager = BufferManager::new();
                let mut buffer = sql_cli::buffer::Buffer::new(1);
                // Sync initial settings from config
                buffer.set_case_insensitive(config.behavior.case_insensitive_default);
                manager.add_buffer(buffer);
                Some(manager)
            },
            current_buffer_name: None,
            query_cache: QueryCache::new().ok(),
            cache_mode: false,
            cached_data: None,
            last_query_source: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            kill_ring: String::new(),
            last_visible_rows: 30, // Default estimate
            compact_mode: config.display.compact_mode,
            viewport_lock: false,
            viewport_lock_row: None,
            show_row_numbers: config.display.show_row_numbers,
            jump_to_row_input: String::new(),
        }
    }

    pub fn new_with_csv(csv_path: &str) -> Result<Self> {
        let mut csv_client = CsvApiClient::new();

        // First create the app to get its config
        let mut app = Self::new(""); // Empty API URL for CSV mode

        // Use the app's config for consistency
        csv_client.set_case_insensitive(app.config.behavior.case_insensitive_default);

        let raw_name = std::path::Path::new(csv_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data")
            .to_string();

        // Sanitize the table name to be SQL-friendly
        let table_name = Self::sanitize_table_name(&raw_name);

        csv_client.load_csv(csv_path, &table_name)?;

        // Get schema from CSV
        let schema = csv_client
            .get_schema()
            .ok_or_else(|| anyhow::anyhow!("Failed to get CSV schema"))?;

        // Configure the app for CSV mode
        app.csv_client = Some(csv_client.clone());
        app.csv_mode = true;
        app.csv_table_name = table_name.clone();
        app.current_buffer_name = Some(format!("{}", raw_name));

        // Replace the default buffer with a CSV buffer
        if let Some(ref mut manager) = app.buffer_manager {
            // Clear all buffers and add a CSV buffer
            manager.clear_all();
            let mut buffer = sql_cli::buffer::Buffer::from_csv(
                1,
                std::path::PathBuf::from(csv_path),
                csv_client,
                table_name.clone(),
            );
            // Apply config settings to the buffer - use app's config
            buffer.set_case_insensitive(app.config.behavior.case_insensitive_default);
            manager.add_buffer(buffer);

            // Sync app-level state from the buffer to ensure status line renders correctly
            if let Some(current_buffer) = manager.current() {
                app.case_insensitive = current_buffer.is_case_insensitive();
            }
        }

        // Update parser with CSV columns
        if let Some(columns) = schema.get(&table_name) {
            // Update the parser with CSV columns
            app.hybrid_parser
                .update_single_table(table_name.clone(), columns.clone());
            let display_msg = if raw_name != table_name {
                format!(
                    "CSV loaded: '{}' as table '{}' with {} columns",
                    raw_name,
                    table_name,
                    columns.len()
                )
            } else {
                format!(
                    "CSV loaded: table '{}' with {} columns",
                    table_name,
                    columns.len()
                )
            };
            app.status_message = display_msg;
        }

        // Auto-execute SELECT * FROM table_name to show data immediately (if configured)
        let auto_query = format!("SELECT * FROM {}", table_name);

        // Populate the input field with the query for easy editing
        app.input = tui_input::Input::new(auto_query.clone()).with_cursor(auto_query.len());

        if app.config.behavior.auto_execute_on_load {
            if let Err(e) = app.execute_query(&auto_query) {
                // If auto-query fails, just log it in status but don't fail the load
                app.status_message = format!(
                    "CSV loaded: table '{}' ({} columns) - Note: {}",
                    table_name,
                    schema.get(&table_name).map(|c| c.len()).unwrap_or(0),
                    e
                );
            }
        }

        Ok(app)
    }

    pub fn new_with_json(json_path: &str) -> Result<Self> {
        let mut csv_client = CsvApiClient::new();

        // First create the app to get its config
        let mut app = Self::new(""); // Empty API URL for JSON mode

        // Use the app's config for consistency
        csv_client.set_case_insensitive(app.config.behavior.case_insensitive_default);

        let raw_name = std::path::Path::new(json_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data")
            .to_string();

        // Sanitize the table name to be SQL-friendly
        let table_name = Self::sanitize_table_name(&raw_name);

        csv_client.load_json(json_path, &table_name)?;

        // Get schema from JSON data
        let schema = csv_client
            .get_schema()
            .ok_or_else(|| anyhow::anyhow!("Failed to get JSON schema"))?;

        // Configure the app for JSON mode
        app.csv_client = Some(csv_client.clone());
        app.csv_mode = true; // Reuse CSV mode since the data structure is the same
        app.csv_table_name = table_name.clone();
        app.current_buffer_name = Some(format!("{}", raw_name));

        // Replace the default buffer with a JSON buffer
        if let Some(ref mut manager) = app.buffer_manager {
            // Clear all buffers and add a JSON buffer
            manager.clear_all();
            let mut buffer = sql_cli::buffer::Buffer::from_json(
                1,
                std::path::PathBuf::from(json_path),
                csv_client,
                table_name.clone(),
            );
            // Apply config settings to the buffer - use app's config
            buffer.set_case_insensitive(app.config.behavior.case_insensitive_default);
            manager.add_buffer(buffer);

            // Sync app-level state from the buffer to ensure status line renders correctly
            if let Some(current_buffer) = manager.current() {
                app.case_insensitive = current_buffer.is_case_insensitive();
            }
        }

        // Update parser with JSON columns
        if let Some(columns) = schema.get(&table_name) {
            app.hybrid_parser
                .update_single_table(table_name.clone(), columns.clone());
            let display_msg = if raw_name != table_name {
                format!(
                    "JSON loaded: '{}' as table '{}' with {} columns",
                    raw_name,
                    table_name,
                    columns.len()
                )
            } else {
                format!(
                    "JSON loaded: table '{}' with {} columns",
                    table_name,
                    columns.len()
                )
            };
            app.status_message = display_msg;
        }

        // Auto-execute SELECT * FROM table_name to show data immediately (if configured)
        let auto_query = format!("SELECT * FROM {}", table_name);

        // Populate the input field with the query for easy editing
        app.input = tui_input::Input::new(auto_query.clone()).with_cursor(auto_query.len());

        if app.config.behavior.auto_execute_on_load {
            if let Err(e) = app.execute_query(&auto_query) {
                // If auto-query fails, just log it in status but don't fail the load
                app.status_message = format!(
                    "JSON loaded: table '{}' ({} columns) - Note: {}",
                    table_name,
                    schema.get(&table_name).map(|c| c.len()).unwrap_or(0),
                    e
                );
            }
        }

        Ok(app)
    }

    pub fn run(mut self) -> Result<()> {
        // Setup terminal with error handling
        if let Err(e) = enable_raw_mode() {
            return Err(anyhow::anyhow!(
                "Failed to enable raw mode: {}. Try running with --classic flag.",
                e
            ));
        }

        let mut stdout = io::stdout();
        if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            let _ = disable_raw_mode();
            return Err(anyhow::anyhow!(
                "Failed to setup terminal: {}. Try running with --classic flag.",
                e
            ));
        }

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(e) => {
                let _ = disable_raw_mode();
                return Err(anyhow::anyhow!(
                    "Failed to create terminal: {}. Try running with --classic flag.",
                    e
                ));
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
            Err(e) => Err(anyhow::anyhow!("TUI error: {}", e)),
        }
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // Initial draw
        terminal.draw(|f| self.ui(f))?;

        loop {
            // Use blocking read for better performance - only process when there's an actual event
            match event::read()? {
                Event::Key(key) => {
                    // On Windows, filter out key release events - only handle key press
                    // This prevents double-triggering of toggles
                    if key.kind != crossterm::event::KeyEventKind::Press {
                        continue;
                    }

                    let should_exit = match self.mode {
                        AppMode::Command => self.handle_command_input(key)?,
                        AppMode::Results => self.handle_results_input(key)?,
                        AppMode::Search => self.handle_search_input(key)?,
                        AppMode::Filter => self.handle_filter_input(key)?,
                        AppMode::FuzzyFilter => self.handle_fuzzy_filter_input(key)?,
                        AppMode::ColumnSearch => self.handle_column_search_input(key)?,
                        AppMode::Help => self.handle_help_input(key)?,
                        AppMode::History => self.handle_history_input(key)?,
                        AppMode::Debug => self.handle_debug_input(key)?,
                        AppMode::PrettyQuery => self.handle_pretty_query_input(key)?,
                        AppMode::CacheList => self.handle_cache_list_input(key)?,
                        AppMode::JumpToRow => self.handle_jump_to_row_input(key)?,
                        AppMode::ColumnStats => self.handle_column_stats_input(key)?,
                    };

                    if should_exit {
                        break;
                    }

                    // Only redraw after handling a key event
                    terminal.draw(|f| self.ui(f))?;
                }
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

        // Debug: Log all Ctrl key combinations
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char(c) = key.code {
                self.set_status_message(format!("DEBUG: Ctrl+{} pressed", c));
            }
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Expand SELECT * to all column names
                self.expand_asterisk();
            }
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                self.set_mode(if self.show_help {
                    AppMode::Help
                } else {
                    AppMode::Command
                });
            }
            KeyCode::F(3) => {
                // Toggle between single-line and multi-line mode
                match self.get_edit_mode() {
                    EditMode::SingleLine => {
                        self.set_edit_mode(EditMode::MultiLine);
                        let current_text = self.input.value().to_string();

                        // Pretty format the query for multi-line editing
                        let formatted_lines = if !current_text.trim().is_empty() {
                            crate::recursive_parser::format_sql_pretty_compact(&current_text, 5)
                        // 5 columns per line for compact multi-line
                        } else {
                            vec![current_text]
                        };

                        self.textarea = TextArea::from(formatted_lines);
                        self.textarea.set_cursor_line_style(
                            Style::default().add_modifier(Modifier::UNDERLINED),
                        );
                        // Move cursor to the beginning
                        self.textarea.move_cursor(CursorMove::Top);
                        self.textarea.move_cursor(CursorMove::Head);
                        self.set_status_message("Multi-line mode (F3 to toggle, Tab for completion, Ctrl+Enter to execute)".to_string());
                    }
                    EditMode::MultiLine => {
                        self.set_edit_mode(EditMode::SingleLine);
                        // Join lines with single space to create compact query
                        let text = self
                            .textarea
                            .lines()
                            .iter()
                            .map(|line| line.trim())
                            .filter(|line| !line.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        self.input = tui_input::Input::new(text);
                        self.set_status_message(
                            "Single-line mode enabled (F3 to toggle multi-line)".to_string(),
                        );
                    }
                }
            }
            KeyCode::F(7) => {
                // F7 - Toggle cache mode or show cache list
                if self.cache_mode {
                    self.mode = AppMode::CacheList;
                } else {
                    self.mode = AppMode::CacheList;
                }
            }
            KeyCode::Enter => {
                let query = match self.get_edit_mode() {
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
                    // Check for special commands
                    if query == ":help" {
                        self.show_help = true;
                        self.mode = AppMode::Help;
                        self.set_status_message("Help Mode - Press ESC to return".to_string());
                    } else if query == ":exit" || query == ":quit" {
                        return Ok(true);
                    } else if query == ":tui" {
                        // Already in TUI mode
                        self.set_status_message("Already in TUI mode".to_string());
                    } else if query.starts_with(":cache ") {
                        self.handle_cache_command(&query)?;
                    } else {
                        self.set_status_message(format!("Processing query: '{}'", query));
                        self.execute_query(&query)?;
                    }
                } else {
                    self.set_status_message("Empty query - please enter a SQL command".to_string());
                }
            }
            KeyCode::Tab => {
                // Tab completion works in both modes
                match self.edit_mode {
                    EditMode::SingleLine => self.apply_completion(),
                    EditMode::MultiLine => {
                        // In vim normal mode, Tab should also trigger completion
                        self.apply_completion_multiline();
                    }
                }
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.mode = AppMode::History;
                self.history_state.search_query.clear();
                self.update_history_matches();
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Jump to beginning of line (like bash/zsh)
                self.input.handle_event(&Event::Key(KeyEvent::new(
                    KeyCode::Home,
                    KeyModifiers::empty(),
                )));
            }
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Jump to end of line (like bash/zsh)
                self.input.handle_event(&Event::Key(KeyEvent::new(
                    KeyCode::End,
                    KeyModifiers::empty(),
                )));
            }
            KeyCode::F(8) => {
                // Toggle case-insensitive string comparisons
                let current = self.get_case_insensitive();
                self.set_case_insensitive(!current);

                // Update CSV client if in CSV mode
                if let Some(ref mut csv_client) = self.csv_client {
                    csv_client.set_case_insensitive(!current);
                }

                self.set_status_message(format!(
                    "Case-insensitive string comparisons: {}",
                    if !current { "ON" } else { "OFF" }
                ));
            }
            KeyCode::F(9) => {
                // F9 as alternative for kill line (for terminals that intercept Ctrl+K)
                self.kill_line();
                self.set_status_message(format!(
                    "Killed to end of line{}",
                    if !self.kill_ring.is_empty() {
                        format!(" ('{}' saved to kill ring)", self.kill_ring)
                    } else {
                        "".to_string()
                    }
                ));
            }
            KeyCode::F(10) => {
                // F10 as alternative for kill line backward (for consistency with F9)
                self.kill_line_backward();
                self.set_status_message(format!(
                    "Killed to beginning of line{}",
                    if !self.kill_ring.is_empty() {
                        format!(" ('{}' saved to kill ring)", self.kill_ring)
                    } else {
                        "".to_string()
                    }
                ));
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Delete word backward (like bash/zsh)
                self.delete_word_backward();
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Delete word forward (like bash/zsh)
                self.delete_word_forward();
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Kill line - delete from cursor to end of line
                self.set_status_message("Ctrl+K pressed - killing to end of line".to_string());
                self.kill_line();
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Alternative: Alt+K for kill line (for terminals that intercept Ctrl+K)
                self.set_status_message("Alt+K - killing to end of line".to_string());
                self.kill_line();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Kill line backward - delete from cursor to beginning of line
                self.kill_line_backward();
            }
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Undo
                self.undo();
            }
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Yank - paste from kill ring
                self.yank();
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Paste from system clipboard
                self.paste_from_clipboard();
            }
            KeyCode::Char('[') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Jump to previous SQL token
                self.jump_to_prev_token();
            }
            KeyCode::Char(']') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Jump to next SQL token
                self.jump_to_next_token();
            }
            KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Move backward one word
                self.move_cursor_word_backward();
            }
            KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Move forward one word
                self.move_cursor_word_forward();
            }
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move backward one word (alt+b like in bash)
                self.move_cursor_word_backward();
            }
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move forward one word (alt+f like in bash)
                self.move_cursor_word_forward();
            }
            KeyCode::Down if self.results.is_some() && self.edit_mode == EditMode::SingleLine => {
                self.mode = AppMode::Results;
                // Restore previous position or default to 0
                let row = self.get_last_results_row().unwrap_or(0);
                self.table_state.select(Some(row));

                // Restore the exact scroll offset from when we left
                self.scroll_offset = self.get_last_scroll_offset();
            }
            KeyCode::F(5) => {
                // Debug command - show detailed parser information
                let cursor_pos = self.input.cursor();
                let visual_cursor = self.input.visual_cursor();
                let query = self.input.value();
                let mut debug_info = self
                    .hybrid_parser
                    .get_detailed_debug_info(query, cursor_pos);

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
                            let (table_name, columns) = schema
                                .iter()
                                .next()
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
                            "\n========== DATASET INFO ==========\nMode: CSV\nNo schema available\n"
                                .to_string()
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
                        SortState {
                            column: Some(col),
                            order,
                        } => format!(
                            "Column {} - {}",
                            col,
                            match order {
                                SortOrder::Ascending => "Ascending",
                                SortOrder::Descending => "Descending",
                                SortOrder::None => "None",
                            }
                        ),
                        _ => "None".to_string(),
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

                // Add status line info
                let status_line_info = format!(
                    "\n========== STATUS LINE INFO ==========\n\
                    Current Mode: {:?}\n\
                    Case Insensitive: {}\n\
                    Compact Mode: {}\n\
                    Viewport Lock: {}\n\
                    CSV Mode: {}\n\
                    Cache Mode: {}\n\
                    Data Source: {}\n\
                    Active Filters: {}\n",
                    self.mode,
                    self.get_case_insensitive(),
                    self.compact_mode,
                    self.viewport_lock,
                    self.csv_mode,
                    self.cache_mode,
                    &self.get_last_query_source().unwrap_or("None".to_string()),
                    if self.fuzzy_filter_state.active {
                        format!("Fuzzy: {}", self.fuzzy_filter_state.pattern)
                    } else if self.filter_state.active {
                        format!("Filter: {}", self.filter_state.pattern)
                    } else {
                        "None".to_string()
                    }
                );
                debug_info.push_str(&status_line_info);

                // Add buffer manager debug info
                debug_info.push_str("\n========== BUFFER MANAGER STATE ==========\n");
                if let Some(ref manager) = self.buffer_manager {
                    debug_info.push_str(&format!("Buffer Manager: INITIALIZED\n"));
                    debug_info.push_str(&format!(
                        "Number of Buffers: {}\n",
                        manager.all_buffers().len()
                    ));
                    debug_info.push_str(&format!(
                        "Current Buffer Index: {}\n",
                        manager.current_index()
                    ));
                    debug_info.push_str(&format!(
                        "Has Multiple Buffers: {}\n",
                        manager.has_multiple()
                    ));

                    // Add info about all buffers
                    for (i, buffer) in manager.all_buffers().iter().enumerate() {
                        debug_info.push_str(&format!(
                            "\nBuffer [{}]: {}\n",
                            i,
                            buffer.display_name()
                        ));
                        debug_info.push_str(&format!("  ID: {}\n", buffer.id));
                        debug_info.push_str(&format!("  Path: {:?}\n", buffer.file_path));
                        debug_info.push_str(&format!("  Modified: {}\n", buffer.modified));
                        debug_info.push_str(&format!("  CSV Mode: {}\n", buffer.csv_mode));
                    }

                    // Add current buffer debug dump
                    if let Some(buffer) = manager.current() {
                        debug_info.push_str("\n========== CURRENT BUFFER DEBUG DUMP ==========\n");
                        debug_info.push_str(&buffer.debug_dump());
                        debug_info.push_str("================================================\n");
                    } else {
                        debug_info.push_str("\nNo current buffer available!\n");
                    }
                } else {
                    debug_info.push_str("Buffer Manager: NOT INITIALIZED\n");
                }
                debug_info.push_str("============================================\n");

                // Store debug info and switch to debug mode
                self.debug_text = debug_info.clone();
                self.debug_scroll = 0;
                self.mode = AppMode::Debug;

                // Try to copy to clipboard
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(&debug_info) {
                        Ok(_) => {
                            self.set_status_message("DEBUG INFO copied to clipboard!".to_string());
                        }
                        Err(e) => {
                            self.set_status_message(format!("Clipboard error: {}", e));
                        }
                    },
                    Err(e) => {
                        self.set_status_message(format!("Can't access clipboard: {}", e));
                    }
                }
            }
            KeyCode::F(6) => {
                // Pretty print query view
                let query = self.input.value();
                if !query.trim().is_empty() {
                    self.debug_text = format!(
                        "Pretty SQL Query\n{}\n\n{}",
                        "=".repeat(50),
                        crate::recursive_parser::format_sql_pretty_compact(query, 5).join("\n")
                    );
                    self.debug_scroll = 0;
                    self.mode = AppMode::PrettyQuery;
                    self.set_status_message(
                        "Pretty query view (press Esc or q to return)".to_string(),
                    );
                } else {
                    self.set_status_message("No query to format".to_string());
                }
            }
            _ => {
                match self.edit_mode {
                    EditMode::SingleLine => {
                        self.input.handle_event(&Event::Key(key));
                        // Clear completion state when typing other characters
                        self.completion_state.suggestions.clear();
                        self.completion_state.current_index = 0;
                        self.handle_completion();
                    }
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
            KeyCode::F(8) => {
                // Toggle case-insensitive string comparisons
                let current = self.get_case_insensitive();
                self.set_case_insensitive(!current);

                // Update CSV client if in CSV mode
                if let Some(ref mut csv_client) = self.csv_client {
                    csv_client.set_case_insensitive(!current);
                }

                self.set_status_message(format!(
                    "Case-insensitive string comparisons: {}",
                    if !current { "ON" } else { "OFF" }
                ));
            }
            KeyCode::Esc => {
                if self.yank_mode.is_some() {
                    // Cancel yank mode
                    self.yank_mode = None;
                    self.set_status_message("Yank cancelled".to_string());
                } else {
                    // Save current position before switching to Command mode
                    if let Some(selected) = self.table_state.selected() {
                        self.set_last_results_row(Some(selected));
                        self.set_last_scroll_offset(self.scroll_offset);
                    }
                    self.mode = AppMode::Command;
                    self.table_state.select(None);
                }
            }
            KeyCode::Up => {
                // Save current position before switching to Command mode
                if let Some(selected) = self.table_state.selected() {
                    self.last_results_row = Some(selected);
                    self.last_scroll_offset = self.scroll_offset;
                }
                self.mode = AppMode::Command;
                self.table_state.select(None);
            }
            // Vim-like navigation
            KeyCode::Char('j') | KeyCode::Down => {
                self.next_row();
            }
            KeyCode::Char('k') => {
                self.previous_row();
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.move_column_left();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.move_column_right();
            }
            KeyCode::Char('^') | KeyCode::Char('0') => {
                // Jump to first column (vim-like)
                self.goto_first_column();
            }
            KeyCode::Char('$') => {
                // Jump to last column (vim-like)
                self.goto_last_column();
            }
            KeyCode::Char('g') => {
                self.goto_first_row();
            }
            KeyCode::Char('G') => {
                self.goto_last_row();
            }
            KeyCode::Char('p') => {
                // Toggle pin for current column
                self.toggle_column_pin();
            }
            KeyCode::Char('P') => {
                // Clear all pinned columns
                self.clear_pinned_columns();
            }
            KeyCode::Char('C') => {
                // Toggle compact mode with Shift+C
                self.compact_mode = !self.compact_mode;
                self.set_status_message(if self.compact_mode {
                    "Compact mode: ON (reduced padding, more columns visible)".to_string()
                } else {
                    "Compact mode: OFF (standard padding)".to_string()
                });
                // Recalculate column widths with new mode
                self.calculate_optimal_column_widths();
            }
            KeyCode::Char(':') => {
                // Start jump to row command
                self.mode = AppMode::JumpToRow;
                self.jump_to_row_input.clear();
                self.set_status_message("Enter row number:".to_string());
            }
            KeyCode::Char(' ') => {
                // Toggle viewport lock with Space
                self.viewport_lock = !self.viewport_lock;
                if self.viewport_lock {
                    // Lock to current position in viewport (middle of screen)
                    let visible_rows = self.last_visible_rows;
                    self.viewport_lock_row = Some(visible_rows / 2);
                    self.set_status_message(format!(
                        "Viewport lock: ON (anchored at row {} of viewport)",
                        visible_rows / 2 + 1
                    ));
                } else {
                    self.viewport_lock_row = None;
                    self.set_status_message("Viewport lock: OFF (normal scrolling)".to_string());
                }
            }
            KeyCode::PageDown | KeyCode::Char('f')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.page_down();
            }
            KeyCode::PageUp | KeyCode::Char('b')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.page_up();
            }
            // Search functionality
            KeyCode::Char('/') => {
                self.mode = AppMode::Search;
                self.search_state.pattern.clear();
                // Save SQL query and use temporary input for search display
                self.undo_stack
                    .push((self.input.value().to_string(), self.input.cursor()));
                self.input = tui_input::Input::default();
            }
            // Column navigation/search functionality (backslash like vim reverse search)
            KeyCode::Char('\\') => {
                self.mode = AppMode::ColumnSearch;
                self.column_search_state.pattern.clear();
                self.column_search_state.matching_columns.clear();
                self.column_search_state.current_match = 0;
                // Save current SQL query before clearing input for column search
                self.undo_stack
                    .push((self.input.value().to_string(), self.input.cursor()));
                self.input = tui_input::Input::default();
            }
            KeyCode::Char('n') => {
                self.next_search_match();
            }
            KeyCode::Char('N') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Only for search navigation when Shift is held
                if !self.search_state.pattern.is_empty() {
                    self.previous_search_match();
                } else {
                    // Toggle row numbers display
                    self.show_row_numbers = !self.show_row_numbers;
                    self.set_status_message(if self.show_row_numbers {
                        "Row numbers: ON (showing line numbers)".to_string()
                    } else {
                        "Row numbers: OFF".to_string()
                    });
                    // Recalculate column widths with new mode
                    self.calculate_optimal_column_widths();
                }
            }
            // Regex filter functionality (uppercase F)
            KeyCode::Char('F') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.mode = AppMode::Filter;
                self.filter_state.pattern.clear();
                // Save SQL query and use temporary input for filter display
                self.undo_stack
                    .push((self.input.value().to_string(), self.input.cursor()));
                self.input = tui_input::Input::default();
            }
            // Fuzzy filter functionality (lowercase f)
            KeyCode::Char('f')
                if !key.modifiers.contains(KeyModifiers::ALT)
                    && !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.mode = AppMode::FuzzyFilter;
                self.fuzzy_filter_state.pattern.clear();
                self.fuzzy_filter_state.filtered_indices.clear();
                self.fuzzy_filter_state.active = false; // Clear active state when entering mode
                                                        // Save SQL query and use temporary input for fuzzy filter display
                self.undo_stack
                    .push((self.input.value().to_string(), self.input.cursor()));
                self.input = tui_input::Input::default();
            }
            // Sort functionality (lowercase s)
            KeyCode::Char('s')
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                self.sort_by_column(self.current_column);
            }
            // Column statistics (uppercase S)
            KeyCode::Char('S') | KeyCode::Char('s')
                if key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                self.calculate_column_statistics();
            }
            // Toggle cell/row selection mode
            KeyCode::Char('v') => {
                self.selection_mode = match self.selection_mode {
                    SelectionMode::Row => {
                        self.set_status_message(
                            "Cell mode - Navigate to select individual cells".to_string(),
                        );
                        SelectionMode::Cell
                    }
                    SelectionMode::Cell => {
                        self.set_status_message(
                            "Row mode - Navigate to select entire rows".to_string(),
                        );
                        SelectionMode::Row
                    }
                };
            }
            // Clipboard operations (vim-like yank)
            KeyCode::Char('y') => {
                match self.selection_mode {
                    SelectionMode::Cell => {
                        // In cell mode, single 'y' yanks the cell
                        self.yank_cell();
                        // Status message will be set by yank_cell
                    }
                    SelectionMode::Row => {
                        if self.yank_mode.is_some() {
                            // Second 'y' for yank row
                            self.yank_row();
                            self.yank_mode = None;
                        } else {
                            // First 'y', enter yank mode
                            self.yank_mode = Some('y');
                            self.set_status_message(
                                "Yank mode: y=row, c=column, a=all, ESC=cancel".to_string(),
                            );
                        }
                    }
                }
            }
            KeyCode::Char('c') if self.yank_mode.is_some() => {
                // 'yc' - yank column
                self.yank_column();
                self.yank_mode = None;
            }
            KeyCode::Char('a') if self.yank_mode.is_some() => {
                // 'ya' - yank all (filtered or all data)
                self.yank_all();
                self.yank_mode = None;
            }
            // Export to CSV
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.export_to_csv();
            }
            // Export to JSON
            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.export_to_json();
            }
            // Number keys for direct column sorting
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(digit) = c.to_digit(10) {
                    let column_index = (digit as usize).saturating_sub(1);
                    self.sort_by_column(column_index);
                }
            }
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.show_help = true;
                self.mode = AppMode::Help;
            }
            _ => {
                // Any other key cancels yank mode
                if self.yank_mode.is_some() {
                    self.yank_mode = None;
                    self.set_status_message("Yank cancelled".to_string());
                }
            }
        }
        Ok(false)
    }

    fn handle_search_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.undo_stack.pop() {
                    self.input = tui_input::Input::new(original_query).with_cursor(cursor_pos);
                }
                self.mode = AppMode::Results;
            }
            KeyCode::Enter => {
                self.perform_search();
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.undo_stack.pop() {
                    self.input = tui_input::Input::new(original_query).with_cursor(cursor_pos);
                }
                self.mode = AppMode::Results;
            }
            KeyCode::Backspace => {
                self.search_state.pattern.pop();
                // Update input for rendering
                self.input = tui_input::Input::new(self.search_state.pattern.clone())
                    .with_cursor(self.search_state.pattern.len());
            }
            KeyCode::Char(c) => {
                self.search_state.pattern.push(c);
                // Update input for rendering
                self.input = tui_input::Input::new(self.search_state.pattern.clone())
                    .with_cursor(self.search_state.pattern.len());
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_filter_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.undo_stack.pop() {
                    self.input = tui_input::Input::new(original_query).with_cursor(cursor_pos);
                }
                self.mode = AppMode::Results;
            }
            KeyCode::Enter => {
                self.apply_filter();
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.undo_stack.pop() {
                    self.input = tui_input::Input::new(original_query).with_cursor(cursor_pos);
                }
                self.mode = AppMode::Results;
            }
            KeyCode::Backspace => {
                self.filter_state.pattern.pop();
                // Update input for rendering
                self.input = tui_input::Input::new(self.filter_state.pattern.clone())
                    .with_cursor(self.filter_state.pattern.len());
            }
            KeyCode::Char(c) => {
                self.filter_state.pattern.push(c);
                // Update input for rendering
                self.input = tui_input::Input::new(self.filter_state.pattern.clone())
                    .with_cursor(self.filter_state.pattern.len());
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_fuzzy_filter_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Clear fuzzy filter and return to results
                self.fuzzy_filter_state.active = false;
                self.fuzzy_filter_state.pattern.clear();
                self.fuzzy_filter_state.filtered_indices.clear();
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.undo_stack.pop() {
                    self.input = tui_input::Input::new(original_query).with_cursor(cursor_pos);
                }
                self.mode = AppMode::Results;
                self.set_status_message("Fuzzy filter cleared".to_string());
            }
            KeyCode::Enter => {
                // Apply fuzzy filter and return to results
                if !self.fuzzy_filter_state.pattern.is_empty() {
                    self.apply_fuzzy_filter();
                    self.fuzzy_filter_state.active = true;
                }
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.undo_stack.pop() {
                    self.input = tui_input::Input::new(original_query).with_cursor(cursor_pos);
                }
                self.mode = AppMode::Results;
            }
            KeyCode::Backspace => {
                self.fuzzy_filter_state.pattern.pop();
                // Update input for rendering
                self.input = tui_input::Input::new(self.fuzzy_filter_state.pattern.clone())
                    .with_cursor(self.fuzzy_filter_state.pattern.len());
                // Re-apply filter in real-time
                if !self.fuzzy_filter_state.pattern.is_empty() {
                    self.apply_fuzzy_filter();
                } else {
                    self.fuzzy_filter_state.filtered_indices.clear();
                    self.fuzzy_filter_state.active = false;
                }
            }
            KeyCode::Char(c) => {
                self.fuzzy_filter_state.pattern.push(c);
                // Update input for rendering
                self.input = tui_input::Input::new(self.fuzzy_filter_state.pattern.clone())
                    .with_cursor(self.fuzzy_filter_state.pattern.len());
                // Apply filter in real-time as user types
                self.apply_fuzzy_filter();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_column_search_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Cancel column search and return to results
                self.mode = AppMode::Results;
                self.column_search_state.pattern.clear();
                self.column_search_state.matching_columns.clear();
                // Restore original SQL query from undo stack
                if let Some((original_query, cursor_pos)) = self.undo_stack.pop() {
                    self.input = tui_input::Input::new(original_query).with_cursor(cursor_pos);
                }
                self.set_status_message("Column search cancelled".to_string());
            }
            KeyCode::Enter => {
                // Jump to first matching column
                if !self.column_search_state.matching_columns.is_empty() {
                    let (column_index, column_name) = &self.column_search_state.matching_columns
                        [self.column_search_state.current_match];
                    self.current_column = *column_index;
                    self.set_status_message(format!("Jumped to column: {}", column_name));
                } else {
                    self.set_status_message("No matching columns found".to_string());
                }
                // Restore original SQL query from undo stack
                if let Some((original_query, cursor_pos)) = self.undo_stack.pop() {
                    self.input = tui_input::Input::new(original_query).with_cursor(cursor_pos);
                }
                self.mode = AppMode::Results;
            }
            KeyCode::Tab => {
                // Next match (Tab only, not 'n' to allow typing 'n' in search)
                if !self.column_search_state.matching_columns.is_empty() {
                    self.column_search_state.current_match =
                        (self.column_search_state.current_match + 1)
                            % self.column_search_state.matching_columns.len();
                    let (column_index, column_name) = &self.column_search_state.matching_columns
                        [self.column_search_state.current_match];
                    self.current_column = *column_index;
                    self.set_status_message(format!(
                        "Column {} of {}: {}",
                        self.column_search_state.current_match + 1,
                        self.column_search_state.matching_columns.len(),
                        column_name
                    ));
                }
            }
            KeyCode::BackTab => {
                // Previous match (Shift+Tab only, not 'N' to allow typing 'N' in search)
                if !self.column_search_state.matching_columns.is_empty() {
                    if self.column_search_state.current_match == 0 {
                        self.column_search_state.current_match =
                            self.column_search_state.matching_columns.len() - 1;
                    } else {
                        self.column_search_state.current_match -= 1;
                    }
                    let (column_index, column_name) = &self.column_search_state.matching_columns
                        [self.column_search_state.current_match];
                    self.current_column = *column_index;
                    self.set_status_message(format!(
                        "Column {} of {}: {}",
                        self.column_search_state.current_match + 1,
                        self.column_search_state.matching_columns.len(),
                        column_name
                    ));
                }
            }
            KeyCode::Backspace => {
                self.column_search_state.pattern.pop();
                // Also update input to keep it in sync for rendering
                self.input = tui_input::Input::new(self.column_search_state.pattern.clone())
                    .with_cursor(self.column_search_state.pattern.len());
                self.update_column_search();
            }
            KeyCode::Char(c) => {
                self.column_search_state.pattern.push(c);
                // Also update input to keep it in sync for rendering
                self.input = tui_input::Input::new(self.column_search_state.pattern.clone())
                    .with_cursor(self.column_search_state.pattern.len());
                self.update_column_search();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_help_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::F(1) => {
                self.show_help = false;
                self.help_scroll = 0; // Reset scroll when closing
                self.mode = if self.results.is_some() {
                    AppMode::Results
                } else {
                    AppMode::Command
                };
            }
            // Scroll help with arrow keys or vim keys
            KeyCode::Down | KeyCode::Char('j') => {
                // Calculate max scroll based on help content
                let max_lines: usize = 58; // Approximate number of lines in help
                let visible_height: usize = 30; // Approximate visible height
                let max_scroll = max_lines.saturating_sub(visible_height);
                if (self.help_scroll as usize) < max_scroll {
                    self.help_scroll += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.help_scroll = self.help_scroll.saturating_sub(1);
            }
            KeyCode::PageDown => {
                let max_lines: usize = 58;
                let visible_height: usize = 30;
                let max_scroll = max_lines.saturating_sub(visible_height);
                self.help_scroll = (self.help_scroll + 10).min(max_scroll as u16);
            }
            KeyCode::PageUp => {
                self.help_scroll = self.help_scroll.saturating_sub(10);
            }
            KeyCode::Home => {
                self.help_scroll = 0;
            }
            KeyCode::End => {
                let max_lines: usize = 58;
                let visible_height: usize = 30;
                let max_scroll = max_lines.saturating_sub(visible_height);
                self.help_scroll = max_scroll as u16;
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_history_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Esc => {
                self.mode = AppMode::Command;
            }
            KeyCode::Enter => {
                if !self.history_state.matches.is_empty()
                    && self.history_state.selected_index < self.history_state.matches.len()
                {
                    let selected_command = self.history_state.matches
                        [self.history_state.selected_index]
                        .entry
                        .command
                        .clone();
                    let cursor_pos = selected_command.len();
                    self.input = tui_input::Input::new(selected_command).with_cursor(cursor_pos);
                    self.mode = AppMode::Command;
                    self.set_status_message("Command loaded from history".to_string());
                    // Reset scroll to show end of command
                    self.input_scroll_offset = 0;
                    self.update_horizontal_scroll(120); // Will be properly updated on next render
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.history_state.matches.is_empty() {
                    self.history_state.selected_index =
                        self.history_state.selected_index.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.history_state.matches.is_empty()
                    && self.history_state.selected_index + 1 < self.history_state.matches.len()
                {
                    self.history_state.selected_index += 1;
                }
            }
            KeyCode::Backspace => {
                self.history_state.search_query.pop();
                self.update_history_matches();
            }
            KeyCode::Char(c) => {
                self.history_state.search_query.push(c);
                self.update_history_matches();
            }
            _ => {}
        }
        Ok(false)
    }

    fn update_history_matches(&mut self) {
        self.history_state.matches = self
            .command_history
            .search(&self.history_state.search_query);
        self.history_state.selected_index = 0;
    }

    fn handle_debug_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.mode = AppMode::Command;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.debug_scroll > 0 {
                    self.debug_scroll = self.debug_scroll.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.debug_scroll = self.debug_scroll.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.debug_scroll = self.debug_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.debug_scroll = self.debug_scroll.saturating_add(10);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_pretty_query_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.mode = AppMode::Command;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.debug_scroll > 0 {
                    self.debug_scroll = self.debug_scroll.saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.debug_scroll = self.debug_scroll.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.debug_scroll = self.debug_scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.debug_scroll = self.debug_scroll.saturating_add(10);
            }
            _ => {}
        }
        Ok(false)
    }

    fn execute_query(&mut self, query: &str) -> Result<()> {
        self.set_status_message(format!("Executing query: '{}'...", query));
        let start_time = std::time::Instant::now();

        let result = if self.cache_mode {
            // When in cache mode, use CSV client to query cached data
            if let Some(ref cached_data) = self.cached_data {
                let mut csv_client = CsvApiClient::new();
                csv_client.load_from_json(cached_data.clone(), "cached_data")?;

                csv_client.query_csv(query).map(|r| QueryResponse {
                    data: r.data,
                    count: r.count,
                    query: crate::api_client::QueryInfo {
                        select: r.query.select,
                        where_clause: r.query.where_clause,
                        order_by: r.query.order_by,
                    },
                    source: Some("cache".to_string()),
                    table: Some("cached_data".to_string()),
                    cached: Some(true),
                })
            } else {
                Err(anyhow::anyhow!("No cached data loaded"))
            }
        } else if self.csv_mode {
            if let Some(ref csv_client) = self.csv_client {
                // Convert CSV result to match the expected type
                csv_client.query_csv(query).map(|r| QueryResponse {
                    data: r.data,
                    count: r.count,
                    query: crate::api_client::QueryInfo {
                        select: r.query.select,
                        where_clause: r.query.where_clause,
                        order_by: r.query.order_by,
                    },
                    source: Some("file".to_string()),
                    table: Some(self.csv_table_name.clone()),
                    cached: Some(false),
                })
            } else {
                Err(anyhow::anyhow!("CSV client not initialized"))
            }
        } else {
            self.api_client
                .query_trades(query)
                .map_err(|e| anyhow::anyhow!("{}", e))
        };

        match result {
            Ok(response) => {
                let duration = start_time.elapsed();
                let _ = self.command_history.add_entry(
                    query.to_string(),
                    true,
                    Some(duration.as_millis() as u64),
                );

                // Add debug info about results
                let row_count = response.data.len();

                // Capture the source from the response
                self.set_last_query_source(response.source.clone());

                self.results = Some(response);
                self.calculate_optimal_column_widths();
                self.reset_table_state();

                if row_count == 0 {
                    self.set_status_message(format!(
                        "Query executed successfully but returned 0 rows ({}ms)",
                        duration.as_millis()
                    ));
                } else {
                    self.set_status_message(format!("Query executed successfully - {} rows returned ({}ms) - Use ↓ or j/k to navigate", row_count, duration.as_millis()));
                }

                self.mode = AppMode::Results;
                self.table_state.select(Some(0));
            }
            Err(e) => {
                let duration = start_time.elapsed();
                let _ = self.command_history.add_entry(
                    query.to_string(),
                    false,
                    Some(duration.as_millis() as u64),
                );
                self.set_status_message(format!("Error: {}", e));
            }
        }
        Ok(())
    }

    fn parse_where_clause_ast(&self, query: &str) -> Result<String> {
        let query_lower = query.to_lowercase();
        if let Some(where_pos) = query_lower.find(" where ") {
            let where_clause = &query[where_pos + 7..]; // Skip " where "

            // Get columns from CSV client if available
            let columns = if self.csv_mode {
                if let Some(ref csv_client) = self.csv_client {
                    if let Some(schema) = csv_client.get_schema() {
                        schema
                            .iter()
                            .next()
                            .map(|(_, cols)| cols.clone())
                            .unwrap_or_default()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else {
                vec![]
            };

            match WhereParser::parse_with_options(
                where_clause,
                columns,
                self.get_case_insensitive(),
            ) {
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
                Err(e) => Err(anyhow::anyhow!("Failed to parse WHERE clause: {}", e)),
            }
        } else {
            Ok(
                "\n========== WHERE CLAUSE AST ==========\nNo WHERE clause found in query\n"
                    .to_string(),
            )
        }
    }

    fn handle_completion(&mut self) {
        let cursor_pos = self.input.cursor();
        let query = self.input.value();

        let hybrid_result = self.hybrid_parser.get_completions(query, cursor_pos);
        if !hybrid_result.suggestions.is_empty() {
            self.set_status_message(format!(
                "Suggestions: {}",
                hybrid_result.suggestions.join(", ")
            ));
        }
    }

    fn apply_completion(&mut self) {
        let cursor_pos = self.input.cursor();
        let query = self.input.value();

        // Check if this is a continuation of the same completion session
        let is_same_context = query == self.completion_state.last_query
            && cursor_pos == self.completion_state.last_cursor_pos;

        if !is_same_context {
            // New completion context - get fresh suggestions
            let hybrid_result = self.hybrid_parser.get_completions(query, cursor_pos);
            if hybrid_result.suggestions.is_empty() {
                self.set_status_message("No completions available".to_string());
                return;
            }

            self.completion_state.suggestions = hybrid_result.suggestions;
            self.completion_state.current_index = 0;
        } else if !self.completion_state.suggestions.is_empty() {
            // Cycle to next suggestion
            self.completion_state.current_index =
                (self.completion_state.current_index + 1) % self.completion_state.suggestions.len();
        } else {
            self.set_status_message("No completions available".to_string());
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
            // Special case: if we completed a string method like Contains(''), position cursor inside quotes
            let cursor_pos = if suggestion_to_use.ends_with("('')") {
                // Position cursor between the quotes
                before_partial.len() + suggestion_to_use.len() - 2
            } else {
                before_partial.len() + suggestion_to_use.len()
            };
            self.input = tui_input::Input::new(new_query.clone()).with_cursor(cursor_pos);

            // Update completion state for next tab press
            self.completion_state.last_query = new_query;
            self.completion_state.last_cursor_pos = cursor_pos;

            let suggestion_info = if self.completion_state.suggestions.len() > 1 {
                format!(
                    "Completed: {} ({}/{} - Tab for next)",
                    suggestion,
                    self.completion_state.current_index + 1,
                    self.completion_state.suggestions.len()
                )
            } else {
                format!("Completed: {}", suggestion)
            };
            self.set_status_message(suggestion_info);
        } else {
            // Just insert the suggestion at cursor position
            let before_cursor = &query[..cursor_pos];
            let after_cursor = &query[cursor_pos..];
            let new_query = format!("{}{}{}", before_cursor, suggestion, after_cursor);

            // Special case: if we completed a string method like Contains(''), position cursor inside quotes
            let cursor_pos_new = if suggestion.ends_with("('')") {
                // Position cursor between the quotes
                cursor_pos + suggestion.len() - 2
            } else {
                cursor_pos + suggestion.len()
            };
            self.input = tui_input::Input::new(new_query.clone()).with_cursor(cursor_pos_new);

            // Update completion state
            self.completion_state.last_query = new_query;
            self.completion_state.last_cursor_pos = cursor_pos_new;

            self.set_status_message(format!("Inserted: {}", suggestion));
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
        let is_same_context = query == self.completion_state.last_query
            && cursor_pos == self.completion_state.last_cursor_pos;

        if !is_same_context {
            // New completion context - get fresh suggestions
            let hybrid_result = self.hybrid_parser.get_completions(&query, cursor_pos);
            if hybrid_result.suggestions.is_empty() {
                self.set_status_message("No completions available".to_string());
                return;
            }

            self.completion_state.suggestions = hybrid_result.suggestions;
            self.completion_state.current_index = 0;
        } else if !self.completion_state.suggestions.is_empty() {
            // Cycle to next suggestion
            self.completion_state.current_index =
                (self.completion_state.current_index + 1) % self.completion_state.suggestions.len();
        } else {
            self.set_status_message("No completions available".to_string());
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
            // Special case: if we completed a string method like Contains(''), position cursor inside quotes
            let new_col = if suggestion_to_use.ends_with("('')") {
                line_before.len() + suggestion_to_use.len() - 2
            } else {
                line_before.len() + suggestion_to_use.len()
            };
            for _ in 0..new_col {
                self.textarea.move_cursor(CursorMove::Forward);
            }

            // Update completion state
            let new_query = self.textarea.lines().join("\n");
            self.completion_state.last_query = new_query;
            self.completion_state.last_cursor_pos =
                cursor_pos - partial.len() + suggestion_to_use.len();

            let suggestion_info = if self.completion_state.suggestions.len() > 1 {
                format!(
                    "Completed: {} ({}/{} - Tab for next)",
                    suggestion,
                    self.completion_state.current_index + 1,
                    self.completion_state.suggestions.len()
                )
            } else {
                format!("Completed: {}", suggestion)
            };
            self.set_status_message(suggestion_info);
        } else {
            // Just insert the suggestion at cursor position
            self.textarea.insert_str(suggestion);

            // Special case: if we inserted a string method like Contains(''), move cursor back inside quotes
            if suggestion.ends_with("('')") {
                self.textarea.move_cursor(CursorMove::Back);
                self.textarea.move_cursor(CursorMove::Back);
            }

            // Update completion state
            let new_query = self.textarea.lines().join("\n");
            self.completion_state.last_query = new_query;
            self.completion_state.last_cursor_pos = if suggestion.ends_with("('')") {
                cursor_pos + suggestion.len() - 2
            } else {
                cursor_pos + suggestion.len()
            };

            self.set_status_message(format!("Inserted: {}", suggestion));
        }
    }

    fn expand_asterisk(&mut self) {
        // Expand SELECT * to all column names
        let query = if self.edit_mode == EditMode::SingleLine {
            self.input.value().to_string()
        } else {
            self.textarea.lines().join(" ")
        };

        // Simple regex-like pattern to find SELECT * FROM table_name
        let query_upper = query.to_uppercase();

        // Find SELECT * pattern
        if let Some(select_pos) = query_upper.find("SELECT") {
            if let Some(star_pos) = query_upper[select_pos..].find("*") {
                let star_abs_pos = select_pos + star_pos;

                // Find FROM clause after the *
                if let Some(from_rel_pos) = query_upper[star_abs_pos..].find("FROM") {
                    let from_abs_pos = star_abs_pos + from_rel_pos;

                    // Extract table name after FROM
                    let after_from = &query[from_abs_pos + 4..].trim_start();
                    let table_name = after_from
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');

                    if !table_name.is_empty() {
                        // Get columns from the schema
                        let columns = self.get_table_columns(table_name);

                        if !columns.is_empty() {
                            // Build the replacement with all columns
                            let columns_str = columns.join(", ");

                            // Replace * with the column list
                            let before_star = &query[..star_abs_pos];
                            let after_star = &query[star_abs_pos + 1..];
                            let new_query = format!("{}{}{}", before_star, columns_str, after_star);

                            // Update the input
                            if self.edit_mode == EditMode::SingleLine {
                                self.input = tui_input::Input::new(new_query.clone())
                                    .with_cursor(new_query.len());
                                self.update_horizontal_scroll(120);
                            } else {
                                // For multiline, format nicely
                                let formatted_lines =
                                    crate::recursive_parser::format_sql_pretty_compact(
                                        &new_query, 5,
                                    );
                                self.textarea = TextArea::from(formatted_lines);
                                self.textarea.set_cursor_line_style(
                                    Style::default().add_modifier(Modifier::UNDERLINED),
                                );
                            }

                            self.set_status_message(format!(
                                "Expanded * to {} columns",
                                columns.len()
                            ));
                        } else {
                            self.set_status_message(format!(
                                "No columns found for table '{}'",
                                table_name
                            ));
                        }
                    } else {
                        self.set_status_message("Could not determine table name".to_string());
                    }
                } else {
                    self.set_status_message("No FROM clause found after SELECT *".to_string());
                }
            } else {
                self.set_status_message("No * found in SELECT clause".to_string());
            }
        } else {
            self.set_status_message("No SELECT clause found".to_string());
        }
    }

    fn get_table_columns(&self, table_name: &str) -> Vec<String> {
        // Try to get columns from the hybrid parser's schema
        // This will include CSV/JSON loaded tables
        self.hybrid_parser.get_table_columns(table_name)
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
            } else if prev_char.is_alphanumeric()
                || prev_char == '_'
                || (prev_char == ' ' && in_quote)
            {
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
        let total_rows = self.get_row_count();
        if total_rows > 0 {
            // Update viewport size before navigation
            self.update_viewport_size();

            let current = self.table_state.selected().unwrap_or(0);
            if current >= total_rows - 1 {
                return;
            } // Already at bottom

            let new_position = current + 1;
            self.table_state.select(Some(new_position));

            // Update viewport based on lock mode
            if self.viewport_lock {
                // In lock mode, keep cursor at fixed viewport position
                if let Some(lock_row) = self.viewport_lock_row {
                    // Adjust viewport so cursor stays at lock_row position
                    self.scroll_offset.0 = new_position.saturating_sub(lock_row);
                }
            } else {
                // Normal scrolling behavior
                let visible_rows = self.last_visible_rows;

                // Check if cursor would be below the last visible row
                if new_position > self.scroll_offset.0 + visible_rows - 1 {
                    // Cursor moved below viewport - scroll down by one
                    self.scroll_offset.0 += 1;
                }
            }
        }
    }

    fn previous_row(&mut self) {
        let current = self.table_state.selected().unwrap_or(0);
        if current == 0 {
            return;
        } // Already at top

        let new_position = current - 1;
        self.table_state.select(Some(new_position));

        // Update viewport based on lock mode
        if self.viewport_lock {
            // In lock mode, keep cursor at fixed viewport position
            if let Some(lock_row) = self.viewport_lock_row {
                // Adjust viewport so cursor stays at lock_row position
                self.scroll_offset.0 = new_position.saturating_sub(lock_row);
            }
        } else {
            // Normal scrolling behavior
            if new_position < self.scroll_offset.0 {
                // Cursor moved above viewport - scroll up
                self.scroll_offset.0 = new_position;
            }
        }
    }

    fn move_column_left(&mut self) {
        self.current_column = self.current_column.saturating_sub(1);
        self.scroll_offset.1 = self.scroll_offset.1.saturating_sub(1);
        self.set_status_message(format!("Column {} selected", self.current_column + 1));
    }

    fn move_column_right(&mut self) {
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let max_columns = obj.len();
                    if self.current_column + 1 < max_columns {
                        self.current_column += 1;
                        self.scroll_offset.1 += 1;
                        self.set_status_message(format!(
                            "Column {} selected",
                            self.current_column + 1
                        ));
                    }
                }
            }
        }
    }

    fn goto_first_column(&mut self) {
        self.current_column = 0;
        self.scroll_offset.1 = 0;
        self.set_status_message("First column selected".to_string());
    }

    fn goto_last_column(&mut self) {
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let max_columns = obj.len();
                    if max_columns > 0 {
                        self.current_column = max_columns - 1;
                        // Update horizontal scroll to show the last column
                        // This ensures the last column is visible in the viewport
                        self.scroll_offset.1 = self.current_column.saturating_sub(5); // Keep some context
                        self.set_status_message(format!(
                            "Last column selected ({})",
                            self.current_column + 1
                        ));
                    }
                }
            }
        }
    }

    fn goto_first_row(&mut self) {
        self.table_state.select(Some(0));
        self.scroll_offset.0 = 0; // Reset viewport to top
    }

    fn toggle_column_pin(&mut self) {
        // Pin or unpin the current column
        if let Some(pos) = self
            .pinned_columns
            .iter()
            .position(|&x| x == self.current_column)
        {
            // Column is already pinned, unpin it
            self.pinned_columns.remove(pos);
            self.set_status_message(format!("Column {} unpinned", self.current_column + 1));
        } else {
            // Pin the column (max 4 pinned columns)
            if self.pinned_columns.len() < 4 {
                self.pinned_columns.push(self.current_column);
                self.pinned_columns.sort(); // Keep them in order
                self.set_status_message(format!("Column {} pinned 📌", self.current_column + 1));
            } else {
                self.set_status_message("Maximum 4 pinned columns allowed".to_string());
            }
        }
    }

    fn clear_pinned_columns(&mut self) {
        self.pinned_columns.clear();
        self.set_status_message("All columns unpinned".to_string());
    }

    fn calculate_column_statistics(&mut self) {
        // Get the current column name and data
        if let Some(results) = &self.results {
            if results.data.is_empty() {
                return;
            }

            // Get column names from first row
            let headers: Vec<String> = if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    obj.keys().map(|k| k.to_string()).collect()
                } else {
                    return;
                }
            } else {
                return;
            };

            if self.current_column >= headers.len() {
                return;
            }

            let column_name = &headers[self.current_column];

            // Use filtered data if available, otherwise use original data
            let data_to_analyze = if let Some(filtered) = &self.filtered_data {
                // Convert filtered data back to JSON values for analysis
                let mut json_data = Vec::new();
                for row in filtered {
                    if self.current_column < row.len() {
                        json_data.push(row[self.current_column].clone());
                    }
                }
                json_data
            } else {
                // Extract column values from JSON data
                results
                    .data
                    .iter()
                    .filter_map(|row| {
                        if let Some(obj) = row.as_object() {
                            obj.get(column_name).map(|v| match v {
                                Value::String(s) => s.clone(),
                                Value::Number(n) => n.to_string(),
                                Value::Bool(b) => b.to_string(),
                                Value::Null => String::new(),
                                _ => v.to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            // Calculate statistics
            let mut stats = ColumnStatistics {
                column_name: column_name.clone(),
                column_type: ColumnType::Mixed,
                total_count: data_to_analyze.len(),
                null_count: 0,
                unique_count: 0,
                frequency_map: None,
                min: None,
                max: None,
                sum: None,
                mean: None,
                median: None,
            };

            // Analyze data type and calculate appropriate statistics
            let mut numeric_values = Vec::new();
            let mut string_values = Vec::new();
            let mut frequency_map: BTreeMap<String, usize> = BTreeMap::new();

            for value in &data_to_analyze {
                if value.is_empty() {
                    stats.null_count += 1;
                } else if let Ok(num) = value.parse::<f64>() {
                    numeric_values.push(num);
                    *frequency_map.entry(value.clone()).or_insert(0) += 1;
                } else {
                    string_values.push(value.clone());
                    *frequency_map.entry(value.clone()).or_insert(0) += 1;
                }
            }

            stats.unique_count = frequency_map.len();

            // Determine column type
            if numeric_values.len() > 0 && string_values.is_empty() {
                stats.column_type = ColumnType::Numeric;

                // Calculate numeric statistics
                if !numeric_values.is_empty() {
                    numeric_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

                    stats.min = Some(numeric_values[0]);
                    stats.max = Some(numeric_values[numeric_values.len() - 1]);
                    stats.sum = Some(numeric_values.iter().sum());
                    stats.mean = Some(stats.sum.unwrap() / numeric_values.len() as f64);

                    // Calculate median
                    let mid = numeric_values.len() / 2;
                    stats.median = if numeric_values.len() % 2 == 0 {
                        Some((numeric_values[mid - 1] + numeric_values[mid]) / 2.0)
                    } else {
                        Some(numeric_values[mid])
                    };
                }

                // Only keep frequency map for small number of unique values
                if frequency_map.len() <= 20 {
                    stats.frequency_map = Some(frequency_map);
                }
            } else if string_values.len() > 0 && numeric_values.is_empty() {
                stats.column_type = ColumnType::String;
                stats.frequency_map = Some(frequency_map);
            } else {
                stats.column_type = ColumnType::Mixed;
                stats.frequency_map = Some(frequency_map);
            }

            self.column_stats = Some(stats);
            self.mode = AppMode::ColumnStats;
        }
    }

    fn check_parser_error(&self, query: &str) -> Option<String> {
        // Quick check for common parser errors
        let mut paren_depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for ch in query.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '\'' => in_string = !in_string,
                '(' if !in_string => paren_depth += 1,
                ')' if !in_string => {
                    paren_depth -= 1;
                    if paren_depth < 0 {
                        return Some("Extra )".to_string());
                    }
                }
                _ => {}
            }
        }

        if paren_depth > 0 {
            return Some(format!("Missing {} )", paren_depth));
        }

        // Could add more checks here (unclosed strings, etc.)
        if in_string {
            return Some("Unclosed string".to_string());
        }

        None
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
        let total_rows = self.get_row_count();
        if total_rows > 0 {
            let last_row = total_rows - 1;
            self.table_state.select(Some(last_row));
            // Position viewport to show the last row at the bottom
            let visible_rows = self.last_visible_rows;
            self.scroll_offset.0 = last_row.saturating_sub(visible_rows - 1);
        }
    }

    fn page_down(&mut self) {
        let total_rows = self.get_row_count();
        if total_rows > 0 {
            let visible_rows = self.last_visible_rows;
            let current = self.table_state.selected().unwrap_or(0);
            let new_position = (current + visible_rows).min(total_rows - 1);

            self.table_state.select(Some(new_position));

            // Scroll viewport down by a page
            self.scroll_offset.0 =
                (self.scroll_offset.0 + visible_rows).min(total_rows.saturating_sub(visible_rows));
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
                    self.set_status_message(format!(
                        "Found {} matches",
                        self.search_state.matches.len()
                    ));
                } else {
                    self.set_status_message("No matches found".to_string());
                }
            } else {
                self.set_status_message("Invalid regex pattern".to_string());
            }
        }
    }

    fn next_search_match(&mut self) {
        if !self.search_state.matches.is_empty() {
            self.search_state.match_index =
                (self.search_state.match_index + 1) % self.search_state.matches.len();
            let (row, _) = self.search_state.matches[self.search_state.match_index];
            self.table_state.select(Some(row));
            self.search_state.current_match =
                Some(self.search_state.matches[self.search_state.match_index]);
            self.set_status_message(format!(
                "Match {} of {}",
                self.search_state.match_index + 1,
                self.search_state.matches.len()
            ));
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
            self.search_state.current_match =
                Some(self.search_state.matches[self.search_state.match_index]);
            self.set_status_message(format!(
                "Match {} of {}",
                self.search_state.match_index + 1,
                self.search_state.matches.len()
            ));
        }
    }

    fn apply_filter(&mut self) {
        if self.filter_state.pattern.is_empty() {
            self.filtered_data = None;
            self.filter_state.active = false;
            self.set_status_message("Filter cleared".to_string());
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

                self.set_status_message(format!("Filtered to {} rows", filtered_count));
            } else {
                self.set_status_message("Invalid regex pattern".to_string());
            }
        }
    }

    fn apply_fuzzy_filter(&mut self) {
        if self.fuzzy_filter_state.pattern.is_empty() {
            self.fuzzy_filter_state.filtered_indices.clear();
            self.fuzzy_filter_state.active = false;
            self.set_status_message("Fuzzy filter cleared".to_string());
            return;
        }

        let pattern = &self.fuzzy_filter_state.pattern;
        let mut filtered_indices = Vec::new();

        // Get the data to filter - either already filtered data or original results
        let data_to_filter = if self.filter_state.active && self.filtered_data.is_some() {
            // If regex filter is active, fuzzy filter on top of that
            self.filtered_data.as_ref()
        } else if let Some(results) = &self.results {
            // Otherwise filter original results
            let mut rows = Vec::new();
            for item in &results.data {
                let mut row = Vec::new();
                if let Some(obj) = item.as_object() {
                    for (_, value) in obj {
                        let cell_str = match value {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "".to_string(),
                            _ => value.to_string(),
                        };
                        row.push(cell_str);
                    }
                    rows.push(row);
                }
            }
            self.filtered_data = Some(rows);
            self.filtered_data.as_ref()
        } else {
            return;
        };

        if let Some(data) = data_to_filter {
            for (index, row) in data.iter().enumerate() {
                // Concatenate all columns into a single string for matching
                let row_text = row.join(" ");

                // Check if pattern starts with ' for exact matching
                let matches = if pattern.starts_with('\'') && pattern.len() > 1 {
                    // Exact substring matching (case-insensitive)
                    let exact_pattern = &pattern[1..];
                    row_text
                        .to_lowercase()
                        .contains(&exact_pattern.to_lowercase())
                } else {
                    // Fuzzy matching
                    if let Some(score) = self
                        .fuzzy_filter_state
                        .matcher
                        .fuzzy_match(&row_text, pattern)
                    {
                        score > 0
                    } else {
                        false
                    }
                };

                if matches {
                    filtered_indices.push(index);
                }
            }
        }

        let match_count = filtered_indices.len();
        self.fuzzy_filter_state.filtered_indices = filtered_indices;
        self.fuzzy_filter_state.active = !self.fuzzy_filter_state.filtered_indices.is_empty();

        if self.fuzzy_filter_state.active {
            let filter_type = if pattern.starts_with('\'') {
                "Exact"
            } else {
                "Fuzzy"
            };
            self.set_status_message(format!(
                "{} filter: {} matches for '{}' (highlighted in magenta)",
                filter_type, match_count, pattern
            ));
            // Reset table state for new filtered view
            self.table_state = TableState::default();
            self.scroll_offset = (0, 0);
        } else {
            let filter_type = if pattern.starts_with('\'') {
                "exact"
            } else {
                "fuzzy"
            };
            self.set_status_message(format!("No {} matches for '{}'", filter_type, pattern));
        }
    }

    fn update_column_search(&mut self) {
        // Get column headers from the current results
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();

                    // Find matching columns (case-insensitive)
                    let pattern = self.column_search_state.pattern.to_lowercase();
                    let mut matching_columns = Vec::new();

                    for (index, header) in headers.iter().enumerate() {
                        if header.to_lowercase().contains(&pattern) {
                            matching_columns.push((index, header.to_string()));
                        }
                    }

                    self.column_search_state.matching_columns = matching_columns;
                    self.column_search_state.current_match = 0;

                    // Update status message
                    if self.column_search_state.pattern.is_empty() {
                        self.set_status_message("Enter column name to search".to_string());
                    } else if self.column_search_state.matching_columns.is_empty() {
                        self.set_status_message(format!(
                            "No columns match '{}'",
                            self.column_search_state.pattern
                        ));
                    } else {
                        let (column_index, column_name) =
                            &self.column_search_state.matching_columns[0];
                        self.current_column = *column_index;
                        self.set_status_message(format!(
                            "Column 1 of {}: {} (Tab=next, Enter=select)",
                            self.column_search_state.matching_columns.len(),
                            column_name
                        ));
                    }
                } else {
                    self.set_status_message("No column data available".to_string());
                }
            } else {
                self.set_status_message("No data available for column search".to_string());
            }
        } else {
            self.set_status_message("No results available for column search".to_string());
        }
    }

    fn sort_by_column(&mut self, column_index: usize) {
        let new_order = match &self.sort_state {
            SortState {
                column: Some(col),
                order,
            } if *col == column_index => match order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::None,
                SortOrder::None => SortOrder::Ascending,
            },
            _ => SortOrder::Ascending,
        };

        if new_order == SortOrder::None {
            // Reset to original order - would need to store original data
            self.sort_state = SortState {
                column: None,
                order: SortOrder::None,
            };
            self.set_status_message("Sort cleared".to_string());
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
                        let mut indexed_rows: Vec<(serde_json::Value, usize)> = results
                            .data
                            .iter()
                            .enumerate()
                            .map(|(i, row)| (row.clone(), i))
                            .collect();

                        // Sort based on the original JSON values
                        indexed_rows.sort_by(|(row_a, _), (row_b, _)| {
                            let val_a = row_a.get(column_name);
                            let val_b = row_b.get(column_name);

                            let cmp = match (val_a, val_b) {
                                (
                                    Some(serde_json::Value::Number(a)),
                                    Some(serde_json::Value::Number(b)),
                                ) => {
                                    // Numeric comparison - this handles integers and floats properly
                                    let a_f64 = a.as_f64().unwrap_or(0.0);
                                    let b_f64 = b.as_f64().unwrap_or(0.0);
                                    a_f64.partial_cmp(&b_f64).unwrap_or(Ordering::Equal)
                                }
                                (
                                    Some(serde_json::Value::String(a)),
                                    Some(serde_json::Value::String(b)),
                                ) => {
                                    // String comparison
                                    a.cmp(b)
                                }
                                (
                                    Some(serde_json::Value::Bool(a)),
                                    Some(serde_json::Value::Bool(b)),
                                ) => {
                                    // Boolean comparison (false < true)
                                    a.cmp(b)
                                }
                                (Some(serde_json::Value::Null), Some(serde_json::Value::Null)) => {
                                    Ordering::Equal
                                }
                                (Some(serde_json::Value::Null), Some(_)) => {
                                    // NULL comes first
                                    Ordering::Less
                                }
                                (Some(_), Some(serde_json::Value::Null)) => {
                                    // NULL comes first
                                    Ordering::Greater
                                }
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
                        let sorted_data: Vec<serde_json::Value> =
                            indexed_rows.into_iter().map(|(row, _)| row).collect();

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

        self.sort_state = SortState {
            column: Some(column_index),
            order: new_order,
        };

        // Reset table state but preserve current column position
        let current_column = self.current_column;
        self.reset_table_state();
        self.current_column = current_column;

        self.set_status_message(format!(
            "Sorted by column {} ({}) - type-aware",
            column_index + 1,
            match new_order {
                SortOrder::Ascending => "ascending",
                SortOrder::Descending => "descending",
                SortOrder::None => "none",
            }
        ));
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

    fn get_row_count(&self) -> usize {
        if let Some(filtered) = &self.filtered_data {
            filtered.len()
        } else if let Some(results) = &self.results {
            results.data.len()
        } else {
            0
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

                results
                    .data
                    .iter()
                    .map(|item| {
                        if let Some(obj) = item.as_object() {
                            headers
                                .iter()
                                .map(|&header| match obj.get(header) {
                                    Some(Value::String(s)) => s.clone(),
                                    Some(Value::Number(n)) => n.to_string(),
                                    Some(Value::Bool(b)) => b.to_string(),
                                    Some(Value::Null) => "".to_string(),
                                    Some(other) => other.to_string(),
                                    None => "".to_string(),
                                })
                                .collect()
                        } else {
                            vec![]
                        }
                    })
                    .collect()
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
        self.set_last_results_row(None); // Reset saved position for new results
        self.set_last_scroll_offset((0, 0)); // Reset saved scroll offset for new results

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

    fn calculate_viewport_column_widths(&mut self, viewport_start: usize, viewport_end: usize) {
        // Calculate column widths based only on visible rows in viewport
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                    let mut widths = Vec::with_capacity(headers.len());

                    // Use compact mode settings
                    let min_width = if self.compact_mode { 4 } else { 6 };
                    let max_width = if self.compact_mode { 20 } else { 30 };
                    let padding = if self.compact_mode { 1 } else { 2 };

                    // Only check visible rows
                    let rows_to_check =
                        &results.data[viewport_start..viewport_end.min(results.data.len())];

                    for header in &headers {
                        // Start with header width
                        let mut max_col_width = header.len();

                        // Check only visible rows for this column
                        for row in rows_to_check {
                            if let Some(obj) = row.as_object() {
                                if let Some(value) = obj.get(*header) {
                                    let display_value = if value.is_null() {
                                        "NULL"
                                    } else if let Some(s) = value.as_str() {
                                        s
                                    } else {
                                        &value.to_string()
                                    };
                                    max_col_width = max_col_width.max(display_value.len());
                                }
                            }
                        }

                        // Apply min/max constraints and padding
                        let width = (max_col_width + padding).clamp(min_width, max_width) as u16;
                        widths.push(width);
                    }

                    self.column_widths = widths;
                }
            }
        }
    }

    fn calculate_optimal_column_widths(&mut self) {
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                    let mut widths = Vec::new();

                    // For large datasets, sample rows instead of checking all
                    const MAX_ROWS_TO_CHECK: usize = 100;
                    let total_rows = results.data.len();

                    // Determine which rows to sample
                    let rows_to_check: Vec<usize> = if total_rows <= MAX_ROWS_TO_CHECK {
                        // Check all rows for small datasets
                        (0..total_rows).collect()
                    } else {
                        // Sample evenly distributed rows for large datasets
                        let step = total_rows / MAX_ROWS_TO_CHECK;
                        (0..MAX_ROWS_TO_CHECK)
                            .map(|i| (i * step).min(total_rows - 1))
                            .collect()
                    };

                    for header in &headers {
                        // Start with header width
                        let mut max_width = header.len();

                        // Check only sampled rows for this column
                        for &row_idx in &rows_to_check {
                            if let Some(row) = results.data.get(row_idx) {
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

    fn escape_csv_field(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
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
                                self.set_status_message(format!("Failed to write headers: {}", e));
                                return;
                            }

                            // Write data rows
                            let mut row_count = 0;
                            for item in &results.data {
                                if let Some(obj) = item.as_object() {
                                    let row: Vec<String> = headers
                                        .iter()
                                        .map(|&header| match obj.get(header) {
                                            Some(Value::String(s)) => Self::escape_csv_field(s),
                                            Some(Value::Number(n)) => n.to_string(),
                                            Some(Value::Bool(b)) => b.to_string(),
                                            Some(Value::Null) => String::new(),
                                            Some(other) => {
                                                Self::escape_csv_field(&other.to_string())
                                            }
                                            None => String::new(),
                                        })
                                        .collect();

                                    let row_line = row.join(",");
                                    if let Err(e) = writeln!(file, "{}", row_line) {
                                        self.set_status_message(format!(
                                            "Failed to write row: {}",
                                            e
                                        ));
                                        return;
                                    }
                                    row_count += 1;
                                }
                            }

                            self.set_status_message(format!(
                                "Exported {} rows to {}",
                                row_count, filename
                            ));
                        }
                        Err(e) => {
                            self.set_status_message(format!("Failed to create file: {}", e));
                        }
                    }
                } else {
                    self.set_status_message("No data to export".to_string());
                }
            } else {
                self.set_status_message("No data to export".to_string());
            }
        } else {
            self.set_status_message("No results to export - run a query first".to_string());
        }
    }

    fn yank_cell(&mut self) {
        if let Some(results) = &self.results {
            if let Some(selected_row) = self.table_state.selected() {
                if let Some(row_data) = results.data.get(selected_row) {
                    if let Some(obj) = row_data.as_object() {
                        let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                        if self.current_column < headers.len() {
                            let header = headers[self.current_column];
                            let value = match obj.get(header) {
                                Some(Value::String(s)) => s.clone(),
                                Some(Value::Number(n)) => n.to_string(),
                                Some(Value::Bool(b)) => b.to_string(),
                                Some(Value::Null) => "NULL".to_string(),
                                Some(other) => other.to_string(),
                                None => String::new(),
                            };

                            // Copy to clipboard
                            match arboard::Clipboard::new() {
                                Ok(mut clipboard) => match clipboard.set_text(&value) {
                                    Ok(_) => {
                                        // Store what was yanked
                                        let col_name = header.to_string();
                                        let display_value = if value.len() > 20 {
                                            format!("{}...", &value[..17])
                                        } else {
                                            value.clone()
                                        };
                                        self.last_yanked = Some((col_name, display_value));
                                        self.set_status_message(format!("Yanked cell: {}", value));
                                    }
                                    Err(e) => {
                                        self.set_status_message(format!("Clipboard error: {}", e));
                                    }
                                },
                                Err(e) => {
                                    self.set_status_message(format!(
                                        "Can't access clipboard: {}",
                                        e
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn yank_row(&mut self) {
        if let Some(results) = &self.results {
            if let Some(selected_row) = self.table_state.selected() {
                if let Some(row_data) = results.data.get(selected_row) {
                    // Convert row to tab-separated values
                    if let Some(obj) = row_data.as_object() {
                        let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                        let values: Vec<String> = headers
                            .iter()
                            .map(|&header| match obj.get(header) {
                                Some(Value::String(s)) => s.clone(),
                                Some(Value::Number(n)) => n.to_string(),
                                Some(Value::Bool(b)) => b.to_string(),
                                Some(Value::Null) => "NULL".to_string(),
                                Some(other) => other.to_string(),
                                None => String::new(),
                            })
                            .collect();

                        let row_text = values.join("\t");

                        // Copy to clipboard
                        match arboard::Clipboard::new() {
                            Ok(mut clipboard) => match clipboard.set_text(&row_text) {
                                Ok(_) => {
                                    self.last_yanked = Some((
                                        format!("Row {}", selected_row + 1),
                                        format!("{} columns", values.len()),
                                    ));
                                    self.set_status_message(format!(
                                        "Yanked row {} ({} columns)",
                                        selected_row + 1,
                                        values.len()
                                    ));
                                }
                                Err(e) => {
                                    self.set_status_message(format!("Clipboard error: {}", e));
                                }
                            },
                            Err(e) => {
                                self.set_status_message(format!("Can't access clipboard: {}", e));
                            }
                        }
                    }
                }
            }
        }
    }

    fn yank_column(&mut self) {
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                    if self.current_column < headers.len() {
                        let header = headers[self.current_column];

                        // Collect all values from this column
                        let column_values: Vec<String> = results
                            .data
                            .iter()
                            .filter_map(|row| {
                                row.as_object().and_then(|obj| {
                                    obj.get(header).map(|v| match v {
                                        Value::String(s) => s.clone(),
                                        Value::Number(n) => n.to_string(),
                                        Value::Bool(b) => b.to_string(),
                                        Value::Null => "NULL".to_string(),
                                        other => other.to_string(),
                                    })
                                })
                            })
                            .collect();

                        let column_text = column_values.join("\n");

                        // Copy to clipboard
                        match arboard::Clipboard::new() {
                            Ok(mut clipboard) => match clipboard.set_text(&column_text) {
                                Ok(_) => {
                                    self.last_yanked = Some((
                                        format!("Column '{}'", header),
                                        format!("{} rows", column_values.len()),
                                    ));
                                    self.set_status_message(format!(
                                        "Yanked column '{}' ({} rows)",
                                        header,
                                        column_values.len()
                                    ));
                                }
                                Err(e) => {
                                    self.set_status_message(format!("Clipboard error: {}", e));
                                }
                            },
                            Err(e) => {
                                self.set_status_message(format!("Can't access clipboard: {}", e));
                            }
                        }
                    }
                }
            }
        }
    }

    fn yank_all(&mut self) {
        if let Some(results) = &self.results {
            // Get the actual data to yank (filtered or all)
            let data_to_export = if self.filter_state.active || self.fuzzy_filter_state.active {
                // Use filtered data
                self.get_filtered_json_data()
            } else {
                // Use all data
                results.data.clone()
            };

            if let Some(first_row) = data_to_export.first() {
                if let Some(obj) = first_row.as_object() {
                    let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();

                    // Create CSV format
                    let mut csv_text = headers.join(",") + "\n";

                    for row in &data_to_export {
                        if let Some(obj) = row.as_object() {
                            let values: Vec<String> = headers
                                .iter()
                                .map(|&header| match obj.get(header) {
                                    Some(Value::String(s)) => escape_csv_field(s),
                                    Some(Value::Number(n)) => n.to_string(),
                                    Some(Value::Bool(b)) => b.to_string(),
                                    Some(Value::Null) => String::new(),
                                    Some(other) => escape_csv_field(&other.to_string()),
                                    None => String::new(),
                                })
                                .collect();
                            csv_text.push_str(&values.join(","));
                            csv_text.push('\n');
                        }
                    }

                    // Copy to clipboard
                    match arboard::Clipboard::new() {
                        Ok(mut clipboard) => match clipboard.set_text(&csv_text) {
                            Ok(_) => {
                                let filter_info =
                                    if self.filter_state.active || self.fuzzy_filter_state.active {
                                        " (filtered)"
                                    } else {
                                        ""
                                    };
                                self.set_status_message(format!(
                                    "Yanked all data{}: {} rows",
                                    filter_info,
                                    data_to_export.len()
                                ));
                            }
                            Err(e) => {
                                self.set_status_message(format!("Clipboard error: {}", e));
                            }
                        },
                        Err(e) => {
                            self.set_status_message(format!("Can't access clipboard: {}", e));
                        }
                    }
                }
            }
        }
    }

    fn paste_from_clipboard(&mut self) {
        // Paste from system clipboard into the current input field
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(text) => {
                    match self.mode {
                        AppMode::Command => {
                            if self.edit_mode == EditMode::SingleLine {
                                // Get current cursor position
                                let cursor_pos = self.input.cursor();
                                let current_value = self.input.value().to_string();

                                // Insert at cursor position
                                let mut new_value = String::new();
                                new_value.push_str(&current_value[..cursor_pos]);
                                new_value.push_str(&text);
                                new_value.push_str(&current_value[cursor_pos..]);

                                self.input = tui_input::Input::new(new_value)
                                    .with_cursor(cursor_pos + text.len());

                                self.set_status_message(format!(
                                    "Pasted {} characters",
                                    text.len()
                                ));
                            } else {
                                // Multi-line mode - insert at cursor
                                self.textarea.insert_str(&text);
                                self.set_status_message(format!(
                                    "Pasted {} characters",
                                    text.len()
                                ));
                            }
                        }
                        AppMode::Filter
                        | AppMode::FuzzyFilter
                        | AppMode::Search
                        | AppMode::ColumnSearch => {
                            // For search/filter modes, append to current pattern
                            let cursor_pos = self.input.cursor();
                            let current_value = self.input.value().to_string();

                            let mut new_value = String::new();
                            new_value.push_str(&current_value[..cursor_pos]);
                            new_value.push_str(&text);
                            new_value.push_str(&current_value[cursor_pos..]);

                            self.input = tui_input::Input::new(new_value)
                                .with_cursor(cursor_pos + text.len());

                            // Update the appropriate filter/search state
                            match self.mode {
                                AppMode::Filter => {
                                    self.filter_state.pattern = self.input.value().to_string();
                                    self.apply_filter();
                                }
                                AppMode::FuzzyFilter => {
                                    self.fuzzy_filter_state.pattern =
                                        self.input.value().to_string();
                                    self.apply_fuzzy_filter();
                                }
                                AppMode::Search => {
                                    self.search_state.pattern = self.input.value().to_string();
                                    // TODO: self.search_in_results();
                                }
                                AppMode::ColumnSearch => {
                                    self.column_search_state.pattern =
                                        self.input.value().to_string();
                                    // TODO: self.search_columns();
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            self.set_status_message("Paste not available in this mode".to_string());
                        }
                    }
                }
                Err(e) => {
                    self.set_status_message(format!("Failed to paste: {}", e));
                }
            },
            Err(e) => {
                self.set_status_message(format!("Can't access clipboard: {}", e));
            }
        }
    }

    fn export_to_json(&mut self) {
        if let Some(results) = &self.results {
            // Get the actual data to export (filtered or all)
            let data_to_export = if self.filter_state.active || self.fuzzy_filter_state.active {
                self.get_filtered_json_data()
            } else {
                results.data.clone()
            };

            // Generate filename with timestamp
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("query_results_{}.json", timestamp);

            match File::create(&filename) {
                Ok(file) => match serde_json::to_writer_pretty(file, &data_to_export) {
                    Ok(_) => {
                        let filter_info =
                            if self.filter_state.active || self.fuzzy_filter_state.active {
                                " (filtered)"
                            } else {
                                ""
                            };
                        self.set_status_message(format!(
                            "Exported{} {} rows to {}",
                            filter_info,
                            data_to_export.len(),
                            filename
                        ));
                    }
                    Err(e) => {
                        self.set_status_message(format!("Failed to write JSON: {}", e));
                    }
                },
                Err(e) => {
                    self.set_status_message(format!("Failed to create file: {}", e));
                }
            }
        } else {
            self.set_status_message("No results to export - run a query first".to_string());
        }
    }

    fn get_filtered_json_data(&self) -> Vec<Value> {
        if let Some(results) = &self.results {
            if self.fuzzy_filter_state.active
                && !self.fuzzy_filter_state.filtered_indices.is_empty()
            {
                self.fuzzy_filter_state
                    .filtered_indices
                    .iter()
                    .filter_map(|&idx| results.data.get(idx).cloned())
                    .collect()
            } else if self.filter_state.active && self.filtered_data.is_some() {
                // Convert filtered_data back to JSON values
                // This is a bit inefficient but maintains consistency
                if let Some(first_row) = results.data.first() {
                    if let Some(obj) = first_row.as_object() {
                        let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                        self.filtered_data
                            .as_ref()
                            .unwrap()
                            .iter()
                            .map(|row| {
                                let mut json_obj = serde_json::Map::new();
                                for (i, value) in row.iter().enumerate() {
                                    if i < headers.len() {
                                        json_obj.insert(
                                            headers[i].to_string(),
                                            Value::String(value.clone()),
                                        );
                                    }
                                }
                                Value::Object(json_obj)
                            })
                            .collect()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                results.data.clone()
            }
        } else {
            Vec::new()
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
                    Token::Asc => "ASC",
                    Token::Desc => "DESC",
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
            self.input.handle_event(&Event::Key(KeyEvent::new(
                KeyCode::Left,
                KeyModifiers::empty(),
            )));
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
        while word_start > 0
            && !chars[word_start - 1].is_whitespace()
            && !is_sql_delimiter(chars[word_start - 1])
        {
            word_start -= 1;
        }

        // If we only moved through whitespace, try to delete at least one word
        if word_start == cursor_pos && word_start > 0 {
            word_start -= 1;
            while word_start > 0
                && !chars[word_start - 1].is_whitespace()
                && !is_sql_delimiter(chars[word_start - 1])
            {
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
            self.input.handle_event(&Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::empty(),
            )));
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
        while word_end < chars.len() && !chars[word_end].is_alphanumeric() && chars[word_end] != '_'
        {
            word_end += 1;
        }

        // Then skip word characters
        while word_end < chars.len()
            && (chars[word_end].is_alphanumeric() || chars[word_end] == '_')
        {
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
        match self.edit_mode {
            EditMode::SingleLine => {
                let query = self.input.value();
                let cursor_pos = self.input.cursor();
                let query_len = query.len();
                let query_str = query.to_string();

                // Debug info
                self.set_status_message(format!(
                    "kill_line: cursor={}, len={}, text='{}'",
                    cursor_pos, query_len, query_str
                ));

                if cursor_pos < query_len {
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

                    // Update status to show what was killed
                    self.set_status_message(format!(
                        "Killed '{}' (cursor was at {})",
                        self.kill_ring, cursor_pos
                    ));
                } else {
                    self.set_status_message(format!(
                        "Nothing to kill - cursor at end (pos={}, len={})",
                        cursor_pos, query_len
                    ));
                }
            }
            EditMode::MultiLine => {
                // For multiline mode, kill from cursor to end of current line
                let (row, col) = self.textarea.cursor();
                let lines = self.textarea.lines();
                if row < lines.len() {
                    let current_line = &lines[row];
                    if col < current_line.len() {
                        // Save killed text to kill ring
                        self.kill_ring = current_line.chars().skip(col).collect::<String>();

                        // Create new line with text up to cursor
                        let new_line = current_line.chars().take(col).collect::<String>();

                        // Update the textarea
                        let mut new_lines: Vec<String> = lines.iter().cloned().collect();
                        new_lines[row] = new_line;
                        self.textarea = TextArea::from(new_lines);
                        self.textarea.set_cursor_line_style(
                            Style::default().add_modifier(Modifier::UNDERLINED),
                        );
                        self.textarea
                            .move_cursor(CursorMove::Jump(row as u16, col as u16));
                    }
                }
            }
        }
    }

    fn kill_line_backward(&mut self) {
        match self.edit_mode {
            EditMode::SingleLine => {
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
            EditMode::MultiLine => {
                // For multiline mode, kill from beginning of line to cursor
                let (row, col) = self.textarea.cursor();
                let lines = self.textarea.lines();
                if row < lines.len() && col > 0 {
                    let current_line = &lines[row];
                    // Save killed text to kill ring
                    self.kill_ring = current_line.chars().take(col).collect::<String>();

                    // Create new line with text after cursor
                    let new_line = current_line.chars().skip(col).collect::<String>();

                    // Update the textarea
                    let mut new_lines: Vec<String> = lines.iter().cloned().collect();
                    new_lines[row] = new_line;
                    self.textarea = TextArea::from(new_lines);
                    self.textarea
                        .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
                    self.textarea.move_cursor(CursorMove::Jump(row as u16, 0));
                }
            }
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
                self.input.handle_event(&Event::Key(KeyEvent::new(
                    KeyCode::Left,
                    KeyModifiers::empty(),
                )));
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
            self.input.handle_event(&Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::empty(),
            )));
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
                        self.set_status_message("Line yanked".to_string());
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
                            self.set_status_message(format!("{} lines yanked", yanked.len()));
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
        self.set_status_message(format!("-- {} -- L{}:C{} (F3 single-line)", mode_str, row + 1, col + 1));
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
            .constraints(
                [
                    Constraint::Length(input_height), // Command input area
                    Constraint::Min(0),               // Results
                    Constraint::Length(3),            // Status bar
                ]
                .as_ref(),
            )
            .split(f.area());

        // Update horizontal scroll based on actual terminal width
        self.update_horizontal_scroll(chunks[0].width);

        // Command input area
        let input_title = match self.mode {
            AppMode::Command => "SQL Query".to_string(),
            AppMode::Results => "SQL Query (Results Mode - Press ↑ to edit)".to_string(),
            AppMode::Search => "Search Pattern".to_string(),
            AppMode::Filter => "Filter Pattern".to_string(),
            AppMode::FuzzyFilter => "Fuzzy Filter".to_string(),
            AppMode::ColumnSearch => "Column Search".to_string(),
            AppMode::Help => "Help".to_string(),
            AppMode::History => format!(
                "History Search: '{}' (Esc to cancel)",
                self.history_state.search_query
            ),
            AppMode::Debug => "Parser Debug (F5)".to_string(),
            AppMode::PrettyQuery => "Pretty Query View (F6)".to_string(),
            AppMode::CacheList => "Cache Management (F7)".to_string(),
            AppMode::JumpToRow => format!("Jump to row: {}", self.jump_to_row_input),
            AppMode::ColumnStats => "Column Statistics (S to close)".to_string(),
        };

        let input_block = Block::default().borders(Borders::ALL).title(input_title);

        let input_text = match self.mode {
            AppMode::Search => self.input.value(), // Use input for rendering
            AppMode::Filter => self.input.value(), // Use input for rendering
            AppMode::FuzzyFilter => self.input.value(), // Use input for rendering
            AppMode::ColumnSearch => self.input.value(), // Column search still uses input since it saves/restores
            AppMode::History => &self.history_state.search_query,
            _ => self.input.value(),
        };

        let input_paragraph = match self.mode {
            AppMode::Command => {
                match self.edit_mode {
                    EditMode::SingleLine => {
                        // Use syntax highlighting for SQL command input with horizontal scrolling
                        let highlighted_line =
                            self.sql_highlighter.simple_sql_highlight(input_text);
                        Paragraph::new(Text::from(vec![highlighted_line]))
                            .block(input_block)
                            .scroll((0, self.get_horizontal_scroll_offset()))
                    }
                    EditMode::MultiLine => {
                        // For multiline mode, we'll render the textarea widget instead
                        // This is a placeholder - actual textarea rendering happens below
                        Paragraph::new("").block(input_block)
                    }
                }
            }
            _ => {
                // Plain text for other modes
                Paragraph::new(input_text)
                    .block(input_block)
                    .style(match self.mode {
                        AppMode::Results => Style::default().fg(Color::DarkGray),
                        AppMode::Search => Style::default().fg(Color::Yellow),
                        AppMode::Filter => Style::default().fg(Color::Cyan),
                        AppMode::FuzzyFilter => Style::default().fg(Color::Magenta),
                        AppMode::ColumnSearch => Style::default().fg(Color::Green),
                        AppMode::Help => Style::default().fg(Color::DarkGray),
                        AppMode::History => Style::default().fg(Color::Magenta),
                        AppMode::Debug => Style::default().fg(Color::Yellow),
                        AppMode::PrettyQuery => Style::default().fg(Color::Green),
                        AppMode::CacheList => Style::default().fg(Color::Cyan),
                        AppMode::JumpToRow => Style::default().fg(Color::Magenta),
                        AppMode::ColumnStats => Style::default().fg(Color::Cyan),
                        _ => Style::default(),
                    })
                    .scroll((0, self.get_horizontal_scroll_offset()))
            }
        };

        // Determine the actual results area based on edit mode
        let results_area = if self.mode == AppMode::Command && self.edit_mode == EditMode::MultiLine
        {
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
                                chunks[0].y + 1,
                            ));
                        }
                    }
                    EditMode::MultiLine => {
                        // Cursor is handled by the textarea widget
                    }
                }
            }
            AppMode::Search => {
                f.set_cursor_position((
                    chunks[0].x + self.input.cursor() as u16 + 1,
                    chunks[0].y + 1,
                ));
            }
            AppMode::Filter => {
                f.set_cursor_position((
                    chunks[0].x + self.input.cursor() as u16 + 1,
                    chunks[0].y + 1,
                ));
            }
            AppMode::FuzzyFilter => {
                f.set_cursor_position((
                    chunks[0].x + self.input.cursor() as u16 + 1,
                    chunks[0].y + 1,
                ));
            }
            AppMode::ColumnSearch => {
                f.set_cursor_position((
                    chunks[0].x + self.input.cursor() as u16 + 1,
                    chunks[0].y + 1,
                ));
            }
            AppMode::JumpToRow => {
                f.set_cursor_position((
                    chunks[0].x + self.jump_to_row_input.len() as u16 + 1,
                    chunks[0].y + 1,
                ));
            }
            AppMode::History => {
                f.set_cursor_position((
                    chunks[0].x + self.history_state.search_query.len() as u16 + 1,
                    chunks[0].y + 1,
                ));
            }
            _ => {}
        }

        // Results area - render based on mode to reduce complexity
        match (&self.mode, self.show_help) {
            (_, true) => self.render_help(f, results_area),
            (AppMode::History, false) => self.render_history(f, results_area),
            (AppMode::Debug, false) => self.render_debug(f, results_area),
            (AppMode::PrettyQuery, false) => self.render_pretty_query(f, results_area),
            (AppMode::CacheList, false) => self.render_cache_list(f, results_area),
            (AppMode::ColumnStats, false) => self.render_column_stats(f, results_area),
            (_, false) if self.results.is_some() => {
                // We need to work around the borrow checker here
                // Calculate widths needs mutable self, but we also need to pass results
                if let Some(results) = &self.results {
                    // Extract viewport info first
                    let terminal_height = results_area.height as usize;
                    let max_visible_rows = terminal_height.saturating_sub(3).max(10);
                    let total_rows = if let Some(filtered) = &self.filtered_data {
                        filtered.len()
                    } else {
                        results.data.len()
                    };
                    let row_viewport_start = self.scroll_offset.0.min(total_rows.saturating_sub(1));
                    let row_viewport_end = (row_viewport_start + max_visible_rows).min(total_rows);

                    // Calculate column widths based on viewport
                    self.calculate_viewport_column_widths(row_viewport_start, row_viewport_end);
                }

                // Now render the table
                if let Some(results) = &self.results {
                    self.render_table_immutable(f, results_area, results);
                }
            }
            _ => {
                // Simple placeholder - reduced text to improve rendering speed
                let placeholder = Paragraph::new("Enter SQL query and press Enter\n\nTip: Use Tab for completion, Ctrl+R for history")
                    .block(Block::default().borders(Borders::ALL).title("Results"))
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(placeholder, results_area);
            }
        }

        // Render mode-specific status line
        self.render_status_line(f, chunks[2]);
    }

    fn render_status_line(&self, f: &mut Frame, area: Rect) {
        // Determine the mode color
        let (status_style, mode_color) = match self.mode {
            AppMode::Command => (Style::default().fg(Color::Green), Color::Green),
            AppMode::Results => (Style::default().fg(Color::Blue), Color::Blue),
            AppMode::Search => (Style::default().fg(Color::Yellow), Color::Yellow),
            AppMode::Filter => (Style::default().fg(Color::Cyan), Color::Cyan),
            AppMode::FuzzyFilter => (Style::default().fg(Color::Magenta), Color::Magenta),
            AppMode::ColumnSearch => (Style::default().fg(Color::Green), Color::Green),
            AppMode::Help => (Style::default().fg(Color::Magenta), Color::Magenta),
            AppMode::History => (Style::default().fg(Color::Magenta), Color::Magenta),
            AppMode::Debug => (Style::default().fg(Color::Yellow), Color::Yellow),
            AppMode::PrettyQuery => (Style::default().fg(Color::Green), Color::Green),
            AppMode::CacheList => (Style::default().fg(Color::Cyan), Color::Cyan),
            AppMode::JumpToRow => (Style::default().fg(Color::Magenta), Color::Magenta),
            AppMode::ColumnStats => (Style::default().fg(Color::Cyan), Color::Cyan),
        };

        let mode_indicator = match self.mode {
            AppMode::Command => "CMD",
            AppMode::Results => "NAV",
            AppMode::Search => "SEARCH",
            AppMode::Filter => "FILTER",
            AppMode::FuzzyFilter => "FUZZY",
            AppMode::ColumnSearch => "COL",
            AppMode::Help => "HELP",
            AppMode::History => "HISTORY",
            AppMode::Debug => "DEBUG",
            AppMode::PrettyQuery => "PRETTY",
            AppMode::CacheList => "CACHE",
            AppMode::JumpToRow => "JUMP",
            AppMode::ColumnStats => "STATS",
        };

        let mut spans = Vec::new();

        // Mode indicator with color
        spans.push(Span::styled(
            format!("[{}]", mode_indicator),
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        ));

        // Show buffer/table name if available
        if let Some(buffer_name) = &self.current_buffer_name {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                buffer_name.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if self.csv_mode && !self.csv_table_name.is_empty() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                self.csv_table_name.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Mode-specific information
        match self.mode {
            AppMode::Command => {
                // In command mode, show editing-related info
                if !self.input.value().trim().is_empty() {
                    let (token_pos, total_tokens) = self.get_cursor_token_position();
                    spans.push(Span::raw(" | "));
                    spans.push(Span::styled(
                        format!("Token {}/{}", token_pos, total_tokens),
                        Style::default().fg(Color::DarkGray),
                    ));

                    // Show current token if available
                    if let Some(token) = self.get_token_at_cursor() {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(
                            format!("[{}]", token),
                            Style::default().fg(Color::Cyan),
                        ));
                    }

                    // Check for parser errors
                    if let Some(error_msg) = self.check_parser_error(self.input.value()) {
                        spans.push(Span::raw(" | "));
                        spans.push(Span::styled(
                            format!("{} {}", self.config.display.icons.warning, error_msg),
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ));
                    }
                }
            }
            AppMode::Results => {
                // In results mode, show navigation and data info
                let total_rows = self.get_row_count();
                if total_rows > 0 {
                    let selected = self.table_state.selected().unwrap_or(0) + 1;
                    spans.push(Span::raw(" | "));

                    // Show selection mode
                    let mode_text = match self.selection_mode {
                        SelectionMode::Cell => "CELL",
                        SelectionMode::Row => "ROW",
                    };
                    spans.push(Span::styled(
                        format!("[{}]", mode_text),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));

                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        format!("Row {}/{}", selected, total_rows),
                        Style::default().fg(Color::White),
                    ));

                    // Column information
                    if let Some(results) = &self.results {
                        if let Some(first_row) = results.data.first() {
                            if let Some(obj) = first_row.as_object() {
                                let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                                if self.current_column < headers.len() {
                                    spans.push(Span::raw(" | Col: "));
                                    spans.push(Span::styled(
                                        headers[self.current_column],
                                        Style::default().fg(Color::Cyan),
                                    ));

                                    // Show pinned columns count if any
                                    if !self.pinned_columns.is_empty() {
                                        spans.push(Span::raw(" | "));
                                        spans.push(Span::styled(
                                            format!("📌{}", self.pinned_columns.len()),
                                            Style::default().fg(Color::Magenta),
                                        ));
                                    }

                                    // In cell mode, show the current cell value
                                    if self.selection_mode == SelectionMode::Cell {
                                        if let Some(selected_row) = self.table_state.selected() {
                                            if let Some(row_data) = results.data.get(selected_row) {
                                                if let Some(row_obj) = row_data.as_object() {
                                                    if let Some(value) =
                                                        row_obj.get(headers[self.current_column])
                                                    {
                                                        let cell_value = match value {
                                                            Value::String(s) => s.clone(),
                                                            Value::Number(n) => n.to_string(),
                                                            Value::Bool(b) => b.to_string(),
                                                            Value::Null => "NULL".to_string(),
                                                            other => other.to_string(),
                                                        };

                                                        // Truncate if too long
                                                        let display_value = if cell_value.len() > 30
                                                        {
                                                            format!("{}...", &cell_value[..27])
                                                        } else {
                                                            cell_value
                                                        };

                                                        spans.push(Span::raw(" = "));
                                                        spans.push(Span::styled(
                                                            display_value,
                                                            Style::default().fg(Color::Yellow),
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Filter indicators
                    if self.fuzzy_filter_state.active {
                        spans.push(Span::raw(" | "));
                        spans.push(Span::styled(
                            format!("Fuzzy: {}", self.fuzzy_filter_state.pattern),
                            Style::default().fg(Color::Magenta),
                        ));
                    } else if self.filter_state.active {
                        spans.push(Span::raw(" | "));
                        spans.push(Span::styled(
                            format!("Filter: {}", self.filter_state.pattern),
                            Style::default().fg(Color::Cyan),
                        ));
                    }

                    // Show last yanked value
                    if let Some((col, val)) = &self.last_yanked {
                        spans.push(Span::raw(" | "));
                        spans.push(Span::styled(
                            "Yanked: ",
                            Style::default().fg(Color::DarkGray),
                        ));
                        spans.push(Span::styled(
                            format!("{}={}", col, val),
                            Style::default().fg(Color::Green),
                        ));
                    }
                }
            }
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                // Show the pattern being typed - always use input for consistency
                let pattern = self.input.value();
                if !pattern.is_empty() {
                    spans.push(Span::raw(" | Pattern: "));
                    spans.push(Span::styled(pattern, Style::default().fg(mode_color)));
                }
            }
            _ => {}
        }

        // Data source indicator (shown in all modes)
        if let Some(source) = self.get_last_query_source() {
            spans.push(Span::raw(" | "));
            let (icon, label, color) = match source.as_str() {
                "cache" => (
                    &self.config.display.icons.cache,
                    "CACHE".to_string(),
                    Color::Cyan,
                ),
                "file" | "FileDataSource" => (
                    &self.config.display.icons.file,
                    "FILE".to_string(),
                    Color::Green,
                ),
                "SqlServerDataSource" => (
                    &self.config.display.icons.database,
                    "SQL".to_string(),
                    Color::Blue,
                ),
                "PublicApiDataSource" => (
                    &self.config.display.icons.api,
                    "API".to_string(),
                    Color::Yellow,
                ),
                _ => (
                    &self.config.display.icons.api,
                    source.clone(),
                    Color::Magenta,
                ),
            };
            spans.push(Span::raw(format!("{} ", icon)));
            spans.push(Span::styled(label, Style::default().fg(color)));
        } else if self.csv_mode {
            spans.push(Span::raw(" | "));
            spans.push(Span::raw(&self.config.display.icons.file));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("CSV: {}", self.csv_table_name),
                Style::default().fg(Color::Green),
            ));
        } else if self.cache_mode {
            spans.push(Span::raw(" | "));
            spans.push(Span::raw(&self.config.display.icons.cache));
            spans.push(Span::raw(" "));
            spans.push(Span::styled("CACHE", Style::default().fg(Color::Cyan)));
        }

        // Global indicators (shown when active)
        let case_insensitive = self.get_case_insensitive();
        if case_insensitive {
            spans.push(Span::raw(" | "));
            // Use to_string() to ensure we get the actual string value
            let icon = self.config.display.icons.case_insensitive.clone();
            spans.push(Span::styled(
                format!("{} CASE", icon),
                Style::default().fg(Color::Cyan),
            ));
        }

        if self.compact_mode {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled("COMPACT", Style::default().fg(Color::Green)));
        }

        if self.viewport_lock {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                &self.config.display.icons.lock,
                Style::default().fg(Color::Magenta),
            ));
        }

        // Help shortcuts (right side)
        let help_text = match self.mode {
            AppMode::Command => "Enter:Run | Tab:Complete | ↓:Results | F1:Help",
            AppMode::Results => match self.selection_mode {
                SelectionMode::Cell => "v:Row mode | y:Yank cell | ↑:Edit | F1:Help",
                SelectionMode::Row => "v:Cell mode | y:Yank | f:Filter | ↑:Edit | F1:Help",
            },
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                "Enter:Apply | Esc:Cancel"
            }
            AppMode::Help
            | AppMode::Debug
            | AppMode::PrettyQuery
            | AppMode::CacheList
            | AppMode::ColumnStats => "Esc:Close",
            AppMode::History => "Enter:Select | Esc:Cancel",
            AppMode::JumpToRow => "Enter:Jump | Esc:Cancel",
        };

        // Calculate available space for help text
        let current_length: usize = spans.iter().map(|s| s.content.len()).sum();
        let available_width = area.width.saturating_sub(4) as usize; // Account for borders
        let help_length = help_text.len();

        if current_length + help_length + 3 < available_width {
            // Add spacing to right-align help text
            let padding = available_width - current_length - help_length - 3;
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                help_text,
                Style::default().fg(Color::DarkGray),
            ));
        }

        let status_line = Line::from(spans);
        let status = Paragraph::new(status_line)
            .block(Block::default().borders(Borders::ALL))
            .style(status_style);
        f.render_widget(status, area);
    }

    fn render_table_immutable(&self, f: &mut Frame, area: Rect, results: &QueryResponse) {
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

        // Split columns into pinned and scrollable
        let mut pinned_headers: Vec<(usize, &str)> = Vec::new();
        let mut scrollable_indices: Vec<usize> = Vec::new();

        for (i, header) in headers.iter().enumerate() {
            if self.pinned_columns.contains(&i) {
                pinned_headers.push((i, header));
            } else {
                scrollable_indices.push(i);
            }
        }

        // Calculate space used by pinned columns
        let mut pinned_width = 0;
        for &(idx, _) in &pinned_headers {
            if idx < self.column_widths.len() {
                pinned_width += self.column_widths[idx] as usize;
            } else {
                pinned_width += 15; // Default width
            }
        }

        // Calculate how many scrollable columns can fit in remaining space
        let remaining_width = available_width.saturating_sub(pinned_width);
        let max_visible_scrollable_cols = if !self.column_widths.is_empty() {
            let mut width_used = 0;
            let mut cols_that_fit = 0;

            for &idx in &scrollable_indices {
                if idx >= headers.len() {
                    break;
                }
                let col_width = if idx < self.column_widths.len() {
                    self.column_widths[idx] as usize
                } else {
                    15
                };
                if width_used + col_width <= remaining_width {
                    width_used += col_width;
                    cols_that_fit += 1;
                } else {
                    break;
                }
            }
            cols_that_fit.max(1)
        } else {
            // Fallback to old method if no calculated widths
            let avg_col_width = 15;
            (remaining_width / avg_col_width).max(1)
        };

        // Calculate viewport for scrollable columns based on current_column
        let current_in_scrollable = scrollable_indices
            .iter()
            .position(|&x| x == self.current_column);
        let viewport_start = if let Some(pos) = current_in_scrollable {
            if pos < max_visible_scrollable_cols / 2 {
                0
            } else if pos + max_visible_scrollable_cols / 2 >= scrollable_indices.len() {
                scrollable_indices
                    .len()
                    .saturating_sub(max_visible_scrollable_cols)
            } else {
                pos.saturating_sub(max_visible_scrollable_cols / 2)
            }
        } else {
            // Current column is pinned, use scroll offset
            self.scroll_offset.1.min(
                scrollable_indices
                    .len()
                    .saturating_sub(max_visible_scrollable_cols),
            )
        };
        let viewport_end =
            (viewport_start + max_visible_scrollable_cols).min(scrollable_indices.len());

        // Build final list of visible columns (pinned + scrollable viewport)
        let mut visible_columns: Vec<(usize, &str)> = Vec::new();
        visible_columns.extend(pinned_headers.iter().copied());
        for i in viewport_start..viewport_end {
            let idx = scrollable_indices[i];
            visible_columns.push((idx, headers[idx]));
        }

        // Only work with visible headers
        let visible_headers: Vec<&str> = visible_columns.iter().map(|(_, h)| *h).collect();

        // Calculate viewport dimensions FIRST before processing any data
        let terminal_height = area.height as usize;
        let max_visible_rows = terminal_height.saturating_sub(3).max(10);

        let total_rows = if let Some(filtered) = &self.filtered_data {
            if self.fuzzy_filter_state.active
                && !self.fuzzy_filter_state.filtered_indices.is_empty()
            {
                self.fuzzy_filter_state.filtered_indices.len()
            } else {
                filtered.len()
            }
        } else {
            results.data.len()
        };

        // Calculate row viewport
        let row_viewport_start = self.scroll_offset.0.min(total_rows.saturating_sub(1));
        let row_viewport_end = (row_viewport_start + max_visible_rows).min(total_rows);

        // Prepare table data (only visible rows AND columns)
        let data_to_display = if let Some(filtered) = &self.filtered_data {
            // Check if fuzzy filter is active
            if self.fuzzy_filter_state.active
                && !self.fuzzy_filter_state.filtered_indices.is_empty()
            {
                // Apply fuzzy filter on top of existing filter
                let mut fuzzy_filtered = Vec::new();
                for &idx in &self.fuzzy_filter_state.filtered_indices {
                    if idx < filtered.len() {
                        fuzzy_filtered.push(filtered[idx].clone());
                    }
                }

                // Recalculate viewport for fuzzy filtered data
                let fuzzy_total = fuzzy_filtered.len();
                let fuzzy_start = self.scroll_offset.0.min(fuzzy_total.saturating_sub(1));
                let fuzzy_end = (fuzzy_start + max_visible_rows).min(fuzzy_total);

                fuzzy_filtered[fuzzy_start..fuzzy_end]
                    .iter()
                    .map(|row| {
                        visible_columns
                            .iter()
                            .map(|(idx, _)| row[*idx].clone())
                            .collect()
                    })
                    .collect()
            } else {
                // Apply both row and column viewport to filtered data
                filtered[row_viewport_start..row_viewport_end]
                    .iter()
                    .map(|row| {
                        visible_columns
                            .iter()
                            .map(|(idx, _)| row[*idx].clone())
                            .collect()
                    })
                    .collect()
            }
        } else {
            // Convert JSON data to string matrix (only visible rows AND columns)
            results.data[row_viewport_start..row_viewport_end]
                .iter()
                .map(|item| {
                    if let Some(obj) = item.as_object() {
                        visible_columns
                            .iter()
                            .map(|(_, header)| match obj.get(*header) {
                                Some(Value::String(s)) => s.clone(),
                                Some(Value::Number(n)) => n.to_string(),
                                Some(Value::Bool(b)) => b.to_string(),
                                Some(Value::Null) => "".to_string(),
                                Some(other) => other.to_string(),
                                None => "".to_string(),
                            })
                            .collect()
                    } else {
                        vec![]
                    }
                })
                .collect::<Vec<Vec<String>>>()
        };

        // Create header row with sort indicators and column selection
        let mut header_cells: Vec<Cell> = Vec::new();

        // Add row number header if enabled
        if self.show_row_numbers {
            header_cells.push(
                Cell::from("#").style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            );
        }

        // Add data headers
        header_cells.extend(visible_columns.iter().map(|(actual_col_index, header)| {
            let sort_indicator = if let Some(col) = self.sort_state.column {
                if col == *actual_col_index {
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

            let column_indicator = if *actual_col_index == self.current_column {
                " [*]"
            } else {
                ""
            };

            // Add pin indicator for pinned columns
            let pin_indicator = if self.pinned_columns.contains(actual_col_index) {
                "📌 "
            } else {
                ""
            };

            let header_text = format!(
                "{}{}{}{}",
                pin_indicator, header, sort_indicator, column_indicator
            );
            let mut style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD);

            // Highlight the current column
            if *actual_col_index == self.current_column {
                style = style.bg(Color::DarkGray);
            }

            Cell::from(header_text).style(style)
        }));

        let selected_row = self.table_state.selected().unwrap_or(0);

        // Create data rows (already filtered to visible rows and columns)
        let rows: Vec<Row> = data_to_display
            .iter()
            .enumerate()
            .map(|(visible_row_idx, row)| {
                let actual_row_idx = row_viewport_start + visible_row_idx;
                let mut cells: Vec<Cell> = Vec::new();

                // Add row number if enabled
                if self.show_row_numbers {
                    let row_num = actual_row_idx + 1; // 1-based numbering
                    cells.push(
                        Cell::from(row_num.to_string()).style(Style::default().fg(Color::Magenta)),
                    );
                }

                // Add data cells
                cells.extend(row.iter().enumerate().map(|(visible_col_idx, cell)| {
                    let actual_col_idx = visible_columns[visible_col_idx].0;
                    let mut style = Style::default();

                    // Cell mode highlighting - highlight only the selected cell
                    let is_selected_row = actual_row_idx == selected_row;
                    let is_selected_cell = is_selected_row && actual_col_idx == self.current_column;

                    if self.selection_mode == SelectionMode::Cell {
                        // In cell mode, only highlight the specific cell
                        if is_selected_cell {
                            // Use a highlighted foreground instead of changing background
                            // This works better with various terminal color schemes
                            style = style
                                .fg(Color::Yellow) // Bright, readable color
                                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
                        }
                    } else {
                        // In row mode, highlight the current column for all rows
                        if actual_col_idx == self.current_column {
                            style = style.bg(Color::DarkGray);
                        }
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

                    // Highlight fuzzy/exact filter matches
                    if self.fuzzy_filter_state.active && !self.fuzzy_filter_state.pattern.is_empty()
                    {
                        let pattern = &self.fuzzy_filter_state.pattern;
                        let cell_matches = if pattern.starts_with('\'') && pattern.len() > 1 {
                            // Exact match highlighting
                            let exact_pattern = &pattern[1..];
                            cell.to_lowercase().contains(&exact_pattern.to_lowercase())
                        } else {
                            // Fuzzy match highlighting - check if this cell contributes to the fuzzy match
                            if let Some(score) =
                                self.fuzzy_filter_state.matcher.fuzzy_match(cell, pattern)
                            {
                                score > 0
                            } else {
                                false
                            }
                        };

                        if cell_matches {
                            style = style.fg(Color::Magenta).add_modifier(Modifier::BOLD);
                        }
                    }

                    Cell::from(cell.as_str()).style(style)
                }));

                Row::new(cells)
            })
            .collect();

        // Calculate column constraints using optimal widths (only for visible columns)
        let mut constraints: Vec<Constraint> = Vec::new();

        // Add constraint for row number column if enabled
        if self.show_row_numbers {
            // Calculate width needed for row numbers (max row count digits + padding)
            let max_row_num = total_rows;
            let row_num_width = max_row_num.to_string().len() as u16 + 2;
            constraints.push(Constraint::Length(row_num_width.min(8))); // Cap at 8 chars
        }

        // Add data column constraints
        if !self.column_widths.is_empty() {
            // Use calculated optimal widths for visible columns
            constraints.extend(visible_columns.iter().map(|(col_idx, _)| {
                if *col_idx < self.column_widths.len() {
                    Constraint::Length(self.column_widths[*col_idx])
                } else {
                    Constraint::Min(10) // Fallback
                }
            }));
        } else {
            // Fallback to minimum width if no calculated widths available
            constraints.extend((0..visible_headers.len()).map(|_| Constraint::Min(10)));
        }

        // Build the table with conditional row highlighting
        let mut table = Table::new(rows, constraints)
            .header(Row::new(header_cells).height(1))
            .block(Block::default()
                .borders(Borders::ALL)
                .title(format!("Results ({} rows) - {} pinned, {} visible of {} | Viewport rows {}-{} (selected: {}) | Use h/l to scroll",
                    total_rows,
                    self.pinned_columns.len(),
                    visible_columns.len(),
                    headers.len(),
                    row_viewport_start + 1,
                    row_viewport_end,
                    selected_row + 1)));

        // Only apply row highlighting in row mode
        if self.selection_mode == SelectionMode::Row {
            table = table
                .row_highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("► ");
        } else {
            // In cell mode, no row highlighting - cell highlighting is handled above
            table = table.highlight_symbol("  ");
        }

        let mut table_state = self.table_state.clone();
        // Adjust table state to use relative position within the viewport
        if let Some(selected) = table_state.selected() {
            let relative_position = selected.saturating_sub(row_viewport_start);
            table_state.select(Some(relative_position));
        }
        f.render_stateful_widget(table, area, &mut table_state);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        // Create two-column layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left column content
        let left_content = vec![
            Line::from("SQL CLI Help - Enhanced Features 🚀").style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::from("COMMAND MODE").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  Enter    - Execute query"),
            Line::from("  Tab      - Auto-complete"),
            Line::from("  Ctrl+R   - Search history"),
            Line::from("  Ctrl+X   - Expand SELECT * to columns"),
            Line::from("  F3       - Toggle multi-line"),
            Line::from(""),
            Line::from("NAVIGATION").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  Ctrl+A   - Beginning of line"),
            Line::from("  Ctrl+E   - End of line"),
            Line::from("  Ctrl+←   - Move backward word"),
            Line::from("  Ctrl+→   - Move forward word"),
            Line::from(""),
            Line::from("EDITING").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  Ctrl+W   - Delete word backward"),
            Line::from("  Alt+D    - Delete word forward"),
            Line::from("  F9       - Kill to end of line (Ctrl+K alternative)"),
            Line::from("  F10      - Kill to beginning (Ctrl+U alternative)"),
            Line::from("  Ctrl+Y   - Yank (paste)"),
            Line::from("  Ctrl+Z   - Undo"),
            Line::from(""),
            Line::from("VIEW MODES").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  F1/?     - Toggle this help"),
            Line::from("  F5       - Debug info"),
            Line::from("  F6       - Pretty query view"),
            Line::from("  F7       - Cache management"),
            Line::from("  F8       - Case-insensitive"),
            Line::from("  ↓        - Enter results mode"),
            Line::from("  Ctrl+C/q - Exit"),
            Line::from(""),
            Line::from("CACHE COMMANDS").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  :cache save [id] - Save with ID"),
            Line::from("  :cache load ID   - Load by ID"),
            Line::from("  :cache list      - Show cached"),
            Line::from("  :cache clear     - Disable cache"),
            Line::from(""),
            Line::from("🌟 FEATURES").style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  • Column statistics (S key)"),
            Line::from("  • Column pinning (p/P keys)"),
            Line::from("  • Dynamic column sizing"),
            Line::from("  • Compact mode (C key)"),
            Line::from("  • Rainbow parentheses"),
            Line::from("  • Auto-execute CSV/JSON"),
            Line::from("  • Multi-source indicators"),
            Line::from("  • LINQ-style null checking"),
            Line::from("  • Named cache IDs"),
            Line::from("  • Row numbers (N key)"),
            Line::from("  • Jump to row (: key)"),
        ];

        // Right column content
        let right_content = vec![
            Line::from("Use ↓/↑ or j/k to scroll help").style(Style::default().fg(Color::DarkGray)),
            Line::from(""),
            Line::from("RESULTS NAVIGATION").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  j/↓      - Next row"),
            Line::from("  k/↑      - Previous row"),
            Line::from("  h/←      - Previous column"),
            Line::from("  l/→      - Next column"),
            Line::from("  g        - First row"),
            Line::from("  G        - Last row"),
            Line::from("  0/^      - First column"),
            Line::from("  $        - Last column"),
            Line::from("  PgDn     - Page down"),
            Line::from("  PgUp     - Page up"),
            Line::from(""),
            Line::from("RESULTS FEATURES").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  C        - 🎯 Toggle compact"),
            Line::from("  N        - 🔢 Toggle row nums"),
            Line::from("  :        - 📍 Jump to row"),
            Line::from("  Space    - 🔒 Viewport lock"),
            Line::from("  p        - 📌 Pin/unpin column"),
            Line::from("  P        - Clear all pins"),
            Line::from("  /        - Search in results"),
            Line::from("  \\        - Search column names"),
            Line::from("  n/N      - Next/prev match"),
            Line::from("  Shift+F  - Filter rows (regex)"),
            Line::from("  f        - Fuzzy filter rows"),
            Line::from("  'text    - Exact match filter"),
            Line::from("             (matches highlighted)"),
            Line::from("  v        - Toggle cell/row mode"),
            Line::from("  s        - Sort by column"),
            Line::from("  S        - 📊 Column statistics"),
            Line::from("  1-9      - Sort by column #"),
            Line::from("  y        - Yank (cell mode: yank cell)"),
            Line::from("    yy     - Yank current row (row mode)"),
            Line::from("    yc     - Yank current column"),
            Line::from("    ya     - Yank all data"),
            Line::from("  Ctrl+E   - Export to CSV"),
            Line::from("  Ctrl+J   - Export to JSON"),
            Line::from("  ↑/Esc    - Back to command"),
            Line::from("  q        - Quit"),
            Line::from(""),
            Line::from("SEARCH/FILTER").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  Enter    - Apply"),
            Line::from("  Esc      - Cancel"),
            Line::from(""),
            Line::from("💡 TIPS").style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from("  • Load CSV: sql-cli data.csv"),
            Line::from("  • Press C for compact view"),
            Line::from("  • Press N for row numbers"),
            Line::from("  • Press : then 200 → row 200"),
            Line::from("  • Space locks viewport"),
            Line::from("  • Columns auto-adjust width"),
            Line::from("  • Named: :cache save q1"),
            Line::from("  • f + 'ubs = exact 'ubs' match"),
            Line::from("  • \\ + name = find column by name"),
            Line::from(""),
            Line::from("📦 Cache 📁 File 🌐 API 🗄️ SQL"),
        ];

        // Calculate visible area for scrolling
        let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
        let left_total_lines = left_content.len();
        let right_total_lines = right_content.len();
        let max_lines = left_total_lines.max(right_total_lines);

        // Apply scroll offset
        let scroll_offset = self.help_scroll as usize;

        // Get visible portions with scrolling
        let left_visible: Vec<Line> = left_content
            .into_iter()
            .skip(scroll_offset)
            .take(visible_height)
            .collect();

        let right_visible: Vec<Line> = right_content
            .into_iter()
            .skip(scroll_offset)
            .take(visible_height)
            .collect();

        // Create scroll indicator in title
        let scroll_indicator = if max_lines > visible_height {
            format!(
                " (↓/↑ to scroll, {}/{})",
                scroll_offset + 1,
                max_lines.saturating_sub(visible_height) + 1
            )
        } else {
            String::new()
        };

        // Render left column
        let left_paragraph = Paragraph::new(Text::from(left_visible))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Help - Commands{}", scroll_indicator)),
            )
            .style(Style::default());

        // Render right column
        let right_paragraph = Paragraph::new(Text::from(right_visible))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help - Navigation & Features"),
            )
            .style(Style::default());

        f.render_widget(left_paragraph, chunks[0]);
        f.render_widget(right_paragraph, chunks[1]);
    }

    fn render_debug(&self, f: &mut Frame, area: Rect) {
        let debug_lines: Vec<Line> = self
            .debug_text
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

        // Check if there's a parse error
        let has_parse_error = self.debug_text.contains("❌ PARSE ERROR ❌");
        let (border_color, title_prefix) = if has_parse_error {
            (Color::Red, "⚠️  Parser Debug Info [PARSE ERROR] ")
        } else {
            (Color::Yellow, "Parser Debug Info ")
        };

        let debug_paragraph = Paragraph::new(debug_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        "{}- Lines {}-{} of {} (↑↓ to scroll, Enter/Esc to close)",
                        title_prefix,
                        start + 1,
                        end,
                        total_lines
                    ))
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });

        f.render_widget(debug_paragraph, area);
    }

    fn render_pretty_query(&self, f: &mut Frame, area: Rect) {
        let pretty_lines: Vec<Line> = self
            .debug_text
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
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Pretty SQL Query (F6) - ↑↓ to scroll, Esc/q to close")
                    .border_style(Style::default().fg(Color::Green)),
            )
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
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Command History"),
                )
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
        let history_items: Vec<Line> = self
            .history_state
            .matches
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
                    format!(
                        "{}…",
                        &entry.command[..available_for_command.saturating_sub(1)]
                    )
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
            .block(Block::default().borders(Borders::ALL).title(format!(
                "History ({} matches) - j/k to navigate, Enter to select",
                self.history_state.matches.len()
            )))
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(history_paragraph, area);
    }

    fn render_selected_command_preview(&self, f: &mut Frame, area: Rect) {
        if let Some(selected_match) = self
            .history_state
            .matches
            .get(self.history_state.selected_index)
        {
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

            let duration_text = entry
                .duration_ms
                .map(|d| format!("{}ms", d))
                .unwrap_or_else(|| "?ms".to_string());

            let success_text = if entry.success {
                "✓ Success"
            } else {
                "✗ Failed"
            };

            let preview = Paragraph::new(preview_text)
                .block(Block::default().borders(Borders::ALL).title(format!(
                    "Pretty SQL Preview: {} | {} | Used {}x",
                    success_text, duration_text, entry.execution_count
                )))
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
            self.set_status_message(
                "Invalid cache command. Use :cache save <query> or :cache load <id>".to_string(),
            );
            return Ok(());
        }

        match parts[1] {
            "save" => {
                // Save last query results to cache with optional custom ID
                if let Some(ref results) = self.results {
                    if let Some(ref mut cache) = self.query_cache {
                        // Check if a custom ID is provided
                        let (custom_id, query) = if parts.len() > 2 {
                            // Check if the first word after "save" could be an ID (alphanumeric)
                            let potential_id = parts[2];
                            if potential_id
                                .chars()
                                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                                && !potential_id.starts_with("SELECT")
                                && !potential_id.starts_with("select")
                            {
                                // First word is likely an ID
                                let id = Some(potential_id.to_string());
                                let query = if parts.len() > 3 {
                                    parts[3..].join(" ")
                                } else if let Some(last_entry) =
                                    self.command_history.get_last_entry()
                                {
                                    last_entry.command.clone()
                                } else {
                                    self.set_status_message("No query to cache".to_string());
                                    return Ok(());
                                };
                                (id, query)
                            } else {
                                // No ID provided, treat everything as the query
                                (None, parts[2..].join(" "))
                            }
                        } else if let Some(last_entry) = self.command_history.get_last_entry() {
                            (None, last_entry.command.clone())
                        } else {
                            self.set_status_message("No query to cache".to_string());
                            return Ok(());
                        };

                        match cache.save_query(&query, &results.data, custom_id) {
                            Ok(id) => {
                                self.set_status_message(format!(
                                    "Query cached with ID: {} ({} rows)",
                                    id,
                                    results.data.len()
                                ));
                            }
                            Err(e) => {
                                self.set_status_message(format!("Failed to cache query: {}", e));
                            }
                        }
                    }
                } else {
                    self.set_status_message(
                        "No results to cache. Execute a query first.".to_string(),
                    );
                }
            }
            "load" => {
                if parts.len() < 3 {
                    self.set_status_message("Usage: :cache load <id>".to_string());
                    return Ok(());
                }

                if let Ok(id) = parts[2].parse::<u64>() {
                    if let Some(ref cache) = self.query_cache {
                        match cache.load_query(id) {
                            Ok((_query, data)) => {
                                self.cached_data = Some(data.clone());
                                self.cache_mode = true;
                                self.set_status_message(format!(
                                    "Loaded cache ID {} with {} rows. Cache mode enabled.",
                                    id,
                                    data.len()
                                ));

                                // Update parser with cached data schema if available
                                if let Some(first_row) = data.first() {
                                    if let Some(obj) = first_row.as_object() {
                                        let columns: Vec<String> =
                                            obj.keys().map(|k| k.to_string()).collect();
                                        self.hybrid_parser.update_single_table(
                                            "cached_data".to_string(),
                                            columns,
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                self.set_status_message(format!("Failed to load cache: {}", e));
                            }
                        }
                    }
                } else {
                    self.set_status_message("Invalid cache ID".to_string());
                }
            }
            "list" => {
                self.mode = AppMode::CacheList;
            }
            "clear" => {
                self.cache_mode = false;
                self.cached_data = None;
                self.set_status_message("Cache mode disabled".to_string());
            }
            _ => {
                self.set_status_message(
                    "Unknown cache command. Use save, load, list, or clear.".to_string(),
                );
            }
        }

        Ok(())
    }

    fn handle_cache_list_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.mode = AppMode::Command;
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_column_stats_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('S') => {
                self.column_stats = None;
                self.mode = AppMode::Results;
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_jump_to_row_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Results;
                self.jump_to_row_input.clear();
                self.set_status_message("Jump cancelled".to_string());
            }
            KeyCode::Enter => {
                if let Ok(row_num) = self.jump_to_row_input.parse::<usize>() {
                    if row_num > 0 {
                        let target_row = row_num - 1; // Convert to 0-based index
                        let max_row = self.get_current_data().map(|d| d.len()).unwrap_or(0);

                        if target_row < max_row {
                            self.table_state.select(Some(target_row));

                            // Adjust viewport to center the target row
                            let visible_rows = self.last_visible_rows;
                            if visible_rows > 0 {
                                self.scroll_offset.0 = target_row.saturating_sub(visible_rows / 2);
                            }

                            self.set_status_message(format!("Jumped to row {}", row_num));
                        } else {
                            self.set_status_message(format!(
                                "Row {} out of range (max: {})",
                                row_num, max_row
                            ));
                        }
                    }
                }
                self.mode = AppMode::Results;
                self.jump_to_row_input.clear();
            }
            KeyCode::Backspace => {
                self.jump_to_row_input.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.jump_to_row_input.push(c);
            }
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
                .map(|h| {
                    Cell::from(h).style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                })
                .collect::<Vec<Cell>>();

            let rows: Vec<Row> = cached_queries
                .iter()
                .map(|query| {
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
                })
                .collect();

            let table = Table::new(
                rows,
                vec![
                    Constraint::Length(6),
                    Constraint::Percentage(50),
                    Constraint::Length(8),
                    Constraint::Length(20),
                ],
            )
            .header(Row::new(header_cells))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Cached Queries (F7) - Use :cache load <id> to load"),
            )
            .row_highlight_style(Style::default().bg(Color::DarkGray));

            f.render_widget(table, area);
        } else {
            let error = Paragraph::new("Cache not available")
                .block(Block::default().borders(Borders::ALL).title("Cache Error"))
                .style(Style::default().fg(Color::Red));
            f.render_widget(error, area);
        }
    }

    fn render_column_stats(&self, f: &mut Frame, area: Rect) {
        if let Some(ref stats) = self.column_stats {
            let mut lines = vec![
                Line::from(format!("Column Statistics: {}", stats.column_name)).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Line::from(""),
                Line::from(format!("Type: {:?}", stats.column_type))
                    .style(Style::default().fg(Color::Yellow)),
                Line::from(format!("Total Rows: {}", stats.total_count)),
                Line::from(format!("Unique Values: {}", stats.unique_count)),
                Line::from(format!("Null/Empty Count: {}", stats.null_count)),
                Line::from(""),
            ];

            // Add numeric statistics if available
            if matches!(stats.column_type, ColumnType::Numeric | ColumnType::Mixed) {
                lines.push(
                    Line::from("Numeric Statistics:").style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
                if let Some(min) = stats.min {
                    lines.push(Line::from(format!("  Min: {:.2}", min)));
                }
                if let Some(max) = stats.max {
                    lines.push(Line::from(format!("  Max: {:.2}", max)));
                }
                if let Some(mean) = stats.mean {
                    lines.push(Line::from(format!("  Mean: {:.2}", mean)));
                }
                if let Some(median) = stats.median {
                    lines.push(Line::from(format!("  Median: {:.2}", median)));
                }
                if let Some(sum) = stats.sum {
                    lines.push(Line::from(format!("  Sum: {:.2}", sum)));
                }
                lines.push(Line::from(""));
            }

            // Add frequency distribution if available
            if let Some(ref freq_map) = stats.frequency_map {
                lines.push(
                    Line::from("Frequency Distribution:").style(
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                );

                // Sort by frequency (descending) and take top 20
                let mut freq_vec: Vec<_> = freq_map.iter().collect();
                freq_vec.sort_by(|a, b| b.1.cmp(a.1));

                let max_count = freq_vec.first().map(|(_, c)| **c).unwrap_or(1);

                for (value, count) in freq_vec.iter().take(20) {
                    let bar_width = ((**count as f64 / max_count as f64) * 30.0) as usize;
                    let bar = "█".repeat(bar_width);
                    let display_value = if value.len() > 30 {
                        format!("{}...", &value[..27])
                    } else {
                        value.to_string()
                    };
                    lines.push(Line::from(format!(
                        "  {:30} {} ({})",
                        display_value, bar, count
                    )));
                }

                if freq_vec.len() > 20 {
                    lines.push(
                        Line::from(format!(
                            "  ... and {} more unique values",
                            freq_vec.len() - 20
                        ))
                        .style(Style::default().fg(Color::DarkGray)),
                    );
                }
            }

            lines.push(Line::from(""));
            lines.push(
                Line::from("Press S or Esc to return to results")
                    .style(Style::default().fg(Color::DarkGray)),
            );

            let stats_paragraph = Paragraph::new(Text::from(lines))
                .block(Block::default().borders(Borders::ALL).title(format!(
                    "Column Statistics - {} (S to close)",
                    stats.column_name
                )))
                .wrap(Wrap { trim: false });

            f.render_widget(stats_paragraph, area);
        } else {
            let error = Paragraph::new("No statistics available")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Column Statistics"),
                )
                .style(Style::default().fg(Color::Red));
            f.render_widget(error, area);
        }
    }
}

pub fn run_enhanced_tui_multi(api_url: &str, data_files: Vec<&str>) -> Result<()> {
    let app = if !data_files.is_empty() {
        // Load the first file using existing logic
        let first_file = data_files[0];
        let extension = std::path::Path::new(first_file)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let mut app = match extension.to_lowercase().as_str() {
            "csv" => EnhancedTuiApp::new_with_csv(first_file)?,
            "json" => EnhancedTuiApp::new_with_json(first_file)?,
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported file type: {}. Use .csv or .json files.",
                    first_file
                ))
            }
        };

        // TODO: Load additional files into buffers
        // For now, just note that we have multiple files
        if data_files.len() > 1 {
            app.status_message = format!(
                "{} | {} more file(s) to load - buffer support coming soon",
                app.status_message,
                data_files.len() - 1
            );
        }

        app
    } else {
        EnhancedTuiApp::new(api_url)
    };

    app.run()
}

pub fn run_enhanced_tui(api_url: &str, data_file: Option<&str>) -> Result<()> {
    // For backward compatibility, convert single file to vec and call multi version
    let files = if let Some(file) = data_file {
        vec![file]
    } else {
        vec![]
    };
    run_enhanced_tui_multi(api_url, files)
}
