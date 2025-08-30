// UI Layout Constants
const TABLE_BORDER_WIDTH: u16 = 4; // Left border (1) + right border (1) + padding (2)
const INPUT_AREA_HEIGHT: u16 = 3; // Height of the command input area
const STATUS_BAR_HEIGHT: u16 = 3; // Height of the status bar
const TOTAL_UI_CHROME: u16 = INPUT_AREA_HEIGHT + STATUS_BAR_HEIGHT; // Total non-table UI height
const TABLE_CHROME_ROWS: u16 = 3; // Table header (1) + top border (1) + bottom border (1)
use crate::app_state_container::{AppStateContainer, SelectionMode};
use crate::buffer::{
    AppMode, Buffer, BufferAPI, BufferManager, ColumnStatistics, ColumnType, EditMode,
};
use crate::buffer_handler::BufferHandler;
use crate::config::config::Config;
use crate::core::search_manager::{SearchConfig, SearchManager};
use crate::cursor_manager::CursorManager;
use crate::data::adapters::BufferAdapter;
use crate::data::data_analyzer::DataAnalyzer;
use crate::data::data_provider::DataProvider;
use crate::data::data_view::DataView;
use crate::debug::{DebugRegistry, MemoryTracker};
use crate::debug_service::DebugService;
use crate::help_text::HelpText;
use crate::services::QueryOrchestrator;
use crate::sql::hybrid_parser::HybridParser;
use crate::sql_highlighter::SqlHighlighter;
use crate::state::StateDispatcher;
use crate::ui::debug::DebugContext;
use crate::ui::input::action_handlers::ActionHandlerContext;
use crate::ui::input::actions::{Action, ActionContext, ActionResult};
use crate::ui::key_handling::{
    format_key_for_display, ChordResult, KeyChordHandler, KeyDispatcher, KeyMapper,
    KeyPressIndicator, KeySequenceRenderer,
};
use crate::ui::rendering::table_widget_manager::TableWidgetManager;
use crate::ui::search::vim_search_adapter::VimSearchAdapter;
use crate::ui::state::shadow_state::ShadowStateManager;
use crate::ui::traits::{
    BufferManagementBehavior, ColumnBehavior, InputBehavior, NavigationBehavior, YankBehavior,
};
use crate::ui::viewport::ColumnPackingMode;
use crate::ui::viewport_manager::{ViewportEfficiency, ViewportManager};
use crate::utils::logging::LogRingBuffer;
use crate::widget_traits::DebugInfoProvider;
use crate::widgets::debug_widget::DebugWidget;
use crate::widgets::editor_widget::{BufferAction, EditorAction, EditorWidget};
use crate::widgets::help_widget::HelpWidget;
use crate::widgets::search_modes_widget::{SearchMode, SearchModesAction, SearchModesWidget};
use crate::widgets::stats_widget::StatsWidget;
use crate::widgets::tab_bar_widget::TabBarWidget;
use crate::{buffer, data_analyzer, dual_logging};
use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::cell::RefCell;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};
use tui_input::{backend::crossterm::EventHandler, Input};

/// CommandEditor handles all command mode input and state management
/// This is the first step in extracting command mode from the main TUI
struct CommandEditor {
    // References to shared state (will be passed in from TUI)
    input: Input,

    // Command-specific state
    scroll_offset: usize,
    last_cursor_position: usize,
    history_search_term: Option<String>,
}

impl CommandEditor {
    fn new() -> Self {
        Self {
            input: Input::default(),
            scroll_offset: 0,
            last_cursor_position: 0,
            history_search_term: None,
        }
    }

    /// Handle command mode input, returning true if the app should exit
    fn handle_input(
        &mut self,
        key: KeyEvent,
        state_container: &mut AppStateContainer,
        shadow_state: &RefCell<crate::ui::state::shadow_state::ShadowStateManager>,
    ) -> Result<bool> {
        debug!(
            "CommandEditor::handle_input - key: {:?}, current text: '{}', cursor: {}",
            key,
            self.input.value(),
            self.input.cursor()
        );

        // Handle comprehensive text editing operations
        match key.code {
            // === Character Input ===
            KeyCode::Char(c) => {
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT {
                    // Simple character input
                    let before = self.input.value().to_string();
                    let before_cursor = self.input.cursor();
                    self.input.handle_event(&Event::Key(key));
                    let after = self.input.value().to_string();
                    let after_cursor = self.input.cursor();
                    debug!(
                        "CommandEditor processed char '{}': text '{}' -> '{}', cursor {} -> {}",
                        c, before, after, before_cursor, after_cursor
                    );
                    return Ok(false);
                }

                // === Ctrl Key Combinations ===
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'a' | 'A' => {
                            // Move to beginning of line
                            self.input = self.input.clone().with_cursor(0);
                            return Ok(false);
                        }
                        'e' | 'E' => {
                            // Move to end of line
                            let len = self.input.value().len();
                            self.input = self.input.clone().with_cursor(len);
                            return Ok(false);
                        }
                        'k' | 'K' => {
                            // Kill line - delete from cursor to end
                            let cursor = self.input.cursor();
                            let text = self.input.value();
                            if cursor < text.len() {
                                let new_text = text[..cursor].to_string();
                                self.input = Input::from(new_text).with_cursor(cursor);
                            }
                            return Ok(false);
                        }
                        'u' | 'U' => {
                            // Kill line backward - delete from start to cursor
                            let cursor = self.input.cursor();
                            let text = self.input.value();
                            if cursor > 0 {
                                let new_text = text[cursor..].to_string();
                                self.input = Input::from(new_text).with_cursor(0);
                            }
                            return Ok(false);
                        }
                        'w' | 'W' => {
                            // Delete word backward
                            self.delete_word_backward();
                            return Ok(false);
                        }
                        'd' | 'D' => {
                            // Delete word forward (if at word boundary)
                            self.delete_word_forward();
                            return Ok(false);
                        }
                        _ => {}
                    }
                }

                // === Alt Key Combinations ===
                if key.modifiers.contains(KeyModifiers::ALT) {
                    match c {
                        'b' | 'B' => {
                            // Move word backward
                            self.move_word_backward();
                            return Ok(false);
                        }
                        'f' | 'F' => {
                            debug!("CommandEditor: Alt+F - Move word forward");
                            // Move word forward
                            self.move_word_forward();
                            return Ok(false);
                        }
                        'd' | 'D' => {
                            // Delete word forward
                            self.delete_word_forward();
                            return Ok(false);
                        }
                        _ => {}
                    }
                }
            }

            // === Basic Navigation and Editing ===
            KeyCode::Backspace => {
                self.input.handle_event(&Event::Key(key));
                return Ok(false);
            }
            KeyCode::Delete => {
                self.input.handle_event(&Event::Key(key));
                return Ok(false);
            }
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_word_backward();
                } else {
                    self.input.handle_event(&Event::Key(key));
                }
                return Ok(false);
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_word_forward();
                } else {
                    self.input.handle_event(&Event::Key(key));
                }
                return Ok(false);
            }
            KeyCode::Home => {
                self.input = self.input.clone().with_cursor(0);
                return Ok(false);
            }
            KeyCode::End => {
                let len = self.input.value().len();
                self.input = self.input.clone().with_cursor(len);
                return Ok(false);
            }

            // === Tab Completion ===
            KeyCode::Tab => {
                // Tab completion needs access to full TUI state
                // Let parent handle it by not returning early
            }

            _ => {}
        }

        // Key not handled by CommandEditor
        Ok(false)
    }

    // === Helper Methods for Text Operations ===

    fn delete_word_backward(&mut self) {
        let cursor = self.input.cursor();
        let text = self.input.value();

        if cursor == 0 {
            return;
        }

        // Find start of current/previous word
        let chars: Vec<char> = text.chars().collect();
        let mut pos = cursor;

        // Skip trailing spaces
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip word characters
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Delete from pos to cursor
        let new_text = format!("{}{}", &text[..pos], &text[cursor..]);
        self.input = Input::from(new_text).with_cursor(pos);
    }

    fn delete_word_forward(&mut self) {
        let cursor = self.input.cursor();
        let text = self.input.value();

        if cursor >= text.len() {
            return;
        }

        // Find end of current/next word
        let chars: Vec<char> = text.chars().collect();
        let mut pos = cursor;

        // Skip word characters
        while pos < chars.len() && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // Skip trailing spaces
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        // Delete from cursor to pos
        let new_text = format!("{}{}", &text[..cursor], &text[pos..]);
        self.input = Input::from(new_text).with_cursor(cursor);
    }

    fn move_word_backward(&mut self) {
        let cursor = self.input.cursor();
        let text = self.input.value();

        if cursor == 0 {
            return;
        }

        let chars: Vec<char> = text.chars().collect();
        let mut pos = cursor;

        // Skip trailing spaces
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Skip word characters
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        self.input = self.input.clone().with_cursor(pos);
    }

    fn move_word_forward(&mut self) {
        let cursor = self.input.cursor();
        let text = self.input.value();

        if cursor >= text.len() {
            return;
        }

        let chars: Vec<char> = text.chars().collect();
        let mut pos = cursor;

        // Skip word characters
        while pos < chars.len() && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // Skip trailing spaces
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        self.input = self.input.clone().with_cursor(pos);
    }

    /// Get the current input text
    fn get_text(&self) -> String {
        self.input.value().to_string()
    }

    /// Set the input text
    fn set_text(&mut self, text: String) {
        self.input = Input::from(text);
    }

    /// Get the current cursor position
    fn get_cursor(&self) -> usize {
        self.input.cursor()
    }

    /// Set the cursor position
    fn set_cursor(&mut self, pos: usize) {
        let text = self.input.value().to_string();
        self.input = tui_input::Input::new(text).with_cursor(pos);
    }
}

pub struct EnhancedTuiApp {
    // State container - manages all state (owned directly, no Arc needed)
    state_container: AppStateContainer,
    // Debug service for logging (ServiceContainer removed)
    debug_service: Option<DebugService>,

    input: Input,
    command_editor: CommandEditor, // New: Handles command mode input and state
    cursor_manager: CursorManager, // New: manages cursor/navigation logic
    data_analyzer: DataAnalyzer,   // New: manages data analysis/statistics
    hybrid_parser: HybridParser,

    // Configuration
    config: Config,

    sql_highlighter: SqlHighlighter,
    pub(crate) debug_widget: DebugWidget,
    editor_widget: EditorWidget,
    stats_widget: StatsWidget,
    help_widget: HelpWidget,
    search_modes_widget: SearchModesWidget,
    vim_search_adapter: RefCell<VimSearchAdapter>, // State-aware vim search adapter
    search_manager: RefCell<SearchManager>,        // New: Centralized search logic
    state_dispatcher: RefCell<StateDispatcher>,    // Coordinates state changes
    key_chord_handler: KeyChordHandler,            // Manages key sequences and history
    key_dispatcher: KeyDispatcher,                 // Maps keys to actions
    key_mapper: KeyMapper,                         // New action-based key mapping system

    // Buffer management now in AppStateContainer
    // buffer_manager field removed - using state_container.buffers() instead
    buffer_handler: BufferHandler, // Handles buffer operations like switching

    // Performance tracking
    pub(crate) navigation_timings: Vec<String>, // Track last N navigation timings for debugging
    pub(crate) render_timings: Vec<String>,     // Track last N render timings for debugging
    // Cache
    log_buffer: Option<LogRingBuffer>, // Ring buffer for debug logs

    // Data source tracking
    data_source: Option<String>, // e.g., "trades.csv", "data.json", "https://api.example.com"

    // Visual enhancements
    key_indicator: KeyPressIndicator,
    key_sequence_renderer: KeySequenceRenderer,

    // Viewport management (RefCell for interior mutability during render)
    pub(crate) viewport_manager: RefCell<Option<ViewportManager>>,
    viewport_efficiency: RefCell<Option<ViewportEfficiency>>,

    // Shadow state manager for observing state transitions
    shadow_state: RefCell<crate::ui::state::shadow_state::ShadowStateManager>,

    // Table widget manager for centralized table state/rendering
    table_widget_manager: RefCell<TableWidgetManager>,

    // Services
    query_orchestrator: QueryOrchestrator,

    // Debug system
    pub(crate) debug_registry: DebugRegistry,
    pub(crate) memory_tracker: MemoryTracker,
}

impl DebugContext for EnhancedTuiApp {
    fn buffer(&self) -> &dyn BufferAPI {
        self.state_container
            .current_buffer()
            .expect("Buffer should exist")
    }

    fn buffer_mut(&mut self) -> &mut dyn BufferAPI {
        self.state_container
            .current_buffer_mut()
            .expect("Buffer should exist")
    }

    fn get_debug_widget(&self) -> &DebugWidget {
        &self.debug_widget
    }

    fn get_debug_widget_mut(&mut self) -> &mut DebugWidget {
        &mut self.debug_widget
    }

    fn get_shadow_state(&self) -> &RefCell<ShadowStateManager> {
        &self.shadow_state
    }

    fn get_buffer_manager(&self) -> &BufferManager {
        self.state_container.buffers()
    }

    fn get_viewport_manager(&self) -> &RefCell<Option<ViewportManager>> {
        &self.viewport_manager
    }

    fn get_state_container(&self) -> &AppStateContainer {
        &self.state_container
    }

    fn get_state_container_mut(&mut self) -> &mut AppStateContainer {
        &mut self.state_container
    }

    fn get_navigation_timings(&self) -> &Vec<String> {
        &self.navigation_timings
    }

    fn get_render_timings(&self) -> &Vec<String> {
        &self.render_timings
    }

    fn debug_current_buffer(&mut self) {
        // Debug output disabled - was corrupting TUI display
        // Use tracing/logging instead if debugging is needed
    }

    fn get_input_cursor(&self) -> usize {
        EnhancedTuiApp::get_input_cursor(self)
    }

    fn get_visual_cursor(&self) -> (usize, usize) {
        EnhancedTuiApp::get_visual_cursor(self)
    }

    fn get_input_text(&self) -> String {
        EnhancedTuiApp::get_input_text(self)
    }

    fn get_buffer_mut_if_available(&mut self) -> Option<&mut Buffer> {
        self.state_container.buffers_mut().current_mut()
    }

