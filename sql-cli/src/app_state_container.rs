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
use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt;

/// Platform type for key handling
#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
    Unknown,
}

impl Platform {
    pub fn detect() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else {
            Platform::Unknown
        }
    }
}

/// Represents a single key press with all metadata
#[derive(Debug, Clone)]
pub struct KeyPressEntry {
    /// The raw key event from crossterm
    pub raw_event: KeyEvent,
    /// First timestamp when this key was pressed
    pub first_timestamp: DateTime<Local>,
    /// Last timestamp when this key was pressed (for repeats)
    pub last_timestamp: DateTime<Local>,
    /// Number of times this key was pressed consecutively
    pub repeat_count: usize,
    /// The platform where the key was pressed
    pub platform: Platform,
    /// The interpreted action (if any) from the key dispatcher
    pub interpreted_action: Option<String>,
    /// The mode the app was in when the key was pressed
    pub app_mode: AppMode,
    /// Formatted display string for the key
    pub display_string: String,
}

impl KeyPressEntry {
    pub fn new(key: KeyEvent, mode: AppMode, action: Option<String>) -> Self {
        let display_string = Self::format_key(&key);
        let now = Local::now();
        Self {
            raw_event: key,
            first_timestamp: now,
            last_timestamp: now,
            repeat_count: 1,
            platform: Platform::detect(),
            interpreted_action: action,
            app_mode: mode,
            display_string,
        }
    }

    /// Check if this entry represents the same key press (for coalescing)
    pub fn is_same_key(&self, key: &KeyEvent, mode: &AppMode) -> bool {
        self.raw_event == *key && self.app_mode == *mode
    }

    /// Add a repeat to this entry
    pub fn add_repeat(&mut self) {
        self.repeat_count += 1;
        self.last_timestamp = Local::now();
    }

    /// Get display string with repeat count
    pub fn display_with_count(&self) -> String {
        if self.repeat_count > 1 {
            format!("{} x{}", self.display_string, self.repeat_count)
        } else {
            self.display_string.clone()
        }
    }

    /// Format a key event for display
    fn format_key(key: &KeyEvent) -> String {
        let mut result = String::new();

        // Add modifiers
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            result.push_str("Ctrl+");
        }
        if key.modifiers.contains(KeyModifiers::ALT) {
            result.push_str("Alt+");
        }
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            result.push_str("Shift+");
        }

        // Add key code
        match key.code {
            KeyCode::Char(c) => result.push(c),
            KeyCode::Enter => result.push_str("Enter"),
            KeyCode::Esc => result.push_str("Esc"),
            KeyCode::Backspace => result.push_str("Backspace"),
            KeyCode::Tab => result.push_str("Tab"),
            KeyCode::Delete => result.push_str("Del"),
            KeyCode::Insert => result.push_str("Ins"),
            KeyCode::F(n) => result.push_str(&format!("F{}", n)),
            KeyCode::Left => result.push_str("←"),
            KeyCode::Right => result.push_str("→"),
            KeyCode::Up => result.push_str("↑"),
            KeyCode::Down => result.push_str("↓"),
            KeyCode::Home => result.push_str("Home"),
            KeyCode::End => result.push_str("End"),
            KeyCode::PageUp => result.push_str("PgUp"),
            KeyCode::PageDown => result.push_str("PgDn"),
            _ => result.push_str("?"),
        }

        result
    }

    /// Get a detailed debug string for this key press
    pub fn debug_string(&self) -> String {
        let modifiers = if self.raw_event.modifiers.is_empty() {
            String::new()
        } else {
            format!(" ({})", self.format_modifiers())
        };

        let action = self
            .interpreted_action
            .as_ref()
            .map(|a| format!(" → {}", a))
            .unwrap_or_default();

        let repeat_info = if self.repeat_count > 1 {
            format!(" x{}", self.repeat_count)
        } else {
            String::new()
        };

        format!(
            "[{}] {}{}{} [{:?}]{}",
            self.last_timestamp.format("%H:%M:%S.%3f"),
            self.display_string,
            repeat_info,
            modifiers,
            self.platform,
            action
        )
    }

    fn format_modifiers(&self) -> String {
        let mut parts = Vec::new();
        if self.raw_event.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if self.raw_event.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt");
        }
        if self.raw_event.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift");
        }
        parts.join("+")
    }
}

