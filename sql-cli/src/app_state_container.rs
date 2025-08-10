use crate::buffer::{AppMode, BufferManager};
use crate::help_widget::HelpWidget;
use crate::history::CommandHistory;
use crate::history_widget::HistoryWidget;
use crate::search_modes_widget::SearchModesWidget;
use crate::stats_widget::StatsWidget;
// TODO: Add DebugWidget when it implements DebugInfoProvider
// use crate::debug_widget::DebugWidget;
use crate::widget_traits::DebugInfoProvider;
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

/// Represents input state for command editing
#[derive(Debug, Clone)]
pub struct InputState {
    pub text: String,
    pub cursor_position: usize,
    pub last_executed_query: String,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            last_executed_query: String::new(),
        }
    }

    pub fn clear(&mut self) {
        let old_text = self.text.clone();
        self.text.clear();
        self.cursor_position = 0;
        // Note: This is on InputState, so we don't have access to debug_service here
        // Logging will need to be done at the AppStateContainer level
    }

    pub fn set_text(&mut self, text: String) {
        let _old_text = self.text.clone();
        // TODO: Add logging when log crate is available
        // info!(target: "state", "InputState::set_text() - '{}' -> '{}'", old_text, text);
        self.cursor_position = text.len();
        self.text = text;
    }

    pub fn set_text_with_cursor(&mut self, text: String, cursor: usize) {
        let _old_text = self.text.clone();
        let _old_cursor = self.cursor_position;
        // TODO: Add logging when log crate is available
        // info!(target: "state", "InputState::set_text_with_cursor() - text: '{}' -> '{}', cursor: {} -> {}",
        //       old_text, text, old_cursor, cursor);
        self.text = text;
        self.cursor_position = cursor;
    }
}

/// Search state for regular search
#[derive(Debug, Clone)]
pub struct SearchState {
    pub pattern: String,
    pub matches: Vec<(usize, usize, usize, usize)>, // (row_start, col_start, row_end, col_end)
    pub current_match: usize,
    pub is_active: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            matches: Vec::new(),
            current_match: 0,
            is_active: false,
        }
    }

    pub fn clear(&mut self) {
        // TODO: Add logging when log crate is available
        // info!(target: "state", "SearchState::clear() - had {} matches for pattern '{}'",
        //       self.matches.len(), self.pattern);
        self.pattern.clear();
        self.matches.clear();
        self.current_match = 0;
        self.is_active = false;
    }
}

/// Filter state for filtering results
#[derive(Debug, Clone)]
pub struct FilterState {
    pub pattern: String,
    pub filtered_indices: Vec<usize>,
    pub is_active: bool,
    pub case_insensitive: bool,
}

impl FilterState {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            filtered_indices: Vec::new(),
            is_active: false,
            case_insensitive: true,
        }
    }

    pub fn clear(&mut self) {
        // TODO: Add logging when log crate is available
        // info!(target: "state", "FilterState::clear() - had {} filtered rows for pattern '{}'",
        //       self.filtered_indices.len(), self.pattern);
        self.pattern.clear();
        self.filtered_indices.clear();
        self.is_active = false;
    }
}

/// Column search state
#[derive(Debug, Clone)]
pub struct ColumnSearchState {
    pub pattern: String,
    pub matching_columns: Vec<(usize, String)>,
    pub current_match: usize,
    pub is_active: bool,
}

impl ColumnSearchState {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            matching_columns: Vec::new(),
            current_match: 0,
            is_active: false,
        }
    }

    pub fn clear(&mut self) {
        // TODO: Add logging when log crate is available
        // info!(target: "state", "ColumnSearchState::clear() - had {} matching columns for pattern '{}'",
        //       self.matching_columns.len(), self.pattern);
        self.pattern.clear();
        self.matching_columns.clear();
        self.current_match = 0;
        self.is_active = false;
    }
}

/// Cache list state
#[derive(Debug, Clone)]
pub struct CacheListState {
    pub selected_index: usize,
    pub cache_names: Vec<String>,
}

impl CacheListState {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            cache_names: Vec::new(),
        }
    }
}

/// Column stats state
#[derive(Debug, Clone)]
pub struct ColumnStatsState {
    pub column_index: usize,
    pub is_visible: bool,
}

impl ColumnStatsState {
    pub fn new() -> Self {
        Self {
            column_index: 0,
            is_visible: false,
        }
    }
}

/// Jump to row state
#[derive(Debug, Clone)]
pub struct JumpToRowState {
    pub input: String,
    pub is_active: bool,
}

impl JumpToRowState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            is_active: false,
        }
    }
}

/// Container for all widget states
pub struct WidgetStates {
    pub search_modes: SearchModesWidget,
    pub history: Option<HistoryWidget>, // Will be initialized with CommandHistory later
    pub help: HelpWidget,
    pub stats: StatsWidget,
    // pub debug: DebugWidget, // TODO: Add when DebugInfoProvider is implemented
}

