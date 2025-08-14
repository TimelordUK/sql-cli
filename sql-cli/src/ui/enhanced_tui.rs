use crate::api_client::{ApiClient, QueryResponse};
use crate::app_state_container::{AppStateContainer, SelectionMode};
use crate::buffer::{
    AppMode, BufferAPI, BufferManager, ColumnStatistics, ColumnType, EditMode, SortOrder, SortState,
};
use crate::buffer_handler::BufferHandler;
use crate::cell_renderer::CellRenderer;
use crate::config::config::Config;
use crate::cursor_manager::CursorManager;
use crate::data::adapters::BufferAdapter;
use crate::data::csv_datasource::CsvApiClient;
use crate::data::data_analyzer::DataAnalyzer;
use crate::data::data_exporter::DataExporter;
use crate::data::data_provider::DataProvider;
use crate::data::data_view::DataView;
// DataTable import removed - TUI only works with DataView
use crate::help_text::HelpText;
use crate::history::{CommandHistory, HistoryMatch};
use crate::key_chord_handler::{ChordResult, KeyChordHandler};
use crate::key_indicator::{format_key_for_display, KeyPressIndicator};
use crate::service_container::ServiceContainer;
use crate::sql::cache::QueryCache;
use crate::sql::hybrid_parser::HybridParser;
use crate::sql_highlighter::SqlHighlighter;
use crate::text_navigation::TextNavigator;
use crate::ui::key_dispatcher::KeyDispatcher;
use crate::utils::debug_info::DebugInfo;
use crate::utils::logging::{get_log_buffer, LogRingBuffer};
use crate::where_ast::format_where_ast;
use crate::where_parser::WhereParser;
use crate::widget_traits::DebugInfoProvider;
use crate::widgets::debug_widget::DebugWidget;
use crate::widgets::editor_widget::{BufferAction, EditorAction, EditorWidget};
use crate::widgets::help_widget::{HelpAction, HelpWidget};
use crate::widgets::search_modes_widget::{SearchMode, SearchModesAction, SearchModesWidget};
use crate::widgets::stats_widget::{StatsAction, StatsWidget};
use crate::yank_manager::YankManager;
use crate::{buffer, data_analyzer, dual_logging};
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
use std::io;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};
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

pub struct EnhancedTuiApp {
    // State container - manages all state
    state_container: std::sync::Arc<AppStateContainer>,
    // Service container for dependency injection
    service_container: Option<ServiceContainer>,

    api_client: ApiClient,
    input: Input,
    cursor_manager: CursorManager, // New: manages cursor/navigation logic
    data_analyzer: DataAnalyzer,   // New: manages data analysis/statistics
    hybrid_parser: HybridParser,

    // Configuration
    config: Config,

    // command_history: CommandHistory, // MIGRATED to AppStateContainer
    sql_highlighter: SqlHighlighter,
    debug_widget: DebugWidget,
    editor_widget: EditorWidget,
    stats_widget: StatsWidget,
    help_widget: HelpWidget,
    search_modes_widget: SearchModesWidget,
    key_chord_handler: KeyChordHandler, // Manages key sequences and history
    key_dispatcher: KeyDispatcher,      // Maps keys to actions

    // Selection and clipboard
    last_yanked: Option<(String, String)>, // (description, value) of last yanked item

    // Buffer management (new - for supporting multiple files)
    buffer_manager: BufferManager,
    buffer_handler: BufferHandler, // Handles buffer operations like switching

    // Performance tracking
    navigation_timings: Vec<String>, // Track last N navigation timings for debugging
    render_timings: Vec<String>,     // Track last N render timings for debugging
    // Cache
    query_cache: Option<QueryCache>,
    log_buffer: Option<LogRingBuffer>, // Ring buffer for debug logs

    // Visual enhancements
    cell_renderer: CellRenderer,
    key_indicator: KeyPressIndicator,
}

impl EnhancedTuiApp {
    // --- State Container Access ---
    // Helper methods for accessing the state container during migration

    /// Toggle help visibility
    fn toggle_help(&mut self) {
        self.state_container.toggle_help();
    }

    // --- Column Visibility Management ---

