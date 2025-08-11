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
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use regex::Regex;
use serde_json::Value;
use sql_cli::api_client::{ApiClient, QueryResponse};
use sql_cli::app_state_container::AppStateContainer;
use sql_cli::buffer::{
    AppMode, BufferAPI, BufferManager, ColumnStatistics, ColumnType, EditMode, SortOrder, SortState,
};
use sql_cli::buffer_handler::BufferHandler;
use sql_cli::cache::QueryCache;
use sql_cli::config::Config;
use sql_cli::csv_datasource::CsvApiClient;
use sql_cli::cursor_manager::CursorManager;
use sql_cli::data_analyzer::DataAnalyzer;
use sql_cli::data_exporter::DataExporter;
use sql_cli::debug_info::DebugInfo;
use sql_cli::debug_widget::DebugWidget;
use sql_cli::editor_widget::{BufferAction, EditorAction, EditorWidget};
use sql_cli::help_text::HelpText;
use sql_cli::help_widget::{HelpAction, HelpWidget};
use sql_cli::history::{CommandHistory, HistoryMatch};
use sql_cli::hybrid_parser::HybridParser;
use sql_cli::key_chord_handler::{ChordResult, KeyChordHandler};
use sql_cli::key_dispatcher::KeyDispatcher;
use sql_cli::logging::{get_log_buffer, LogRingBuffer};
use sql_cli::search_modes_widget::{SearchMode, SearchModesAction, SearchModesWidget};
use sql_cli::service_container::ServiceContainer;
use sql_cli::stats_widget::{StatsAction, StatsWidget};
use sql_cli::text_navigation::TextNavigator;
use sql_cli::where_ast::format_where_ast;
use sql_cli::where_parser::WhereParser;
use sql_cli::widget_traits::DebugInfoProvider;
use sql_cli::yank_manager::YankManager;
use std::cmp::Ordering;
use std::io;
use tracing::{debug, info, trace, warn};
use tui_input::{backend::crossterm::EventHandler, Input};

// Using AppMode and EditMode from sql_cli::buffer module

/// Macro for logging state changes with caller information
/// Usage: log_state_change!(self, "field_name", old_value, new_value, "caller_function")
macro_rules! log_state_change {
    ($self:expr, $field:expr, $old:expr, $new:expr, $caller:expr) => {
        if let Some(ref services) = $self.service_container {
            services.debug_service.info(
                "StateManager",
                format!(
                    "[{}] {} changed: {} -> {} (in {})",
                    chrono::Local::now().format("%H:%M:%S%.3f"),
                    $field,
                    $old,
                    $new,
                    $caller
                ),
            );
        }
    };
}

/// Macro for logging state clears/resets
/// Usage: log_state_clear!(self, "field_name", "caller_function")
macro_rules! log_state_clear {
    ($self:expr, $field:expr, $caller:expr) => {
        if let Some(ref services) = $self.service_container {
            services.debug_service.info(
                "StateManager",
                format!(
                    "[{}] {} cleared (in {})",
                    chrono::Local::now().format("%H:%M:%S%.3f"),
                    $field,
                    $caller
                ),
            );
        }
    };
}

#[derive(Clone, PartialEq, Debug)]
enum SelectionMode {
    Row,
    Cell,
}

// Using SortOrder and SortState from sql_cli::buffer module

#[derive(Clone)]
struct FilterState {
    pattern: String,
    regex: Option<Regex>,
    active: bool,
}

#[derive(Clone)]
struct ColumnSearchState {
    pattern: String,
    matching_columns: Vec<(usize, String)>, // (index, column_name)
    current_match: usize,                   // Index into matching_columns
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
    // State container - will gradually take over all state management
    state_container: Option<std::sync::Arc<AppStateContainer>>,
    // Service container for dependency injection
    service_container: Option<ServiceContainer>,

    api_client: ApiClient,
    input: Input,
    cursor_manager: CursorManager, // New: manages cursor/navigation logic
    data_analyzer: DataAnalyzer,   // New: manages data analysis/statistics
    // results: Option<QueryResponse>, // MIGRATED to buffer system
    table_state: TableState,
    show_help: bool, // TODO: Remove once fully migrated to state_container
    sql_parser: SqlParser,
    hybrid_parser: HybridParser,

    // Configuration
    config: Config,

    // Enhanced features
    sort_state: SortState,
    // filter_state: FilterState, // MIGRATED to AppStateContainer
    // search_state: SearchState, // MIGRATED to AppStateContainer
    column_search_state: ColumnSearchState,
    completion_state: CompletionState,
    history_state: HistoryState,
    command_history: CommandHistory,
    scroll_offset: (usize, usize), // (row, col)
    current_column: usize,         // For column-based operations
    sql_highlighter: SqlHighlighter,
    debug_widget: DebugWidget,
    editor_widget: EditorWidget,
    stats_widget: StatsWidget,
    help_widget: HelpWidget,
    search_modes_widget: SearchModesWidget,
    key_chord_handler: KeyChordHandler, // Manages key sequences and history
    key_dispatcher: KeyDispatcher,      // Maps keys to actions
    help_scroll: u16,                   // Scroll offset for help page
    input_scroll_offset: u16,           // Horizontal scroll offset for input

    // Selection and clipboard
    selection_mode: SelectionMode,         // Row or Cell mode
    last_yanked: Option<(String, String)>, // (description, value) of last yanked item

    // Buffer management (new - for supporting multiple files)
    buffer_manager: BufferManager,
    buffer_handler: BufferHandler, // Handles buffer operations like switching
    // Cache
    query_cache: Option<QueryCache>,
    // Data source tracking

    // Undo/redo and kill ring
    undo_stack: Vec<(String, usize)>, // (text, cursor_pos)
    redo_stack: Vec<(String, usize)>,

    // Viewport tracking
    last_visible_rows: usize, // Track the last calculated viewport height

    // Display options
    jump_to_row_input: String, // TODO: Remove once fully migrated to state_container
    log_buffer: Option<LogRingBuffer>, // Ring buffer for debug logs
}

impl EnhancedTuiApp {
    // --- State Container Access ---
    // Helper methods for accessing the state container during migration

    /// Check if help is visible (uses state_container if available, falls back to local field)
    fn is_help_visible(&self) -> bool {
        if let Some(ref container_arc) = self.state_container {
            container_arc.is_help_visible()
        } else {
            self.show_help
        }
    }

    /// Toggle help visibility (uses state_container if available, falls back to local field)
    fn toggle_help(&mut self) {
        let old_value = self.show_help;
        // TODO: Will need Arc<Mutex<>> or interior mutability to modify through Arc
        // For now, just use local field
        self.show_help = !self.show_help;

        // Log the state change
        log_state_change!(self, "show_help", old_value, self.show_help, "toggle_help");
    }

    /// Set help visibility (uses state_container if available, falls back to local field)
    fn set_help_visible(&mut self, visible: bool) {
        let old_value = self.show_help;
        // TODO: Will need Arc<Mutex<>> or interior mutability to modify through Arc
        // For now, just use local field
        self.show_help = visible;

        // Log the state change
        log_state_change!(self, "show_help", old_value, visible, "set_help_visible");
    }

    /// Get jump-to-row input text (uses state_container if available, falls back to local field)
    fn get_jump_to_row_input(&self) -> String {
        if let Some(ref container_arc) = self.state_container {
            container_arc.jump_to_row().input.clone()
        } else {
            self.jump_to_row_input.clone()
        }
    }

    /// Set jump-to-row input text (uses state_container if available, falls back to local field)
    fn set_jump_to_row_input(&mut self, input: String) {
        let old_value = self.jump_to_row_input.clone();
        // TODO: Will need Arc<Mutex<>> for state_container modification
        // For now, just use local field
        self.jump_to_row_input = input.clone();

        // Log the state change
        log_state_change!(
            self,
            "jump_to_row_input",
            old_value,
            input,
            "set_jump_to_row_input"
        );
    }

    /// Clear jump-to-row input (uses state_container if available, falls back to local field)
    fn clear_jump_to_row_input(&mut self) {
        // TODO: Will need Arc<Mutex<>> for state_container modification
        // For now, just use local field
        self.jump_to_row_input.clear();

        // Log the state clear
        log_state_clear!(self, "jump_to_row_input", "clear_jump_to_row_input");
    }

    // --- Buffer Compatibility Layer ---
    // These methods provide a gradual migration path from direct field access to BufferAPI

    /// Get current buffer if available (for reading)
    fn current_buffer(&self) -> Option<&dyn sql_cli::buffer::BufferAPI> {
        self.buffer_manager
            .current()
            .map(|b| b as &dyn sql_cli::buffer::BufferAPI)
    }

    /// Get current buffer (panics if none exists)
    /// Use this when we know a buffer should always exist
    fn buffer(&self) -> &dyn sql_cli::buffer::BufferAPI {
        self.current_buffer()
            .expect("No buffer available - this should not happen")
    }

    // Note: current_buffer_mut removed - use buffer_manager.current_mut() directly

    /// Get current mutable buffer (panics if none exists)
    /// Use this when we know a buffer should always exist
    fn buffer_mut(&mut self) -> &mut sql_cli::buffer::Buffer {
        self.buffer_manager
            .current_mut()
            .expect("No buffer available - this should not happen")
    }

    // Note: edit_mode methods removed - use buffer directly