impl WidgetStates {
    pub fn new() -> Self {
        Self {
            search_modes: SearchModesWidget::new(),
            history: None, // Will be set when CommandHistory is available
            help: HelpWidget::new(),
            stats: StatsWidget::new(),
            // debug: DebugWidget::new(), // TODO: Add when available
        }
    }

    pub fn set_history(&mut self, history: HistoryWidget) {
        self.history = Some(history);
    }
}

/// Results cache for storing query results
#[derive(Debug, Clone)]
pub struct ResultsCache {
    cache: HashMap<String, Vec<Vec<String>>>,
    max_size: usize,
}

impl ResultsCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<Vec<String>>> {
        self.cache.get(key)
    }

    pub fn insert(&mut self, key: String, value: Vec<Vec<String>>) {
        if self.cache.len() >= self.max_size {
            // Remove oldest entry (simplified - in practice use LRU)
            if let Some(first_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&first_key);
            }
        }
        self.cache.insert(key, value);
    }
}

/// Main application state container
pub struct AppStateContainer {
    // Document/Buffer state
    buffers: BufferManager,
    current_buffer_id: usize,

    // Input state
    command_input: InputState,

    // Search/Filter states
    search: SearchState,
    filter: FilterState,
    column_search: ColumnSearchState,

    // Widget states
    widgets: WidgetStates,

    // UI states
    cache_list: CacheListState,
    column_stats: ColumnStatsState,
    jump_to_row: JumpToRowState,

    // History
    command_history: CommandHistory,

    // Results cache
    results_cache: ResultsCache,

    // Mode stack for nested modes
    mode_stack: Vec<AppMode>,

    // Debug/logging
    debug_enabled: bool,
    debug_service: RefCell<Option<crate::debug_service::DebugService>>,

    // UI visibility flags
    show_help: bool,
}

impl AppStateContainer {
    pub fn new(buffers: BufferManager) -> Result<Self> {
        let command_history = CommandHistory::new()?;
        let mut widgets = WidgetStates::new();
        widgets.set_history(HistoryWidget::new(command_history.clone()));

        Ok(Self {
            buffers,
            current_buffer_id: 0,
            command_input: InputState::new(),
            search: SearchState::new(),
            filter: FilterState::new(),
            column_search: ColumnSearchState::new(),
            widgets,
            cache_list: CacheListState::new(),
            column_stats: ColumnStatsState::new(),
            jump_to_row: JumpToRowState::new(),
            command_history,
            results_cache: ResultsCache::new(100),
            mode_stack: vec![AppMode::Command],
            debug_enabled: false,
            debug_service: RefCell::new(None), // Will be set later via set_debug_service
            show_help: false,
        })
    }

    // Buffer access
    pub fn current_buffer(&self) -> Option<&crate::buffer::Buffer> {
        self.buffers.current()
    }

    pub fn current_buffer_mut(&mut self) -> Option<&mut crate::buffer::Buffer> {
        self.buffers.current_mut()
    }

    pub fn buffers(&self) -> &BufferManager {
        &self.buffers
    }

    pub fn buffers_mut(&mut self) -> &mut BufferManager {
        &mut self.buffers
    }

    // Input state access
    pub fn command_input(&self) -> &InputState {
        &self.command_input
    }

    pub fn command_input_mut(&mut self) -> &mut InputState {
        &mut self.command_input
    }

    // Search/Filter state access
    pub fn search(&self) -> &SearchState {
        &self.search
    }

    pub fn search_mut(&mut self) -> &mut SearchState {
        &mut self.search
    }

    pub fn filter(&self) -> &FilterState {
        &self.filter
    }

    pub fn filter_mut(&mut self) -> &mut FilterState {
        &mut self.filter
    }

    pub fn column_search(&self) -> &ColumnSearchState {
        &self.column_search
    }

    pub fn column_search_mut(&mut self) -> &mut ColumnSearchState {
        &mut self.column_search
    }

    // Widget access
    pub fn widgets(&self) -> &WidgetStates {
        &self.widgets
    }

    pub fn widgets_mut(&mut self) -> &mut WidgetStates {
        &mut self.widgets
    }

    // UI state access
    pub fn cache_list(&self) -> &CacheListState {
        &self.cache_list
    }

    pub fn cache_list_mut(&mut self) -> &mut CacheListState {
        &mut self.cache_list
    }

    pub fn column_stats(&self) -> &ColumnStatsState {
        &self.column_stats
    }

    pub fn column_stats_mut(&mut self) -> &mut ColumnStatsState {
        &mut self.column_stats
    }

    pub fn jump_to_row(&self) -> &JumpToRowState {
        &self.jump_to_row
    }