    /// Hide the currently selected column
    pub fn hide_current_column(&mut self) {
        debug!(
            "hide_current_column called, mode={:?}",
            self.buffer().get_mode()
        );
        if self.buffer().get_mode() != AppMode::Results {
            debug!("Not in Results mode, returning");
            return;
        }

        // Get current column index and name
        let col_idx = self.state_container.navigation().selected_column;
        debug!("Current column index: {}", col_idx);

        if let Some(dataview) = self.buffer().get_dataview() {
            let columns = dataview.column_names();
            // DataView already tracks hidden columns, so visible count is just the current count
            let visible_count = columns.len();
            debug!("Visible columns: {:?}, count: {}", columns, visible_count);

            if col_idx < columns.len() {
                let col_name = columns[col_idx].clone();

                // Don't hide if it's the last visible column
                // With DataView, columns.len() IS the visible count
                if visible_count > 1 {
                    if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
                        dataview.hide_column_by_name(&col_name);
                    }
                    debug!(
                        "Hiding column '{}', remaining visible: {}",
                        col_name,
                        visible_count - 1
                    );

                    // Force immediate re-render to reflect the change
                    debug!("Triggering immediate re-render after hiding column");

                    self.buffer_mut().set_status_message(format!(
                        "Hidden column: '{}' (Press + or = to unhide all)",
                        col_name
                    ));
                } else {
                    debug!("Cannot hide last visible column");
                    self.buffer_mut()
                        .set_status_message("Cannot hide last visible column".to_string());
                }
            }
        }
    }

    /// Unhide all columns
    pub fn unhide_all_columns(&mut self) {
        let hidden_columns = self
            .buffer()
            .get_dataview()
            .map(|v| v.get_hidden_column_names())
            .unwrap_or_default();
        if !hidden_columns.is_empty() {
            let count = hidden_columns.len();
            if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
                dataview.unhide_all_columns();
            }

            // Force immediate re-render to reflect the change
            debug!("Triggering immediate re-render after unhiding all columns");

            self.buffer_mut()
                .set_status_message(format!("Unhidden {} column(s)", count));
        }
    }

    /// Move the current column left in the view
    pub fn move_current_column_left(&mut self) {
        if self.buffer().get_mode() != AppMode::Results {
            return;
        }

        let col_idx = self.state_container.navigation().selected_column;

        // Get the current column name from DataView (not DataTable!)
        // This ensures we use the view's current column order
        if let Some(dataview) = self.buffer().get_dataview() {
            let columns = dataview.column_names();
            if col_idx < columns.len() {
                let col_name = columns[col_idx].clone();

                // Move the column in the DataView
                if let Some(dataview_mut) = self.buffer_mut().get_dataview_mut() {
                    // Use the visible column index directly
                    if dataview_mut.move_column_left(col_idx) {
                        // Cursor follows the column to its new position
                        // This allows pressing < multiple times to move the same column
                        if col_idx == 0 {
                            // Column wrapped to end, cursor follows to last position
                            let new_col_count = dataview_mut.column_count();
                            if new_col_count > 0 {
                                self.state_container.navigation_mut().selected_column =
                                    new_col_count - 1;
                                self.buffer_mut().set_current_column(new_col_count - 1);
                            }
                            self.buffer_mut()
                                .set_status_message(format!("Moved column '{}' to end", col_name));
                        } else {
                            // Normal move left, cursor follows to new position
                            let new_position = col_idx - 1;
                            self.state_container.navigation_mut().selected_column = new_position;
                            self.buffer_mut().set_current_column(new_position);
                            self.buffer_mut()
                                .set_status_message(format!("Moved column '{}' left", col_name));
                        }
                        debug!(
                            "Moved column '{}' left in DataView (from {} to {})",
                            col_name,
                            col_idx,
                            self.state_container.navigation().selected_column
                        );
                    }
                }
            }
        }
    }

    /// Move the current column right in the view
    pub fn move_current_column_right(&mut self) {
        if self.buffer().get_mode() != AppMode::Results {
            return;
        }

        let col_idx = self.state_container.navigation().selected_column;

        // Get the current column name from DataView (not DataTable!)
        // This ensures we use the view's current column order
        if let Some(dataview) = self.buffer().get_dataview() {
            let columns = dataview.column_names();
            if col_idx < columns.len() {
                let col_name = columns[col_idx].clone();
                let col_count = columns.len();

                // Move the column in the DataView
                if let Some(dataview_mut) = self.buffer_mut().get_dataview_mut() {
                    // Use the visible column index directly
                    if dataview_mut.move_column_right(col_idx) {
                        // Cursor follows the column to its new position
                        // This allows pressing > multiple times to move the same column
                        if col_idx == col_count - 1 {
                            // Column wrapped to beginning, cursor follows to first position
                            self.state_container.navigation_mut().selected_column = 0;
                            self.buffer_mut().set_current_column(0);
                            self.buffer_mut().set_status_message(format!(
                                "Moved column '{}' to beginning",
                                col_name
                            ));
                        } else {
                            // Normal move right, cursor follows to new position
                            let new_position = col_idx + 1;
                            self.state_container.navigation_mut().selected_column = new_position;
                            self.buffer_mut().set_current_column(new_position);
                            self.buffer_mut()
                                .set_status_message(format!("Moved column '{}' right", col_name));
                        }
                        debug!(
                            "Moved column '{}' right in DataView (from {} to {})",
                            col_name,
                            col_idx,
                            self.state_container.navigation().selected_column
                        );
                    }
                }
            }
        }
    }

    /// Set help visibility
    fn set_help_visible(&mut self, visible: bool) {
        self.state_container.set_help_visible(visible);
    }

    /// Get jump-to-row input text
    fn get_jump_to_row_input(&self) -> String {
        self.state_container.jump_to_row().input.clone()
    }

    /// Set jump-to-row input text
    fn set_jump_to_row_input(&mut self, input: String) {
        let old_value = self.get_jump_to_row_input();

        // Use unsafe to get mutable access through Arc
        let container_ptr = Arc::as_ptr(&self.state_container) as *mut AppStateContainer;
        unsafe {
            (*container_ptr).jump_to_row_mut().input = input.clone();
        }

        // Log the state change
        log_state_change!(
            self,
            "jump_to_row_input",
            old_value,
            input,
            "set_jump_to_row_input"
        );
    }

    /// Clear jump-to-row input
    fn clear_jump_to_row_input(&mut self) {
        // Use unsafe to get mutable access through Arc
        let container_ptr = Arc::as_ptr(&self.state_container) as *mut AppStateContainer;
        unsafe {
            (*container_ptr).jump_to_row_mut().input.clear();
        }

        // Log the state clear
        log_state_clear!(self, "jump_to_row_input", "clear_jump_to_row_input");
    }

    // Helper to get sort state from AppStateContainer
    fn get_sort_state(&self) -> SortState {
        let sort = self.state_container.sort();
        SortState {
            column: sort.column,
            order: sort.order.clone(),
        }
    }

    // --- Buffer Compatibility Layer ---
    // These methods provide a gradual migration path from direct field access to BufferAPI

    /// Get current buffer if available (for reading)
    fn current_buffer(&self) -> Option<&dyn buffer::BufferAPI> {
        self.buffer_manager
            .current()
            .map(|b| b as &dyn buffer::BufferAPI)
    }

    /// Get current buffer (panics if none exists)
    /// Use this when we know a buffer should always exist
    fn buffer(&self) -> &dyn buffer::BufferAPI {
        self.current_buffer()
            .expect("No buffer available - this should not happen")
    }

    // Note: current_buffer_mut removed - use buffer_manager.current_mut() directly

    /// Get current mutable buffer (panics if none exists)
    /// Use this when we know a buffer should always exist
    fn buffer_mut(&mut self) -> &mut buffer::Buffer {
        self.buffer_manager
            .current_mut()
            .expect("No buffer available - this should not happen")
    }

    /// Get a DataProvider view of the current buffer
    /// This allows using the new trait-based data access pattern
    fn get_data_provider(&self) -> Option<Box<dyn DataProvider + '_>> {
        // For now, we'll use BufferAdapter for Buffer data
        // In the future, we can check data source type and return appropriate adapter
        if let Some(buffer) = self.buffer_manager.current() {
            // V51: Check for DataView first, then DataTable
            if buffer.has_dataview() {
                return Some(Box::new(BufferAdapter::new(buffer)));
            }
        }
        None
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

        // Transaction-like block for input updates
        {
            let mut buffer = self.buffer_mut();
            buffer.set_input_text(text.clone());
            // Also sync cursor position to end of text
            buffer.set_input_cursor_position(text.len());
        }

        // Always update the input field for all modes
        // TODO: Eventually migrate special modes to use buffer input
        self.input = tui_input::Input::new(text.clone()).with_cursor(text.len());

        // IMPORTANT: Also sync with AppStateContainer's command_input to prevent desync
        self.state_container.set_input_text(text);
    }

    // Helper to set input text with specific cursor position
    fn set_input_text_with_cursor(&mut self, text: String, cursor_pos: usize) {
        let (old_text, old_cursor, mode) = {
            let buffer = self.buffer();
            let old_text = buffer.get_input_text();
            let old_cursor = buffer.get_input_cursor_position();
            let mode = buffer.get_mode();
            (old_text, old_cursor, mode)
        };

        // Log every input text change with cursor position
        info!(target: "input", "SET_INPUT_TEXT_WITH_CURSOR: '{}' (cursor {}) -> '{}' (cursor {}) (mode: {:?})", 
              if old_text.len() > 50 { format!("{}...", &old_text[..50]) } else { old_text.clone() },
              old_cursor,
              if text.len() > 50 { format!("{}...", &text[..50]) } else { text.clone() },
              cursor_pos,
              mode);

        // Transaction-like block for input updates
        {
            let buffer = self.buffer_mut();
            buffer.set_input_text(text.clone());
            buffer.set_input_cursor_position(cursor_pos);
        }

        // Always update the input field for consistency
        // TODO: Eventually migrate special modes to use buffer input
        self.input = tui_input::Input::new(text.clone()).with_cursor(cursor_pos);

        // IMPORTANT: Also sync with AppStateContainer's command_input to prevent desync
        self.state_container
            .set_input_text_with_cursor(text, cursor_pos);
    }

    // MASTER SYNC METHOD - Use this whenever input changes!
    // This ensures all three input states stay synchronized:
    // 1. Buffer's input_text and cursor
    // 2. self.input (tui_input widget)
    // 3. AppStateContainer's command_input
    fn sync_all_input_states(&mut self) {
        let buffer = self.buffer();
        let text = buffer.get_input_text();
        let cursor = buffer.get_input_cursor_position();
        let mode = buffer.get_mode();

        // Get caller for debugging
        let backtrace_str = std::backtrace::Backtrace::capture().to_string();
        let caller = backtrace_str
            .lines()
            .skip(3) // Skip backtrace frames
            .find(|line| line.contains("enhanced_tui") && !line.contains("sync_all_input_states"))
            .and_then(|line| line.split("::").last())
            .unwrap_or("unknown");

        // Update the tui_input widget
        self.input = tui_input::Input::new(text.clone()).with_cursor(cursor);

        // Sync with AppStateContainer
        self.state_container
            .set_input_text_with_cursor(text.clone(), cursor);

        // Reset horizontal scroll to show the cursor properly
        // This fixes the issue where switching between queries of different lengths
        // leaves the scroll offset in the wrong position
        self.cursor_manager.reset_horizontal_scroll();
        self.state_container.scroll_mut().input_scroll_offset = 0;

        // Update scroll to ensure cursor is visible
        self.update_horizontal_scroll(120); // Will be properly updated on next render

        info!(target: "input", "SYNC_ALL [{}]: text='{}', cursor={}, mode={:?}, scroll_reset", 
              caller,
              if text.len() > 50 { format!("{}...", &text[..50]) } else { text.clone() },
              cursor,
              mode);
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
        let buffer = self.buffer();
        let (text, cursor) = match buffer.get_mode() {
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                // Special modes use self.input directly
                (self.input.value().to_string(), self.input.cursor())
            }
            _ => {
                // Other modes use buffer
                (buffer.get_input_text(), buffer.get_input_cursor_position())
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

    // get_filter_state methods REMOVED - now use state_container.filter()

    fn get_selection_mode(&self) -> SelectionMode {
        self.state_container.get_selection_mode()
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

        // Log initialization
        if let Some(logger) = dual_logging::get_dual_logger() {
            logger.log(
                "INFO",
                "EnhancedTuiApp",
                &format!("Initializing TUI with API URL: {}", api_url),
            );
        }

        // Create buffer manager first

        let mut buffer_manager = BufferManager::new();
        let mut buffer = buffer::Buffer::new(1);
        // Sync initial settings from config
        buffer.set_case_insensitive(config.behavior.case_insensitive_default);
        buffer.set_compact_mode(config.display.compact_mode);
        buffer.set_show_row_numbers(config.display.show_row_numbers);
        buffer_manager.add_buffer(buffer);

        // Create a second buffer manager for the state container (temporary during migration)
        let mut container_buffer_manager = BufferManager::new();
        let mut container_buffer = buffer::Buffer::new(1);
        container_buffer.set_case_insensitive(config.behavior.case_insensitive_default);
        container_buffer.set_compact_mode(config.display.compact_mode);
        container_buffer.set_show_row_numbers(config.display.show_row_numbers);
        container_buffer_manager.add_buffer(container_buffer);

        // Initialize state container as Arc
        let state_container = match AppStateContainer::new(container_buffer_manager) {
            Ok(container) => std::sync::Arc::new(container),
            Err(e) => {
                panic!("Failed to initialize AppStateContainer: {}", e);
            }
        };

        // Initialize service container and help widget
        let services = ServiceContainer::new(state_container.clone());

        // Inject debug service into AppStateContainer (now works with RefCell)
        state_container.set_debug_service(services.debug_service.clone_service());

        // IMPORTANT: Enable the debug service so it actually logs!
        services.enable_debug();

        // Create help widget and set services
        let mut help_widget = HelpWidget::new();
        help_widget.set_services(services.clone_for_widget());

        let service_container = Some(services);

        Self {
            state_container,
            service_container,
            api_client: ApiClient::new(api_url),
            input: Input::default(),
            cursor_manager: CursorManager::new(),
            data_analyzer: DataAnalyzer::new(),
            // sql_parser: SqlParser::new(), // Not in struct anymore
            hybrid_parser: HybridParser::new(),
            config: config.clone(),
            // command_history: CommandHistory::new().unwrap_or_default(), // MIGRATED to AppStateContainer
            sql_highlighter: SqlHighlighter::new(),
            debug_widget: DebugWidget::new(),
            editor_widget: EditorWidget::new(),
            stats_widget: StatsWidget::new(),
            help_widget,
            search_modes_widget: SearchModesWidget::new(),
            key_chord_handler: KeyChordHandler::new(),
            key_dispatcher: KeyDispatcher::new(),
            last_yanked: None,
            // CSV fields now in Buffer
            buffer_manager,
            buffer_handler: BufferHandler::new(),
            navigation_timings: Vec::new(),
            render_timings: Vec::new(),
            query_cache: QueryCache::new().ok(),
            log_buffer: dual_logging::get_dual_logger().map(|logger| logger.ring_buffer().clone()),
            cell_renderer: CellRenderer::new(config.theme.cell_selection_style.clone()),
            key_indicator: {
                let mut indicator = KeyPressIndicator::new();
                if config.display.show_key_indicator {
                    indicator.set_enabled(true);
                }
                indicator
            },
        }
    }

    pub fn new_with_csv(csv_path: &str) -> Result<Self> {
        // First create the app to get its config
        let mut app = Self::new(""); // Empty API URL for CSV mode

        let raw_name = std::path::Path::new(csv_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data")
            .to_string();

        // Sanitize the table name to be SQL-friendly
        let table_name = Self::sanitize_table_name(&raw_name);

        // Direct DataTable loading is now the default for CSV files
        info!("Using direct DataTable loading for CSV (bypassing JSON intermediate)");
        crate::utils::memory_tracker::track_memory("before_direct_csv_load");

        // Load CSV directly to DataTable
        let datatable =
            crate::data::datatable_loaders::load_csv_to_datatable(csv_path, &table_name)?;

        crate::utils::memory_tracker::track_memory("after_direct_csv_load");
        info!(
            "Loaded {} rows directly to DataTable, memory: {} MB",
            datatable.row_count(),
            datatable.estimate_memory_size() / 1024 / 1024
        );

        // Create schema from DataTable columns
        let mut schema = std::collections::HashMap::new();
        schema.insert(table_name.clone(), datatable.column_names());

        // TEMPORARY: Also create CsvApiClient for complex query support
        // This will be removed once QueryEngine is implemented
        let mut csv_client = CsvApiClient::new();
        csv_client.set_case_insensitive(app.config.behavior.case_insensitive_default);
        csv_client.load_csv(csv_path, &table_name)?;
        info!("Created CsvApiClient as fallback for complex queries (temporary)");

        let (datatable_opt, schema) = (Some(datatable), schema);

        // Replace the default buffer with a CSV buffer using direct DataTable
        {
            // Clear all buffers and add a CSV buffer
            app.buffer_manager.clear_all();
            // Direct DataTable mode - create buffer with both DataTable and CSV client
            let mut buffer = buffer::Buffer::new(1);
            buffer.set_datatable(datatable_opt);
            info!("Created buffer with direct DataTable and CSV client fallback");
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

            // DataTable is already set directly from CSV file
        }

        Ok(app)
    }

    pub fn new_with_json(json_path: &str) -> Result<Self> {
        // First create the app to get its config
        let mut app = Self::new(""); // Empty API URL for JSON mode

        let raw_name = std::path::Path::new(json_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data")
            .to_string();

        // Sanitize the table name to be SQL-friendly
        let table_name = Self::sanitize_table_name(&raw_name);

        // Direct DataTable loading for JSON files
        info!("Using direct DataTable loading for JSON (bypassing intermediate format)");
        crate::utils::memory_tracker::track_memory("before_direct_json_load");

        // Load JSON directly to DataTable
        let datatable =
            crate::data::datatable_loaders::load_json_to_datatable(json_path, &table_name)?;

        crate::utils::memory_tracker::track_memory("after_direct_json_load");
        info!(
            "Loaded {} rows directly to DataTable from JSON, memory: {} MB",
            datatable.row_count(),
            datatable.estimate_memory_size() / 1024 / 1024
        );

        // Create schema from DataTable columns
        let mut schema = std::collections::HashMap::new();
        schema.insert(table_name.clone(), datatable.column_names());

        let datatable_opt = Some(datatable);

        // Replace the default buffer with a JSON buffer using direct DataTable
        {
            // Clear all buffers and add a buffer with DataTable
            app.buffer_manager.clear_all();
            let mut buffer = buffer::Buffer::new(1);
            buffer.set_datatable(datatable_opt);
            info!("Created buffer with direct DataTable and CSV client fallback");
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

            // DataTable is already set directly from JSON file
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
        // Initialize viewport size before first draw
        self.update_viewport_size();
        info!(target: "navigation", "Initial viewport size update completed");

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

                        // Record key press for visual indicator
                        self.key_indicator.record_key(format_key_for_display(&key));

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
        let normalized = self.state_container.normalize_key(key);

        // Get the action that will be performed (if any)
        let action = self
            .key_dispatcher
            .get_command_action(&normalized)
            .map(|s| s.to_string());

        // Log the key press
        if normalized != key {
            self.state_container
                .log_key_press(key, Some(format!("normalized to {:?}", normalized)));
        }
        self.state_container.log_key_press(normalized, action);

        let normalized_key = normalized;

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
                    eprintln!("[DEBUG] Using AppStateContainer for history search");
                    let current_input = self.get_input_text();

                    // Start history search
                    self.state_container.start_history_search(current_input);

                    // Initialize with schema context
                    self.update_history_matches_in_container();

                    // Get match count
                    let match_count = self.state_container.history_search().matches.len();

                    self.buffer_mut()
                        .set_status_message(format!("History search: {} matches", match_count));
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
                // Start history search mode
                let current_input = self.get_input_text();
                eprintln!(
                    "[DEBUG] Starting history search with input: '{}'",
                    current_input
                );

                // Start history search
                self.state_container.start_history_search(current_input);

                // Initialize with schema context
                self.update_history_matches_in_container();

                // Get status
                let is_active = self.state_container.is_history_search_active();
                let match_count = self.state_container.history_search().matches.len();

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
                    let mode = if self.buffer().has_dataview() {
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
                    } else if query == ":exit" || query == ":quit" || query == ":q" {
                        return Ok(true);
                    } else if query == ":tui" {
                        // Already in TUI mode
                        self.buffer_mut()
                            .set_status_message("Already in TUI mode".to_string());
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
                let history_entries = self
                    .state_container
                    .command_history()
                    .get_navigation_entries();
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

                        // Sync all input states
                        self.sync_all_input_states();
                        self.buffer_mut().set_status_message(debug_msg);
                    }
                }
            }
            // History navigation - Ctrl+N or Alt+Down
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Navigate to next command in history
                // Get history entries first, before mutable borrow
                let history_entries = self
                    .state_container
                    .command_history()
                    .get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.buffer_manager.current_mut() {
                    if buffer.navigate_history_down(&history_commands) {
                        // Sync all input states
                        self.sync_all_input_states();
                        self.buffer_mut()
                            .set_status_message("Next command from history".to_string());
                    }
                }
            }
            // Alternative: Alt+Up for history previous (in case Ctrl+P is intercepted)
            KeyCode::Up if key.modifiers.contains(KeyModifiers::ALT) => {
                let history_entries = self
                    .state_container
                    .command_history()
                    .get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.buffer_manager.current_mut() {
                    if buffer.navigate_history_up(&history_commands) {
                        // Sync all input states
                        self.sync_all_input_states();
                        self.buffer_mut()
                            .set_status_message("Previous command (Alt+Up)".to_string());
                    }
                }
            }
            // Alternative: Alt+Down for history next
            KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => {
                let history_entries = self
                    .state_container
                    .command_history()
                    .get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.buffer_manager.current_mut() {
                    if buffer.navigate_history_down(&history_commands) {
                        // Sync all input states
                        self.sync_all_input_states();
                        self.buffer_mut()
                            .set_status_message("Next command (Alt+Down)".to_string());
                    }
                }
            }
            KeyCode::F(8) => {
                // Toggle case-insensitive string comparisons
                let current = self.buffer().is_case_insensitive();
                self.buffer_mut().set_case_insensitive(!current);
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
            KeyCode::F(12) => {
                // Toggle key press indicator
                let enabled = !self.key_indicator.enabled;
                self.key_indicator.set_enabled(enabled);
                self.buffer_mut().set_status_message(format!(
                    "Key press indicator {}",
                    if enabled { "enabled" } else { "disabled" }
                ));
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
                if self.buffer().has_dataview()
                    && self.buffer().get_edit_mode() == EditMode::SingleLine =>
            {
                self.buffer_mut().set_mode(AppMode::Results);
                // Restore previous position or default to 0
                let row = self.buffer().get_last_results_row().unwrap_or(0);
                self.state_container.set_table_selected_row(Some(row));

                // Restore the exact scroll offset from when we left
                let last_offset = self.buffer().get_last_scroll_offset();
                self.buffer_mut().set_scroll_offset(last_offset);
            }
            KeyCode::F(5) => {
                // Use the unified debug handler
                self.toggle_debug_mode();
            }
            KeyCode::F(6) => {
                // V46: Demo DataTable conversion
                self.demo_datatable_conversion();
                // Don't switch to PrettyQuery mode immediately - let user see the DataTable message
            }
            KeyCode::F(7) => {
                // Pretty query view (moved from F6)
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
                self.state_container.clear_completion();

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
        let selection_mode = self.state_container.get_selection_mode();

        debug!(
            "handle_results_input: key={:?}, selection_mode={:?}",
            key, selection_mode
        );

        // Normalize the key for platform differences
        let normalized = self.state_container.normalize_key(key);

        // Get the action that will be performed (if any)
        let action = self
            .key_dispatcher
            .get_results_action(&normalized)
            .map(|s| s.to_string());

        // Log the key press
        if normalized != key {
            self.state_container
                .log_key_press(key, Some(format!("normalized to {:?}", normalized)));
        }
        self.state_container
            .log_key_press(normalized, action.clone());

        let normalized_key = normalized;

        // Debug uppercase G specifically
        if matches!(key.code, KeyCode::Char('G')) {
            debug!("Detected uppercase G key press!");
        }

        // Handle F6 for DataTable demo (V46)
        if matches!(key.code, KeyCode::F(6)) {
            self.demo_datatable_conversion();
            return Ok(false);
        }

        // Handle F12 for key indicator toggle
        if matches!(key.code, KeyCode::F(12)) {
            let enabled = !self.key_indicator.enabled;
            self.key_indicator.set_enabled(enabled);
            self.buffer_mut().set_status_message(format!(
                "Key press indicator {}",
                if enabled { "enabled" } else { "disabled" }
            ));
            return Ok(false);
        }

        // In cell mode, skip chord handler for 'y' key - handle it directly
        // Also skip uppercase single-key actions as they're not chords
        let should_skip_chord = (matches!(self.get_selection_mode(), SelectionMode::Cell)
            && matches!(normalized_key.code, KeyCode::Char('y')))
            || matches!(
                normalized_key.code,
                KeyCode::Char('G')
                    | KeyCode::Char('H')
                    | KeyCode::Char('M')
                    | KeyCode::Char('L')
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
                    "yank_query" => {
                        self.yank_query();
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
                    if let Some(selected) = self.state_container.get_table_selected_row() {
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
                        // Use the helper method to sync all three input states
                        self.set_input_text(last_query.clone());
                    } else if !current_input.is_empty() {
                        debug!(target: "buffer", "No last_query but input_text has content, keeping: '{}'", current_input);
                    } else {
                        debug!(target: "buffer", "No last_query to restore when exiting Results mode");
                    }

                    debug!(target: "mode", "Switching from Results to Command mode");
                    self.buffer_mut().set_mode(AppMode::Command);
                    self.state_container.set_table_selected_row(None);
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
                "goto_viewport_top" => self.goto_viewport_top(),
                "goto_viewport_middle" => self.goto_viewport_middle(),
                "goto_viewport_bottom" => self.goto_viewport_bottom(),
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
                "sort_by_column" => {
                    // TODO: Add toggle logic for sort direction
                    let ascending = true; // Default to ascending for now
                    self.sort_by_column(self.buffer().get_current_column(), ascending);
                    return Ok(false); // Event handled, continue running
                }
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

                    // Set jump-to-row state as active
                    let container_ptr =
                        Arc::as_ptr(&self.state_container) as *mut AppStateContainer;
                    unsafe {
                        (*container_ptr).jump_to_row_mut().is_active = true;
                    }

                    self.buffer_mut()
                        .set_status_message("Enter row number:".to_string());
                }
                "pin_column" => self.toggle_column_pin(),
                "clear_pins" => self.clear_all_pinned_columns(),
                "toggle_selection_mode" => {
                    self.state_container.toggle_selection_mode();
                    let new_mode = self.state_container.get_selection_mode();
                    let msg = match new_mode {
                        SelectionMode::Cell => "Cell mode - Navigate to select individual cells",
                        SelectionMode::Row => "Row mode - Navigate to select rows",
                        SelectionMode::Column => "Column mode - Navigate to select columns",
                    };
                    self.buffer_mut().set_status_message(msg.to_string());
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
                    self.buffer_mut().set_status_message(format!(
                        "Case-insensitive string comparisons: {}",
                        if !current { "ON" } else { "OFF" }
                    ));
                }
                "start_history_search" => {
                    // Switch to Command mode first
                    let last_query = self.buffer().get_last_query();

                    if !last_query.is_empty() {
                        // Use helper to sync all states
                        self.set_input_text(last_query.clone());
                    }

                    self.buffer_mut().set_mode(AppMode::Command);
                    self.state_container.set_table_selected_row(None);

                    // Start history search
                    let current_input = self.get_input_text();

                    // Start history search
                    self.state_container.start_history_search(current_input);

                    // Initialize with schema context
                    self.update_history_matches_in_container();

                    // Get match count
                    let match_count = self.state_container.history_search().matches.len();

                    self.buffer_mut()
                        .set_status_message(format!("History search: {} matches", match_count));

                    // Switch to History mode to show the search interface
                    self.buffer_mut().set_mode(AppMode::History);
                }
                _ => {
                    // Action not recognized, continue to handle key directly
                }
            }
        }

        // Fall back to direct key handling for special cases not in dispatcher
        match normalized_key.code {
            KeyCode::Char(' ') if !normalized_key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Toggle viewport lock with Space (but not Ctrl+Space) - using AppStateContainer
                self.state_container.toggle_viewport_lock();

                // Extract values we need before mutable borrows
                let (is_locked, lock_row, position_status) = {
                    let navigation = self.state_container.navigation();
                    (
                        navigation.viewport_lock,
                        navigation.viewport_lock_row,
                        navigation.get_position_status(),
                    )
                };

                // Update buffer state to match NavigationState
                self.buffer_mut().set_viewport_lock(is_locked);
                self.buffer_mut().set_viewport_lock_row(lock_row);

                if is_locked {
                    self.buffer_mut().set_status_message(format!(
                        "Viewport lock: ON (locked at row {}){}",
                        lock_row.map_or(0, |r| r + 1),
                        position_status
                    ));
                } else {
                    self.buffer_mut()
                        .set_status_message("Viewport lock: OFF (normal scrolling)".to_string());
                }
            }
            // Note: Many terminals can't distinguish Shift+Space from Space
            // So we support 'x' as an alternative for cursor lock
            KeyCode::Char('x') | KeyCode::Char('X') => {
                // Toggle cursor lock with 'x' key - using AppStateContainer
                self.state_container.toggle_cursor_lock();

                // Extract values we need before mutable borrows
                let (is_locked, lock_position) = {
                    let navigation = self.state_container.navigation();
                    (navigation.cursor_lock, navigation.cursor_lock_position)
                };

                // Update buffer state (we might need separate buffer fields for this)
                // For now, we'll just show status message
                if is_locked {
                    self.buffer_mut().set_status_message(format!(
                        "Cursor lock: ON (locked at visual position {})",
                        lock_position.map_or(0, |p| p + 1)
                    ));
                } else {
                    self.buffer_mut()
                        .set_status_message("Cursor lock: OFF (cursor moves normally)".to_string());
                }
            }
            KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Also support Ctrl+Space for cursor lock
                self.state_container.toggle_cursor_lock();

                // Extract values we need before mutable borrows
                let (is_locked, lock_position) = {
                    let navigation = self.state_container.navigation();
                    (navigation.cursor_lock, navigation.cursor_lock_position)
                };

                if is_locked {
                    self.buffer_mut().set_status_message(format!(
                        "Cursor lock: ON (locked at visual position {})",
                        lock_position.map_or(0, |p| p + 1)
                    ));
                } else {
                    self.buffer_mut()
                        .set_status_message("Cursor lock: OFF (cursor moves normally)".to_string());
                }
            }
            // Column visibility - Ctrl+H to hide current column
            // Note: Some terminals may intercept Ctrl+H as backspace
            KeyCode::Char('h')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                debug!("Ctrl+H pressed in Results mode - hiding current column");
                self.hide_current_column();
            }
            // Alternative: Backspace with Ctrl modifier (some terminals send this for Ctrl+H)
            KeyCode::Backspace if normalized_key.modifiers.contains(KeyModifiers::CONTROL) => {
                debug!("Ctrl+Backspace detected (may be Ctrl+H) - hiding current column");
                self.hide_current_column();
            }
            // Alternative keybinding: Alt+H to hide column (more reliable across terminals)
            KeyCode::Char('h') if normalized_key.modifiers.contains(KeyModifiers::ALT) => {
                debug!("Alt+H pressed - hiding current column");
                self.hide_current_column();
            }
            // Simple keybinding: minus key to hide column (only in Results mode)
            KeyCode::Char('-') if !normalized_key.modifiers.contains(KeyModifiers::CONTROL) => {
                debug!("Minus key pressed in Results mode - hiding current column");
                self.hide_current_column();
            }
            // Plus key to unhide all columns (only in Results mode)
            KeyCode::Char('+') | KeyCode::Char('=')
                if !normalized_key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                debug!("Plus/equals key pressed in Results mode - unhiding all columns");
                self.unhide_all_columns();
            }
            // Less-than key to move column left (only in Results mode)
            KeyCode::Char('<') if !normalized_key.modifiers.contains(KeyModifiers::CONTROL) => {
                debug!("< key pressed in Results mode - moving column left");
                self.move_current_column_left();
            }
            // Greater-than key to move column right (only in Results mode)
            KeyCode::Char('>') if !normalized_key.modifiers.contains(KeyModifiers::CONTROL) => {
                debug!("> key pressed in Results mode - moving column right");
                self.move_current_column_right();
            }
            // Column visibility - Ctrl+Shift+H to unhide all columns
            KeyCode::Char('H')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                debug!("Ctrl+Shift+H pressed in Results mode - unhiding all columns");
                self.unhide_all_columns();
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
            // Sort functionality (lowercase s) - handled by dispatcher above
            // Removed to prevent double handling
            // Column statistics (uppercase S only)
            KeyCode::Char('S') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.calculate_column_statistics();
            }
            // Clipboard operations (vim-like yank)
            KeyCode::Char('y') => {
                let selection_mode = self.get_selection_mode();
                debug!("'y' key pressed - selection_mode={:?}", selection_mode);
                match selection_mode {
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
                    SelectionMode::Column => {
                        // In column mode, 'y' yanks the current column
                        debug!("Yanking column in column selection mode");
                        self.buffer_mut()
                            .set_status_message("Yanking column...".to_string());
                        self.yank_column();
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
                    // TODO: Add toggle logic for sort direction
                    self.sort_by_column(column_index, true); // Default to ascending
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
                       self.buffer().get_dataview().map(|v| v.source().row_count()).unwrap_or(0));

                // Set search pattern in AppStateContainer
                self.state_container.start_search(pattern.clone());

                self.buffer_mut().set_search_pattern(pattern);
                self.perform_search();
                let matches_count = self.state_container.search().matches.len();
                debug!(target: "search", "After perform_search, app_mode={:?}, matches_found={}", 
                       self.buffer().get_mode(),
                       matches_count);
            }
            SearchMode::Filter => {
                debug!(target: "search", "Executing filter with pattern: '{}', app_mode={:?}", pattern, self.buffer().get_mode());
                debug!(target: "search", "Filter: case_insensitive={}, current results count={}", 
                       self.buffer().is_case_insensitive(),
                       self.buffer().get_dataview().map(|v| v.source().row_count()).unwrap_or(0));
                self.buffer_mut().set_filter_pattern(pattern.clone());
                self.state_container
                    .filter_mut()
                    .set_pattern(pattern.clone());
                debug!(target: "search", "After apply_filter, app_mode={:?}, filtered_count={}", 
                       self.buffer().get_mode(),
                self.buffer().get_dataview().map(|v| v.row_count()).unwrap_or(0));
            }
            SearchMode::FuzzyFilter => {
                debug!(target: "search", "Executing fuzzy filter with pattern: '{}', app_mode={:?}", pattern, self.buffer().get_mode());
                debug!(target: "search", "FuzzyFilter: current results count={}", 
                       self.buffer().get_dataview().map(|v| v.source().row_count()).unwrap_or(0));
                self.buffer_mut().set_fuzzy_filter_pattern(pattern);
                self.apply_fuzzy_filter();
                let indices_count = self.buffer().get_fuzzy_filter_indices().len();
                debug!(target: "search", "After apply_fuzzy_filter, app_mode={:?}, matched_indices={}", 
                       self.buffer().get_mode(), indices_count);
            }
            SearchMode::ColumnSearch => {
                debug!(target: "search", "Executing column search with pattern: '{}', app_mode={:?}", pattern, self.buffer().get_mode());

                // Use AppStateContainer for column search
                self.state_container.start_column_search(pattern.clone());

                // Pattern is now stored in AppStateContainer via start_column_search()
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
                // Clear search in AppStateContainer
                self.state_container.clear_search();
                self.buffer_mut().set_search_pattern(String::new());
            }
            SearchMode::Filter => {
                self.buffer_mut().set_filter_pattern(String::new());
                self.state_container.filter_mut().clear();
            }
            SearchMode::FuzzyFilter => {
                self.buffer_mut().set_fuzzy_filter_pattern(String::new());
                self.buffer_mut().set_fuzzy_filter_indices(Vec::new());
                self.buffer_mut().set_fuzzy_filter_active(false);
            }
            SearchMode::ColumnSearch => {
                // Use AppStateContainer to clear column search
                self.state_container.clear_column_search();

                // All column search state is now managed by AppStateContainer
            }
        }

        // Clear input field for search mode use
        self.input = tui_input::Input::default();
        // Note: Not syncing here as search modes use input differently
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
                        let mut filter = self.state_container.filter_mut();
                        filter.pattern = pattern.clone();
                        filter.is_active = true;
                    }
                    SearchMode::FuzzyFilter => {
                        self.buffer_mut().set_fuzzy_filter_pattern(pattern);
                    }
                    SearchMode::ColumnSearch => {
                        // Pattern is stored in AppStateContainer via start_column_search
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
                        {
                            let mut filter = self.state_container.filter_mut();
                            filter.pattern = pattern.clone();
                            filter.is_active = true;
                        } // filter borrow ends here
                        self.apply_filter("");
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

                        let column_info = {
                            let column_search = self.state_container.column_search();
                            if !column_search.matching_columns.is_empty() {
                                let current_match = column_search.current_match;
                                Some(column_search.matching_columns[current_match].clone())
                            } else {
                                None
                            }
                        };

                        if let Some((col_idx, col_name)) = column_info {
                            self.state_container.set_current_column(col_idx);
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
                        // Use helper to sync all states
                        self.set_input_text_with_cursor(sql, cursor);
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
                        self.state_container.filter_mut().clear();
                        self.buffer_mut().set_filter_pattern(String::new());
                        self.buffer_mut().set_filter_active(false);
                        // Re-apply empty filter to restore all results
                        self.apply_filter("");
                    }
                    AppMode::ColumnSearch => {
                        // Clear column search state using AppStateContainer
                        self.state_container.clear_column_search();
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
                        // Use helper to sync all states
                        self.set_input_text_with_cursor(sql, cursor);
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
                // Clear filter state using AppStateContainer
                self.state_container.filter_mut().clear();
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.buffer_mut().pop_undo() {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                }
                self.buffer_mut().set_mode(AppMode::Results);
            }
            KeyCode::Enter => {
                self.apply_filter("");
                // Restore original SQL query
                if let Some((original_query, cursor_pos)) = self.buffer_mut().pop_undo() {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                }
                self.buffer_mut().set_mode(AppMode::Results);
            }
            KeyCode::Backspace => {
                let pattern = {
                    let mut filter = self.state_container.filter_mut();
                    filter.pattern.pop(); // Remove last character
                    filter.pattern.clone() // Return the updated pattern
                };
                // Update input for rendering
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());
            }
            KeyCode::Char(c) => {
                let pattern = {
                    let mut filter = self.state_container.filter_mut();
                    filter.pattern.push(c);
                    filter.pattern.clone()
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
                // Clear fuzzy filter and return to results - transaction-like block
                let undo_state = {
                    let mut buffer = self.buffer_mut();
                    buffer.set_fuzzy_filter_active(false);
                    buffer.set_fuzzy_filter_pattern(String::new());
                    buffer.set_fuzzy_filter_indices(Vec::new());
                    let undo = buffer.pop_undo();
                    buffer.set_mode(AppMode::Results);
                    buffer.set_status_message("Fuzzy filter cleared".to_string());
                    undo
                };

                // Restore original SQL query if we had one
                if let Some((original_query, cursor_pos)) = undo_state {
                    self.set_input_text_with_cursor(original_query, cursor_pos);
                }
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
                    // Sync all input states
                    self.sync_all_input_states();
                }

                // Cancel column search and return to results
                self.state_container.clear_column_search();
                {
                    let mut buffer = self.buffer_mut();
                    buffer.set_mode(AppMode::Results);
                    buffer.set_status_message("Column search cancelled".to_string());
                }
            }
            KeyCode::Enter => {
                // Jump to first matching column
                if let Some((column_index, column_name)) =
                    self.state_container.accept_column_match()
                {
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
                    // Sync all input states
                    self.sync_all_input_states();
                }

                self.buffer_mut().set_mode(AppMode::Results);
            }
            KeyCode::Tab => {
                // Next match (Tab only, not 'n' to allow typing 'n' in search)
                if let Some((column_index, column_name)) = self.state_container.next_column_match()
                {
                    let (current_match, total_matches) = {
                        let column_search = self.state_container.column_search();
                        (
                            column_search.current_match + 1,
                            column_search.matching_columns.len(),
                        )
                    };
                    self.buffer_mut().set_current_column(column_index);
                    self.buffer_mut().set_status_message(format!(
                        "Column {} of {}: {}",
                        current_match, total_matches, column_name
                    ));
                }
            }
            KeyCode::BackTab => {
                // Previous match (Shift+Tab only, not 'N' to allow typing 'N' in search)
                if let Some((column_index, column_name)) =
                    self.state_container.previous_column_match()
                {
                    let (current_match, total_matches) = {
                        let column_search = self.state_container.column_search();
                        (
                            column_search.current_match + 1,
                            column_search.matching_columns.len(),
                        )
                    };
                    self.buffer_mut().set_current_column(column_index);
                    self.buffer_mut().set_status_message(format!(
                        "Column {} of {}: {}",
                        current_match, total_matches, column_name
                    ));
                }
            }
            KeyCode::Backspace => {
                let mut pattern = self.state_container.column_search().pattern.clone();
                pattern.pop();
                self.state_container.start_column_search(pattern.clone());
                // Also update input to keep it in sync for rendering
                self.set_input_text_with_cursor(pattern.clone(), pattern.len());
                self.update_column_search();
            }
            KeyCode::Char(c) => {
                let mut pattern = self.state_container.column_search().pattern.clone();
                pattern.push(c);
                self.state_container.start_column_search(pattern.clone());
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
                                      // Scroll is automatically reset when help is hidden in state_container
        let mode = if self.buffer().has_dataview() {
            AppMode::Results
        } else {
            AppMode::Command
        };
        self.buffer_mut().set_mode(mode);
    }

    fn scroll_help_down(&mut self) {
        let max_lines: usize = 58;
        let visible_height: usize = 30;

        self.state_container
            .set_help_max_scroll(max_lines, visible_height);
        self.state_container.help_scroll_down();
    }

    fn scroll_help_up(&mut self) {
        self.state_container.help_scroll_up();
    }

    fn help_page_down(&mut self) {
        let max_lines: usize = 58;
        let visible_height: usize = 30;

        self.state_container
            .set_help_max_scroll(max_lines, visible_height);
        self.state_container.help_page_down();
    }

    fn help_page_up(&mut self) {
        self.state_container.help_page_up();
    }

    fn handle_history_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Esc => {
                // Cancel history search and restore original input
                let original_input = self.state_container.cancel_history_search();
                self.set_input_text(original_input);
                self.buffer_mut().set_mode(AppMode::Command);
                self.buffer_mut()
                    .set_status_message("History search cancelled".to_string());
            }
            KeyCode::Enter => {
                // Accept the selected history command
                if let Some(command) = self.state_container.accept_history_search() {
                    // Set text with cursor at the beginning for better visibility
                    self.set_input_text_with_cursor(command, 0);
                    self.buffer_mut().set_mode(AppMode::Command);
                    self.buffer_mut().set_status_message(
                        "Command loaded from history (cursor at start)".to_string(),
                    );
                    // Sync to ensure scroll is reset properly
                    self.sync_all_input_states()
                }
            }
            KeyCode::Up => {
                self.state_container.history_search_previous();
            }
            KeyCode::Down => {
                self.state_container.history_search_next();
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+R cycles through matches
                self.state_container.history_search_next();
            }
            KeyCode::Backspace => {
                self.state_container.history_search_backspace();
                self.update_history_matches_in_container();
            }
            KeyCode::Char(c) => {
                self.state_container.history_search_add_char(c);
                self.update_history_matches_in_container();
            }
            _ => {}
        }
        Ok(false)
    }

    /// Update history matches in the AppStateContainer with schema context
    fn update_history_matches_in_container(&mut self) {
        // Get current schema columns and data source for better matching
        let (current_columns, current_source_str) =
            if let Some(dataview) = self.buffer().get_dataview() {
                (
                    dataview.column_names(),              // Gets visible columns
                    Some(dataview.source().name.clone()), // Gets table name from DataTable
                )
            } else {
                (vec![], None)
            };

        let current_source = current_source_str.as_deref();
        let query = self.state_container.history_search().query.clone();

        self.state_container.update_history_search_with_schema(
            query,
            &current_columns,
            current_source,
        );
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

        // 1. Save query to buffer and state container
        self.buffer_mut().set_last_query(query.to_string());
        self.state_container
            .set_last_executed_query(query.to_string());

        // 2. Update status
        self.buffer_mut()
            .set_status_message(format!("Executing query: '{}'...", query));
        let start_time = std::time::Instant::now();

        // 3. Execute query on DataView
        let result = if let Some(dataview) = self.buffer().get_dataview() {
            // Get the DataTable Arc (should add source_arc() method to DataView to avoid cloning)
            let table_arc = Arc::new(dataview.source().clone());
            let case_insensitive = self.buffer().is_case_insensitive();

            // Execute using QueryEngine
            let engine =
                crate::data::query_engine::QueryEngine::with_case_insensitive(case_insensitive);
            engine.execute(table_arc, query)
        } else {
            return Err(anyhow::anyhow!("No data loaded"));
        };

        // 4. Handle result
        match result {
            Ok(new_dataview) => {
                let duration = start_time.elapsed();
                let row_count = new_dataview.row_count();
                let col_count = new_dataview.column_count();

                // Store the new DataView in buffer
                self.buffer_mut().set_dataview(Some(new_dataview));

                // Update status
                self.buffer_mut().set_status_message(format!(
                    "Query executed: {} rows, {} columns ({} ms)",
                    row_count,
                    col_count,
                    duration.as_millis()
                ));

                // 5. Add to history
                let columns = self
                    .buffer()
                    .get_dataview()
                    .map(|v| v.column_names())
                    .unwrap_or_default();

                let table_name = self
                    .buffer()
                    .get_dataview()
                    .map(|v| v.source().name.clone())
                    .unwrap_or_else(|| "data".to_string());

                self.state_container
                    .command_history_mut()
                    .add_entry_with_schema(
                        query.to_string(),
                        true, // success
                        Some(duration.as_millis() as u64),
                        columns,
                        Some(table_name),
                    )?;

                // 6. Switch to results mode and reset navigation
                self.buffer_mut().set_mode(AppMode::Results);
                self.buffer_mut().set_selected_row(Some(0));
                self.buffer_mut().set_current_column(0);
                self.buffer_mut().set_scroll_offset((0, 0));

                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Query error: {}", e);
                self.buffer_mut().set_status_message(error_msg.clone());

                // Add failed query to history
                self.state_container.command_history_mut().add_entry(
                    query.to_string(),
                    false,
                    None,
                )?;

                Err(anyhow::anyhow!(error_msg))
            }
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

        // Use AppStateContainer for completion
        let is_same_context = self
            .state_container
            .is_same_completion_context(&query, cursor_pos);

        if !is_same_context {
            // New completion context - get fresh suggestions
            let hybrid_result = self.hybrid_parser.get_completions(&query, cursor_pos);
            if hybrid_result.suggestions.is_empty() {
                self.buffer_mut()
                    .set_status_message("No completions available".to_string());
                return;
            }

            self.state_container
                .set_completion_suggestions(hybrid_result.suggestions);
        } else if self.state_container.is_completion_active() {
            // Cycle to next suggestion
            self.state_container.next_completion();
        } else {
            self.buffer_mut()
                .set_status_message("No completions available".to_string());
            return;
        }

        // Get the current suggestion from AppStateContainer
        let suggestion = if let Some(sugg) = self.state_container.get_current_completion() {
            sugg
        } else {
            self.buffer_mut()
                .set_status_message("No completion selected".to_string());
            return;
        };
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
            self.state_container
                .update_completion_context(new_query.clone(), cursor_pos);

            let completion = self.state_container.completion();
            let suggestion_info = if completion.suggestions.len() > 1 {
                format!(
                    "Completed: {} ({}/{} - Tab for next)",
                    suggestion,
                    completion.current_index + 1,
                    completion.suggestions.len()
                )
            } else {
                format!("Completed: {}", suggestion)
            };
            drop(completion);
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
                // Sync all input states after undo/redo
                self.sync_all_input_states();
            }

            // Update completion state
            self.state_container
                .update_completion_context(new_query, cursor_pos_new);

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

    fn get_column_count(&self) -> usize {
        // Use DataProvider trait for column count (migration step)
        if let Some(provider) = self.get_data_provider() {
            provider.get_column_count()
        } else {
            0
        }
    }

    /// Get column count using DataProvider trait (new pattern)
    /// This demonstrates using the trait-based approach for column information
    fn get_column_count_via_provider(&self) -> usize {
        if let Some(provider) = self.get_data_provider() {
            provider.get_column_count()
        } else {
            0
        }
    }

    /// Get column names using DataProvider trait
    /// Part of the migration to trait-based data access
    fn get_column_names_via_provider(&self) -> Vec<String> {
        if let Some(provider) = self.get_data_provider() {
            provider.get_column_names()
        } else {
            Vec::new()
        }
    }

    /// Sort data using DataProvider (V44 migration helper)
    /// Returns sorted indices without modifying underlying data
    fn sort_via_provider(&self, column_index: usize, ascending: bool) -> Option<Vec<usize>> {
        let provider = self.get_data_provider()?;
        let row_count = provider.get_row_count();

        // Collect column values with their original indices
        let mut indexed_values: Vec<(String, usize)> = Vec::with_capacity(row_count);

        for row_idx in 0..row_count {
            if let Some(row) = provider.get_row(row_idx) {
                if column_index < row.len() {
                    indexed_values.push((row[column_index].clone(), row_idx));
                } else {
                    indexed_values.push((String::new(), row_idx));
                }
            }
        }

        // Sort by value, maintaining stable sort for equal values
        indexed_values.sort_by(|(a, _), (b, _)| {
            // Try numeric comparison first
            match (a.parse::<f64>(), b.parse::<f64>()) {
                (Ok(num_a), Ok(num_b)) => {
                    let cmp = num_a
                        .partial_cmp(&num_b)
                        .unwrap_or(std::cmp::Ordering::Equal);
                    if ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                }
                _ => {
                    // Fall back to string comparison
                    let cmp = a.cmp(b);
                    if ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                }
            }
        });

        // Extract the sorted indices
        Some(indexed_values.into_iter().map(|(_, idx)| idx).collect())
    }

    // Navigation functions
    fn next_row(&mut self) {
        use std::time::Instant;
        let start = Instant::now();

        let total_rows = self.get_row_count();
        let t1 = start.elapsed();

        if total_rows > 0 {
            // PERF: Don't update viewport size on every navigation
            // self.update_viewport_size();

            // Get column count and time it
            let total_cols = self.get_column_count();
            let t2 = start.elapsed();

            // Extract values we need before mutable borrows
            let (new_row, new_scroll_offset, t3, t4) = {
                let mut nav = self.state_container.navigation_mut();

                nav.update_totals(total_rows, total_cols);
                let t3_inner = start.elapsed();

                // Move to next row
                let result = if nav.next_row() {
                    (
                        Some(nav.selected_row),
                        nav.scroll_offset,
                        t3_inner,
                        start.elapsed(),
                    )
                } else {
                    (None, nav.scroll_offset, t3_inner, start.elapsed())
                };
                result
            };

            // Now we can use mutable self since we've dropped the nav borrow
            if let Some(row) = new_row {
                // Sync with local table_state for rendering
                self.state_container.set_table_selected_row(Some(row));

                // Sync with buffer's table state so it shows in debug and rendering
                self.buffer_mut().set_selected_row(Some(row));

                // Sync scroll offset with buffer
                self.buffer_mut().set_scroll_offset(new_scroll_offset);
            }

            let total = start.elapsed();

            // Store timing for debug display (keep last 20 timings)
            let timing_msg = format!("get_row_count={:?}, get_col_count={:?}, update_totals={:?}, nav={:?}, total={:?}, rows={}",
                t1, t2 - t1, t3 - t2, t4 - t3, total, total_rows);

            // Keep only the last 20 timings
            if self.navigation_timings.len() >= 20 {
                self.navigation_timings.remove(0);
            }
            self.navigation_timings.push(timing_msg.clone());

            // Debug output now available in F5 view, no need for stderr
            // eprintln!("next_row timing: {}", timing_msg);
        }
    }

    fn previous_row(&mut self) {
        // Use AppStateContainer for navigation
        // Extract values we need before mutable borrows
        let (new_row, new_scroll_offset) = {
            let mut nav = self.state_container.navigation_mut();

            // Update totals if needed
            let total_rows = self.get_row_count();
            let total_cols = self.get_column_count();
            nav.update_totals(total_rows, total_cols);

            // Move to previous row
            if nav.previous_row() {
                (Some(nav.selected_row), nav.scroll_offset)
            } else {
                (None, nav.scroll_offset)
            }
        };

        // Now we can use mutable self since we've dropped the nav borrow
        if let Some(row) = new_row {
            // Sync with local table_state for rendering
            self.state_container.set_table_selected_row(Some(row));

            // Sync with buffer's table state so it shows in debug and rendering
            self.buffer_mut().set_selected_row(Some(row));

            // Sync scroll offset with buffer
            self.buffer_mut().set_scroll_offset(new_scroll_offset);
        }
    }

    fn move_column_left(&mut self) {
        // Update cursor_manager for table navigation (incremental step)
        let (_row, _col) = self.cursor_manager.table_position();
        self.cursor_manager.move_table_left();

        // Keep existing logic for now
        let new_column = self.buffer().get_current_column().saturating_sub(1);
        self.buffer_mut().set_current_column(new_column);

        // Sync with navigation state in AppStateContainer
        self.state_container.navigation_mut().selected_column = new_column;

        let mut offset = self.buffer().get_scroll_offset();
        offset.1 = offset.1.saturating_sub(1);
        let column_num = self.buffer().get_current_column() + 1;
        self.buffer_mut().set_scroll_offset(offset);
        self.buffer_mut()
            .set_status_message(format!("Column {} selected", column_num));
    }

    fn move_column_right(&mut self) {
        // Use DataProvider trait to get column count
        let max_columns = if let Some(provider) = self.get_data_provider() {
            provider.get_column_count()
        } else {
            0
        };

        if max_columns > 0 {
            // Update cursor_manager for table navigation (incremental step)
            self.cursor_manager.move_table_right(max_columns);

            // Keep existing logic for now
            let current_column = self.buffer().get_current_column();
            if current_column + 1 < max_columns {
                self.buffer_mut().set_current_column(current_column + 1);

                // Sync with navigation state in AppStateContainer
                let new_column = current_column + 1;
                self.state_container.navigation_mut().selected_column = new_column;

                let mut offset = self.buffer().get_scroll_offset();
                offset.1 += 1;
                let column_num = self.buffer().get_current_column() + 1;
                self.buffer_mut().set_scroll_offset(offset);
                self.buffer_mut()
                    .set_status_message(format!("Column {} selected", column_num));
            }
        }
    }

    fn goto_first_column(&mut self) {
        self.buffer_mut().set_current_column(0);

        // Sync with navigation state in AppStateContainer
        self.state_container.navigation_mut().selected_column = 0;

        let mut offset = self.buffer().get_scroll_offset();
        offset.1 = 0;
        self.buffer_mut().set_scroll_offset(offset);
        self.buffer_mut()
            .set_status_message("First column selected".to_string());
    }

    fn goto_last_column(&mut self) {
        // Use DataProvider trait to get column count
        let max_columns = if let Some(provider) = self.get_data_provider() {
            provider.get_column_count()
        } else {
            0
        };

        if max_columns > 0 {
            let last_column = max_columns - 1;
            self.buffer_mut().set_current_column(last_column);

            // Sync with navigation state in AppStateContainer
            self.state_container.navigation_mut().selected_column = last_column;

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

    fn goto_first_row(&mut self) {
        // Update NavigationState
        {
            let mut nav = self.state_container.navigation_mut();
            nav.jump_to_first_row();
        } // nav borrow ends here

        self.state_container.set_table_selected_row(Some(0));
        let offset = {
            let mut offset = self.buffer().get_scroll_offset();
            offset.0 = 0; // Reset viewport to top
            offset
        }; // immutable borrow ends here
        self.buffer_mut().set_scroll_offset(offset);

        let total_rows = self.get_row_count();
        if total_rows > 0 {
            self.buffer_mut()
                .set_status_message(format!("Jumped to first row (1/{})", total_rows));
        }
    }

    fn goto_viewport_top(&mut self) {
        let (new_row, status_msg) = {
            let mut nav = self.state_container.navigation_mut();
            nav.jump_to_viewport_top();
            let row = nav.selected_row;
            let total = nav.total_rows;
            (
                row,
                format!("Jumped to viewport top (row {}/{})", row + 1, total),
            )
        };

        self.state_container.set_table_selected_row(Some(new_row));
        self.buffer_mut().set_status_message(status_msg);
    }

    fn goto_viewport_middle(&mut self) {
        // Jump to middle of current viewport (M in vim)

        let (new_row, status_msg) = {
            let mut nav = self.state_container.navigation_mut();
            nav.jump_to_viewport_middle();
            let row = nav.selected_row;
            let total = nav.total_rows;
            (
                row,
                format!("Jumped to viewport middle (row {}/{})", row + 1, total),
            )
        };

        self.state_container.set_table_selected_row(Some(new_row));
        self.buffer_mut().set_status_message(status_msg);
    }

    fn goto_viewport_bottom(&mut self) {
        // Jump to bottom of current viewport (L in vim)
        let (new_row, status_msg) = {
            let mut nav = self.state_container.navigation_mut();
            nav.jump_to_viewport_bottom();
            let row = nav.selected_row;
            let total = nav.total_rows;
            (
                row,
                format!("Jumped to viewport bottom (row {}/{})", row + 1, total),
            )
        };

        self.state_container.set_table_selected_row(Some(new_row));
        self.buffer_mut().set_status_message(status_msg);
    }

    fn toggle_column_pin(&mut self) {
        // Pin or unpin the current column using DataView
        let current_col = self.buffer().get_current_column();

        if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
            if dataview.is_column_pinned(current_col) {
                // Column is already pinned, unpin it
                dataview.unpin_column(current_col);
                self.buffer_mut()
                    .set_status_message(format!("Column {} unpinned", current_col + 1));
            } else {
                // Try to pin the column
                match dataview.pin_column(current_col) {
                    Ok(_) => {
                        self.buffer_mut()
                            .set_status_message(format!("Column {} pinned 📌", current_col + 1));
                    }
                    Err(e) => {
                        self.buffer_mut().set_status_message(e.to_string());
                    }
                }
            }
        }
    }

    fn clear_all_pinned_columns(&mut self) {
        if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
            dataview.clear_pinned_columns();
        }
        self.buffer_mut()
            .set_status_message("All columns unpinned".to_string());
    }

    fn calculate_column_statistics(&mut self) {
        use std::time::Instant;

        let start_total = Instant::now();

        // Collect all data first, then drop the buffer reference before calling analyzer
        let (column_name, data_to_analyze) = {
            // Get column names using DataProvider trait
            let headers = self.get_column_names_via_provider();
            if headers.is_empty() {
                return;
            }

            let current_column = self.buffer().get_current_column();
            if current_column >= headers.len() {
                return;
            }

            let column_name = headers[current_column].clone();

            // Extract column data using DataProvider trait
            let data_to_analyze: Vec<String> = if let Some(provider) = self.get_data_provider() {
                let row_count = provider.get_row_count();
                let mut column_data = Vec::with_capacity(row_count);

                for row_idx in 0..row_count {
                    if let Some(row) = provider.get_row(row_idx) {
                        if current_column < row.len() {
                            column_data.push(row[current_column].clone());
                        } else {
                            // Handle missing column data
                            column_data.push(String::new());
                        }
                    }
                }

                column_data
            } else {
                // No data provider available
                return;
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
                data_analyzer::ColumnType::Integer | data_analyzer::ColumnType::Float => {
                    ColumnType::Numeric
                }
                data_analyzer::ColumnType::String
                | data_analyzer::ColumnType::Boolean
                | data_analyzer::ColumnType::Date => ColumnType::String,
                data_analyzer::ColumnType::Mixed => ColumnType::Mixed,
                data_analyzer::ColumnType::Unknown => ColumnType::Mixed,
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
        if let Ok((width, height)) = crossterm::terminal::size() {
            let terminal_width = width as usize;
            let terminal_height = height as usize;

            info!(target: "navigation", "update_viewport_size - terminal dimensions: {}x{}", terminal_width, terminal_height);

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
            let visible_rows = results_area_height.saturating_sub(3).max(10);

            // Update buffer's last_visible_rows
            self.buffer_mut().set_last_visible_rows(visible_rows);

            // Update NavigationState's viewport dimensions

            self.state_container
                .navigation_mut()
                .set_viewport_size(visible_rows, terminal_width);

            info!(target: "navigation", "update_viewport_size - viewport set to: {}x{} rows", visible_rows, terminal_width);
        }
    }

    fn goto_last_row(&mut self) {
        let total_rows = self.get_row_count();
        if total_rows > 0 {
            let last_row = total_rows - 1;
            // Update NavigationState
            {
                let mut nav = self.state_container.navigation_mut();
                nav.jump_to_last_row();
            }

            self.state_container.set_table_selected_row(Some(last_row));
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
            let current = self.state_container.get_table_selected_row().unwrap_or(0);
            let new_position = (current + visible_rows).min(total_rows - 1);

            self.state_container
                .set_table_selected_row(Some(new_position));

            // Scroll viewport down by a page
            let mut offset = self.buffer().get_scroll_offset();
            offset.0 = (offset.0 + visible_rows).min(total_rows.saturating_sub(visible_rows));
            self.buffer_mut().set_scroll_offset(offset);
        }
    }

    fn page_up(&mut self) {
        let visible_rows = self.buffer().get_last_visible_rows();
        let current = self.state_container.get_table_selected_row().unwrap_or(0);
        let new_position = current.saturating_sub(visible_rows);

        self.state_container
            .set_table_selected_row(Some(new_position));

        // Scroll viewport up by a page
        let mut offset = self.buffer().get_scroll_offset();
        offset.0 = offset.0.saturating_sub(visible_rows);
        self.buffer_mut().set_scroll_offset(offset);
    }

    // Search and filter functions
    fn perform_search(&mut self) {
        if let Some(dataview) = self.get_current_data() {
            // Convert DataView rows to Vec<Vec<String>> for AppStateContainer
            let data: Vec<Vec<String>> = (0..dataview.row_count())
                .filter_map(|i| dataview.get_row(i))
                .map(|row| row.values.iter().map(|v| v.to_string()).collect())
                .collect();

            // Perform search using AppStateContainer
            let matches = self.state_container.perform_search(&data);

            // Update buffer with matches for now (until we fully migrate)
            let buffer_matches: Vec<(usize, usize)> = matches
                .iter()
                .map(|(row, col, _, _)| (*row, *col))
                .collect();

            if !buffer_matches.is_empty() {
                let (row, _) = buffer_matches[0];
                self.state_container.set_table_selected_row(Some(row));

                let buffer = self.buffer_mut();
                buffer.set_search_matches(buffer_matches.clone());
                buffer.set_search_match_index(0);
                buffer.set_current_match(Some(buffer_matches[0]));
                buffer.set_status_message(format!("Found {} matches", buffer_matches.len()));
            } else {
                let buffer = self.buffer_mut();
                buffer.set_status_message("No matches found".to_string());
                buffer.set_search_matches(buffer_matches.clone());
            }
        }
    }

    fn next_search_match(&mut self) {
        // Use AppStateContainer for search navigation if available

        if let Some((row, col)) = self.state_container.next_search_match() {
            // Extract values before mutable borrows
            let current_idx = self.state_container.search().current_match + 1;
            let total = self.state_container.search().matches.len();
            let search_match_index = self.state_container.search().current_match;

            // Now do mutable operations
            self.state_container.set_table_selected_row(Some(row));
            self.buffer_mut().set_current_match(Some((row, col)));
            self.buffer_mut()
                .set_status_message(format!("Match {} of {}", current_idx, total));
            self.buffer_mut().set_search_match_index(search_match_index);
        } else {
            self.buffer_mut()
                .set_status_message("No search matches".to_string());
        }
    }

    fn previous_search_match(&mut self) {
        // Use AppStateContainer for search navigation if available
        if let Some((row, col)) = self.state_container.previous_search_match() {
            // Extract values before mutable borrows
            let current_idx = self.state_container.search().current_match + 1;
            let total = self.state_container.search().matches.len();
            let search_match_index = self.state_container.search().current_match;

            // Now do mutable operations
            self.state_container.set_table_selected_row(Some(row));
            self.buffer_mut().set_current_match(Some((row, col)));
            self.buffer_mut()
                .set_status_message(format!("Match {} of {}", current_idx, total));
            self.buffer_mut().set_search_match_index(search_match_index);
        } else {
            self.buffer_mut()
                .set_status_message("No search matches".to_string());
        }
    }

    fn apply_filter(&mut self, pattern: &str) {
        let case_insensitive = { self.buffer().is_case_insensitive() };
        if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
            dataview.apply_text_filter(pattern, !case_insensitive);

            let status = if pattern.is_empty() {
                "Filter cleared".to_string()
            } else {
                format!("Filter applied: {} matches", dataview.row_count())
            };
            self.buffer_mut().set_status_message(status);
        }
    }
    fn search_columns(&mut self) {
        let pattern = self.state_container.column_search().pattern.clone();
        debug!(target: "search", "search_columns called with pattern: '{}'", pattern);
        if pattern.is_empty() {
            debug!(target: "search", "Pattern is empty, skipping column search");
            return;
        }

        // Get columns using the new unified method
        let column_names = self.buffer().get_column_names();
        let mut columns = Vec::new();
        for (index, col_name) in column_names.iter().enumerate() {
            columns.push((col_name.clone(), index));
        }

        debug!(target: "search", "Got {} columns from buffer", columns.len());
        if !columns.is_empty() {
            debug!(target: "search", "Column names: {:?}", columns.iter().map(|(name, _)| name).collect::<Vec<_>>());
        }

        // Use AppStateContainer for column search
        let matching_columns = self
            .state_container
            .update_column_search_matches(&columns, &pattern);
        debug!(target: "search", "Found {} matching columns", matching_columns.len());
        if !matching_columns.is_empty() {
            for (idx, (col_idx, col_name)) in matching_columns.iter().enumerate() {
                debug!(target: "search", "  Match {}: '{}' at index {}", idx + 1, col_name, col_idx);
            }
        }

        if !matching_columns.is_empty() {
            // Move to first match
            let first_match_index = matching_columns[0].0;
            let first_match_name = &matching_columns[0].1;

            self.state_container.set_current_column(first_match_index);
            self.buffer_mut().set_current_column(first_match_index);

            debug!(target: "search", "Setting current column to index {} ('{}')", 
                   first_match_index, first_match_name);
            let status_msg = format!(
                "Found {} columns matching '{}'. Tab/Shift-Tab to navigate.",
                matching_columns.len(),
                pattern
            );
            debug!(target: "search", "Setting status: {}", status_msg);
            self.buffer_mut().set_status_message(status_msg);

            // Column search matches are now managed by AppStateContainer
        } else {
            let status_msg = format!("No columns matching '{}'", pattern);
            debug!(target: "search", "Setting status: {}", status_msg);
            self.buffer_mut().set_status_message(status_msg);
        }

        // Matching columns are now stored in AppStateContainer
    }

    fn next_column_match(&mut self) {
        // Use AppStateContainer for navigation
        if let Some((col_index, col_name)) = self.state_container.next_column_match() {
            // Get the info we need before mutating self
            let (current_match, total_matches) = {
                let column_search = self.state_container.column_search();
                (
                    column_search.current_match + 1,
                    column_search.matching_columns.len(),
                )
            };

            // Now we can mutate self
            self.state_container.set_current_column(col_index);
            self.buffer_mut().set_current_column(col_index);
            self.buffer_mut().set_status_message(format!(
                "Column {}/{}: {} - Tab/Shift-Tab to navigate",
                current_match, total_matches, col_name
            ));
        }
    }

    fn previous_column_match(&mut self) {
        // Use AppStateContainer for navigation

        if let Some((col_index, col_name)) = self.state_container.previous_column_match() {
            // Get the info we need before mutating self
            let (current_match, total_matches) = {
                let column_search = self.state_container.column_search();
                (
                    column_search.current_match + 1,
                    column_search.matching_columns.len(),
                )
            };

            // Now we can mutate self
            self.state_container.set_current_column(col_index);
            self.buffer_mut().set_current_column(col_index);
            self.buffer_mut().set_status_message(format!(
                "Column {}/{}: {} - Tab/Shift-Tab to navigate",
                current_match, total_matches, col_name
            ));
        }
    }

    fn apply_fuzzy_filter(&mut self) {
        let pattern = self.buffer().get_fuzzy_filter_pattern();
        let case_insensitive = self.buffer().is_case_insensitive();

        // Apply filter and get results
        let (match_count, indices) = if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
            dataview.apply_fuzzy_filter(&pattern, case_insensitive);
            let match_count = dataview.row_count();
            let indices = dataview.get_fuzzy_filter_indices();
            (match_count, indices)
        } else {
            (0, Vec::new())
        };

        // Update buffer state after releasing the borrow
        if pattern.is_empty() {
            self.buffer_mut().set_fuzzy_filter_active(false);
            self.buffer_mut()
                .set_status_message("Fuzzy filter cleared".to_string());
        } else {
            self.buffer_mut().set_fuzzy_filter_active(true);
            self.buffer_mut()
                .set_status_message(format!("Fuzzy filter: {} matches", match_count));
        }

        // Update fuzzy filter indices for compatibility
        self.buffer_mut().set_fuzzy_filter_indices(indices);
    }

    fn update_column_search(&mut self) {
        // Get column headers using DataProvider - extract data in a scoped block
        let column_data = if let Some(provider) = self.get_data_provider() {
            let headers = provider.get_column_names();

            // Create columns list for AppStateContainer
            let columns: Vec<(String, usize)> = headers
                .iter()
                .enumerate()
                .map(|(idx, name)| (name.clone(), idx))
                .collect();

            Some((headers, columns))
        } else {
            None
        };

        // Now provider borrow is dropped, we can use mutable methods
        if let Some((_headers, columns)) = column_data {
            // Update matches in AppStateContainer
            let pattern = self.state_container.column_search().pattern.clone();
            self.state_container
                .update_column_search_matches(&columns, &pattern);

            // Update status message
            if pattern.is_empty() {
                self.buffer_mut()
                    .set_status_message("Enter column name to search".to_string());
            } else {
                let (matching_columns, matches_len) = {
                    let column_search = self.state_container.column_search();
                    (
                        column_search.matching_columns.clone(),
                        column_search.matching_columns.len(),
                    )
                };
                if matching_columns.is_empty() {
                    self.buffer_mut()
                        .set_status_message(format!("No columns match '{}'", pattern));
                } else {
                    let (column_index, column_name) = matching_columns[0].clone();
                    self.buffer_mut().set_current_column(column_index);
                    self.buffer_mut().set_status_message(format!(
                        "Column 1 of {}: {} (Tab=next, Enter=select)",
                        matches_len, column_name
                    ));
                }
            }
        } else {
            self.buffer_mut()
                .set_status_message("No results available for column search".to_string());
        }
    }

    fn sort_by_column(&mut self, column_index: usize, ascending: bool) {
        if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
            if let Err(e) = dataview.apply_sort(column_index, ascending) {
                self.buffer_mut()
                    .set_status_message(format!("Sort error: {}", e));
            } else {
                self.buffer_mut().set_status_message(format!(
                    "Sorted by column {} ({})",
                    column_index,
                    if ascending { "ascending" } else { "descending" }
                ));
            }
        }
    }

    fn get_current_data(&self) -> Option<&DataView> {
        self.buffer().get_dataview()
    }

    fn get_row_count(&self) -> usize {
        // Check if fuzzy filter is active first (most specific filter)
        if self.buffer().is_fuzzy_filter_active() {
            // Return the count of fuzzy filtered indices
            self.buffer().get_fuzzy_filter_indices().len()
        } else if let Some(dataview) = self.buffer().get_dataview() {
            // Return count from WHERE clause or other filters
            dataview.row_count()
        } else if let Some(provider) = self.get_data_provider() {
            // Use DataProvider trait for data access (migration step)
            provider.get_row_count()
        } else {
            0
        }
    }

    /// Get row count using DataProvider trait (new pattern)
    /// This is a parallel implementation that uses the trait-based approach
    fn get_row_count_via_provider(&self) -> usize {
        // First check for filters - these still need buffer access for now
        if self.buffer().is_fuzzy_filter_active() {
            return self.buffer().get_fuzzy_filter_indices().len();
        } else if let Some(filtered) = self.buffer().get_dataview() {
            return filtered.row_count();
        }

        // Use DataProvider for unfiltered data
        if let Some(provider) = self.get_data_provider() {
            provider.get_row_count()
        } else {
            0
        }
    }

    fn reset_table_state(&mut self) {
        self.state_container.set_table_selected_row(Some(0));

        // Transaction-like block for multiple buffer resets
        {
            let mut buffer = self.buffer_mut();
            buffer.set_scroll_offset((0, 0));
            buffer.set_current_column(0);
            buffer.set_last_results_row(None); // Reset saved position for new results
            buffer.set_last_scroll_offset((0, 0)); // Reset saved scroll offset for new results
        }

        // Clear filter state to prevent old filtered data from persisting
        // Clear filter state in container
        self.state_container.filter_mut().clear();

        // Clear search state
        {
            let mut search = self.state_container.search_mut();
            search.pattern = String::new();
            search.current_match = 0;
            search.matches = Vec::new();
            search.is_active = false;
        }

        // Clear fuzzy filter state to prevent it from persisting across queries
        {
            let buffer = self.buffer_mut();
            buffer.clear_fuzzy_filter();
            buffer.set_fuzzy_filter_pattern(String::new());
            buffer.set_fuzzy_filter_active(false);
            buffer.set_fuzzy_filter_indices(Vec::new());
        };
    }

    fn calculate_viewport_column_widths(&mut self, viewport_start: usize, viewport_end: usize) {
        // Calculate column widths based on DataView
        if let Some(dataview) = self.buffer().get_dataview() {
            let headers = dataview.column_names();
            let mut widths = Vec::with_capacity(headers.len());

            // Use compact mode settings
            let compact = self.buffer().is_compact_mode();
            let min_width = if compact { 4 } else { 6 };
            let max_width = if compact { 20 } else { 30 };
            let padding = if compact { 1 } else { 2 };

            // PERF FIX: Only convert viewport rows to strings, not entire table!
            // Get string representation of ONLY visible rows to avoid converting 100k rows
            let mut rows_to_check = Vec::new();
            let source_table = dataview.source();
            for i in viewport_start..viewport_end.min(source_table.row_count()) {
                if let Some(row_strings) = source_table.get_row_as_strings(i) {
                    rows_to_check.push(row_strings);
                }
            }

            for (col_idx, header) in headers.iter().enumerate() {
                // Start with header width
                let mut max_col_width = header.len();

                // Check only visible rows for this column
                for row in &rows_to_check {
                    if let Some(value) = row.get(col_idx) {
                        let display_value = if value.is_empty() {
                            "NULL"
                        } else {
                            value.as_str()
                        };
                        max_col_width = max_col_width.max(display_value.len());
                    }
                }

                // Apply min/max constraints and padding
                let width = (max_col_width + padding).clamp(min_width, max_width) as u16;
                widths.push(width);
            }

            self.buffer_mut().set_column_widths(widths);
        }
    }

    fn update_parser_for_current_buffer(&mut self) {
        // Sync input states
        self.sync_all_input_states();

        // Update parser schema from DataView
        if let Some(dataview) = self.buffer().get_dataview() {
            let table_name = dataview.source().name.clone();
            let columns = dataview.source().column_names();

            debug!(target: "buffer", "Updating parser with {} columns for table '{}'", columns.len(), table_name);
            self.hybrid_parser.update_single_table(table_name, columns);
        }
    }

    fn calculate_optimal_column_widths(&mut self) {
        // Use DataProvider trait to get column widths
        let widths_u16 = if let Some(provider) = self.get_data_provider() {
            let widths = provider.get_column_widths();
            if !widths.is_empty() {
                // Convert usize to u16 for buffer compatibility
                Some(
                    widths
                        .iter()
                        .map(|&w| w.min(u16::MAX as usize) as u16)
                        .collect::<Vec<u16>>(),
                )
            } else {
                None
            }
        } else {
            None
        };

        // Now the provider borrow is dropped, we can mutably borrow self
        if let Some(widths) = widths_u16 {
            self.buffer_mut().set_column_widths(widths);
        }
    }

    /// Centralized method for setting status messages
    /// Ensures consistent logging and state synchronization
    fn set_status_message(&mut self, message: impl Into<String>) {
        let msg = message.into();
        debug!("Status: {}", msg);
        self.buffer_mut().set_status_message(msg.clone());
        // Future: Could also sync to state_container if needed
        // self.state_container.set_status(msg);
    }

    /// Set error status message with consistent formatting
    fn set_error_status(&mut self, context: &str, error: impl std::fmt::Display) {
        let msg = format!("{}: {}", context, error);
        debug!("Error status: {}", msg);
        self.set_status_message(msg);
    }

    fn export_to_csv(&mut self) {
        // Use trait-based export with DataProvider
        let result = if let Some(provider) = self.get_data_provider() {
            DataExporter::export_provider_to_csv(provider.as_ref())
        } else {
            Err(anyhow::anyhow!("No data available to export"))
        };

        match result {
            Ok(message) => self.set_status_message(message),
            Err(e) => self.set_error_status("Export failed", e),
        }
    }

    fn yank_cell(&mut self) {
        debug!("yank_cell called");
        if let Some(selected_row) = self.state_container.get_table_selected_row() {
            let column = self.buffer().get_current_column();
            debug!("Yanking cell at row={}, column={}", selected_row, column);
            match YankManager::yank_cell(self.buffer(), &self.state_container, selected_row, column)
            {
                Ok(result) => {
                    // YankManager already handled clipboard via AppStateContainer
                    // Keep local copy for backward compatibility (will be removed later)
                    self.last_yanked = Some((result.description.clone(), result.preview.clone()));
                    self.set_status_message(format!("Yanked cell: {}", result.full_value));
                }
                Err(e) => {
                    self.set_error_status("Failed to yank cell", e);
                }
            }
        } else {
            debug!("No row selected for yank");
        }
    }

    fn yank_row(&mut self) {
        if let Some(selected_row) = self.state_container.get_table_selected_row() {
            match YankManager::yank_row(self.buffer(), &self.state_container, selected_row) {
                Ok(result) => {
                    // YankManager already handled clipboard via AppStateContainer
                    // Keep local copy for backward compatibility
                    self.last_yanked = Some((result.description.clone(), result.preview));
                    self.set_status_message(format!("Yanked {}", result.description));
                }
                Err(e) => {
                    self.set_error_status("Failed to yank row", e);
                }
            }
        }
    }

    fn yank_column(&mut self) {
        let column = self.buffer().get_current_column();
        match YankManager::yank_column(self.buffer(), &self.state_container, column) {
            Ok(result) => {
                // YankManager already handled clipboard via AppStateContainer
                // Keep local copy for backward compatibility
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
        match YankManager::yank_all(self.buffer(), &self.state_container) {
            Ok(result) => {
                // YankManager already handled clipboard via AppStateContainer
                // Keep local copy for backward compatibility
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

    fn yank_query(&mut self) {
        let query = self.get_input_text();
        if query.trim().is_empty() {
            self.buffer_mut()
                .set_status_message("No query to yank".to_string());
            return;
        }

        match self.state_container.write_to_clipboard(&query) {
            Ok(_) => {
                let char_count = query.len();
                let preview = if query.len() > 50 {
                    format!("{}...", &query[..50])
                } else {
                    query.clone()
                };

                // Create status message with character count
                let status_msg = format!("Yanked SQL ({} chars)", char_count);
                debug!("Yanking query: {}", &status_msg);
                self.buffer_mut().set_status_message(status_msg.clone());

                // Update local last_yanked for backward compatibility
                self.last_yanked = Some(("Query".to_string(), preview.clone()));

                // Also update clipboard state for tracking
                use crate::app_state_container::YankedItem;
                self.state_container.clipboard_mut().last_yanked = Some(YankedItem {
                    description: "SQL Query".to_string(),
                    full_value: query,
                    preview,
                    yank_type: crate::app_state_container::YankType::Query,
                    yanked_at: chrono::Local::now(),
                    size_bytes: char_count,
                });
            }
            Err(e) => {
                self.buffer_mut()
                    .set_status_message(format!("Failed to yank query: {}", e));
            }
        }
    }

    /// Yank current query and results as a complete test case (Ctrl+T in debug mode)
    fn yank_as_test_case(&mut self) {
        let test_case = DebugInfo::generate_test_case(self.buffer());

        match self.state_container.yank_test_case(test_case.clone()) {
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
                self.buffer_mut()
                    .set_status_message(format!("Failed to copy test case to clipboard: {}", e));
            }
        }
    }

    /// Yank debug dump with context for manual test creation (Shift+Y in debug mode)
    fn yank_debug_with_context(&mut self) {
        let debug_context = DebugInfo::generate_debug_context(self.buffer());

        match self
            .state_container
            .yank_debug_context(debug_context.clone())
        {
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
        }
    }

    fn paste_from_clipboard(&mut self) {
        // Paste from system clipboard into the current input field
        match self.state_container.read_from_clipboard() {
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
                                let pattern = self.get_input_text();
                                self.state_container.filter_mut().pattern = pattern.clone();
                                self.apply_filter(&pattern);
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
                                self.state_container.start_column_search(input_text);
                                // Column search pattern is now in AppStateContainer
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
        }
    }

    fn export_to_json(&mut self) {
        // Use trait-based export with DataProvider
        // TODO: Handle filtered data in future DataView implementation
        let result = if let Some(provider) = self.get_data_provider() {
            DataExporter::export_provider_to_json(provider.as_ref())
        } else {
            Err(anyhow::anyhow!("No data available to export"))
        };

        match result {
            Ok(message) => self.set_status_message(message),
            Err(e) => self.set_error_status("Export failed", e),
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

        // Update scroll state in container
        let mut scroll = self.state_container.scroll_mut();
        if cursor_pos < scroll.input_scroll_offset as usize {
            scroll.input_scroll_offset = cursor_pos as u16;
        }
        // If cursor is after the scroll window, scroll right
        else if cursor_pos >= scroll.input_scroll_offset as usize + inner_width {
            scroll.input_scroll_offset = (cursor_pos + 1).saturating_sub(inner_width) as u16;
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
        let mut new_buffer = buffer::Buffer::new(self.buffer_manager.all_buffers().len() + 1);
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
        // Get the current input text length and cursor position for display
        let input_text_for_count = self.get_input_text();
        let char_count = input_text_for_count.len();
        let cursor_pos = self.get_input_cursor();
        let char_count_display = if char_count > 0 {
            format!(" [{}/{} chars]", cursor_pos, char_count)
        } else {
            String::new()
        };

        let scroll_offset = self.get_horizontal_scroll_offset();
        let scroll_indicator = if scroll_offset > 0 {
            " ◀ " // Indicate text is scrolled (text hidden to the left)
        } else {
            ""
        };

        let input_title = match self.buffer().get_mode() {
            AppMode::Command => format!("SQL Query{}{}", char_count_display, scroll_indicator),
            AppMode::Results => format!(
                "SQL Query (Results Mode - Press ↑ to edit){}{}",
                char_count_display, scroll_indicator
            ),
            AppMode::Search => format!("Search Pattern{}{}", char_count_display, scroll_indicator),
            AppMode::Filter => format!("Filter Pattern{}{}", char_count_display, scroll_indicator),
            AppMode::FuzzyFilter => {
                format!("Fuzzy Filter{}{}", char_count_display, scroll_indicator)
            }
            AppMode::ColumnSearch => {
                format!("Column Search{}{}", char_count_display, scroll_indicator)
            }
            AppMode::Help => "Help".to_string(),
            AppMode::History => {
                let query = self.state_container.history_search().query.clone();
                format!("History Search: '{}' (Esc to cancel)", query)
            }
            AppMode::Debug => "Parser Debug (F5)".to_string(),
            AppMode::PrettyQuery => "Pretty Query View (F6)".to_string(),
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

            // Debug log to track rendering issues
            trace!(target: "render", "Rendering input: text='{}', mode={:?}, cursor={}",
                   if input_text_string.len() > 50 {
                       format!("{}...", &input_text_string[..50])
                   } else {
                       input_text_string.clone()
                   },
                   self.buffer().get_mode(),
                   self.get_input_cursor());

            // Get history search query if in history mode
            let history_query_string = if self.buffer().get_mode() == AppMode::History {
                self.state_container.history_search().query.clone()
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
                    let query_len = self.state_container.history_search().query.len();
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
            AppMode::ColumnStats => self.render_column_stats(f, results_area),
            _ if self.buffer().has_dataview() => {
                // Calculate viewport using DataView
                if let Some(dataview) = self.buffer().get_dataview() {
                    // Extract viewport info first
                    let terminal_height = results_area.height as usize;
                    let max_visible_rows = terminal_height.saturating_sub(3).max(10);
                    let total_rows = dataview.row_count();
                    let row_viewport_start = self
                        .buffer()
                        .get_scroll_offset()
                        .0
                        .min(total_rows.saturating_sub(1));
                    let row_viewport_end = (row_viewport_start + max_visible_rows).min(total_rows);

                    // PERF: Skip column width calculation for now - it's expensive even with viewport
                    // TODO: Re-enable when we have lazy column width calculation
                    // self.calculate_viewport_column_widths(row_viewport_start, row_viewport_end);
                }

                // V50: Render using DataProvider which works with DataTable
                if let Some(provider) = self.get_data_provider() {
                    self.render_table_with_provider(f, results_area, provider.as_ref());
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
                    let selected = self.state_container.get_table_selected_row().unwrap_or(0) + 1;
                    spans.push(Span::raw(" | "));

                    // Show selection mode
                    let selection_mode = self.get_selection_mode();
                    let mode_text = match selection_mode {
                        SelectionMode::Cell => "CELL",
                        SelectionMode::Row => "ROW",
                        SelectionMode::Column => "COL",
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

                    // Add cursor coordinates (x,y) - column and row position
                    let current_col = self.buffer().get_current_column() + 1; // Make it 1-based
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        format!("({},{})", current_col, selected),
                        Style::default().fg(Color::DarkGray),
                    ));

                    // Column information
                    if let Some(dataview) = self.buffer().get_dataview() {
                        let headers = dataview.column_names();
                        if self.buffer().get_current_column() < headers.len() {
                            spans.push(Span::raw(" | Col: "));
                            spans.push(Span::styled(
                                headers[self.buffer().get_current_column()].clone(),
                                Style::default().fg(Color::Cyan),
                            ));

                            // Show pinned columns count if any
                            if let Some(dataview) = self.buffer().get_dataview() {
                                let pinned_count = dataview.get_pinned_columns().len();
                                if pinned_count > 0 {
                                    spans.push(Span::raw(" | "));
                                    spans.push(Span::styled(
                                        format!("📌{}", pinned_count),
                                        Style::default().fg(Color::Magenta),
                                    ));
                                }

                                // Show hidden columns count if any
                                let hidden_count = dataview.get_hidden_column_names().len();
                                if hidden_count > 0 {
                                    spans.push(Span::raw(" | "));
                                    spans.push(Span::styled(
                                        format!("👁️‍🗨️{} hidden", hidden_count),
                                        Style::default().fg(Color::DarkGray),
                                    ));
                                    spans.push(Span::raw(" "));
                                    spans.push(Span::styled(
                                        "[- hide/+ unhide]",
                                        Style::default()
                                            .fg(Color::DarkGray)
                                            .add_modifier(Modifier::DIM),
                                    ));
                                } else {
                                    // Show hint about column hiding when no columns are hidden
                                    spans.push(Span::raw(" "));
                                    spans.push(Span::styled(
                                        "[- to hide col]",
                                        Style::default()
                                            .fg(Color::DarkGray)
                                            .add_modifier(Modifier::DIM),
                                    ));
                                }
                            } // Close the dataview if let

                            // In cell mode, show the current cell value
                            if self.get_selection_mode() == SelectionMode::Cell {
                                if let Some(selected_row) =
                                    self.state_container.get_table_selected_row()
                                {
                                    if let Some(row_data) =
                                        dataview.source().get_row_as_strings(selected_row)
                                    {
                                        let col_idx = self.buffer().get_current_column();
                                        if let Some(cell_value) = row_data.get(col_idx) {
                                            // Truncate if too long
                                            let display_value = if cell_value.len() > 30 {
                                                format!("{}...", &cell_value[..27])
                                            } else {
                                                cell_value.clone()
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

                    // Filter indicators
                    if self.buffer().is_fuzzy_filter_active() {
                        spans.push(Span::raw(" | "));
                        spans.push(Span::styled(
                            format!("Fuzzy: {}", self.buffer().get_fuzzy_filter_pattern()),
                            Style::default().fg(Color::Magenta),
                        ));
                    } else if self.state_container.filter().is_active {
                        spans.push(Span::raw(" | "));
                        spans.push(Span::styled(
                            format!("Filter: {}", self.state_container.filter().pattern),
                            Style::default().fg(Color::Cyan),
                        ));
                    }

                    // Show last yanked value from AppStateContainer
                    {
                        if let Some(ref yanked) = self.state_container.clipboard().last_yanked {
                            spans.push(Span::raw(" | "));
                            spans.push(Span::styled(
                                "Yanked: ",
                                Style::default().fg(Color::DarkGray),
                            ));
                            spans.push(Span::styled(
                                format!("{}={}", yanked.description, yanked.preview),
                                Style::default().fg(Color::Green),
                            ));
                        }
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

        // Show lock status indicators
        {
            let navigation = self.state_container.navigation();

            // Viewport lock indicator with boundary status
            if navigation.viewport_lock {
                spans.push(Span::raw(" | "));
                let lock_text = if navigation.is_at_viewport_top() {
                    format!("{}V↑", &self.config.display.icons.lock)
                } else if navigation.is_at_viewport_bottom() {
                    format!("{}V↓", &self.config.display.icons.lock)
                } else {
                    format!("{}V", &self.config.display.icons.lock)
                };
                spans.push(Span::styled(lock_text, Style::default().fg(Color::Magenta)));
            }

            // Cursor lock indicator
            if navigation.cursor_lock {
                spans.push(Span::raw(" | "));
                spans.push(Span::styled(
                    format!("{}C", &self.config.display.icons.lock),
                    Style::default().fg(Color::Yellow),
                ));
            }
        }

        // Show status message if present
        let status_msg = self.buffer().get_status_message();
        if !status_msg.is_empty() {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                status_msg,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        // Help shortcuts (right side)
        let help_text = match self.buffer().get_mode() {
            AppMode::Command => "Enter:Run | Tab:Complete | ↓:Results | F1:Help",
            AppMode::Results => match self.get_selection_mode() {
                SelectionMode::Cell => "v:Row mode | y:Yank cell | ↑:Edit | F1:Help",
                SelectionMode::Row => "v:Cell mode | y:Yank | f:Filter | ↑:Edit | F1:Help",
                SelectionMode::Column => "v:Cell mode | y:Yank col | ↑:Edit | F1:Help",
            },
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                "Enter:Apply | Esc:Cancel"
            }
            AppMode::Help | AppMode::Debug | AppMode::PrettyQuery | AppMode::ColumnStats => {
                "Esc:Close"
            }
            AppMode::History => "Enter:Select | Esc:Cancel",
            AppMode::JumpToRow => "Enter:Jump | Esc:Cancel",
        };

        // Add key press indicator if enabled
        if self.key_indicator.enabled {
            let key_display = self.key_indicator.to_string();
            if !key_display.is_empty() {
                spans.push(Span::raw(" | Keys: "));
                spans.push(Span::styled(
                    key_display,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::ITALIC),
                ));
            }
        }

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

    /// New trait-based table rendering method
    /// This uses DataProvider trait instead of directly accessing QueryResponse
    fn render_table_with_provider(&self, f: &mut Frame, area: Rect, provider: &dyn DataProvider) {
        use std::time::Instant;
        let render_start = Instant::now();

        let row_count = provider.get_row_count();
        let t1 = render_start.elapsed();

        if row_count == 0 {
            let empty = Paragraph::new("No results found")
                .block(Block::default().borders(Borders::ALL).title("Results"))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(empty, area);
            return;
        }

        // Get headers from provider (which should apply hidden column filtering)
        let headers = provider.get_column_names();
        debug!(
            "render_table_with_provider: Got {} column headers from provider",
            headers.len()
        );
        debug!("Column headers: {:?}", headers);
        debug!(
            "Buffer has {} hidden columns",
            self.buffer()
                .get_dataview()
                .map(|v| v.get_hidden_column_names().len())
                .unwrap_or(0)
        );

        // Calculate visible columns for virtual scrolling based on actual widths
        let terminal_width = area.width as usize;
        let available_width = terminal_width.saturating_sub(4); // Account for borders and padding

        // Split columns into pinned and scrollable
        let mut pinned_headers: Vec<(usize, String)> = Vec::new();
        let mut scrollable_indices: Vec<usize> = Vec::new();

        for (i, header) in headers.iter().enumerate() {
            if self.buffer().get_pinned_columns().contains(&i) {
                pinned_headers.push((i, header.clone()));
            } else {
                scrollable_indices.push(i);
            }
        }

        // Calculate space used by pinned columns
        let mut pinned_width = 0;
        // PERF TEST: Commenting out get_column_widths() call to test performance
        // let column_widths = provider.get_column_widths();
        let column_widths = vec![15; headers.len()]; // Use fixed width for now
        for &(idx, _) in &pinned_headers {
            if idx < column_widths.len() {
                pinned_width += column_widths[idx];
            } else {
                pinned_width += 15; // Default width
            }
        }

        // Calculate how many scrollable columns can fit in remaining space
        let remaining_width = available_width.saturating_sub(pinned_width);
        let max_visible_scrollable_cols = if !column_widths.is_empty() {
            let mut width_used = 0;
            let mut cols_that_fit = 0;

            for &idx in &scrollable_indices {
                if idx >= headers.len() {
                    break;
                }
                let col_width = if idx < column_widths.len() {
                    column_widths[idx]
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
            // Fallback if no calculated widths
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
            // Current column is pinned, use scroll offset from navigation state
            self.state_container.navigation().scroll_offset.1.min(
                scrollable_indices
                    .len()
                    .saturating_sub(max_visible_scrollable_cols),
            )
        };
        let viewport_end =
            (viewport_start + max_visible_scrollable_cols).min(scrollable_indices.len());

        // Build final list of visible columns (pinned + scrollable viewport)
        let mut visible_columns: Vec<(usize, String)> = Vec::new();
        visible_columns.extend(pinned_headers.iter().cloned());
        for i in viewport_start..viewport_end {
            let idx = scrollable_indices[i];
            visible_columns.push((idx, headers[idx].clone()));
        }

        // Calculate viewport dimensions
        let terminal_height = area.height as usize;
        let max_visible_rows = terminal_height.saturating_sub(3).max(10);

        // Calculate row viewport using navigation state as source of truth
        let row_viewport_start = self
            .state_container
            .navigation()
            .scroll_offset
            .0
            .min(row_count.saturating_sub(1));
        let row_viewport_end = (row_viewport_start + max_visible_rows).min(row_count);

        // Get visible rows from provider
        let t2 = render_start.elapsed();
        let visible_rows =
            provider.get_visible_rows(row_viewport_start, row_viewport_end - row_viewport_start);
        let t3 = render_start.elapsed();

        // Transform to only show visible columns
        let data_to_display: Vec<Vec<String>> = visible_rows
            .iter()
            .map(|row| {
                visible_columns
                    .iter()
                    .map(|(idx, _)| row.get(*idx).cloned().unwrap_or_default())
                    .collect()
            })
            .collect();
        let t4 = render_start.elapsed();

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
            // Get sort indicator from AppStateContainer if available
            let sort_indicator = {
                let sort = self.state_container.sort();
                if let Some(col) = sort.column {
                    if col == *actual_col_index {
                        match sort.order {
                            SortOrder::Ascending => " ↑",
                            SortOrder::Descending => " ↓",
                            SortOrder::None => "",
                        }
                    } else {
                        ""
                    }
                } else {
                    ""
                }
            };

            let column_indicator = if *actual_col_index == self.buffer().get_current_column() {
                " [*]"
            } else {
                ""
            };

            // Check if this column is pinned
            let pinned_cols = self.buffer().get_pinned_columns();
            let pinned_indicator = if pinned_cols.contains(actual_col_index) {
                " 📌"
            } else {
                ""
            };

            let mut style = Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD);

            if *actual_col_index == self.buffer().get_current_column() {
                style = style.fg(Color::Yellow).add_modifier(Modifier::UNDERLINED);
            }

            Cell::from(format!(
                "{}{}{}{}",
                header, sort_indicator, column_indicator, pinned_indicator
            ))
            .style(style)
        }));

        let header = Row::new(header_cells);

        // Create data rows
        let mut rows: Vec<Row> = Vec::new();
        for (i, row_data) in data_to_display.iter().enumerate() {
            let mut cells: Vec<Cell> = Vec::new();

            // Add row number if enabled
            if self.buffer().is_show_row_numbers() {
                let row_num = row_viewport_start + i + 1;
                cells.push(
                    Cell::from(row_num.to_string()).style(Style::default().fg(Color::DarkGray)),
                );
            }

            // Add data cells with column highlighting
            let current_column = self.state_container.navigation().selected_column;
            let selected_row = self.state_container.navigation().selected_row;
            let is_current_row = row_viewport_start + i == selected_row;

            // Get fuzzy filter pattern for cell-level matching
            let fuzzy_pattern = if self.buffer().is_fuzzy_filter_active() {
                let pattern = self.buffer().get_fuzzy_filter_pattern();
                if !pattern.is_empty() {
                    Some(pattern)
                } else {
                    None
                }
            } else {
                None
            };

            cells.extend(row_data.iter().enumerate().map(|(col_idx, val)| {
                // Check if this column matches the selected column in visible columns
                let is_selected_column = visible_columns
                    .get(col_idx)
                    .map(|(actual_col, _)| *actual_col == current_column)
                    .unwrap_or(false);

                let mut cell = Cell::from(val.clone());

                // Check if THIS SPECIFIC CELL contains the fuzzy filter match
                if let Some(ref pattern) = fuzzy_pattern {
                    if !is_current_row {
                        let case_insensitive = self.buffer().is_case_insensitive();
                        let cell_matches = if pattern.starts_with('\'') && pattern.len() > 1 {
                            // Exact match mode - check if this cell contains the pattern
                            let search_pattern = &pattern[1..];
                            if case_insensitive {
                                val.to_lowercase().contains(&search_pattern.to_lowercase())
                            } else {
                                val.contains(search_pattern)
                            }
                        } else if !pattern.is_empty() {
                            // Fuzzy match mode - check if this cell fuzzy matches
                            use fuzzy_matcher::skim::SkimMatcherV2;
                            use fuzzy_matcher::FuzzyMatcher;
                            let matcher = if case_insensitive {
                                SkimMatcherV2::default().ignore_case()
                            } else {
                                SkimMatcherV2::default().respect_case()
                            };
                            matcher
                                .fuzzy_match(val, pattern)
                                .map(|score| score > 0)
                                .unwrap_or(false)
                        } else {
                            false
                        };

                        if cell_matches {
                            // Only highlight cells that actually contain the match
                            cell = cell.style(Style::default().fg(Color::Magenta));
                        }
                    }
                }

                if is_current_row && is_selected_column {
                    // Crosshair cell - both row and column selected (overrides fuzzy highlight)
                    cell.style(
                        Style::default()
                            .bg(Color::Yellow)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    )
                } else if is_selected_column {
                    // Column highlight
                    cell.style(Style::default().bg(Color::Rgb(50, 50, 50)))
                } else {
                    cell
                }
            }));

            // Apply row highlighting
            let row_style = if is_current_row {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            rows.push(Row::new(cells).style(row_style));
        }

        // Calculate column widths for the table widget
        let mut widths: Vec<Constraint> = Vec::new();

        // Add row number column width if enabled
        if self.buffer().is_show_row_numbers() {
            widths.push(Constraint::Length(8)); // Fixed width for row numbers
        }

        // Add widths for visible data columns
        for (idx, _) in &visible_columns {
            let width = if *idx < column_widths.len() {
                column_widths[*idx] as u16
            } else {
                15
            };
            widths.push(Constraint::Length(width.min(50))); // Cap at 50 to prevent overly wide columns
        }

        // Create the table widget
        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Results ({} rows)", row_count)),
            )
            .column_spacing(1)
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(table, area);

        let total = render_start.elapsed();

        // Store render timing (mutable access through unsafe - bit hacky but for debugging)
        let timing_msg = format!(
            "get_row_count={:?}, calc_viewport={:?}, get_visible_rows={:?}, transform={:?}, total={:?}, rows={}, visible={}",
            t1, t2 - t1, t3 - t2, t4 - t3, total, row_count, row_viewport_end - row_viewport_start
        );

        // This is a bit ugly but we need mutable access in a &self method for debugging
        unsafe {
            let self_mut = self as *const Self as *mut Self;
            if (*self_mut).render_timings.len() >= 20 {
                (*self_mut).render_timings.remove(0);
            }
            (*self_mut).render_timings.push(timing_msg.clone());
        }

        // Debug output now available in F5 view, no need for stderr
        // eprintln!("render_table timing: {}", timing_msg);
    }

    fn render_table_immutable(&self, f: &mut Frame, area: Rect, _results: &QueryResponse) {
        // V40: Now using trait-based rendering via DataProvider
        // The BufferAdapter makes this seamless - the Buffer implements DataProvider
        if let Some(provider) = self.get_data_provider() {
            self.render_table_with_provider(f, area, provider.as_ref());
        } else {
            // Minimal fallback - should rarely if ever be hit
            let msg = Paragraph::new("No data provider available")
                .block(Block::default().borders(Borders::ALL).title("Error"))
                .style(Style::default().fg(Color::Red));
            f.render_widget(msg, area);
        }
    }

    fn render_help(&mut self, f: &mut Frame, area: Rect) {
        // Use simple two-column layout - shows everything at once
        self.render_help_two_column(f, area);
    }

    fn render_help_two_column(&self, f: &mut Frame, area: Rect) {
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

        // Apply scroll offset (from state container or local)
        let scroll_offset = { self.state_container.help_scroll_offset() as usize };

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
        // Get history state from AppStateContainer
        let history_search = self.state_container.history_search();
        let matches_empty = history_search.matches.is_empty();
        let search_query_empty = history_search.query.is_empty();

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
        // Get history data from AppStateContainer
        let history_search = self.state_container.history_search();
        let matches = history_search.matches.clone();
        let selected_index = history_search.selected_index;
        let match_count = matches.len();

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
        // Get the selected match from AppStateContainer
        let history_search = self.state_container.history_search();
        let selected_match = history_search
            .matches
            .get(history_search.selected_index)
            .cloned();

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

                // Clear is_active flag
                {
                    let container_ptr =
                        Arc::as_ptr(&self.state_container) as *mut AppStateContainer;
                    unsafe {
                        (*container_ptr).jump_to_row_mut().is_active = false;
                    }
                }

                self.buffer_mut()
                    .set_status_message("Jump cancelled".to_string());
            }
            KeyCode::Enter => {
                if let Ok(row_num) = self.get_jump_to_row_input().parse::<usize>() {
                    if row_num > 0 {
                        let target_row = row_num - 1; // Convert to 0-based index
                        let max_row = self.get_current_data().map(|d| d.row_count()).unwrap_or(0);

                        if target_row < max_row {
                            // Calculate centered viewport position
                            let visible_rows = self.buffer().get_last_visible_rows();
                            let centered_scroll_offset = if visible_rows > 0 {
                                target_row.saturating_sub(visible_rows / 2)
                            } else {
                                target_row
                            };

                            // Update NavigationState with proper scroll offset
                            {
                                let mut nav = self.state_container.navigation_mut();
                                nav.jump_to_row(target_row);
                                // Also update NavigationState's scroll offset to center the row
                                nav.scroll_offset.0 = centered_scroll_offset;
                                info!(target: "navigation", "Jump-to-row: set scroll_offset to {} to center row {}", centered_scroll_offset, target_row);
                            }

                            self.state_container
                                .set_table_selected_row(Some(target_row));

                            // Update buffer's scroll offset to match
                            let mut offset = self.buffer().get_scroll_offset();
                            offset.0 = centered_scroll_offset;
                            self.buffer_mut().set_scroll_offset(offset);

                            self.buffer_mut().set_status_message(format!(
                                "Jumped to row {} (centered)",
                                row_num
                            ));
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

                // Clear is_active flag
                {
                    let container_ptr =
                        Arc::as_ptr(&self.state_container) as *mut AppStateContainer;
                    unsafe {
                        (*container_ptr).jump_to_row_mut().is_active = false;
                    }
                }
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
            } else if query == ":exit" || query == ":quit" || query == ":q" {
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

    /// Get current process memory usage in KB (Linux only)
    #[cfg(target_os = "linux")]
    fn get_process_memory_kb() -> Option<usize> {
        std::fs::read_to_string("/proc/self/status")
            .ok()?
            .lines()
            .find(|line| line.starts_with("VmRSS:"))
            .and_then(|line| {
                line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<usize>().ok())
            })
    }

    #[cfg(not(target_os = "linux"))]
    fn get_process_memory_kb() -> Option<usize> {
        None
    }

    /// V46: Demonstrate DataTable conversion
    fn demo_datatable_conversion(&mut self) {
        // V47: Collect all data before any mutable borrows to avoid borrow checker issues
        let (has_datatable, datatable_info, json_size) = {
            let buffer = self.buffer();

            let datatable_info = if let Some(dataview) = buffer.get_dataview() {
                let datatable = dataview.source();
                // Log column types
                for (idx, column) in datatable.columns.iter().enumerate() {
                    debug!(
                        "V47: Column {}: {} ({:?}, nullable: {}, nulls: {})",
                        idx, column.name, column.data_type, column.nullable, column.null_count
                    );
                }

                let dump = datatable.debug_dump();
                debug!("V47 DataTable dump:\n{}", dump);

                Some((
                    datatable.row_count(),
                    datatable.column_count(),
                    datatable.estimate_memory_size(),
                ))
            } else {
                None
            };

            // V50: JSON size is no longer applicable - DataTable is primary storage
            let json_size: Option<usize> = None;

            (buffer.has_dataview(), datatable_info, json_size)
        };

        // Now handle the results with mutable borrows
        if let Some((row_count, col_count, datatable_size)) = datatable_info {
            info!(
                "V47: Using stored DataTable: {} rows x {} columns",
                row_count, col_count
            );

            // Get actual process memory
            let process_memory = Self::get_process_memory_kb();

            let message = if let Some(json_size) = json_size {
                if let Some(mem_kb) = process_memory {
                    format!(
                        "V47: {} rows × {} cols | JSON ~{}KB, DataTable ~{}KB | Process total: {}MB",
                        row_count,
                        col_count,
                        json_size / 1024,
                        datatable_size / 1024,
                        mem_kb / 1024
                    )
                } else {
                    format!(
                        "V47: DataTable stored! {} rows, {} cols. Memory: JSON ~{}KB vs DataTable ~{}KB",
                        row_count,
                        col_count,
                        json_size / 1024,
                        datatable_size / 1024
                    )
                }
            } else {
                if let Some(mem_kb) = process_memory {
                    format!(
                        "V47: {} rows × {} cols | DataTable ~{}KB | Process total: {}MB",
                        row_count,
                        col_count,
                        datatable_size / 1024,
                        mem_kb / 1024
                    )
                } else {
                    format!(
                        "V47: DataTable stored! {} rows, {} cols. Memory: ~{}KB",
                        row_count,
                        col_count,
                        datatable_size / 1024
                    )
                }
            };

            self.buffer_mut().set_status_message(message);
        } else {
            self.buffer_mut()
                .set_status_message("V50: No DataTable available".to_string());
        }
    }

    /// V50: DataTable is now the primary storage, so this is a no-op
    fn ensure_datatable_exists(&mut self) {
        // V50: DataTable is always created when data is loaded
        // This function is kept for compatibility but does nothing
        if self.buffer().has_dataview() {
            debug!("V50: DataView already exists");
        }
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
                        self.state_container.get_current_column(),
                        buffer
                            .get_dataview()
                            .map(|v| v.source().row_count())
                            .unwrap_or(0),
                        buffer.get_dataview().map(|v| v.row_count()).unwrap_or(0),
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

                // Add DataTable schema information
                if let Some(buffer) = self.buffer_manager.current() {
                    if let Some(dataview) = buffer.get_dataview() {
                        let datatable = dataview.source();
                        debug_info.push_str("\n========== DATATABLE SCHEMA ==========\n");
                        debug_info.push_str(&datatable.get_schema_summary());
                    }

                    // Add DataView information - shows the actual view state
                    if let Some(dataview) = buffer.get_dataview() {
                        debug_info.push_str("\n========== DATAVIEW STATE ==========\n");

                        // Show visible columns in order
                        let visible_columns = dataview.column_names();
                        debug_info
                            .push_str(&format!("Visible Columns ({}):\n", visible_columns.len()));
                        for (idx, col_name) in visible_columns.iter().enumerate() {
                            debug_info.push_str(&format!("  [{}] {}\n", idx, col_name));
                        }

                        // Show row information
                        debug_info.push_str(&format!("\nVisible Rows: {}\n", dataview.row_count()));

                        // Show if columns have been reordered
                        // Use the DataView's source to get original column order
                        {
                            let original_columns = dataview.source().column_names();
                            if visible_columns != original_columns {
                                debug_info.push_str("\nColumn Order Changed: YES\n");

                                // Show what columns are hidden
                                let hidden: Vec<String> = original_columns
                                    .iter()
                                    .filter(|col| !visible_columns.contains(col))
                                    .cloned()
                                    .collect();
                                if !hidden.is_empty() {
                                    debug_info
                                        .push_str(&format!("Hidden Columns ({}):\n", hidden.len()));
                                    for col in hidden {
                                        debug_info.push_str(&format!("  - {}\n", col));
                                    }
                                }
                            } else {
                                debug_info.push_str("\nColumn Order Changed: NO\n");
                            }
                        }
                    } else {
                        debug_info.push_str("\n========== DATAVIEW STATE ==========\n");
                        debug_info.push_str("No DataView available (using DataTable directly)\n");
                    }
                }

                // Add memory tracking history
                debug_info.push_str("\n========== MEMORY USAGE ==========\n");
                debug_info.push_str(&format!(
                    "Current Memory: {} MB\n",
                    crate::utils::memory_tracker::get_memory_mb()
                ));
                debug_info.push_str(&crate::utils::memory_tracker::format_memory_history());

                // Add navigation timing statistics
                debug_info.push_str("\n========== NAVIGATION TIMING ==========\n");
                if !self.navigation_timings.is_empty() {
                    debug_info.push_str(&format!(
                        "Last {} navigation timings:\n",
                        self.navigation_timings.len()
                    ));
                    for timing in &self.navigation_timings {
                        debug_info.push_str(&format!("  {}\n", timing));
                    }
                    // Calculate average
                    if self.navigation_timings.len() > 0 {
                        let total_ms: f64 = self
                            .navigation_timings
                            .iter()
                            .filter_map(|s| {
                                // Extract total time from the timing string
                                if let Some(total_pos) = s.find("total=") {
                                    let after_total = &s[total_pos + 6..];
                                    if let Some(end_pos) =
                                        after_total.find(',').or_else(|| after_total.find(')'))
                                    {
                                        let time_str = &after_total[..end_pos];
                                        // Parse duration - expecting format like "123.456µs" or "1.234ms"
                                        if let Some(us_pos) = time_str.find("µs") {
                                            time_str[..us_pos]
                                                .parse::<f64>()
                                                .ok()
                                                .map(|us| us / 1000.0)
                                        } else if let Some(ms_pos) = time_str.find("ms") {
                                            time_str[..ms_pos].parse::<f64>().ok()
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .sum();
                        let avg_ms = total_ms / self.navigation_timings.len() as f64;
                        debug_info.push_str(&format!("Average navigation time: {:.3}ms\n", avg_ms));
                    }
                } else {
                    debug_info.push_str("No navigation timing data yet (press j/k to navigate)\n");
                }

                // Add render timing statistics
                debug_info.push_str("\n========== RENDER TIMING ==========\n");
                if !self.render_timings.is_empty() {
                    debug_info.push_str(&format!(
                        "Last {} render timings:\n",
                        self.render_timings.len()
                    ));
                    for timing in &self.render_timings {
                        debug_info.push_str(&format!("  {}\n", timing));
                    }
                    // Calculate average render time
                    if self.render_timings.len() > 0 {
                        let total_ms: f64 = self
                            .render_timings
                            .iter()
                            .filter_map(|s| {
                                // Extract total time from the timing string
                                if let Some(total_pos) = s.find("total=") {
                                    let after_total = &s[total_pos + 6..];
                                    if let Some(end_pos) =
                                        after_total.find(',').or_else(|| after_total.find(')'))
                                    {
                                        let time_str = &after_total[..end_pos];
                                        // Parse duration
                                        if let Some(us_pos) = time_str.find("µs") {
                                            time_str[..us_pos]
                                                .parse::<f64>()
                                                .ok()
                                                .map(|us| us / 1000.0)
                                        } else if let Some(ms_pos) = time_str.find("ms") {
                                            time_str[..ms_pos].parse::<f64>().ok()
                                        } else if let Some(s_pos) = time_str.find("s") {
                                            time_str[..s_pos]
                                                .parse::<f64>()
                                                .ok()
                                                .map(|s| s * 1000.0)
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .sum();
                        let avg_ms = total_ms / self.render_timings.len() as f64;
                        debug_info.push_str(&format!("Average render time: {:.3}ms\n", avg_ms));
                    }
                } else {
                    debug_info.push_str("No render timing data yet\n");
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
                /*
                if query.to_lowercase().contains(" where ") {
                    let where_ast_info = match self.parse_where_clause_ast(&query) {
                            Ok(ast_str) => ast_str,
                            Err(e) => format!("\n========== WHERE CLAUSE AST ==========\nError parsing WHERE clause: {}\n", e)
                        };
                    debug_info.push_str(&where_ast_info);
                }*/

                // Add key chord handler debug info
                debug_info.push_str("\n");
                debug_info.push_str(&self.key_chord_handler.format_debug_info());
                debug_info.push_str("========================================\n");

                // Add search modes widget debug info
                debug_info.push_str("\n");
                debug_info.push_str(&self.search_modes_widget.debug_info());

                // Add column search state if active
                let show_column_search = self.buffer().get_mode() == AppMode::ColumnSearch
                    || !self.state_container.column_search().pattern.is_empty();
                if show_column_search {
                    {
                        let column_search = self.state_container.column_search();
                        debug_info.push_str("\n========== COLUMN SEARCH STATE ==========\n");
                        debug_info.push_str(&format!("Pattern: '{}'\n", column_search.pattern));
                        debug_info.push_str(&format!(
                            "Matching Columns: {} found\n",
                            column_search.matching_columns.len()
                        ));
                        if !column_search.matching_columns.is_empty() {
                            debug_info.push_str("Matches:\n");
                            for (idx, (col_idx, col_name)) in
                                column_search.matching_columns.iter().enumerate()
                            {
                                let marker = if idx == column_search.current_match {
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
                            column_search.current_match
                        ));
                        debug_info.push_str(&format!(
                            "Current Column: {}\n",
                            self.state_container.get_current_column()
                        ));
                        debug_info.push_str("==========================================\n");
                    }
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
                {
                    debug_info.push_str("\n");
                    debug_info.push_str(&self.state_container.debug_dump());
                    debug_info.push_str("\n");
                }

                // Set the final content in debug widget
                self.debug_widget.set_content(debug_info.clone());

                // Try to copy to clipboard
                match self.state_container.write_to_clipboard(&debug_info) {
                    Ok(_) => {
                        self.buffer_mut().set_status_message(format!(
                            "DEBUG INFO copied to clipboard ({} chars)!",
                            debug_info.len()
                        ));
                    }
                    Err(e) => {
                        self.buffer_mut()
                            .set_status_message(format!("Clipboard error: {}", e));
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