/// Manages key press history with a ring buffer and smart coalescing
#[derive(Debug, Clone)]
pub struct KeyPressHistory {
    /// Ring buffer of key presses
    entries: VecDeque<KeyPressEntry>,
    /// Maximum number of entries to keep
    max_size: usize,
}

impl KeyPressHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Check if a key is considered a navigation key
    fn is_navigation_key(key: &KeyEvent) -> bool {
        matches!(
            key.code,
            KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End
        )
    }

    /// Add a new key press to the history with smart coalescing
    pub fn add(&mut self, entry: KeyPressEntry) {
        // Check if we can coalesce with the last entry
        if let Some(last_entry) = self.entries.back_mut() {
            if last_entry.is_same_key(&entry.raw_event, &entry.app_mode) {
                // Same key pressed again in same mode, just increment counter
                last_entry.add_repeat();
                // Update the action in case it changed
                if entry.interpreted_action != last_entry.interpreted_action {
                    last_entry.interpreted_action = entry.interpreted_action;
                }
                return;
            }
        }

        // Not a repeat, need to add new entry
        // But first check if buffer is full
        if self.entries.len() >= self.max_size {
            // Smart removal strategy:
            // 1. First try to remove old navigation key entries with low repeat counts
            // 2. Then remove any old entry with low repeat count
            // 3. Finally just remove the oldest

            let mut removed = false;

            // Try to remove single-press navigation keys
            for i in 0..self.entries.len() {
                if Self::is_navigation_key(&self.entries[i].raw_event)
                    && self.entries[i].repeat_count == 1
                {
                    self.entries.remove(i);
                    removed = true;
                    break;
                }
            }

            // If no single navigation keys, remove any single-press entry from first half
            if !removed {
                let half = self.entries.len() / 2;
                for i in 0..half {
                    if self.entries[i].repeat_count == 1 {
                        self.entries.remove(i);
                        removed = true;
                        break;
                    }
                }
            }

            // Last resort: remove oldest
            if !removed {
                self.entries.pop_front();
            }
        }

        self.entries.push_back(entry);
    }

    /// Get all entries
    pub fn entries(&self) -> &VecDeque<KeyPressEntry> {
        &self.entries
    }

    /// Clear the history
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get formatted history for display
    pub fn format_history(&self) -> String {
        let mut output = String::new();
        output.push_str("========== KEY PRESS HISTORY ==========\n");
        output.push_str(&format!(
            "(Most recent at bottom, {} unique entries, max {})\n",
            self.entries.len(),
            self.max_size
        ));

        // Count total key presses including repeats
        let total_presses: usize = self.entries.iter().map(|e| e.repeat_count).sum();
        output.push_str(&format!(
            "Total key presses (with repeats): {}\n",
            total_presses
        ));

        for entry in &self.entries {
            output.push_str(&format!(
                "[{}] {}",
                entry.last_timestamp.format("%H:%M:%S.%3f"),
                entry.display_with_count()
            ));

            if !entry.raw_event.modifiers.is_empty() {
                output.push_str(&format!(" ({})", entry.format_modifiers()));
            }

            output.push('\n');
        }

        output.push_str("========================================\n");
        output
    }

    /// Get detailed debug history with platform info and actions
    pub fn format_debug_history(&self) -> String {
        let mut output = String::new();
        output.push_str("========== DETAILED KEY HISTORY ==========\n");
        output.push_str(&format!("Platform: {:?}\n", Platform::detect()));
        output.push_str(&format!(
            "(Most recent at bottom, last {} keys)\n",
            self.max_size
        ));

        for entry in &self.entries {
            output.push_str(&entry.debug_string());
            output.push('\n');
        }

        output.push_str("==========================================\n");
        output
    }
}

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
        let _old_text = self.text.clone();
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

/// Search operation types for tracking
#[derive(Debug, Clone)]
pub enum SearchOperation {
    StartSearch(String),
    UpdatePattern(String, String), // old, new
    MatchesFound(usize),
    NavigateToMatch(usize),
    ClearSearch,
    NoMatchesFound,
}