    // Helper to get input text from buffer or fallback to direct input
    fn get_input_text(&self) -> String {
        // For special modes that use the input field for their own purposes
        match self.buffer().get_mode() {
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                // These modes temporarily use the input field for their patterns
                self.input.value().to_string() // TODO: Migrate to buffer-based input
            }
            _ => {
                // All other modes use the buffer
                self.buffer().get_input_text()
            }
        }
    }

    // Helper to get cursor position from buffer or fallback to direct input
    fn get_input_cursor(&self) -> usize {
        // For special modes that use the input field directly
        match self.buffer().get_mode() {
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                // These modes use the input field for their patterns
                self.input.cursor()
            }
            _ => {
                // All other modes use the buffer
                self.buffer().get_input_cursor_position()
            }
        }
    }

    // Helper to set input text through buffer and sync input field
    fn set_input_text(&mut self, text: String) {
        let old_text = self.buffer().get_input_text();
        let mode = self.buffer().get_mode();

        // Log every input text change with context
        info!(target: "input", "SET_INPUT_TEXT: '{}' -> '{}' (mode: {:?})", 
              if old_text.len() > 50 { format!("{}...", &old_text[..50]) } else { old_text.clone() },
              if text.len() > 50 { format!("{}...", &text[..50]) } else { text.clone() },
              mode);

        self.buffer_mut().set_input_text(text.clone());
        // Also sync cursor position to end of text
        self.buffer_mut().set_input_cursor_position(text.len());

        // Always update the input field for all modes
        // TODO: Eventually migrate special modes to use buffer input
        self.input = tui_input::Input::new(text.clone()).with_cursor(text.len());
    }

    // Helper to set input text with specific cursor position
    fn set_input_text_with_cursor(&mut self, text: String, cursor_pos: usize) {
        let old_text = self.buffer().get_input_text();
        let old_cursor = self.buffer().get_input_cursor_position();
        let mode = self.buffer().get_mode();

        // Log every input text change with cursor position
        info!(target: "input", "SET_INPUT_TEXT_WITH_CURSOR: '{}' (cursor {}) -> '{}' (cursor {}) (mode: {:?})", 
              if old_text.len() > 50 { format!("{}...", &old_text[..50]) } else { old_text.clone() },
              old_cursor,
              if text.len() > 50 { format!("{}...", &text[..50]) } else { text.clone() },
              cursor_pos,
              mode);

        self.buffer_mut().set_input_text(text.clone());
        self.buffer_mut().set_input_cursor_position(cursor_pos);

        // Always update the input field for consistency
        // TODO: Eventually migrate special modes to use buffer input
        self.input = tui_input::Input::new(text).with_cursor(cursor_pos);
    }

    // Helper to clear input
    fn clear_input(&mut self) {
        self.set_input_text(String::new());
    }

    // Helper to handle key events in the input
    fn handle_input_key(&mut self, key: KeyEvent) -> bool {
        // For special modes that handle input directly
        match self.buffer().get_mode() {
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                self.input.handle_event(&Event::Key(key));
                false
            }
            _ => {
                // Route to buffer's input handling
                self.buffer_mut().handle_input_key(key)
            }
        }
    }

    // Helper to get visual cursor position (for rendering)
    fn get_visual_cursor(&self) -> (usize, usize) {
        // Get text and cursor from appropriate source based on mode
        let (text, cursor) = match self.buffer().get_mode() {
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                // Special modes use self.input directly
                (self.input.value().to_string(), self.input.cursor())
            }
            _ => {
                // Other modes use buffer
                (
                    self.buffer().get_input_text(),
                    self.buffer().get_input_cursor_position(),
                )
            }
        };

        let lines: Vec<&str> = text.split('\n').collect();
        let mut current_pos = 0;
        for (row, line) in lines.iter().enumerate() {
            if current_pos + line.len() >= cursor {
                return (row, cursor - current_pos);
            }
            current_pos += line.len() + 1; // +1 for newline
        }
        (0, cursor)
    }

    // Note: mode methods removed - use buffer directly

    // get_filter_state methods MIGRATED - now use state_container.filter()
    // These methods are kept temporarily for fallback compatibility
    fn get_filter_state(&self) -> &FilterState {
        static FALLBACK_FILTER: FilterState = FilterState {
            pattern: String::new(),
            regex: None,
            active: false,
        };
        // This should not be called - prefer state_container.filter()
        &FALLBACK_FILTER
    }

    fn get_filter_state_mut(&mut self) -> &mut FilterState {
        static mut FALLBACK_FILTER: FilterState = FilterState {
            pattern: String::new(),
            regex: None,
            active: false,
        };
        // This should not be called - prefer state_container.filter_mut()
        unsafe { &mut FALLBACK_FILTER }
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

    pub fn new(api_url: &str) -> Self {
        // Load configuration
        let config = Config::load().unwrap_or_else(|_e| {
            // Config loading error - using defaults
            Config::default()
        });

        // Create buffer manager first
        let mut buffer_manager = BufferManager::new();
        let mut buffer = sql_cli::buffer::Buffer::new(1);
        // Sync initial settings from config
        buffer.set_case_insensitive(config.behavior.case_insensitive_default);
        buffer.set_compact_mode(config.display.compact_mode);
        buffer.set_show_row_numbers(config.display.show_row_numbers);
        buffer_manager.add_buffer(buffer);

        // Create a second buffer manager for the state container (temporary during migration)
        let mut container_buffer_manager = BufferManager::new();
        let mut container_buffer = sql_cli::buffer::Buffer::new(1);
        container_buffer.set_case_insensitive(config.behavior.case_insensitive_default);
        container_buffer.set_compact_mode(config.display.compact_mode);
        container_buffer.set_show_row_numbers(config.display.show_row_numbers);
        container_buffer_manager.add_buffer(container_buffer);

        // Initialize state container as Arc
        let state_container = match AppStateContainer::new(container_buffer_manager) {
            Ok(container) => Some(std::sync::Arc::new(container)),
            Err(e) => {
                eprintln!("WARNING: Failed to initialize AppStateContainer: {}", e);
                eprintln!("Falling back to legacy initialization without state container");
                None
            }
        };

        // Initialize service container and help widget
        let (service_container, help_widget) = if let Some(ref state_arc) = state_container {
            let services = ServiceContainer::new(state_arc.clone());

            // Inject debug service into AppStateContainer (now works with RefCell)
            state_arc.set_debug_service(services.debug_service.clone_service());

            // IMPORTANT: Enable the debug service so it actually logs!
            services.enable_debug();

            // Create help widget and set services
            let mut widget = HelpWidget::new();
            widget.set_services(services.clone_for_widget());

            (Some(services), widget)
        } else {
            (None, HelpWidget::new())
        };

        Self {
            state_container,
            service_container,
            api_client: ApiClient::new(api_url),
            input: Input::default(),
            cursor_manager: CursorManager::new(),
            data_analyzer: DataAnalyzer::new(),
            // results: None, // MIGRATED to buffer system
            table_state: TableState::default(),
            show_help: false,
            sql_parser: SqlParser::new(),
            hybrid_parser: HybridParser::new(),
            config: config.clone(),

            sort_state: SortState {
                column: None,
                order: SortOrder::None,
            },
            // filter_state: FilterState { ... }, // MIGRATED to AppStateContainer
            // fuzzy_filter_state: FuzzyFilterState { ... }, // MIGRATED to buffer system
            // search_state: SearchState { // MIGRATED to AppStateContainer
            //     pattern: String::new(),
            //     current_match: None,
            //     matches: Vec::new(),
            //     match_index: 0,
            // },
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
            scroll_offset: (0, 0),
            current_column: 0,
            sql_highlighter: SqlHighlighter::new(),
            debug_widget: DebugWidget::new(),
            editor_widget: EditorWidget::new(),
            stats_widget: StatsWidget::new(),
            help_widget,
            search_modes_widget: SearchModesWidget::new(),
            key_chord_handler: KeyChordHandler::new(),
            key_dispatcher: KeyDispatcher::new(),
            help_scroll: 0,
            input_scroll_offset: 0,
            selection_mode: SelectionMode::Row, // Default to row mode
            last_yanked: None,
            // CSV fields now in Buffer
            buffer_manager,
            buffer_handler: BufferHandler::new(),
            query_cache: QueryCache::new().ok(),
            // Cache fields now in Buffer
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_visible_rows: 30, // Default estimate
            jump_to_row_input: String::new(),
            log_buffer: get_log_buffer(),
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

        // Replace the default buffer with a CSV buffer
        {
            // Clear all buffers and add a CSV buffer
            app.buffer_manager.clear_all();
            let mut buffer = sql_cli::buffer::Buffer::from_csv(
                1,
                std::path::PathBuf::from(csv_path),
                csv_client,
                table_name.clone(),
            );
            // Apply config settings to the buffer - use app's config
            buffer.set_case_insensitive(app.config.behavior.case_insensitive_default);
            buffer.set_compact_mode(app.config.display.compact_mode);
            buffer.set_show_row_numbers(app.config.display.show_row_numbers);

            info!(target: "buffer", "Configured CSV buffer with: compact_mode={}, case_insensitive={}, show_row_numbers={}",
                  app.config.display.compact_mode,
                  app.config.behavior.case_insensitive_default,
                  app.config.display.show_row_numbers);
            app.buffer_manager.add_buffer(buffer);
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
            app.buffer_mut().set_status_message(display_msg);
        }

        // Auto-execute SELECT * FROM table_name to show data immediately (if configured)
        let auto_query = format!("SELECT * FROM {}", table_name);

        // Populate the input field with the query for easy editing
        app.set_input_text(auto_query.clone());

        if app.config.behavior.auto_execute_on_load {
            if let Err(e) = app.execute_query(&auto_query) {
                // If auto-query fails, just log it in status but don't fail the load
                app.buffer_mut().set_status_message(format!(
                    "CSV loaded: table '{}' ({} columns) - Note: {}",
                    table_name,
                    schema.get(&table_name).map(|c| c.len()).unwrap_or(0),
                    e
                ));
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

        // Replace the default buffer with a JSON buffer
        {
            // Clear all buffers and add a JSON buffer
            app.buffer_manager.clear_all();
            let mut buffer = sql_cli::buffer::Buffer::from_json(
                1,
                std::path::PathBuf::from(json_path),
                csv_client,
                table_name.clone(),
            );
            // Apply config settings to the buffer - use app's config
            buffer.set_case_insensitive(app.config.behavior.case_insensitive_default);
            buffer.set_compact_mode(app.config.display.compact_mode);
            buffer.set_show_row_numbers(app.config.display.show_row_numbers);

            info!(target: "buffer", "Configured CSV buffer with: compact_mode={}, case_insensitive={}, show_row_numbers={}",
                  app.config.display.compact_mode,
                  app.config.behavior.case_insensitive_default,
                  app.config.display.show_row_numbers);
            app.buffer_manager.add_buffer(buffer);
        }

        // Buffer state is now initialized

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
            app.buffer_mut().set_status_message(display_msg);
        }

        // Auto-execute SELECT * FROM table_name to show data immediately (if configured)
        let auto_query = format!("SELECT * FROM {}", table_name);

        // Populate the input field with the query for easy editing
        app.set_input_text(auto_query.clone());

        if app.config.behavior.auto_execute_on_load {
            if let Err(e) = app.execute_query(&auto_query) {
                // If auto-query fails, just log it in status but don't fail the load
                app.buffer_mut().set_status_message(format!(
                    "JSON loaded: table '{}' ({} columns) - Note: {}",
                    table_name,
                    schema.get(&table_name).map(|c| c.len()).unwrap_or(0),
                    e
                ));
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
            // Check for debounced actions from search modes widget
            if self.search_modes_widget.is_active() {
                if let Some(action) = self.search_modes_widget.check_debounce() {
                    match action {
                        SearchModesAction::ExecuteDebounced(mode, pattern) => {
                            debug!(target: "search", "Processing ExecuteDebounced action, current_mode={:?}", self.buffer().get_mode());
                            self.execute_search_action(mode, pattern);
                            debug!(target: "search", "After execute_search_action, current_mode={:?}", self.buffer().get_mode());
                        }
                        _ => {}
                    }
                }
            }

            // Use poll with timeout to allow checking for debounced actions
            if event::poll(std::time::Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        // On Windows, filter out key release events - only handle key press
                        // This prevents double-triggering of toggles
                        if key.kind != crossterm::event::KeyEventKind::Press {
                            continue;
                        }

                        let should_exit = match self.buffer().get_mode() {
                            AppMode::Command => self.handle_command_input(key)?,
                            AppMode::Results => self.handle_results_input(key)?,
                            AppMode::Search
                            | AppMode::Filter
                            | AppMode::FuzzyFilter
                            | AppMode::ColumnSearch => self.handle_search_modes_input(key)?,
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
            } else {
                // No event available, but still redraw if we have pending debounced actions
                if self.search_modes_widget.is_active() {
                    terminal.draw(|f| self.ui(f))?;
                }
            }
        }
        Ok(())
    }

    fn handle_command_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Normalize the key for platform differences
        let normalized_key = if let Some(ref state_container) = self.state_container {
            // First normalize the key for platform differences
            let normalized = state_container.normalize_key(key);

            // Get the action that will be performed (if any)
            let action = self
                .key_dispatcher
                .get_command_action(&normalized)
                .map(|s| s.to_string());

            // Log the key press (mutable borrow needed)
            if let Some(ref mut state_container) = self.state_container {
                // Log both the original and normalized key if different
                if normalized != key {
                    state_container
                        .log_key_press(key, Some(format!("normalized to {:?}", normalized)));
                }
                state_container.log_key_press(normalized, action);
            }

            normalized
        } else {
            key
        };

        // NEW: Try editor widget first for high-level actions
        let key_dispatcher = self.key_dispatcher.clone();
        // Handle editor widget actions by splitting the borrow
        let editor_result = if let Some(buffer) = self.buffer_manager.current_mut() {
            self.editor_widget
                .handle_key(normalized_key.clone(), &key_dispatcher, buffer)?
        } else {
            EditorAction::PassToMainApp(normalized_key.clone())
        };

        match editor_result {
            EditorAction::Quit => return Ok(true),
            EditorAction::ExecuteQuery => {
                // Execute the current query - delegate to existing logic for now
                return self.handle_execute_query();
            }
            EditorAction::BufferAction(buffer_action) => {
                return self.handle_buffer_action(buffer_action);
            }
            EditorAction::ExpandAsterisk => {
                return self.handle_expand_asterisk();
            }
            EditorAction::ShowHelp => {
                self.set_help_visible(true);
                self.buffer_mut().set_mode(AppMode::Help);
                return Ok(false);
            }
            EditorAction::ShowDebug => {
                // This is now handled by passing through to original F5 handler
                return Ok(false);
            }
            EditorAction::ShowPrettyQuery => {
                self.show_pretty_query();
                return Ok(false);
            }
            EditorAction::SwitchMode(mode) => {
                if let Some(buffer) = self.buffer_manager.current_mut() {
                    buffer.set_mode(mode.clone());
                }
                // Special handling for History mode - initialize history search
                if mode == AppMode::History {
                    // Use AppStateContainer if available, otherwise fall back to legacy
                    if let Some(ref state_container) = self.state_container {
                        eprintln!("[DEBUG] Using AppStateContainer for history search");
                        let current_input = self.get_input_text();
                        state_container.start_history_search(current_input);
                        let match_count = state_container.history_search().matches.len();
                        self.buffer_mut()
                            .set_status_message(format!("History search: {} matches", match_count));
                    } else {
                        eprintln!("[DEBUG] Using legacy history search");
                        self.history_state.search_query.clear();
                        self.update_history_matches();
                        // Debug: log how many history entries we have
                        let total_entries = self.command_history.get_all().len();
                        self.buffer_mut().set_status_message(format!(
                            "History search: {} total entries",
                            total_entries
                        ));
                    }
                }
                return Ok(false);
            }
            EditorAction::PassToMainApp(_) => {
                // Fall through to original logic below
            }
            EditorAction::Continue => return Ok(false),
        }

        // ORIGINAL LOGIC: Keep all existing logic as fallback

        // Handle Ctrl+R for history search
        if let KeyCode::Char('r') = normalized_key.code {
            if normalized_key.modifiers.contains(KeyModifiers::CONTROL) {
                if let Some(ref state_container) = self.state_container {
                    // Start history search mode
                    let current_input = self.get_input_text();
                    eprintln!(
                        "[DEBUG] Starting history search with input: '{}'",
                        current_input
                    );
                    state_container.start_history_search(current_input);

                    // Check if history search is active
                    let is_active = state_container.is_history_search_active();
                    let match_count = state_container.history_search().matches.len();
                    eprintln!(
                        "[DEBUG] History search active: {}, matches: {}",
                        is_active, match_count
                    );

                    self.buffer_mut().set_mode(AppMode::History);
                    self.buffer_mut().set_status_message(format!(
                        "History search started (Ctrl+R) - {} matches",
                        match_count
                    ));
                    return Ok(false);
                }
            }
        }

        // Store old cursor position
        let old_cursor = self.get_input_cursor();

        // Also log to tracing
        trace!(target: "input", "Key: {:?} Modifiers: {:?}", key.code, key.modifiers);

        // DON'T process chord handler in Command mode - yanking makes no sense when editing queries!
        // The 'y' key should just type 'y' in the query editor.

        // Try dispatcher first for buffer operations and other actions
        if let Some(action) = self.key_dispatcher.get_command_action(&key) {
            match action {
                "quit" => return Ok(true),
                "next_buffer" => {
                    let message = self.buffer_handler.next_buffer(&mut self.buffer_manager);
                    debug!("{}", message);
                    return Ok(false);
                }
                "previous_buffer" => {
                    let message = self
                        .buffer_handler
                        .previous_buffer(&mut self.buffer_manager);
                    debug!("{}", message);
                    return Ok(false);
                }
                "quick_switch_buffer" => {
                    let message = self.buffer_handler.quick_switch(&mut self.buffer_manager);
                    debug!("{}", message);
                    return Ok(false);
                }
                "new_buffer" => {
                    let message = self
                        .buffer_handler
                        .new_buffer(&mut self.buffer_manager, &self.config);
                    debug!("{}", message);
                    return Ok(false);
                }
                "close_buffer" => {
                    let (success, message) =
                        self.buffer_handler.close_buffer(&mut self.buffer_manager);
                    debug!("{}", message);
                    return Ok(!success); // Exit if we couldn't close (only one left)
                }
                "list_buffers" => {
                    let buffer_list = self.buffer_handler.list_buffers(&self.buffer_manager);
                    // For now, just log the list - later we can show a popup
                    for line in &buffer_list {
                        debug!("{}", line);
                    }
                    return Ok(false);
                }
                action if action.starts_with("switch_to_buffer_") => {
                    if let Some(buffer_num_str) = action.strip_prefix("switch_to_buffer_") {
                        if let Ok(buffer_num) = buffer_num_str.parse::<usize>() {
                            let message = self
                                .buffer_handler
                                .switch_to_buffer(&mut self.buffer_manager, buffer_num - 1); // Convert to 0-based
                            debug!("{}", message);
                        }
                    }
                    return Ok(false);
                }
                "expand_asterisk" => {
                    if let Some(buffer) = self.buffer_manager.current_mut() {
                        if buffer.expand_asterisk(&self.hybrid_parser) {
                            // Sync for rendering if needed
                            if buffer.get_edit_mode() == EditMode::SingleLine {
                                let text = buffer.get_input_text();
                                let cursor = buffer.get_input_cursor_position();
                                self.set_input_text_with_cursor(text, cursor);
                            }
                        }
                    }
                    return Ok(false);
                }
                // "move_to_line_start" and "move_to_line_end" now handled by editor_widget
                "delete_word_backward" => {
                    if let Some(buffer) = self.buffer_manager.current_mut() {
                        buffer.save_state_for_undo();
                        buffer.delete_word_backward();
                        // Sync for rendering
                        if buffer.get_edit_mode() == EditMode::SingleLine {
                            let text = buffer.get_input_text();
                            let cursor = buffer.get_input_cursor_position();
                            self.set_input_text_with_cursor(text, cursor);
                            self.cursor_manager.set_position(cursor);
                        }
                    }
                    return Ok(false);
                }
                "delete_word_forward" => {
                    if let Some(buffer) = self.buffer_manager.current_mut() {
                        buffer.save_state_for_undo();
                        buffer.delete_word_forward();
                        // Sync for rendering
                        if buffer.get_edit_mode() == EditMode::SingleLine {
                            let text = buffer.get_input_text();
                            let cursor = buffer.get_input_cursor_position();
                            self.set_input_text_with_cursor(text, cursor);
                            self.cursor_manager.set_position(cursor);
                        }
                    }
                    return Ok(false);
                }
                "kill_line" => {
                    if let Some(buffer) = self.buffer_manager.current_mut() {
                        buffer.save_state_for_undo();
                        buffer.kill_line();
                        // Sync for rendering
                        if buffer.get_edit_mode() == EditMode::SingleLine {
                            let text = buffer.get_input_text();
                            let cursor = buffer.get_input_cursor_position();
                            self.set_input_text_with_cursor(text, cursor);
                            self.cursor_manager.set_position(cursor);
                        }
                    }
                    return Ok(false);
                }
                "kill_line_backward" => {
                    if let Some(buffer) = self.buffer_manager.current_mut() {
                        buffer.save_state_for_undo();
                        buffer.kill_line_backward();
                        // Sync for rendering
                        if buffer.get_edit_mode() == EditMode::SingleLine {
                            let text = buffer.get_input_text();
                            let cursor = buffer.get_input_cursor_position();
                            self.set_input_text_with_cursor(text, cursor);
                            self.cursor_manager.set_position(cursor);
                        }
                    }
                    return Ok(false);
                }
                "move_word_backward" => {
                    self.move_cursor_word_backward();
                    return Ok(false);
                }
                "move_word_forward" => {
                    self.move_cursor_word_forward();
                    return Ok(false);
                }
                "jump_to_prev_token" => {
                    if let Some(buffer) = self.buffer_manager.current_mut() {
                        buffer.jump_to_prev_token();
                        // Sync for rendering
                        if buffer.get_edit_mode() == EditMode::SingleLine {
                            let text = buffer.get_input_text();
                            let cursor = buffer.get_input_cursor_position();
                            self.set_input_text_with_cursor(text, cursor);
                            self.cursor_manager.set_position(cursor);
                        }
                    }
                    return Ok(false);
                }
                "jump_to_next_token" => {
                    if let Some(buffer) = self.buffer_manager.current_mut() {
                        buffer.jump_to_next_token();
                        // Sync for rendering
                        if buffer.get_edit_mode() == EditMode::SingleLine {
                            let text = buffer.get_input_text();
                            let cursor = buffer.get_input_cursor_position();
                            self.set_input_text_with_cursor(text, cursor);
                            self.cursor_manager.set_position(cursor);
                        }
                    }
                    return Ok(false);
                }
                _ => {} // Fall through to hardcoded handling
            }
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            // KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::ALT) && key.modifiers.contains(KeyModifiers::SHIFT) => {
            //     // Alt+Shift+D - new DataTable buffer (for testing) - disabled during revert
            //     self.new_datatable_buffer();
            // }
            KeyCode::F(1) | KeyCode::Char('?') => {
                // Toggle between Help mode and previous mode
                if self.buffer().get_mode() == AppMode::Help {
                    // Exit help mode
                    let mode = if self.buffer().get_results().is_some() {
                        AppMode::Results
                    } else {
                        AppMode::Command
                    };
                    self.buffer_mut().set_mode(mode);
                    self.set_help_visible(false); // Keep state_container in sync
                    self.help_widget.on_exit();
                } else {
                    // Enter help mode
                    eprintln!("DEBUG: F1 pressed - entering help mode");
                    eprintln!(
                        "DEBUG: service_container is: {}",
                        if self.service_container.is_some() {
                            "Some"
                        } else {
                            "None"
                        }
                    );
                    self.buffer_mut().set_mode(AppMode::Help);
                    self.set_help_visible(true); // Keep state_container in sync
                    self.help_widget.on_enter();
                }
            }
            KeyCode::F(3) => {
                // F3 no longer toggles modes - always stay in single-line mode
                self.buffer_mut().set_status_message(
                    "Multi-line mode has been removed. Use F6 for pretty print.".to_string(),
                );
            }
            KeyCode::F(7) => {
                // F7 - Toggle cache mode or show cache list
                if self.buffer().is_cache_mode() {
                    self.buffer_mut().set_mode(AppMode::CacheList);
                } else {
                    self.buffer_mut().set_mode(AppMode::CacheList);
                }
            }
            KeyCode::Enter => {
                // Always use single-line mode handling
                let query = self.get_input_text().trim().to_string();
                debug!(target: "action", "Executing query: {}", query);

                if !query.is_empty() {
                    // Check for special commands
                    if query == ":help" {
                        self.set_help_visible(true);
                        self.buffer_mut().set_mode(AppMode::Help);
                        self.buffer_mut()
                            .set_status_message("Help Mode - Press ESC to return".to_string());
                    } else if query == ":exit" || query == ":quit" {
                        return Ok(true);
                    } else if query == ":tui" {
                        // Already in TUI mode
                        self.buffer_mut()
                            .set_status_message("Already in TUI mode".to_string());
                    } else if query.starts_with(":cache ") {
                        self.handle_cache_command(&query)?;
                    } else {
                        self.buffer_mut()
                            .set_status_message(format!("Processing query: '{}'", query));
                        self.execute_query(&query)?;
                    }
                } else {
                    self.buffer_mut()
                        .set_status_message("Empty query - please enter a SQL command".to_string());
                }
            }
            KeyCode::Tab => {
                // Tab completion works in both modes
                // Always use single-line completion
                self.apply_completion()
            }
            // Ctrl+R is now handled by the editor widget above
            // History navigation - Ctrl+P or Alt+Up
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Navigate to previous command in history
                // Get history entries first, before mutable borrow
                let history_entries = self.command_history.get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.buffer_manager.current_mut() {
                    if buffer.navigate_history_up(&history_commands) {
                        // Sync the input field with buffer (for now, until we complete migration)
                        let text = buffer.get_input_text();

                        // Debug: show what we got from history
                        let debug_msg = if text.is_empty() {
                            "History navigation returned empty text!".to_string()
                        } else {
                            format!(
                                "History: {}",
                                if text.len() > 50 {
                                    format!("{}...", &text[..50])
                                } else {
                                    text.clone()
                                }
                            )
                        };

                        // Update the appropriate input field based on edit mode
                        // Always use single-line mode
                        self.input = tui_input::Input::new(text.clone()).with_cursor(text.len());
                        self.buffer_mut().set_status_message(debug_msg);
                    }
                }
            }
            // History navigation - Ctrl+N or Alt+Down
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Navigate to next command in history
                // Get history entries first, before mutable borrow
                let history_entries = self.command_history.get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.buffer_manager.current_mut() {
                    if buffer.navigate_history_down(&history_commands) {
                        // Sync the input field with buffer (for now, until we complete migration)
                        let text = buffer.get_input_text();

                        // Update the appropriate input field based on edit mode
                        // Always use single-line mode
                        self.input = tui_input::Input::new(text.clone()).with_cursor(text.len());
                        self.buffer_mut()
                            .set_status_message("Next command from history".to_string());
                    }
                }
            }
            // Alternative: Alt+Up for history previous (in case Ctrl+P is intercepted)
            KeyCode::Up if key.modifiers.contains(KeyModifiers::ALT) => {
                let history_entries = self.command_history.get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.buffer_manager.current_mut() {
                    if buffer.navigate_history_up(&history_commands) {
                        let text = buffer.get_input_text();
                        // Always use single-line mode
                        self.input = tui_input::Input::new(text.clone()).with_cursor(text.len());
                        self.buffer_mut()
                            .set_status_message("Previous command (Alt+Up)".to_string());
                    }
                }
            }
            // Alternative: Alt+Down for history next
            KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => {
                let history_entries = self.command_history.get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.buffer_manager.current_mut() {
                    if buffer.navigate_history_down(&history_commands) {
                        let text = buffer.get_input_text();
                        // Always use single-line mode
                        self.input = tui_input::Input::new(text.clone()).with_cursor(text.len());
                        self.buffer_mut()
                            .set_status_message("Next command (Alt+Down)".to_string());
                    }
                }
            }
            KeyCode::F(8) => {
                // Toggle case-insensitive string comparisons
                let current = self.buffer().is_case_insensitive();
                self.buffer_mut().set_case_insensitive(!current);

                // Update CSV client if in CSV mode
                // Update CSV client if in CSV mode
                if let Some(csv_client) = self.buffer_mut().get_csv_client_mut() {
                    csv_client.set_case_insensitive(!current);
                }

                self.buffer_mut().set_status_message(format!(
                    "Case-insensitive string comparisons: {}",
                    if !current { "ON" } else { "OFF" }
                ));
            }
            KeyCode::F(9) => {
                // F9 as alternative for kill line (for terminals that intercept Ctrl+K)
                self.kill_line();
                let message = if !self.buffer().is_kill_ring_empty() {
                    format!(
                        "Killed to end of line ('{}' saved to kill ring)",
                        self.buffer().get_kill_ring()
                    )
                } else {
                    "Killed to end of line".to_string()
                };
                self.buffer_mut().set_status_message(message);
            }
            KeyCode::F(10) => {
                // F10 as alternative for kill line backward (for consistency with F9)
                self.kill_line_backward();
                let message = if !self.buffer().is_kill_ring_empty() {
                    format!(
                        "Killed to beginning of line ('{}' saved to kill ring)",
                        self.buffer().get_kill_ring()
                    )
                } else {
                    "Killed to beginning of line".to_string()
                };
                self.buffer_mut().set_status_message(message);
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Kill line - delete from cursor to end of line
                self.buffer_mut()
                    .set_status_message("Ctrl+K pressed - killing to end of line".to_string());
                self.kill_line();
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Alternative: Alt+K for kill line (for terminals that intercept Ctrl+K)
                self.buffer_mut()
                    .set_status_message("Alt+K - killing to end of line".to_string());
                self.kill_line();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Kill line backward - delete from cursor to beginning of line
                self.kill_line_backward();
            }
            // Ctrl+Z (undo) now handled by editor_widget
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
                if let Some(buffer) = self.buffer_manager.current_mut() {
                    buffer.jump_to_prev_token();
                    // Sync for rendering
                    if buffer.get_edit_mode() == EditMode::SingleLine {
                        let text = buffer.get_input_text();
                        let cursor = buffer.get_input_cursor_position();
                        self.set_input_text_with_cursor(text, cursor);
                        self.cursor_manager.set_position(cursor);
                    }
                }
            }
            KeyCode::Char(']') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Jump to next SQL token
                if let Some(buffer) = self.buffer_manager.current_mut() {
                    buffer.jump_to_next_token();
                    // Sync for rendering
                    if buffer.get_edit_mode() == EditMode::SingleLine {
                        let text = buffer.get_input_text();
                        let cursor = buffer.get_input_cursor_position();
                        self.set_input_text_with_cursor(text, cursor);
                        self.cursor_manager.set_position(cursor);
                    }
                }
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
            KeyCode::Down
                if self.buffer().get_results().is_some()
                    && self.buffer().get_edit_mode() == EditMode::SingleLine =>
            {
                self.buffer_mut().set_mode(AppMode::Results);
                // Restore previous position or default to 0
                let row = self.buffer().get_last_results_row().unwrap_or(0);
                self.table_state.select(Some(row));

                // Restore the exact scroll offset from when we left
                let last_offset = self.buffer().get_last_scroll_offset();
                self.buffer_mut().set_scroll_offset(last_offset);
            }
            KeyCode::F(5) => {
                // Use the unified debug handler
                self.toggle_debug_mode();
            }
            KeyCode::F(6) => {
                // Pretty print query view
                let query = self.get_input_text();
                if !query.trim().is_empty() {
                    self.debug_widget.generate_pretty_sql(&query);
                    self.buffer_mut().set_mode(AppMode::PrettyQuery);
                    self.buffer_mut().set_status_message(
                        "Pretty query view (press Esc or q to return)".to_string(),
                    );
                } else {
                    self.buffer_mut()
                        .set_status_message("No query to format".to_string());
                }
            }
            _ => {
                // Use the new helper to handle input keys through buffer
                self.handle_input_key(key);

                // Clear completion state when typing other characters
                self.completion_state.suggestions.clear();
                self.completion_state.current_index = 0;

                // Always use single-line completion
                self.handle_completion()
            }
        }

        // Update horizontal scroll if cursor moved
        if self.get_input_cursor() != old_cursor {
            self.update_horizontal_scroll(120); // Assume reasonable terminal width, will be adjusted in render
        }

        Ok(false)
    }

    fn handle_results_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        debug!(
            "handle_results_input: key={:?}, selection_mode={:?}",
            key, self.selection_mode
        );

        // Normalize the key for platform differences
        let normalized_key = if let Some(ref state_container) = self.state_container {
            // First normalize the key for platform differences
            let normalized = state_container.normalize_key(key);

            // Get the action that will be performed (if any)
            let action = self
                .key_dispatcher
                .get_results_action(&normalized)
                .map(|s| s.to_string());

            // Log the key press (mutable borrow needed)
            if let Some(ref mut state_container) = self.state_container {
                // Log both the original and normalized key if different
                if normalized != key {
                    state_container
                        .log_key_press(key, Some(format!("normalized to {:?}", normalized)));
                }
                state_container.log_key_press(normalized, action.clone());
            }

            normalized
        } else {
            key
        };

        // Debug uppercase G specifically
        if matches!(key.code, KeyCode::Char('G')) {
            debug!("Detected uppercase G key press!");
        }

        // In cell mode, skip chord handler for 'y' key - handle it directly
        // Also skip uppercase single-key actions as they're not chords
        let should_skip_chord = (matches!(self.selection_mode, SelectionMode::Cell)
            && matches!(normalized_key.code, KeyCode::Char('y')))
            || matches!(
                normalized_key.code,
                KeyCode::Char('G')
                    | KeyCode::Char('C')
                    | KeyCode::Char('F')
                    | KeyCode::Char('S')
                    | KeyCode::Char('N')
                    | KeyCode::Char('P')
            );

        let chord_result = if should_skip_chord {
            debug!("Skipping chord handler for key {:?}", normalized_key.code);
            // Still log the key press even when skipping chord handler
            self.key_chord_handler.log_key_press(&normalized_key);
            ChordResult::SingleKey(normalized_key.clone())
        } else {
            // Process key through chord handler
            self.key_chord_handler.process_key(normalized_key.clone())
        };

        // Handle chord results
        match chord_result {
            ChordResult::CompleteChord(action) => {
                // Handle completed chord actions
                match action.as_str() {
                    "yank_row" => {
                        self.yank_row();
                        return Ok(false);
                    }
                    "yank_column" => {
                        self.yank_column();
                        return Ok(false);
                    }
                    "yank_all" => {
                        self.yank_all();
                        return Ok(false);
                    }
                    "yank_cell" => {
                        self.yank_cell();
                        return Ok(false);
                    }
                    _ => {
                        // Unknown action, continue with normal key handling
                    }
                }
            }
            ChordResult::PartialChord(description) => {
                // Update status to show chord mode
                self.buffer_mut().set_status_message(description);
                return Ok(false);
            }
            ChordResult::Cancelled => {
                self.buffer_mut()
                    .set_status_message("Chord cancelled".to_string());
                return Ok(false);
            }
            ChordResult::SingleKey(_) => {
                // Continue with normal key handling
            }
        }

        // Use dispatcher to get action first
        if let Some(action) = self.key_dispatcher.get_results_action(&normalized_key) {
            debug!(
                "Dispatcher returned action '{}' for key {:?}",
                action, normalized_key
            );
            match action {
                "quit" => return Ok(true),
                "exit_results_mode" => {
                    // Save current position before switching to Command mode
                    if let Some(selected) = self.table_state.selected() {
                        self.buffer_mut().set_last_results_row(Some(selected));
                        let scroll_offset = self.buffer().get_scroll_offset();
                        self.buffer_mut().set_last_scroll_offset(scroll_offset);
                    }

                    // Restore the last executed query to input_text for editing
                    let last_query = self.buffer().get_last_query();
                    let current_input = self.buffer().get_input_text();
                    debug!(target: "mode", "Exiting Results mode: current input_text='{}', last_query='{}'", current_input, last_query);

                    if !last_query.is_empty() {
                        debug!(target: "buffer", "Restoring last_query to input_text: '{}'", last_query);
                        self.buffer_mut().set_input_text(last_query.clone());
                        self.buffer_mut()
                            .set_input_cursor_position(last_query.len());
                        self.input =
                            tui_input::Input::new(last_query.clone()).with_cursor(last_query.len());
                    } else if !current_input.is_empty() {
                        debug!(target: "buffer", "No last_query but input_text has content, keeping: '{}'", current_input);
                    } else {
                        debug!(target: "buffer", "No last_query to restore when exiting Results mode");
                    }

                    debug!(target: "mode", "Switching from Results to Command mode");
                    self.buffer_mut().set_mode(AppMode::Command);
                    self.table_state.select(None);
                }
                "next_row" => self.next_row(),
                "previous_row" => self.previous_row(),
                "move_column_left" => self.move_column_left(),
                "move_column_right" => self.move_column_right(),
                "goto_first_row" => self.goto_first_row(),
                "goto_last_row" => {
                    debug!("Executing goto_last_row action");
                    self.goto_last_row();
                }
                "goto_first_column" => self.goto_first_column(),
                "goto_last_column" => self.goto_last_column(),
                "page_up" => self.page_up(),
                "page_down" => self.page_down(),
                "start_search" => {
                    self.enter_search_mode(SearchMode::Search);
                }
                "start_column_search" => {
                    self.enter_search_mode(SearchMode::ColumnSearch);
                }
                "start_filter" => {
                    self.enter_search_mode(SearchMode::Filter);
                }
                "start_fuzzy_filter" => {
                    self.enter_search_mode(SearchMode::FuzzyFilter);
                }
                "sort_by_column" => self.sort_by_column(self.buffer().get_current_column()),
                "show_column_stats" => self.calculate_column_statistics(),
                "next_search_match" => self.next_search_match(),
                "previous_search_match" => self.previous_search_match(),
                "toggle_compact_mode" => {
                    let current_mode = self.buffer().is_compact_mode();
                    self.buffer_mut().set_compact_mode(!current_mode);
                    let message = if !current_mode {
                        "Compact mode: ON (reduced padding, more columns visible)".to_string()
                    } else {
                        "Compact mode: OFF (normal padding)".to_string()
                    };
                    self.buffer_mut().set_status_message(message);
                }
                "toggle_row_numbers" => {
                    let current_mode = self.buffer().is_show_row_numbers();
                    self.buffer_mut().set_show_row_numbers(!current_mode);
                    let message = if !current_mode {
                        "Row numbers: ON".to_string()
                    } else {
                        "Row numbers: OFF".to_string()
                    };
                    self.buffer_mut().set_status_message(message);
                }
                "jump_to_row" => {
                    self.buffer_mut().set_mode(AppMode::JumpToRow);
                    self.clear_jump_to_row_input();
                    self.buffer_mut()
                        .set_status_message("Enter row number:".to_string());
                }
                "pin_column" => self.toggle_column_pin(),
                "clear_pins" => self.clear_all_pinned_columns(),
                "toggle_selection_mode" => {
                    self.selection_mode = match self.selection_mode {
                        SelectionMode::Row => {
                            self.buffer_mut().set_status_message(
                                "Cell mode - Navigate to select individual cells".to_string(),
                            );
                            SelectionMode::Cell
                        }
                        SelectionMode::Cell => {
                            self.buffer_mut().set_status_message(
                                "Row mode - Navigate to select rows".to_string(),
                            );
                            SelectionMode::Row
                        }
                    };
                    return Ok(false); // Return to prevent duplicate handling
                }
                "export_to_csv" => self.export_to_csv(),
                "export_to_json" => self.export_to_json(),
                "toggle_help" => {
                    if self.buffer().get_mode() == AppMode::Help {
                        self.buffer_mut().set_mode(AppMode::Results);
                        self.set_help_visible(false); // Keep state_container in sync
                    } else {
                        self.buffer_mut().set_mode(AppMode::Help);
                        self.set_help_visible(true); // Keep state_container in sync
                    }
                }
                "toggle_debug" => {
                    // Use the unified debug handler
                    self.toggle_debug_mode();
                }
                "toggle_case_insensitive" => {
                    // Toggle case-insensitive string comparisons
                    let current = self.buffer().is_case_insensitive();
                    self.buffer_mut().set_case_insensitive(!current);

                    // Update CSV client if in CSV mode
                    if let Some(csv_client) = self.buffer_mut().get_csv_client_mut() {
                        csv_client.set_case_insensitive(!current);
                    }

                    self.buffer_mut().set_status_message(format!(
                        "Case-insensitive string comparisons: {}",
                        if !current { "ON" } else { "OFF" }
                    ));
                }
                "start_history_search" => {
                    // Switch to Command mode first
                    let last_query = self.buffer().get_last_query();

                    if !last_query.is_empty() {
                        self.buffer_mut().set_input_text(last_query.clone());
                        self.buffer_mut()
                            .set_input_cursor_position(last_query.len());
                        self.input =
                            tui_input::Input::new(last_query.clone()).with_cursor(last_query.len());
                    }

                    self.buffer_mut().set_mode(AppMode::Command);
                    self.table_state.select(None);

                    // Start history search
                    if let Some(ref state_container) = self.state_container {
                        let current_input = self.get_input_text();
                        state_container.start_history_search(current_input);
                        let match_count = state_container.history_search().matches.len();
                        self.buffer_mut()
                            .set_status_message(format!("History search: {} matches", match_count));

                        // Switch to History mode to show the search interface
                        self.buffer_mut().set_mode(AppMode::History);
                    }
                }
                _ => {
                    // Action not recognized, continue to handle key directly
                }
            }
        }

        // Fall back to direct key handling for special cases not in dispatcher
        match key.code {
            KeyCode::Char(' ') => {
                // Toggle viewport lock with Space
                let current_lock = self.buffer().is_viewport_lock();
                self.buffer_mut().set_viewport_lock(!current_lock);
                if self.buffer().is_viewport_lock() {
                    // Lock to current position in viewport (middle of screen)
                    let visible_rows = self.buffer().get_last_visible_rows();
                    self.buffer_mut()
                        .set_viewport_lock_row(Some(visible_rows / 2));
                    self.buffer_mut().set_status_message(format!(
                        "Viewport lock: ON (anchored at row {} of viewport)",
                        visible_rows / 2 + 1
                    ));
                } else {
                    self.buffer_mut().set_viewport_lock_row(None);
                    self.buffer_mut()
                        .set_status_message("Viewport lock: OFF (normal scrolling)".to_string());
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
            // Search functionality is handled by dispatcher above
            // Removed duplicate handlers for search keys (/, \)
            KeyCode::Char('n') => {
                self.next_search_match();
            }
            KeyCode::Char('N') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Only for search navigation when Shift is held
                if !self.buffer().get_search_pattern().is_empty() {
                    self.previous_search_match();
                } else {
                    // Toggle row numbers display
                    let current = self.buffer().is_show_row_numbers();
                    self.buffer_mut().set_show_row_numbers(!current);
                    let message = if !current {
                        "Row numbers: ON (showing line numbers)".to_string()
                    } else {
                        "Row numbers: OFF".to_string()
                    };
                    self.buffer_mut().set_status_message(message);
                    // Recalculate column widths with new mode
                    self.calculate_optimal_column_widths();
                }
            }
            // Filter functionality is handled by dispatcher above
            // Removed duplicate handlers for filter keys (F, f)
            // Sort functionality (lowercase s)
            KeyCode::Char('s')
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                self.sort_by_column(self.buffer().get_current_column());
            }
            // Column statistics (uppercase S)
            KeyCode::Char('S') | KeyCode::Char('s')
                if key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                self.calculate_column_statistics();
            }
            // Clipboard operations (vim-like yank)
            KeyCode::Char('y') => {
                debug!("'y' key pressed - selection_mode={:?}", self.selection_mode);
                match self.selection_mode {
                    SelectionMode::Cell => {
                        // In cell mode, single 'y' yanks the cell directly
                        debug!("Yanking cell in cell selection mode");
                        self.buffer_mut()
                            .set_status_message("Yanking cell...".to_string());
                        self.yank_cell();
                        // Status message will be set by yank_cell
                    }
                    SelectionMode::Row => {
                        // In row mode, 'y' is handled by chord handler (yy, yc, ya)
                        // The chord handler will process the key sequence
                        debug!("'y' pressed in row mode - waiting for chord completion");
                        self.buffer_mut().set_status_message(
                            "Press second key for chord: yy=row, yc=column, ya=all, yv=cell"
                                .to_string(),
                        );
                    }
                }
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
                self.set_help_visible(true);
                self.buffer_mut().set_mode(AppMode::Help);
                self.help_widget.on_enter();
            }
            _ => {
                // Other keys handled normally
            }
        }
        Ok(false)
    }

    fn execute_search_action(&mut self, mode: SearchMode, pattern: String) {
        debug!(target: "search", "execute_search_action called: mode={:?}, pattern='{}', current_app_mode={:?}", mode, pattern, self.buffer().get_mode());
        match mode {
            SearchMode::Search => {
                debug!(target: "search", "Executing search with pattern: '{}', app_mode={:?}", pattern, self.buffer().get_mode());
                debug!(target: "search", "Search: current results count={}", 
                       self.buffer().get_results().map(|r| r.data.len()).unwrap_or(0));

                // Set search pattern in AppStateContainer if available
                if let Some(ref state_container) = self.state_container {
                    state_container.start_search(pattern.clone());
                }

                self.buffer_mut().set_search_pattern(pattern);
                self.perform_search();
                let matches_count = if let Some(ref state_container) = self.state_container {
                    state_container.search().matches.len()
                } else {
                    0 // Fallback when state_container not available
                };
                debug!(target: "search", "After perform_search, app_mode={:?}, matches_found={}", 
                       self.buffer().get_mode(),
                       matches_count);
            }
            SearchMode::Filter => {
                debug!(target: "search", "Executing filter with pattern: '{}', app_mode={:?}", pattern, self.buffer().get_mode());
                debug!(target: "search", "Filter: case_insensitive={}, current results count={}", 
                       self.buffer().is_case_insensitive(),
                       self.buffer().get_results().map(|r| r.data.len()).unwrap_or(0));
                self.buffer_mut().set_filter_pattern(pattern.clone());
                if let Some(ref state_container) = self.state_container {
                    let mut filter = state_container.filter_mut();
                    filter.pattern = pattern.clone();
                    filter.is_active = true;
                } else {
                    // Fallback when state_container not available
                    // This shouldn't happen in normal operation
                    eprintln!("[WARNING] FilterState migration: state_container not available");
                }
                self.apply_filter();
                debug!(target: "search", "After apply_filter, app_mode={:?}, filtered_count={}", 
                       self.buffer().get_mode(),
                       self.buffer().get_filtered_data().map(|d| d.len()).unwrap_or(0));
            }
            SearchMode::FuzzyFilter => {
                debug!(target: "search", "Executing fuzzy filter with pattern: '{}', app_mode={:?}", pattern, self.buffer().get_mode());
                debug!(target: "search", "FuzzyFilter: current results count={}", 
                       self.buffer().get_results().map(|r| r.data.len()).unwrap_or(0));
                self.buffer_mut().set_fuzzy_filter_pattern(pattern);
                self.apply_fuzzy_filter();
                let indices_count = self.buffer().get_fuzzy_filter_indices().len();
                debug!(target: "search", "After apply_fuzzy_filter, app_mode={:?}, matched_indices={}", 
                       self.buffer().get_mode(), indices_count);
            }
            SearchMode::ColumnSearch => {
                debug!(target: "search", "Executing column search with pattern: '{}', app_mode={:?}", pattern, self.buffer().get_mode());
                self.buffer_mut().set_column_search_pattern(pattern.clone());
                self.column_search_state.pattern = pattern;
                self.search_columns();

                // IMPORTANT: Ensure we stay in ColumnSearch mode after search
                if self.buffer().get_mode() != AppMode::ColumnSearch {
                    debug!(target: "search", "WARNING: Mode changed after search_columns, restoring to ColumnSearch");
                    self.buffer_mut().set_mode(AppMode::ColumnSearch);
                }
                debug!(target: "search", "After search_columns, app_mode={:?}", self.buffer().get_mode());
            }
        }
    }

    fn enter_search_mode(&mut self, mode: SearchMode) {
        debug!(target: "search", "enter_search_mode called for {:?}, current_mode={:?}, input_text='{}'", 
               mode, self.buffer().get_mode(), self.buffer().get_input_text());

        // Get the SQL text based on the current mode
        let current_sql = if self.buffer().get_mode() == AppMode::Results {
            // In Results mode, use the last executed query
            let last_query = self.buffer().get_last_query();
            if !last_query.is_empty() {
                debug!("Using last_query for search mode: '{}'", last_query);
                last_query
            } else {
                // This shouldn't happen if we're properly saving queries
                warn!("No last_query found when entering search mode from Results!");
                String::new()
            }
        } else {
            // In Command mode, use the current input text
            self.get_input_text()
        };

        let cursor_pos = current_sql.len();

        debug!(
            "Entering {} mode, saving SQL: '{}', cursor: {}",
            mode.title(),
            current_sql,
            cursor_pos
        );

        // Initialize the widget with saved state
        self.search_modes_widget
            .enter_mode(mode.clone(), current_sql, cursor_pos);

        // Set the app mode
        debug!(target: "mode", "Setting app mode from {:?} to {:?}", self.buffer().get_mode(), mode.to_app_mode());
        self.buffer_mut().set_mode(mode.to_app_mode());

        // Clear patterns
        match mode {
            SearchMode::Search => {
                // Clear search in AppStateContainer if available
                if let Some(ref state_container) = self.state_container {
                    state_container.clear_search();
                }
                self.buffer_mut().set_search_pattern(String::new());
            }
            SearchMode::Filter => {
                self.buffer_mut().set_filter_pattern(String::new());
                if let Some(ref state_container) = self.state_container {
                    state_container.filter_mut().clear();
                } else {
                    // Fallback when state_container not available
                    eprintln!(
                        "[WARNING] FilterState migration: state_container not available for clear"
                    );
                }
            }
            SearchMode::FuzzyFilter => {
                self.buffer_mut().set_fuzzy_filter_pattern(String::new());
                self.buffer_mut().set_fuzzy_filter_indices(Vec::new());
                self.buffer_mut().set_fuzzy_filter_active(false);
            }
            SearchMode::ColumnSearch => {
                self.buffer_mut().set_column_search_pattern(String::new());
                self.buffer_mut().set_column_search_matches(Vec::new());
                self.buffer_mut().set_column_search_current_match(0);
                self.column_search_state.pattern.clear();
            }
        }

        // Clear input field for search mode use
        self.input = tui_input::Input::default();
    }

    fn handle_search_modes_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        let action = self.search_modes_widget.handle_key(key);

        match action {
            SearchModesAction::Continue => {
                // No pattern change, nothing to do
            }
            SearchModesAction::InputChanged(mode, pattern) => {
                // Pattern changed, update UI but don't apply filter yet (will be debounced)
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());

                // Update the stored pattern
                match mode {
                    SearchMode::Search => {
                        self.buffer_mut().set_search_pattern(pattern);
                    }
                    SearchMode::Filter => {
                        self.buffer_mut().set_filter_pattern(pattern.clone());
                        if let Some(ref state_container) = self.state_container {
                            let mut filter = state_container.filter_mut();
                            filter.pattern = pattern.clone();
                            filter.is_active = true;
                        } else {
                            // Fallback when state_container not available
                            eprintln!("[WARNING] FilterState migration: state_container not available in Set Filter");
                        }
                    }
                    SearchMode::FuzzyFilter => {
                        self.buffer_mut().set_fuzzy_filter_pattern(pattern);
                    }
                    SearchMode::ColumnSearch => {
                        self.buffer_mut().set_column_search_pattern(pattern.clone());
                        self.column_search_state.pattern = pattern;
                    }
                }
            }
            SearchModesAction::ExecuteDebounced(mode, pattern) => {
                // Execute the search but DON'T exit the mode - stay in search mode
                // This is for debounced typing updates
                self.execute_search_action(mode, pattern);
                // Don't exit! User is still typing/searching
            }
            SearchModesAction::Apply(mode, pattern) => {
                debug!(target: "search", "Apply action triggered for {:?} with pattern '{}'", mode, pattern);
                // Apply the filter/search with the pattern
                match mode {
                    SearchMode::Search => {
                        debug!(target: "search", "Search Apply: Applying search with pattern '{}'", pattern);
                        self.buffer_mut().set_search_pattern(pattern);
                        self.perform_search();
                        debug!(target: "search", "Search Apply: last_query='{}', will restore saved SQL from widget", self.buffer().get_last_query());
                    }
                    SearchMode::Filter => {
                        debug!(target: "search", "Filter Apply: Applying filter with pattern '{}'", pattern);
                        self.buffer_mut().set_filter_pattern(pattern.clone());
                        if let Some(ref state_container) = self.state_container {
                            let mut filter = state_container.filter_mut();
                            filter.pattern = pattern.clone();
                            filter.is_active = true;
                        } else {
                            // Fallback when state_container not available
                            eprintln!("[WARNING] FilterState migration: state_container not available in Filter Apply");
                        }
                        self.apply_filter();
                        debug!(target: "search", "Filter Apply: last_query='{}', will restore saved SQL from widget", self.buffer().get_last_query());
                    }
                    SearchMode::FuzzyFilter => {
                        debug!(target: "search", "FuzzyFilter Apply: Applying filter with pattern '{}'", pattern);
                        self.buffer_mut().set_fuzzy_filter_pattern(pattern);
                        self.apply_fuzzy_filter();
                        debug!(target: "search", "FuzzyFilter Apply: last_query='{}', will restore saved SQL from widget", self.buffer().get_last_query());
                    }
                    SearchMode::ColumnSearch => {
                        // For column search, Apply (Enter key) jumps to the current match and exits
                        if !self.column_search_state.matching_columns.is_empty() {
                            let current_match = self.column_search_state.current_match;
                            let (col_idx, col_name) =
                                self.column_search_state.matching_columns[current_match].clone();
                            self.current_column = col_idx;
                            self.buffer_mut().set_current_column(col_idx);
                            self.buffer_mut()
                                .set_status_message(format!("Jumped to column: {}", col_name));
                        }

                        // IMPORTANT: Don't modify input_text when exiting column search!
                        // The widget will restore the original SQL that was saved when entering the mode
                        debug!(target: "search", "ColumnSearch Apply: Exiting without modifying input_text");
                        debug!(target: "search", "ColumnSearch Apply: last_query='{}', will restore saved SQL from widget", self.buffer().get_last_query());
                        // Note: We'll exit the mode below and the widget will restore the saved SQL
                    }
                }

                // Exit search mode and return to Results (except for certain cases)
                // For ColumnSearch, we DO want to exit on Apply (Enter key)
                if let Some((sql, cursor)) = self.search_modes_widget.exit_mode() {
                    debug!(target: "search", "Exiting search mode. Original SQL was: '{}', cursor: {}", sql, cursor);
                    debug!(target: "buffer", "Returning to Results mode, preserving last_query: '{}'", 
                           self.buffer().get_last_query());

                    // IMPORTANT: Restore the saved SQL to input_text!
                    // This is the SQL that was saved when we entered the search mode
                    if !sql.is_empty() {
                        debug!(target: "search", "Restoring saved SQL to input_text: '{}'", sql);
                        self.buffer_mut().set_input_text(sql.clone());
                        self.buffer_mut().set_input_cursor_position(cursor);
                        self.input = tui_input::Input::new(sql).with_cursor(cursor);
                    } else {
                        debug!(target: "search", "No saved SQL to restore, keeping input_text as is");
                    }

                    // Switch back to Results mode
                    self.buffer_mut().set_mode(AppMode::Results);

                    // Show status message
                    let filter_msg = match mode {
                        SearchMode::FuzzyFilter => {
                            let query = self.buffer().get_last_query();
                            format!(
                                "Fuzzy filter applied. Query: '{}'. Press 'f' again to modify.",
                                if query.len() > 30 {
                                    format!("{}...", &query[..30])
                                } else {
                                    query
                                }
                            )
                        }
                        SearchMode::Filter => {
                            "Filter applied. Press 'F' again to modify.".to_string()
                        }
                        SearchMode::Search => "Search applied.".to_string(),
                        SearchMode::ColumnSearch => "Column search complete.".to_string(),
                    };
                    self.buffer_mut().set_status_message(filter_msg);
                } else {
                    self.buffer_mut().set_mode(AppMode::Results);
                }
            }
            SearchModesAction::Cancel => {
                // Clear the filter and restore original SQL
                match self.buffer().get_mode() {
                    AppMode::FuzzyFilter => {
                        // Clear fuzzy filter
                        self.buffer_mut().set_fuzzy_filter_pattern(String::new());
                        self.buffer_mut().set_fuzzy_filter_indices(Vec::new());
                        self.buffer_mut().set_fuzzy_filter_active(false);
                    }
                    AppMode::Filter => {
                        // Clear both local and buffer filter state
                        debug!(target: "search", "Filter Cancel: Clearing filter pattern and state");
                        if let Some(ref state_container) = self.state_container {
                            state_container.filter_mut().clear();
                        } else {
                            // Fallback when state_container not available
                            eprintln!("[WARNING] FilterState migration: state_container not available in Filter Cancel");
                        }
                        self.buffer_mut().set_filter_pattern(String::new());
                        self.buffer_mut().set_filter_active(false);
                        // Re-apply empty filter to restore all results
                        self.apply_filter();
                    }
                    AppMode::ColumnSearch => {
                        // Clear column search state
                        self.buffer_mut().set_column_search_pattern(String::new());
                        self.buffer_mut().set_column_search_matches(Vec::new());
                        self.buffer_mut().set_column_search_current_match(0);
                        self.column_search_state.pattern.clear();
                        self.column_search_state.matching_columns.clear();
                        self.column_search_state.current_match = 0;

                        // IMPORTANT: Don't modify input_text when cancelling column search!
                        // The widget will restore the original SQL that was saved when entering the mode
                        debug!(target: "search", "ColumnSearch Cancel: Exiting without modifying input_text");
                        debug!(target: "search", "ColumnSearch Cancel: last_query='{}', will restore saved SQL from widget", self.buffer().get_last_query());
                    }
                    _ => {}
                }

                // Exit mode and restore the saved SQL
                if let Some((sql, cursor)) = self.search_modes_widget.exit_mode() {
                    debug!(target: "search", "Cancel: Restoring saved SQL: '{}', cursor: {}", sql, cursor);
                    if !sql.is_empty() {
                        self.buffer_mut().set_input_text(sql.clone());
                        self.buffer_mut().set_input_cursor_position(cursor);
                        self.input = tui_input::Input::new(sql).with_cursor(cursor);
                    }
                } else {
                    debug!(target: "search", "Cancel: No saved SQL from widget");
                }

                // Switch back to Results mode
                self.buffer_mut().set_mode(AppMode::Results);
            }
            SearchModesAction::NextMatch => {
                debug!(target: "search", "NextMatch action, current_mode={:?}, widget_mode={:?}", 
                       self.buffer().get_mode(), self.search_modes_widget.current_mode());

                // Check both buffer mode and widget mode for consistency
                if self.buffer().get_mode() == AppMode::ColumnSearch
                    || self.search_modes_widget.current_mode() == Some(SearchMode::ColumnSearch)
                {
                    debug!(target: "search", "Calling next_column_match");
                    // Ensure mode is correctly set
                    if self.buffer().get_mode() != AppMode::ColumnSearch {
                        debug!(target: "search", "WARNING: Mode mismatch - fixing");
                        self.buffer_mut().set_mode(AppMode::ColumnSearch);
                    }
                    self.next_column_match();
                } else {
                    debug!(target: "search", "Not in ColumnSearch mode, skipping next_column_match");
                }
            }
            SearchModesAction::PreviousMatch => {
                debug!(target: "search", "PreviousMatch action, current_mode={:?}, widget_mode={:?}", 
                       self.buffer().get_mode(), self.search_modes_widget.current_mode());

                // Check both buffer mode and widget mode for consistency
                if self.buffer().get_mode() == AppMode::ColumnSearch
                    || self.search_modes_widget.current_mode() == Some(SearchMode::ColumnSearch)
                {
                    debug!(target: "search", "Calling previous_column_match");
                    // Ensure mode is correctly set
                    if self.buffer().get_mode() != AppMode::ColumnSearch {
                        debug!(target: "search", "WARNING: Mode mismatch - fixing");
                        self.buffer_mut().set_mode(AppMode::ColumnSearch);
                    }
                    self.previous_column_match();
                } else {
                    debug!(target: "search", "Not in ColumnSearch mode, skipping previous_column_match");
                }
            }
            SearchModesAction::PassThrough => {}
        }

        Ok(false)
    }

    fn handle_search_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.buffer_mut().pop_undo() {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                }
                self.buffer_mut().set_mode(AppMode::Results);
            }
            KeyCode::Enter => {
                self.perform_search();
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.buffer_mut().pop_undo() {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                }
                self.buffer_mut().set_mode(AppMode::Results);
            }
            KeyCode::Backspace => {
                {
                    let mut pattern = self.buffer().get_search_pattern();
                    pattern.pop();
                    self.buffer_mut().set_search_pattern(pattern);
                };
                // Update input for rendering
                let pattern = self.buffer().get_search_pattern();
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());
            }
            KeyCode::Char(c) => {
                {
                    let mut pattern = self.buffer().get_search_pattern();
                    pattern.push(c);
                    self.buffer_mut().set_search_pattern(pattern);
                }
                // Update input for rendering
                let pattern = self.buffer().get_search_pattern();
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_filter_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.buffer_mut().pop_undo() {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                }
                self.buffer_mut().set_mode(AppMode::Results);
            }
            KeyCode::Enter => {
                self.apply_filter();
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.buffer_mut().pop_undo() {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                }
                self.buffer_mut().set_mode(AppMode::Results);
            }
            KeyCode::Backspace => {
                let pattern = if let Some(ref state_container) = self.state_container {
                    let mut filter = state_container.filter_mut();
                    filter.pattern.pop();
                    filter.pattern.clone()
                } else {
                    self.get_filter_state_mut().pattern.pop();
                    self.get_filter_state().pattern.clone()
                };
                // Update input for rendering
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());
            }
            KeyCode::Char(c) => {
                let pattern = if let Some(ref state_container) = self.state_container {
                    let mut filter = state_container.filter_mut();
                    filter.pattern.push(c);
                    filter.pattern.clone()
                } else {
                    self.get_filter_state_mut().pattern.push(c);
                    self.get_filter_state().pattern.clone()
                };
                // Update input for rendering
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_fuzzy_filter_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Clear fuzzy filter and return to results
                self.buffer_mut().set_fuzzy_filter_active(false);
                self.buffer_mut().set_fuzzy_filter_pattern(String::new());
                self.buffer_mut().set_fuzzy_filter_indices(Vec::new());
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.buffer_mut().pop_undo() {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                }
                self.buffer_mut().set_mode(AppMode::Results);
                self.buffer_mut()
                    .set_status_message("Fuzzy filter cleared".to_string());
            }
            KeyCode::Enter => {
                // Apply fuzzy filter and return to results
                if !self.buffer().get_fuzzy_filter_pattern().is_empty() {
                    self.apply_fuzzy_filter();
                    self.buffer_mut().set_fuzzy_filter_active(true);
                }
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.buffer_mut().pop_undo() {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                }
                self.buffer_mut().set_mode(AppMode::Results);
            }
            KeyCode::Backspace => {
                {
                    let mut pattern = self.buffer().get_fuzzy_filter_pattern();
                    pattern.pop();
                    self.buffer_mut().set_fuzzy_filter_pattern(pattern);
                };
                // Update input for rendering
                let pattern = self.buffer().get_fuzzy_filter_pattern();
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());
                // Don't apply filter here - let the debouncer handle it
                // Only clear if pattern is empty
                if self.buffer().get_fuzzy_filter_pattern().is_empty() {
                    self.buffer_mut().set_fuzzy_filter_indices(Vec::new());
                    self.buffer_mut().set_fuzzy_filter_active(false);
                }
            }
            KeyCode::Char(c) => {
                {
                    let mut pattern = self.buffer().get_fuzzy_filter_pattern();
                    pattern.push(c);
                    self.buffer_mut().set_fuzzy_filter_pattern(pattern);
                };
                // Update input for rendering
                let pattern = self.buffer().get_fuzzy_filter_pattern();
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());
                // Don't apply filter here - let the debouncer handle it
                // The search widget's debounced execute_search will call apply_fuzzy_filter()
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_column_search_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Restore original SQL query from undo stack FIRST
                if let Some((original_query, cursor_pos)) = self.buffer_mut().pop_undo() {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                } else {
                    // Fallback: restore from buffer's stored text if undo fails
                    let text = self.buffer().get_input_text();
                    let cursor = self.buffer().get_input_cursor_position();
                    self.input = tui_input::Input::new(text.clone()).with_cursor(cursor);
                }

                // Cancel column search and return to results
                self.buffer_mut().set_mode(AppMode::Results);
                self.buffer_mut().set_column_search_pattern(String::new());
                self.buffer_mut().set_column_search_matches(Vec::new());
                self.buffer_mut()
                    .set_status_message("Column search cancelled".to_string());
            }
            KeyCode::Enter => {
                // Jump to first matching column
                if !self.buffer().get_column_search_matches().clone().is_empty() {
                    let (column_index, column_name) =
                        self.buffer().get_column_search_matches().clone()
                            [self.buffer().get_column_search_current_match()]
                        .clone();
                    self.buffer_mut().set_current_column(column_index);
                    self.buffer_mut()
                        .set_status_message(format!("Jumped to column: {}", column_name));
                } else {
                    self.buffer_mut()
                        .set_status_message("No matching columns found".to_string());
                }

                // Restore original SQL query from undo stack
                if let Some((original_query, cursor_pos)) = self.buffer_mut().pop_undo() {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                } else {
                    // Fallback: restore from buffer's stored text if undo fails
                    let text = self.buffer().get_input_text();
                    let cursor = self.buffer().get_input_cursor_position();
                    self.input = tui_input::Input::new(text.clone()).with_cursor(cursor);
                }

                self.buffer_mut().set_mode(AppMode::Results);
            }
            KeyCode::Tab => {
                // Next match (Tab only, not 'n' to allow typing 'n' in search)
                if !self.buffer().get_column_search_matches().clone().is_empty() {
                    let matches_len = self.buffer().get_column_search_matches().clone().len();
                    let current = self.buffer().get_column_search_current_match();
                    self.buffer_mut()
                        .set_column_search_current_match((current + 1) % matches_len);
                    let (column_index, column_name) =
                        self.buffer().get_column_search_matches().clone()
                            [self.buffer().get_column_search_current_match()]
                        .clone();
                    let current_match = self.buffer().get_column_search_current_match() + 1;
                    let total_matches = self.buffer().get_column_search_matches().clone().len();
                    self.buffer_mut().set_current_column(column_index);
                    self.buffer_mut().set_status_message(format!(
                        "Column {} of {}: {}",
                        current_match, total_matches, column_name
                    ));
                }
            }
            KeyCode::BackTab => {
                // Previous match (Shift+Tab only, not 'N' to allow typing 'N' in search)
                if !self.buffer().get_column_search_matches().clone().is_empty() {
                    let current = self.buffer().get_column_search_current_match();
                    if current == 0 {
                        let matches_len = self.buffer().get_column_search_matches().clone().len();
                        self.buffer_mut()
                            .set_column_search_current_match(matches_len - 1);
                    } else {
                        self.buffer_mut()
                            .set_column_search_current_match(current - 1);
                    }
                    let (column_index, column_name) =
                        self.buffer().get_column_search_matches().clone()
                            [self.buffer().get_column_search_current_match()]
                        .clone();
                    let current_match = self.buffer().get_column_search_current_match() + 1;
                    let total_matches = self.buffer().get_column_search_matches().clone().len();
                    self.buffer_mut().set_current_column(column_index);
                    self.buffer_mut().set_status_message(format!(
                        "Column {} of {}: {}",
                        current_match, total_matches, column_name
                    ));
                }
            }
            KeyCode::Backspace => {
                let mut pattern = self.buffer().get_column_search_pattern();
                pattern.pop();
                self.buffer_mut().set_column_search_pattern(pattern.clone());
                // Also update input to keep it in sync for rendering
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());
                self.update_column_search();
            }
            KeyCode::Char(c) => {
                let mut pattern = self.buffer().get_column_search_pattern();
                pattern.push(c);
                self.buffer_mut().set_column_search_pattern(pattern.clone());
                // Also update input to keep it in sync for rendering
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());
                self.update_column_search();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_help_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Use the new HelpWidget
        match self.help_widget.handle_key(key) {
            HelpAction::Exit => {
                self.exit_help();
            }
            HelpAction::ShowDebug => {
                // F5 was pressed in help - this is handled by the widget itself
            }
            _ => {
                // Other actions are handled internally by the widget
            }
        }
        Ok(false)
    }

    // Helper methods for help mode actions
    fn exit_help(&mut self) {
        self.help_widget.on_exit();
        self.set_help_visible(false); // Keep state_container in sync
        self.help_scroll = 0;
        let mode = if self.buffer().get_results().is_some() {
            AppMode::Results
        } else {
            AppMode::Command
        };
        self.buffer_mut().set_mode(mode);
    }

    fn scroll_help_down(&mut self) {
        let max_lines: usize = 58;
        let visible_height: usize = 30;
        let max_scroll = max_lines.saturating_sub(visible_height);
        if (self.help_scroll as usize) < max_scroll {
            self.help_scroll += 1;
        }
    }

    fn scroll_help_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }

    fn help_page_down(&mut self) {
        let max_lines: usize = 58;
        let visible_height: usize = 30;
        let max_scroll = max_lines.saturating_sub(visible_height);
        self.help_scroll = (self.help_scroll + 10).min(max_scroll as u16);
    }

    fn help_page_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(10);
    }

    fn handle_history_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        if let Some(ref state_container) = self.state_container {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(true)
                }
                KeyCode::Esc => {
                    // Cancel history search and restore original input
                    let original_input = state_container.cancel_history_search();
                    self.set_input_text(original_input);
                    self.buffer_mut().set_mode(AppMode::Command);
                    self.buffer_mut()
                        .set_status_message("History search cancelled".to_string());
                }
                KeyCode::Enter => {
                    // Accept the selected history command
                    if let Some(command) = state_container.accept_history_search() {
                        self.set_input_text(command);
                        self.buffer_mut().set_mode(AppMode::Command);
                        self.buffer_mut()
                            .set_status_message("Command loaded from history".to_string());
                        // Reset scroll to show end of command
                        self.input_scroll_offset = 0;
                        self.update_horizontal_scroll(120); // Will be properly updated on next render
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state_container.history_search_previous();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state_container.history_search_next();
                }
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Ctrl+R cycles through matches
                    state_container.history_search_next();
                }
                KeyCode::Backspace => {
                    let history_search = state_container.history_search();
                    let mut query = history_search.query.clone();
                    drop(history_search); // Release the borrow
                    query.pop();
                    state_container.update_history_search(query);
                }
                KeyCode::Char(c) => {
                    let history_search = state_container.history_search();
                    let mut query = history_search.query.clone();
                    drop(history_search); // Release the borrow
                    query.push(c);
                    state_container.update_history_search(query);
                }
                _ => {}
            }
        } else {
            // Fallback to old behavior if no state container
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(true)
                }
                KeyCode::Esc => {
                    self.buffer_mut().set_mode(AppMode::Command);
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
                        // Use helper to set text through buffer
                        self.set_input_text(selected_command);
                        self.buffer_mut().set_mode(AppMode::Command);
                        self.buffer_mut()
                            .set_status_message("Command loaded from history".to_string());
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
        }
        Ok(false)
    }

    fn update_history_matches(&mut self) {
        // Get current schema columns and data source for better matching
        let (current_columns, current_source_str) = if self.buffer().is_csv_mode() {
            if let Some(csv_client) = self.buffer().get_csv_client() {
                if let Some(schema) = csv_client.get_schema() {
                    // Get the first (and usually only) table's columns and name
                    let (cols, table_name) = schema
                        .iter()
                        .next()
                        .map(|(table_name, cols)| (cols.clone(), Some(table_name.clone())))
                        .unwrap_or((vec![], None));
                    (cols, table_name)
                } else {
                    (vec![], None)
                }
            } else {
                (vec![], None)
            }
        } else if self.buffer().is_cache_mode() {
            (vec![], Some("cache".to_string()))
        } else {
            (vec![], Some("api".to_string()))
        };

        let current_source = current_source_str.as_deref();

        self.history_state.matches = self.command_history.search_with_schema(
            &self.history_state.search_query,
            &current_columns,
            current_source,
        );
        self.history_state.selected_index = 0;
    }

    fn handle_debug_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Handle special keys for test case generation
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+C to quit
                return Ok(true);
            }
            KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+T: "Yank as Test" - capture current session as test case
                self.yank_as_test_case();
                return Ok(false);
            }
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Shift+Y: Yank debug dump with test context
                self.yank_debug_with_context();
                return Ok(false);
            }
            _ => {}
        }

        // Let the widget handle navigation and exit
        if self.debug_widget.handle_key(key) {
            // Widget returned true - exit debug mode
            self.buffer_mut().set_mode(AppMode::Command);
        }
        Ok(false)
    }

    fn handle_pretty_query_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(true);
        }

        // Let debug widget handle the key (includes scrolling and exit)
        if self.debug_widget.handle_key(key) {
            // Widget returned true - exit pretty query mode
            self.buffer_mut().set_mode(AppMode::Command);
        }
        Ok(false)
    }

    fn execute_query(&mut self, query: &str) -> Result<()> {
        info!(target: "query", "Executing query: {}", query);

        // Save the query being executed to last_query BEFORE execution
        // This ensures we preserve the actual query that was run
        self.buffer_mut().set_last_query(query.to_string());
        debug!(target: "buffer", "Saved query to last_query: '{}'", query);

        self.buffer_mut()
            .set_status_message(format!("Executing query: '{}'...", query));
        let start_time = std::time::Instant::now();

        let result = if self.buffer().is_cache_mode() {
            // When in cache mode, use CSV client to query cached data
            if let Some(cached_data) = self.buffer().get_cached_data() {
                let mut csv_client = CsvApiClient::new();
                csv_client.set_case_insensitive(self.buffer().is_case_insensitive());
                csv_client.load_from_json(cached_data.clone(), "cached_data")?;

                csv_client.query_csv(query).map(|r| QueryResponse {
                    data: r.data,
                    count: r.count,
                    query: sql_cli::api_client::QueryInfo {
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
        } else if self.buffer().is_csv_mode() {
            if let Some(csv_client) = self.buffer().get_csv_client() {
                // Convert CSV result to match the expected type
                csv_client.query_csv(query).map(|r| QueryResponse {
                    data: r.data,
                    count: r.count,
                    query: sql_cli::api_client::QueryInfo {
                        select: r.query.select,
                        where_clause: r.query.where_clause,
                        order_by: r.query.order_by,
                    },
                    source: Some("file".to_string()),
                    table: Some(self.buffer().get_table_name()),
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

                // Get schema columns and data source for history
                let (schema_columns, data_source) = if self.buffer().is_csv_mode() {
                    if let Some(csv_client) = self.buffer().get_csv_client() {
                        if let Some(schema) = csv_client.get_schema() {
                            // Get the first (and usually only) table's columns
                            let cols = schema
                                .iter()
                                .next()
                                .map(|(table_name, cols)| (cols.clone(), Some(table_name.clone())))
                                .unwrap_or((vec![], None));
                            cols
                        } else {
                            (vec![], None)
                        }
                    } else {
                        (vec![], None)
                    }
                } else if self.buffer().is_cache_mode() {
                    (vec![], Some("cache".to_string()))
                } else {
                    (vec![], Some("api".to_string()))
                };

                let _ = self.command_history.add_entry_with_schema(
                    query.to_string(),
                    true,
                    Some(duration.as_millis() as u64),
                    schema_columns,
                    data_source.clone(),
                );

                // Add debug info about results
                let row_count = response.data.len();

                // Capture the source from the response
                self.buffer_mut()
                    .set_last_query_source(response.source.clone());

                // Store results in the current buffer
                if let Some(buffer) = self.buffer_manager.current_mut() {
                    let buffer_id = buffer.get_id();
                    buffer.set_results(Some(response.clone()));
                    info!(target: "buffer", "Stored {} results in buffer {}", row_count, buffer_id);
                }
                self.buffer_mut().set_results(Some(response.clone())); // Keep for compatibility during migration

                // Also update AppStateContainer with results and performance metrics
                if let Some(ref state_container) = self.state_container {
                    let from_cache = data_source.as_deref() == Some("cache");
                    if let Err(e) =
                        state_container.set_results(response.clone(), duration, from_cache)
                    {
                        warn!(target: "results", "Failed to update results in AppStateContainer: {}", e);
                    }

                    // Also cache results for future use
                    let query_key = format!("{}:{}", query, self.buffer().get_table_name());
                    if let Err(e) = state_container.cache_results(query_key, response.clone()) {
                        warn!(target: "results", "Failed to cache results in AppStateContainer: {}", e);
                    }
                }

                // Update parser with the FULL schema if we're in CSV/cache mode
                // For CSV mode, get the complete schema from the CSV client, not from query results
                if self.buffer().is_csv_mode() {
                    let table_name = self.buffer().get_table_name();
                    if let Some(csv_client) = self.buffer().get_csv_client() {
                        if let Some(schema) = csv_client.get_schema() {
                            // Get the full column list from the schema
                            if let Some(columns) = schema.get(&table_name) {
                                info!(target: "buffer", "Query executed, updating parser with FULL schema ({} columns) for table '{}'", columns.len(), table_name);
                                self.hybrid_parser
                                    .update_single_table(table_name, columns.clone());
                            }
                        }
                    }
                } else if self.buffer().is_cache_mode() {
                    // For cache mode, we still use the results columns since cached data might be filtered
                    if let Some(first_row) = response.data.first() {
                        if let Some(obj) = first_row.as_object() {
                            let columns: Vec<String> = obj.keys().map(|k| k.to_string()).collect();
                            info!(target: "buffer", "Query executed, updating parser with {} columns for cached table", columns.len());
                            self.hybrid_parser
                                .update_single_table("cached_data".to_string(), columns);
                        }
                    }
                }

                self.calculate_optimal_column_widths();
                self.reset_table_state();

                if row_count == 0 {
                    self.buffer_mut().set_status_message(format!(
                        "Query executed successfully but returned 0 rows ({}ms)",
                        duration.as_millis()
                    ));
                } else {
                    self.buffer_mut().set_status_message(format!("Query executed successfully - {} rows returned ({}ms) - Use ↓ or j/k to navigate", row_count, duration.as_millis()));
                }

                self.buffer_mut().set_mode(AppMode::Results);
                self.table_state.select(Some(0));
            }
            Err(e) => {
                let duration = start_time.elapsed();

                // Get schema columns and data source for history (even for failed queries)
                let (schema_columns, data_source) = if self.buffer().is_csv_mode() {
                    if let Some(csv_client) = self.buffer().get_csv_client() {
                        if let Some(schema) = csv_client.get_schema() {
                            // Get the first (and usually only) table's columns
                            let cols = schema
                                .iter()
                                .next()
                                .map(|(table_name, cols)| (cols.clone(), Some(table_name.clone())))
                                .unwrap_or((vec![], None));
                            cols
                        } else {
                            (vec![], None)
                        }
                    } else {
                        (vec![], None)
                    }
                } else if self.buffer().is_cache_mode() {
                    (vec![], Some("cache".to_string()))
                } else {
                    (vec![], Some("api".to_string()))
                };

                let _ = self.command_history.add_entry_with_schema(
                    query.to_string(),
                    false,
                    Some(duration.as_millis() as u64),
                    schema_columns,
                    data_source,
                );
                self.buffer_mut()
                    .set_status_message(format!("Error: {}", e));
            }
        }
        Ok(())
    }

    fn parse_where_clause_ast(&self, query: &str) -> Result<String> {
        let query_lower = query.to_lowercase();
        if let Some(where_pos) = query_lower.find(" where ") {
            let where_clause = &query[where_pos + 7..]; // Skip " where "

            // Get columns from CSV client if available
            let columns = if self.buffer().is_csv_mode() {
                if let Some(csv_client) = self.buffer().get_csv_client() {
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
                self.buffer().is_case_insensitive(),
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
        let cursor_pos = self.get_input_cursor();
        let query_str = self.get_input_text();
        let query = query_str.as_str();

        let hybrid_result = self.hybrid_parser.get_completions(query, cursor_pos);
        if !hybrid_result.suggestions.is_empty() {
            self.buffer_mut().set_status_message(format!(
                "Suggestions: {}",
                hybrid_result.suggestions.join(", ")
            ));
        }
    }

    fn apply_completion(&mut self) {
        let cursor_pos = self.get_input_cursor();
        let query = self.get_input_text();

        // Check if this is a continuation of the same completion session
        let is_same_context = query == self.completion_state.last_query
            && cursor_pos == self.completion_state.last_cursor_pos;

        if !is_same_context {
            // New completion context - get fresh suggestions
            let hybrid_result = self.hybrid_parser.get_completions(&query, cursor_pos);
            if hybrid_result.suggestions.is_empty() {
                self.buffer_mut()
                    .set_status_message("No completions available".to_string());
                return;
            }

            self.completion_state.suggestions = hybrid_result.suggestions;
            self.completion_state.current_index = 0;
        } else if !self.completion_state.suggestions.is_empty() {
            // Cycle to next suggestion
            self.completion_state.current_index =
                (self.completion_state.current_index + 1) % self.completion_state.suggestions.len();
        } else {
            self.buffer_mut()
                .set_status_message("No completions available".to_string());
            return;
        }

        // Apply the current suggestion (clone to avoid borrow issues)
        let suggestion =
            self.completion_state.suggestions[self.completion_state.current_index].clone();
        let partial_word = self.extract_partial_word_at_cursor(&query, cursor_pos);

        if let Some(partial) = partial_word {
            // Replace the partial word with the suggestion
            let before_partial = &query[..cursor_pos - partial.len()];
            let after_cursor = &query[cursor_pos..];

            // Handle quoted identifiers - if both partial and suggestion start with quotes,
            // we need to avoid double quotes
            let suggestion_to_use = if partial.starts_with('"') && suggestion.starts_with('"') {
                // The partial already includes the opening quote, so use suggestion without its quote
                if suggestion.len() > 1 {
                    suggestion[1..].to_string()
                } else {
                    suggestion.clone()
                }
            } else {
                suggestion.clone()
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
            // Use helper to set text through buffer
            self.set_input_text(new_query.clone());
            // Set cursor to correct position
            if let Some(buffer) = self.buffer_manager.current_mut() {
                buffer.set_input_cursor_position(cursor_pos);
                // Sync for rendering
                if self.buffer().get_edit_mode() == EditMode::SingleLine {
                    self.set_input_text_with_cursor(new_query.clone(), cursor_pos);
                }
            }

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
            self.buffer_mut().set_status_message(suggestion_info);
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
            // Use helper to set text through buffer
            self.set_input_text(new_query.clone());
            // Set cursor to correct position
            if let Some(buffer) = self.buffer_manager.current_mut() {
                buffer.set_input_cursor_position(cursor_pos_new);
                // Sync for rendering
                if self.buffer().get_edit_mode() == EditMode::SingleLine {
                    self.input =
                        tui_input::Input::new(new_query.clone()).with_cursor(cursor_pos_new);
                }
            }

            // Update completion state
            self.completion_state.last_query = new_query;
            self.completion_state.last_cursor_pos = cursor_pos_new;

            self.buffer_mut()
                .set_status_message(format!("Inserted: {}", suggestion));
        }
    }

    // Note: expand_asterisk and get_table_columns removed - moved to Buffer and use hybrid_parser directly

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
            if self.buffer().is_viewport_lock() {
                // In lock mode, keep cursor at fixed viewport position
                if let Some(lock_row) = self.buffer().get_viewport_lock_row() {
                    // Adjust viewport so cursor stays at lock_row position
                    let mut offset = self.buffer().get_scroll_offset();
                    offset.0 = new_position.saturating_sub(lock_row);
                    self.buffer_mut().set_scroll_offset(offset);
                }
            } else {
                // Normal scrolling behavior
                let visible_rows = self.buffer().get_last_visible_rows();

                // Check if cursor would be below the last visible row
                let offset = self.buffer().get_scroll_offset();
                if new_position > offset.0 + visible_rows - 1 {
                    // Cursor moved below viewport - scroll down by one
                    self.buffer_mut()
                        .set_scroll_offset((offset.0 + 1, offset.1));
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
        if self.buffer().is_viewport_lock() {
            // In lock mode, keep cursor at fixed viewport position
            if let Some(lock_row) = self.buffer().get_viewport_lock_row() {
                // Adjust viewport so cursor stays at lock_row position
                let mut offset = self.buffer().get_scroll_offset();
                offset.0 = new_position.saturating_sub(lock_row);
                self.buffer_mut().set_scroll_offset(offset);
            }
        } else {
            // Normal scrolling behavior
            let mut offset = self.buffer().get_scroll_offset();
            if new_position < offset.0 {
                // Cursor moved above viewport - scroll up
                offset.0 = new_position;
                self.buffer_mut().set_scroll_offset(offset);
            }
        }
    }

    fn move_column_left(&mut self) {
        // Update cursor_manager for table navigation (incremental step)
        let (_row, _col) = self.cursor_manager.table_position();
        self.cursor_manager.move_table_left();

        // Keep existing logic for now
        let new_column = self.buffer().get_current_column().saturating_sub(1);
        self.buffer_mut().set_current_column(new_column);
        let mut offset = self.buffer().get_scroll_offset();
        offset.1 = offset.1.saturating_sub(1);
        let column_num = self.buffer().get_current_column() + 1;
        self.buffer_mut().set_scroll_offset(offset);
        self.buffer_mut()
            .set_status_message(format!("Column {} selected", column_num));
    }

    fn move_column_right(&mut self) {
        if let Some(results) = self.buffer().get_results() {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let max_columns = obj.len();

                    // Update cursor_manager for table navigation (incremental step)
                    self.cursor_manager.move_table_right(max_columns);

                    // Keep existing logic for now
                    let current_column = self.buffer().get_current_column();
                    if current_column + 1 < max_columns {
                        self.buffer_mut().set_current_column(current_column + 1);
                        let mut offset = self.buffer().get_scroll_offset();
                        offset.1 += 1;
                        let column_num = self.buffer().get_current_column() + 1;
                        self.buffer_mut().set_scroll_offset(offset);
                        self.buffer_mut()
                            .set_status_message(format!("Column {} selected", column_num));
                    }
                }
            }
        }
    }

    fn goto_first_column(&mut self) {
        self.buffer_mut().set_current_column(0);
        let mut offset = self.buffer().get_scroll_offset();
        offset.1 = 0;
        self.buffer_mut().set_scroll_offset(offset);
        self.buffer_mut()
            .set_status_message("First column selected".to_string());
    }

    fn goto_last_column(&mut self) {
        if let Some(results) = self.buffer().get_results() {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let max_columns = obj.len();
                    if max_columns > 0 {
                        self.buffer_mut().set_current_column(max_columns - 1);
                        // Update horizontal scroll to show the last column
                        // This ensures the last column is visible in the viewport
                        let mut offset = self.buffer().get_scroll_offset();
                        let column = self.buffer().get_current_column();
                        offset.1 = column.saturating_sub(5); // Keep some context
                        self.buffer_mut().set_scroll_offset(offset);
                        self.buffer_mut()
                            .set_status_message(format!("Last column selected ({})", column + 1));
                    }
                }
            }
        }
    }

    fn goto_first_row(&mut self) {
        self.table_state.select(Some(0));
        let mut offset = self.buffer().get_scroll_offset();
        offset.0 = 0; // Reset viewport to top
        self.buffer_mut().set_scroll_offset(offset);

        let total_rows = self.get_row_count();
        if total_rows > 0 {
            self.buffer_mut()
                .set_status_message(format!("Jumped to first row (1/{})", total_rows));
        }
    }

    fn toggle_column_pin(&mut self) {
        // Pin or unpin the current column
        let current_col = self.buffer().get_current_column();
        if self.buffer().get_pinned_columns().contains(&current_col) {
            // Column is already pinned, unpin it
            self.buffer_mut().remove_pinned_column(current_col);
            self.buffer_mut()
                .set_status_message(format!("Column {} unpinned", current_col + 1));
        } else {
            // Pin the column (max 4 pinned columns)
            if self.buffer().get_pinned_columns().clone().len() < 4 {
                self.buffer_mut().add_pinned_column(current_col);
                self.buffer_mut()
                    .set_status_message(format!("Column {} pinned 📌", current_col + 1));
            } else {
                self.buffer_mut()
                    .set_status_message("Maximum 4 pinned columns allowed".to_string());
            }
        }
    }

    fn clear_all_pinned_columns(&mut self) {
        self.buffer_mut().clear_pinned_columns();
        self.buffer_mut()
            .set_status_message("All columns unpinned".to_string());
    }

    fn calculate_column_statistics(&mut self) {
        use std::time::Instant;

        let start_total = Instant::now();

        // Collect all data first, then drop the buffer reference before calling analyzer
        let (column_name, data_to_analyze) = {
            // Get the current column name and data
            let results = match self.buffer().get_results() {
                Some(r) if !r.data.is_empty() => r,
                _ => return,
            };

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

            let current_column = self.buffer().get_current_column();
            if current_column >= headers.len() {
                return;
            }

            let column_name = headers[current_column].clone();

            // Extract column data more efficiently - avoid cloning strings when possible
            let data_to_analyze: Vec<String> =
                if let Some(filtered) = self.buffer().get_filtered_data() {
                    // For filtered data, we already have strings
                    let mut string_data = Vec::new();
                    for row in filtered {
                        if current_column < row.len() {
                            string_data.push(row[current_column].clone());
                        }
                    }
                    string_data
                } else {
                    // For JSON data, we need to convert to owned strings
                    results
                        .data
                        .iter()
                        .filter_map(|row| {
                            if let Some(obj) = row.as_object() {
                                obj.get(&column_name).map(|v| match v {
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

            (column_name, data_to_analyze)
        };

        // Convert to references for the analyzer
        let data_refs: Vec<&str> = data_to_analyze.iter().map(|s| s.as_str()).collect();

        // Use DataAnalyzer to calculate statistics
        let analyzer_stats = self
            .data_analyzer
            .calculate_column_statistics(&column_name, &data_refs);

        // Convert from DataAnalyzer's ColumnStatistics to buffer's ColumnStatistics
        let stats = ColumnStatistics {
            column_name: analyzer_stats.column_name,
            column_type: match analyzer_stats.data_type {
                sql_cli::data_analyzer::ColumnType::Integer
                | sql_cli::data_analyzer::ColumnType::Float => ColumnType::Numeric,
                sql_cli::data_analyzer::ColumnType::String
                | sql_cli::data_analyzer::ColumnType::Boolean
                | sql_cli::data_analyzer::ColumnType::Date => ColumnType::String,
                sql_cli::data_analyzer::ColumnType::Mixed => ColumnType::Mixed,
                sql_cli::data_analyzer::ColumnType::Unknown => ColumnType::Mixed,
            },
            total_count: analyzer_stats.total_values,
            null_count: analyzer_stats.null_values,
            unique_count: analyzer_stats.unique_values,
            frequency_map: analyzer_stats.frequency_map.clone(),
            // For numeric columns, parse the min/max strings to f64
            min: analyzer_stats
                .min_value
                .as_ref()
                .and_then(|s| s.parse::<f64>().ok()),
            max: analyzer_stats
                .max_value
                .as_ref()
                .and_then(|s| s.parse::<f64>().ok()),
            sum: analyzer_stats.sum_value,
            mean: analyzer_stats.avg_value,
            median: analyzer_stats.median_value,
        };

        // Calculate total time
        let elapsed = start_total.elapsed();

        self.buffer_mut().set_column_stats(Some(stats));

        // Show timing in status message
        self.buffer_mut().set_status_message(format!(
            "Column stats: {:.1}ms for {} values ({} unique)",
            elapsed.as_secs_f64() * 1000.0,
            data_to_analyze.len(),
            analyzer_stats.unique_values
        ));

        self.buffer_mut().set_mode(AppMode::ColumnStats);
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
            self.buffer_mut()
                .set_last_visible_rows(results_area_height.saturating_sub(3).max(10));
        }
    }

    fn goto_last_row(&mut self) {
        let total_rows = self.get_row_count();
        if total_rows > 0 {
            let last_row = total_rows - 1;
            self.table_state.select(Some(last_row));
            // Position viewport to show the last row at the bottom
            let visible_rows = self.buffer().get_last_visible_rows();
            let mut offset = self.buffer().get_scroll_offset();
            offset.0 = last_row.saturating_sub(visible_rows - 1);
            self.buffer_mut().set_scroll_offset(offset);

            // Set status to confirm action
            self.buffer_mut().set_status_message(format!(
                "Jumped to last row ({}/{})",
                last_row + 1,
                total_rows
            ));
        }
    }

    fn page_down(&mut self) {
        let total_rows = self.get_row_count();
        if total_rows > 0 {
            let visible_rows = self.buffer().get_last_visible_rows();
            let current = self.table_state.selected().unwrap_or(0);
            let new_position = (current + visible_rows).min(total_rows - 1);

            self.table_state.select(Some(new_position));

            // Scroll viewport down by a page
            let mut offset = self.buffer().get_scroll_offset();
            offset.0 = (offset.0 + visible_rows).min(total_rows.saturating_sub(visible_rows));
            self.buffer_mut().set_scroll_offset(offset);
        }
    }

    fn page_up(&mut self) {
        let visible_rows = self.buffer().get_last_visible_rows();
        let current = self.table_state.selected().unwrap_or(0);
        let new_position = current.saturating_sub(visible_rows);

        self.table_state.select(Some(new_position));

        // Scroll viewport up by a page
        let mut offset = self.buffer().get_scroll_offset();
        offset.0 = offset.0.saturating_sub(visible_rows);
        self.buffer_mut().set_scroll_offset(offset);
    }

    // Search and filter functions
    fn perform_search(&mut self) {
        // Use AppStateContainer for search if available
        if let Some(ref state_container) = self.state_container {
            if let Some(data) = self.get_current_data() {
                // Perform search using AppStateContainer
                let matches = state_container.perform_search(&data);

                // Update buffer with matches for now (until we fully migrate)
                let buffer_matches: Vec<(usize, usize)> = matches
                    .iter()
                    .map(|(row, col, _, _)| (*row, *col))
                    .collect();

                self.buffer_mut().set_search_matches(buffer_matches.clone());

                if !buffer_matches.is_empty() {
                    self.buffer_mut().set_search_match_index(0);
                    self.buffer_mut().set_current_match(Some(buffer_matches[0]));
                    let (row, _) = buffer_matches[0];
                    self.table_state.select(Some(row));
                    self.buffer_mut()
                        .set_status_message(format!("Found {} matches", buffer_matches.len()));
                } else {
                    self.buffer_mut()
                        .set_status_message("No matches found".to_string());
                }
            }
        } else {
            // Fallback to old implementation
            if let Some(data) = self.get_current_data() {
                self.buffer_mut().set_search_matches(Vec::new());

                if let Ok(regex) = Regex::new(&self.buffer().get_search_pattern()) {
                    for (row_idx, row) in data.iter().enumerate() {
                        for (col_idx, cell) in row.iter().enumerate() {
                            if regex.is_match(cell) {
                                let mut matches = self.buffer().get_search_matches();
                                matches.push((row_idx, col_idx));
                                self.buffer_mut().set_search_matches(matches);
                            }
                        }
                    }

                    if !self.buffer().get_search_matches().is_empty() {
                        self.buffer_mut().set_search_match_index(0);
                        let matches = self.buffer().get_search_matches();
                        self.buffer_mut().set_current_match(Some(matches[0]));
                        let (row, _) = matches[0];
                        self.table_state.select(Some(row));
                        self.buffer_mut()
                            .set_status_message(format!("Found {} matches", matches.len()));
                    } else {
                        self.buffer_mut()
                            .set_status_message("No matches found".to_string());
                    }
                } else {
                    self.buffer_mut()
                        .set_status_message("Invalid regex pattern".to_string());
                }
            }
        }
    }

    fn next_search_match(&mut self) {
        // Use AppStateContainer for search navigation if available
        if let Some(ref state_container) = self.state_container {
            if let Some((row, col)) = state_container.next_search_match() {
                // Extract values before mutable borrows
                let current_idx = state_container.search().current_match + 1;
                let total = state_container.search().matches.len();
                let search_match_index = state_container.search().current_match;

                // Now do mutable operations
                self.table_state.select(Some(row));
                self.buffer_mut().set_current_match(Some((row, col)));
                self.buffer_mut()
                    .set_status_message(format!("Match {} of {}", current_idx, total));
                self.buffer_mut().set_search_match_index(search_match_index);
            } else {
                self.buffer_mut()
                    .set_status_message("No search matches".to_string());
            }
        } else {
            // Fallback to old implementation
            if !self.buffer().get_search_matches().is_empty() {
                let matches = self.buffer().get_search_matches();
                let new_index = (self.buffer().get_search_match_index() + 1) % matches.len();
                self.buffer_mut().set_search_match_index(new_index);
                let (row, _) = matches[new_index];
                self.table_state.select(Some(row));
                self.buffer_mut()
                    .set_current_match(Some(matches[new_index]));
                self.buffer_mut().set_status_message(format!(
                    "Match {} of {}",
                    new_index + 1,
                    matches.len()
                ));
            }
        }
    }

    fn previous_search_match(&mut self) {
        // Use AppStateContainer for search navigation if available
        if let Some(ref state_container) = self.state_container {
            if let Some((row, col)) = state_container.previous_search_match() {
                // Extract values before mutable borrows
                let current_idx = state_container.search().current_match + 1;
                let total = state_container.search().matches.len();
                let search_match_index = state_container.search().current_match;

                // Now do mutable operations
                self.table_state.select(Some(row));
                self.buffer_mut().set_current_match(Some((row, col)));
                self.buffer_mut()
                    .set_status_message(format!("Match {} of {}", current_idx, total));
                self.buffer_mut().set_search_match_index(search_match_index);
            } else {
                self.buffer_mut()
                    .set_status_message("No search matches".to_string());
            }
        } else {
            // Fallback to old implementation
            if !self.buffer().get_search_matches().is_empty() {
                let matches = self.buffer().get_search_matches();
                let current_index = self.buffer().get_search_match_index();
                let new_index = if current_index == 0 {
                    matches.len() - 1
                } else {
                    current_index - 1
                };
                self.buffer_mut().set_search_match_index(new_index);
                let (row, _) = matches[new_index];
                self.table_state.select(Some(row));
                self.buffer_mut()
                    .set_current_match(Some(matches[new_index]));
                self.buffer_mut().set_status_message(format!(
                    "Match {} of {}",
                    new_index + 1,
                    matches.len()
                ));
            }
        }
    }

    fn apply_filter(&mut self) {
        let pattern = if let Some(ref state_container) = self.state_container {
            state_container.filter().pattern.clone()
        } else {
            self.get_filter_state().pattern.clone()
        };

        debug!(target: "filter", "apply_filter called with pattern: '{}', case_insensitive: {}", 
               pattern, self.buffer().is_case_insensitive());

        if pattern.is_empty() {
            debug!(target: "filter", "Pattern is empty, clearing filter");
            self.buffer_mut().set_filtered_data(None);
            if let Some(ref state_container) = self.state_container {
                state_container.filter_mut().is_active = false;
            } else {
                self.get_filter_state_mut().active = false;
            }
            self.buffer_mut()
                .set_status_message("Filter cleared".to_string());
            return;
        }

        if let Some(results) = self.buffer().get_results() {
            // Build regex with case-insensitive flag if needed
            let case_insensitive = self.buffer().is_case_insensitive();
            let regex_pattern = if case_insensitive {
                format!("(?i){}", pattern)
            } else {
                pattern.clone()
            };
            debug!(target: "filter", "Building regex pattern: '{}' (case_insensitive: {})", regex_pattern, case_insensitive);

            if let Ok(regex) = Regex::new(&regex_pattern) {
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
                                // Debug first few matches
                                if filtered.len() < 3 {
                                    debug!(target: "filter", "  Match found in cell: '{}'", 
                                           if cell_str.len() > 50 { format!("{}...", &cell_str[..50]) } else { cell_str.clone() });
                                }
                            }
                            row.push(cell_str);
                        }

                        if matches {
                            filtered.push(row);
                        }
                    }
                }

                let filtered_count = filtered.len();
                debug!(target: "filter", "Filter applied: {} rows matched out of {}", 
                       filtered_count, results.data.len());
                self.buffer_mut().set_filtered_data(Some(filtered));
                if let Some(ref state_container) = self.state_container {
                    let mut filter = state_container.filter_mut();
                    filter.is_active = true;
                    // Note: regex isn't stored in AppStateContainer FilterState yet
                } else {
                    self.get_filter_state_mut().regex = Some(regex);
                    self.get_filter_state_mut().active = true;
                }
                self.buffer_mut().set_filter_active(true);

                // Reset table state but preserve filtered data
                self.table_state = TableState::default();
                self.buffer_mut().set_scroll_offset((0, 0));
                self.buffer_mut().set_current_column(0);

                // Clear search state but keep filter state
                if let Some(ref state_container) = self.state_container {
                    let mut search = state_container.search_mut();
                    search.pattern = String::new();
                    search.current_match = 0;
                    search.matches = Vec::new();
                    search.is_active = false;
                } else {
                    // Fallback when state_container not available
                    eprintln!("[WARNING] SearchState migration: state_container not available for search reset");
                }

                self.buffer_mut()
                    .set_status_message(format!("Filtered to {} rows", filtered_count));
            } else {
                self.buffer_mut()
                    .set_status_message("Invalid regex pattern".to_string());
            }
        }
    }

    fn search_columns(&mut self) {
        let pattern = self.column_search_state.pattern.clone();
        debug!(target: "search", "search_columns called with pattern: '{}'", pattern);
        if pattern.is_empty() {
            debug!(target: "search", "Pattern is empty, skipping column search");
            return;
        }

        // Find matching columns
        let mut matching_columns = Vec::new();

        // Get columns from results
        if let Some(results) = self.buffer().get_results() {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    for (index, col_name) in obj.keys().enumerate() {
                        if col_name.to_lowercase().contains(&pattern.to_lowercase()) {
                            matching_columns.push((index, col_name.to_string()));
                        }
                    }
                }
            }
        }

        debug!(target: "search", "Found {} matching columns", matching_columns.len());
        if !matching_columns.is_empty() {
            for (idx, (col_idx, col_name)) in matching_columns.iter().enumerate() {
                debug!(target: "search", "  Match {}: '{}' at index {}", idx + 1, col_name, col_idx);
            }
        }

        if !matching_columns.is_empty() {
            // Move to first match
            self.current_column = matching_columns[0].0;
            self.column_search_state.current_match = 0;
            debug!(target: "search", "Setting current column to index {} ('{}')", 
                   matching_columns[0].0, matching_columns[0].1);
            let status_msg = format!(
                "Found {} columns matching '{}'. Tab/Shift-Tab to navigate.",
                matching_columns.len(),
                pattern
            );
            debug!(target: "search", "Setting status: {}", status_msg);
            self.buffer_mut().set_status_message(status_msg);

            // Also update buffer's column search matches
            self.buffer_mut()
                .set_column_search_matches(matching_columns.clone());
            self.buffer_mut().set_column_search_current_match(0);
            self.buffer_mut().set_current_column(matching_columns[0].0);
        } else {
            let status_msg = format!("No columns matching '{}'", pattern);
            debug!(target: "search", "Setting status: {}", status_msg);
            self.buffer_mut().set_status_message(status_msg);
            self.buffer_mut().set_column_search_matches(Vec::new());
        }

        self.column_search_state.matching_columns = matching_columns;
    }

    fn next_column_match(&mut self) {
        if self.column_search_state.matching_columns.is_empty() {
            debug!(target: "search", "next_column_match: No matching columns");
            return;
        }

        self.column_search_state.current_match = (self.column_search_state.current_match + 1)
            % self.column_search_state.matching_columns.len();

        let col_index =
            self.column_search_state.matching_columns[self.column_search_state.current_match].0;
        let col_name = self.column_search_state.matching_columns
            [self.column_search_state.current_match]
            .1
            .clone();
        self.current_column = col_index;

        let current_match_idx = self.column_search_state.current_match;
        let current_match = current_match_idx + 1;
        let total_matches = self.column_search_state.matching_columns.len();

        debug!(target: "search", "next_column_match: Moving to column {}/{}: {} (index {})", 
               current_match, total_matches, col_name, col_index);

        self.buffer_mut().set_current_column(col_index);
        self.buffer_mut().set_status_message(format!(
            "Column {}/{}: {} - Tab/Shift-Tab to navigate",
            current_match, total_matches, col_name
        ));

        // Update buffer's column search state
        self.buffer_mut()
            .set_column_search_current_match(current_match_idx);
    }

    fn previous_column_match(&mut self) {
        if self.column_search_state.matching_columns.is_empty() {
            debug!(target: "search", "previous_column_match: No matching columns");
            return;
        }

        if self.column_search_state.current_match == 0 {
            self.column_search_state.current_match =
                self.column_search_state.matching_columns.len() - 1;
        } else {
            self.column_search_state.current_match -= 1;
        }

        let col_index =
            self.column_search_state.matching_columns[self.column_search_state.current_match].0;
        let col_name = self.column_search_state.matching_columns
            [self.column_search_state.current_match]
            .1
            .clone();
        self.current_column = col_index;

        let current_match_idx = self.column_search_state.current_match;
        let current_match = current_match_idx + 1;
        let total_matches = self.column_search_state.matching_columns.len();

        debug!(target: "search", "previous_column_match: Moving to column {}/{}: {} (index {})", 
               current_match, total_matches, col_name, col_index);

        self.buffer_mut().set_current_column(col_index);
        self.buffer_mut().set_status_message(format!(
            "Column {}/{}: {} - Tab/Shift-Tab to navigate",
            current_match, total_matches, col_name
        ));

        // Update buffer's column search state
        self.buffer_mut()
            .set_column_search_current_match(current_match_idx);
    }

    fn apply_fuzzy_filter(&mut self) {
        if self.buffer().get_fuzzy_filter_pattern().is_empty() {
            self.buffer_mut().set_fuzzy_filter_indices(Vec::new());
            self.buffer_mut().set_fuzzy_filter_active(false);
            self.buffer_mut()
                .set_status_message("Fuzzy filter cleared".to_string());
            return;
        }

        let pattern = self.buffer().get_fuzzy_filter_pattern();
        let mut filtered_indices = Vec::new();

        // Get the data to filter - either already filtered data or original results
        let data_to_filter =
            if self.get_filter_state().active && self.buffer().get_filtered_data().is_some() {
                // If regex filter is active, fuzzy filter on top of that
                self.buffer().get_filtered_data()
            } else if let Some(results) = self.buffer().get_results() {
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
                self.buffer_mut().set_filtered_data(Some(rows));
                self.buffer().get_filtered_data()
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
                    let matcher = SkimMatcherV2::default();
                    if let Some(score) = matcher.fuzzy_match(&row_text, &pattern) {
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
        let is_active = !filtered_indices.is_empty();
        self.buffer_mut().set_fuzzy_filter_indices(filtered_indices);
        self.buffer_mut().set_fuzzy_filter_active(is_active);

        if self.buffer().is_fuzzy_filter_active() {
            let filter_type = if pattern.starts_with('\'') {
                "Exact"
            } else {
                "Fuzzy"
            };
            self.buffer_mut().set_status_message(format!(
                "{} filter: {} matches for '{}' (highlighted in magenta)",
                filter_type, match_count, pattern
            ));
            // Reset table state for new filtered view
            self.table_state = TableState::default();
            self.buffer_mut().set_scroll_offset((0, 0));
        } else {
            let filter_type = if pattern.starts_with('\'') {
                "exact"
            } else {
                "fuzzy"
            };
            self.buffer_mut()
                .set_status_message(format!("No {} matches for '{}'", filter_type, pattern));
        }
    }

    fn update_column_search(&mut self) {
        // Get column headers from the current results
        if let Some(results) = self.buffer().get_results() {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();

                    // Find matching columns (case-insensitive)
                    let pattern = self.buffer().get_column_search_pattern().to_lowercase();
                    let mut matching_columns = Vec::new();

                    for (index, header) in headers.iter().enumerate() {
                        if header.to_lowercase().contains(&pattern) {
                            matching_columns.push((index, header.to_string()));
                        }
                    }

                    self.buffer_mut()
                        .set_column_search_matches(matching_columns);
                    self.buffer_mut().set_column_search_current_match(0);

                    // Update status message
                    if self.buffer().get_column_search_pattern().is_empty() {
                        self.buffer_mut()
                            .set_status_message("Enter column name to search".to_string());
                    } else if self.buffer().get_column_search_matches().clone().is_empty() {
                        let pattern = self.buffer().get_column_search_pattern();
                        self.buffer_mut()
                            .set_status_message(format!("No columns match '{}'", pattern));
                    } else {
                        let (column_index, column_name) =
                            self.buffer().get_column_search_matches().clone()[0].clone();
                        let matches_len = self.buffer().get_column_search_matches().clone().len();
                        self.buffer_mut().set_current_column(column_index);
                        self.buffer_mut().set_status_message(format!(
                            "Column 1 of {}: {} (Tab=next, Enter=select)",
                            matches_len, column_name
                        ));
                    }
                } else {
                    self.buffer_mut()
                        .set_status_message("No column data available".to_string());
                }
            } else {
                self.buffer_mut()
                    .set_status_message("No data available for column search".to_string());
            }
        } else {
            self.buffer_mut()
                .set_status_message("No results available for column search".to_string());
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
            self.buffer_mut()
                .set_status_message("Sort cleared".to_string());
            return;
        }

        // Sort using original JSON values for proper type-aware comparison
        if let Some(results) = self.buffer().get_results() {
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
                        self.buffer_mut().set_results(Some(new_results.clone()));
                        self.buffer_mut().set_filtered_data(None); // Force regeneration of string data

                        // Also update AppStateContainer with sorted results
                        if let Some(ref state_container) = self.state_container {
                            // Sorting doesn't change execution time or cache status, so use existing values
                            let last_execution_time = state_container.get_last_execution_time();
                            let from_cache = state_container.is_results_from_cache();
                            if let Err(e) = state_container.set_results(
                                new_results,
                                last_execution_time,
                                from_cache,
                            ) {
                                warn!(target: "results", "Failed to update sorted results in AppStateContainer: {}", e);
                            }
                        }
                    }
                }
            }
        } else if let Some(data) = self.buffer().get_filtered_data() {
            // Fallback to string-based sorting if no JSON data available
            // Clone the data, sort it, and set it back
            let mut sorted_data = data.clone();
            sorted_data.sort_by(|a, b| {
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
            self.buffer_mut().set_filtered_data(Some(sorted_data));
        }

        self.sort_state = SortState {
            column: Some(column_index),
            order: new_order,
        };

        // Reset table state but preserve current column position
        let current_column = self.buffer().get_current_column();
        self.reset_table_state();
        self.buffer_mut().set_current_column(current_column);

        self.buffer_mut().set_status_message(format!(
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
        if let Some(filtered) = self.buffer().get_filtered_data() {
            Some(filtered.clone())
        } else if let Some(results) = self.buffer().get_results() {
            Some(DataExporter::convert_json_to_strings(&results.data))
        } else {
            None
        }
    }

    fn get_row_count(&self) -> usize {
        // TODO: Fix row count when fuzzy filter is active
        // Currently this returns the count from filtered_data (WHERE clause results)
        // but doesn't account for fuzzy_filter_state.filtered_indices
        // This causes incorrect row counts in the status line (e.g., showing 1/1513 instead of 1/257)
        // This will be fixed when fuzzy_filter_state is migrated to the buffer system
        // and we have a single source of truth for visible rows
        if let Some(filtered) = self.buffer().get_filtered_data() {
            filtered.len()
        } else if let Some(results) = self.buffer().get_results() {
            results.data.len()
        } else {
            0
        }
    }

    // Removed get_current_data_mut - sorting now uses immutable data and clones when needed
    // Removed convert_json_to_strings - moved to DataExporter module

    fn reset_table_state(&mut self) {
        self.table_state = TableState::default();
        self.buffer_mut().set_scroll_offset((0, 0));
        self.buffer_mut().set_current_column(0);
        self.buffer_mut().set_last_results_row(None); // Reset saved position for new results
        self.buffer_mut().set_last_scroll_offset((0, 0)); // Reset saved scroll offset for new results

        // Clear filter state to prevent old filtered data from persisting
        *self.get_filter_state_mut() = FilterState {
            pattern: String::new(),
            regex: None,
            active: false,
        };

        // Clear search state
        if let Some(ref state_container) = self.state_container {
            let mut search = state_container.search_mut();
            search.pattern = String::new();
            search.current_match = 0;
            search.matches = Vec::new();
            search.is_active = false;
        } else {
            // Fallback when state_container not available
            eprintln!(
                "[WARNING] SearchState migration: state_container not available for search clear"
            );
        }

        // Clear fuzzy filter state to prevent it from persisting across queries
        {
            let buffer = self.buffer_mut();
            buffer.clear_fuzzy_filter();
            buffer.set_fuzzy_filter_pattern(String::new());
            buffer.set_fuzzy_filter_active(false);
            buffer.set_fuzzy_filter_indices(Vec::new());
        };

        // Clear filtered data
        self.buffer_mut().set_filtered_data(None);
    }

    fn calculate_viewport_column_widths(&mut self, viewport_start: usize, viewport_end: usize) {
        // Calculate column widths based only on visible rows in viewport
        if let Some(results) = self.buffer().get_results() {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                    let mut widths = Vec::with_capacity(headers.len());

                    // Use compact mode settings
                    let compact = self.buffer().is_compact_mode();
                    let min_width = if compact { 4 } else { 6 };
                    let max_width = if compact { 20 } else { 30 };
                    let padding = if compact { 1 } else { 2 };

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

                    self.buffer_mut().set_column_widths(widths);
                }
            }
        }
    }

    fn update_parser_for_current_buffer(&mut self) {
        // Sync the input field with the current buffer's text
        if let Some(buffer) = self.buffer_manager.current() {
            let text = buffer.get_input_text();
            let cursor_pos = buffer.get_input_cursor_position();
            self.input = tui_input::Input::new(text.clone()).with_cursor(cursor_pos);
            debug!(target: "buffer", "Synced input field with buffer text: '{}' (cursor: {})", text, cursor_pos);
        }

        // Update the parser's schema based on the current buffer's data source
        if let Some(buffer) = self.buffer_manager.current() {
            if buffer.is_csv_mode() {
                let table_name = buffer.get_table_name();
                if let Some(csv_client) = buffer.get_csv_client() {
                    if let Some(schema) = csv_client.get_schema() {
                        // Get the full column list from the schema
                        if let Some(columns) = schema.get(&table_name) {
                            debug!(target: "buffer", "Updating parser with {} columns for table '{}'", columns.len(), table_name);
                            self.hybrid_parser
                                .update_single_table(table_name, columns.clone());
                        }
                    }
                }
            } else if buffer.is_cache_mode() {
                // For cache mode, use cached data schema if available
                if let Some(cached_data) = buffer.get_cached_data() {
                    if let Some(first_row) = cached_data.first() {
                        if let Some(obj) = first_row.as_object() {
                            let columns: Vec<String> = obj.keys().map(|k| k.to_string()).collect();
                            debug!(target: "buffer", "Updating parser with {} columns for cached data", columns.len());
                            self.hybrid_parser
                                .update_single_table("cached_data".to_string(), columns);
                        }
                    }
                }
            } else if let Some(results) = buffer.get_results() {
                // For API mode or when we have results, use the result columns
                if let Some(first_row) = results.data.first() {
                    if let Some(obj) = first_row.as_object() {
                        let columns: Vec<String> = obj.keys().map(|k| k.to_string()).collect();
                        let table_name = buffer.get_table_name();
                        debug!(target: "buffer", "Updating parser with {} columns for table '{}'", columns.len(), table_name);
                        self.hybrid_parser.update_single_table(table_name, columns);
                    }
                }
            }
        }
    }

    fn calculate_optimal_column_widths(&mut self) {
        use sql_cli::column_manager::ColumnManager;

        if let Some(results) = self.buffer().get_results() {
            let widths = ColumnManager::calculate_optimal_widths(&results.data);
            if !widths.is_empty() {
                self.buffer_mut().set_column_widths(widths);
            }
        }
    }

    fn export_to_csv(&mut self) {
        match DataExporter::export_to_csv(self.buffer()) {
            Ok(message) => {
                self.buffer_mut().set_status_message(message);
            }
            Err(e) => {
                self.buffer_mut()
                    .set_status_message(format!("Export failed: {}", e));
            }
        }
    }

    fn yank_cell(&mut self) {
        debug!("yank_cell called");
        if let Some(selected_row) = self.table_state.selected() {
            let column = self.buffer().get_current_column();
            debug!("Yanking cell at row={}, column={}", selected_row, column);
            match YankManager::yank_cell(self.buffer(), selected_row, column) {
                Ok(result) => {
                    self.last_yanked = Some((result.description.clone(), result.preview.clone()));
                    let message = format!("Yanked cell: {}", result.full_value);
                    debug!("Yank successful: {}", message);
                    self.buffer_mut().set_status_message(message);
                }
                Err(e) => {
                    let message = format!("Failed to yank cell: {}", e);
                    debug!("Yank failed: {}", message);
                    self.buffer_mut().set_status_message(message);
                }
            }
        } else {
            debug!("No row selected for yank");
        }
    }

    fn yank_row(&mut self) {
        if let Some(selected_row) = self.table_state.selected() {
            match YankManager::yank_row(self.buffer(), selected_row) {
                Ok(result) => {
                    self.last_yanked = Some((result.description.clone(), result.preview));
                    self.buffer_mut()
                        .set_status_message(format!("Yanked {}", result.description));
                }
                Err(e) => {
                    self.buffer_mut()
                        .set_status_message(format!("Failed to yank row: {}", e));
                }
            }
        }
    }

    fn yank_column(&mut self) {
        let column = self.buffer().get_current_column();
        match YankManager::yank_column(self.buffer(), column) {
            Ok(result) => {
                self.last_yanked = Some((result.description.clone(), result.preview));
                self.buffer_mut()
                    .set_status_message(format!("Yanked {}", result.description));
            }
            Err(e) => {
                self.buffer_mut()
                    .set_status_message(format!("Failed to yank column: {}", e));
            }
        }
    }

    fn yank_all(&mut self) {
        match YankManager::yank_all(self.buffer()) {
            Ok(result) => {
                self.last_yanked = Some((result.description.clone(), result.preview.clone()));
                self.buffer_mut().set_status_message(format!(
                    "Yanked {}: {}",
                    result.description, result.preview
                ));
            }
            Err(e) => {
                self.buffer_mut()
                    .set_status_message(format!("Failed to yank all: {}", e));
            }
        }
    }

    /// Yank current query and results as a complete test case (Ctrl+T in debug mode)
    fn yank_as_test_case(&mut self) {
        let test_case = DebugInfo::generate_test_case(self.buffer());

        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(&test_case) {
                Ok(_) => {
                    self.buffer_mut().set_status_message(format!(
                        "Copied complete test case to clipboard ({} lines)",
                        test_case.lines().count()
                    ));
                    self.last_yanked = Some((
                        "Test Case".to_string(),
                        format!(
                            "{}...",
                            test_case.lines().take(3).collect::<Vec<_>>().join("; ")
                        ),
                    ));
                }
                Err(e) => {
                    self.buffer_mut().set_status_message(format!(
                        "Failed to copy test case to clipboard: {}",
                        e
                    ));
                }
            },
            Err(e) => {
                self.buffer_mut()
                    .set_status_message(format!("Failed to access clipboard: {}", e));
            }
        }
    }

    /// Yank debug dump with context for manual test creation (Shift+Y in debug mode)
    fn yank_debug_with_context(&mut self) {
        let debug_context = DebugInfo::generate_debug_context(self.buffer());

        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(&debug_context) {
                Ok(_) => {
                    self.buffer_mut().set_status_message(format!(
                        "Copied debug context to clipboard ({} lines)",
                        debug_context.lines().count()
                    ));
                    self.last_yanked = Some((
                        "Debug Context".to_string(),
                        "Query context with data for test creation".to_string(),
                    ));
                }
                Err(e) => {
                    self.buffer_mut()
                        .set_status_message(format!("Failed to copy debug context: {}", e));
                }
            },
            Err(e) => {
                self.buffer_mut()
                    .set_status_message(format!("Failed to access clipboard: {}", e));
            }
        }
    }

    fn paste_from_clipboard(&mut self) {
        // Paste from system clipboard into the current input field
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_text() {
                Ok(text) => {
                    match self.buffer().get_mode() {
                        AppMode::Command => {
                            // Always use single-line mode paste
                            // Get current cursor position
                            let cursor_pos = self.get_input_cursor();
                            let current_value = self.get_input_text();

                            // Insert at cursor position
                            let mut new_value = String::new();
                            new_value.push_str(&current_value[..cursor_pos]);
                            new_value.push_str(&text);
                            new_value.push_str(&current_value[cursor_pos..]);

                            self.set_input_text_with_cursor(new_value, cursor_pos + text.len());

                            self.buffer_mut()
                                .set_status_message(format!("Pasted {} characters", text.len()));
                        }
                        AppMode::Filter
                        | AppMode::FuzzyFilter
                        | AppMode::Search
                        | AppMode::ColumnSearch => {
                            // For search/filter modes, append to current pattern
                            let cursor_pos = self.get_input_cursor();
                            let current_value = self.get_input_text();

                            let mut new_value = String::new();
                            new_value.push_str(&current_value[..cursor_pos]);
                            new_value.push_str(&text);
                            new_value.push_str(&current_value[cursor_pos..]);

                            self.set_input_text_with_cursor(new_value, cursor_pos + text.len());

                            // Update the appropriate filter/search state
                            match self.buffer().get_mode() {
                                AppMode::Filter => {
                                    self.get_filter_state_mut().pattern = self.get_input_text();
                                    self.apply_filter();
                                }
                                AppMode::FuzzyFilter => {
                                    let input_text = self.get_input_text();
                                    self.buffer_mut().set_fuzzy_filter_pattern(input_text);
                                    self.apply_fuzzy_filter();
                                }
                                AppMode::Search => {
                                    let search_text = self.get_input_text();
                                    self.buffer_mut().set_search_pattern(search_text);
                                    // TODO: self.search_in_results();
                                }
                                AppMode::ColumnSearch => {
                                    let input_text = self.get_input_text();
                                    self.buffer_mut().set_column_search_pattern(input_text);
                                    // TODO: self.search_columns();
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            self.buffer_mut()
                                .set_status_message("Paste not available in this mode".to_string());
                        }
                    }
                }
                Err(e) => {
                    self.buffer_mut()
                        .set_status_message(format!("Failed to paste: {}", e));
                }
            },
            Err(e) => {
                self.buffer_mut()
                    .set_status_message(format!("Can't access clipboard: {}", e));
            }
        }
    }

    fn export_to_json(&mut self) {
        // Include filtered data if filters are active
        let include_filtered =
            self.get_filter_state().active || self.buffer().is_fuzzy_filter_active();

        match DataExporter::export_to_json(self.buffer(), include_filtered) {
            Ok(message) => {
                self.buffer_mut().set_status_message(message);
            }
            Err(e) => {
                self.buffer_mut()
                    .set_status_message(format!("Export failed: {}", e));
            }
        }
    }

    // Removed get_filtered_json_data - moved to YankManager::convert_filtered_to_json

    fn get_horizontal_scroll_offset(&self) -> u16 {
        // Delegate to cursor_manager (incremental refactoring)
        let (horizontal, _vertical) = self.cursor_manager.scroll_offsets();
        horizontal
    }

    fn update_horizontal_scroll(&mut self, terminal_width: u16) {
        let inner_width = terminal_width.saturating_sub(3) as usize; // Account for borders + 1 char padding
        let cursor_pos = self.get_input_cursor();

        // Update cursor_manager scroll (incremental refactoring)
        self.cursor_manager
            .update_horizontal_scroll(cursor_pos, terminal_width.saturating_sub(3));

        // Keep legacy field in sync for now
        if cursor_pos < self.input_scroll_offset as usize {
            self.input_scroll_offset = cursor_pos as u16;
        }
        // If cursor is after the scroll window, scroll right
        else if cursor_pos >= self.input_scroll_offset as usize + inner_width {
            self.input_scroll_offset = (cursor_pos + 1).saturating_sub(inner_width) as u16;
        }
    }

    fn get_cursor_token_position(&self) -> (usize, usize) {
        let query = self.get_input_text();
        let cursor_pos = self.get_input_cursor();
        TextNavigator::get_cursor_token_position(&query, cursor_pos)
    }

    fn get_token_at_cursor(&self) -> Option<String> {
        let query = self.get_input_text();
        let cursor_pos = self.get_input_cursor();
        TextNavigator::get_token_at_cursor(&query, cursor_pos)
    }

    fn move_cursor_word_backward(&mut self) {
        if let Some(buffer) = self.buffer_manager.current_mut() {
            buffer.move_cursor_word_backward();

            // Sync for rendering if single-line mode
            if buffer.get_edit_mode() == EditMode::SingleLine {
                let text = buffer.get_input_text();
                let cursor = buffer.get_input_cursor_position();
                self.set_input_text_with_cursor(text, cursor);
                self.cursor_manager.set_position(cursor);
            }
        }
    }

    fn move_cursor_word_forward(&mut self) {
        if let Some(buffer) = self.buffer_manager.current_mut() {
            buffer.move_cursor_word_forward();

            // Sync for rendering if single-line mode
            if buffer.get_edit_mode() == EditMode::SingleLine {
                let text = buffer.get_input_text();
                let cursor = buffer.get_input_cursor_position();
                self.set_input_text_with_cursor(text, cursor);
                self.cursor_manager.set_position(cursor);
            }
        }
    }

    fn kill_line(&mut self) {
        if let Some(buffer) = self.buffer_manager.current_mut() {
            buffer.kill_line();

            // Sync for rendering if single-line mode
            if buffer.get_edit_mode() == EditMode::SingleLine {
                let text = buffer.get_input_text();
                let cursor = buffer.get_input_cursor_position();
                self.set_input_text_with_cursor(text, cursor);
                self.cursor_manager.set_position(cursor);
            }
        }
    }

    fn kill_line_backward(&mut self) {
        if let Some(buffer) = self.buffer_manager.current_mut() {
            buffer.kill_line_backward();

            // Sync for rendering if single-line mode
            if buffer.get_edit_mode() == EditMode::SingleLine {
                let text = buffer.get_input_text();
                let cursor = buffer.get_input_cursor_position();
                self.set_input_text_with_cursor(text, cursor);
                self.cursor_manager.set_position(cursor);
            }
        }
    }

    fn undo(&mut self) {
        // Use buffer's high-level undo operation
        if let Some(buffer) = self.buffer_manager.current_mut() {
            if buffer.perform_undo() {
                self.buffer_mut()
                    .set_status_message("Undo performed".to_string());
            } else {
                self.buffer_mut()
                    .set_status_message("Nothing to undo".to_string());
            }
        }
    }

    // Buffer management methods

    fn new_buffer(&mut self) {
        let mut new_buffer =
            sql_cli::buffer::Buffer::new(self.buffer_manager.all_buffers().len() + 1);
        // Apply config settings to the new buffer
        new_buffer.set_compact_mode(self.config.display.compact_mode);
        new_buffer.set_case_insensitive(self.config.behavior.case_insensitive_default);
        new_buffer.set_show_row_numbers(self.config.display.show_row_numbers);

        info!(target: "buffer", "Creating new buffer with config: compact_mode={}, case_insensitive={}, show_row_numbers={}",
              self.config.display.compact_mode,
              self.config.behavior.case_insensitive_default,
              self.config.display.show_row_numbers);

        let index = self.buffer_manager.add_buffer(new_buffer);
        self.buffer_mut()
            .set_status_message(format!("Created new buffer #{}", index + 1));
    }

    // DataTable buffer creation disabled during revert
    // fn new_datatable_buffer(&mut self) { ... }

    /// Debug method to dump current buffer state (disabled to prevent TUI corruption)
    #[allow(dead_code)]
    fn debug_current_buffer(&self) {
        // Debug output disabled - was corrupting TUI display
        // Use tracing/logging instead if debugging is needed
    }

    fn yank(&mut self) {
        if let Some(buffer) = self.buffer_manager.current_mut() {
            buffer.yank();

            // Sync for rendering if single-line mode
            if buffer.get_edit_mode() == EditMode::SingleLine {
                let text = buffer.get_input_text();
                let cursor = buffer.get_input_cursor_position();
                self.set_input_text_with_cursor(text, cursor);
                self.cursor_manager.set_position(cursor);
            }
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        // Always use single-line mode input height
        let input_height = 3;

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
        let input_title = match self.buffer().get_mode() {
            AppMode::Command => "SQL Query".to_string(),
            AppMode::Results => "SQL Query (Results Mode - Press ↑ to edit)".to_string(),
            AppMode::Search => "Search Pattern".to_string(),
            AppMode::Filter => "Filter Pattern".to_string(),
            AppMode::FuzzyFilter => "Fuzzy Filter".to_string(),
            AppMode::ColumnSearch => "Column Search".to_string(),
            AppMode::Help => "Help".to_string(),
            AppMode::History => {
                let query = if let Some(ref state_container) = self.state_container {
                    state_container.history_search().query.clone()
                } else {
                    self.history_state.search_query.clone()
                };
                format!("History Search: '{}' (Esc to cancel)", query)
            }
            AppMode::Debug => "Parser Debug (F5)".to_string(),
            AppMode::PrettyQuery => "Pretty Query View (F6)".to_string(),
            AppMode::CacheList => "Cache Management (F7)".to_string(),
            AppMode::JumpToRow => format!("Jump to row: {}", self.get_jump_to_row_input()),
            AppMode::ColumnStats => "Column Statistics (S to close)".to_string(),
        };

        let input_block = Block::default().borders(Borders::ALL).title(input_title);

        // Check if we should use the search modes widget for rendering
        let use_search_widget = matches!(
            self.buffer().get_mode(),
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch
        ) && self.search_modes_widget.is_active();

        if use_search_widget {
            // Let the search modes widget render the input field with debounce indicator
            self.search_modes_widget.render(f, chunks[0]);
        } else {
            // Always get input text through the buffer API for consistency
            let input_text_string = self.get_input_text();

            // Get history search query if in history mode
            let history_query_string = if self.buffer().get_mode() == AppMode::History {
                if let Some(ref state_container) = self.state_container {
                    state_container.history_search().query.clone()
                } else {
                    self.history_state.search_query.clone()
                }
            } else {
                String::new()
            };

            let input_text = match self.buffer().get_mode() {
                AppMode::History => &history_query_string,
                _ => &input_text_string,
            };

            let input_paragraph = match self.buffer().get_mode() {
                AppMode::Command => {
                    match self.buffer().get_edit_mode() {
                        EditMode::SingleLine => {
                            // Use syntax highlighting for SQL command input with horizontal scrolling
                            let highlighted_line =
                                self.sql_highlighter.simple_sql_highlight(input_text);
                            Paragraph::new(Text::from(vec![highlighted_line]))
                                .block(input_block)
                                .scroll((0, self.get_horizontal_scroll_offset()))
                        }
                        EditMode::MultiLine => {
                            // MultiLine mode is no longer supported, always use single-line
                            let highlighted_line =
                                self.sql_highlighter.simple_sql_highlight(input_text);
                            Paragraph::new(Text::from(vec![highlighted_line]))
                                .block(input_block)
                                .scroll((0, self.get_horizontal_scroll_offset()))
                        }
                    }
                }
                _ => {
                    // Plain text for other modes
                    Paragraph::new(input_text.as_str())
                        .block(input_block)
                        .style(match self.buffer().get_mode() {
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

            // Render the input paragraph (single-line mode)
            f.render_widget(input_paragraph, chunks[0]);
        }
        let results_area = chunks[1];

        // Set cursor position for input modes (skip if search widget is handling it)
        if !use_search_widget {
            match self.buffer().get_mode() {
                AppMode::Command => {
                    // Always use single-line cursor handling
                    // Calculate cursor position with horizontal scrolling
                    let inner_width = chunks[0].width.saturating_sub(2) as usize;
                    let cursor_pos = self.get_visual_cursor().1; // Get column position for single-line
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
                AppMode::Search => {
                    f.set_cursor_position((
                        chunks[0].x + self.get_input_cursor() as u16 + 1,
                        chunks[0].y + 1,
                    ));
                }
                AppMode::Filter => {
                    f.set_cursor_position((
                        chunks[0].x + self.get_input_cursor() as u16 + 1,
                        chunks[0].y + 1,
                    ));
                }
                AppMode::FuzzyFilter => {
                    f.set_cursor_position((
                        chunks[0].x + self.get_input_cursor() as u16 + 1,
                        chunks[0].y + 1,
                    ));
                }
                AppMode::ColumnSearch => {
                    f.set_cursor_position((
                        chunks[0].x + self.get_input_cursor() as u16 + 1,
                        chunks[0].y + 1,
                    ));
                }
                AppMode::JumpToRow => {
                    f.set_cursor_position((
                        chunks[0].x + self.get_jump_to_row_input().len() as u16 + 1,
                        chunks[0].y + 1,
                    ));
                }
                AppMode::History => {
                    let query_len = if let Some(ref state_container) = self.state_container {
                        state_container.history_search().query.len()
                    } else {
                        self.history_state.search_query.len()
                    };
                    f.set_cursor_position((chunks[0].x + query_len as u16 + 1, chunks[0].y + 1));
                }
                _ => {}
            }
        }

        // Results area - render based on mode to reduce complexity
        match self.buffer().get_mode() {
            AppMode::Help => self.render_help(f, results_area),
            AppMode::History => self.render_history(f, results_area),
            AppMode::Debug => self.render_debug(f, results_area),
            AppMode::PrettyQuery => self.render_pretty_query(f, results_area),
            AppMode::CacheList => self.render_cache_list(f, results_area),
            AppMode::ColumnStats => self.render_column_stats(f, results_area),
            _ if self.buffer().get_results().is_some() => {
                // We need to work around the borrow checker here
                // Calculate widths needs mutable self, but we also need to pass results
                if let Some(results) = self.buffer().get_results() {
                    // Extract viewport info first
                    let terminal_height = results_area.height as usize;
                    let max_visible_rows = terminal_height.saturating_sub(3).max(10);
                    let total_rows = if let Some(filtered) = self.buffer().get_filtered_data() {
                        filtered.len()
                    } else {
                        results.data.len()
                    };
                    let row_viewport_start = self
                        .buffer()
                        .get_scroll_offset()
                        .0
                        .min(total_rows.saturating_sub(1));
                    let row_viewport_end = (row_viewport_start + max_visible_rows).min(total_rows);

                    // Calculate column widths based on viewport
                    self.calculate_viewport_column_widths(row_viewport_start, row_viewport_end);
                }

                // Now render the table
                if let Some(results) = self.buffer().get_results() {
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
        let (status_style, mode_color) = match self.buffer().get_mode() {
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

        let mode_indicator = match self.buffer().get_mode() {
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

        // Show buffer information
        {
            let index = self.buffer_manager.current_index();
            let total = self.buffer_manager.all_buffers().len();

            // Show buffer indicator if multiple buffers
            if total > 1 {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("[{}/{}]", index + 1, total),
                    Style::default().fg(Color::Yellow),
                ));
            }

            // Show current buffer name
            if let Some(buffer) = self.buffer_manager.current() {
                spans.push(Span::raw(" "));
                let name = buffer.get_name();
                let modified = if buffer.is_modified() { "*" } else { "" };
                spans.push(Span::styled(
                    format!("{}{}", name, modified),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }

        // Get buffer name from the current buffer
        let buffer_name = self.buffer().get_name();
        if !buffer_name.is_empty() && buffer_name != "[Buffer 1]" {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                buffer_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if self.buffer().is_csv_mode() && !self.buffer().get_table_name().is_empty() {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                self.buffer().get_table_name(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Mode-specific information
        match self.buffer().get_mode() {
            AppMode::Command => {
                // In command mode, show editing-related info
                if !self.get_input_text().trim().is_empty() {
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
                    if let Some(error_msg) = self.check_parser_error(&self.get_input_text()) {
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
                    if let Some(results) = self.buffer().get_results() {
                        if let Some(first_row) = results.data.first() {
                            if let Some(obj) = first_row.as_object() {
                                let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                                if self.buffer().get_current_column() < headers.len() {
                                    spans.push(Span::raw(" | Col: "));
                                    spans.push(Span::styled(
                                        headers[self.buffer().get_current_column()],
                                        Style::default().fg(Color::Cyan),
                                    ));

                                    // Show pinned columns count if any
                                    if !self.buffer().get_pinned_columns().clone().is_empty() {
                                        spans.push(Span::raw(" | "));
                                        spans.push(Span::styled(
                                            format!(
                                                "📌{}",
                                                self.buffer().get_pinned_columns().clone().len()
                                            ),
                                            Style::default().fg(Color::Magenta),
                                        ));
                                    }

                                    // In cell mode, show the current cell value
                                    if self.selection_mode == SelectionMode::Cell {
                                        if let Some(selected_row) = self.table_state.selected() {
                                            if let Some(row_data) = results.data.get(selected_row) {
                                                if let Some(row_obj) = row_data.as_object() {
                                                    if let Some(value) = row_obj.get(
                                                        headers[self.buffer().get_current_column()],
                                                    ) {
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
                    if self.buffer().is_fuzzy_filter_active() {
                        spans.push(Span::raw(" | "));
                        spans.push(Span::styled(
                            format!("Fuzzy: {}", self.buffer().get_fuzzy_filter_pattern()),
                            Style::default().fg(Color::Magenta),
                        ));
                    } else if self.get_filter_state().active {
                        spans.push(Span::raw(" | "));
                        spans.push(Span::styled(
                            format!("Filter: {}", self.get_filter_state().pattern),
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
                let pattern = self.get_input_text();
                if !pattern.is_empty() {
                    spans.push(Span::raw(" | Pattern: "));
                    spans.push(Span::styled(pattern, Style::default().fg(mode_color)));
                }
            }
            _ => {}
        }

        // Data source indicator (shown in all modes)
        if let Some(source) = self.buffer().get_last_query_source() {
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
        } else if self.buffer().is_csv_mode() {
            spans.push(Span::raw(" | "));
            spans.push(Span::raw(&self.config.display.icons.file));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("CSV: {}", self.buffer().get_table_name()),
                Style::default().fg(Color::Green),
            ));
        } else if self.buffer().is_cache_mode() {
            spans.push(Span::raw(" | "));
            spans.push(Span::raw(&self.config.display.icons.cache));
            spans.push(Span::raw(" "));
            spans.push(Span::styled("CACHE", Style::default().fg(Color::Cyan)));
        }

        // Global indicators (shown when active)
        let case_insensitive = self.buffer().is_case_insensitive();
        if case_insensitive {
            spans.push(Span::raw(" | "));
            // Use to_string() to ensure we get the actual string value
            let icon = self.config.display.icons.case_insensitive.clone();
            spans.push(Span::styled(
                format!("{} CASE", icon),
                Style::default().fg(Color::Cyan),
            ));
        }

        if self.buffer().is_compact_mode() {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled("COMPACT", Style::default().fg(Color::Green)));
        }

        if self.buffer().is_viewport_lock() {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                &self.config.display.icons.lock,
                Style::default().fg(Color::Magenta),
            ));
        }

        // Help shortcuts (right side)
        let help_text = match self.buffer().get_mode() {
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
            if self.buffer().get_pinned_columns().contains(&i) {
                pinned_headers.push((i, header));
            } else {
                scrollable_indices.push(i);
            }
        }

        // Calculate space used by pinned columns
        let mut pinned_width = 0;
        for &(idx, _) in &pinned_headers {
            let column_widths = self.buffer().get_column_widths().clone();
            if idx < column_widths.len() {
                pinned_width += column_widths[idx] as usize;
            } else {
                pinned_width += 15; // Default width
            }
        }

        // Calculate how many scrollable columns can fit in remaining space
        let remaining_width = available_width.saturating_sub(pinned_width);
        let column_widths = self.buffer().get_column_widths().clone();
        let max_visible_scrollable_cols = if !column_widths.is_empty() {
            let mut width_used = 0;
            let mut cols_that_fit = 0;

            for &idx in &scrollable_indices {
                if idx >= headers.len() {
                    break;
                }
                let col_width = if idx < column_widths.len() {
                    column_widths[idx] as usize
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
            .position(|&x| x == self.buffer().get_current_column());
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
            self.buffer().get_scroll_offset().1.min(
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

        let total_rows = if let Some(filtered) = self.buffer().get_filtered_data() {
            if self.buffer().is_fuzzy_filter_active()
                && !self.buffer().get_fuzzy_filter_indices().clone().is_empty()
            {
                self.buffer().get_fuzzy_filter_indices().clone().len()
            } else {
                filtered.len()
            }
        } else {
            results.data.len()
        };

        // Calculate row viewport
        let row_viewport_start = self
            .buffer()
            .get_scroll_offset()
            .0
            .min(total_rows.saturating_sub(1));
        let row_viewport_end = (row_viewport_start + max_visible_rows).min(total_rows);

        // Prepare table data (only visible rows AND columns)
        let data_to_display = if let Some(filtered) = self.buffer().get_filtered_data() {
            // Check if fuzzy filter is active
            if self.buffer().is_fuzzy_filter_active()
                && !self.buffer().get_fuzzy_filter_indices().clone().is_empty()
            {
                // Apply fuzzy filter on top of existing filter
                let mut fuzzy_filtered = Vec::new();
                for &idx in &self.buffer().get_fuzzy_filter_indices().clone() {
                    if idx < filtered.len() {
                        fuzzy_filtered.push(filtered[idx].clone());
                    }
                }

                // Recalculate viewport for fuzzy filtered data
                let fuzzy_total = fuzzy_filtered.len();
                let fuzzy_start = self
                    .buffer()
                    .get_scroll_offset()
                    .0
                    .min(fuzzy_total.saturating_sub(1));
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
        if self.buffer().is_show_row_numbers() {
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

            let column_indicator = if *actual_col_index == self.buffer().get_current_column() {
                " [*]"
            } else {
                ""
            };

            // Add pin indicator for pinned columns
            let pin_indicator = if self
                .buffer()
                .get_pinned_columns()
                .contains(&*actual_col_index)
            {
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
            if *actual_col_index == self.buffer().get_current_column() {
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
                if self.buffer().is_show_row_numbers() {
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
                    let is_selected_cell =
                        is_selected_row && actual_col_idx == self.buffer().get_current_column();

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
                        if actual_col_idx == self.buffer().get_current_column() {
                            style = style.bg(Color::DarkGray);
                        }
                    }

                    // Highlight search matches (override column highlight)
                    if let Some((match_row, match_col)) = self.buffer().get_current_match() {
                        if actual_row_idx == match_row && actual_col_idx == match_col {
                            style = style.bg(Color::Yellow).fg(Color::Black);
                        }
                    }

                    // Highlight filter matches
                    if self.get_filter_state().active {
                        if let Some(ref regex) = self.get_filter_state().regex {
                            if regex.is_match(cell) {
                                style = style.fg(Color::Cyan);
                            }
                        }
                    }

                    // Highlight fuzzy/exact filter matches
                    if self.buffer().is_fuzzy_filter_active()
                        && !self.buffer().get_fuzzy_filter_pattern().is_empty()
                    {
                        let pattern = &self.buffer().get_fuzzy_filter_pattern();
                        let cell_matches = if pattern.starts_with('\'') && pattern.len() > 1 {
                            // Exact match highlighting
                            let exact_pattern = &pattern[1..];
                            cell.to_lowercase().contains(&exact_pattern.to_lowercase())
                        } else {
                            // Fuzzy match highlighting - check if this cell contributes to the fuzzy match
                            if let Some(score) =
                                SkimMatcherV2::default().fuzzy_match(cell, &pattern)
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
        if self.buffer().is_show_row_numbers() {
            // Calculate width needed for row numbers (max row count digits + padding)
            let max_row_num = total_rows;
            let row_num_width = max_row_num.to_string().len() as u16 + 2;
            constraints.push(Constraint::Length(row_num_width.min(8))); // Cap at 8 chars
        }

        // Add data column constraints
        let column_widths = self.buffer().get_column_widths().clone();
        if !column_widths.is_empty() {
            // Use calculated optimal widths for visible columns
            constraints.extend(visible_columns.iter().map(|(col_idx, _)| {
                if *col_idx < column_widths.len() {
                    Constraint::Length(column_widths[*col_idx])
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
                    self.buffer().get_pinned_columns().clone().len(),
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

    fn render_help(&mut self, f: &mut Frame, area: Rect) {
        // Use the new HelpWidget for rendering
        self.help_widget.render(f, area);
    }

    fn render_help_old(&self, f: &mut Frame, area: Rect) {
        // Create two-column layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Get help content from HelpText module
        let left_content = HelpText::left_column();
        let right_content = HelpText::right_column();

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
        self.debug_widget.render(f, area, AppMode::Debug);
    }

    fn render_pretty_query(&self, f: &mut Frame, area: Rect) {
        self.debug_widget.render(f, area, AppMode::PrettyQuery);
    }

    fn render_history(&self, f: &mut Frame, area: Rect) {
        // Get history state from AppStateContainer if available
        let (matches_empty, search_query_empty) =
            if let Some(ref state_container) = self.state_container {
                let history_search = state_container.history_search();
                (
                    history_search.matches.is_empty(),
                    history_search.query.is_empty(),
                )
            } else {
                (
                    self.history_state.matches.is_empty(),
                    self.history_state.search_query.is_empty(),
                )
            };

        if matches_empty {
            let no_history = if search_query_empty {
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
        // Get history data from AppStateContainer if available, otherwise use local state
        let (matches, selected_index, match_count) =
            if let Some(ref state_container) = self.state_container {
                let history_search = state_container.history_search();
                let matches = history_search.matches.clone();
                let selected_index = history_search.selected_index;
                let match_count = matches.len();
                (matches, selected_index, match_count)
            } else {
                (
                    self.history_state.matches.clone(),
                    self.history_state.selected_index,
                    self.history_state.matches.len(),
                )
            };

        // Create more compact history list - just show essential info
        let history_items: Vec<Line> = matches
            .iter()
            .enumerate()
            .map(|(i, history_match)| {
                let entry = &history_match.entry;
                let is_selected = i == selected_index;

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
                match_count
            )))
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(history_paragraph, area);
    }

    fn render_selected_command_preview(&self, f: &mut Frame, area: Rect) {
        // Get the selected match from AppStateContainer if available
        let selected_match = if let Some(ref state_container) = self.state_container {
            let history_search = state_container.history_search();
            history_search
                .matches
                .get(history_search.selected_index)
                .cloned()
        } else {
            self.history_state
                .matches
                .get(self.history_state.selected_index)
                .cloned()
        };

        if let Some(selected_match) = selected_match {
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
            self.buffer_mut().set_status_message(
                "Invalid cache command. Use :cache save <query> or :cache load <id>".to_string(),
            );
            return Ok(());
        }

        match parts[1] {
            "save" => {
                // Save last query results to cache with optional custom ID
                if let Some(results) = self.buffer().get_results() {
                    let data_to_save = results.data.clone(); // Extract the data we need
                    let _ = results; // Explicitly drop the borrow

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
                                    self.buffer_mut()
                                        .set_status_message("No query to cache".to_string());
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
                            self.buffer_mut()
                                .set_status_message("No query to cache".to_string());
                            return Ok(());
                        };

                        match cache.save_query(&query, &data_to_save, custom_id) {
                            Ok(id) => {
                                self.buffer_mut().set_status_message(format!(
                                    "Query cached with ID: {} ({} rows)",
                                    id,
                                    data_to_save.len()
                                ));
                            }
                            Err(e) => {
                                self.buffer_mut()
                                    .set_status_message(format!("Failed to cache query: {}", e));
                            }
                        }
                    }
                } else {
                    self.buffer_mut().set_status_message(
                        "No results to cache. Execute a query first.".to_string(),
                    );
                }
            }
            "load" => {
                if parts.len() < 3 {
                    self.buffer_mut()
                        .set_status_message("Usage: :cache load <id>".to_string());
                    return Ok(());
                }

                if let Ok(id) = parts[2].parse::<u64>() {
                    if let Some(ref cache) = self.query_cache {
                        match cache.load_query(id) {
                            Ok((_query, data)) => {
                                self.buffer_mut().set_cached_data(Some(data.clone()));
                                self.buffer_mut().set_cache_mode(true);
                                self.buffer_mut().set_status_message(format!(
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
                                self.buffer_mut()
                                    .set_status_message(format!("Failed to load cache: {}", e));
                            }
                        }
                    }
                } else {
                    self.buffer_mut()
                        .set_status_message("Invalid cache ID".to_string());
                }
            }
            "list" => {
                self.buffer_mut().set_mode(AppMode::CacheList);
            }
            "clear" => {
                self.buffer_mut().set_cache_mode(false);
                self.buffer_mut().set_cached_data(None);
                self.buffer_mut()
                    .set_status_message("Cache mode disabled".to_string());
            }
            _ => {
                self.buffer_mut().set_status_message(
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
                self.buffer_mut().set_mode(AppMode::Command);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_column_stats_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match self.stats_widget.handle_key(key) {
            StatsAction::Quit => return Ok(true),
            StatsAction::Close => {
                self.buffer_mut().set_column_stats(None);
                self.buffer_mut().set_mode(AppMode::Results);
            }
            StatsAction::Continue | StatsAction::PassThrough => {}
        }
        Ok(false)
    }

    fn handle_jump_to_row_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.buffer_mut().set_mode(AppMode::Results);
                self.clear_jump_to_row_input();
                self.buffer_mut()
                    .set_status_message("Jump cancelled".to_string());
            }
            KeyCode::Enter => {
                if let Ok(row_num) = self.get_jump_to_row_input().parse::<usize>() {
                    if row_num > 0 {
                        let target_row = row_num - 1; // Convert to 0-based index
                        let max_row = self.get_current_data().map(|d| d.len()).unwrap_or(0);

                        if target_row < max_row {
                            self.table_state.select(Some(target_row));

                            // Adjust viewport to center the target row
                            let visible_rows = self.buffer().get_last_visible_rows();
                            if visible_rows > 0 {
                                let mut offset = self.buffer().get_scroll_offset();
                                offset.0 = target_row.saturating_sub(visible_rows / 2);
                                self.buffer_mut().set_scroll_offset(offset);
                            }

                            self.buffer_mut()
                                .set_status_message(format!("Jumped to row {}", row_num));
                        } else {
                            self.buffer_mut().set_status_message(format!(
                                "Row {} out of range (max: {})",
                                row_num, max_row
                            ));
                        }
                    }
                }
                self.buffer_mut().set_mode(AppMode::Results);
                self.clear_jump_to_row_input();
            }
            KeyCode::Backspace => {
                let mut input = self.get_jump_to_row_input();
                input.pop();
                self.set_jump_to_row_input(input);
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let mut input = self.get_jump_to_row_input();
                input.push(c);
                self.set_jump_to_row_input(input);
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
        // Delegate to the stats widget
        self.stats_widget.render(f, area, self.buffer());
    }

    // === Editor Widget Helper Methods ===
    // These methods handle the actions returned by the editor widget

    fn handle_execute_query(&mut self) -> Result<bool> {
        // Get the current query text and execute it directly
        let query = self.get_input_text().trim().to_string();
        debug!(target: "action", "Executing query: {}", query);
        if !query.is_empty() {
            // Check for special commands
            if query == ":help" {
                self.set_help_visible(true);
                self.buffer_mut().set_mode(AppMode::Help);
                self.buffer_mut()
                    .set_status_message("Help Mode - Press ESC to return".to_string());
            } else if query == ":exit" || query == ":quit" {
                return Ok(true);
            } else {
                // Execute the SQL query
                self.buffer_mut()
                    .set_status_message(format!("Processing query: '{}'", query));
                if let Err(e) = self.execute_query(&query) {
                    self.buffer_mut()
                        .set_status_message(format!("Error executing query: {}", e));
                }
                // Don't clear input - preserve query for editing
            }
        }
        Ok(false) // Continue running, don't exit
    }

    fn handle_buffer_action(&mut self, action: BufferAction) -> Result<bool> {
        match action {
            BufferAction::NextBuffer => {
                let message = self.buffer_handler.next_buffer(&mut self.buffer_manager);
                debug!("{}", message);
                // Update parser schema for the new buffer
                self.update_parser_for_current_buffer();
                Ok(false)
            }
            BufferAction::PreviousBuffer => {
                let message = self
                    .buffer_handler
                    .previous_buffer(&mut self.buffer_manager);
                debug!("{}", message);
                // Update parser schema for the new buffer
                self.update_parser_for_current_buffer();
                Ok(false)
            }
            BufferAction::QuickSwitch => {
                let message = self.buffer_handler.quick_switch(&mut self.buffer_manager);
                debug!("{}", message);
                // Update parser schema for the new buffer
                self.update_parser_for_current_buffer();
                Ok(false)
            }
            BufferAction::NewBuffer => {
                let message = self
                    .buffer_handler
                    .new_buffer(&mut self.buffer_manager, &self.config);
                debug!("{}", message);
                Ok(false)
            }
            BufferAction::CloseBuffer => {
                let (success, message) = self.buffer_handler.close_buffer(&mut self.buffer_manager);
                debug!("{}", message);
                Ok(!success) // Exit if we couldn't close (only one left)
            }
            BufferAction::ListBuffers => {
                let buffer_list = self.buffer_handler.list_buffers(&self.buffer_manager);
                // For now, just log the list - later we can show a popup
                for line in &buffer_list {
                    debug!("{}", line);
                }
                Ok(false)
            }
            BufferAction::SwitchToBuffer(buffer_index) => {
                let message = self
                    .buffer_handler
                    .switch_to_buffer(&mut self.buffer_manager, buffer_index);
                debug!("{}", message);

                // Update parser schema for the new buffer
                self.update_parser_for_current_buffer();

                Ok(false)
            }
        }
    }

    fn handle_expand_asterisk(&mut self) -> Result<bool> {
        if let Some(buffer) = self.buffer_manager.current_mut() {
            if buffer.expand_asterisk(&self.hybrid_parser) {
                // Sync for rendering if needed
                if buffer.get_edit_mode() == EditMode::SingleLine {
                    let text = buffer.get_input_text();
                    let cursor = buffer.get_input_cursor_position();
                    self.set_input_text_with_cursor(text, cursor);
                }
            }
        }
        Ok(false)
    }

    fn toggle_debug_mode(&mut self) {
        // First, collect all the data we need without any mutable borrows
        let (
            should_exit_debug,
            previous_mode,
            last_query,
            input_text,
            selected_row,
            current_column,
            results_count,
            filtered_count,
        ) = {
            if let Some(buffer) = self.buffer_manager.current() {
                let mode = buffer.get_mode();
                if mode == AppMode::Debug {
                    (true, mode, String::new(), String::new(), None, 0, 0, 0)
                } else {
                    (
                        false,
                        mode,
                        buffer.get_last_query(),
                        buffer.get_input_text(),
                        buffer.get_selected_row(),
                        buffer.get_current_column(),
                        buffer.get_results().map(|r| r.data.len()).unwrap_or(0),
                        buffer.get_filtered_data().map(|d| d.len()).unwrap_or(0),
                    )
                }
            } else {
                return;
            }
        };

        // Collect buffer manager info without mutable borrow
        let buffer_names: Vec<String> = self
            .buffer_manager
            .all_buffers()
            .iter()
            .map(|b| b.get_name())
            .collect();
        let buffer_count = self.buffer_manager.all_buffers().len();
        let buffer_index = self.buffer_manager.current_index();

        // Now handle the mode transition with mutable borrow
        if let Some(buffer) = self.buffer_manager.current_mut() {
            if should_exit_debug {
                buffer.set_mode(AppMode::Command);
            } else {
                buffer.set_mode(AppMode::Debug);
                // Generate full debug information like the original F5 handler
                self.debug_current_buffer();
                let cursor_pos = self.get_input_cursor();
                let visual_cursor = self.get_visual_cursor().1;
                let query = self.get_input_text();

                // Use the appropriate query for parser debug based on mode
                let query_for_parser =
                    if previous_mode == AppMode::Results && !last_query.is_empty() {
                        // In Results mode, show parser info for the executed query
                        last_query.clone()
                    } else if !query.is_empty() {
                        // In Command mode, show parser info for current input
                        query.clone()
                    } else if !last_query.is_empty() {
                        // Fallback to last query if input is empty
                        last_query.clone()
                    } else {
                        query.clone()
                    };

                // Generate debug info directly without buffer reference
                let mut debug_info = self
                    .hybrid_parser
                    .get_detailed_debug_info(&query_for_parser, query_for_parser.len());

                // Add comprehensive buffer state
                debug_info.push_str(&format!(
                    "\n========== BUFFER STATE ==========\n\
                    Current Mode: {:?}\n\
                    Last Executed Query: '{}'\n\
                    Input Text: '{}'\n\
                    Input Cursor: {}\n\
                    Visual Cursor: {}\n",
                    previous_mode, last_query, input_text, cursor_pos, visual_cursor
                ));

                // Add results state if in Results mode
                if results_count > 0 {
                    debug_info.push_str(&format!(
                        "\n========== RESULTS STATE ==========\n\
                            Total Rows: {}\n\
                            Filtered Rows: {}\n\
                            Selected Row: {:?}\n\
                            Current Column: {}\n",
                        results_count, filtered_count, selected_row, current_column
                    ));
                }

                // Add buffer state info
                debug_info.push_str(&format!(
                    "\n========== BUFFER MANAGER STATE ==========\n\
                        Number of Buffers: {}\n\
                        Current Buffer Index: {}\n\
                        Buffer Names: {}\n",
                    buffer_count,
                    buffer_index,
                    buffer_names.join(", ")
                ));

                // Add WHERE clause AST if needed
                if query.to_lowercase().contains(" where ") {
                    let where_ast_info = match self.parse_where_clause_ast(&query) {
                            Ok(ast_str) => ast_str,
                            Err(e) => format!("\n========== WHERE CLAUSE AST ==========\nError parsing WHERE clause: {}\n", e)
                        };
                    debug_info.push_str(&where_ast_info);
                }

                // Add key chord handler debug info
                debug_info.push_str("\n");
                debug_info.push_str(&self.key_chord_handler.format_debug_info());
                debug_info.push_str("========================================\n");

                // Add search modes widget debug info
                debug_info.push_str("\n");
                debug_info.push_str(&self.search_modes_widget.debug_info());

                // Add column search state if active
                if self.buffer().get_mode() == AppMode::ColumnSearch
                    || !self.column_search_state.pattern.is_empty()
                {
                    debug_info.push_str("\n========== COLUMN SEARCH STATE ==========\n");
                    debug_info.push_str(&format!(
                        "Pattern: '{}'\n",
                        self.column_search_state.pattern
                    ));
                    debug_info.push_str(&format!(
                        "Buffer Pattern: '{}'\n",
                        self.buffer().get_column_search_pattern()
                    ));
                    debug_info.push_str(&format!(
                        "Matching Columns: {} found\n",
                        self.column_search_state.matching_columns.len()
                    ));
                    if !self.column_search_state.matching_columns.is_empty() {
                        debug_info.push_str("Matches:\n");
                        for (idx, (col_idx, col_name)) in
                            self.column_search_state.matching_columns.iter().enumerate()
                        {
                            let marker = if idx == self.column_search_state.current_match {
                                " <--"
                            } else {
                                ""
                            };
                            debug_info.push_str(&format!(
                                "  [{}] {} (index {}){}
",
                                idx, col_name, col_idx, marker
                            ));
                        }
                    }
                    debug_info.push_str(&format!(
                        "Current Match Index: {}\n",
                        self.column_search_state.current_match
                    ));
                    debug_info.push_str(&format!("Current Column: {}\n", self.current_column));
                    debug_info.push_str("==========================================\n");
                }

                // Add trace logs from ring buffer
                debug_info.push_str("\n========== TRACE LOGS ==========\n");
                debug_info.push_str("(Most recent at bottom, last 100 entries)\n");
                if let Some(ref log_buffer) = self.log_buffer {
                    let recent_logs = log_buffer.get_recent(100);
                    for entry in recent_logs {
                        debug_info.push_str(&entry.format_for_display());
                        debug_info.push('\n');
                    }
                    debug_info.push_str(&format!("Total log entries: {}\n", log_buffer.len()));
                } else {
                    debug_info.push_str("Log buffer not initialized\n");
                }
                debug_info.push_str("================================\n");

                // Add DebugService logs (our StateManager logs!)
                if let Some(ref services) = self.service_container {
                    debug_info.push_str("\n========== STATE CHANGE LOGS ==========\n");
                    debug_info.push_str("(Most recent at bottom, from DebugService)\n");
                    let debug_entries = services.debug_service.get_entries();
                    let recent = debug_entries.iter().rev().take(50).rev(); // Last 50 entries
                    for entry in recent {
                        debug_info.push_str(&format!(
                            "[{}] {:?} [{}]: {}\n",
                            entry.timestamp, entry.level, entry.component, entry.message
                        ));
                    }
                    debug_info.push_str(&format!(
                        "Total state change entries: {}\n",
                        debug_entries.len()
                    ));
                    debug_info.push_str("================================\n");
                } else {
                    debug_info.push_str("\n========== STATE CHANGE LOGS ==========\n");
                    debug_info.push_str("DebugService not available (service_container is None)\n");
                    debug_info.push_str("================================\n");
                }

                // Add AppStateContainer debug dump if available
                if let Some(ref container) = self.state_container {
                    debug_info.push_str("\n");
                    debug_info.push_str(&container.debug_dump());
                    debug_info.push_str("\n");
                }

                // Set the final content in debug widget
                self.debug_widget.set_content(debug_info.clone());

                // Try to copy to clipboard
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => match clipboard.set_text(&debug_info) {
                        Ok(_) => {
                            // Verify clipboard write by reading it back
                            match clipboard.get_text() {
                                Ok(clipboard_content) => {
                                    let clipboard_len = clipboard_content.len();
                                    if clipboard_content == debug_info {
                                        self.buffer_mut().set_status_message(format!(
                                            "DEBUG INFO copied to clipboard ({} chars)!",
                                            clipboard_len
                                        ));
                                    } else {
                                        self.buffer_mut().set_status_message(format!(
                                                "Clipboard verification failed! Expected {} chars, got {} chars",
                                                debug_info.len(), clipboard_len
                                            ));
                                    }
                                }
                                Err(e) => {
                                    self.buffer_mut().set_status_message(format!(
                                        "Debug info copied but verification failed: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            self.buffer_mut()
                                .set_status_message(format!("Clipboard error: {}", e));
                        }
                    },
                    Err(e) => {
                        self.buffer_mut()
                            .set_status_message(format!("Can't access clipboard: {}", e));
                    }
                }
            }
        }
    }

    fn show_pretty_query(&mut self) {
        if let Some(buffer) = self.buffer_manager.current_mut() {
            buffer.set_mode(AppMode::PrettyQuery);
            let query = buffer.get_input_text();
            self.debug_widget.generate_pretty_sql(&query);
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

        // Set the file path for the first buffer if we have multiple files
        if data_files.len() > 1 {
            if let Some(buffer) = app.buffer_manager.current_mut() {
                buffer.set_file_path(Some(first_file.to_string()));
                let filename = std::path::Path::new(first_file)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                buffer.set_name(filename.to_string());
            }
        }

        // Load additional files into separate buffers
        if data_files.len() > 1 {
            for (_index, file_path) in data_files.iter().skip(1).enumerate() {
                let extension = std::path::Path::new(file_path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                match extension.to_lowercase().as_str() {
                    "csv" | "json" => {
                        // Get config value before mutable borrow
                        let case_insensitive = app.config.behavior.case_insensitive_default;

                        // Create a new buffer for each additional file
                        app.new_buffer();

                        // Get the current buffer and set it up
                        if let Some(buffer) = app.buffer_manager.current_mut() {
                            // Create and configure CSV client for this buffer
                            let mut csv_client = CsvApiClient::new();
                            csv_client.set_case_insensitive(case_insensitive);

                            // Get table name from file
                            let raw_name = std::path::Path::new(file_path)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("data")
                                .to_string();
                            let table_name = EnhancedTuiApp::sanitize_table_name(&raw_name);

                            // Load the data
                            if extension.to_lowercase() == "csv" {
                                if let Err(e) = csv_client.load_csv(file_path, &table_name) {
                                    app.buffer_mut().set_status_message(format!(
                                        "Error loading {}: {}",
                                        file_path, e
                                    ));
                                    continue;
                                }
                            } else {
                                if let Err(e) = csv_client.load_json(file_path, &table_name) {
                                    app.buffer_mut().set_status_message(format!(
                                        "Error loading {}: {}",
                                        file_path, e
                                    ));
                                    continue;
                                }
                            }

                            // Set the CSV client and metadata in the buffer
                            buffer.set_csv_client(Some(csv_client));
                            buffer.set_csv_mode(true);
                            buffer.set_table_name(table_name.clone());

                            info!(target: "buffer", "Loaded {} file '{}' into buffer {}: table='{}', case_insensitive={}", 
                                  extension.to_uppercase(), file_path, buffer.get_id(), table_name, case_insensitive);

                            // Set query
                            let query = format!("SELECT * FROM {}", table_name);
                            buffer.set_input_text(query);

                            // Store the file path and name
                            buffer.set_file_path(Some(file_path.to_string()));
                            let filename = std::path::Path::new(file_path)
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy();
                            buffer.set_name(filename.to_string());
                        }
                    }
                    _ => {
                        app.buffer_mut().set_status_message(format!(
                            "Skipping unsupported file: {}",
                            file_path
                        ));
                        continue;
                    }
                }
            }

            // Switch back to the first buffer
            app.buffer_manager.switch_to(0);

            app.buffer_mut().set_status_message(format!(
                "Loaded {} files into separate buffers. Use Alt+Tab to switch.",
                data_files.len()
            ));
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