    pub fn jump_to_row_mut(&mut self) -> &mut JumpToRowState {
        &mut self.jump_to_row
    }

    // History access
    pub fn command_history(&self) -> &CommandHistory {
        &self.command_history
    }

    pub fn command_history_mut(&mut self) -> &mut CommandHistory {
        &mut self.command_history
    }

    // Results cache access
    pub fn results_cache(&self) -> &ResultsCache {
        &self.results_cache
    }

    pub fn results_cache_mut(&mut self) -> &mut ResultsCache {
        &mut self.results_cache
    }

    // Mode management with validation
    pub fn current_mode(&self) -> AppMode {
        self.mode_stack.last().cloned().unwrap_or(AppMode::Command)
    }

    pub fn enter_mode(&mut self, mode: AppMode) -> Result<()> {
        let current = self.current_mode();
        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "AppStateContainer",
                format!("MODE TRANSITION: {:?} -> {:?}", current, mode),
            );
        }

        // Validate transition
        match (current, mode.clone()) {
            // Add validation rules here
            _ => {
                // debug!(target: "state", "Mode transition allowed");
            }
        }

        self.mode_stack.push(mode);
        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "AppStateContainer",
                format!("Mode stack: {:?}", self.mode_stack),
            );
        }
        Ok(())
    }

    pub fn exit_mode(&mut self) -> Result<AppMode> {
        if self.mode_stack.len() > 1 {
            let exited = self.mode_stack.pop().unwrap();
            let new_mode = self.current_mode();
            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "AppStateContainer",
                    format!("MODE EXIT: {:?} -> {:?}", exited, new_mode),
                );
                debug_service.info(
                    "AppStateContainer",
                    format!("Mode stack after exit: {:?}", self.mode_stack),
                );
            }
            Ok(new_mode)
        } else {
            // debug!(target: "state", "Cannot exit base mode");
            Ok(self.current_mode())
        }
    }

    // Debug control
    pub fn toggle_debug(&mut self) {
        self.debug_enabled = !self.debug_enabled;
        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "AppStateContainer",
                format!("Debug mode: {}", self.debug_enabled),
            );
        }
    }

    /// Set the debug service for logging (can be called through Arc due to RefCell)
    pub fn set_debug_service(&self, debug_service: crate::debug_service::DebugService) {
        *self.debug_service.borrow_mut() = Some(debug_service);
        if let Some(ref service) = *self.debug_service.borrow() {
            service.info("AppStateContainer", "Debug service connected".to_string());
            service.info(
                "AppStateContainer",
                "AppStateContainer constructed with debug logging".to_string(),
            );
        }
    }

    pub fn is_debug_enabled(&self) -> bool {
        self.debug_enabled
    }

    // Help control
    pub fn toggle_help(&mut self) {
        let old_value = self.show_help;
        self.show_help = !self.show_help;
        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "AppStateContainer",
                format!(
                    "Help mode changed: {} -> {} (in toggle_help)",
                    old_value, self.show_help
                ),
            );
        }
    }

    pub fn is_help_visible(&self) -> bool {
        self.show_help
    }

    pub fn set_help_visible(&mut self, visible: bool) {
        let old_value = self.show_help;
        self.show_help = visible;
        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "AppStateContainer",
                format!(
                    "Help visibility changed: {} -> {} (in set_help_visible)",
                    old_value, visible
                ),
            );
        }
    }

    /// Generate comprehensive debug dump for F5
    pub fn debug_dump(&self) -> String {
        let mut dump = String::new();

        dump.push_str("=== APP STATE CONTAINER DEBUG DUMP ===\n\n");

        // Mode information
        dump.push_str("MODE INFORMATION:\n");
        dump.push_str(&format!("  Current Mode: {:?}\n", self.current_mode()));
        dump.push_str(&format!("  Mode Stack: {:?}\n", self.mode_stack));
        dump.push_str("\n");

        // UI Flags
        dump.push_str("UI FLAGS:\n");
        dump.push_str(&format!("  Help Visible: {}\n", self.show_help));
        dump.push_str(&format!("  Debug Enabled: {}\n", self.debug_enabled));
        dump.push_str("\n");

        // Input state
        dump.push_str("INPUT STATE:\n");
        dump.push_str(&format!("  Text: '{}'\n", self.command_input.text));
        dump.push_str(&format!(
            "  Cursor: {}\n",
            self.command_input.cursor_position
        ));
        dump.push_str(&format!(
            "  Last Query: '{}'\n",
            if self.command_input.last_executed_query.len() > 100 {
                format!("{}...", &self.command_input.last_executed_query[..100])
            } else {
                self.command_input.last_executed_query.clone()
            }
        ));
        dump.push_str("\n");

        // Search state
        if self.search.is_active {
            dump.push_str("SEARCH STATE (ACTIVE):\n");
            dump.push_str(&format!("  Pattern: '{}'\n", self.search.pattern));
            dump.push_str(&format!("  Matches: {} found\n", self.search.matches.len()));
            dump.push_str(&format!("  Current: {}\n", self.search.current_match));
            dump.push_str("\n");
        }

        // Filter state
        if self.filter.is_active {
            dump.push_str("FILTER STATE (ACTIVE):\n");
            dump.push_str(&format!("  Pattern: '{}'\n", self.filter.pattern));
            dump.push_str(&format!(
                "  Filtered Rows: {}\n",
                self.filter.filtered_indices.len()
            ));
            dump.push_str(&format!(
                "  Case Insensitive: {}\n",
                self.filter.case_insensitive
            ));
            dump.push_str("\n");
        }

        // Column search state
        if self.column_search.is_active {
            dump.push_str("COLUMN SEARCH STATE (ACTIVE):\n");
            dump.push_str(&format!("  Pattern: '{}'\n", self.column_search.pattern));
            dump.push_str(&format!(
                "  Matching Columns: {}\n",
                self.column_search.matching_columns.len()
            ));
            if !self.column_search.matching_columns.is_empty() {
                for (i, (idx, name)) in self
                    .column_search
                    .matching_columns
                    .iter()
                    .take(5)
                    .enumerate()
                {
                    dump.push_str(&format!(
                        "    [{}] {}: '{}'\n",
                        if i == self.column_search.current_match {
                            "*"
                        } else {
                            " "
                        },
                        idx,
                        name
                    ));
                }
            }
            dump.push_str("\n");
        }

        // Widget states using DebugInfoProvider trait
        dump.push_str(&self.widgets.search_modes.debug_info());
        dump.push_str("\n");
        if let Some(ref history) = self.widgets.history {
            dump.push_str(&history.debug_info());
            dump.push_str("\n");
        }
        dump.push_str(&self.widgets.help.debug_info());
        dump.push_str("\n");
        dump.push_str(&self.widgets.stats.debug_info());
        dump.push_str("\n");
        // TODO: Add debug widget info when available
        // dump.push_str(&self.widgets.debug.debug_info());
        // dump.push_str("\n");

        // Buffer information
        dump.push_str("BUFFER STATE:\n");
        dump.push_str(&format!(
            "  Current Buffer ID: {}\n",
            self.current_buffer_id
        ));
        // TODO: Add buffer count when method is available
        // dump.push_str(&format!("  Total Buffers: {}\n", self.buffers.count()));
        if let Some(buffer) = self.current_buffer() {
            // TODO: Add buffer mode and results when methods are available
            // dump.push_str(&format!("  Buffer Mode: {:?}\n", buffer.get_mode()));
            // if let Some(results) = buffer.get_results() {
            //     dump.push_str(&format!("  Results: {} rows x {} cols\n",
            //         results.data.len(),
            //         results.columns.len()
            //     ));
            // }
            dump.push_str("  Buffer: Present\n");
        } else {
            dump.push_str("  Buffer: None\n");
        }
        dump.push_str("\n");

        // Cache state
        dump.push_str("CACHE STATE:\n");
        dump.push_str(&format!(
            "  Cached Results: {}\n",
            self.results_cache.cache.len()
        ));
        dump.push_str(&format!(
            "  Max Cache Size: {}\n",
            self.results_cache.max_size
        ));
        dump.push_str("\n");

        // History state
        dump.push_str("HISTORY STATE:\n");
        dump.push_str(&format!(
            "  Total Commands: {}\n",
            self.command_history.get_all().len()
        ));
        dump.push_str("\n");

        dump.push_str("=== END DEBUG DUMP ===\n");

        dump
    }

    /// Pretty print the state for debugging
    pub fn pretty_print(&self) -> String {
        format!("{:#?}", self)
    }
}

impl fmt::Debug for AppStateContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateContainer")
            .field("current_mode", &self.current_mode())
            .field("mode_stack", &self.mode_stack)
            // TODO: Add buffer count when method is available
            // .field("buffer_count", &self.buffers.count())
            .field("current_buffer_id", &self.current_buffer_id)
            .field("command_input", &self.command_input)
            .field("search_active", &self.search.is_active)
            .field("filter_active", &self.filter.is_active)
            .field("column_search_active", &self.column_search.is_active)
            .field("debug_enabled", &self.debug_enabled)
            .field("show_help", &self.show_help)
            .field("cached_results", &self.results_cache.cache.len())
            .field("history_count", &self.command_history.get_all().len())
            .finish()
    }
}

impl fmt::Debug for WidgetStates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WidgetStates")
            .field("search_modes_active", &self.search_modes.is_active())
            .field("history", &self.history.is_some())
            .field("help", &"HelpWidget")
            .field("stats", &"StatsWidget")
            // .field("debug", &"DebugWidget") // TODO: Add when available
            .finish()
    }
}