/// Search history entry
#[derive(Debug, Clone)]
pub struct SearchHistoryEntry {
    pub pattern: String,
    pub match_count: usize,
    pub timestamp: DateTime<Local>,
    pub duration_ms: Option<u64>,
}

/// Search state for regular search
#[derive(Debug, Clone)]
pub struct SearchState {
    pub pattern: String,
    pub matches: Vec<(usize, usize, usize, usize)>, // (row_start, col_start, row_end, col_end)
    pub current_match: usize,
    pub is_active: bool,
    pub history: VecDeque<SearchHistoryEntry>,
    pub last_search_time: Option<std::time::Instant>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            matches: Vec::new(),
            current_match: 0,
            is_active: false,
            history: VecDeque::with_capacity(20), // Keep last 20 searches
            last_search_time: None,
        }
    }

    pub fn clear(&mut self) {
        // Save to history if we had an active search
        if self.is_active && !self.pattern.is_empty() {
            let duration_ms = self
                .last_search_time
                .map(|t| t.elapsed().as_millis() as u64);
            let entry = SearchHistoryEntry {
                pattern: self.pattern.clone(),
                match_count: self.matches.len(),
                timestamp: Local::now(),
                duration_ms,
            };

            // Keep history size limited
            if self.history.len() >= 20 {
                self.history.pop_front();
            }
            self.history.push_back(entry);
        }

        self.pattern.clear();
        self.matches.clear();
        self.current_match = 0;
        self.is_active = false;
        self.last_search_time = None;
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

/// History search state (for Ctrl+R functionality)
#[derive(Debug, Clone)]
pub struct HistorySearchState {
    pub query: String,
    pub matches: Vec<crate::history::HistoryMatch>,
    pub selected_index: usize,
    pub is_active: bool,
    pub original_input: String,
}

impl HistorySearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            selected_index: 0,
            is_active: false,
            original_input: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.matches.clear();
        self.selected_index = 0;
        self.is_active = false;
        self.original_input.clear();
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
    search: RefCell<SearchState>,
    filter: RefCell<FilterState>,
    column_search: RefCell<ColumnSearchState>,
    history_search: RefCell<HistorySearchState>,

    // Widget states
    widgets: WidgetStates,

    // UI states
    cache_list: CacheListState,
    column_stats: ColumnStatsState,
    jump_to_row: JumpToRowState,

    // History
    command_history: CommandHistory,
    key_press_history: RefCell<KeyPressHistory>,

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
            search: RefCell::new(SearchState::new()),
            filter: RefCell::new(FilterState::new()),
            column_search: RefCell::new(ColumnSearchState::new()),
            history_search: RefCell::new(HistorySearchState::new()),
            widgets,
            cache_list: CacheListState::new(),
            column_stats: ColumnStatsState::new(),
            jump_to_row: JumpToRowState::new(),
            command_history,
            key_press_history: RefCell::new(KeyPressHistory::new(50)), // Keep last 50 key presses
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
    pub fn search(&self) -> std::cell::Ref<'_, SearchState> {
        self.search.borrow()
    }

    pub fn search_mut(&self) -> std::cell::RefMut<'_, SearchState> {
        self.search.borrow_mut()
    }

    // Search operations with logging

    /// Start a new search with the given pattern
    pub fn start_search(&self, pattern: String) -> usize {
        let mut search = self.search.borrow_mut();
        let old_pattern = search.pattern.clone();
        let old_active = search.is_active;

        search.pattern = pattern.clone();
        search.is_active = true;
        search.last_search_time = Some(std::time::Instant::now());

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Search",
                format!(
                    "Starting search: '{}' (was: '{}', active: {})",
                    pattern, old_pattern, old_active
                ),
            );
        }

        // Return match count (to be filled by caller for now)
        0
    }

    /// Update search matches
    pub fn update_search_matches(&self, matches: Vec<(usize, usize, usize, usize)>) {
        let match_count = matches.len();
        let mut search = self.search.borrow_mut();
        let pattern = search.pattern.clone();
        search.matches = matches;
        search.current_match = if match_count > 0 { 0 } else { 0 };

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Search",
                format!(
                    "Search found {} matches for pattern '{}'",
                    match_count, pattern
                ),
            );
        }

        // Record in history if this completes a search
        if !pattern.is_empty() {
            let duration_ms = search
                .last_search_time
                .map(|t| t.elapsed().as_millis() as u64);

            let entry = SearchHistoryEntry {
                pattern: pattern.clone(),
                match_count,
                timestamp: Local::now(),
                duration_ms,
            };

            if search.history.len() >= 20 {
                search.history.pop_front();
            }
            search.history.push_back(entry);
        }
    }

    /// Navigate to next search match
    pub fn next_search_match(&self) -> Option<(usize, usize)> {
        let mut search = self.search.borrow_mut();
        if search.matches.is_empty() {
            return None;
        }

        let old_match = search.current_match;
        search.current_match = (search.current_match + 1) % search.matches.len();

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Search",
                format!(
                    "Navigate to next match: {} -> {} (of {})",
                    old_match,
                    search.current_match,
                    search.matches.len()
                ),
            );
        }

        let match_pos = search.matches[search.current_match];
        Some((match_pos.0, match_pos.1))
    }

    /// Navigate to previous search match
    pub fn previous_search_match(&self) -> Option<(usize, usize)> {
        let mut search = self.search.borrow_mut();
        if search.matches.is_empty() {
            return None;
        }

        let old_match = search.current_match;
        search.current_match = if search.current_match == 0 {
            search.matches.len() - 1
        } else {
            search.current_match - 1
        };

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Search",
                format!(
                    "Navigate to previous match: {} -> {} (of {})",
                    old_match,
                    search.current_match,
                    search.matches.len()
                ),
            );
        }

        let match_pos = search.matches[search.current_match];
        Some((match_pos.0, match_pos.1))
    }

    /// Clear current search
    pub fn clear_search(&self) {
        let mut search = self.search.borrow_mut();
        let had_matches = search.matches.len();
        let had_pattern = search.pattern.clone();

        search.clear();

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Search",
                format!(
                    "Cleared search (had pattern: '{}', {} matches)",
                    had_pattern, had_matches
                ),
            );
        }
    }

    /// Perform search on provided data
    /// Returns the search matches as a vector of (row, col, row_end, col_end) tuples
    pub fn perform_search(&self, data: &[Vec<String>]) -> Vec<(usize, usize, usize, usize)> {
        use regex::Regex;

        let pattern = self.search.borrow().pattern.clone();
        if pattern.is_empty() {
            let mut search = self.search.borrow_mut();
            search.matches.clear();
            search.current_match = 0;
            return Vec::new();
        }

        let start_time = std::time::Instant::now();
        let mut matches = Vec::new();

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Search",
                format!(
                    "Performing search for pattern '{}' on {} rows",
                    pattern,
                    data.len()
                ),
            );
        }

        // Try to compile regex pattern
        match Regex::new(&pattern) {
            Ok(regex) => {
                for (row_idx, row) in data.iter().enumerate() {
                    for (col_idx, cell) in row.iter().enumerate() {
                        if regex.is_match(cell) {
                            // For now, just store simple match positions
                            // In future, could store actual match spans
                            matches.push((row_idx, col_idx, row_idx, col_idx));
                        }
                    }
                }
            }
            Err(e) => {
                if let Some(ref debug_service) = *self.debug_service.borrow() {
                    debug_service.info(
                        "Search",
                        format!("Invalid regex pattern '{}': {}", pattern, e),
                    );
                }
                // Fall back to simple string contains search
                let pattern_lower = pattern.to_lowercase();
                for (row_idx, row) in data.iter().enumerate() {
                    for (col_idx, cell) in row.iter().enumerate() {
                        if cell.to_lowercase().contains(&pattern_lower) {
                            matches.push((row_idx, col_idx, row_idx, col_idx));
                        }
                    }
                }
            }
        }

        let elapsed = start_time.elapsed();
        self.search.borrow_mut().last_search_time = Some(start_time);

        // Update search state with matches
        self.update_search_matches(matches.clone());

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Search",
                format!(
                    "Search completed in {:?}: found {} matches for '{}'",
                    elapsed,
                    matches.len(),
                    pattern
                ),
            );
        }

        matches
    }

    /// Get current search match position (for highlighting)
    pub fn get_current_match(&self) -> Option<(usize, usize)> {
        let search = self.search.borrow();
        if search.matches.is_empty() || !search.is_active {
            return None;
        }

        let match_pos = search.matches[search.current_match];
        Some((match_pos.0, match_pos.1))
    }

    pub fn filter(&self) -> std::cell::Ref<'_, FilterState> {
        self.filter.borrow()
    }

    pub fn filter_mut(&self) -> std::cell::RefMut<'_, FilterState> {
        self.filter.borrow_mut()
    }

    pub fn column_search(&self) -> std::cell::Ref<'_, ColumnSearchState> {
        self.column_search.borrow()
    }

    pub fn column_search_mut(&self) -> std::cell::RefMut<'_, ColumnSearchState> {
        self.column_search.borrow_mut()
    }

    // History search operations (Ctrl+R)
    pub fn start_history_search(&self, original_input: String) {
        let mut history_search = self.history_search.borrow_mut();
        history_search.query.clear();
        history_search.matches.clear();
        history_search.selected_index = 0;
        history_search.is_active = true;
        history_search.original_input = original_input;

        // Initialize with all history entries
        let all_entries = self.command_history.get_all();
        history_search.matches = all_entries
            .iter()
            .cloned()
            .map(|entry| crate::history::HistoryMatch {
                entry,
                indices: Vec::new(),
                score: 0,
            })
            .collect();

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "HistorySearch",
                format!(
                    "Started history search with {} entries",
                    history_search.matches.len()
                ),
            );
        }
    }

    pub fn update_history_search(&self, query: String) {
        let mut history_search = self.history_search.borrow_mut();
        let old_query = history_search.query.clone();
        history_search.query = query.clone();

        if query.is_empty() {
            // Show all history when no search
            let all_entries = self.command_history.get_all();
            history_search.matches = all_entries
                .iter()
                .cloned()
                .map(|entry| crate::history::HistoryMatch {
                    entry,
                    indices: Vec::new(),
                    score: 0,
                })
                .collect();
        } else {
            // Use fuzzy search
            use fuzzy_matcher::skim::SkimMatcherV2;
            use fuzzy_matcher::FuzzyMatcher;

            let matcher = SkimMatcherV2::default();
            let mut matches: Vec<crate::history::HistoryMatch> = self
                .command_history
                .get_all()
                .iter()
                .cloned()
                .filter_map(|entry| {
                    matcher
                        .fuzzy_indices(&entry.command, &query)
                        .map(|(score, indices)| crate::history::HistoryMatch {
                            entry,
                            indices,
                            score,
                        })
                })
                .collect();

            // Sort by score (highest first)
            matches.sort_by(|a, b| b.score.cmp(&a.score));
            history_search.matches = matches;
        }

        // Reset selected index if it's out of bounds
        if history_search.selected_index >= history_search.matches.len() {
            history_search.selected_index = 0;
        }

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "HistorySearch",
                format!(
                    "Updated history search: '{}' -> '{}', {} matches",
                    old_query,
                    query,
                    history_search.matches.len()
                ),
            );
        }
    }

    pub fn history_search_next(&self) {
        let mut history_search = self.history_search.borrow_mut();
        if !history_search.matches.is_empty() {
            let old_index = history_search.selected_index;
            history_search.selected_index =
                (history_search.selected_index + 1) % history_search.matches.len();

            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "HistorySearch",
                    format!(
                        "Navigate next: {} -> {}",
                        old_index, history_search.selected_index
                    ),
                );
            }
        }
    }

    pub fn history_search_previous(&self) {
        let mut history_search = self.history_search.borrow_mut();
        if !history_search.matches.is_empty() {
            let old_index = history_search.selected_index;
            history_search.selected_index = if history_search.selected_index == 0 {
                history_search.matches.len() - 1
            } else {
                history_search.selected_index - 1
            };

            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "HistorySearch",
                    format!(
                        "Navigate previous: {} -> {}",
                        old_index, history_search.selected_index
                    ),
                );
            }
        }
    }

    pub fn get_selected_history_command(&self) -> Option<String> {
        let history_search = self.history_search.borrow();
        history_search
            .matches
            .get(history_search.selected_index)
            .map(|m| m.entry.command.clone())
    }

    pub fn accept_history_search(&self) -> Option<String> {
        let mut history_search = self.history_search.borrow_mut();
        if history_search.is_active {
            let command = history_search
                .matches
                .get(history_search.selected_index)
                .map(|m| m.entry.command.clone());

            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "HistorySearch",
                    format!("Accepted history command: {:?}", command),
                );
            }

            history_search.clear();
            command
        } else {
            None
        }
    }

    pub fn cancel_history_search(&self) -> String {
        let mut history_search = self.history_search.borrow_mut();
        let original = history_search.original_input.clone();

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "HistorySearch",
                format!("Cancelled history search, restoring: '{}'", original),
            );
        }

        history_search.clear();
        original
    }

    pub fn history_search(&self) -> std::cell::Ref<'_, HistorySearchState> {
        self.history_search.borrow()
    }

    pub fn is_history_search_active(&self) -> bool {
        self.history_search.borrow().is_active
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

    // Key press management - uses interior mutability so it can be called through Arc
    pub fn log_key_press(&self, key: KeyEvent, action: Option<String>) {
        let mode = self.current_mode();
        let entry = KeyPressEntry::new(key, mode.clone(), action.clone());

        // Log to debug service with platform info
        if let Some(ref debug_service) = *self.debug_service.borrow() {
            let platform_info = if entry.platform == Platform::Windows
                && (key.code == KeyCode::Char('$') || key.code == KeyCode::Char('^'))
                && key.modifiers.contains(KeyModifiers::SHIFT)
            {
                " [Windows: SHIFT modifier present]"
            } else {
                ""
            };

            debug_service.info(
                "KeyPress",
                format!(
                    "Key: {:?}, Mode: {:?}, Action: {:?}, Platform: {:?}{}",
                    key, mode, action, entry.platform, platform_info
                ),
            );
        }

        self.key_press_history.borrow_mut().add(entry);
    }

    pub fn clear_key_history(&self) {
        self.key_press_history.borrow_mut().clear();
        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info("AppStateContainer", "Key press history cleared".to_string());
        }
    }

    /// Normalize a key event for platform-specific differences
    /// This handles cases like Windows sending Shift+$ instead of just $
    pub fn normalize_key(&self, key: KeyEvent) -> KeyEvent {
        let platform = Platform::detect();

        // On Windows, special characters like $ and ^ come with SHIFT modifier
        // but the key dispatcher expects them without SHIFT
        if platform == Platform::Windows {
            match key.code {
                KeyCode::Char('$')
                | KeyCode::Char('^')
                | KeyCode::Char('!')
                | KeyCode::Char('@')
                | KeyCode::Char('#')
                | KeyCode::Char('%')
                | KeyCode::Char('&')
                | KeyCode::Char('*')
                | KeyCode::Char('(')
                | KeyCode::Char(')') => {
                    // Remove SHIFT modifier for these characters on Windows
                    let mut normalized_modifiers = key.modifiers;
                    normalized_modifiers.remove(KeyModifiers::SHIFT);

                    if let Some(ref debug_service) = *self.debug_service.borrow() {
                        if normalized_modifiers != key.modifiers {
                            debug_service.info(
                                "KeyNormalize",
                                format!(
                                    "Windows key normalization: {:?} with {:?} -> {:?}",
                                    key.code, key.modifiers, normalized_modifiers
                                ),
                            );
                        }
                    }

                    KeyEvent::new(key.code, normalized_modifiers)
                }
                _ => key,
            }
        } else {
            key
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
        dump.push_str("SEARCH STATE:\n");
        let search = self.search.borrow();
        if search.is_active {
            dump.push_str(&format!("  Pattern: '{}'\n", search.pattern));
            dump.push_str(&format!("  Matches: {} found\n", search.matches.len()));
            dump.push_str(&format!(
                "  Current: {} of {}\n",
                if search.matches.is_empty() {
                    0
                } else {
                    search.current_match + 1
                },
                search.matches.len()
            ));
            if let Some(ref last_time) = search.last_search_time {
                dump.push_str(&format!("  Search time: {:?}\n", last_time.elapsed()));
            }
        } else {
            dump.push_str("  [Inactive]\n");
        }

        // Search history
        if !search.history.is_empty() {
            dump.push_str("  Recent searches:\n");
            for (i, entry) in search.history.iter().rev().take(5).enumerate() {
                dump.push_str(&format!(
                    "    {}. '{}' → {} matches",
                    i + 1,
                    if entry.pattern.len() > 30 {
                        format!("{}...", &entry.pattern[..30])
                    } else {
                        entry.pattern.clone()
                    },
                    entry.match_count
                ));
                if let Some(duration) = entry.duration_ms {
                    dump.push_str(&format!(" ({}ms)", duration));
                }
                dump.push_str(&format!(" at {}\n", entry.timestamp.format("%H:%M:%S")));
            }
        }
        dump.push_str("\n");

        // Filter state
        let filter = self.filter.borrow();
        if filter.is_active {
            dump.push_str("FILTER STATE (ACTIVE):\n");
            dump.push_str(&format!("  Pattern: '{}'\n", filter.pattern));
            dump.push_str(&format!(
                "  Filtered Rows: {}\n",
                filter.filtered_indices.len()
            ));
            dump.push_str(&format!(
                "  Case Insensitive: {}\n",
                filter.case_insensitive
            ));
            dump.push_str("\n");
        }

        // Column search state
        let column_search = self.column_search.borrow();
        if column_search.is_active {
            dump.push_str("COLUMN SEARCH STATE (ACTIVE):\n");
            dump.push_str(&format!("  Pattern: '{}'\n", column_search.pattern));
            dump.push_str(&format!(
                "  Matching Columns: {}\n",
                column_search.matching_columns.len()
            ));
            if !column_search.matching_columns.is_empty() {
                for (i, (idx, name)) in column_search.matching_columns.iter().take(5).enumerate() {
                    dump.push_str(&format!(
                        "    [{}] {}: '{}'\n",
                        if i == column_search.current_match {
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

        // History search state (Ctrl+R)
        let history_search = self.history_search.borrow();
        if history_search.is_active {
            dump.push_str("HISTORY SEARCH STATE (ACTIVE):\n");
            dump.push_str(&format!("  Query: '{}'\n", history_search.query));
            dump.push_str(&format!("  Matches: {}\n", history_search.matches.len()));
            dump.push_str(&format!("  Selected: {}\n", history_search.selected_index));
            dump.push_str(&format!(
                "  Original Input: '{}'\n",
                history_search.original_input
            ));
            if !history_search.matches.is_empty() {
                dump.push_str("  Top matches:\n");
                for (i, m) in history_search.matches.iter().take(5).enumerate() {
                    dump.push_str(&format!(
                        "    [{}] Score: {}, '{}'\n",
                        if i == history_search.selected_index {
                            "*"
                        } else {
                            " "
                        },
                        m.score,
                        if m.entry.command.len() > 50 {
                            format!("{}...", &m.entry.command[..50])
                        } else {
                            m.entry.command.clone()
                        }
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
        if let Some(_buffer) = self.current_buffer() {
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

        // Key press history
        dump.push_str(&self.key_press_history.borrow().format_history());
        dump.push_str("\n");

        // Platform-specific key information
        dump.push_str("PLATFORM INFO:\n");
        dump.push_str(&format!("  Platform: {:?}\n", Platform::detect()));
        dump.push_str("  Key Normalization: ");
        if Platform::detect() == Platform::Windows {
            dump.push_str("ACTIVE (Windows special chars)\n");
        } else {
            dump.push_str("INACTIVE\n");
        }
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
            .field("search_active", &self.search.borrow().is_active)
            .field("filter_active", &self.filter.borrow().is_active)
            .field(
                "column_search_active",
                &self.column_search.borrow().is_active,
            )
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