    fn set_mode_via_shadow_state(&mut self, mode: AppMode, trigger: &str) {
        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            debug!(
                "set_mode_via_shadow_state: Setting mode to {:?} with trigger '{}'",
                mode, trigger
            );
            self.shadow_state
                .borrow_mut()
                .set_mode(mode, buffer, trigger);
        } else {
            error!(
                "set_mode_via_shadow_state: No buffer available! Cannot set mode to {:?}",
                mode
            );
        }
    }

    fn collect_current_state(
        &self,
    ) -> (AppMode, String, String, Option<usize>, usize, usize, usize) {
        if let Some(buffer) = self.state_container.buffers().current() {
            let mode = buffer.get_mode();
            if mode == AppMode::Debug {
                (mode, String::new(), String::new(), None, 0, 0, 0)
            } else {
                (
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
            (
                AppMode::Command,
                String::new(),
                String::new(),
                None,
                0,
                0,
                0,
            )
        }
    }

    fn format_buffer_manager_state(&self) -> String {
        let buffer_names: Vec<String> = self
            .state_container
            .buffers()
            .all_buffers()
            .iter()
            .map(|b| b.get_name())
            .collect();
        let buffer_count = self.state_container.buffers().all_buffers().len();
        let buffer_index = self.state_container.buffers().current_index();

        format!(
            "\n========== BUFFER MANAGER STATE ==========\n\
            Number of Buffers: {}\n\
            Current Buffer Index: {}\n\
            Buffer Names: {}\n",
            buffer_count,
            buffer_index,
            buffer_names.join(", ")
        )
    }

    fn debug_generate_viewport_efficiency(&self) -> String {
        if let Some(ref efficiency) = *self.viewport_efficiency.borrow() {
            let mut result = String::from("\n========== VIEWPORT EFFICIENCY ==========\n");
            result.push_str(&efficiency.to_debug_string());
            result.push_str("\n==========================================\n");
            result
        } else {
            String::new()
        }
    }

    fn debug_generate_key_chord_info(&self) -> String {
        let mut result = String::from("\n");
        result.push_str(&self.key_chord_handler.format_debug_info());
        result.push_str("========================================\n");
        result
    }

    fn debug_generate_search_modes_info(&self) -> String {
        let mut result = String::from("\n");
        result.push_str(&self.search_modes_widget.debug_info());
        result
    }

    fn debug_generate_state_container_info(&self) -> String {
        let mut result = String::from("\n");
        result.push_str(&self.state_container.debug_dump());
        result.push_str("\n");
        result
    }

    fn collect_debug_info(&self) -> String {
        // Simplified version - the full version is in toggle_debug_mode
        let mut debug_info = String::new();
        debug_info
            .push_str(&self.debug_generate_parser_info(&self.state_container.get_input_text()));
        debug_info.push_str(&self.debug_generate_memory_info());
        debug_info
    }

    // Forward all the debug generation methods to the existing implementations
    fn debug_generate_parser_info(&self, query: &str) -> String {
        EnhancedTuiApp::debug_generate_parser_info(self, query)
    }

    fn debug_generate_navigation_state(&self) -> String {
        // Call the actual implementation method on self (defined below in impl EnhancedTuiApp)
        Self::debug_generate_navigation_state(self)
    }

    fn debug_generate_column_search_state(&self) -> String {
        // Call the actual implementation method on self (defined below in impl EnhancedTuiApp)
        Self::debug_generate_column_search_state(self)
    }

    fn debug_generate_trace_logs(&self) -> String {
        // Call the actual implementation method on self (defined below in impl EnhancedTuiApp)
        Self::debug_generate_trace_logs(self)
    }

    fn debug_generate_state_logs(&self) -> String {
        // Call the actual implementation method on self (defined below in impl EnhancedTuiApp)
        Self::debug_generate_state_logs(self)
    }
}

impl EnhancedTuiApp {
    // ========== STATE ACCESS ==========
    /// Get immutable reference to state container for debug purposes
    pub(crate) fn state_container(&self) -> &AppStateContainer {
        &self.state_container
    }

    /// Get mutable reference to state container for debug purposes  
    pub(crate) fn state_container_mut(&mut self) -> &mut AppStateContainer {
        &mut self.state_container
    }

    // ========== STATE SYNCHRONIZATION ==========

    /// Synchronize NavigationState with ViewportManager
    /// This is the reverse - update NavigationState from ViewportManager
    pub(crate) fn sync_navigation_with_viewport(&self) {
        debug!(target: "column_search_sync", "sync_navigation_with_viewport: ENTRY");
        let viewport_borrow = self.viewport_manager.borrow();

        if let Some(ref viewport) = viewport_borrow.as_ref() {
            let mut nav = self.state_container.navigation_mut();

            // Log current state before sync
            debug!(target: "column_search_sync", "sync_navigation_with_viewport: BEFORE - nav.selected_column: {}, viewport.crosshair_col: {}", 
                nav.selected_column, viewport.get_crosshair_col());
            debug!(target: "column_search_sync", "sync_navigation_with_viewport: BEFORE - nav.selected_row: {}, viewport.crosshair_row: {}", 
                nav.selected_row, viewport.get_crosshair_row());

            // Update NavigationState from ViewportManager's authoritative position
            nav.selected_row = viewport.get_selected_row();
            nav.selected_column = viewport.get_selected_column();
            nav.scroll_offset = viewport.get_scroll_offset();

            // Log state after sync
            debug!(target: "column_search_sync", "sync_navigation_with_viewport: AFTER - nav.selected_column: {}, nav.selected_row: {}", 
                nav.selected_column, nav.selected_row);
            debug!(target: "column_search_sync", "sync_navigation_with_viewport: Successfully synced NavigationState with ViewportManager");
        } else {
            debug!(target: "column_search_sync", "sync_navigation_with_viewport: No ViewportManager available to sync with");
        }
        debug!(target: "column_search_sync", "sync_navigation_with_viewport: EXIT");
    }

    /// Synchronize mode across all state containers
    /// This ensures AppStateContainer, Buffer, and ShadowState are all in sync
    fn sync_mode(&mut self, mode: AppMode, trigger: &str) {
        debug!(target: "column_search_sync", "sync_mode: ENTRY - mode: {:?}, trigger: '{}'", mode, trigger);
        // Delegate to StateCoordinator for centralized sync logic
        use crate::ui::state::state_coordinator::StateCoordinator;
        StateCoordinator::sync_mode_with_refs(
            &mut self.state_container,
            &self.shadow_state,
            mode,
            trigger,
        );
        debug!(target: "column_search_sync", "sync_mode: EXIT - StateCoordinator::sync_mode_with_refs completed");
    }

    /// Save current ViewportManager state to the current buffer
    fn save_viewport_to_current_buffer(&mut self) {
        let viewport_borrow = self.viewport_manager.borrow();

        if let Some(ref viewport) = viewport_borrow.as_ref() {
            if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
                // Save crosshair position
                buffer.set_selected_row(Some(viewport.get_selected_row()));
                buffer.set_current_column(viewport.get_selected_column());

                // Save scroll offset
                buffer.set_scroll_offset(viewport.get_scroll_offset());
            }
        }
    }

    /// Restore ViewportManager state from the current buffer
    fn restore_viewport_from_current_buffer(&mut self) {
        if let Some(buffer) = self.state_container.buffers().current() {
            // Check if this buffer has a DataView
            if let Some(dataview) = buffer.get_dataview() {
                // Check if we need to create or update ViewportManager
                let needs_new_viewport = {
                    let viewport_borrow = self.viewport_manager.borrow();
                    viewport_borrow.is_none()
                };

                if needs_new_viewport {
                    // Create new ViewportManager for this buffer's DataView
                    self.viewport_manager =
                        RefCell::new(Some(ViewportManager::new(Arc::new(dataview.clone()))));
                } else {
                    // Update existing ViewportManager with new DataView
                    let mut viewport_borrow = self.viewport_manager.borrow_mut();
                    if let Some(ref mut viewport) = viewport_borrow.as_mut() {
                        viewport.set_dataview(Arc::new(dataview.clone()));
                    }
                }

                // Now restore the position
                let mut viewport_borrow = self.viewport_manager.borrow_mut();
                if let Some(ref mut viewport) = viewport_borrow.as_mut() {
                    // The data dimensions are already updated by set_dataview above
                    // Just restore the saved position

                    // Restore crosshair position
                    let row = buffer.get_selected_row().unwrap_or(0);
                    let col = buffer.get_current_column();
                    viewport.set_crosshair(row, col);

                    // Restore scroll offset
                    let scroll_offset = buffer.get_scroll_offset();
                    viewport.set_scroll_offset(scroll_offset.0, scroll_offset.1);

                    // Update terminal size to trigger viewport recalculation
                    let term_width = viewport.get_terminal_width();
                    let term_height = viewport.get_terminal_height() as u16;
                    viewport.update_terminal_size(term_width, term_height);

                    // Also update NavigationState for consistency
                    drop(viewport_borrow);
                    self.sync_navigation_with_viewport();
                }

                // Also update TableWidgetManager with the new DataView
                let mut table_manager = self.table_widget_manager.borrow_mut();
                table_manager.set_dataview(Arc::new(dataview.clone()));
                table_manager.force_render(); // Force a re-render with new data
            }
        }
    }

    // ========== VIEWPORT CALCULATIONS ==========

    /// Calculate the number of data rows available for the table
    /// This accounts for all UI chrome (input area, status bar) and table chrome (header, borders)
    fn calculate_available_data_rows(terminal_height: u16) -> u16 {
        crate::ui::rendering::ui_layout_utils::calculate_available_data_rows(terminal_height)
    }

    /// Calculate the number of data rows available for a table area
    /// This accounts only for table chrome (header, borders)
    fn calculate_table_data_rows(table_area_height: u16) -> u16 {
        crate::ui::rendering::ui_layout_utils::calculate_table_data_rows(table_area_height)
    }

    // ========== BUFFER MANAGEMENT ==========

    /// Get current buffer if available (for reading)
    fn current_buffer(&self) -> Option<&dyn buffer::BufferAPI> {
        self.state_container
            .buffers()
            .current()
            .map(|b| b as &dyn buffer::BufferAPI)
    }

    /// Get current buffer (panics if none exists)
    /// Use this when we know a buffer should always exist
    fn buffer(&self) -> &dyn buffer::BufferAPI {
        self.current_buffer()
            .expect("No buffer available - this should not happen")
    }

    // ========== ACTION CONTEXT ==========

    /// Build action context from current state
    fn build_action_context(&self) -> ActionContext {
        let nav = self.state_container.navigation();
        let dataview = self.state_container.get_buffer_dataview();

        ActionContext {
            mode: self.state_container.get_mode(),
            selection_mode: self.state_container.get_selection_mode(),
            has_results: dataview.is_some(),
            has_filter: !self.state_container.get_filter_pattern().is_empty()
                || !self.state_container.get_fuzzy_filter_pattern().is_empty(),
            has_search: !self.state_container.get_search_pattern().is_empty()
                || self
                    .vim_search_adapter
                    .borrow()
                    .should_handle_key(&self.state_container)
                || self.state_container.column_search().is_active,
            row_count: dataview.as_ref().map_or(0, |v| v.row_count()),
            column_count: dataview.as_ref().map_or(0, |v| v.column_count()),
            current_row: nav.selected_row,
            current_column: nav.selected_column,
        }
    }

    // ========== ACTION HANDLERS ==========

    /// Try to handle an action using the new action system
    fn try_handle_action(
        &mut self,
        action: Action,
        context: &ActionContext,
    ) -> Result<ActionResult> {
        // First, try the visitor pattern action handlers
        // Create a temporary dispatcher to avoid borrowing conflicts
        let temp_dispatcher = crate::ui::input::action_handlers::ActionDispatcher::new();
        match temp_dispatcher.dispatch(&action, context, self) {
            Ok(ActionResult::NotHandled) => {
                // Action not handled by visitor pattern, fall back to existing switch
                // This allows for gradual migration
            }
            result => {
                // Action was handled (successfully or with error) by visitor pattern
                return result;
            }
        }

        use Action::*;

        // Fallback to existing switch statement for actions not yet in visitor pattern
        match action {
            // The following actions are now handled by visitor pattern handlers:
            // - Navigation: NavigationActionHandler
            // - Toggle operations: ToggleActionHandler (ToggleSelectionMode, ToggleRowNumbers, etc.)
            // - Clear operations: ClearActionHandler (ClearFilter, ClearLine)
            // - Exit operations: ExitActionHandler (Quit, ForceQuit)
            // - Column operations: ColumnActionHandler
            // - Export operations: ExportActionHandler
            // - Yank operations: YankActionHandler
            // - UI operations: UIActionHandler (ShowHelp)
            // - Debug/Viewport operations: DebugViewportActionHandler (ShowDebugInfo, StartJumpToRow, ToggleCursorLock, ToggleViewportLock)

            // NextColumn and PreviousColumn are now handled by NavigationActionHandler in visitor pattern
            Sort(_column_idx) => {
                // For now, always sort by current column (like 's' key does)
                self.toggle_sort_current_column();
                Ok(ActionResult::Handled)
            }
            // HideColumn and UnhideAllColumns are now handled by ColumnActionHandler in visitor pattern
            HideEmptyColumns => {
                tracing::info!("HideEmptyColumns action triggered");

                // Use ViewportManager to hide empty columns
                let (count, updated_dataview) = {
                    let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
                    if let Some(ref mut viewport_manager) = *viewport_manager_borrow {
                        tracing::debug!("ViewportManager available, checking for empty columns");
                        let count = viewport_manager.hide_empty_columns();
                        if count > 0 {
                            (count, Some(viewport_manager.clone_dataview()))
                        } else {
                            (count, None)
                        }
                    } else {
                        tracing::warn!("No ViewportManager available to hide columns");
                        (0, None)
                    }
                };

                // Sync the updated DataView back to the Buffer if columns were hidden
                if let Some(updated_dataview) = updated_dataview {
                    self.state_container.set_dataview(Some(updated_dataview));
                }

                tracing::info!("Hidden {} empty columns", count);
                let message = if count > 0 {
                    format!(
                        "Hidden {} empty columns (press Ctrl+Shift+H to unhide)",
                        count
                    )
                } else {
                    "No empty columns found".to_string()
                };
                self.state_container.set_status_message(message);
                Ok(ActionResult::Handled)
            }
            // MoveColumnLeft, MoveColumnRight - handled by ColumnArrangementActionHandler
            // StartSearch, StartColumnSearch, StartFilter, StartFuzzyFilter - handled by ModeActionHandler
            ExitCurrentMode => {
                // Handle escape based on current mode
                match context.mode {
                    AppMode::Results => {
                        // VimSearchAdapter now handles Escape in Results mode
                        // If we get here, search wasn't active, so switch to Command mode
                        // Save current position before switching to Command mode
                        if let Some(selected) = self.state_container.get_table_selected_row() {
                            self.state_container.set_last_results_row(Some(selected));
                            let scroll_offset = self.state_container.get_scroll_offset();
                            self.state_container.set_last_scroll_offset(scroll_offset);
                        }

                        // Restore the last executed query to input_text for editing
                        let last_query = self.state_container.get_last_query();
                        let current_input = self.state_container.get_input_text();
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
                        self.state_container.set_mode(AppMode::Command);
                        self.shadow_state
                            .borrow_mut()
                            .observe_mode_change(AppMode::Command, "escape_from_results");
                        self.state_container.set_table_selected_row(None);
                    }
                    AppMode::Help => {
                        // Return to previous mode (usually Results)
                        // Use proper mode synchronization
                        self.set_mode_via_shadow_state(AppMode::Results, "escape_from_help");
                        self.state_container.set_help_visible(false);
                    }
                    AppMode::Debug => {
                        // Return to Results mode
                        // Use proper mode synchronization
                        self.set_mode_via_shadow_state(AppMode::Results, "escape_from_debug");
                    }
                    _ => {
                        // For other modes, generally go back to Command
                        // Use proper mode synchronization
                        self.set_mode_via_shadow_state(AppMode::Command, "escape_to_command");
                    }
                }
                Ok(ActionResult::Handled)
            }
            SwitchMode(target_mode) => {
                // Switch to the specified mode
                // For Command->Results, only switch if we have results
                if target_mode == AppMode::Results && !context.has_results {
                    // Can't switch to Results mode without results
                    self.state_container.set_status_message(
                        "No results to display. Run a query first.".to_string(),
                    );
                    Ok(ActionResult::Handled)
                } else {
                    self.state_container.set_mode(target_mode.clone());

                    // Observe the mode change in shadow state
                    let trigger = match target_mode {
                        AppMode::Command => "switch_to_command",
                        AppMode::Results => "switch_to_results",
                        AppMode::Help => "switch_to_help",
                        AppMode::History => "switch_to_history",
                        _ => "switch_mode",
                    };
                    self.shadow_state
                        .borrow_mut()
                        .observe_mode_change(target_mode.clone(), trigger);

                    let msg = match target_mode {
                        AppMode::Command => "Command mode - Enter SQL queries",
                        AppMode::Results => {
                            "Results mode - Navigate with arrows/hjkl, Tab for command"
                        }
                        _ => "",
                    };
                    if !msg.is_empty() {
                        self.state_container.set_status_message(msg.to_string());
                    }
                    Ok(ActionResult::Handled)
                }
            }
            SwitchModeWithCursor(target_mode, cursor_position) => {
                use crate::ui::input::actions::{CursorPosition, SqlClause};

                // Switch to the target mode
                self.state_container.set_mode(target_mode.clone());

                // Observe the mode change in shadow state
                let trigger = match cursor_position {
                    CursorPosition::End => "a_key_pressed",
                    CursorPosition::Current => "i_key_pressed",
                    CursorPosition::AfterClause(_) => "clause_navigation",
                };
                self.shadow_state
                    .borrow_mut()
                    .observe_mode_change(target_mode.clone(), trigger);

                // Position the cursor based on the requested position
                match cursor_position {
                    CursorPosition::Current => {
                        // Keep cursor where it is (do nothing)
                    }
                    CursorPosition::End => {
                        // Move cursor to end of input
                        let text = self.state_container.get_buffer_input_text();
                        let text_len = text.len();
                        self.state_container.set_input_cursor_position(text_len);
                        // Also sync the TUI's input field
                        self.input = tui_input::Input::new(text).with_cursor(text_len);
                    }
                    CursorPosition::AfterClause(clause) => {
                        // Use the SQL parser to find the clause position
                        let input_text = self.state_container.get_input_text();

                        // Use the lexer to tokenize with positions
                        use crate::sql::recursive_parser::Lexer;
                        let mut lexer = Lexer::new(&input_text);
                        let tokens = lexer.tokenize_all_with_positions();

                        // Find the position after the specified clause
                        let mut cursor_pos = None;
                        for i in 0..tokens.len() {
                            let (_, end_pos, ref token) = tokens[i];

                            // Check if this token matches the clause we're looking for
                            use crate::sql::recursive_parser::Token;
                            let clause_matched = match (&clause, token) {
                                (SqlClause::Select, Token::Select) => true,
                                (SqlClause::From, Token::From) => true,
                                (SqlClause::Where, Token::Where) => true,
                                (SqlClause::OrderBy, Token::OrderBy) => true,
                                (SqlClause::GroupBy, Token::GroupBy) => true,
                                (SqlClause::Having, Token::Having) => true,
                                (SqlClause::Limit, Token::Limit) => true,
                                _ => false,
                            };

                            if clause_matched {
                                // Find the end of this clause (before the next keyword or end of query)
                                let mut clause_end = end_pos;

                                // Skip to the next significant token after the clause keyword
                                for j in (i + 1)..tokens.len() {
                                    let (_, token_end, ref next_token) = tokens[j];

                                    // Stop at the next SQL clause keyword
                                    match next_token {
                                        Token::Select
                                        | Token::From
                                        | Token::Where
                                        | Token::OrderBy
                                        | Token::GroupBy
                                        | Token::Having
                                        | Token::Limit => break,
                                        _ => clause_end = token_end,
                                    }
                                }

                                cursor_pos = Some(clause_end);
                                break;
                            }
                        }

                        // If we found the clause, position cursor after it
                        // If not found, append at the end with the clause
                        if let Some(pos) = cursor_pos {
                            self.state_container.set_input_cursor_position(pos);
                            // Also sync the TUI's input field
                            let text = self.state_container.get_buffer_input_text();
                            self.input = tui_input::Input::new(text).with_cursor(pos);
                        } else {
                            // Clause not found, append it at the end
                            let clause_text = match clause {
                                SqlClause::Where => " WHERE ",
                                SqlClause::OrderBy => " ORDER BY ",
                                SqlClause::GroupBy => " GROUP BY ",
                                SqlClause::Having => " HAVING ",
                                SqlClause::Limit => " LIMIT ",
                                SqlClause::Select => "SELECT ",
                                SqlClause::From => " FROM ",
                            };

                            let mut new_text = self.state_container.get_input_text();
                            new_text.push_str(clause_text);
                            let cursor_pos = new_text.len();
                            // Use the helper method that syncs everything
                            self.set_input_text_with_cursor(new_text, cursor_pos);
                        }
                    }
                }

                // Update status message
                let msg = match target_mode {
                    AppMode::Command => "Command mode - Enter SQL queries",
                    _ => "",
                };
                if !msg.is_empty() {
                    self.state_container.set_status_message(msg.to_string());
                }

                Ok(ActionResult::Handled)
            }

            // Editing actions - only work in Command mode
            // MoveCursorLeft is now handled by InputCursorActionHandler in visitor pattern
            MoveCursorLeft => Ok(ActionResult::NotHandled),
            // MoveCursorRight is now handled by InputCursorActionHandler in visitor pattern
            MoveCursorRight => Ok(ActionResult::NotHandled),
            // MoveCursorHome is now handled by InputCursorActionHandler in visitor pattern
            MoveCursorHome => Ok(ActionResult::NotHandled),
            // MoveCursorEnd is now handled by InputCursorActionHandler in visitor pattern
            MoveCursorEnd => Ok(ActionResult::NotHandled),
            // Backspace is now handled by TextEditActionHandler in visitor pattern
            Backspace => Ok(ActionResult::NotHandled),
            // Delete is now handled by TextEditActionHandler in visitor pattern
            Delete => Ok(ActionResult::NotHandled),
            // ClearLine is now handled by ClearActionHandler in visitor pattern
            // Undo is now handled by TextEditActionHandler in visitor pattern
            Undo => Ok(ActionResult::NotHandled),
            // Redo is now handled by TextEditActionHandler in visitor pattern
            Redo => Ok(ActionResult::NotHandled),
            ExecuteQuery => {
                if context.mode == AppMode::Command {
                    // Delegate to existing execute query logic
                    self.handle_execute_query()?;
                    Ok(ActionResult::Handled)
                } else {
                    Ok(ActionResult::NotHandled)
                }
            }
            InsertChar(c) => {
                if context.mode == AppMode::Command {
                    self.state_container.insert_char_at_cursor(c);

                    // Clear completion state when typing
                    self.state_container.clear_completion();

                    // Handle completion
                    self.handle_completion();

                    Ok(ActionResult::Handled)
                } else {
                    Ok(ActionResult::NotHandled)
                }
            }

            NextSearchMatch => {
                // n key: navigate to next search match only if search is active (not after Escape)
                debug!(target: "column_search_sync", "NextSearchMatch: 'n' key pressed, checking if search navigation should be handled");
                // Use StateCoordinator to determine if search navigation should be handled
                use crate::ui::state::state_coordinator::StateCoordinator;
                let should_handle = StateCoordinator::should_handle_next_match(
                    &self.state_container,
                    Some(&self.vim_search_adapter),
                );
                debug!(target: "column_search_sync", "NextSearchMatch: StateCoordinator::should_handle_next_match returned: {}", should_handle);
                if should_handle {
                    debug!(target: "column_search_sync", "NextSearchMatch: Calling vim_search_next()");
                    self.vim_search_next();
                    debug!(target: "column_search_sync", "NextSearchMatch: vim_search_next() completed");
                } else {
                    debug!(target: "column_search_sync", "NextSearchMatch: No active search (or cancelled with Escape), ignoring 'n' key");
                }
                Ok(ActionResult::Handled)
            }
            PreviousSearchMatch => {
                // Shift+N behavior: search navigation only if vim search is active, otherwise toggle row numbers
                // Use StateCoordinator to determine if search navigation should be handled
                use crate::ui::state::state_coordinator::StateCoordinator;
                if StateCoordinator::should_handle_previous_match(
                    &self.state_container,
                    Some(&self.vim_search_adapter),
                ) {
                    self.vim_search_previous();
                } else {
                    // Delegate to the ToggleRowNumbers action for consistency
                    return self.try_handle_action(Action::ToggleRowNumbers, context);
                }
                Ok(ActionResult::Handled)
            }
            ShowColumnStatistics => {
                self.calculate_column_statistics();
                Ok(ActionResult::Handled)
            }
            StartHistorySearch => {
                use crate::ui::state::state_coordinator::StateCoordinator;

                // Get current input before delegating
                let current_input = self.get_input_text();

                // Use StateCoordinator for all state transitions
                let (input_to_use, _match_count) = StateCoordinator::start_history_search_with_refs(
                    &mut self.state_container,
                    &self.shadow_state,
                    current_input,
                );

                // Update input if it changed (e.g., from Results mode)
                if input_to_use != self.get_input_text() {
                    self.set_input_text(input_to_use);
                }

                // Initialize with schema context (implementation stays in TUI)
                self.update_history_matches_in_container();

                Ok(ActionResult::Handled)
            }
            CycleColumnPacking => {
                let message = {
                    let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
                    let viewport_manager = viewport_manager_borrow
                        .as_mut()
                        .expect("ViewportManager must exist");
                    let new_mode = viewport_manager.cycle_packing_mode();
                    format!("Column packing: {}", new_mode.display_name())
                };
                self.state_container.set_status_message(message);
                Ok(ActionResult::Handled)
            }
            _ => {
                // Action not yet implemented in new system
                Ok(ActionResult::NotHandled)
            }
        }
    }

    // ========== DATA PROVIDER ACCESS ==========

    /// Get a DataProvider view of the current buffer
    /// This allows using the new trait-based data access pattern
    fn get_data_provider(&self) -> Option<Box<dyn DataProvider + '_>> {
        // For now, we'll use BufferAdapter for Buffer data
        // In the future, we can check data source type and return appropriate adapter
        if let Some(buffer) = self.state_container.buffers().current() {
            // V51: Check for DataView first, then DataTable
            if buffer.has_dataview() {
                return Some(Box::new(BufferAdapter::new(buffer)));
            }
        }
        None
    }

    // Note: edit_mode methods removed - use buffer directly

    // ========== INPUT MANAGEMENT ==========

    // Helper to get input text from buffer or fallback to direct input
    fn get_input_text(&self) -> String {
        // For special modes that use the input field for their own purposes
        if self.shadow_state.borrow().is_in_search_mode() {
            // These modes temporarily use the input field for their patterns
            self.input.value().to_string() // TODO: Migrate to buffer-based input
        } else {
            // Get from the actual buffer to ensure consistency
            self.state_container.get_buffer_input_text()
        }
    }

    // Helper to get cursor position from buffer or fallback to direct input
    fn get_input_cursor(&self) -> usize {
        // For special modes that use the input field directly
        if self.shadow_state.borrow().is_in_search_mode() {
            // These modes use the input field for their patterns
            self.input.cursor()
        } else {
            // All other modes use the buffer
            self.state_container.get_input_cursor_position()
        }
    }

    // Helper to set input text through buffer and sync input field
    fn set_input_text(&mut self, text: String) {
        let old_text = self.state_container.get_input_text();
        let mode = self.shadow_state.borrow().get_mode();

        // Log every input text change with context
        info!(target: "input", "SET_INPUT_TEXT: '{}' -> '{}' (mode: {:?})",
              if old_text.len() > 50 { format!("{}...", &old_text[..50]) } else { old_text.clone() },
              if text.len() > 50 { format!("{}...", &text[..50]) } else { text.clone() },
              mode);

        // Use the proper proxy method that syncs both buffer and command_input
        self.state_container.set_buffer_input_text(text.clone());

        // Always update the input field for all modes
        // TODO: Eventually migrate special modes to use buffer input
        self.input = tui_input::Input::new(text.clone()).with_cursor(text.len());
    }

    // Helper to set input text with specific cursor position
    fn set_input_text_with_cursor(&mut self, text: String, cursor_pos: usize) {
        let old_text = self.state_container.get_buffer_input_text();
        let old_cursor = self.state_container.get_input_cursor_position();
        let mode = self.state_container.get_mode();

        // Log every input text change with cursor position
        info!(target: "input", "SET_INPUT_TEXT_WITH_CURSOR: '{}' (cursor {}) -> '{}' (cursor {}) (mode: {:?})",
              if old_text.len() > 50 { format!("{}...", &old_text[..50]) } else { old_text.clone() },
              old_cursor,
              if text.len() > 50 { format!("{}...", &text[..50]) } else { text.clone() },
              cursor_pos,
              mode);

        // Use the proper proxy method that syncs both buffer and command_input
        self.state_container
            .set_buffer_input_text_with_cursor(text.clone(), cursor_pos);

        // Always update the input field for consistency
        // TODO: Eventually migrate special modes to use buffer input
        self.input = tui_input::Input::new(text.clone()).with_cursor(cursor_pos);
    }

    // MASTER SYNC METHOD - Use this whenever input changes!
    // This ensures all three input states stay synchronized:
    // 1. Buffer's input_text and cursor
    // 2. self.input (tui_input widget)
    // 3. AppStateContainer's command_input
    fn sync_all_input_states(&mut self) {
        let text = self.state_container.get_input_text();
        let cursor = self.state_container.get_input_cursor_position();
        let mode = self.state_container.get_mode();

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

    // Helper to handle key events in the input
    fn handle_input_key(&mut self, key: KeyEvent) -> bool {
        // For special modes that handle input directly
        let mode = self.shadow_state.borrow().get_mode();
        match mode {
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                self.input.handle_event(&Event::Key(key));
                false
            }
            _ => {
                // Route to buffer's input handling
                self.state_container.handle_input_key(key)
            }
        }
    }
    // ========== CURSOR AND SELECTION ==========

    // Helper to get visual cursor position (for rendering)
    fn get_visual_cursor(&self) -> (usize, usize) {
        // Get text and cursor from appropriate source based on mode
        let mode = self.state_container.get_mode();
        let (text, cursor) = match mode {
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                // Special modes use self.input directly
                (self.input.value().to_string(), self.input.cursor())
            }
            _ => {
                // Other modes use state_container
                (
                    self.state_container.get_input_text(),
                    self.state_container.get_input_cursor_position(),
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

    fn get_selection_mode(&self) -> SelectionMode {
        self.state_container.get_selection_mode()
        // ========== UTILITY FUNCTIONS ==========
    }

    pub fn new(api_url: &str) -> Self {
        // Load configuration
        let config = Config::load().unwrap_or_else(|_e| {
            // Config loading error - using defaults
            Config::default()
        });

        // Store API URL as data source if provided
        let data_source = if !api_url.is_empty() {
            Some(api_url.to_string())
        } else {
            None
        };

        // Log initialization
        if let Some(logger) = dual_logging::get_dual_logger() {
            logger.log(
                "INFO",
                "EnhancedTuiApp",
                &format!("Initializing TUI with API URL: {}", api_url),
            );
        }

        // Create buffer manager for the state container (no longer duplicated)
        let mut buffer_manager = BufferManager::new();
        let mut buffer = buffer::Buffer::new(1);
        // Sync initial settings from config
        buffer.set_case_insensitive(config.behavior.case_insensitive_default);
        buffer.set_compact_mode(config.display.compact_mode);
        buffer.set_show_row_numbers(config.display.show_row_numbers);
        buffer_manager.add_buffer(buffer);

        // Initialize state container (owns the only buffer_manager)
        let state_container = match AppStateContainer::new(buffer_manager) {
            Ok(container) => container,
            Err(e) => {
                panic!("Failed to initialize AppStateContainer: {}", e);
            }
        };

        // Initialize debug service directly (no ServiceContainer needed)
        let debug_service = DebugService::new(1000); // Keep last 1000 debug entries
        debug_service.set_enabled(true); // Enable the debug service
        state_container.set_debug_service(debug_service.clone_service());

        // Create help widget
        let help_widget = HelpWidget::new();
        // Note: help_widget.set_services() removed - ServiceContainer no longer exists

        let mut app = Self {
            state_container,
            debug_service: Some(debug_service),
            input: Input::default(),
            command_editor: CommandEditor::new(),
            cursor_manager: CursorManager::new(),
            data_analyzer: DataAnalyzer::new(),
            hybrid_parser: HybridParser::new(),
            config: config.clone(),
            sql_highlighter: SqlHighlighter::new(),
            debug_widget: DebugWidget::new(),
            editor_widget: EditorWidget::new(),
            stats_widget: StatsWidget::new(),
            help_widget,
            search_modes_widget: SearchModesWidget::new(),
            vim_search_adapter: RefCell::new(VimSearchAdapter::new()),
            state_dispatcher: RefCell::new(StateDispatcher::new()),
            search_manager: RefCell::new({
                let mut search_config = SearchConfig::default();
                search_config.case_sensitive = !config.behavior.case_insensitive_default;
                SearchManager::with_config(search_config)
            }),
            key_chord_handler: KeyChordHandler::new(),
            key_dispatcher: KeyDispatcher::new(),
            key_mapper: KeyMapper::new(),
            // ========== INITIALIZATION ==========
            // CSV fields now in Buffer (buffer_manager in AppStateContainer)
            buffer_handler: BufferHandler::new(),
            navigation_timings: Vec::new(),
            render_timings: Vec::new(),
            log_buffer: dual_logging::get_dual_logger().map(|logger| logger.ring_buffer().clone()),
            data_source,
            key_indicator: {
                let mut indicator = KeyPressIndicator::new();
                if config.display.show_key_indicator {
                    indicator.set_enabled(true);
                }
                indicator
            },
            key_sequence_renderer: {
                let mut renderer = KeySequenceRenderer::new();
                if config.display.show_key_indicator {
                    renderer.set_enabled(true);
                }
                renderer
            },
            viewport_manager: RefCell::new(None), // Will be initialized when DataView is set
            viewport_efficiency: RefCell::new(None),
            shadow_state: RefCell::new(crate::ui::state::shadow_state::ShadowStateManager::new()),
            table_widget_manager: RefCell::new(TableWidgetManager::new()),
            query_orchestrator: QueryOrchestrator::new(
                config.behavior.case_insensitive_default,
                config.behavior.hide_empty_columns,
            ),
            debug_registry: DebugRegistry::new(),
            memory_tracker: MemoryTracker::new(100),
        };

        // Set up state dispatcher
        app.setup_state_coordination();

        app
    }

    /// Set up state coordination between dispatcher and components
    fn setup_state_coordination(&mut self) {
        // Connect state dispatcher to current buffer
        if let Some(current_buffer) = self.state_container.buffers().current() {
            let buffer_rc = std::rc::Rc::new(std::cell::RefCell::new(current_buffer.clone()));
            self.state_dispatcher.borrow_mut().set_buffer(buffer_rc);
        }

        // NOTE: We would add VimSearchAdapter as subscriber here, but it requires
        // moving the adapter out of RefCell temporarily. For now, we'll handle
        // this connection when events are dispatched.

        info!("State coordination setup complete");
    }

    /// Create a TUI with a DataView - no file loading knowledge needed
    /// This is the clean separation: TUI only knows about DataViews
    pub fn new_with_dataview(dataview: DataView, source_name: &str) -> Result<Self> {
        // Create the base app
        let mut app = Self::new("");

        // Store the data source name
        app.data_source = Some(source_name.to_string());

        // Create a buffer with the DataView
        app.state_container.buffers_mut().clear_all();
        let mut buffer = buffer::Buffer::new(1);

        // Set the DataView directly
        buffer.set_dataview(Some(dataview.clone()));
        // Use just the filename for the buffer name, not the full path
        let buffer_name = std::path::Path::new(source_name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(source_name)
            .to_string();
        buffer.set_name(buffer_name);

        // Apply config settings to the buffer
        buffer.set_case_insensitive(app.config.behavior.case_insensitive_default);
        buffer.set_compact_mode(app.config.display.compact_mode);
        buffer.set_show_row_numbers(app.config.display.show_row_numbers);

        // Add the buffer
        app.state_container.buffers_mut().add_buffer(buffer);

        // Update state container with the DataView
        app.state_container.set_dataview(Some(dataview.clone()));

        // Initialize viewport manager with the DataView
        app.viewport_manager = RefCell::new(Some(ViewportManager::new(Arc::new(dataview.clone()))));

        // Calculate initial column widths
        app.calculate_optimal_column_widths();

        // Set initial navigation state
        let row_count = dataview.row_count();
        let column_count = dataview.column_count();
        app.state_container
            .update_data_size(row_count, column_count);

        Ok(app)
    }

    /// Add a DataView to the existing TUI (creates a new buffer)
    pub fn add_dataview(&mut self, dataview: DataView, source_name: &str) -> Result<()> {
        // Delegate all the complex state coordination to StateCoordinator
        use crate::ui::state::state_coordinator::StateCoordinator;

        StateCoordinator::add_dataview_with_refs(
            &mut self.state_container,
            &self.viewport_manager,
            dataview,
            source_name,
            &self.config,
        )?;

        // TUI-specific: Calculate optimal column widths for display
        self.calculate_optimal_column_widths();

        Ok(())
    }

    /// Update the viewport with a new DataView
    pub fn update_viewport_with_dataview(&mut self, dataview: DataView) {
        self.viewport_manager = RefCell::new(Some(ViewportManager::new(Arc::new(dataview))));
    }

    /// Get vim search adapter (public for orchestrator)
    pub fn vim_search_adapter(&self) -> &RefCell<VimSearchAdapter> {
        &self.vim_search_adapter
    }

    /// Pre-populate SQL command with SELECT * FROM table
    pub fn set_sql_query(&mut self, table_name: &str, raw_table_name: &str) {
        use crate::ui::state::state_coordinator::StateCoordinator;

        // Delegate all state coordination to StateCoordinator
        let auto_query = StateCoordinator::set_sql_query_with_refs(
            &mut self.state_container,
            &self.shadow_state,
            &mut self.hybrid_parser,
            table_name,
            raw_table_name,
            &self.config,
        );

        // Pre-populate the input field with the query
        self.set_input_text(auto_query);
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

    /// Initialize viewport and perform initial draw
    fn initialize_viewport<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        self.update_viewport_size();
        info!(target: "navigation", "Initial viewport size update completed");
        terminal.draw(|f| self.ui(f))?;
        Ok(())
    }

    /// Handle debounced search actions, returns true if exit is requested
    fn try_handle_debounced_actions<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<bool> {
        if !self.search_modes_widget.is_active() {
            return Ok(false);
        }

        if let Some(action) = self.search_modes_widget.check_debounce() {
            let mut needs_redraw = false;
            match action {
                SearchModesAction::ExecuteDebounced(mode, pattern) => {
                    info!(target: "search", "=== DEBOUNCED SEARCH EXECUTING ===");
                    info!(target: "search", "Mode: {:?}, Pattern: '{}', AppMode: {:?}",
                          mode, pattern, self.shadow_state.borrow().get_mode());

                    // Log current position before search
                    {
                        let nav = self.state_container.navigation();
                        info!(target: "search", "BEFORE: nav.selected_row={}, nav.selected_column={}",
                              nav.selected_row, nav.selected_column);
                        info!(target: "search", "BEFORE: buffer.selected_row={:?}, buffer.current_column={}",
                              self.state_container.get_buffer_selected_row(), self.state_container.get_current_column());
                    }

                    self.execute_search_action(mode, pattern);

                    // Log position after search
                    {
                        let nav = self.state_container.navigation();
                        info!(target: "search", "AFTER: nav.selected_row={}, nav.selected_column={}",
                              nav.selected_row, nav.selected_column);
                        info!(target: "search", "AFTER: buffer.selected_row={:?}, buffer.current_column={}",
                              self.state_container.get_buffer_selected_row(), self.state_container.get_current_column());

                        // Check ViewportManager state
                        let viewport_manager = self.viewport_manager.borrow();
                        if let Some(ref vm) = *viewport_manager {
                            info!(target: "search", "AFTER: ViewportManager crosshair=({}, {})",
                                  vm.get_crosshair_row(), vm.get_crosshair_col());
                        }
                    }

                    info!(target: "search", "=== FORCING REDRAW ===");
                    // CRITICAL: Force immediate redraw after search navigation
                    needs_redraw = true;
                }
                _ => {}
            }

            // Redraw immediately if search moved the cursor OR if TableWidgetManager needs render
            if needs_redraw || self.table_widget_manager.borrow().needs_render() {
                info!(target: "search", "Triggering redraw: needs_redraw={}, table_needs_render={}",
                      needs_redraw, self.table_widget_manager.borrow().needs_render());
                terminal.draw(|f| self.ui(f))?;
                self.table_widget_manager.borrow_mut().rendered();
            }
        }
        Ok(false)
    }

    /// Handle chord processing for Results mode, returns true if exit is requested
    fn try_handle_chord_processing(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // FIRST: Give VimSearchAdapter a chance to handle the key
        // This allows it to handle search navigation (n/N) and Escape in Results mode
        if self
            .vim_search_adapter
            .borrow()
            .should_handle_key(&self.state_container)
        {
            let handled = self
                .vim_search_adapter
                .borrow_mut()
                .handle_key(key.code, &mut self.state_container);
            if handled {
                debug!("VimSearchAdapter handled key: {:?}", key.code);
                return Ok(false); // Key was handled, don't exit
            }
        }

        // SECOND: Try buffer operations (F11/F12, Ctrl-6, etc) - these should work in Results mode too
        if let Some(result) = self.try_handle_buffer_operations(&key)? {
            return Ok(result);
        }

        let chord_result = self.key_chord_handler.process_key(key);
        debug!("Chord handler returned: {:?}", chord_result);

        match chord_result {
            ChordResult::CompleteChord(action) => {
                // Handle completed chord actions through the action system
                debug!("Chord completed: {:?}", action);
                // Clear chord mode in renderer
                self.key_sequence_renderer.clear_chord_mode();

                // Get mode before the borrow
                let current_mode = self.shadow_state.borrow().get_mode();

                // Use the action system to handle the chord action
                self.try_handle_action(
                    action,
                    &ActionContext {
                        mode: current_mode,
                        selection_mode: self.state_container.get_selection_mode(),
                        has_results: self.state_container.get_buffer_dataview().is_some(),
                        has_filter: false,
                        has_search: false,
                        row_count: self.get_row_count(),
                        column_count: self.get_column_count(),
                        current_row: self.state_container.get_table_selected_row().unwrap_or(0),
                        current_column: self.state_container.get_current_column(),
                    },
                )?;
                Ok(false)
            }
            ChordResult::PartialChord(description) => {
                // Update status to show chord mode
                self.state_container.set_status_message(description.clone());
                // Update chord mode in renderer with available completions
                // Extract the completions from the description
                if description.contains("y=row") {
                    self.key_sequence_renderer
                        .set_chord_mode(Some("y(a,c,q,r,v)".to_string()));
                } else {
                    self.key_sequence_renderer
                        .set_chord_mode(Some(description.clone()));
                }
                Ok(false) // Don't exit, waiting for more keys
            }
            ChordResult::Cancelled => {
                self.state_container
                    .set_status_message("Chord cancelled".to_string());
                // Clear chord mode in renderer
                self.key_sequence_renderer.clear_chord_mode();
                Ok(false)
            }
            ChordResult::SingleKey(single_key) => {
                // Not a chord, process normally
                self.handle_results_input(single_key)
            }
        }
    }

    /// Dispatch key to appropriate mode handler, returns true if exit is requested
    fn try_handle_mode_dispatch(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        let mode = self.shadow_state.borrow().get_mode();
        debug!(
            "try_handle_mode_dispatch: mode={:?}, key={:?}",
            mode, key.code
        );
        match mode {
            AppMode::Command => self.handle_command_input(key),
            AppMode::Results => {
                // Results mode uses chord processing
                self.try_handle_chord_processing(key)
            }
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch => {
                self.handle_search_modes_input(key)
            }
            AppMode::Help => self.handle_help_input(key),
            AppMode::History => self.handle_history_input(key),
            AppMode::Debug => self.handle_debug_input(key),
            AppMode::PrettyQuery => self.handle_pretty_query_input(key),
            AppMode::JumpToRow => self.handle_jump_to_row_input(key),
            AppMode::ColumnStats => self.handle_column_stats_input(key),
        }
    }

    /// Handle key event processing, returns true if exit is requested
    fn try_handle_key_event<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        key: crossterm::event::KeyEvent,
    ) -> Result<bool> {
        // On Windows, filter out key release events - only handle key press
        // This prevents double-triggering of toggles
        if key.kind != crossterm::event::KeyEventKind::Press {
            return Ok(false);
        }

        // SAFETY: Always allow Ctrl-C to exit, regardless of app state
        // This prevents getting stuck in unresponsive states
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            info!(target: "app", "Ctrl-C detected, forcing exit");
            return Ok(true);
        }

        // Record key press for visual indicator
        let key_display = format_key_for_display(&key);
        self.key_indicator.record_key(key_display.clone());
        self.key_sequence_renderer.record_key(key_display);

        // Dispatch to appropriate mode handler
        let should_exit = self.try_handle_mode_dispatch(key)?;

        if should_exit {
            return Ok(true);
        }

        // Only redraw after handling a key event OR if TableWidgetManager needs render
        if self.table_widget_manager.borrow().needs_render() {
            info!("TableWidgetManager needs render after key event");
        }
        terminal.draw(|f| self.ui(f))?;
        self.table_widget_manager.borrow_mut().rendered();

        Ok(false)
    }

    /// Handle all events, returns true if exit is requested
    fn try_handle_events<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<bool> {
        // Use poll with timeout to allow checking for debounced actions
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if self.try_handle_key_event(terminal, key)? {
                        return Ok(true);
                    }
                }
                _ => {
                    // Ignore other events (mouse, resize, etc.) to reduce CPU
                }
            }
        } else {
            // No event available, but still redraw if we have pending debounced actions or table needs render
            if self.search_modes_widget.is_active()
                || self.table_widget_manager.borrow().needs_render()
            {
                if self.table_widget_manager.borrow().needs_render() {
                    info!("TableWidgetManager needs periodic render");
                }
                terminal.draw(|f| self.ui(f))?;
                self.table_widget_manager.borrow_mut().rendered();
            }
        }
        Ok(false)
    }

    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        self.initialize_viewport(terminal)?;

        loop {
            // Handle debounced search actions
            if self.try_handle_debounced_actions(terminal)? {
                break;
            }

            // Handle all events (key presses, etc.)
            if self.try_handle_events(terminal)? {
                break;
            }
        }
        Ok(())
    }

    fn handle_command_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Normalize and log the key
        let normalized_key = self.normalize_and_log_key(key);

        // === PHASE 1: Try CommandEditor for text input and editing ===
        // IMPORTANT: This must come BEFORE try_action_system to properly intercept keys
        // We handle comprehensive text editing operations
        // Let CommandEditor handle most text-related keys (except Tab which needs full TUI state)

        // Check for special Ctrl/Alt combinations that should NOT go to CommandEditor
        let is_special_combo = if let KeyCode::Char(c) = normalized_key.code {
            // Special Ctrl combinations
            (normalized_key.modifiers.contains(KeyModifiers::CONTROL) && matches!(c,
                'x' | 'X' | // Expand asterisk
                'p' | 'P' | // Previous history  
                'n' | 'N' | // Next history
                'r' | 'R' | // History search
                'j' | 'J' | // Export JSON
                'o' | 'O' | // Open buffer
                'b' | 'B' | // Buffer operations
                'l' | 'L'   // Clear screen
            )) ||
            // Special Alt combinations that aren't word navigation
            (normalized_key.modifiers.contains(KeyModifiers::ALT) && matches!(c,
                'x' | 'X'   // Expand asterisk visible only
            ))
        } else {
            false
        };

        let should_try_command_editor = !is_special_combo
            && matches!(
                normalized_key.code,
                KeyCode::Char(_)
                    | KeyCode::Backspace
                    | KeyCode::Delete
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::Home
                    | KeyCode::End
            );

        if should_try_command_editor {
            // IMPORTANT: Sync state TO CommandEditor before processing
            // This ensures CommandEditor has the current text/cursor position
            let before_text = self.input.value().to_string();
            let before_cursor = self.input.cursor();

            if self.command_editor.get_text() != before_text {
                self.command_editor.set_text(before_text.clone());
            }
            if self.command_editor.get_cursor() != before_cursor {
                self.command_editor.set_cursor(before_cursor);
            }

            // Now handle the input with proper state
            let result = self.command_editor.handle_input(
                normalized_key.clone(),
                &mut self.state_container,
                &self.shadow_state,
            )?;

            // Sync the text back to the main input after processing
            // (This is temporary until we fully migrate)
            let new_text = self.command_editor.get_text();
            let new_cursor = self.command_editor.get_cursor();

            // Debug logging to see what's happening
            if new_text != before_text || new_cursor != before_cursor {
                debug!(
                    "CommandEditor changed input: '{}' -> '{}', cursor: {} -> {}",
                    before_text, new_text, before_cursor, new_cursor
                );
            }

            // Use from() instead of new() to preserve any internal state
            self.input = tui_input::Input::from(new_text.clone()).with_cursor(new_cursor);

            // CRITICAL: Update the buffer, not just command_input!
            // The rendering uses get_buffer_input_text() which reads from the buffer
            if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
                buffer.set_input_text(new_text.clone());
                buffer.set_input_cursor_position(new_cursor);
            }

            // Also update command_input for consistency
            self.state_container.set_input_text(new_text);
            self.state_container.set_input_cursor_position(new_cursor);

            if result {
                return Ok(true);
            }

            // For char/backspace, we've handled it - don't let action system process it again
            return Ok(false);
        }
        // === END PHASE 1 ===

        // Try the new action system first
        if let Some(result) = self.try_action_system(normalized_key.clone())? {
            return Ok(result);
        }

        // Try editor widget for high-level actions
        if let Some(result) = self.try_editor_widget(normalized_key.clone())? {
            return Ok(result);
        }

        // ORIGINAL LOGIC: Keep all existing logic as fallback

        // Try history navigation first
        if let Some(result) = self.try_handle_history_navigation(&normalized_key)? {
            return Ok(result);
        }

        // Store old cursor position
        let old_cursor = self.get_input_cursor();

        // Also log to tracing
        trace!(target: "input", "Key: {:?} Modifiers: {:?}", key.code, key.modifiers);

        // DON'T process chord handler in Command mode - yanking makes no sense when editing queries!
        // The 'y' key should just type 'y' in the query editor.

        // Try buffer operations
        if let Some(result) = self.try_handle_buffer_operations(&key)? {
            return Ok(result);
        }

        // Try function keys
        if let Some(result) = self.try_handle_function_keys(&key)? {
            return Ok(result);
        }

        // Try text editing operations
        if let Some(result) = self.try_handle_text_editing(&key)? {
            return Ok(result);
        }

        // Try mode transitions and core input handling
        if let Some(result) = self.try_handle_mode_transitions(&key, old_cursor)? {
            return Ok(result);
        }

        // All input should be handled by the try_handle_* methods above
        // If we reach here, it means we missed handling a key combination

        Ok(false)
    }

    // ========== COMMAND INPUT HELPER METHODS ==========
    // These helpers break down the massive handle_command_input method into logical groups

    /// Normalize key for platform differences and log it
    fn normalize_and_log_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> crossterm::event::KeyEvent {
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

        normalized
    }

    /// Try handling with the new action system
    fn try_action_system(
        &mut self,
        normalized_key: crossterm::event::KeyEvent,
    ) -> Result<Option<bool>> {
        let action_context = self.build_action_context();
        if let Some(action) = self
            .key_mapper
            .map_key(normalized_key.clone(), &action_context)
        {
            info!(
                "✓ Action system (Command): key {:?} -> action {:?}",
                normalized_key.code, action
            );
            if let Ok(result) = self.try_handle_action(action, &action_context) {
                match result {
                    ActionResult::Handled => {
                        debug!("Action handled by new system in Command mode");
                        return Ok(Some(false));
                    }
                    ActionResult::Exit => {
                        return Ok(Some(true));
                    }
                    ActionResult::NotHandled => {
                        // Fall through to existing handling
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }

    /// Try handling with editor widget
    fn try_editor_widget(
        &mut self,
        normalized_key: crossterm::event::KeyEvent,
    ) -> Result<Option<bool>> {
        let key_dispatcher = self.key_dispatcher.clone();
        let editor_result = if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            self.editor_widget
                .handle_key(normalized_key.clone(), &key_dispatcher, buffer)?
        } else {
            EditorAction::PassToMainApp(normalized_key.clone())
        };

        match editor_result {
            EditorAction::Quit => return Ok(Some(true)),
            EditorAction::ExecuteQuery => {
                return self.handle_execute_query().map(Some);
            }
            EditorAction::BufferAction(buffer_action) => {
                return self.handle_buffer_action(buffer_action).map(Some);
            }
            EditorAction::ExpandAsterisk => {
                return self.handle_expand_asterisk().map(Some);
            }
            EditorAction::ShowHelp => {
                self.state_container.set_help_visible(true);
                // Use proper mode synchronization
                self.set_mode_via_shadow_state(AppMode::Help, "help_requested");
                return Ok(Some(false));
            }
            EditorAction::ShowDebug => {
                // This is now handled by passing through to original F5 handler
                return Ok(Some(false));
            }
            EditorAction::ShowPrettyQuery => {
                self.show_pretty_query();
                return Ok(Some(false));
            }
            EditorAction::SwitchMode(mode) => {
                self.handle_editor_mode_switch(mode);
                return Ok(Some(false));
            }
            EditorAction::PassToMainApp(_) => {
                // Fall through to original logic
            }
            EditorAction::Continue => return Ok(Some(false)),
        }

        Ok(None)
    }

    /// Handle mode switch from editor widget
    fn handle_editor_mode_switch(&mut self, mode: AppMode) {
        debug!(target: "shadow_state", "EditorAction::SwitchMode to {:?}", mode);
        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            // Use shadow state to set mode (with write-through to buffer)
            let trigger = match mode {
                AppMode::Results => "enter_results_mode",
                AppMode::Command => "enter_command_mode",
                AppMode::History => "enter_history_mode",
                _ => "switch_mode",
            };
            debug!(target: "shadow_state", "Setting mode via shadow state to {:?} with trigger {}", mode, trigger);
            self.shadow_state
                .borrow_mut()
                .set_mode(mode.clone(), buffer, trigger);
        } else {
            debug!(target: "shadow_state", "No buffer available for mode switch!");
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

            self.state_container
                .set_status_message(format!("History search: {} matches", match_count));
        }
    }

    /// Handle function key inputs (F1-F12)
    fn try_handle_function_keys(
        &mut self,
        key: &crossterm::event::KeyEvent,
    ) -> Result<Option<bool>> {
        match key.code {
            KeyCode::F(1) | KeyCode::Char('?') => {
                // Toggle between Help mode and previous mode
                if self.shadow_state.borrow().is_in_help_mode() {
                    // Exit help mode
                    let mode = if self.state_container.has_dataview() {
                        AppMode::Results
                    } else {
                        AppMode::Command
                    };
                    // Use proper mode synchronization
                    self.set_mode_via_shadow_state(mode, "exit_help");
                    self.state_container.set_help_visible(false);
                    self.help_widget.on_exit();
                } else {
                    // Enter help mode
                    self.state_container.set_help_visible(true);
                    // Use proper mode synchronization
                    self.set_mode_via_shadow_state(AppMode::Help, "help_requested");
                    self.help_widget.on_enter();
                }
                Ok(Some(false))
            }
            KeyCode::F(3) => {
                // Show pretty printed query
                self.show_pretty_query();
                Ok(Some(false))
            }
            KeyCode::F(5) => {
                // Toggle debug mode
                self.toggle_debug_mode();
                Ok(Some(false))
            }
            KeyCode::F(6) => {
                // Toggle row numbers
                let current = self.state_container.is_show_row_numbers();
                self.state_container.set_show_row_numbers(!current);
                self.state_container.set_status_message(format!(
                    "Row numbers: {}",
                    if !current { "ON" } else { "OFF" }
                ));
                Ok(Some(false))
            }
            KeyCode::F(7) => {
                // Toggle compact mode
                let current_mode = self.state_container.is_compact_mode();
                self.state_container.set_compact_mode(!current_mode);
                let message = if !current_mode {
                    "Compact mode enabled"
                } else {
                    "Compact mode disabled"
                };
                self.state_container.set_status_message(message.to_string());
                Ok(Some(false))
            }
            KeyCode::F(8) => {
                // Toggle case-insensitive string comparisons
                let current = self.state_container.is_case_insensitive();
                self.state_container.set_case_insensitive(!current);
                self.state_container.set_status_message(format!(
                    "Case-insensitive string comparisons: {}",
                    if !current { "ON" } else { "OFF" }
                ));
                Ok(Some(false))
            }
            KeyCode::F(9) => {
                // F9 as alternative for kill line (for terminals that intercept Ctrl+K)
                use crate::ui::traits::input_ops::InputBehavior;
                InputBehavior::kill_line(self);
                let message = if !self.state_container.is_kill_ring_empty() {
                    format!(
                        "Killed to end of line ('{}' saved to kill ring)",
                        self.state_container.get_kill_ring()
                    )
                } else {
                    "Killed to end of line".to_string()
                };
                self.state_container.set_status_message(message);
                Ok(Some(false))
            }
            KeyCode::F(10) => {
                // F10 as alternative for kill line backward
                use crate::ui::traits::input_ops::InputBehavior;
                InputBehavior::kill_line_backward(self);
                let message = if !self.state_container.is_kill_ring_empty() {
                    format!(
                        "Killed to beginning of line ('{}' saved to kill ring)",
                        self.state_container.get_kill_ring()
                    )
                } else {
                    "Killed to beginning of line".to_string()
                };
                self.state_container.set_status_message(message);
                Ok(Some(false))
            }
            KeyCode::F(12) => {
                // Toggle key press indicator
                let enabled = !self.key_indicator.enabled;
                self.key_indicator.set_enabled(enabled);
                self.key_sequence_renderer.set_enabled(enabled);
                self.state_container.set_status_message(format!(
                    "Key press indicator {}",
                    if enabled { "enabled" } else { "disabled" }
                ));
                Ok(Some(false))
            }
            _ => Ok(None), // Not a function key we handle
        }
    }

    /// Handle buffer management operations
    fn try_handle_buffer_operations(
        &mut self,
        key: &crossterm::event::KeyEvent,
    ) -> Result<Option<bool>> {
        if let Some(action) = self.key_dispatcher.get_command_action(key) {
            match action {
                "quit" => return Ok(Some(true)),
                "next_buffer" => {
                    // Save viewport state before switching
                    self.save_viewport_to_current_buffer();

                    let message = self
                        .buffer_handler
                        .next_buffer(self.state_container.buffers_mut());
                    debug!("{}", message);

                    // Sync all state after buffer switch
                    self.sync_after_buffer_switch();
                    return Ok(Some(false));
                }
                "previous_buffer" => {
                    // Save viewport state before switching
                    self.save_viewport_to_current_buffer();

                    let message = self
                        .buffer_handler
                        .previous_buffer(self.state_container.buffers_mut());
                    debug!("{}", message);

                    // Sync all state after buffer switch
                    self.sync_after_buffer_switch();
                    return Ok(Some(false));
                }
                "quick_switch_buffer" => {
                    // Save viewport state before switching
                    self.save_viewport_to_current_buffer();

                    let message = self
                        .buffer_handler
                        .quick_switch(self.state_container.buffers_mut());
                    debug!("{}", message);

                    // Sync all state after buffer switch
                    self.sync_after_buffer_switch();

                    return Ok(Some(false));
                }
                "new_buffer" => {
                    let message = self
                        .buffer_handler
                        .new_buffer(self.state_container.buffers_mut(), &self.config);
                    debug!("{}", message);
                    return Ok(Some(false));
                }
                "close_buffer" => {
                    let (success, message) = self
                        .buffer_handler
                        .close_buffer(self.state_container.buffers_mut());
                    debug!("{}", message);
                    return Ok(Some(!success)); // Exit if we couldn't close (only one left)
                }
                "list_buffers" => {
                    let buffer_list = self
                        .buffer_handler
                        .list_buffers(self.state_container.buffers());
                    for line in &buffer_list {
                        debug!("{}", line);
                    }
                    return Ok(Some(false));
                }
                action if action.starts_with("switch_to_buffer_") => {
                    if let Some(buffer_num_str) = action.strip_prefix("switch_to_buffer_") {
                        if let Ok(buffer_num) = buffer_num_str.parse::<usize>() {
                            // Save viewport state before switching
                            self.save_viewport_to_current_buffer();

                            let message = self.buffer_handler.switch_to_buffer(
                                self.state_container.buffers_mut(),
                                buffer_num - 1,
                            );
                            debug!("{}", message);

                            // Sync all state after buffer switch
                            self.sync_after_buffer_switch();
                        }
                    }
                    return Ok(Some(false));
                }
                _ => {} // Not a buffer operation
            }
        }
        Ok(None)
    }

    /// Handle history navigation operations
    fn try_handle_history_navigation(
        &mut self,
        key: &crossterm::event::KeyEvent,
    ) -> Result<Option<bool>> {
        // Handle Ctrl+R for history search
        if let KeyCode::Char('r') = key.code {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                // Start history search mode
                let current_input = self.get_input_text();

                // Start history search
                self.state_container.start_history_search(current_input);

                // Initialize with schema context
                self.update_history_matches_in_container();

                // Get status
                let match_count = self.state_container.history_search().matches.len();

                self.state_container.set_mode(AppMode::History);
                self.shadow_state
                    .borrow_mut()
                    .observe_mode_change(AppMode::History, "history_search_started");
                self.state_container.set_status_message(format!(
                    "History search started (Ctrl+R) - {} matches",
                    match_count
                ));
                return Ok(Some(false));
            }
        }

        // Handle Ctrl+P for previous history
        if let KeyCode::Char('p') = key.code {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                let history_entries = self
                    .state_container
                    .command_history()
                    .get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
                    if buffer.navigate_history_up(&history_commands) {
                        self.sync_all_input_states();
                        self.state_container
                            .set_status_message("Previous command from history".to_string());
                    }
                }
                return Ok(Some(false));
            }
        }

        // Handle Ctrl+N for next history
        if let KeyCode::Char('n') = key.code {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                let history_entries = self
                    .state_container
                    .command_history()
                    .get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
                    if buffer.navigate_history_down(&history_commands) {
                        self.sync_all_input_states();
                        self.state_container
                            .set_status_message("Next command from history".to_string());
                    }
                }
                return Ok(Some(false));
            }
        }

        // Handle Alt+Up/Down as alternatives
        match key.code {
            KeyCode::Up if key.modifiers.contains(KeyModifiers::ALT) => {
                let history_entries = self
                    .state_container
                    .command_history()
                    .get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
                    if buffer.navigate_history_up(&history_commands) {
                        self.sync_all_input_states();
                        self.state_container
                            .set_status_message("Previous command (Alt+Up)".to_string());
                    }
                }
                Ok(Some(false))
            }
            KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => {
                let history_entries = self
                    .state_container
                    .command_history()
                    .get_navigation_entries();
                let history_commands: Vec<String> =
                    history_entries.iter().map(|e| e.command.clone()).collect();

                if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
                    if buffer.navigate_history_down(&history_commands) {
                        self.sync_all_input_states();
                        self.state_container
                            .set_status_message("Next command (Alt+Down)".to_string());
                    }
                }
                Ok(Some(false))
            }
            _ => Ok(None),
        }
    }

    /// Handle text editing operations (word movement, kill line, clipboard, etc.)
    fn try_handle_text_editing(
        &mut self,
        key: &crossterm::event::KeyEvent,
    ) -> Result<Option<bool>> {
        // Try dispatcher actions first
        if let Some(action) = self.key_dispatcher.get_command_action(key) {
            match action {
                "expand_asterisk" => {
                    if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
                        if buffer.expand_asterisk(&self.hybrid_parser) {
                            // Sync for rendering if needed
                            if buffer.get_edit_mode() == EditMode::SingleLine {
                                let text = buffer.get_input_text();
                                let cursor = buffer.get_input_cursor_position();
                                self.set_input_text_with_cursor(text, cursor);
                            }
                        }
                    }
                    return Ok(Some(false));
                }
                "expand_asterisk_visible" => {
                    if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
                        if buffer.expand_asterisk_visible() {
                            // Sync for rendering if needed
                            if buffer.get_edit_mode() == EditMode::SingleLine {
                                let text = buffer.get_input_text();
                                let cursor = buffer.get_input_cursor_position();
                                self.set_input_text_with_cursor(text, cursor);
                            }
                        }
                    }
                    return Ok(Some(false));
                }
                // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
                "delete_word_backward" => {
                    use crate::ui::traits::input_ops::InputBehavior;
                    InputBehavior::delete_word_backward(self);
                    return Ok(Some(false));
                }
                // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
                "delete_word_forward" => {
                    use crate::ui::traits::input_ops::InputBehavior;
                    InputBehavior::delete_word_forward(self);
                    return Ok(Some(false));
                }
                // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
                "kill_line" => {
                    use crate::ui::traits::input_ops::InputBehavior;
                    InputBehavior::kill_line(self);
                    return Ok(Some(false));
                }
                // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
                "kill_line_backward" => {
                    use crate::ui::traits::input_ops::InputBehavior;
                    InputBehavior::kill_line_backward(self);
                    return Ok(Some(false));
                }
                // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
                "move_word_backward" => {
                    self.move_cursor_word_backward();
                    return Ok(Some(false));
                }
                // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
                "move_word_forward" => {
                    self.move_cursor_word_forward();
                    return Ok(Some(false));
                }
                // TODO: NOT IN COMMAND_EDITOR - Keep for Phase 4 (SQL navigation)
                "jump_to_prev_token" => {
                    self.jump_to_prev_token();
                    return Ok(Some(false));
                }
                // TODO: NOT IN COMMAND_EDITOR - Keep for Phase 4 (SQL navigation)
                "jump_to_next_token" => {
                    self.jump_to_next_token();
                    return Ok(Some(false));
                }
                // TODO: NOT IN COMMAND_EDITOR - Keep for Phase 3 (clipboard operations)
                "paste_from_clipboard" => {
                    self.paste_from_clipboard();
                    return Ok(Some(false));
                }
                _ => {} // Not a text editing action, fall through
            }
        }

        // Handle hardcoded text editing keys
        match key.code {
            // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Kill line - delete from cursor to end of line
                self.state_container
                    .set_status_message("Ctrl+K pressed - killing to end of line".to_string());
                use crate::ui::traits::input_ops::InputBehavior;
                InputBehavior::kill_line(self);
                Ok(Some(false))
            }
            // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Alternative: Alt+K for kill line (for terminals that intercept Ctrl+K)
                self.state_container
                    .set_status_message("Alt+K - killing to end of line".to_string());
                use crate::ui::traits::input_ops::InputBehavior;
                InputBehavior::kill_line(self);
                Ok(Some(false))
            }
            // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Kill line backward - delete from cursor to beginning of line
                use crate::ui::traits::input_ops::InputBehavior;
                InputBehavior::kill_line_backward(self);
                Ok(Some(false))
            }
            // TODO: NOT IN COMMAND_EDITOR - Keep for Phase 3 (clipboard operations)
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Yank - paste from kill ring
                self.yank();
                Ok(Some(false))
            }
            // TODO: NOT IN COMMAND_EDITOR - Keep for Phase 3 (clipboard operations)
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Paste from system clipboard
                self.paste_from_clipboard();
                Ok(Some(false))
            }
            // TODO: NOT IN COMMAND_EDITOR - Keep for Phase 4 (SQL navigation)
            KeyCode::Char('[') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Jump to previous SQL token
                self.jump_to_prev_token();
                Ok(Some(false))
            }
            // TODO: NOT IN COMMAND_EDITOR - Keep for Phase 4 (SQL navigation)
            KeyCode::Char(']') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Jump to next SQL token
                self.jump_to_next_token();
                Ok(Some(false))
            }
            // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
            KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Move backward one word
                self.move_cursor_word_backward();
                Ok(Some(false))
            }
            // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
            KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Move forward one word
                self.move_cursor_word_forward();
                Ok(Some(false))
            }
            // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move backward one word (alt+b like in bash)
                self.move_cursor_word_backward();
                Ok(Some(false))
            }
            // TODO: DUPLICATED IN COMMAND_EDITOR - Can be removed after full migration
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::ALT) => {
                // Move forward one word (alt+f like in bash)
                self.move_cursor_word_forward();
                Ok(Some(false))
            }
            _ => Ok(None), // Not a text editing key we handle
        }
    }

    /// Handle mode transitions and core input processing (Enter, Tab, Down arrow, input)
    fn try_handle_mode_transitions(
        &mut self,
        key: &crossterm::event::KeyEvent,
        old_cursor: usize,
    ) -> Result<Option<bool>> {
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+C - exit application
                Ok(Some(true))
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+D - exit application
                Ok(Some(true))
            }
            KeyCode::Enter => {
                // Query execution and special command handling
                let query = self.get_input_text().trim().to_string();
                debug!(target: "action", "Executing query: {}", query);

                if !query.is_empty() {
                    // Check for special commands
                    if query == ":help" {
                        self.state_container.set_help_visible(true);
                        // Use proper mode synchronization
                        self.set_mode_via_shadow_state(AppMode::Help, "help_requested");
                        self.state_container
                            .set_status_message("Help Mode - Press ESC to return".to_string());
                    } else if query == ":exit" || query == ":quit" || query == ":q" {
                        return Ok(Some(true));
                    } else if query == ":tui" {
                        // Already in TUI mode
                        self.state_container
                            .set_status_message("Already in TUI mode".to_string());
                    } else {
                        self.state_container
                            .set_status_message(format!("Processing query: '{}'", query));
                        self.execute_query_v2(&query)?;
                    }
                } else {
                    self.state_container
                        .set_status_message("Empty query - please enter a SQL command".to_string());
                }
                Ok(Some(false))
            }
            KeyCode::Tab => {
                // Tab completion works in both modes
                self.apply_completion();
                Ok(Some(false))
            }
            KeyCode::Down => {
                debug!(target: "shadow_state", "Down arrow pressed in Command mode. has_dataview={}, edit_mode={:?}",
                    self.state_container.has_dataview(),
                    self.state_container.get_edit_mode());

                if self.state_container.has_dataview()
                    && self.state_container.get_edit_mode() == Some(EditMode::SingleLine)
                {
                    debug!(target: "shadow_state", "Down arrow conditions met, switching to Results via set_mode");
                    // Switch to Results mode and restore state
                    self.state_container.set_mode(AppMode::Results);
                    self.shadow_state
                        .borrow_mut()
                        .observe_mode_change(AppMode::Results, "down_arrow_to_results");
                    // Restore previous position or default to 0
                    let row = self.state_container.get_last_results_row().unwrap_or(0);
                    self.state_container.set_table_selected_row(Some(row));

                    // Restore the exact scroll offset from when we left
                    let last_offset = self.state_container.get_last_scroll_offset();
                    self.state_container.set_scroll_offset(last_offset);
                    Ok(Some(false))
                } else {
                    debug!(target: "shadow_state", "Down arrow conditions not met, falling through");
                    // Fall through to default handling
                    Ok(None)
                }
            }
            _ => {
                // Fallback input handling and completion
                self.handle_input_key(*key);

                // Clear completion state when typing other characters
                self.state_container.clear_completion();

                // Always use single-line completion
                self.handle_completion();

                // Update horizontal scroll if cursor moved
                if self.get_input_cursor() != old_cursor {
                    self.update_horizontal_scroll(120); // Assume reasonable terminal width, will be adjusted in render
                }

                Ok(Some(false))
            }
        }
    }

    /// Handle navigation keys specific to Results mode
    fn try_handle_results_navigation(
        &mut self,
        key: &crossterm::event::KeyEvent,
    ) -> Result<Option<bool>> {
        match key.code {
            KeyCode::PageDown | KeyCode::Char('f')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                NavigationBehavior::page_down(self);
                Ok(Some(false))
            }
            KeyCode::PageUp | KeyCode::Char('b')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                NavigationBehavior::page_up(self);
                Ok(Some(false))
            }
            _ => Ok(None), // Not a navigation key we handle
        }
    }

    /// Handle clipboard/yank operations in Results mode
    fn try_handle_results_clipboard(
        &mut self,
        key: &crossterm::event::KeyEvent,
    ) -> Result<Option<bool>> {
        match key.code {
            KeyCode::Char('y') => {
                let selection_mode = self.get_selection_mode();
                debug!("'y' key pressed - selection_mode={:?}", selection_mode);
                match selection_mode {
                    SelectionMode::Cell => {
                        // In cell mode, single 'y' yanks the cell directly
                        debug!("Yanking cell in cell selection mode");
                        self.state_container
                            .set_status_message("Yanking cell...".to_string());
                        YankBehavior::yank_cell(self);
                        // Status message will be set by yank_cell
                    }
                    SelectionMode::Row => {
                        // In row mode, 'y' is handled by chord handler (yy, yc, ya)
                        // The chord handler will process the key sequence
                        debug!("'y' pressed in row mode - waiting for chord completion");
                        self.state_container.set_status_message(
                            "Press second key for chord: yy=row, yc=column, ya=all, yv=cell"
                                .to_string(),
                        );
                    }
                    SelectionMode::Column => {
                        // In column mode, 'y' yanks the current column
                        debug!("Yanking column in column selection mode");
                        self.state_container
                            .set_status_message("Yanking column...".to_string());
                        YankBehavior::yank_column(self);
                    }
                }
                Ok(Some(false))
            }
            _ => Ok(None),
        }
    }

    /// Handle export operations in Results mode
    fn try_handle_results_export(
        &mut self,
        key: &crossterm::event::KeyEvent,
    ) -> Result<Option<bool>> {
        match key.code {
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.export_to_csv();
                Ok(Some(false))
            }
            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.export_to_json();
                Ok(Some(false))
            }
            _ => Ok(None),
        }
    }

    /// Handle help and mode transitions in Results mode
    fn try_handle_results_help(
        &mut self,
        key: &crossterm::event::KeyEvent,
    ) -> Result<Option<bool>> {
        match key.code {
            KeyCode::F(1) | KeyCode::Char('?') => {
                self.state_container.set_help_visible(true);
                // Use proper mode synchronization
                self.set_mode_via_shadow_state(AppMode::Help, "help_requested");
                self.help_widget.on_enter();
                Ok(Some(false))
            }
            _ => Ok(None),
        }
    }

    fn handle_results_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Log all keys in Results mode to debug Escape handling
        debug!(
            "handle_results_input: Processing key {:?} in Results mode",
            key
        );

        // Check if vim search is active/navigating
        let is_vim_navigating = self.vim_search_adapter.borrow().is_navigating();
        let vim_is_active = self.vim_search_adapter.borrow().is_active();
        let has_search_pattern = !self.state_container.get_search_pattern().is_empty();

        debug!(
            "Search state check: vim_navigating={}, vim_active={}, has_pattern={}, pattern='{}'",
            is_vim_navigating,
            vim_is_active,
            has_search_pattern,
            self.state_container.get_search_pattern()
        );

        // If Escape is pressed and there's an active search or vim navigation, handle it properly
        if key.code == KeyCode::Esc {
            info!("ESCAPE KEY DETECTED in Results mode!");

            if is_vim_navigating || vim_is_active || has_search_pattern {
                info!("Escape pressed with active search - clearing via StateCoordinator");
                debug!(
                    "Pre-clear state: vim_navigating={}, vim_active={}, pattern='{}'",
                    is_vim_navigating,
                    vim_is_active,
                    self.state_container.get_search_pattern()
                );

                // Use StateCoordinator to handle ALL search cancellation logic
                use crate::ui::state::state_coordinator::StateCoordinator;
                StateCoordinator::cancel_search_with_refs(
                    &mut self.state_container,
                    &self.shadow_state,
                    Some(&self.vim_search_adapter),
                );

                // Verify clearing worked
                let post_pattern = self.state_container.get_search_pattern();
                let post_vim_active = self.vim_search_adapter.borrow().is_active();
                info!(
                    "Post-clear state: pattern='{}', vim_active={}",
                    post_pattern, post_vim_active
                );

                self.state_container
                    .set_status_message("Search cleared".to_string());
                return Ok(false);
            } else {
                info!("Escape pressed but no active search to clear");
            }
        }

        let selection_mode = self.state_container.get_selection_mode();

        debug!(
            "handle_results_input: key={:?}, selection_mode={:?}",
            key, selection_mode
        );

        // Normalize the key for platform differences
        let normalized = self.state_container.normalize_key(key);

        // Get the action that will be performed (if any) - for logging purposes
        let action_context = self.build_action_context();
        let mapped_action = self.key_mapper.map_key(normalized, &action_context);
        let action = mapped_action.as_ref().map(|a| format!("{:?}", a));

        // Log the key press
        if normalized != key {
            self.state_container
                .log_key_press(key, Some(format!("normalized to {:?}", normalized)));
        }
        self.state_container
            .log_key_press(normalized, action.clone());

        let normalized_key = normalized;

        // Try the new action system first
        // Note: Even if chord mode is active, single keys that aren't part of chords
        // should still be processed by the action system
        let action_context = self.build_action_context();
        debug!(
            "Action context for key {:?}: mode={:?}",
            normalized_key.code, action_context.mode
        );
        if let Some(action) = self
            .key_mapper
            .map_key(normalized_key.clone(), &action_context)
        {
            info!(
                "✓ Action system: key {:?} -> action {:?}",
                normalized_key.code, action
            );
            if let Ok(result) = self.try_handle_action(action, &action_context) {
                match result {
                    ActionResult::Handled => {
                        debug!("Action handled by new system");
                        return Ok(false);
                    }
                    ActionResult::Exit => {
                        debug!("Action requested exit");
                        return Ok(true);
                    }
                    ActionResult::SwitchMode(mode) => {
                        debug!("Action requested mode switch to {:?}", mode);
                        self.state_container.set_mode(mode);
                        return Ok(false);
                    }
                    ActionResult::Error(err) => {
                        warn!("Action error: {}", err);
                        self.state_container
                            .set_status_message(format!("Error: {}", err));
                        return Ok(false);
                    }
                    ActionResult::NotHandled => {
                        // Fall through to existing handling
                        debug!("Action not handled, falling back to legacy system");
                    }
                }
            }
        }

        // Debug uppercase G specifically
        if matches!(key.code, KeyCode::Char('G')) {
            debug!("Detected uppercase G key press!");
        }

        // F6 is now available for future use

        // F12 is now handled by the action system

        // NOTE: Chord handling has been moved to handle_input level
        // This ensures chords work correctly before any other key processing

        // All keys should now be handled through the action system above
        // Any keys that reach here are either:
        // 1. Not mapped in the action system yet
        // 2. Special cases that need direct handling

        // For now, log unmapped keys for debugging
        if mapped_action.is_none() {
            debug!(
                "No action mapping for key {:?} in Results mode",
                normalized_key
            );
        }

        // Try Results-specific navigation keys
        if let Some(result) = self.try_handle_results_navigation(&normalized_key)? {
            return Ok(result);
        }

        // Try clipboard/yank operations
        if let Some(result) = self.try_handle_results_clipboard(&normalized_key)? {
            return Ok(result);
        }

        // Try export operations
        if let Some(result) = self.try_handle_results_export(&normalized_key)? {
            return Ok(result);
        }

        // Try help and mode transitions
        if let Some(result) = self.try_handle_results_help(&normalized_key)? {
            return Ok(result);
        }

        // All key handling has been migrated to:
        // - Action system (handles most keys)
        // - try_handle_results_* methods (handles specific Results mode keys)
        // This completes the orchestration pattern for Results mode input
        Ok(false)
    }
    // ========== SEARCH OPERATIONS ==========

    fn execute_search_action(&mut self, mode: SearchMode, pattern: String) {
        debug!(target: "search", "execute_search_action called: mode={:?}, pattern='{}', current_app_mode={:?}, thread={:?}",
               mode, pattern, self.shadow_state.borrow().get_mode(), std::thread::current().id());
        match mode {
            SearchMode::Search => {
                debug!(target: "search", "Executing search with pattern: '{}', app_mode={:?}", pattern, self.shadow_state.borrow().get_mode());
                debug!(target: "search", "Search: current results count={}",
                       self.state_container.get_buffer_dataview().map(|v| v.source().row_count()).unwrap_or(0));

                // Set search pattern in AppStateContainer
                self.state_container.start_search(pattern.clone());

                self.state_container.set_search_pattern(pattern.clone());
                self.perform_search();
                let matches_count = self.state_container.search().matches.len();
                debug!(target: "search", "After perform_search, app_mode={:?}, matches_found={}",
                       self.shadow_state.borrow().get_mode(),
                       matches_count);

                // CRITICAL: Sync search results to VimSearchManager so 'n' and 'N' work
                if matches_count > 0 {
                    // Get matches from SearchManager to sync to VimSearchManager
                    let matches_for_vim: Vec<(usize, usize)> = {
                        let search_manager = self.search_manager.borrow();
                        search_manager
                            .all_matches()
                            .iter()
                            .map(|m| (m.row, m.column))
                            .collect()
                    };

                    // Sync to VimSearchManager
                    if let Some(dataview) = self.state_container.get_buffer_dataview() {
                        info!(target: "search", "Syncing {} matches to VimSearchManager for pattern '{}'",
                              matches_for_vim.len(), pattern);
                        self.vim_search_adapter
                            .borrow_mut()
                            .set_search_state_from_external(
                                pattern.clone(),
                                matches_for_vim,
                                dataview,
                            );
                    }
                }

                // Navigate to the first match if found (like vim)
                if matches_count > 0 {
                    // Get the first match position from SearchManager
                    let (row, col) = {
                        let search_manager = self.search_manager.borrow();
                        if let Some(first_match) = search_manager.first_match() {
                            (first_match.row, first_match.column)
                        } else {
                            // Fallback to state_container if SearchManager is empty (shouldn't happen)
                            let search_state = self.state_container.search();
                            if let Some((row, col, _, _)) = search_state.matches.first() {
                                (*row, *col)
                            } else {
                                (0, 0)
                            }
                        }
                    };

                    info!(target: "search", "NAVIGATION START: Moving to first match at data row={}, col={}", row, col);

                    // Navigate to the match position
                    // Set the row position
                    self.state_container.set_table_selected_row(Some(row));
                    self.state_container.set_selected_row(Some(row));
                    info!(target: "search", "  Set row position to {}", row);

                    // Set the column position
                    {
                        let mut nav = self.state_container.navigation_mut();
                        nav.selected_column = col;
                    }
                    self.state_container.set_current_column_buffer(col);
                    info!(target: "search", "  Set column position to {}", col);

                    // CRITICAL: Update TableWidgetManager for debounced search navigation
                    info!(target: "search", "Updating TableWidgetManager for debounced search to ({}, {})", row, col);
                    self.table_widget_manager
                        .borrow_mut()
                        .on_debounced_search(row, col);

                    // Update ViewportManager and ensure match is visible
                    {
                        let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
                        if let Some(ref mut viewport_manager) = *viewport_manager_borrow {
                            // Update the actual viewport to show the first match
                            let viewport_height = self.state_container.navigation().viewport_rows;
                            let viewport_width = self.state_container.navigation().viewport_columns;
                            let current_scroll = self.state_container.navigation().scroll_offset.0;

                            info!(target: "search", "  Viewport dimensions: {}x{}, current_scroll: {}",
                                  viewport_height, viewport_width, current_scroll);

                            // Calculate new scroll offset if needed to show the match
                            let new_row_offset = if row < current_scroll {
                                info!(target: "search", "  Match is above viewport, scrolling up to row {}", row);
                                row // Match is above, scroll up
                            } else if row >= current_scroll + viewport_height.saturating_sub(1) {
                                let centered = row.saturating_sub(viewport_height / 2);
                                info!(target: "search", "  Match is below viewport, centering at row {}", centered);
                                centered // Match is below, center it
                            } else {
                                info!(target: "search", "  Match is already visible, keeping scroll at {}", current_scroll);
                                current_scroll // Already visible
                            };

                            // Calculate column scroll if needed
                            let current_col_scroll =
                                self.state_container.navigation().scroll_offset.1;
                            let new_col_offset = if col < current_col_scroll {
                                info!(target: "search", "  Match column {} is left of viewport (scroll={}), scrolling left", col, current_col_scroll);
                                col // Match is to the left, scroll left
                            } else if col >= current_col_scroll + viewport_width.saturating_sub(1) {
                                let centered = col.saturating_sub(viewport_width / 4);
                                info!(target: "search", "  Match column {} is right of viewport (scroll={}, width={}), scrolling to {}",
                                      col, current_col_scroll, viewport_width, centered);
                                centered // Match is to the right, scroll right but keep some context
                            } else {
                                info!(target: "search", "  Match column {} is visible, keeping scroll at {}", col, current_col_scroll);
                                current_col_scroll // Already visible
                            };

                            // Update viewport to show the match (both row and column)
                            viewport_manager.set_viewport(
                                new_row_offset,
                                new_col_offset,
                                viewport_width as u16,
                                viewport_height as u16,
                            );
                            info!(target: "search", "  Set viewport to row_offset={}, col_offset={}", new_row_offset, new_col_offset);

                            // Set crosshair to match position (needs viewport-relative coordinates)
                            let crosshair_row = row - new_row_offset;
                            let crosshair_col = col - new_col_offset;
                            viewport_manager.set_crosshair(crosshair_row, crosshair_col);
                            info!(target: "search", "  Set crosshair to viewport-relative ({}, {}), absolute was ({}, {})", 
                                  crosshair_row, crosshair_col, row, col);

                            // Update navigation scroll offset (both row and column)
                            let mut nav = self.state_container.navigation_mut();
                            nav.scroll_offset.0 = new_row_offset;
                            nav.scroll_offset.1 = new_col_offset;
                        }
                    }

                    // Also update the buffer's current match to trigger UI updates
                    self.state_container.set_current_match(Some((row, col)));

                    // CRITICAL: Force the visual cursor position to update
                    // The crosshair is set but we need to ensure the visual cursor moves
                    {
                        let mut nav = self.state_container.navigation_mut();
                        nav.selected_row = row;
                        nav.selected_column = col;
                    }
                    info!(target: "search", "  Forced navigation state to row={}, col={}", row, col);

                    // Update status to show we're at match 1 of N
                    self.state_container.set_status_message(format!(
                        "Match 1/{} at row {}, col {}",
                        matches_count,
                        row + 1,
                        col + 1
                    ));
                }
            }
            SearchMode::Filter => {
                use crate::ui::state::state_coordinator::StateCoordinator;

                // Use StateCoordinator for all state transitions
                StateCoordinator::apply_filter_search_with_refs(
                    &mut self.state_container,
                    &self.shadow_state,
                    &pattern,
                );

                // Apply the actual filter (implementation stays in TUI)
                self.apply_filter(&pattern);

                debug!(target: "search", "After apply_filter, filtered_count={}",
                    self.state_container.get_buffer_dataview().map(|v| v.row_count()).unwrap_or(0));
            }
            SearchMode::FuzzyFilter => {
                use crate::ui::state::state_coordinator::StateCoordinator;

                // Use StateCoordinator for all state transitions
                StateCoordinator::apply_fuzzy_filter_search_with_refs(
                    &mut self.state_container,
                    &self.shadow_state,
                    &pattern,
                );

                // Apply the actual fuzzy filter (implementation stays in TUI)
                self.apply_fuzzy_filter();

                let indices_count = self.state_container.get_fuzzy_filter_indices().len();
                debug!(target: "search", "After apply_fuzzy_filter, matched_indices={}", indices_count);
            }
            SearchMode::ColumnSearch => {
                use crate::ui::state::state_coordinator::StateCoordinator;

                debug!(target: "search", "Executing column search with pattern: '{}'", pattern);

                // Use StateCoordinator for all state transitions
                StateCoordinator::apply_column_search_with_refs(
                    &mut self.state_container,
                    &self.shadow_state,
                    &pattern,
                );

                // Apply the actual column search (implementation stays in TUI)
                self.search_columns();

                debug!(target: "search", "After search_columns, app_mode={:?}", self.shadow_state.borrow().get_mode());
            }
        }
    }

    fn enter_search_mode(&mut self, mode: SearchMode) {
        debug!(target: "search", "enter_search_mode called for {:?}, current_mode={:?}, input_text='{}'",
               mode, self.shadow_state.borrow().get_mode(), self.state_container.get_input_text());

        // Get the SQL text based on the current mode
        let current_sql = if self.shadow_state.borrow().is_in_results_mode() {
            // In Results mode, use the last executed query
            let last_query = self.state_container.get_last_query();
            let input_text = self.state_container.get_input_text();
            debug!(target: "search", "COLUMN_SEARCH_SAVE_DEBUG: last_query='{}', input_text='{}'", last_query, input_text);

            if !last_query.is_empty() {
                debug!(target: "search", "Using last_query for search mode: '{}'", last_query);
                last_query
            } else if !input_text.is_empty() {
                // If last_query is empty but we have input_text, use that as fallback
                // This handles the case where data is loaded but no query has been executed yet
                debug!(target: "search", "No last_query, using input_text as fallback: '{}'", input_text);
                input_text
            } else {
                // This shouldn't happen if we're properly saving queries
                warn!(target: "search", "No last_query or input_text found when entering search mode from Results!");
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

        // Set the app mode - use sync_mode to ensure all state is synchronized
        debug!(target: "mode", "Setting app mode from {:?} to {:?}", self.shadow_state.borrow().get_mode(), mode.to_app_mode());
        let trigger = match mode {
            SearchMode::ColumnSearch => "backslash_column_search",
            SearchMode::Search => "data_search_started",
            SearchMode::FuzzyFilter => "fuzzy_filter_started",
            SearchMode::Filter => "filter_started",
        };
        self.sync_mode(mode.to_app_mode(), trigger);

        // Also observe the search mode start in shadow state for search-specific tracking
        let search_type = match mode {
            SearchMode::ColumnSearch => crate::ui::state::shadow_state::SearchType::Column,
            SearchMode::Search => crate::ui::state::shadow_state::SearchType::Data,
            SearchMode::FuzzyFilter | SearchMode::Filter => {
                crate::ui::state::shadow_state::SearchType::Fuzzy
            }
        };
        self.shadow_state
            .borrow_mut()
            .observe_search_start(search_type, trigger);

        // Clear patterns
        match mode {
            SearchMode::Search => {
                // Clear search in AppStateContainer
                self.state_container.clear_search();
                self.state_container.set_search_pattern(String::new());
            }
            SearchMode::Filter => {
                self.state_container.set_filter_pattern(String::new());
                self.state_container.filter_mut().clear();
            }
            SearchMode::FuzzyFilter => {
                self.state_container.set_fuzzy_filter_pattern(String::new());
                self.state_container.set_fuzzy_filter_indices(Vec::new());
                self.state_container.set_fuzzy_filter_active(false);
            }
            SearchMode::ColumnSearch => {
                // Clear column search in both AppStateContainer and DataView
                self.state_container.clear_column_search();
                if let Some(dataview) = self.state_container.get_buffer_dataview_mut() {
                    dataview.clear_column_search();
                }

                // All column search state is now managed by AppStateContainer
            }
        }

        // Clear input field for search mode use
        self.input = tui_input::Input::default();
        // Note: Not syncing here as search modes use input differently
    }

    fn handle_search_modes_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Safety check: Always allow Ctrl-C to exit regardless of state
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(true); // Signal to quit
        }

        // All search modes now use the same SearchModesWidget for consistent debouncing

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
                        self.state_container.set_search_pattern(pattern);
                    }
                    SearchMode::Filter => {
                        self.state_container.set_filter_pattern(pattern.clone());
                        let mut filter = self.state_container.filter_mut();
                        filter.pattern = pattern.clone();
                        filter.is_active = true;
                    }
                    SearchMode::FuzzyFilter => {
                        self.state_container.set_fuzzy_filter_pattern(pattern);
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
                        // Use execute_search_action to get the navigation to first match
                        self.execute_search_action(SearchMode::Search, pattern);
                        debug!(target: "search", "Search Apply: last_query='{}', will restore saved SQL from widget", self.state_container.get_last_query());
                        // For search, we always want to exit to Results mode after applying
                    }
                    SearchMode::Filter => {
                        debug!(target: "search", "Filter Apply: Applying filter with pattern '{}'", pattern);
                        self.state_container.set_filter_pattern(pattern.clone());
                        {
                            let mut filter = self.state_container.filter_mut();
                            filter.pattern = pattern.clone();
                            filter.is_active = true;
                        } // filter borrow ends here
                        self.apply_filter(&pattern); // Use the actual pattern, not empty string
                        debug!(target: "search", "Filter Apply: last_query='{}', will restore saved SQL from widget", self.state_container.get_last_query());
                    }
                    SearchMode::FuzzyFilter => {
                        debug!(target: "search", "FuzzyFilter Apply: Applying filter with pattern '{}'", pattern);
                        self.state_container.set_fuzzy_filter_pattern(pattern);
                        self.apply_fuzzy_filter();
                        debug!(target: "search", "FuzzyFilter Apply: last_query='{}', will restore saved SQL from widget", self.state_container.get_last_query());
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
                            self.state_container.set_current_column_buffer(col_idx);

                            // Update ViewportManager to ensure the column is visible
                            let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
                            if let Some(viewport_manager) = viewport_manager_borrow.as_mut() {
                                viewport_manager.set_current_column(col_idx);
                            }
                            drop(viewport_manager_borrow);

                            // CRITICAL: Sync NavigationState with ViewportManager after column navigation
                            // This ensures all state systems are consistent (like vim search)
                            debug!(target: "column_search_sync", "ColumnSearch Apply: About to call sync_navigation_with_viewport() for column: {}", col_name);
                            debug!(target: "column_search_sync", "ColumnSearch Apply: Pre-sync - viewport current_column: {}", 
                                if let Some(vm) = self.viewport_manager.try_borrow().ok() {
                                    vm.as_ref().map(|v| v.get_crosshair_col()).unwrap_or(0)
                                } else { 0 });
                            self.sync_navigation_with_viewport();
                            debug!(target: "column_search_sync", "ColumnSearch Apply: Post-sync - navigation current_column: {}", 
                                self.state_container.navigation().selected_column);
                            debug!(target: "column_search_sync", "ColumnSearch Apply: sync_navigation_with_viewport() completed for column: {}", col_name);

                            self.state_container
                                .set_status_message(format!("Jumped to column: {}", col_name));
                        }

                        // IMPORTANT: Don't modify input_text when exiting column search!
                        // The widget will restore the original SQL that was saved when entering the mode
                        debug!(target: "search", "ColumnSearch Apply: Exiting without modifying input_text");
                        debug!(target: "search", "ColumnSearch Apply: last_query='{}', will restore saved SQL from widget", self.state_container.get_last_query());
                        // Note: Column search state will be cleared by cancel_search_with_refs below
                    }
                }

                // Exit search mode and return to Results
                // Try to get saved SQL from widget
                let saved_state = self.search_modes_widget.exit_mode();

                if let Some((sql, cursor)) = saved_state {
                    debug!(target: "search", "Exiting search mode. Original SQL was: '{}', cursor: {}", sql, cursor);
                    debug!(target: "buffer", "Returning to Results mode, preserving last_query: '{}'",
                           self.state_container.get_last_query());

                    // IMPORTANT: Always restore the saved SQL to input_text!
                    // This includes empty strings - we need to clear the search term
                    debug!(target: "search", "Restoring saved SQL to input_text: '{}'", sql);
                    // Use helper to sync all states
                    self.set_input_text_with_cursor(sql, cursor);
                } else {
                    // Widget didn't have saved state - restore appropriate SQL based on mode
                    if mode == SearchMode::ColumnSearch {
                        // For column search, restore the last executed query or pre-populated query
                        let last_query = self.state_container.get_last_query();
                        if !last_query.is_empty() {
                            debug!(target: "search", "Column search: No saved state, restoring last_query: '{}'", last_query);
                            self.set_input_text(last_query);
                        } else {
                            debug!(target: "search", "Column search: No saved state or last_query, clearing input");
                            self.set_input_text(String::new());
                        }
                    } else {
                        debug!(target: "search", "No saved state from widget, keeping current SQL");
                    }
                }

                // ALWAYS switch back to Results mode after Apply for all search modes
                use crate::ui::state::state_coordinator::StateCoordinator;

                // For column search, we cancel completely (no n/N navigation)
                // For regular search, we complete but keep pattern for n/N
                if mode == SearchMode::ColumnSearch {
                    debug!(target: "column_search_sync", "ColumnSearch Apply: Canceling column search completely with cancel_search_with_refs()");

                    // Also clear column search in DataView
                    if let Some(dataview) = self.state_container.get_buffer_dataview_mut() {
                        dataview.clear_column_search();
                        debug!(target: "column_search_sync", "ColumnSearch Apply: Cleared column search in DataView");
                    }

                    StateCoordinator::cancel_search_with_refs(
                        &mut self.state_container,
                        &self.shadow_state,
                        Some(&self.vim_search_adapter),
                    );
                    // Note: cancel_search_with_refs already switches to Results mode
                    debug!(target: "column_search_sync", "ColumnSearch Apply: Column search canceled and mode switched to Results");
                } else {
                    // For regular search modes, keep pattern for n/N navigation
                    debug!(target: "column_search_sync", "Search Apply: About to call StateCoordinator::complete_search_with_refs() for mode: {:?}", mode);
                    StateCoordinator::complete_search_with_refs(
                        &mut self.state_container,
                        &self.shadow_state,
                        Some(&self.vim_search_adapter),
                        AppMode::Results,
                        "search_applied",
                    );
                    debug!(target: "column_search_sync", "Search Apply: StateCoordinator::complete_search_with_refs() completed - should now be in Results mode");
                }

                // Show status message
                let filter_msg = match mode {
                    SearchMode::FuzzyFilter => {
                        let query = self.state_container.get_last_query();
                        format!(
                            "Fuzzy filter applied. Query: '{}'. Press 'f' again to modify.",
                            if query.len() > 30 {
                                format!("{}...", &query[..30])
                            } else {
                                query
                            }
                        )
                    }
                    SearchMode::Filter => "Filter applied. Press 'F' again to modify.".to_string(),
                    SearchMode::Search => {
                        let matches = self.state_container.search().matches.len();
                        if matches > 0 {
                            format!("Found {} matches. Use n/N to navigate.", matches)
                        } else {
                            "No matches found.".to_string()
                        }
                    }
                    SearchMode::ColumnSearch => "Column search complete.".to_string(),
                };
                self.state_container.set_status_message(filter_msg);
            }
            SearchModesAction::Cancel => {
                // Clear the filter and restore original SQL
                let mode = self.shadow_state.borrow().get_mode();
                match mode {
                    AppMode::FuzzyFilter => {
                        // Clear fuzzy filter - must apply empty filter to DataView
                        debug!(target: "search", "FuzzyFilter Cancel: Clearing fuzzy filter");
                        self.state_container.set_fuzzy_filter_pattern(String::new());
                        self.apply_fuzzy_filter(); // This will clear the filter in DataView
                        self.state_container.set_fuzzy_filter_indices(Vec::new());
                        self.state_container.set_fuzzy_filter_active(false);
                    }
                    AppMode::Filter => {
                        // Clear both local and buffer filter state
                        debug!(target: "search", "Filter Cancel: Clearing filter pattern and state");
                        self.state_container.filter_mut().clear();
                        self.state_container.set_filter_pattern(String::new());
                        self.state_container.set_filter_active(false);
                        // Re-apply empty filter to restore all results
                        self.apply_filter("");
                    }
                    AppMode::ColumnSearch => {
                        // Clear column search state using AppStateContainer
                        self.state_container.clear_column_search();
                        // The widget will restore the original SQL that was saved when entering the mode
                        debug!(target: "search", "ColumnSearch Cancel: Exiting without modifying input_text");
                        debug!(target: "search", "ColumnSearch Cancel: last_query='{}', will restore saved SQL from widget", self.state_container.get_last_query());
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

                // Use StateCoordinator to properly cancel search and restore state
                // StateCoordinator handles clearing vim search adapter too
                use crate::ui::state::state_coordinator::StateCoordinator;
                StateCoordinator::cancel_search_with_refs(
                    &mut self.state_container,
                    &self.shadow_state,
                    Some(&self.vim_search_adapter),
                );
            }
            SearchModesAction::NextMatch => {
                debug!(target: "search", "NextMatch action, current_mode={:?}, widget_mode={:?}",
                       self.shadow_state.borrow().get_mode(), self.search_modes_widget.current_mode());

                // Check both shadow state and widget mode for consistency
                if self.shadow_state.borrow().is_in_column_search()
                    || self.search_modes_widget.current_mode() == Some(SearchMode::ColumnSearch)
                {
                    debug!(target: "search", "Calling next_column_match");
                    // Ensure mode is correctly set
                    if !self.shadow_state.borrow().is_in_column_search() {
                        debug!(target: "search", "WARNING: Mode mismatch - fixing");
                        self.state_container.set_mode(AppMode::ColumnSearch);
                        self.shadow_state.borrow_mut().observe_search_start(
                            crate::ui::state::shadow_state::SearchType::Column,
                            "column_search_mode_fix_next",
                        );
                    }
                    self.next_column_match();
                } else {
                    debug!(target: "search", "Not in ColumnSearch mode, skipping next_column_match");
                }
            }
            SearchModesAction::PreviousMatch => {
                debug!(target: "search", "PreviousMatch action, current_mode={:?}, widget_mode={:?}",
                       self.shadow_state.borrow().get_mode(), self.search_modes_widget.current_mode());

                // Check both buffer mode and widget mode for consistency
                if self.shadow_state.borrow().get_mode() == AppMode::ColumnSearch
                    || self.search_modes_widget.current_mode() == Some(SearchMode::ColumnSearch)
                {
                    debug!(target: "search", "Calling previous_column_match");
                    // Ensure mode is correctly set
                    if self.shadow_state.borrow().get_mode() != AppMode::ColumnSearch {
                        debug!(target: "search", "WARNING: Mode mismatch - fixing");
                        self.state_container.set_mode(AppMode::ColumnSearch);
                        self.shadow_state.borrow_mut().observe_search_start(
                            crate::ui::state::shadow_state::SearchType::Column,
                            "column_search_mode_fix_prev",
                        );
                    }
                    self.previous_column_match();
                } else {
                    debug!(target: "search", "Not in ColumnSearch mode, skipping previous_column_match");
                }
            }
            SearchModesAction::PassThrough => {}
        }

        // ========== FILTER OPERATIONS ==========

        Ok(false)
    }

    fn handle_help_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Handle help input directly to avoid borrow conflicts
        let result = match key.code {
            crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('q') => {
                self.help_widget.on_exit();
                self.state_container.set_help_visible(false);

                // Return to Results mode if we have data, otherwise Command mode
                let target_mode = if self.state_container.has_dataview() {
                    AppMode::Results
                } else {
                    AppMode::Command
                };

                // Use proper mode synchronization
                self.set_mode_via_shadow_state(target_mode, "escape_from_help");

                // Return false to stay in the TUI (not exit)
                Ok(false)
            }
            _ => {
                // Delegate other keys to help widget
                self.help_widget.handle_key(key);
                Ok(false)
            }
        };

        result
    }

    // ========== HELP NAVIGATION ==========

    fn handle_history_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Handle history input directly to avoid borrow conflicts
        use crossterm::event::{KeyCode, KeyModifiers};

        let result = match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                crate::ui::input::history_input_handler::HistoryInputResult::Exit
            }
            KeyCode::Esc => {
                // Cancel history search and restore original input
                let original_input = self.state_container.cancel_history_search();
                if let Some(buffer) = self.state_container.current_buffer_mut() {
                    self.shadow_state.borrow_mut().set_mode(
                        crate::buffer::AppMode::Command,
                        buffer,
                        "history_cancelled",
                    );
                    buffer.set_status_message("History search cancelled".to_string());
                }
                crate::ui::input::history_input_handler::HistoryInputResult::SwitchToCommand(Some(
                    (original_input, 0),
                ))
            }
            KeyCode::Enter => {
                // Accept the selected history command
                if let Some(command) = self.state_container.accept_history_search() {
                    if let Some(buffer) = self.state_container.current_buffer_mut() {
                        self.shadow_state.borrow_mut().set_mode(
                            crate::buffer::AppMode::Command,
                            buffer,
                            "history_accepted",
                        );
                        buffer.set_status_message(
                            "Command loaded from history (cursor at start)".to_string(),
                        );
                    }
                    // Return command with cursor at the beginning for better visibility
                    crate::ui::input::history_input_handler::HistoryInputResult::SwitchToCommand(
                        Some((command, 0)),
                    )
                } else {
                    crate::ui::input::history_input_handler::HistoryInputResult::Continue
                }
            }
            KeyCode::Up => {
                self.state_container.history_search_previous();
                crate::ui::input::history_input_handler::HistoryInputResult::Continue
            }
            KeyCode::Down => {
                self.state_container.history_search_next();
                crate::ui::input::history_input_handler::HistoryInputResult::Continue
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+R cycles through matches
                self.state_container.history_search_next();
                crate::ui::input::history_input_handler::HistoryInputResult::Continue
            }
            KeyCode::Backspace => {
                self.state_container.history_search_backspace();
                crate::ui::input::history_input_handler::HistoryInputResult::Continue
            }
            KeyCode::Char(c) => {
                self.state_container.history_search_add_char(c);
                crate::ui::input::history_input_handler::HistoryInputResult::Continue
            }
            _ => crate::ui::input::history_input_handler::HistoryInputResult::Continue,
        };

        // Handle the result
        match result {
            crate::ui::input::history_input_handler::HistoryInputResult::Exit => return Ok(true),
            crate::ui::input::history_input_handler::HistoryInputResult::SwitchToCommand(
                input_data,
            ) => {
                if let Some((text, cursor_pos)) = input_data {
                    self.set_input_text_with_cursor(text, cursor_pos);
                    // Sync to ensure scroll is reset properly
                    self.sync_all_input_states();
                }
            }
            crate::ui::input::history_input_handler::HistoryInputResult::Continue => {
                // Update history matches if needed
                if crate::ui::input::history_input_handler::key_updates_search(key) {
                    self.update_history_matches_in_container();
                }
            }
        }

        Ok(false)
    }

    /// Update history matches in the AppStateContainer with schema context
    fn update_history_matches_in_container(&mut self) {
        // Get current schema columns and data source for better matching
        let (current_columns, current_source_str) =
            if let Some(dataview) = self.state_container.get_buffer_dataview() {
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
        // Create context and delegate to extracted handler
        let mut ctx = crate::ui::input::input_handlers::DebugInputContext {
            buffer_manager: self.state_container.buffers_mut(),
            debug_widget: &mut self.debug_widget,
            shadow_state: &self.shadow_state,
        };

        let should_quit = crate::ui::input::input_handlers::handle_debug_input(&mut ctx, key)?;

        // If the extracted handler didn't handle these special keys, we still do them here
        // (until we can extract yank operations too)
        if !should_quit {
            match key.code {
                KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Ctrl+T: "Yank as Test" - capture current session as test case
                    self.yank_as_test_case();
                }
                KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    // Shift+Y: Yank debug dump with context
                    self.yank_debug_with_context();
                }
                _ => {}
            }
        }

        Ok(should_quit)
        // ========== QUERY OPERATIONS ==========
    }

    fn handle_pretty_query_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Create context and delegate to extracted handler
        let mut ctx = crate::ui::input::input_handlers::DebugInputContext {
            buffer_manager: self.state_container.buffers_mut(),
            debug_widget: &mut self.debug_widget,
            shadow_state: &self.shadow_state,
        };

        crate::ui::input::input_handlers::handle_pretty_query_input(&mut ctx, key)
    }

    pub fn execute_query_v2(&mut self, query: &str) -> Result<()> {
        // Use orchestrator to handle all the query execution logic
        let context = self.query_orchestrator.execute_query(
            query,
            &mut self.state_container,
            &self.vim_search_adapter,
        );

        match context {
            Ok(ctx) => {
                // Apply the new DataView
                self.state_container
                    .set_dataview(Some(ctx.result.dataview.clone()));

                // Update viewport
                self.update_viewport_manager(Some(ctx.result.dataview.clone()));

                // Update navigation state
                self.state_container
                    .update_data_size(ctx.result.stats.row_count, ctx.result.stats.column_count);

                // Calculate column widths
                self.calculate_optimal_column_widths();

                // Update status message
                self.state_container
                    .set_status_message(ctx.result.status_message());

                // Add to history
                self.state_container
                    .command_history_mut()
                    .add_entry_with_schema(
                        ctx.query.clone(),
                        true,
                        Some(ctx.result.stats.execution_time.as_millis() as u64),
                        ctx.result.column_names(),
                        Some(ctx.result.table_name()),
                    )?;

                // Switch to results mode - use sync_mode to ensure all state is synchronized
                self.sync_mode(AppMode::Results, "execute_query_success");

                // Reset table
                self.reset_table_state();

                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Query error: {}", e);
                self.state_container.set_status_message(error_msg.clone());

                // Add to history as failed
                self.state_container
                    .command_history_mut()
                    .add_entry_with_schema(query.to_string(), false, None, vec![], None)?;

                Err(e)
            }
        }
    }

    fn handle_completion(&mut self) {
        let cursor_pos = self.get_input_cursor();
        let query_str = self.get_input_text();
        let query = query_str.as_str();

        let hybrid_result = self.hybrid_parser.get_completions(query, cursor_pos);
        if !hybrid_result.suggestions.is_empty() {
            self.state_container.set_status_message(format!(
                "Suggestions: {}",
                hybrid_result.suggestions.join(", ")
            ));
        }
    }

    fn apply_completion(&mut self) {
        let cursor_pos = self.get_input_cursor();
        let query = self.get_input_text();

        // Get the current completion suggestion
        let suggestion = match self.get_or_refresh_completion(&query, cursor_pos) {
            Some(s) => s,
            None => return,
        };

        // Apply the completion to the text
        self.apply_completion_to_input(&query, cursor_pos, &suggestion);
    }

    /// Get current completion or refresh if context changed
    /// Returns None if no completions available
    fn get_or_refresh_completion(&mut self, query: &str, cursor_pos: usize) -> Option<String> {
        let is_same_context = self
            .state_container
            .is_same_completion_context(query, cursor_pos);

        if !is_same_context {
            // New completion context - get fresh suggestions
            let hybrid_result = self.hybrid_parser.get_completions(query, cursor_pos);
            if hybrid_result.suggestions.is_empty() {
                self.state_container
                    .set_status_message("No completions available".to_string());
                return None;
            }

            self.state_container
                .set_completion_suggestions(hybrid_result.suggestions);
        } else if self.state_container.is_completion_active() {
            // Cycle to next suggestion
            self.state_container.next_completion();
        } else {
            self.state_container
                .set_status_message("No completions available".to_string());
            return None;
        }

        // Get the current suggestion from AppStateContainer
        match self.state_container.get_current_completion() {
            Some(sugg) => Some(sugg),
            None => {
                self.state_container
                    .set_status_message("No completion selected".to_string());
                None
            }
        }
    }

    /// Apply a completion suggestion to the input
    fn apply_completion_to_input(&mut self, query: &str, cursor_pos: usize, suggestion: &str) {
        let partial_word =
            crate::ui::utils::text_operations::extract_partial_word_at_cursor(query, cursor_pos);

        if let Some(partial) = partial_word {
            self.apply_partial_completion(query, cursor_pos, &partial, suggestion);
        } else {
            self.apply_full_insertion(query, cursor_pos, suggestion);
        }
    }

    /// Apply completion when we have a partial word to complete
    fn apply_partial_completion(
        &mut self,
        query: &str,
        cursor_pos: usize,
        partial: &str,
        suggestion: &str,
    ) {
        // Use extracted completion logic
        let result = crate::ui::utils::text_operations::apply_completion_to_text(
            query, cursor_pos, partial, suggestion,
        );

        // Use helper to set text and cursor together - this ensures sync
        self.set_input_text_with_cursor(result.new_text.clone(), result.new_cursor_position);

        // Update completion state for next tab press
        self.state_container
            .update_completion_context(result.new_text.clone(), result.new_cursor_position);

        // Generate status message
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
        self.state_container.set_status_message(suggestion_info);
    }

    /// Apply completion as a full insertion at cursor position
    fn apply_full_insertion(&mut self, query: &str, cursor_pos: usize, suggestion: &str) {
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
        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            buffer.set_input_cursor_position(cursor_pos_new);
            // Sync all input states after undo/redo
            self.sync_all_input_states();
        }

        // Update completion state
        self.state_container
            .update_completion_context(new_query, cursor_pos_new);

        self.state_container
            .set_status_message(format!("Inserted: {}", suggestion));
    }

    // Note: expand_asterisk and get_table_columns removed - moved to Buffer and use hybrid_parser directly

    // ========== COLUMN INFO ==========

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
    /// Get column names using DataProvider trait
    /// Part of the migration to trait-based data access
    fn get_column_names_via_provider(&self) -> Vec<String> {
        if let Some(provider) = self.get_data_provider() {
            provider.get_column_names()
        } else {
            Vec::new()
        }
    }

    // ========== NAVIGATION METHODS ==========

    // ========== COLUMN PIN/HIDE ==========

    fn toggle_column_pin_impl(&mut self) {
        // Get visual column index from ViewportManager's crosshair
        let visual_col_idx = if let Some(ref viewport_manager) = *self.viewport_manager.borrow() {
            viewport_manager.get_crosshair_col()
        } else {
            0
        };

        // Get column name at visual position
        let column_name = if let Some(dataview) = self.state_container.get_buffer_dataview() {
            let all_columns = dataview.column_names();
            all_columns.get(visual_col_idx).cloned()
        } else {
            None
        };

        if let Some(col_name) = column_name {
            if let Some(dataview) = self.state_container.get_buffer_dataview_mut() {
                // Check if this column name is already pinned
                let pinned_names = dataview.get_pinned_column_names();
                if pinned_names.contains(&col_name) {
                    // Column is already pinned, unpin it
                    dataview.unpin_column_by_name(&col_name);
                    self.state_container
                        .set_status_message(format!("Column '{}' unpinned", col_name));
                } else {
                    // Try to pin the column by name
                    match dataview.pin_column_by_name(&col_name) {
                        Ok(_) => {
                            self.state_container
                                .set_status_message(format!("Column '{}' pinned [P]", col_name));
                        }
                        Err(e) => {
                            self.state_container.set_status_message(e.to_string());
                        }
                    }
                }

                // Update ViewportManager with the modified DataView
                if let Some(updated_dataview) = self.state_container.get_buffer_dataview() {
                    self.update_viewport_manager(Some(updated_dataview.clone()));
                }
            }
        } else {
            self.state_container
                .set_status_message("No column to pin at current position".to_string());
        }
    }

    fn clear_all_pinned_columns_impl(&mut self) {
        if let Some(dataview) = self.state_container.get_buffer_dataview_mut() {
            dataview.clear_pinned_columns();
        }
        self.state_container
            .set_status_message("All columns unpinned".to_string());

        // Update ViewportManager with the modified DataView
        if let Some(updated_dataview) = self.state_container.get_buffer_dataview() {
            self.update_viewport_manager(Some(updated_dataview.clone()));
        }
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

            let current_column = self.state_container.get_current_column();
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

        self.state_container.set_column_stats(Some(stats));

        // Show timing in status message
        self.state_container.set_status_message(format!(
            "Column stats: {:.1}ms for {} values ({} unique)",
            elapsed.as_secs_f64() * 1000.0,
            data_to_analyze.len(),
            analyzer_stats.unique_values
        ));

        self.state_container.set_mode(AppMode::ColumnStats);
        self.shadow_state
            .borrow_mut()
            .observe_mode_change(AppMode::ColumnStats, "column_stats_requested");
    }

    fn check_parser_error(&self, query: &str) -> Option<String> {
        crate::ui::operations::simple_operations::check_parser_error(query)
    }

    fn update_viewport_size(&mut self) {
        // Update the stored viewport size based on current terminal size
        if let Ok((width, height)) = crossterm::terminal::size() {
            // Calculate the actual data area height
            let data_rows_available = Self::calculate_available_data_rows(height);

            // Let ViewportManager handle the calculations
            let visible_rows = {
                let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
                let viewport_manager = viewport_manager_borrow
                    .as_mut()
                    .expect("ViewportManager must exist for viewport size update");
                viewport_manager.update_terminal_size(width, data_rows_available)
            };

            // Update buffer's last_visible_rows
            self.state_container.set_last_visible_rows(visible_rows);

            // Update NavigationState's viewport dimensions
            self.state_container
                .navigation_mut()
                .set_viewport_size(visible_rows, width as usize);

            info!(target: "navigation", "update_viewport_size - viewport set to: {}x{} rows", visible_rows, width);
        }
    }

    // ========== SEARCH EXECUTION ==========

    // Search and filter functions
    fn perform_search(&mut self) {
        if let Some(dataview) = self.get_current_data() {
            // Convert DataView rows to Vec<Vec<String>> for SearchManager
            let data: Vec<Vec<String>> = (0..dataview.row_count())
                .filter_map(|i| dataview.get_row(i))
                .map(|row| row.values.iter().map(|v| v.to_string()).collect())
                .collect();

            // Get search pattern from buffer
            let pattern = self.state_container.get_search_pattern().to_string();

            info!(target: "search", "=== SEARCH START ===");
            info!(target: "search", "Pattern: '{}', case_insensitive: {}",
                  pattern, self.state_container.is_case_insensitive());
            info!(target: "search", "Data dimensions: {} rows x {} columns",
                  data.len(), data.first().map(|r| r.len()).unwrap_or(0));

            // Log column names to understand ordering
            let column_names = dataview.column_names();
            info!(target: "search", "Column names (first 5): {:?}",
                  column_names.iter().take(5).collect::<Vec<_>>());

            // Log the first few rows of data we're searching
            for (i, row) in data.iter().take(10).enumerate() {
                info!(target: "search", "  Data row {}: [{}]", i,
                      row.iter().take(5).map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "));
            }

            // Get visible columns if needed (for now search all columns)
            let visible_columns = None;

            // Perform search using SearchManager
            let match_count = {
                let mut search_manager = self.search_manager.borrow_mut();

                // IMPORTANT: Clear any previous search results first
                search_manager.clear();
                info!(target: "search", "Cleared previous search results");

                // Update case sensitivity based on current setting
                search_manager.set_case_sensitive(!self.state_container.is_case_insensitive());
                info!(target: "search", "Set case_sensitive to {}", !self.state_container.is_case_insensitive());

                // Perform the search
                let count = search_manager.search(&pattern, &data, visible_columns);
                info!(target: "search", "SearchManager.search() returned {} matches", count);
                count
            };

            info!(target: "search", "SearchManager found {} matches", match_count);

            // Process the matches
            if match_count > 0 {
                // Get first match for navigation and log details
                let (first_row, first_col) = {
                    let search_manager = self.search_manager.borrow();
                    if let Some(first_match) = search_manager.first_match() {
                        info!(target: "search", "FIRST MATCH DETAILS:");
                        info!(target: "search", "  Data coordinates: row={}, col={}",
                              first_match.row, first_match.column);
                        info!(target: "search", "  Matched value: '{}'", first_match.value);
                        info!(target: "search", "  Highlight range: {:?}", first_match.highlight_range);

                        // Log first 5 matches for debugging
                        for (i, m) in search_manager.all_matches().iter().take(5).enumerate() {
                            info!(target: "search", "  Match #{}: row={}, col={}, value='{}'",
                                  i + 1, m.row, m.column, m.value);
                        }

                        (first_match.row, first_match.column)
                    } else {
                        warn!(target: "search", "SearchManager reported matches but first_match() is None!");
                        (0, 0)
                    }
                };

                // Update state container and buffer with matches
                self.state_container.set_table_selected_row(Some(first_row));

                // CRITICAL: Update TableWidgetManager to trigger re-render
                info!(target: "search", "Updating TableWidgetManager to navigate to ({}, {})", first_row, first_col);
                self.table_widget_manager
                    .borrow_mut()
                    .navigate_to_search_match(first_row, first_col);

                // Log what's actually at the position we're navigating to
                if let Some(dataview) = self.get_current_data() {
                    if let Some(row_data) = dataview.get_row(first_row) {
                        if first_col < row_data.values.len() {
                            info!(target: "search", "VALUE AT NAVIGATION TARGET ({}, {}): '{}'",
                                  first_row, first_col, row_data.values[first_col]);
                        }
                    }
                }

                // Convert matches to buffer format
                let buffer_matches: Vec<(usize, usize)> = {
                    let search_manager = self.search_manager.borrow();
                    search_manager
                        .all_matches()
                        .iter()
                        .map(|m| (m.row, m.column))
                        .collect()
                };

                // Also update AppStateContainer with matches (for compatibility)
                // Convert SearchManager matches to state_container format (row_start, col_start, row_end, col_end)
                let state_matches: Vec<(usize, usize, usize, usize)> = {
                    let search_manager = self.search_manager.borrow();
                    search_manager
                        .all_matches()
                        .iter()
                        .map(|m| {
                            // For now, treat each match as a single cell
                            (m.row, m.column, m.row, m.column)
                        })
                        .collect()
                };
                self.state_container.search_mut().matches = state_matches;

                self.state_container
                    .set_search_matches_with_index(buffer_matches.clone(), 0);
                self.state_container
                    .set_current_match(Some((first_row, first_col)));
                self.state_container
                    .set_status_message(format!("Found {} matches", match_count));

                info!(target: "search", "Search found {} matches for pattern '{}'", match_count, pattern);
            } else {
                // Clear search state
                self.state_container.search_mut().matches.clear();
                self.state_container.clear_search_state();
                self.state_container.set_current_match(None);

                info!(target: "search", "No matches found for pattern '{}'", pattern);
            }
        }
    }

    // --- Vim Search Methods ---

    /// Start vim-like forward search (/ key)
    fn start_vim_search(&mut self) {
        info!(target: "vim_search", "Starting vim search mode");

        // Start search mode in VimSearchManager
        self.vim_search_adapter.borrow_mut().start_search();

        // Observe search start
        self.shadow_state.borrow_mut().observe_search_start(
            crate::ui::state::shadow_state::SearchType::Vim,
            "slash_key_pressed",
        );

        // Use the existing SearchModesWidget which already has perfect debouncing
        self.enter_search_mode(SearchMode::Search);
    }

    /// Navigate to next vim search match (n key)
    fn vim_search_next(&mut self) {
        if !self.vim_search_adapter.borrow().is_navigating() {
            // Try to resume last search if not currently navigating
            let resumed = {
                let mut viewport_borrow = self.viewport_manager.borrow_mut();
                if let Some(ref mut viewport) = *viewport_borrow {
                    if let Some(dataview) = self.state_container.get_buffer_dataview() {
                        self.vim_search_adapter
                            .borrow_mut()
                            .resume_last_search(dataview, viewport)
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if !resumed {
                self.state_container
                    .set_status_message("No previous search pattern".to_string());
                return;
            }
        }

        // Navigate to next match
        let result = {
            let mut viewport_borrow = self.viewport_manager.borrow_mut();
            if let Some(ref mut viewport) = *viewport_borrow {
                let search_match = self.vim_search_adapter.borrow_mut().next_match(viewport);
                if search_match.is_some() {
                    let match_info = self.vim_search_adapter.borrow().get_match_info();
                    search_match.map(|m| (m, match_info))
                } else {
                    None
                }
            } else {
                None
            }
        }; // Drop viewport_borrow here

        // Update selected row AND column AFTER dropping the viewport borrow
        if let Some((ref search_match, _)) = result {
            // Log what we're updating
            info!(target: "search", 
                "=== UPDATING TUI STATE FOR VIM SEARCH ===");
            info!(target: "search", 
                "Setting selected row to: {}", search_match.row);
            info!(target: "search", 
                "Setting selected column to: {} (visual col)", search_match.col);

            // Verify what's actually at this position
            if let Some(dataview) = self.state_container.get_buffer_dataview() {
                // First, let's verify what row we're actually getting
                info!(target: "search",
                    "DEBUG: Fetching row {} from dataview with {} total rows",
                    search_match.row, dataview.row_count());

                if let Some(row_data) = dataview.get_row(search_match.row) {
                    // Log ALL values in this row to debug the mismatch
                    info!(target: "search",
                        "Row {} has {} values, first 5: {:?}",
                        search_match.row, row_data.values.len(),
                        row_data.values.iter().take(5).map(|v| v.to_string()).collect::<Vec<_>>());

                    if search_match.col < row_data.values.len() {
                        let actual_value = &row_data.values[search_match.col];
                        info!(target: "search", 
                            "Actual value at row {} col {}: '{}'", 
                            search_match.row, search_match.col, actual_value);

                        // Check if it actually contains the pattern
                        let pattern = self
                            .vim_search_adapter
                            .borrow()
                            .get_pattern()
                            .unwrap_or_default();
                        let contains_pattern = actual_value
                            .to_string()
                            .to_lowercase()
                            .contains(&pattern.to_lowercase());
                        if !contains_pattern {
                            warn!(target: "search", 
                                "WARNING: Cell at ({}, {}) = '{}' does NOT contain pattern '{}'!", 
                                search_match.row, search_match.col, actual_value, pattern);
                        } else {
                            info!(target: "search", 
                                "✓ Confirmed: Cell contains pattern '{}'", pattern);
                        }
                    } else {
                        warn!(target: "search", 
                            "Column {} is out of bounds for row {} (row has {} values)", 
                            search_match.col, search_match.row, row_data.values.len());
                    }
                } else {
                    warn!(target: "search", 
                        "Could not get row data for row {}", search_match.row);
                }

                // Also log display columns to understand the mapping
                let display_columns = dataview.get_display_columns();
                info!(target: "search", 
                    "Display columns mapping (first 10): {:?}", 
                    display_columns.iter().take(10).collect::<Vec<_>>());
            }

            self.state_container
                .set_table_selected_row(Some(search_match.row));
            self.state_container
                .set_selected_row(Some(search_match.row));

            // Update the selected column - search_match.col is already in visual coordinates
            // Keep everything in visual column indices for consistency
            info!(target: "search",
                "Setting column to visual index {}",
                search_match.col);

            // Update all column-related state to the visual column index
            self.state_container
                .set_current_column_buffer(search_match.col);
            self.state_container.navigation_mut().selected_column = search_match.col;

            // CRITICAL: Update SelectionState's selected_column too!
            self.state_container.select_column(search_match.col);
            info!(target: "search", 
                "Updated SelectionState column to: {}", search_match.col);

            // Log the current state of all column-related fields
            info!(target: "search", 
                "Column state after update: nav.selected_column={}, buffer.current_column={}, selection.selected_column={}", 
                self.state_container.navigation().selected_column,
                self.state_container.get_current_column(),
                self.state_container.selection().selected_column);

            // CRITICAL: Sync NavigationState with ViewportManager after vim search navigation
            // ViewportManager has the correct state after navigation, sync it back
            self.sync_navigation_with_viewport();

            // Also update the buffer scroll offset
            let scroll_offset = self.state_container.navigation().scroll_offset;
            self.state_container.set_scroll_offset(scroll_offset);

            // CRITICAL: Update TableWidgetManager to trigger re-render
            info!(target: "search", "Updating TableWidgetManager for vim search navigation to ({}, {})",
                  search_match.row, search_match.col);
            self.table_widget_manager
                .borrow_mut()
                .navigate_to(search_match.row, search_match.col);

            // Log column state after TableWidgetManager update
            info!(target: "search", 
                "After TableWidgetManager update: nav.selected_column={}, buffer.current_column={}, selection.selected_column={}", 
                self.state_container.navigation().selected_column,
                self.state_container.get_current_column(),
                self.state_container.selection().selected_column);

            // The ViewportManager has already handled all scrolling logic
            // Our sync_navigation_with_viewport() call above has updated NavigationState
            // No need for additional manual scroll updates
        }

        // Update status without borrow conflicts
        if let Some((search_match, match_info)) = result {
            if let Some((current, total)) = match_info {
                self.state_container.set_status_message(format!(
                    "Match {}/{} at ({}, {})",
                    current,
                    total,
                    search_match.row + 1,
                    search_match.col + 1
                ));
            }

            // Final column state logging
            info!(target: "search", 
                "FINAL vim_search_next state: nav.selected_column={}, buffer.current_column={}, selection.selected_column={}", 
                self.state_container.navigation().selected_column,
                self.state_container.get_current_column(),
                self.state_container.selection().selected_column);

            // CRITICAL: Verify what's actually at the final position
            if let Some(dataview) = self.state_container.get_buffer_dataview() {
                let final_row = self.state_container.navigation().selected_row;
                let final_col = self.state_container.navigation().selected_column;

                if let Some(row_data) = dataview.get_row(final_row) {
                    if final_col < row_data.values.len() {
                        let actual_value = &row_data.values[final_col];
                        info!(target: "search", 
                            "VERIFICATION: Cell at final position ({}, {}) contains: '{}'",
                            final_row, final_col, actual_value);

                        let pattern = self
                            .vim_search_adapter
                            .borrow()
                            .get_pattern()
                            .unwrap_or_default();
                        if !actual_value
                            .to_string()
                            .to_lowercase()
                            .contains(&pattern.to_lowercase())
                        {
                            error!(target: "search",
                                "ERROR: Final cell '{}' does NOT contain search pattern '{}'!",
                                actual_value, pattern);
                        }
                    }
                }
            }
        }
    }

    /// Navigate to previous vim search match (N key)
    fn vim_search_previous(&mut self) {
        if !self.vim_search_adapter.borrow().is_navigating() {
            // Try to resume last search if not currently navigating
            let resumed = {
                let mut viewport_borrow = self.viewport_manager.borrow_mut();
                if let Some(ref mut viewport) = *viewport_borrow {
                    if let Some(dataview) = self.state_container.get_buffer_dataview() {
                        self.vim_search_adapter
                            .borrow_mut()
                            .resume_last_search(dataview, viewport)
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if !resumed {
                self.state_container
                    .set_status_message("No previous search pattern".to_string());
                return;
            }
        }

        // Navigate to previous match
        let result = {
            let mut viewport_borrow = self.viewport_manager.borrow_mut();
            if let Some(ref mut viewport) = *viewport_borrow {
                let search_match = self
                    .vim_search_adapter
                    .borrow_mut()
                    .previous_match(viewport);
                if search_match.is_some() {
                    let match_info = self.vim_search_adapter.borrow().get_match_info();
                    search_match.map(|m| (m, match_info))
                } else {
                    None
                }
            } else {
                None
            }
        }; // Drop viewport_borrow here

        // Update selected row AND column AFTER dropping the viewport borrow
        if let Some((ref search_match, _)) = result {
            self.state_container
                .set_table_selected_row(Some(search_match.row));
            self.state_container
                .set_selected_row(Some(search_match.row));

            // Update the selected column - search_match.col is already in visual coordinates
            // Keep everything in visual column indices for consistency
            info!(target: "search",
                "Setting column to visual index {}",
                search_match.col);

            // Update all column-related state to the visual column index
            self.state_container
                .set_current_column_buffer(search_match.col);
            self.state_container.navigation_mut().selected_column = search_match.col;

            // CRITICAL: Update SelectionState's selected_column too!
            self.state_container.select_column(search_match.col);
            info!(target: "search", 
                "Updated SelectionState column to: {}", search_match.col);

            // Log the current state of all column-related fields
            info!(target: "search", 
                "Column state after update: nav.selected_column={}, buffer.current_column={}, selection.selected_column={}", 
                self.state_container.navigation().selected_column,
                self.state_container.get_current_column(),
                self.state_container.selection().selected_column);

            // CRITICAL: Sync NavigationState with ViewportManager after vim search navigation
            // ViewportManager has the correct state after navigation, sync it back
            self.sync_navigation_with_viewport();

            // Also update the buffer scroll offset
            let scroll_offset = self.state_container.navigation().scroll_offset;
            self.state_container.set_scroll_offset(scroll_offset);

            // CRITICAL: Update TableWidgetManager to trigger re-render
            info!(target: "search", "Updating TableWidgetManager for vim search navigation to ({}, {})",
                  search_match.row, search_match.col);
            self.table_widget_manager
                .borrow_mut()
                .navigate_to(search_match.row, search_match.col);

            // Log column state after TableWidgetManager update
            info!(target: "search", 
                "After TableWidgetManager update: nav.selected_column={}, buffer.current_column={}, selection.selected_column={}", 
                self.state_container.navigation().selected_column,
                self.state_container.get_current_column(),
                self.state_container.selection().selected_column);

            // The ViewportManager has already handled all scrolling logic
            // Our sync_navigation_with_viewport() call above has updated NavigationState
            // No need for additional manual scroll updates
        }

        // Update status without borrow conflicts
        if let Some((search_match, match_info)) = result {
            if let Some((current, total)) = match_info {
                self.state_container.set_status_message(format!(
                    "Match {}/{} at ({}, {})",
                    current,
                    total,
                    search_match.row + 1,
                    search_match.col + 1
                ));
            }
        }
        // ========== FILTER EXECUTION ==========
    }

    fn apply_filter(&mut self, pattern: &str) {
        use std::sync::atomic::{AtomicUsize, Ordering};

        // Simple re-entrancy detection without macros
        static FILTER_DEPTH: AtomicUsize = AtomicUsize::new(0);
        let depth = FILTER_DEPTH.fetch_add(1, Ordering::SeqCst);
        if depth > 0 {
            eprintln!(
                "WARNING: apply_filter re-entrancy detected! depth={}, pattern='{}', thread={:?}",
                depth,
                pattern,
                std::thread::current().id()
            );
        }

        info!(
            "Applying filter: '{}' on thread {:?}",
            pattern,
            std::thread::current().id()
        );

        // Delegate state coordination to StateCoordinator
        use crate::ui::state::state_coordinator::StateCoordinator;
        let _rows_after =
            StateCoordinator::apply_text_filter_with_refs(&mut self.state_container, pattern);

        // Update ViewportManager with the filtered DataView
        // Sync the dataview to both managers
        self.sync_dataview_to_managers();

        // Decrement re-entrancy counter
        FILTER_DEPTH.fetch_sub(1, Ordering::SeqCst);
    }
    fn search_columns(&mut self) {
        // Safety: Prevent infinite recursion with a static counter
        static SEARCH_DEPTH: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let depth = SEARCH_DEPTH.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Guard against excessive recursion
        if depth > 10 {
            error!(target: "search", "Column search depth exceeded limit, aborting to prevent infinite loop");
            SEARCH_DEPTH.store(0, std::sync::atomic::Ordering::SeqCst);
            return;
        }

        // Create a guard that will decrement on drop
        struct DepthGuard;
        impl Drop for DepthGuard {
            fn drop(&mut self) {
                SEARCH_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
        let _guard = DepthGuard;

        let pattern = self.state_container.column_search().pattern.clone();
        debug!(target: "search", "search_columns called with pattern: '{}', depth: {}", pattern, depth);

        if pattern.is_empty() {
            debug!(target: "search", "Pattern is empty, skipping column search");
            return;
        }

        // Update DataView's column search and get matches
        let matching_columns = if let Some(dataview) =
            self.state_container.get_buffer_dataview_mut()
        {
            dataview.search_columns(&pattern);

            // Get matching columns from DataView
            let matches = dataview.get_matching_columns().to_vec();
            debug!(target: "search", "DataView found {} matching columns", matches.len());
            if !matches.is_empty() {
                for (idx, (col_idx, col_name)) in matches.iter().enumerate() {
                    debug!(target: "search", "  Match {}: '{}' at visual index {}", idx + 1, col_name, col_idx);
                }
            }

            // Also sync with AppStateContainer for compatibility
            let columns: Vec<(String, usize)> = matches
                .iter()
                .map(|(idx, name)| (name.clone(), *idx))
                .collect();
            self.state_container
                .update_column_search_matches(&columns, &pattern);

            matches
        } else {
            debug!(target: "search", "No DataView available for column search");
            Vec::new()
        };

        if !matching_columns.is_empty() {
            // Move to first match - the index from DataView is already a VISUAL index
            let first_match_visual_idx = matching_columns[0].0;
            let first_match_name = &matching_columns[0].1;

            // Convert visual index to DataTable index for Buffer/AppStateContainer (legacy compatibility)
            let datatable_idx = if let Some(dataview) = self.state_container.get_buffer_dataview() {
                let display_columns = dataview.get_display_columns();
                if first_match_visual_idx < display_columns.len() {
                    display_columns[first_match_visual_idx]
                } else {
                    first_match_visual_idx // Fallback
                }
            } else {
                first_match_visual_idx
            };

            self.state_container.set_current_column(datatable_idx);
            self.state_container
                .set_current_column_buffer(datatable_idx);

            // Update viewport to show the first match using ViewportManager
            // ViewportManager expects VISUAL index
            {
                let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
                if let Some(viewport_manager) = viewport_manager_borrow.as_mut() {
                    let viewport_changed =
                        viewport_manager.set_current_column(first_match_visual_idx);

                    // Sync navigation state with updated viewport
                    if viewport_changed {
                        let new_viewport = viewport_manager.viewport_cols().clone();
                        let pinned_count =
                            if let Some(dv) = self.state_container.get_buffer_dataview() {
                                dv.get_pinned_columns().len()
                            } else {
                                0
                            };
                        let scrollable_offset = new_viewport.start.saturating_sub(pinned_count);
                        self.state_container.navigation_mut().scroll_offset.1 = scrollable_offset;

                        debug!(target: "navigation",
                            "Column search initial: Jumped to column {} '{}', viewport adjusted to {:?}",
                            first_match_visual_idx, first_match_name, new_viewport);
                    }
                }
            }

            debug!(target: "search", "Setting current column to visual index {} ('{}')",
                   first_match_visual_idx, first_match_name);
            let status_msg = format!(
                "Found {} columns matching '{}'. Tab/Shift-Tab to navigate.",
                matching_columns.len(),
                pattern
            );
            debug!(target: "search", "Setting status: {}", status_msg);
            self.state_container.set_status_message(status_msg);

            // Column search matches are now managed by AppStateContainer
        } else {
            let status_msg = format!("No columns matching '{}'", pattern);
            debug!(target: "search", "Setting status: {}", status_msg);
            self.state_container.set_status_message(status_msg);
        }

        // Matching columns are now stored in AppStateContainer
    }

    fn next_column_match(&mut self) {
        // Use DataView's column search navigation
        // Extract all needed data first to avoid borrow conflicts
        let column_match_data =
            if let Some(dataview) = self.state_container.get_buffer_dataview_mut() {
                if let Some(visual_idx) = dataview.next_column_match() {
                    // Get the column name and match info
                    let matching_columns = dataview.get_matching_columns();
                    let current_match_index = dataview.current_column_match_index();
                    let current_match = current_match_index + 1;
                    let total_matches = matching_columns.len();
                    let col_name = matching_columns
                        .get(current_match_index)
                        .map(|(_, name)| name.clone())
                        .unwrap_or_default();

                    // Convert visual index to DataTable index for Buffer/AppStateContainer
                    // (they still use DataTable indices for now)
                    let display_columns = dataview.get_display_columns();
                    let datatable_idx = if visual_idx < display_columns.len() {
                        display_columns[visual_idx]
                    } else {
                        visual_idx // Fallback
                    };

                    Some((
                        visual_idx,
                        datatable_idx,
                        col_name,
                        current_match,
                        total_matches,
                        current_match_index,
                    ))
                } else {
                    None
                }
            } else {
                None
            };

        // Now process the match data without holding dataview reference
        if let Some((
            visual_idx,
            datatable_idx,
            col_name,
            current_match,
            total_matches,
            current_match_index,
        )) = column_match_data
        {
            // Update both AppStateContainer and Buffer with DataTable index (for legacy compatibility)
            self.state_container.set_current_column(datatable_idx);
            self.state_container
                .set_current_column_buffer(datatable_idx);

            // Update viewport to show the column using ViewportManager
            // ViewportManager's set_current_column now expects VISUAL index
            {
                let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
                if let Some(viewport_manager) = viewport_manager_borrow.as_mut() {
                    viewport_manager.set_current_column(visual_idx);
                }
            }

            // Always sync navigation state with updated viewport (like vim search)
            debug!(target: "column_search_sync", "next_column_match: About to call sync_navigation_with_viewport() - visual_idx: {}, datatable_idx: {}", visual_idx, datatable_idx);
            debug!(target: "column_search_sync", "next_column_match: Pre-sync - viewport current_column: {}", 
                if let Some(vm) = self.viewport_manager.try_borrow().ok() {
                    vm.as_ref().map(|v| v.get_crosshair_col()).unwrap_or(0)
                } else { 0 });
            self.sync_navigation_with_viewport();
            debug!(target: "column_search_sync", "next_column_match: Post-sync - navigation current_column: {}", 
                self.state_container.navigation().selected_column);
            debug!(target: "column_search_sync", "next_column_match: sync_navigation_with_viewport() completed");

            debug!(target: "navigation",
                "Column search: Jumped to visual column {} (datatable: {}) '{}', synced with viewport",
                visual_idx, datatable_idx, col_name);

            // CRITICAL: Update AppStateContainer's column_search.current_match
            // This ensures Enter key will jump to the correct column
            {
                let mut column_search = self.state_container.column_search_mut();
                column_search.current_match = current_match_index;
                debug!(target: "column_search_sync", "next_column_match: Updated AppStateContainer column_search.current_match to {}", column_search.current_match);
            }

            self.state_container.set_status_message(format!(
                "Column {}/{}: {} - Tab/Shift-Tab to navigate",
                current_match, total_matches, col_name
            ));
        }
    }

    fn previous_column_match(&mut self) {
        // Use DataView's column search navigation
        // Extract all needed data first to avoid borrow conflicts
        let column_match_data =
            if let Some(dataview) = self.state_container.get_buffer_dataview_mut() {
                if let Some(visual_idx) = dataview.prev_column_match() {
                    // Get the column name and match info
                    let matching_columns = dataview.get_matching_columns();
                    let current_match_index = dataview.current_column_match_index();
                    let current_match = current_match_index + 1;
                    let total_matches = matching_columns.len();
                    let col_name = matching_columns
                        .get(current_match_index)
                        .map(|(_, name)| name.clone())
                        .unwrap_or_default();

                    // Convert visual index to DataTable index for Buffer/AppStateContainer
                    // (they still use DataTable indices for now)
                    let display_columns = dataview.get_display_columns();
                    let datatable_idx = if visual_idx < display_columns.len() {
                        display_columns[visual_idx]
                    } else {
                        visual_idx // Fallback
                    };

                    Some((
                        visual_idx,
                        datatable_idx,
                        col_name,
                        current_match,
                        total_matches,
                        current_match_index,
                    ))
                } else {
                    None
                }
            } else {
                None
            };

        // Now process the match data without holding dataview reference
        if let Some((
            visual_idx,
            datatable_idx,
            col_name,
            current_match,
            total_matches,
            current_match_index,
        )) = column_match_data
        {
            // Update both AppStateContainer and Buffer with DataTable index (for legacy compatibility)
            self.state_container.set_current_column(datatable_idx);
            self.state_container
                .set_current_column_buffer(datatable_idx);

            // Update viewport to show the column using ViewportManager
            // ViewportManager's set_current_column now expects VISUAL index
            {
                let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
                if let Some(viewport_manager) = viewport_manager_borrow.as_mut() {
                    viewport_manager.set_current_column(visual_idx);
                }
            }

            // Always sync navigation state with updated viewport (like vim search)
            debug!(target: "column_search_sync", "previous_column_match: About to call sync_navigation_with_viewport() - visual_idx: {}, datatable_idx: {}", visual_idx, datatable_idx);
            debug!(target: "column_search_sync", "previous_column_match: Pre-sync - viewport current_column: {}", 
                if let Some(vm) = self.viewport_manager.try_borrow().ok() {
                    vm.as_ref().map(|v| v.get_crosshair_col()).unwrap_or(0)
                } else { 0 });
            self.sync_navigation_with_viewport();
            debug!(target: "column_search_sync", "previous_column_match: Post-sync - navigation current_column: {}", 
                self.state_container.navigation().selected_column);
            debug!(target: "column_search_sync", "previous_column_match: sync_navigation_with_viewport() completed");

            debug!(target: "navigation",
                "Column search (prev): Jumped to visual column {} (datatable: {}) '{}', synced with viewport",
                visual_idx, datatable_idx, col_name);

            // CRITICAL: Update AppStateContainer's column_search.current_match
            // This ensures Enter key will jump to the correct column
            {
                let mut column_search = self.state_container.column_search_mut();
                column_search.current_match = current_match_index;
                debug!(target: "column_search_sync", "previous_column_match: Updated AppStateContainer column_search.current_match to {}", column_search.current_match);
            }

            self.state_container.set_status_message(format!(
                "Column {}/{}: {} - Tab/Shift-Tab to navigate",
                current_match, total_matches, col_name
            ));
        }
    }

    fn apply_fuzzy_filter(&mut self) {
        info!(
            "apply_fuzzy_filter called on thread {:?}",
            std::thread::current().id()
        );

        // Delegate all state coordination to StateCoordinator
        use crate::ui::state::state_coordinator::StateCoordinator;
        let (_match_count, indices) = StateCoordinator::apply_fuzzy_filter_with_refs(
            &mut self.state_container,
            &self.viewport_manager,
        );

        // Update fuzzy filter indices for compatibility
        self.state_container.set_fuzzy_filter_indices(indices);

        // Update ViewportManager with the filtered DataView
        // Sync the dataview to both managers
        self.sync_dataview_to_managers();
    }

    fn toggle_sort_current_column(&mut self) {
        // Get visual column index from ViewportManager's crosshair
        let visual_col_idx = if let Some(ref viewport_manager) = *self.viewport_manager.borrow() {
            viewport_manager.get_crosshair_col()
        } else {
            0
        };

        if let Some(dataview) = self.state_container.get_buffer_dataview_mut() {
            // DataView.toggle_sort expects VISIBLE column index
            // Get column name for display
            let column_names = dataview.column_names();
            let col_name = column_names
                .get(visual_col_idx)
                .map(|s| s.clone())
                .unwrap_or_else(|| format!("Column {}", visual_col_idx));

            debug!(
                "toggle_sort_current_column: visual_idx={}, column_name={}",
                visual_col_idx, col_name
            );

            if let Err(e) = dataview.toggle_sort(visual_col_idx) {
                self.state_container
                    .set_status_message(format!("Sort error: {}", e));
            } else {
                // Get the new sort state for status message
                let sort_state = dataview.get_sort_state();
                let message = match sort_state.order {
                    crate::data::data_view::SortOrder::Ascending => {
                        format!("Sorted '{}' ascending ↑", col_name)
                    }
                    crate::data::data_view::SortOrder::Descending => {
                        format!("Sorted '{}' descending ↓", col_name)
                    }
                    crate::data::data_view::SortOrder::None => {
                        format!("Cleared sort on '{}'", col_name)
                    }
                };
                self.state_container.set_status_message(message);

                // Update ViewportManager with the sorted DataView to keep them in sync
                if let Some(updated_dataview) = self.state_container.get_buffer_dataview() {
                    // Update TableWidgetManager with the sorted dataview as well
                    self.table_widget_manager
                        .borrow_mut()
                        .set_dataview(Arc::new(updated_dataview.clone()));

                    let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
                    if let Some(ref mut viewport_manager) = *viewport_manager_borrow {
                        viewport_manager.set_dataview(Arc::new(updated_dataview.clone()));
                        debug!("Updated ViewportManager with sorted DataView");
                    }
                }
            }
        } else {
            // Could not find display position in DataTable
            self.state_container
                .set_status_message("Error: Invalid column position".to_string());
        }
    }

    fn get_current_data(&self) -> Option<&DataView> {
        self.state_container.get_buffer_dataview()
    }

    fn get_row_count(&self) -> usize {
        // Check if fuzzy filter is active first (most specific filter)
        if self.state_container.is_fuzzy_filter_active() {
            // Return the count of fuzzy filtered indices
            self.state_container.get_fuzzy_filter_indices().len()
        } else if let Some(dataview) = self.state_container.get_buffer_dataview() {
            // Return count from WHERE clause or other filters
            dataview.row_count()
        } else if let Some(provider) = self.get_data_provider() {
            // Use DataProvider trait for data access (migration step)
            provider.get_row_count()
        } else {
            0
        }
    }

    /// Helper to sync dataview to both ViewportManager and TableWidgetManager
    fn sync_dataview_to_managers(&self) {
        if let Some(dataview) = self.state_container.get_buffer_dataview() {
            let arc_dataview = Arc::new(dataview.clone());

            // Update ViewportManager
            if let Some(ref mut viewport_manager) = *self.viewport_manager.borrow_mut() {
                viewport_manager.set_dataview(arc_dataview.clone());
                debug!(
                    "Updated ViewportManager with DataView (row_count={})",
                    arc_dataview.row_count()
                );
            }

            // Update TableWidgetManager
            self.table_widget_manager
                .borrow_mut()
                .set_dataview(arc_dataview);
            debug!("Updated TableWidgetManager with DataView");
        }
    }

    pub fn reset_table_state(&mut self) {
        // Delegate all state reset coordination to StateCoordinator
        use crate::ui::state::state_coordinator::StateCoordinator;
        StateCoordinator::reset_table_state_with_refs(
            &mut self.state_container,
            &self.viewport_manager,
        );
    }

    fn update_parser_for_current_buffer(&mut self) {
        // Sync input states
        self.sync_all_input_states();

        // Delegate parser update to StateCoordinator
        use crate::ui::state::state_coordinator::StateCoordinator;
        StateCoordinator::update_parser_with_refs(&self.state_container, &mut self.hybrid_parser);
    }

    /// Synchronize all state after buffer switch
    /// This should be called after any buffer switch operation to ensure:
    /// 1. Viewport is restored from the new buffer
    /// 2. Parser schema is updated with the new buffer's columns
    fn sync_after_buffer_switch(&mut self) {
        // Restore viewport state from new buffer
        self.restore_viewport_from_current_buffer();

        // Update parser schema for the new buffer
        self.update_parser_for_current_buffer();
    }

    /// Update ViewportManager when DataView changes
    fn update_viewport_manager(&mut self, dataview: Option<DataView>) {
        if let Some(dv) = dataview {
            // Get current column position to preserve it
            let current_column = self.state_container.get_current_column();

            // Create new ViewportManager with the new DataView
            let mut new_viewport_manager = ViewportManager::new(Arc::new(dv));

            // Update terminal size from current terminal
            if let Ok((width, height)) = crossterm::terminal::size() {
                // Calculate the actual data area height
                let data_rows_available = Self::calculate_available_data_rows(height);
                new_viewport_manager.update_terminal_size(width, data_rows_available);
                debug!(
                    "Updated new ViewportManager terminal size: {}x{} (data rows)",
                    width, data_rows_available
                );
            }

            // Set the current column position to ensure proper viewport initialization
            // This is crucial for SELECT queries that subset columns
            if current_column < new_viewport_manager.dataview().column_count() {
                new_viewport_manager.set_current_column(current_column);
            } else {
                // If current column is out of bounds, reset to first column
                new_viewport_manager.set_current_column(0);
                self.state_container.set_current_column_buffer(0);
            }

            *self.viewport_manager.borrow_mut() = Some(new_viewport_manager);
            debug!(
                "ViewportManager updated with new DataView, current_column={}",
                current_column
            );
        } else {
            // Clear ViewportManager if no DataView
            *self.viewport_manager.borrow_mut() = None;
            debug!("ViewportManager cleared (no DataView)");
        }
    }

    pub fn calculate_optimal_column_widths(&mut self) {
        // Delegate to ViewportManager for optimal column width calculations
        let widths_from_viewport = {
            let mut viewport_opt = self.viewport_manager.borrow_mut();
            if let Some(ref mut viewport_manager) = *viewport_opt {
                Some(viewport_manager.calculate_optimal_column_widths())
            } else {
                None
            }
        };

        if let Some(widths) = widths_from_viewport {
            self.state_container.set_column_widths(widths);
        }
    }

    /// Centralized method for setting status messages
    /// Ensures consistent logging and state synchronization
    pub fn set_status_message(&mut self, message: impl Into<String>) {
        let msg = message.into();
        debug!("Status: {}", msg);
        self.state_container.set_status_message(msg.clone());
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
        let result = {
            let ctx = crate::ui::operations::data_export_operations::DataExportContext {
                data_provider: self.get_data_provider(),
            };
            crate::ui::operations::data_export_operations::export_to_csv(&ctx)
        };

        match result {
            crate::ui::operations::data_export_operations::ExportResult::Success(message) => {
                self.set_status_message(message);
            }
            crate::ui::operations::data_export_operations::ExportResult::Error(error) => {
                self.set_error_status("Export failed", error);
            }
        }
    }

    // ========== YANK OPERATIONS ==========

    // Yank operations are provided by the YankBehavior trait in traits/yank_ops.rs
    // The trait provides: yank_cell, yank_row, yank_column, yank_all, yank_query,
    // yank_as_test_case, and yank_debug_with_context

    fn paste_from_clipboard(&mut self) {
        // Paste from system clipboard into the current input field
        match self.state_container.read_from_clipboard() {
            Ok(text) => {
                let mode = self.shadow_state.borrow().get_mode();
                match mode {
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

                        self.state_container
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

                        // Update the appropriate filter/search state (reuse the mode we already have)
                        match mode {
                            AppMode::Filter => {
                                let pattern = self.get_input_text();
                                self.state_container.filter_mut().pattern = pattern.clone();
                                self.apply_filter(&pattern);
                            }
                            AppMode::FuzzyFilter => {
                                let input_text = self.get_input_text();
                                self.state_container.set_fuzzy_filter_pattern(input_text);
                                self.apply_fuzzy_filter();
                            }
                            AppMode::Search => {
                                let search_text = self.get_input_text();
                                self.state_container.set_search_pattern(search_text);
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
                        self.state_container
                            .set_status_message("Paste not available in this mode".to_string());
                    }
                }
            }
            Err(e) => {
                self.state_container
                    .set_status_message(format!("Failed to paste: {}", e));
            }
        }
    }

    fn export_to_json(&mut self) {
        // TODO: Handle filtered data in future DataView implementation
        let result = {
            let ctx = crate::ui::operations::data_export_operations::DataExportContext {
                data_provider: self.get_data_provider(),
            };
            crate::ui::operations::data_export_operations::export_to_json(&ctx)
        };

        match result {
            crate::ui::operations::data_export_operations::ExportResult::Success(message) => {
                self.set_status_message(message);
            }
            crate::ui::operations::data_export_operations::ExportResult::Error(error) => {
                self.set_error_status("Export failed", error);
            }
        }
    }

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
        let ctx = crate::ui::operations::simple_operations::TextNavigationContext {
            query: &self.get_input_text(),
            cursor_pos: self.get_input_cursor(),
        };
        crate::ui::operations::simple_operations::get_cursor_token_position(&ctx)
    }

    fn get_token_at_cursor(&self) -> Option<String> {
        let ctx = crate::ui::operations::simple_operations::TextNavigationContext {
            query: &self.get_input_text(),
            cursor_pos: self.get_input_cursor(),
        };
        crate::ui::operations::simple_operations::get_token_at_cursor(&ctx)
    }

    /// Debug method to dump current buffer state (disabled to prevent TUI corruption)
    #[allow(dead_code)]

    fn ui(&mut self, f: &mut Frame) {
        // Always use single-line mode input height
        let input_height = INPUT_AREA_HEIGHT;

        // Always show tab bar for consistent layout
        let buffer_count = self.state_container.buffers().all_buffers().len();
        let tab_bar_height = 2; // Always reserve space for tab bar

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(tab_bar_height),    // Tab bar (always shown)
                    Constraint::Length(input_height),      // Command input area
                    Constraint::Min(0),                    // Results
                    Constraint::Length(STATUS_BAR_HEIGHT), // Status bar
                ]
                .as_ref(),
            )
            .split(f.area());

        // Always render tab bar (even with single buffer)
        if buffer_count > 0 {
            let buffer_names: Vec<String> = self
                .state_container
                .buffers()
                .all_buffers()
                .iter()
                .map(|b| b.get_name())
                .collect();
            let current_index = self.state_container.buffers().current_index();

            let tab_widget = TabBarWidget::new(current_index, buffer_names);
            tab_widget.render(f, chunks[0]);
        }

        // Fixed chunk indices since tab bar is always present
        let input_chunk_idx = 1;
        let results_chunk_idx = 2;
        let status_chunk_idx = 3;

        // Update horizontal scroll based on actual terminal width
        self.update_horizontal_scroll(chunks[input_chunk_idx].width);

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

        let input_title = match self.shadow_state.borrow().get_mode() {
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
            self.shadow_state.borrow().get_mode(),
            AppMode::Search | AppMode::Filter | AppMode::FuzzyFilter | AppMode::ColumnSearch
        ) && self.search_modes_widget.is_active();

        if use_search_widget {
            // Let the search modes widget render the input field with debounce indicator
            self.search_modes_widget.render(f, chunks[input_chunk_idx]);
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
                   self.shadow_state.borrow().get_mode(),
                   self.get_input_cursor());

            // Get history search query if in history mode
            let history_query_string = if self.shadow_state.borrow().is_in_history_mode() {
                self.state_container.history_search().query.clone()
            } else {
                String::new()
            };

            let input_text = match self.shadow_state.borrow().get_mode() {
                AppMode::History => &history_query_string,
                _ => &input_text_string,
            };

            let input_paragraph = match self.shadow_state.borrow().get_mode() {
                AppMode::Command => {
                    match self.state_container.get_edit_mode() {
                        Some(EditMode::SingleLine) => {
                            // Use syntax highlighting for SQL command input with horizontal scrolling
                            let highlighted_line =
                                self.sql_highlighter.simple_sql_highlight(input_text);
                            Paragraph::new(Text::from(vec![highlighted_line]))
                                .block(input_block)
                                .scroll((0, self.get_horizontal_scroll_offset()))
                        }
                        Some(EditMode::MultiLine) => {
                            // MultiLine mode is no longer supported, always use single-line
                            let highlighted_line =
                                self.sql_highlighter.simple_sql_highlight(input_text);
                            Paragraph::new(Text::from(vec![highlighted_line]))
                                .block(input_block)
                                .scroll((0, self.get_horizontal_scroll_offset()))
                        }
                        None => {
                            // Default to single-line mode
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
                        .style(match self.shadow_state.borrow().get_mode() {
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
            f.render_widget(input_paragraph, chunks[input_chunk_idx]);
        }
        let results_area = chunks[results_chunk_idx];

        // Set cursor position for input modes (skip if search widget is handling it)
        if !use_search_widget {
            match self.shadow_state.borrow().get_mode() {
                AppMode::Command => {
                    // Always use single-line cursor handling
                    // Calculate cursor position with horizontal scrolling
                    let inner_width = chunks[input_chunk_idx].width.saturating_sub(2) as usize;
                    let cursor_pos = self.get_visual_cursor().1; // Get column position for single-line
                    let scroll_offset = self.get_horizontal_scroll_offset() as usize;

                    // Calculate visible cursor position
                    if cursor_pos >= scroll_offset && cursor_pos < scroll_offset + inner_width {
                        let visible_pos = cursor_pos - scroll_offset;
                        f.set_cursor_position((
                            chunks[input_chunk_idx].x + visible_pos as u16 + 1,
                            chunks[input_chunk_idx].y + 1,
                        ));
                    }
                }
                AppMode::Search => {
                    f.set_cursor_position((
                        chunks[input_chunk_idx].x + self.get_input_cursor() as u16 + 1,
                        chunks[input_chunk_idx].y + 1,
                    ));
                }
                AppMode::Filter => {
                    f.set_cursor_position((
                        chunks[input_chunk_idx].x + self.get_input_cursor() as u16 + 1,
                        chunks[input_chunk_idx].y + 1,
                    ));
                }
                AppMode::FuzzyFilter => {
                    f.set_cursor_position((
                        chunks[input_chunk_idx].x + self.get_input_cursor() as u16 + 1,
                        chunks[input_chunk_idx].y + 1,
                    ));
                }
                AppMode::ColumnSearch => {
                    f.set_cursor_position((
                        chunks[input_chunk_idx].x + self.get_input_cursor() as u16 + 1,
                        chunks[input_chunk_idx].y + 1,
                    ));
                }
                AppMode::JumpToRow => {
                    f.set_cursor_position((
                        chunks[input_chunk_idx].x + self.get_jump_to_row_input().len() as u16 + 1,
                        chunks[input_chunk_idx].y + 1,
                    ));
                }
                AppMode::History => {
                    let query_len = self.state_container.history_search().query.len();
                    f.set_cursor_position((
                        chunks[input_chunk_idx].x + query_len as u16 + 1,
                        chunks[input_chunk_idx].y + 1,
                    ));
                }
                _ => {}
            }
        }

        // Results area - render based on mode to reduce complexity
        let mode = self.shadow_state.borrow().get_mode();
        match mode {
            AppMode::Help => self.render_help(f, results_area),
            AppMode::History => self.render_history(f, results_area),
            AppMode::Debug => self.render_debug(f, results_area),
            AppMode::PrettyQuery => self.render_pretty_query(f, results_area),
            AppMode::ColumnStats => self.render_column_stats(f, results_area),
            _ if self.state_container.has_dataview() => {
                // Calculate viewport using DataView
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
        self.render_status_line(f, chunks[status_chunk_idx]);
        // ========== RENDERING ==========
    }

    /// Add mode styling and indicator to status spans
    fn add_mode_styling(&self, spans: &mut Vec<Span>) -> (Style, Color) {
        // Determine the mode color
        let (status_style, mode_color) = match self.shadow_state.borrow().get_mode() {
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

        let mode_indicator = match self.shadow_state.borrow().get_mode() {
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

        // Mode indicator with color
        spans.push(Span::styled(
            format!("[{}]", mode_indicator),
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        ));

        (status_style, mode_color)
    }

    /// Add data source display to status spans
    fn add_data_source_display(&self, spans: &mut Vec<Span>) {
        // Skip showing data source in status line since tab bar shows file names
        // This avoids redundancy like "[trades.csv] [1/2] trades.csv"
    }

    /// Add buffer information to status spans
    fn add_buffer_information(&self, spans: &mut Vec<Span>) {
        let index = self.state_container.buffers().current_index();
        let total = self.state_container.buffers().all_buffers().len();

        // Show buffer indicator if multiple buffers
        if total > 1 {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("[{}/{}]", index + 1, total),
                Style::default().fg(Color::Yellow),
            ));
        }

        // Show table name from current query (simplified)
        // Since tab bar shows file names, we just show the table being queried
        if let Some(buffer) = self.state_container.buffers().current() {
            let query = buffer.get_input_text();
            // Simple extraction of table name from "SELECT ... FROM table" pattern
            if let Some(from_pos) = query.to_uppercase().find(" FROM ") {
                let after_from = &query[from_pos + 6..];
                // Take the first word after FROM as the table name
                if let Some(table_name) = after_from.split_whitespace().next() {
                    // Clean up the table name (remove quotes, etc.)
                    let clean_name = table_name
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim_matches('`');

                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        clean_name.to_string(),
                        Style::default().fg(Color::Cyan),
                    ));
                }
            }
        }
    }

    /// Add mode-specific information to status spans
    fn add_mode_specific_info(&self, spans: &mut Vec<Span>, mode_color: Color, area: Rect) {
        match self.shadow_state.borrow().get_mode() {
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
                // Extract this separately due to its size
                self.add_results_mode_info(spans, area);
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
    }

    /// Add Results mode specific information (restored critical navigation info)
    fn add_results_mode_info(&self, spans: &mut Vec<Span>, area: Rect) {
        let total_rows = self.get_row_count();
        if total_rows > 0 {
            // Get selected row directly from navigation state (0-indexed) and add 1 for display
            let selected = self.state_container.navigation().selected_row + 1;
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
            // Use ViewportManager's visual column position (1-based for display)
            let visual_col_display =
                if let Some(ref viewport_manager) = *self.viewport_manager.borrow() {
                    viewport_manager.get_crosshair_col() + 1
                } else {
                    1
                };
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("({},{})", visual_col_display, selected),
                Style::default().fg(Color::DarkGray),
            ));

            // Add actual terminal cursor position if we can calculate it
            if let Some(ref mut viewport_manager) = *self.viewport_manager.borrow_mut() {
                let available_width = area.width.saturating_sub(TABLE_BORDER_WIDTH) as u16;
                // Use ViewportManager's crosshair column position
                let visual_col = viewport_manager.get_crosshair_col();
                if let Some(x_pos) =
                    viewport_manager.get_column_x_position(visual_col, available_width)
                {
                    // Add 2 for left border and padding, add 3 for header rows
                    let terminal_x = x_pos + 2;
                    let terminal_y = (selected as u16)
                        .saturating_sub(self.state_container.get_scroll_offset().0 as u16)
                        + 3;
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        format!("[{}x{}]", terminal_x, terminal_y),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            }

            // Column information
            if let Some(dataview) = self.state_container.get_buffer_dataview() {
                let headers = dataview.column_names();

                // Get ViewportManager's crosshair position (visual coordinates)
                // and use it to get the correct column name
                let (visual_row, visual_col) =
                    if let Some(ref viewport_manager) = *self.viewport_manager.borrow() {
                        (
                            viewport_manager.get_crosshair_row(),
                            viewport_manager.get_crosshair_col(),
                        )
                    } else {
                        (0, 0)
                    };

                // Use ViewportManager's visual column index to get the correct column name
                if visual_col < headers.len() {
                    spans.push(Span::raw(" | Col: "));
                    spans.push(Span::styled(
                        headers[visual_col].clone(),
                        Style::default().fg(Color::Cyan),
                    ));

                    // Show ViewportManager's crosshair position and viewport size
                    let viewport_info =
                        if let Some(ref viewport_manager) = *self.viewport_manager.borrow() {
                            let viewport_rows = viewport_manager.get_viewport_rows();
                            let viewport_height = viewport_rows.end - viewport_rows.start;
                            format!("[V:{},{} @ {}r]", visual_row, visual_col, viewport_height)
                        } else {
                            format!("[V:{},{}]", visual_row, visual_col)
                        };
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        viewport_info,
                        Style::default().fg(Color::Magenta),
                    ));
                }
            }
        }
    }

    fn render_status_line(&self, f: &mut Frame, area: Rect) {
        let mut spans = Vec::new();

        // Add mode styling and indicator
        let (status_style, mode_color) = self.add_mode_styling(&mut spans);

        // Add data source display
        self.add_data_source_display(&mut spans);

        // Add buffer information
        self.add_buffer_information(&mut spans);

        // Add mode-specific information
        self.add_mode_specific_info(&mut spans, mode_color, area);

        // Add query source indicator
        self.add_query_source_indicator(&mut spans);

        // Add case sensitivity indicator
        self.add_case_sensitivity_indicator(&mut spans);

        // Add column packing mode indicator
        self.add_column_packing_indicator(&mut spans);

        // Add status message
        self.add_status_message(&mut spans);

        // Determine help text based on current mode
        let help_text = self.get_help_text_for_mode();

        self.add_global_indicators(&mut spans);

        // Add shadow state display
        self.add_shadow_state_display(&mut spans);

        self.add_help_text_display(&mut spans, help_text, area);

        let status_line = Line::from(spans);
        let status = Paragraph::new(status_line)
            .block(Block::default().borders(Borders::ALL))
            .style(status_style);
        f.render_widget(status, area);
    }

    /// Build a TableRenderContext with all data needed for rendering
    /// This collects all the scattered data into a single struct
    fn build_table_context(
        &self,
        area: Rect,
        provider: &dyn DataProvider,
    ) -> crate::ui::rendering::table_render_context::TableRenderContext {
        use crate::ui::rendering::table_render_context::TableRenderContextBuilder;

        let row_count = provider.get_row_count();
        let available_width = area.width.saturating_sub(TABLE_BORDER_WIDTH) as u16;
        // The area passed here is already the table area
        // The Table widget itself handles borders and header
        let available_height = area.height as u16;

        // Get headers from ViewportManager (single source of truth)
        let headers = {
            let viewport_manager = self.viewport_manager.borrow();
            let viewport_manager = viewport_manager
                .as_ref()
                .expect("ViewportManager must exist");
            viewport_manager.get_column_names_ordered()
        };

        // Update ViewportManager with current terminal dimensions
        {
            let mut viewport_opt = self.viewport_manager.borrow_mut();
            if let Some(ref mut viewport_manager) = *viewport_opt {
                // Calculate data rows for the table area
                let data_rows = Self::calculate_table_data_rows(available_height);
                viewport_manager.update_terminal_size(available_width, data_rows);
                let _ = viewport_manager.get_column_widths(); // Trigger recalculation
            }
        }

        // Get structured column information from ViewportManager
        let (pinned_visual_positions, crosshair_column_position, _) = {
            let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
            let viewport_manager = viewport_manager_borrow
                .as_mut()
                .expect("ViewportManager must exist for rendering");
            let info = viewport_manager.get_visible_columns_info(available_width);

            // info.0 = visible_indices (all visible column source indices)
            // info.1 = pinned_visible (pinned column source indices)
            // info.2 = scrollable_visible (scrollable column source indices)
            let visible_indices = &info.0;
            let pinned_source_indices = &info.1;

            // Convert pinned source indices to visual positions
            // The TableRenderContext expects visual positions (0-based positions in the visible array)
            let mut pinned_visual_positions = Vec::new();
            for &source_idx in pinned_source_indices {
                if let Some(visual_pos) = visible_indices.iter().position(|&x| x == source_idx) {
                    pinned_visual_positions.push(visual_pos);
                }
            }

            // Get the crosshair's viewport-relative position for rendering
            // The viewport manager stores crosshair in absolute coordinates
            // but we need viewport-relative for rendering
            let crosshair_column_position =
                if let Some((_, col_pos)) = viewport_manager.get_crosshair_viewport_position() {
                    col_pos
                } else {
                    // Crosshair is outside viewport, default to 0
                    0
                };

            let crosshair_visual = viewport_manager.get_crosshair_col();

            (
                pinned_visual_positions,
                crosshair_column_position,
                crosshair_visual,
            )
        };

        // Calculate row viewport
        let row_viewport_start = self
            .state_container
            .navigation()
            .scroll_offset
            .0
            .min(row_count.saturating_sub(1));
        let row_viewport_end = (row_viewport_start + available_height as usize).min(row_count);
        let visible_row_indices: Vec<usize> = (row_viewport_start..row_viewport_end).collect();

        // Get the visual display data from ViewportManager
        let (column_headers, data_to_display, column_widths_visual) = {
            let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
            if let Some(ref mut viewport_manager) = *viewport_manager_borrow {
                viewport_manager.get_visual_display(available_width, &visible_row_indices)
            } else {
                // Fallback
                let visible_rows = provider
                    .get_visible_rows(row_viewport_start, row_viewport_end - row_viewport_start);
                let widths = vec![15u16; headers.len()];
                (headers.clone(), visible_rows, widths)
            }
        };

        // Get sort state
        let sort_state = self
            .buffer()
            .get_dataview()
            .map(|dv| dv.get_sort_state().clone());

        // Get filter info
        let fuzzy_filter_pattern = if self.state_container.is_fuzzy_filter_active() {
            let pattern = self.state_container.get_fuzzy_filter_pattern();
            if !pattern.is_empty() {
                Some(pattern)
            } else {
                None
            }
        } else {
            None
        };

        // Build the context
        let selected_row = self.state_container.navigation().selected_row;
        let selected_col = crosshair_column_position;

        // Log what we're passing to the renderer
        trace!(target: "search", "Building TableRenderContext: selected_row={}, selected_col={}, mode={:?}",
               selected_row, selected_col, self.state_container.get_selection_mode());

        TableRenderContextBuilder::new()
            .row_count(row_count)
            .visible_rows(visible_row_indices.clone(), data_to_display)
            .columns(column_headers, column_widths_visual)
            .pinned_columns(pinned_visual_positions)
            .selection(
                selected_row,
                selected_col,
                self.state_container.get_selection_mode(),
            )
            .row_viewport(row_viewport_start..row_viewport_end)
            .sort_state(sort_state)
            .display_options(
                self.state_container.is_show_row_numbers(),
                self.shadow_state.borrow().get_mode(),
            )
            .filter(
                fuzzy_filter_pattern,
                self.state_container.is_case_insensitive(),
            )
            .dimensions(available_width, available_height)
            .build()
    }

    /// New trait-based table rendering method
    /// This uses DataProvider trait instead of directly accessing QueryResponse
    fn render_table_with_provider(&self, f: &mut Frame, area: Rect, provider: &dyn DataProvider) {
        // Build the context with all data needed for rendering
        let context = self.build_table_context(area, provider);

        // Use the pure table renderer
        crate::ui::rendering::table_renderer::render_table(f, area, &context);
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
        <Self as DebugContext>::render_debug(self, f, area);
    }

    fn render_pretty_query(&self, f: &mut Frame, area: Rect) {
        <Self as DebugContext>::render_pretty_query(self, f, area);
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

    fn handle_column_stats_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Create context and delegate to extracted handler
        let mut ctx = crate::ui::input::input_handlers::StatsInputContext {
            buffer_manager: self.state_container.buffers_mut(),
            stats_widget: &mut self.stats_widget,
            shadow_state: &self.shadow_state,
        };

        let result = crate::ui::input::input_handlers::handle_column_stats_input(&mut ctx, key)?;

        // Check if mode changed to Results (happens when stats view is closed)
        if self.shadow_state.borrow().get_mode() == AppMode::Results {
            self.shadow_state
                .borrow_mut()
                .observe_mode_change(AppMode::Results, "column_stats_closed");
        }

        Ok(result)
    }

    fn handle_jump_to_row_input(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter => {
                // Use NavigationBehavior trait's complete_jump_to_row method
                let input = self.get_jump_to_row_input();
                self.complete_jump_to_row(&input);
            }
            _ => {
                // Use InputBehavior trait's process_jump_to_row_key for other keys
                self.process_jump_to_row_key(key);
            }
        }
        Ok(false)
    }

    fn render_column_stats(&self, f: &mut Frame, area: Rect) {
        // Delegate to the stats widget
        self.stats_widget.render(
            f,
            area,
            self.state_container
                .current_buffer()
                .expect("Buffer should exist"),
        );
    }

    // === Editor Widget Helper Methods ===
    // ========== QUERY EXECUTION ==========

    // These methods handle the actions returned by the editor widget

    fn handle_execute_query(&mut self) -> Result<bool> {
        use crate::ui::state::state_coordinator::StateCoordinator;

        let query = self.get_input_text().trim().to_string();
        debug!(target: "action", "Executing query: {}", query);

        // Use StateCoordinator to handle special commands and state changes
        let should_exit = StateCoordinator::handle_execute_query_with_refs(
            &mut self.state_container,
            &self.shadow_state,
            &query,
        )?;

        if should_exit {
            return Ok(true);
        }

        // If not a special command and not empty, execute the SQL query
        if !query.is_empty() && !query.starts_with(':') {
            if let Err(e) = self.execute_query_v2(&query) {
                self.state_container
                    .set_status_message(format!("Error executing query: {}", e));
            }
            // Don't clear input - preserve query for editing
        }

        Ok(false) // Continue running, don't exit
    }

    fn handle_buffer_action(&mut self, action: BufferAction) -> Result<bool> {
        match action {
            BufferAction::NextBuffer => {
                let message = self
                    .buffer_handler
                    .next_buffer(self.state_container.buffers_mut());
                debug!("{}", message);
                // Sync all state after buffer switch
                self.sync_after_buffer_switch();
                Ok(false)
            }
            BufferAction::PreviousBuffer => {
                let message = self
                    .buffer_handler
                    .previous_buffer(self.state_container.buffers_mut());
                debug!("{}", message);
                // Sync all state after buffer switch
                self.sync_after_buffer_switch();
                Ok(false)
            }
            BufferAction::QuickSwitch => {
                let message = self
                    .buffer_handler
                    .quick_switch(self.state_container.buffers_mut());
                debug!("{}", message);
                // Sync all state after buffer switch
                self.sync_after_buffer_switch();
                Ok(false)
            }
            BufferAction::NewBuffer => {
                let message = self
                    .buffer_handler
                    .new_buffer(self.state_container.buffers_mut(), &self.config);
                debug!("{}", message);
                Ok(false)
            }
            BufferAction::CloseBuffer => {
                let (success, message) = self
                    .buffer_handler
                    .close_buffer(self.state_container.buffers_mut());
                debug!("{}", message);
                Ok(!success) // Exit if we couldn't close (only one left)
            }
            BufferAction::ListBuffers => {
                let buffer_list = self
                    .buffer_handler
                    .list_buffers(self.state_container.buffers());
                // For now, just log the list - later we can show a popup
                for line in &buffer_list {
                    debug!("{}", line);
                }
                Ok(false)
            }
            BufferAction::SwitchToBuffer(buffer_index) => {
                let message = self
                    .buffer_handler
                    .switch_to_buffer(self.state_container.buffers_mut(), buffer_index);
                debug!("{}", message);

                // Sync all state after buffer switch
                self.sync_after_buffer_switch();

                Ok(false)
            }
        }
    }

    fn handle_expand_asterisk(&mut self) -> Result<bool> {
        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
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

    pub(crate) fn toggle_debug_mode(&mut self) {
        // Use the DebugContext trait which has all the logic
        DebugContext::toggle_debug_mode(self);
    }

    // ==================== Debug Helper Methods ====================
    // These are kept in the TUI to avoid regressions from moving data access

    /// Generate the parser debug section
    pub(crate) fn debug_generate_parser_info(&self, query: &str) -> String {
        self.hybrid_parser
            .get_detailed_debug_info(query, query.len())
    }

    fn debug_generate_navigation_state(&self) -> String {
        let mut debug_info = String::new();
        debug_info.push_str("\n========== NAVIGATION DEBUG ==========\n");
        let current_column = self.state_container.get_current_column();
        let scroll_offset = self.state_container.get_scroll_offset();
        let nav_state = self.state_container.navigation();

        debug_info.push_str(&format!("Buffer Column Position: {}\n", current_column));
        debug_info.push_str(&format!(
            "Buffer Scroll Offset: row={}, col={}\n",
            scroll_offset.0, scroll_offset.1
        ));
        debug_info.push_str(&format!(
            "NavigationState Column: {}\n",
            nav_state.selected_column
        ));
        debug_info.push_str(&format!(
            "NavigationState Row: {:?}\n",
            nav_state.selected_row
        ));
        debug_info.push_str(&format!(
            "NavigationState Scroll Offset: row={}, col={}\n",
            nav_state.scroll_offset.0, nav_state.scroll_offset.1
        ));

        // Show if synchronization is correct
        if current_column != nav_state.selected_column {
            debug_info.push_str(&format!(
                "⚠️  WARNING: Column mismatch! Buffer={}, Nav={}\n",
                current_column, nav_state.selected_column
            ));
        }
        if scroll_offset.1 != nav_state.scroll_offset.1 {
            debug_info.push_str(&format!(
                "⚠️  WARNING: Scroll column mismatch! Buffer={}, Nav={}\n",
                scroll_offset.1, nav_state.scroll_offset.1
            ));
        }

        debug_info.push_str("\n--- Navigation Flow ---\n");
        debug_info.push_str(
            "(Enable RUST_LOG=sql_cli::ui::viewport_manager=debug,navigation=debug to see flow)\n",
        );

        // Show pinned column info for navigation context
        if let Some(dataview) = self.state_container.get_buffer_dataview() {
            let pinned_count = dataview.get_pinned_columns().len();
            let pinned_names = dataview.get_pinned_column_names();
            debug_info.push_str(&format!("Pinned Column Count: {}\n", pinned_count));
            if !pinned_names.is_empty() {
                debug_info.push_str(&format!("Pinned Column Names: {:?}\n", pinned_names));
            }
            debug_info.push_str(&format!("First Scrollable Column: {}\n", pinned_count));

            // Show if current column is in pinned or scrollable area
            if current_column < pinned_count {
                debug_info.push_str(&format!(
                    "Current Position: PINNED area (column {})\n",
                    current_column
                ));
            } else {
                debug_info.push_str(&format!(
                    "Current Position: SCROLLABLE area (column {}, scrollable index {})\n",
                    current_column,
                    current_column - pinned_count
                ));
            }

            // Show display column order
            let display_columns = dataview.get_display_columns();
            debug_info.push_str(&format!("\n--- COLUMN ORDERING ---\n"));
            debug_info.push_str(&format!(
                "Display column order (first 10): {:?}\n",
                &display_columns[..display_columns.len().min(10)]
            ));
            if display_columns.len() > 10 {
                debug_info.push_str(&format!(
                    "... and {} more columns\n",
                    display_columns.len() - 10
                ));
            }

            // Find current column in display order
            if let Some(display_idx) = display_columns
                .iter()
                .position(|&idx| idx == current_column)
            {
                debug_info.push_str(&format!(
                    "Current column {} is at display index {}/{}\n",
                    current_column,
                    display_idx,
                    display_columns.len()
                ));

                // Show what happens on next move
                if display_idx + 1 < display_columns.len() {
                    let next_col = display_columns[display_idx + 1];
                    debug_info.push_str(&format!(
                        "Next 'l' press should move to column {} (display index {})\n",
                        next_col,
                        display_idx + 1
                    ));
                } else {
                    debug_info.push_str("Next 'l' press should wrap to first column\n");
                }
            } else {
                debug_info.push_str(&format!(
                    "WARNING: Current column {} not found in display order!\n",
                    current_column
                ));
            }
        }
        debug_info.push_str("==========================================\n");
        debug_info
    }

    fn debug_generate_column_search_state(&self) -> String {
        let mut debug_info = String::new();
        let show_column_search = self.shadow_state.borrow().get_mode() == AppMode::ColumnSearch
            || !self.state_container.column_search().pattern.is_empty();
        if show_column_search {
            let column_search = self.state_container.column_search();
            debug_info.push_str("\n========== COLUMN SEARCH STATE ==========\n");
            debug_info.push_str(&format!("Pattern: '{}'\n", column_search.pattern));
            debug_info.push_str(&format!(
                "Matching Columns: {} found\n",
                column_search.matching_columns.len()
            ));
            if !column_search.matching_columns.is_empty() {
                debug_info.push_str("Matches:\n");
                for (idx, (col_idx, col_name)) in column_search.matching_columns.iter().enumerate()
                {
                    let marker = if idx == column_search.current_match {
                        " <--"
                    } else {
                        ""
                    };
                    debug_info.push_str(&format!(
                        "  [{}] {} (index {}){}\n",
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
        debug_info
    }

    fn debug_generate_key_renderer_info(&self) -> String {
        let mut debug_info = String::new();
        debug_info.push_str("\n========== KEY SEQUENCE RENDERER ==========\n");
        debug_info.push_str(&format!(
            "Enabled: {}\n",
            self.key_sequence_renderer.is_enabled()
        ));
        debug_info.push_str(&format!(
            "Has Content: {}\n",
            self.key_sequence_renderer.has_content()
        ));
        debug_info.push_str(&format!(
            "Display String: '{}'\n",
            self.key_sequence_renderer.get_display()
        ));

        // Show detailed state if enabled
        if self.key_sequence_renderer.is_enabled() {
            debug_info.push_str(&format!(
                "Chord Mode: {:?}\n",
                self.key_sequence_renderer.get_chord_mode()
            ));
            debug_info.push_str(&format!(
                "Key History Size: {}\n",
                self.key_sequence_renderer.sequence_count()
            ));
            let sequences = self.key_sequence_renderer.get_sequences();
            if !sequences.is_empty() {
                debug_info.push_str("Recent Keys:\n");
                for (key, count) in sequences {
                    debug_info.push_str(&format!("  - '{}' ({} times)\n", key, count));
                }
            }
        }
        debug_info.push_str("==========================================\n");
        debug_info
    }

    pub(crate) fn debug_generate_trace_logs(&self) -> String {
        let mut debug_info = String::from("\n========== TRACE LOGS ==========\n");
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

        debug_info
    }

    /// Generate the state change logs debug section
    pub(crate) fn debug_generate_state_logs(&self) -> String {
        let mut debug_info = String::new();

        if let Some(ref debug_service) = self.debug_service {
            debug_info.push_str("\n========== STATE CHANGE LOGS ==========\n");
            debug_info.push_str("(Most recent at bottom, from DebugService)\n");
            let debug_entries = debug_service.get_entries();
            let recent = debug_entries.iter().rev().take(50).rev();
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

        debug_info
    }

    /// Extract time in milliseconds from a timing string
    pub(crate) fn debug_extract_timing(&self, s: &str) -> Option<f64> {
        crate::ui::rendering::ui_layout_utils::extract_timing_from_debug_string(s)
    }

    fn show_pretty_query(&mut self) {
        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            self.shadow_state.borrow_mut().set_mode(
                AppMode::PrettyQuery,
                buffer,
                "pretty_query_show",
            );
            let query = buffer.get_input_text();
            self.debug_widget.generate_pretty_sql(&query);
        }
    }

    /// Add global indicators like key sequence display
    fn add_global_indicators(&self, spans: &mut Vec<Span>) {
        if self.key_sequence_renderer.has_content() {
            let key_display = self.key_sequence_renderer.get_display();
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
    }

    fn add_query_source_indicator(&self, spans: &mut Vec<Span>) {
        if let Some(source) = self.state_container.get_last_query_source() {
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
    }

    fn add_case_sensitivity_indicator(&self, spans: &mut Vec<Span>) {
        let case_insensitive = self.state_container.is_case_insensitive();
        if case_insensitive {
            spans.push(Span::raw(" | "));
            let icon = self.config.display.icons.case_insensitive.clone();
            spans.push(Span::styled(
                format!("{} CASE", icon),
                Style::default().fg(Color::Cyan),
            ));
        }
    }

    fn add_column_packing_indicator(&self, spans: &mut Vec<Span>) {
        if let Some(ref viewport_manager) = *self.viewport_manager.borrow() {
            let packing_mode = viewport_manager.get_packing_mode();
            spans.push(Span::raw(" | "));
            let (text, color) = match packing_mode {
                ColumnPackingMode::DataFocus => ("DATA", Color::Cyan),
                ColumnPackingMode::HeaderFocus => ("HEADER", Color::Yellow),
                ColumnPackingMode::Balanced => ("BALANCED", Color::Green),
            };
            spans.push(Span::styled(text, Style::default().fg(color)));
        }
    }

    fn add_status_message(&self, spans: &mut Vec<Span>) {
        let status_msg = self.state_container.get_buffer_status_message();
        if !status_msg.is_empty() {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                status_msg,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
    }

    fn get_help_text_for_mode(&self) -> &str {
        match self.shadow_state.borrow().get_mode() {
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
        }
    }

    fn add_shadow_state_display(&self, spans: &mut Vec<Span>) {
        let shadow_display = self.shadow_state.borrow().status_display();
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            shadow_display,
            Style::default().fg(Color::Cyan),
        ));
    }

    /// Add right-aligned help text if space allows
    fn add_help_text_display<'a>(&self, spans: &mut Vec<Span<'a>>, help_text: &'a str, area: Rect) {
        let current_length: usize = spans.iter().map(|s| s.content.len()).sum();
        let available_width = area.width.saturating_sub(TABLE_BORDER_WIDTH) as usize;
        let help_length = help_text.len();

        if current_length + help_length + 3 < available_width {
            let padding = available_width - current_length - help_length - 3;
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                help_text,
                Style::default().fg(Color::DarkGray),
            ));
        }
    }
}

// Implement ActionHandlerContext trait for EnhancedTuiApp
impl ActionHandlerContext for EnhancedTuiApp {
    // Navigation methods - delegate to trait implementations
    fn previous_row(&mut self) {
        <Self as NavigationBehavior>::previous_row(self);

        // Update TableWidgetManager for rendering
        let current_row = self.state_container.navigation().selected_row;
        let current_col = self.state_container.navigation().selected_column;
        self.table_widget_manager
            .borrow_mut()
            .navigate_to(current_row, current_col);
        info!(target: "navigation", "previous_row: Updated TableWidgetManager to ({}, {})", current_row, current_col);
    }

    fn next_row(&mut self) {
        info!(target: "navigation", "next_row called - calling NavigationBehavior::next_row");
        <Self as NavigationBehavior>::next_row(self);

        // Update TableWidgetManager for rendering
        let current_row = self.state_container.navigation().selected_row;
        let current_col = self.state_container.navigation().selected_column;
        self.table_widget_manager
            .borrow_mut()
            .navigate_to(current_row, current_col);
        info!(target: "navigation", "next_row: Updated TableWidgetManager to ({}, {})", current_row, current_col);
    }

    fn move_column_left(&mut self) {
        <Self as ColumnBehavior>::move_column_left(self);

        // Update TableWidgetManager for rendering
        let current_row = self.state_container.navigation().selected_row;
        let current_col = self.state_container.navigation().selected_column;
        self.table_widget_manager
            .borrow_mut()
            .navigate_to(current_row, current_col);
        info!(target: "navigation", "move_column_left: Updated TableWidgetManager to ({}, {})", current_row, current_col);
    }

    fn move_column_right(&mut self) {
        <Self as ColumnBehavior>::move_column_right(self);

        // Update TableWidgetManager for rendering
        let current_row = self.state_container.navigation().selected_row;
        let current_col = self.state_container.navigation().selected_column;
        self.table_widget_manager
            .borrow_mut()
            .navigate_to(current_row, current_col);
        info!(target: "navigation", "move_column_right: Updated TableWidgetManager to ({}, {})", current_row, current_col);
    }

    fn page_up(&mut self) {
        <Self as NavigationBehavior>::page_up(self);
    }

    fn page_down(&mut self) {
        <Self as NavigationBehavior>::page_down(self);
    }

    fn goto_first_row(&mut self) {
        use crate::ui::state::state_coordinator::StateCoordinator;

        // Always perform the normal goto first row behavior
        <Self as NavigationBehavior>::goto_first_row(self);

        // Use StateCoordinator to handle vim search coordination
        StateCoordinator::goto_first_row_with_refs(
            &mut self.state_container,
            Some(&self.vim_search_adapter),
            Some(&self.viewport_manager),
        );
    }

    fn goto_last_row(&mut self) {
        use crate::ui::state::state_coordinator::StateCoordinator;

        // Perform the normal goto last row behavior
        <Self as NavigationBehavior>::goto_last_row(self);

        // Use StateCoordinator for additional coordination
        StateCoordinator::goto_last_row_with_refs(&mut self.state_container);
    }

    fn goto_first_column(&mut self) {
        <Self as ColumnBehavior>::goto_first_column(self);
    }

    fn goto_last_column(&mut self) {
        <Self as ColumnBehavior>::goto_last_column(self);
    }

    fn goto_row(&mut self, row: usize) {
        use crate::ui::state::state_coordinator::StateCoordinator;

        // Perform the normal goto line behavior
        <Self as NavigationBehavior>::goto_line(self, row + 1); // Convert to 1-indexed

        // Use StateCoordinator for additional coordination
        StateCoordinator::goto_row_with_refs(&mut self.state_container, row);
    }

    fn goto_column(&mut self, col: usize) {
        // For now, implement basic column navigation
        // TODO: Implement proper goto_column functionality
        let current_col = self.state_container.get_current_column();
        if col < current_col {
            for _ in 0..(current_col - col) {
                <Self as ColumnBehavior>::move_column_left(self);
            }
        } else if col > current_col {
            for _ in 0..(col - current_col) {
                <Self as ColumnBehavior>::move_column_right(self);
            }
        }
    }

    // Mode and UI state
    fn set_mode(&mut self, mode: AppMode) {
        // Use the proper synchronization method that updates both buffer and shadow_state
        self.set_mode_via_shadow_state(mode, "action_handler");
    }

    fn get_mode(&self) -> AppMode {
        self.shadow_state.borrow().get_mode()
    }

    fn set_status_message(&mut self, message: String) {
        self.state_container.set_status_message(message);
    }

    // Column operations - delegate to trait implementations
    fn toggle_column_pin(&mut self) {
        // Call the existing toggle_column_pin implementation directly
        self.toggle_column_pin_impl();
    }

    fn hide_current_column(&mut self) {
        <Self as ColumnBehavior>::hide_current_column(self);
    }

    fn unhide_all_columns(&mut self) {
        <Self as ColumnBehavior>::unhide_all_columns(self);
    }

    fn clear_all_pinned_columns(&mut self) {
        // Call the existing clear_all_pinned_columns implementation directly
        self.clear_all_pinned_columns_impl();
    }

    // Export operations
    fn export_to_csv(&mut self) {
        // For now, just set a status message - actual implementation will be added later
        self.state_container
            .set_status_message("CSV export not yet implemented".to_string());
    }

    fn export_to_json(&mut self) {
        // For now, just set a status message - actual implementation will be added later
        self.state_container
            .set_status_message("JSON export not yet implemented".to_string());
    }

    // Yank operations - delegate to trait implementations
    fn yank_cell(&mut self) {
        YankBehavior::yank_cell(self);
    }

    fn yank_row(&mut self) {
        YankBehavior::yank_row(self);
    }

    fn yank_column(&mut self) {
        YankBehavior::yank_column(self);
    }

    fn yank_all(&mut self) {
        YankBehavior::yank_all(self);
    }

    fn yank_query(&mut self) {
        YankBehavior::yank_query(self);
    }

    // Toggle operations
    fn toggle_selection_mode(&mut self) {
        self.state_container.toggle_selection_mode();
        let new_mode = self.state_container.get_selection_mode();
        let msg = match new_mode {
            SelectionMode::Cell => "Cell mode - Navigate to select individual cells",
            SelectionMode::Row => "Row mode - Navigate to select rows",
            SelectionMode::Column => "Column mode - Navigate to select columns",
        };
        self.state_container.set_status_message(msg.to_string());
    }

    fn toggle_row_numbers(&mut self) {
        let current = self.state_container.is_show_row_numbers();
        self.state_container.set_show_row_numbers(!current);
        let message = if !current {
            "Row numbers: ON (showing line numbers)".to_string()
        } else {
            "Row numbers: OFF".to_string()
        };
        self.state_container.set_status_message(message);
        // Recalculate column widths with new mode
        self.calculate_optimal_column_widths();
    }

    fn toggle_compact_mode(&mut self) {
        let current_mode = self.state_container.is_compact_mode();
        self.state_container.set_compact_mode(!current_mode);
        let message = if !current_mode {
            "Compact mode enabled"
        } else {
            "Compact mode disabled"
        };
        self.state_container.set_status_message(message.to_string());
    }

    fn toggle_case_insensitive(&mut self) {
        let current = self.state_container.is_case_insensitive();
        self.state_container.set_case_insensitive(!current);
        self.state_container.set_status_message(format!(
            "Case-insensitive string comparisons: {}",
            if !current { "ON" } else { "OFF" }
        ));
    }

    fn toggle_key_indicator(&mut self) {
        let enabled = !self.key_indicator.enabled;
        self.key_indicator.set_enabled(enabled);
        self.key_sequence_renderer.set_enabled(enabled);
        self.state_container.set_status_message(format!(
            "Key press indicator {}",
            if enabled { "enabled" } else { "disabled" }
        ));
    }

    // Clear operations
    fn clear_filter(&mut self) {
        // Check if we have an active filter to clear
        if let Some(dataview) = self.state_container.get_buffer_dataview() {
            if dataview.has_filter() {
                // Clear the filter
                if let Some(dataview_mut) = self.state_container.get_buffer_dataview_mut() {
                    dataview_mut.clear_filter();
                    self.state_container
                        .set_status_message("Filter cleared".to_string());
                }

                // Update ViewportManager after clearing filter
                // Sync the dataview to both managers
                self.sync_dataview_to_managers();
            } else {
                self.state_container
                    .set_status_message("No active filter to clear".to_string());
            }
        } else {
            self.state_container
                .set_status_message("No data loaded".to_string());
        }
    }

    fn clear_line(&mut self) {
        self.state_container.clear_line();
    }

    // Mode operations
    fn start_search(&mut self) {
        self.start_vim_search();
    }

    fn start_column_search(&mut self) {
        self.enter_search_mode(SearchMode::ColumnSearch);
    }

    fn start_filter(&mut self) {
        self.enter_search_mode(SearchMode::Filter);
    }

    fn start_fuzzy_filter(&mut self) {
        self.enter_search_mode(SearchMode::FuzzyFilter);
    }

    fn exit_current_mode(&mut self) {
        // Handle escape based on current mode
        let mode = self.shadow_state.borrow().get_mode();
        match mode {
            AppMode::Results => {
                // VimSearchAdapter now handles Escape in Results mode when search is active
                // If we get here, it means search wasn't active, so switch to Command mode
                self.state_container.set_mode(AppMode::Command);
            }
            AppMode::Command => {
                self.state_container.set_mode(AppMode::Results);
            }
            AppMode::Help => {
                self.state_container.set_mode(AppMode::Results);
            }
            AppMode::JumpToRow => {
                self.state_container.set_mode(AppMode::Results);
                <Self as InputBehavior>::clear_jump_to_row_input(self);
                // Clear jump-to-row state (can mutate directly now)
                self.state_container.jump_to_row_mut().is_active = false;
                self.state_container
                    .set_status_message("Jump to row cancelled".to_string());
            }
            _ => {
                // For any other mode, return to Results
                self.state_container.set_mode(AppMode::Results);
            }
        }
    }

    fn toggle_debug_mode(&mut self) {
        // Use the DebugContext trait
        <Self as DebugContext>::toggle_debug_mode(self);
    }

    // Column arrangement operations
    fn move_current_column_left(&mut self) {
        <Self as ColumnBehavior>::move_current_column_left(self);
    }

    fn move_current_column_right(&mut self) {
        <Self as ColumnBehavior>::move_current_column_right(self);
    }

    // Search navigation
    fn next_search_match(&mut self) {
        self.vim_search_next();
    }

    fn previous_search_match(&mut self) {
        if self.vim_search_adapter.borrow().is_active()
            || self.vim_search_adapter.borrow().get_pattern().is_some()
        {
            self.vim_search_previous();
        }
    }

    // Statistics and display
    fn show_column_statistics(&mut self) {
        self.calculate_column_statistics();
    }

    fn cycle_column_packing(&mut self) {
        let message = {
            let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
            let viewport_manager = viewport_manager_borrow
                .as_mut()
                .expect("ViewportManager must exist");
            let new_mode = viewport_manager.cycle_packing_mode();
            format!("Column packing: {}", new_mode.display_name())
        };
        self.state_container.set_status_message(message);
    }

    // Viewport navigation
    fn navigate_to_viewport_top(&mut self) {
        let result = {
            let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
            if let Some(ref mut viewport_manager) = *viewport_manager_borrow {
                Some(viewport_manager.navigate_to_viewport_top())
            } else {
                None
            }
        };

        if let Some(result) = result {
            // ViewportManager has updated, sync NavigationState from it
            self.sync_navigation_with_viewport();

            // Update buffer's selected row
            self.state_container
                .set_selected_row(Some(result.row_position));

            // Update buffer's scroll offset
            if result.viewport_changed {
                let scroll_offset = self.state_container.navigation().scroll_offset;
                self.state_container.set_scroll_offset(scroll_offset);
            }
        }
    }

    fn navigate_to_viewport_middle(&mut self) {
        let result = {
            let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
            if let Some(ref mut viewport_manager) = *viewport_manager_borrow {
                Some(viewport_manager.navigate_to_viewport_middle())
            } else {
                None
            }
        };

        if let Some(result) = result {
            // ViewportManager has updated, sync NavigationState from it
            self.sync_navigation_with_viewport();

            // Update buffer's selected row
            self.state_container
                .set_selected_row(Some(result.row_position));

            // Update buffer's scroll offset
            if result.viewport_changed {
                let scroll_offset = self.state_container.navigation().scroll_offset;
                self.state_container.set_scroll_offset(scroll_offset);
            }
        }
    }

    fn navigate_to_viewport_bottom(&mut self) {
        let result = {
            let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
            if let Some(ref mut viewport_manager) = *viewport_manager_borrow {
                Some(viewport_manager.navigate_to_viewport_bottom())
            } else {
                None
            }
        };

        if let Some(result) = result {
            // ViewportManager has updated, sync NavigationState from it
            self.sync_navigation_with_viewport();

            // Update buffer's selected row
            self.state_container
                .set_selected_row(Some(result.row_position));

            // Update buffer's scroll offset
            if result.viewport_changed {
                let scroll_offset = self.state_container.navigation().scroll_offset;
                self.state_container.set_scroll_offset(scroll_offset);
            }
        }
    }

    // Input and text editing methods
    fn move_input_cursor_left(&mut self) {
        self.state_container.move_input_cursor_left();
    }

    fn move_input_cursor_right(&mut self) {
        self.state_container.move_input_cursor_right();
    }

    fn move_input_cursor_home(&mut self) {
        self.state_container.set_input_cursor_position(0);
    }

    fn move_input_cursor_end(&mut self) {
        let text_len = self.state_container.get_input_text().chars().count();
        self.state_container.set_input_cursor_position(text_len);
    }

    fn backspace(&mut self) {
        self.state_container.backspace();
    }

    fn delete(&mut self) {
        self.state_container.delete();
    }

    fn undo(&mut self) {
        self.state_container.perform_undo();
    }

    fn redo(&mut self) {
        self.state_container.perform_redo();
    }

    fn start_jump_to_row(&mut self) {
        self.state_container.set_mode(AppMode::JumpToRow);
        self.shadow_state
            .borrow_mut()
            .observe_mode_change(AppMode::JumpToRow, "jump_to_row_requested");
        <Self as InputBehavior>::clear_jump_to_row_input(self);

        // Set jump-to-row state as active (can mutate directly now)
        self.state_container.jump_to_row_mut().is_active = true;

        self.state_container
            .set_status_message("Enter row number (1-based):".to_string());
    }

    fn clear_jump_to_row_input(&mut self) {
        <Self as InputBehavior>::clear_jump_to_row_input(self);
    }

    fn toggle_cursor_lock(&mut self) {
        // Toggle cursor lock in ViewportManager
        let is_locked = {
            let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
            if let Some(ref mut viewport_manager) = *viewport_manager_borrow {
                viewport_manager.toggle_cursor_lock();
                Some(viewport_manager.is_cursor_locked())
            } else {
                None
            }
        };

        if let Some(is_locked) = is_locked {
            let msg = if is_locked {
                "Cursor lock ON - cursor stays in viewport position while scrolling"
            } else {
                "Cursor lock OFF"
            };
            self.state_container.set_status_message(msg.to_string());

            // Log for shadow state learning (not tracking as state change yet)
            info!(target: "shadow_state",
                "Cursor lock toggled: {} (in {:?} mode)",
                if is_locked { "ON" } else { "OFF" },
                self.shadow_state.borrow().get_mode()
            );
        }
    }

    fn toggle_viewport_lock(&mut self) {
        // Toggle viewport lock in ViewportManager
        let is_locked = {
            let mut viewport_manager_borrow = self.viewport_manager.borrow_mut();
            if let Some(ref mut viewport_manager) = *viewport_manager_borrow {
                viewport_manager.toggle_viewport_lock();
                Some(viewport_manager.is_viewport_locked())
            } else {
                None
            }
        };

        if let Some(is_locked) = is_locked {
            let msg = if is_locked {
                "Viewport lock ON - navigation constrained to current viewport"
            } else {
                "Viewport lock OFF"
            };
            self.state_container.set_status_message(msg.to_string());

            // Log for shadow state learning (not tracking as state change yet)
            info!(target: "shadow_state",
                "Viewport lock toggled: {} (in {:?} mode)",
                if is_locked { "ON" } else { "OFF" },
                self.shadow_state.borrow().get_mode()
            );
        }
    }

    // Debug and development operations
    fn show_debug_info(&mut self) {
        <Self as DebugContext>::toggle_debug_mode(self);
    }

    fn show_pretty_query(&mut self) {
        self.show_pretty_query();
    }

    fn show_help(&mut self) {
        self.state_container.set_help_visible(true);
        self.set_mode_via_shadow_state(AppMode::Help, "help_requested");
        self.help_widget.on_enter();
    }

    // Text editing operations
    fn kill_line(&mut self) {
        use crate::ui::traits::input_ops::InputBehavior;
        InputBehavior::kill_line(self);
        let message = if !self.state_container.is_kill_ring_empty() {
            let kill_ring = self.state_container.get_kill_ring();
            format!(
                "Killed to end of line - {} chars in kill ring",
                kill_ring.len()
            )
        } else {
            "Kill line - nothing to kill".to_string()
        };
        self.state_container.set_status_message(message);
    }

    fn kill_line_backward(&mut self) {
        use crate::ui::traits::input_ops::InputBehavior;
        InputBehavior::kill_line_backward(self);
        let message = if !self.state_container.is_kill_ring_empty() {
            let kill_ring = self.state_container.get_kill_ring();
            format!(
                "Killed to beginning of line - {} chars in kill ring",
                kill_ring.len()
            )
        } else {
            "Kill line backward - nothing to kill".to_string()
        };
        self.state_container.set_status_message(message);
    }

    fn delete_word_backward(&mut self) {
        use crate::ui::traits::input_ops::InputBehavior;
        InputBehavior::delete_word_backward(self);
    }

    fn delete_word_forward(&mut self) {
        use crate::ui::traits::input_ops::InputBehavior;
        InputBehavior::delete_word_forward(self);
    }

    fn expand_asterisk(&mut self) {
        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            if buffer.expand_asterisk(&self.hybrid_parser) {
                // Sync for rendering if needed
                if buffer.get_edit_mode() == EditMode::SingleLine {
                    let text = buffer.get_input_text();
                    let cursor = buffer.get_input_cursor_position();
                    self.set_input_text_with_cursor(text, cursor);
                }
            }
        }
    }

    fn expand_asterisk_visible(&mut self) {
        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            if buffer.expand_asterisk_visible() {
                // Sync for rendering if needed
                if buffer.get_edit_mode() == EditMode::SingleLine {
                    let text = buffer.get_input_text();
                    let cursor = buffer.get_input_cursor_position();
                    self.set_input_text_with_cursor(text, cursor);
                }
            }
        }
    }

    fn previous_history_command(&mut self) {
        let history_entries = self
            .state_container
            .command_history()
            .get_navigation_entries();
        let history_commands: Vec<String> =
            history_entries.iter().map(|e| e.command.clone()).collect();

        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            if buffer.navigate_history_up(&history_commands) {
                self.sync_all_input_states();
                self.state_container
                    .set_status_message("Previous command from history".to_string());
            }
        }
    }

    fn next_history_command(&mut self) {
        let history_entries = self
            .state_container
            .command_history()
            .get_navigation_entries();
        let history_commands: Vec<String> =
            history_entries.iter().map(|e| e.command.clone()).collect();

        if let Some(buffer) = self.state_container.buffers_mut().current_mut() {
            if buffer.navigate_history_down(&history_commands) {
                self.sync_all_input_states();
                self.state_container
                    .set_status_message("Next command from history".to_string());
            }
        }
    }
}

// Implement NavigationBehavior trait for EnhancedTuiApp
impl NavigationBehavior for EnhancedTuiApp {
    fn viewport_manager(&self) -> &RefCell<Option<ViewportManager>> {
        &self.viewport_manager
    }

    fn buffer_mut(&mut self) -> &mut dyn BufferAPI {
        self.state_container
            .current_buffer_mut()
            .expect("Buffer should exist")
    }

    fn buffer(&self) -> &dyn BufferAPI {
        self.state_container
            .current_buffer()
            .expect("Buffer should exist")
    }

    fn state_container(&self) -> &AppStateContainer {
        &self.state_container
    }

    fn state_container_mut(&mut self) -> &mut AppStateContainer {
        &mut self.state_container
    }

    fn get_row_count(&self) -> usize {
        self.get_row_count()
    }

    fn set_mode_with_sync(&mut self, mode: AppMode, trigger: &str) {
        // Use the existing method that properly synchronizes both buffer and shadow_state
        self.set_mode_via_shadow_state(mode, trigger);
    }
}

// Implement ColumnBehavior trait for EnhancedTuiApp
impl ColumnBehavior for EnhancedTuiApp {
    fn viewport_manager(&self) -> &RefCell<Option<ViewportManager>> {
        &self.viewport_manager
    }

    fn buffer_mut(&mut self) -> &mut dyn BufferAPI {
        self.state_container
            .current_buffer_mut()
            .expect("Buffer should exist")
    }

    fn buffer(&self) -> &dyn BufferAPI {
        self.state_container
            .current_buffer()
            .expect("Buffer should exist")
    }

    fn state_container(&self) -> &AppStateContainer {
        &self.state_container
    }

    fn is_in_results_mode(&self) -> bool {
        self.shadow_state.borrow().is_in_results_mode()
    }
}

impl InputBehavior for EnhancedTuiApp {
    fn buffer_manager(&mut self) -> &mut BufferManager {
        self.state_container.buffers_mut()
    }

    fn cursor_manager(&mut self) -> &mut CursorManager {
        &mut self.cursor_manager
    }

    fn set_input_text_with_cursor(&mut self, text: String, cursor: usize) {
        self.set_input_text_with_cursor(text, cursor)
    }

    fn state_container(&self) -> &AppStateContainer {
        &self.state_container
    }

    fn state_container_mut(&mut self) -> &mut AppStateContainer {
        &mut self.state_container
    }

    fn buffer_mut(&mut self) -> &mut dyn BufferAPI {
        self.state_container
            .current_buffer_mut()
            .expect("Buffer should exist")
    }

    fn set_mode_with_sync(&mut self, mode: AppMode, trigger: &str) {
        // Use the existing method that properly synchronizes both buffer and shadow_state
        self.set_mode_via_shadow_state(mode, trigger);
    }
}

impl YankBehavior for EnhancedTuiApp {
    fn buffer(&self) -> &dyn BufferAPI {
        self.state_container
            .current_buffer()
            .expect("Buffer should exist")
    }

    fn buffer_mut(&mut self) -> &mut dyn BufferAPI {
        self.state_container
            .current_buffer_mut()
            .expect("Buffer should exist")
    }

    fn state_container(&self) -> &AppStateContainer {
        &self.state_container
    }

    fn set_status_message(&mut self, message: String) {
        self.state_container.set_status_message(message)
    }

    fn set_error_status(&mut self, prefix: &str, error: anyhow::Error) {
        self.set_error_status(prefix, error)
    }
}

impl BufferManagementBehavior for EnhancedTuiApp {
    fn buffer_manager(&mut self) -> &mut BufferManager {
        self.state_container.buffers_mut()
    }

    fn buffer_handler(&mut self) -> &mut BufferHandler {
        &mut self.buffer_handler
    }

    fn buffer(&self) -> &dyn BufferAPI {
        self.state_container
            .current_buffer()
            .expect("Buffer should exist")
    }

    fn buffer_mut(&mut self) -> &mut dyn BufferAPI {
        self.state_container
            .current_buffer_mut()
            .expect("Buffer should exist")
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn cursor_manager(&mut self) -> &mut CursorManager {
        &mut self.cursor_manager
    }

    fn set_input_text_with_cursor(&mut self, text: String, cursor: usize) {
        self.set_input_text_with_cursor(text, cursor)
    }

    fn next_buffer(&mut self) -> String {
        // Save current viewport state to current buffer before switching
        self.save_viewport_to_current_buffer();

        let result = self
            .buffer_handler
            .next_buffer(self.state_container.buffers_mut());

        // Sync all state after buffer switch
        self.sync_after_buffer_switch();

        result
    }

    fn previous_buffer(&mut self) -> String {
        // Save current viewport state to current buffer before switching
        self.save_viewport_to_current_buffer();

        let result = self
            .buffer_handler
            .previous_buffer(self.state_container.buffers_mut());

        // Sync all state after buffer switch
        self.sync_after_buffer_switch();

        result
    }

    fn quick_switch_buffer(&mut self) -> String {
        // Save current viewport state to current buffer before switching
        self.save_viewport_to_current_buffer();

        let result = self
            .buffer_handler
            .quick_switch(self.state_container.buffers_mut());

        // Sync all state after buffer switch
        self.sync_after_buffer_switch();

        result
    }

    fn close_buffer(&mut self) -> (bool, String) {
        self.buffer_handler
            .close_buffer(self.state_container.buffers_mut())
    }

    fn switch_to_buffer(&mut self, index: usize) -> String {
        // Save current viewport state to current buffer before switching
        self.save_viewport_to_current_buffer();

        // Switch buffer
        let result = self
            .buffer_handler
            .switch_to_buffer(self.state_container.buffers_mut(), index);

        // Sync all state after buffer switch
        self.sync_after_buffer_switch();

        result
    }

    fn buffer_count(&self) -> usize {
        self.state_container.buffers().all_buffers().len()
    }

    fn current_buffer_index(&self) -> usize {
        self.state_container.buffers().current_index()
    }
}

pub fn run_enhanced_tui_multi(api_url: &str, data_files: Vec<&str>) -> Result<()> {
    let app = if !data_files.is_empty() {
        // Use ApplicationOrchestrator for clean data source separation
        use crate::services::ApplicationOrchestrator;

        // Get config for orchestrator setup
        let config = Config::default();
        let orchestrator = ApplicationOrchestrator::new(
            config.behavior.case_insensitive_default,
            config.behavior.hide_empty_columns,
        );

        // Load the first file through the orchestrator
        let mut app = orchestrator.create_tui_with_file(data_files[0])?;

        // Load additional files into separate buffers
        for file_path in data_files.iter().skip(1) {
            if let Err(e) = orchestrator.load_additional_file(&mut app, file_path) {
                app.state_container
                    .set_status_message(format!("Error loading {}: {}", file_path, e));
                continue;
            }
        }

        // Switch back to the first buffer if we loaded multiple
        if data_files.len() > 1 {
            app.state_container.buffers_mut().switch_to(0);
            // Sync all state after buffer switch
            app.sync_after_buffer_switch();
            app.state_container.set_status_message(format!(
                "Loaded {} files into separate buffers. Use Alt+Tab to switch.",
                data_files.len()
            ));
        } else if data_files.len() == 1 {
            // Even for single file, ensure parser is initialized
            app.update_parser_for_current_buffer();
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
