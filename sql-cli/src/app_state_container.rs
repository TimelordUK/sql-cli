use crate::api_client::QueryResponse;
use crate::buffer::{AppMode, BufferManager, SortOrder};
use crate::debug_service::DebugLevel;
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
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::{Duration, Instant};
use tracing::info;

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
    pub filtered_data: Option<Vec<Vec<String>>>,
    pub is_active: bool,
    pub case_insensitive: bool,
    pub total_filters: usize,
    pub last_filter_time: Option<Instant>,
    pub history: VecDeque<FilterHistoryEntry>,
    pub max_history: usize,
}

#[derive(Debug, Clone)]
pub struct FilterHistoryEntry {
    pub pattern: String,
    pub match_count: usize,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub duration_ms: Option<u64>,
}

impl FilterState {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            filtered_indices: Vec::new(),
            filtered_data: None,
            is_active: false,
            case_insensitive: true,
            total_filters: 0,
            last_filter_time: None,
            history: VecDeque::with_capacity(20),
            max_history: 20,
        }
    }

    pub fn clear(&mut self) {
        info!(target: "filter", "FilterState::clear() - had {} filtered rows for pattern '{}'",
              self.filtered_indices.len(), self.pattern);

        // Add to history before clearing
        if !self.pattern.is_empty() && self.is_active {
            let duration_ms = self
                .last_filter_time
                .as_ref()
                .map(|t| t.elapsed().as_millis() as u64);
            let entry = FilterHistoryEntry {
                pattern: self.pattern.clone(),
                match_count: self.filtered_indices.len(),
                timestamp: chrono::Local::now(),
                duration_ms,
            };
            self.history.push_front(entry);
            if self.history.len() > self.max_history {
                self.history.pop_back();
            }
        }

        self.pattern.clear();
        self.filtered_indices.clear();
        self.filtered_data = None;
        self.is_active = false;
        self.last_filter_time = None;
    }

    /// Set filter pattern and mark as active
    pub fn set_pattern(&mut self, pattern: String) {
        info!(target: "filter", "FilterState::set_pattern('{}') - was '{}'", pattern, self.pattern);
        self.pattern = pattern;
        if !self.pattern.is_empty() {
            self.is_active = true;
            self.total_filters += 1;
            self.last_filter_time = Some(Instant::now());
        } else {
            self.is_active = false;
        }
    }

    /// Set filtered indices from filter operation
    pub fn set_filtered_indices(&mut self, indices: Vec<usize>) {
        info!(target: "filter", "FilterState::set_filtered_indices - {} rows match pattern '{}'", 
              indices.len(), self.pattern);
        self.filtered_indices = indices;
    }

    /// Set filtered data from filter operation
    pub fn set_filtered_data(&mut self, data: Option<Vec<Vec<String>>>) {
        let count = data.as_ref().map(|d| d.len()).unwrap_or(0);
        info!(target: "filter", "FilterState::set_filtered_data - {} rows", count);
        self.filtered_data = data;
    }

    /// Get filter statistics
    pub fn get_stats(&self) -> String {
        format!(
            "Total filters: {}, History items: {}, Current matches: {}",
            self.total_filters,
            self.history.len(),
            self.filtered_indices.len()
        )
    }
}

/// Column search state management
#[derive(Debug, Clone)]
pub struct ColumnSearchState {
    /// Current search pattern
    pub pattern: String,

    /// Matching columns (index, column_name)
    pub matching_columns: Vec<(usize, String)>,

    /// Current match index (index into matching_columns)
    pub current_match: usize,

    /// Whether column search is active
    pub is_active: bool,

    /// Search history
    pub history: VecDeque<ColumnSearchHistoryEntry>,

    /// Total searches performed
    pub total_searches: usize,

    /// Last search time
    pub last_search_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct ColumnSearchHistoryEntry {
    /// Search pattern
    pub pattern: String,

    /// Number of matching columns
    pub match_count: usize,

    /// Column names that matched
    pub matched_columns: Vec<String>,

    /// When this search was performed
    pub timestamp: DateTime<Local>,

    /// How long the search took
    pub duration_ms: Option<u64>,
}

impl Default for ColumnSearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl ColumnSearchState {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            matching_columns: Vec::new(),
            current_match: 0,
            is_active: false,
            history: VecDeque::with_capacity(20),
            total_searches: 0,
            last_search_time: None,
        }
    }

    /// Clear the column search state
    pub fn clear(&mut self) {
        // Save to history if we had an active search
        if self.is_active && !self.pattern.is_empty() {
            let duration_ms = self
                .last_search_time
                .map(|t| t.elapsed().as_millis() as u64);
            let entry = ColumnSearchHistoryEntry {
                pattern: self.pattern.clone(),
                match_count: self.matching_columns.len(),
                matched_columns: self
                    .matching_columns
                    .iter()
                    .map(|(_, name)| name.clone())
                    .collect(),
                timestamp: Local::now(),
                duration_ms,
            };
            self.history.push_front(entry);

            // Trim history
            while self.history.len() > 20 {
                self.history.pop_back();
            }
        }

        self.pattern.clear();
        self.matching_columns.clear();
        self.current_match = 0;
        self.is_active = false;
        self.last_search_time = None;
    }

    /// Set search results
    pub fn set_matches(&mut self, matches: Vec<(usize, String)>) {
        self.matching_columns = matches;
        self.current_match = 0;
        self.total_searches += 1;
        self.last_search_time = Some(Instant::now());
    }

    /// Navigate to next match
    pub fn next_match(&mut self) -> Option<(usize, String)> {
        if self.matching_columns.is_empty() {
            return None;
        }

        self.current_match = (self.current_match + 1) % self.matching_columns.len();
        Some(self.matching_columns[self.current_match].clone())
    }

    /// Navigate to previous match
    pub fn prev_match(&mut self) -> Option<(usize, String)> {
        if self.matching_columns.is_empty() {
            return None;
        }

        self.current_match = if self.current_match == 0 {
            self.matching_columns.len() - 1
        } else {
            self.current_match - 1
        };
        Some(self.matching_columns[self.current_match].clone())
    }

    /// Get current match
    pub fn current_match(&self) -> Option<(usize, String)> {
        if self.matching_columns.is_empty() {
            None
        } else {
            Some(self.matching_columns[self.current_match].clone())
        }
    }

    /// Get search statistics
    pub fn get_stats(&self) -> String {
        format!(
            "Total searches: {}, History items: {}, Current matches: {}",
            self.total_searches,
            self.history.len(),
            self.matching_columns.len()
        )
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

/// Navigation and viewport state
#[derive(Debug, Clone)]
pub struct NavigationState {
    pub selected_row: usize,
    pub selected_column: usize,
    pub scroll_offset: (usize, usize), // (row, col)
    pub viewport_rows: usize,
    pub viewport_columns: usize,
    pub total_rows: usize,
    pub total_columns: usize,
    pub last_visible_rows: usize,
    pub viewport_lock: bool, // Lock viewport position (cursor moves within)
    pub viewport_lock_row: Option<usize>,
    pub cursor_lock: bool, // Lock cursor at visual position (data scrolls)
    pub cursor_lock_position: Option<usize>, // Visual position to lock cursor at
    pub selection_history: VecDeque<(usize, usize)>, // Track navigation history
}

impl NavigationState {
    pub fn new() -> Self {
        Self {
            selected_row: 0,
            selected_column: 0,
            scroll_offset: (0, 0),
            viewport_rows: 30,
            viewport_columns: 10,
            total_rows: 0,
            total_columns: 0,
            last_visible_rows: 0,
            viewport_lock: false,
            viewport_lock_row: None,
            cursor_lock: false,
            cursor_lock_position: None,
            selection_history: VecDeque::with_capacity(50), // Keep last 50 positions
        }
    }

    pub fn update_totals(&mut self, rows: usize, columns: usize) {
        info!(target: "navigation", "NavigationState::update_totals - rows: {} -> {}, columns: {} -> {}", 
              self.total_rows, rows, self.total_columns, columns);

        self.total_rows = rows;
        self.total_columns = columns;

        // Adjust selected position if it's out of bounds
        if self.selected_row >= rows && rows > 0 {
            let old_row = self.selected_row;
            self.selected_row = rows - 1;
            info!(target: "navigation", "Adjusted selected_row from {} to {} (out of bounds)", old_row, self.selected_row);
        }
        if self.selected_column >= columns && columns > 0 {
            let old_col = self.selected_column;
            self.selected_column = columns - 1;
            info!(target: "navigation", "Adjusted selected_column from {} to {} (out of bounds)", old_col, self.selected_column);
        }
    }

    pub fn set_viewport_size(&mut self, rows: usize, columns: usize) {
        info!(target: "navigation", "NavigationState::set_viewport_size - rows: {} -> {}, columns: {} -> {}", 
              self.viewport_rows, rows, self.viewport_columns, columns);
        self.viewport_rows = rows;
        self.viewport_columns = columns;
    }

    /// Move to next row
    pub fn next_row(&mut self) -> bool {
        if self.cursor_lock {
            // In cursor lock mode, scroll the data instead of moving cursor
            if let Some(lock_position) = self.cursor_lock_position {
                // Check if we can scroll down
                let max_scroll = self.total_rows.saturating_sub(self.viewport_rows);
                if self.scroll_offset.0 < max_scroll {
                    self.scroll_offset.0 += 1;
                    // Keep cursor at the locked visual position
                    let new_data_row = self.scroll_offset.0 + lock_position;
                    if new_data_row < self.total_rows {
                        self.selected_row = new_data_row;
                        self.add_to_history(self.selected_row, self.selected_column);
                        info!(target: "navigation", "NavigationState::next_row (cursor locked) - scrolled to offset {}, cursor at row {}", 
                              self.scroll_offset.0, self.selected_row);
                        return true;
                    }
                }
                return false;
            }
        }

        // Check viewport lock boundaries
        if self.viewport_lock {
            // In viewport lock mode, don't allow cursor to leave visible area
            let viewport_bottom = self.scroll_offset.0 + self.viewport_rows - 1;
            if self.selected_row >= viewport_bottom {
                info!(target: "navigation", "NavigationState::next_row - at viewport bottom (row {}), viewport locked", self.selected_row);
                return false; // Already at bottom of viewport
            }
        }

        // Normal navigation (with viewport lock boundary check)
        if self.selected_row < self.total_rows.saturating_sub(1) {
            self.selected_row += 1;
            self.add_to_history(self.selected_row, self.selected_column);
            self.ensure_visible(self.selected_row, self.selected_column);
            info!(target: "navigation", "NavigationState::next_row - moved to row {}", self.selected_row);
            true
        } else {
            false
        }
    }

    /// Move to previous row
    pub fn previous_row(&mut self) -> bool {
        if self.cursor_lock {
            // In cursor lock mode, scroll the data instead of moving cursor
            if let Some(lock_position) = self.cursor_lock_position {
                // Check if we can scroll up
                if self.scroll_offset.0 > 0 {
                    self.scroll_offset.0 -= 1;
                    // Keep cursor at the locked visual position
                    let new_data_row = self.scroll_offset.0 + lock_position;
                    self.selected_row = new_data_row;
                    self.add_to_history(self.selected_row, self.selected_column);
                    info!(target: "navigation", "NavigationState::previous_row (cursor locked) - scrolled to offset {}, cursor at row {}", 
                          self.scroll_offset.0, self.selected_row);
                    return true;
                }
                return false;
            }
        }

        // Check viewport lock boundaries
        if self.viewport_lock {
            // In viewport lock mode, don't allow cursor to leave visible area
            let viewport_top = self.scroll_offset.0;
            if self.selected_row <= viewport_top {
                info!(target: "navigation", "NavigationState::previous_row - at viewport top (row {}), viewport locked", self.selected_row);
                return false; // Already at top of viewport
            }
        }

        // Normal navigation (with viewport lock boundary check)
        if self.selected_row > 0 {
            self.selected_row -= 1;
            self.add_to_history(self.selected_row, self.selected_column);
            self.ensure_visible(self.selected_row, self.selected_column);
            info!(target: "navigation", "NavigationState::previous_row - moved to row {}", self.selected_row);
            true
        } else {
            false
        }
    }

    /// Move to next column
    pub fn next_column(&mut self) -> bool {
        if self.selected_column < self.total_columns.saturating_sub(1) {
            self.selected_column += 1;
            self.add_to_history(self.selected_row, self.selected_column);
            self.ensure_visible(self.selected_row, self.selected_column);
            info!(target: "navigation", "NavigationState::next_column - moved to column {}", self.selected_column);
            true
        } else {
            false
        }
    }

    /// Move to previous column
    pub fn previous_column(&mut self) -> bool {
        if self.selected_column > 0 {
            self.selected_column -= 1;
            self.add_to_history(self.selected_row, self.selected_column);
            self.ensure_visible(self.selected_row, self.selected_column);
            info!(target: "navigation", "NavigationState::previous_column - moved to column {}", self.selected_column);
            true
        } else {
            false
        }
    }

    /// Jump to specific row
    pub fn jump_to_row(&mut self, row: usize) {
        let target_row = row.min(self.total_rows.saturating_sub(1));
        info!(target: "navigation", "NavigationState::jump_to_row - from {} to {}", self.selected_row, target_row);
        self.selected_row = target_row;
        self.add_to_history(self.selected_row, self.selected_column);
        self.ensure_visible(self.selected_row, self.selected_column);
    }

    /// Jump to first row
    pub fn jump_to_first_row(&mut self) {
        info!(target: "navigation", "NavigationState::jump_to_first_row - from row {}", self.selected_row);
        self.selected_row = 0;
        self.add_to_history(self.selected_row, self.selected_column);
        self.ensure_visible(self.selected_row, self.selected_column);
    }

    /// Jump to last row
    pub fn jump_to_last_row(&mut self) {
        let last_row = self.total_rows.saturating_sub(1);
        info!(target: "navigation", "NavigationState::jump_to_last_row - from {} to {}", self.selected_row, last_row);
        self.selected_row = last_row;
        self.add_to_history(self.selected_row, self.selected_column);
        self.ensure_visible(self.selected_row, self.selected_column);
    }

    /// Set selected position
    pub fn set_position(&mut self, row: usize, column: usize) {
        info!(target: "navigation", "NavigationState::set_position - ({}, {}) -> ({}, {})", 
              self.selected_row, self.selected_column, row, column);
        self.selected_row = row.min(self.total_rows.saturating_sub(1));
        self.selected_column = column.min(self.total_columns.saturating_sub(1));
        self.add_to_history(self.selected_row, self.selected_column);
        self.ensure_visible(self.selected_row, self.selected_column);
    }

    /// Page down
    pub fn page_down(&mut self) {
        if self.cursor_lock {
            // In cursor lock mode, scroll the data by a page
            if let Some(lock_position) = self.cursor_lock_position {
                let max_scroll = self.total_rows.saturating_sub(self.viewport_rows);
                let new_scroll = (self.scroll_offset.0 + self.viewport_rows).min(max_scroll);
                if new_scroll != self.scroll_offset.0 {
                    self.scroll_offset.0 = new_scroll;
                    // Keep cursor at the locked visual position
                    let new_data_row = self.scroll_offset.0 + lock_position;
                    if new_data_row < self.total_rows {
                        self.selected_row = new_data_row;
                        self.add_to_history(self.selected_row, self.selected_column);
                        info!(target: "navigation", "NavigationState::page_down (cursor locked) - scrolled to offset {}, cursor at row {}", 
                              self.scroll_offset.0, self.selected_row);
                    }
                }
                return;
            }
        }

        // Normal page down when not locked
        let old_row = self.selected_row;
        self.selected_row =
            (self.selected_row + self.viewport_rows).min(self.total_rows.saturating_sub(1));
        if self.selected_row != old_row {
            info!(target: "navigation", "NavigationState::page_down - from {} to {}", old_row, self.selected_row);
            self.add_to_history(self.selected_row, self.selected_column);
            self.ensure_visible(self.selected_row, self.selected_column);
        }
    }

    /// Page up
    pub fn page_up(&mut self) {
        if self.cursor_lock {
            // In cursor lock mode, scroll the data by a page
            if let Some(lock_position) = self.cursor_lock_position {
                let new_scroll = self.scroll_offset.0.saturating_sub(self.viewport_rows);
                if new_scroll != self.scroll_offset.0 {
                    self.scroll_offset.0 = new_scroll;
                    // Keep cursor at the locked visual position
                    let new_data_row = self.scroll_offset.0 + lock_position;
                    self.selected_row = new_data_row;
                    self.add_to_history(self.selected_row, self.selected_column);
                    info!(target: "navigation", "NavigationState::page_up (cursor locked) - scrolled to offset {}, cursor at row {}", 
                          self.scroll_offset.0, self.selected_row);
                }
                return;
            }
        }

        // Normal page up when not locked
        let old_row = self.selected_row;
        self.selected_row = self.selected_row.saturating_sub(self.viewport_rows);
        if self.selected_row != old_row {
            info!(target: "navigation", "NavigationState::page_up - from {} to {}", old_row, self.selected_row);
            self.add_to_history(self.selected_row, self.selected_column);
            self.ensure_visible(self.selected_row, self.selected_column);
        }
    }

    /// Jump to top of viewport (H in vim)
    pub fn jump_to_viewport_top(&mut self) {
        let target_row = self.scroll_offset.0;
        if target_row != self.selected_row && target_row < self.total_rows {
            info!(target: "navigation", "NavigationState::jump_to_viewport_top - from {} to {} (viewport top)", 
                  self.selected_row, target_row);
            self.selected_row = target_row;
            self.add_to_history(self.selected_row, self.selected_column);
            // No need to ensure_visible since we're jumping to a visible position
        }
    }

    /// Jump to middle of viewport (M in vim)
    pub fn jump_to_viewport_middle(&mut self) {
        let viewport_start = self.scroll_offset.0;
        let viewport_end = (viewport_start + self.viewport_rows).min(self.total_rows);
        let target_row = viewport_start + (viewport_end - viewport_start) / 2;

        if target_row != self.selected_row && target_row < self.total_rows {
            info!(target: "navigation", "NavigationState::jump_to_viewport_middle - from {} to {} (viewport middle)", 
                  self.selected_row, target_row);
            self.selected_row = target_row;
            self.add_to_history(self.selected_row, self.selected_column);
            // No need to ensure_visible since we're jumping to a visible position
        }
    }

    /// Jump to bottom of viewport (L in vim)
    pub fn jump_to_viewport_bottom(&mut self) {
        let viewport_start = self.scroll_offset.0;
        let viewport_end = (viewport_start + self.viewport_rows).min(self.total_rows);
        let target_row = viewport_end.saturating_sub(1);

        if target_row != self.selected_row && target_row < self.total_rows {
            info!(target: "navigation", "NavigationState::jump_to_viewport_bottom - from {} to {} (viewport bottom)", 
                  self.selected_row, target_row);
            self.selected_row = target_row;
            self.add_to_history(self.selected_row, self.selected_column);
            // No need to ensure_visible since we're jumping to a visible position
        }
    }

    pub fn is_position_visible(&self, row: usize, col: usize) -> bool {
        let (scroll_row, scroll_col) = self.scroll_offset;
        row >= scroll_row
            && row < scroll_row + self.viewport_rows
            && col >= scroll_col
            && col < scroll_col + self.viewport_columns
    }

    pub fn ensure_visible(&mut self, row: usize, col: usize) {
        // If viewport is locked, don't adjust scroll offset
        if self.viewport_lock {
            info!(target: "navigation", "NavigationState::ensure_visible - viewport locked, not adjusting scroll");
            return;
        }

        let (mut scroll_row, mut scroll_col) = self.scroll_offset;

        // Adjust row scrolling
        if row < scroll_row {
            scroll_row = row;
        } else if row >= scroll_row + self.viewport_rows {
            scroll_row = row.saturating_sub(self.viewport_rows - 1);
        }

        // Adjust column scrolling
        if col < scroll_col {
            scroll_col = col;
        } else if col >= scroll_col + self.viewport_columns {
            scroll_col = col.saturating_sub(self.viewport_columns - 1);
        }

        if self.scroll_offset != (scroll_row, scroll_col) {
            info!(target: "navigation", "NavigationState::ensure_visible - scroll_offset: {:?} -> {:?}", 
                  self.scroll_offset, (scroll_row, scroll_col));
            self.scroll_offset = (scroll_row, scroll_col);
        }
    }

    /// Check if cursor is at top of viewport
    pub fn is_at_viewport_top(&self) -> bool {
        self.selected_row == self.scroll_offset.0
    }

    /// Check if cursor is at bottom of viewport
    pub fn is_at_viewport_bottom(&self) -> bool {
        self.selected_row == self.scroll_offset.0 + self.viewport_rows - 1
    }

    /// Get position description for status
    pub fn get_position_status(&self) -> String {
        if self.viewport_lock {
            if self.is_at_viewport_top() {
                " (at viewport top)".to_string()
            } else if self.is_at_viewport_bottom() {
                " (at viewport bottom)".to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }

    pub fn add_to_history(&mut self, row: usize, col: usize) {
        // Don't add if it's the same as the last position
        if let Some(&(last_row, last_col)) = self.selection_history.back() {
            if last_row == row && last_col == col {
                return;
            }
        }

        if self.selection_history.len() >= 50 {
            self.selection_history.pop_front();
        }
        self.selection_history.push_back((row, col));
    }
}

impl JumpToRowState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            is_active: false,
        }
    }
}

/// State for column sorting
#[derive(Debug, Clone)]
pub struct SortState {
    /// Currently sorted column index
    pub column: Option<usize>,
    /// Column name (for display)
    pub column_name: Option<String>,
    /// Sort order (Ascending, Descending, None)
    pub order: SortOrder,
    /// History of sort operations
    pub history: VecDeque<SortHistoryEntry>,
    /// Maximum history size
    pub max_history: usize,
    /// Total sorts performed
    pub total_sorts: usize,
    /// Last sort time
    pub last_sort_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct SortHistoryEntry {
    /// Column that was sorted
    pub column_index: usize,
    /// Column name
    pub column_name: String,
    /// Sort order applied
    pub order: SortOrder,
    /// When the sort was performed
    pub sorted_at: Instant,
    /// Number of rows sorted
    pub row_count: usize,
}

impl Default for SortState {
    fn default() -> Self {
        Self::new()
    }
}

impl SortState {
    pub fn new() -> Self {
        Self {
            column: None,
            column_name: None,
            order: SortOrder::None,
            history: VecDeque::with_capacity(20),
            max_history: 20,
            total_sorts: 0,
            last_sort_time: None,
        }
    }

    /// Set sort column and order
    pub fn set_sort(
        &mut self,
        column_index: usize,
        column_name: String,
        order: SortOrder,
        row_count: usize,
    ) {
        // Add to history
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }

        self.history.push_back(SortHistoryEntry {
            column_index,
            column_name: column_name.clone(),
            order: order.clone(),
            sorted_at: Instant::now(),
            row_count,
        });

        // Update current state
        self.column = Some(column_index);
        self.column_name = Some(column_name);
        self.order = order;
        self.total_sorts += 1;
        self.last_sort_time = Some(Instant::now());
    }

    /// Clear sort (return to original order)
    pub fn clear_sort(&mut self) {
        self.column = None;
        self.column_name = None;
        self.order = SortOrder::None;
        self.last_sort_time = Some(Instant::now());
    }

    /// Get the next sort order for a column
    pub fn get_next_order(&self, column_index: usize) -> SortOrder {
        let next_order = if let Some(current_col) = self.column {
            if current_col == column_index {
                // Same column - cycle through orders
                match self.order {
                    SortOrder::None => SortOrder::Ascending,
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::None,
                }
            } else {
                // Different column - start with ascending
                SortOrder::Ascending
            }
        } else {
            // No column sorted - start with ascending
            SortOrder::Ascending
        };

        // Debug: GET_NEXT_ORDER calculation
        next_order
    }

    /// Advance the sort state for the given column
    pub fn advance_sort_state(
        &mut self,
        column_index: usize,
        column_name: Option<String>,
        new_order: SortOrder,
    ) {
        // Update history before changing state
        if let (Some(col), Some(name)) = (self.column, &self.column_name) {
            self.history.push_back(SortHistoryEntry {
                column_index: col,
                column_name: name.clone(),
                order: self.order.clone(),
                sorted_at: std::time::Instant::now(),
                row_count: 0, // We don't track row count here, could be added later
            });
        }

        // Update statistics
        self.total_sorts += 1;

        // Update current state
        if new_order == SortOrder::None {
            self.column = None;
            self.column_name = None;
        } else {
            self.column = Some(column_index);
            self.column_name = column_name;
        }
        self.order = new_order;
        self.last_sort_time = Some(std::time::Instant::now());
    }

    /// Get sort statistics
    pub fn get_stats(&self) -> String {
        let current = if let (Some(col), Some(name)) = (self.column, &self.column_name) {
            format!(
                "Column {} ({}) {}",
                col,
                name,
                match self.order {
                    SortOrder::Ascending => "↑",
                    SortOrder::Descending => "↓",
                    SortOrder::None => "-",
                }
            )
        } else {
            "None".to_string()
        };

        format!(
            "Current: {}, Total sorts: {}, History items: {}",
            current,
            self.total_sorts,
            self.history.len()
        )
    }
}

/// Selection mode for results view
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionMode {
    Row,
    Cell,
    Column,
}

/// Selection state for managing row/cell/column selections
#[derive(Debug, Clone)]
pub struct SelectionState {
    /// Current selection mode
    pub mode: SelectionMode,
    /// Currently selected row (for table navigation)
    pub selected_row: Option<usize>,
    /// Currently selected column (always tracked)
    pub selected_column: usize,
    /// Selected cells for multi-cell operations
    pub selected_cells: Vec<(usize, usize)>,
    /// Selection anchor for range selections
    pub selection_anchor: Option<(usize, usize)>,
    /// Selection history for undo
    pub history: VecDeque<SelectionHistoryEntry>,
    /// Maximum history size
    pub max_history: usize,
    /// Total selections made
    pub total_selections: usize,
    /// Last selection time
    pub last_selection_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct SelectionHistoryEntry {
    pub mode: SelectionMode,
    pub row: Option<usize>,
    pub column: usize,
    pub cells: Vec<(usize, usize)>,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

impl SelectionState {
    pub fn new() -> Self {
        Self {
            mode: SelectionMode::Row,
            selected_row: None,
            selected_column: 0,
            selected_cells: Vec::new(),
            selection_anchor: None,
            history: VecDeque::new(),
            max_history: 50,
            total_selections: 0,
            last_selection_time: None,
        }
    }

    /// Set selection mode
    pub fn set_mode(&mut self, mode: SelectionMode) {
        if self.mode != mode {
            // Save to history before changing
            self.save_to_history();
            self.mode = mode;
            // Clear multi-cell selections when changing modes
            self.selected_cells.clear();
            self.selection_anchor = None;
        }
    }

    /// Select a row
    pub fn select_row(&mut self, row: Option<usize>) {
        if self.selected_row != row {
            self.save_to_history();
            self.selected_row = row;
            self.total_selections += 1;
            self.last_selection_time = Some(Instant::now());
        }
    }

    /// Select a column
    pub fn select_column(&mut self, column: usize) {
        if self.selected_column != column {
            self.save_to_history();
            self.selected_column = column;
            self.total_selections += 1;
            self.last_selection_time = Some(Instant::now());
        }
    }

    /// Select a cell
    pub fn select_cell(&mut self, row: usize, column: usize) {
        self.save_to_history();
        self.selected_row = Some(row);
        self.selected_column = column;
        self.total_selections += 1;
        self.last_selection_time = Some(Instant::now());
    }

    /// Add cell to multi-selection
    pub fn add_cell_to_selection(&mut self, row: usize, column: usize) {
        let cell = (row, column);
        if !self.selected_cells.contains(&cell) {
            self.selected_cells.push(cell);
            self.total_selections += 1;
            self.last_selection_time = Some(Instant::now());
        }
    }

    /// Clear all selections
    pub fn clear_selections(&mut self) {
        self.save_to_history();
        self.selected_cells.clear();
        self.selection_anchor = None;
    }

    /// Save current state to history
    fn save_to_history(&mut self) {
        let entry = SelectionHistoryEntry {
            mode: self.mode.clone(),
            row: self.selected_row,
            column: self.selected_column,
            cells: self.selected_cells.clone(),
            timestamp: chrono::Local::now(),
        };

        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(entry);
    }

    /// Get selection statistics
    pub fn get_stats(&self) -> String {
        let mode_str = match self.mode {
            SelectionMode::Row => "Row",
            SelectionMode::Cell => "Cell",
            SelectionMode::Column => "Column",
        };

        let selection_str = match (self.selected_row, self.selected_cells.len()) {
            (Some(row), 0) => format!("Row {}, Col {}", row, self.selected_column),
            (_, n) if n > 0 => format!("{} cells selected", n),
            _ => format!("Col {}", self.selected_column),
        };

        format!(
            "Mode: {}, Selection: {}, Total: {}",
            mode_str, selection_str, self.total_selections
        )
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

/// Centralized query results state management
#[derive(Debug, Clone)]
pub struct ResultsState {
    /// Current query results for active buffer
    pub current_results: Option<QueryResponse>,

    /// Results cache with LRU behavior
    pub results_cache: HashMap<String, CachedResult>,

    /// Maximum cache size (number of queries)
    pub max_cache_size: usize,

    /// Memory usage tracking
    pub total_memory_usage: usize,

    /// Memory limit in bytes
    pub memory_limit: usize,

    /// Last query executed
    pub last_query: String,

    /// Last query execution time
    pub last_execution_time: Duration,

    /// Query history for performance analysis
    pub query_performance_history: VecDeque<QueryPerformance>,

    /// Whether results are from cache
    pub from_cache: bool,

    /// Last modification timestamp
    pub last_modified: Instant,
}

#[derive(Debug, Clone)]
pub struct CachedResult {
    /// The actual query response
    pub response: QueryResponse,

    /// When this result was cached
    pub cached_at: Instant,

    /// How often this result was accessed (for LRU)
    pub access_count: u32,

    /// Last access time (for LRU)
    pub last_access: Instant,

    /// Memory size of this result
    pub memory_size: usize,
}

#[derive(Debug, Clone)]
pub struct QueryPerformance {
    /// The query that was executed
    pub query: String,

    /// Execution time
    pub execution_time: Duration,

    /// Number of rows returned
    pub row_count: usize,

    /// Whether result came from cache
    pub from_cache: bool,

    /// Memory usage
    pub memory_usage: usize,

    /// Timestamp of execution
    pub executed_at: Instant,
}

impl Default for ResultsState {
    fn default() -> Self {
        Self {
            current_results: None,
            results_cache: HashMap::new(),
            max_cache_size: 100, // Cache up to 100 queries
            total_memory_usage: 0,
            memory_limit: 512 * 1024 * 1024, // 512MB limit
            last_query: String::new(),
            last_execution_time: Duration::from_millis(0),
            query_performance_history: VecDeque::with_capacity(1000),
            from_cache: false,
            last_modified: Instant::now(),
        }
    }
}

/// Clipboard/Yank state management
#[derive(Debug, Clone)]
pub struct ClipboardState {
    /// Last yanked item (description, full_value, preview)
    pub last_yanked: Option<YankedItem>,

    /// History of yanked items
    pub yank_history: VecDeque<YankedItem>,

    /// Maximum history size
    pub max_history: usize,

    /// Current yank register (for multi-register support in future)
    pub current_register: char,

    /// Statistics
    pub total_yanks: usize,
    pub last_yank_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct YankedItem {
    /// Description of what was yanked (e.g., "cell at [2,3]", "row 5", "column 'name'")
    pub description: String,

    /// The full value that was yanked
    pub full_value: String,

    /// Preview of the value (truncated for display)
    pub preview: String,

    /// Type of yank operation
    pub yank_type: YankType,

    /// When this was yanked
    pub yanked_at: DateTime<Local>,

    /// Size in bytes
    pub size_bytes: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum YankType {
    Cell {
        row: usize,
        column: usize,
    },
    Row {
        row: usize,
    },
    Column {
        name: String,
        index: usize,
    },
    All,
    Selection {
        start: (usize, usize),
        end: (usize, usize),
    },
    TestCase,
    DebugContext,
}

impl Default for ClipboardState {
    fn default() -> Self {
        Self {
            last_yanked: None,
            yank_history: VecDeque::with_capacity(50),
            max_history: 50,
            current_register: '"', // Default register like vim
            total_yanks: 0,
            last_yank_time: None,
        }
    }
}

impl ClipboardState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new yanked item
    pub fn add_yank(&mut self, item: YankedItem) {
        // Add to history
        self.yank_history.push_front(item.clone());

        // Trim history if needed
        while self.yank_history.len() > self.max_history {
            self.yank_history.pop_back();
        }

        // Update current
        self.last_yanked = Some(item);
        self.total_yanks += 1;
        self.last_yank_time = Some(Instant::now());
    }

    /// Clear clipboard
    pub fn clear(&mut self) {
        self.last_yanked = None;
    }

    /// Clear all history
    pub fn clear_history(&mut self) {
        self.yank_history.clear();
        self.last_yanked = None;
    }

    /// Get clipboard statistics
    pub fn get_stats(&self) -> String {
        format!(
            "Total yanks: {}, History items: {}, Last yank: {}",
            self.total_yanks,
            self.yank_history.len(),
            self.last_yank_time
                .map(|t| format!("{:?} ago", t.elapsed()))
                .unwrap_or_else(|| "never".to_string())
        )
    }
}

impl ResultsState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set current query results with performance tracking
    pub fn set_results(
        &mut self,
        results: QueryResponse,
        execution_time: Duration,
        from_cache: bool,
    ) -> Result<()> {
        let row_count = results.count;
        let memory_usage = self.estimate_memory_usage(&results);

        // Record performance metrics
        let performance = QueryPerformance {
            query: results.query.select.join(", "),
            execution_time,
            row_count,
            from_cache,
            memory_usage,
            executed_at: Instant::now(),
        };

        // Add to performance history (keep last 1000)
        self.query_performance_history.push_back(performance);
        if self.query_performance_history.len() > 1000 {
            self.query_performance_history.pop_front();
        }

        // Update state
        self.current_results = Some(results);
        self.last_execution_time = execution_time;
        self.from_cache = from_cache;
        self.last_modified = Instant::now();

        Ok(())
    }

    /// Get current results
    pub fn get_results(&self) -> Option<&QueryResponse> {
        self.current_results.as_ref()
    }

    /// Cache query results with LRU management
    pub fn cache_results(&mut self, query_key: String, results: QueryResponse) -> Result<()> {
        let memory_usage = self.estimate_memory_usage(&results);

        // Check memory limit
        if self.total_memory_usage + memory_usage > self.memory_limit {
            self.evict_to_fit(memory_usage)?;
        }

        // Create cached result
        let cached_result = CachedResult {
            response: results,
            cached_at: Instant::now(),
            access_count: 1,
            last_access: Instant::now(),
            memory_size: memory_usage,
        };

        // Remove oldest if at capacity
        if self.results_cache.len() >= self.max_cache_size {
            self.evict_oldest()?;
        }

        self.results_cache.insert(query_key, cached_result);
        self.total_memory_usage += memory_usage;

        Ok(())
    }

    /// Get cached results
    pub fn get_cached_results(&mut self, query_key: &str) -> Option<&QueryResponse> {
        if let Some(cached) = self.results_cache.get_mut(query_key) {
            cached.access_count += 1;
            cached.last_access = Instant::now();
            Some(&cached.response)
        } else {
            None
        }
    }

    /// Clear all cached results
    pub fn clear_cache(&mut self) {
        self.results_cache.clear();
        self.total_memory_usage = 0;
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.results_cache.len(),
            memory_usage: self.total_memory_usage,
            memory_limit: self.memory_limit,
            hit_rate: self.calculate_hit_rate(),
        }
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let total_queries = self.query_performance_history.len();
        let cached_queries = self
            .query_performance_history
            .iter()
            .filter(|q| q.from_cache)
            .count();
        let avg_execution_time = if total_queries > 0 {
            self.query_performance_history
                .iter()
                .map(|q| q.execution_time.as_millis() as f64)
                .sum::<f64>()
                / total_queries as f64
        } else {
            0.0
        };

        PerformanceStats {
            total_queries,
            cached_queries,
            cache_hit_rate: if total_queries > 0 {
                cached_queries as f64 / total_queries as f64
            } else {
                0.0
            },
            average_execution_time_ms: avg_execution_time,
            last_execution_time: self.last_execution_time,
        }
    }

    // Private helper methods

    fn estimate_memory_usage(&self, results: &QueryResponse) -> usize {
        // Rough estimation of memory usage
        let data_size = results
            .data
            .iter()
            .map(|row| serde_json::to_string(row).unwrap_or_default().len())
            .sum::<usize>();

        // Add overhead for structure
        data_size + std::mem::size_of::<QueryResponse>() + 1024 // Extra overhead
    }

    fn evict_to_fit(&mut self, needed_space: usize) -> Result<()> {
        // Evict least recently used items until we have enough space
        while self.total_memory_usage + needed_space > self.memory_limit
            && !self.results_cache.is_empty()
        {
            self.evict_oldest()?;
        }
        Ok(())
    }

    fn evict_oldest(&mut self) -> Result<()> {
        if let Some((key, cached)) = self
            .results_cache
            .iter()
            .min_by_key(|(_, cached)| cached.last_access)
            .map(|(k, v)| (k.clone(), v.memory_size))
        {
            self.results_cache.remove(&key);
            self.total_memory_usage = self.total_memory_usage.saturating_sub(cached);
        }
        Ok(())
    }

    fn calculate_hit_rate(&self) -> f64 {
        // Simple hit rate based on recent performance history
        let total = self.query_performance_history.len();
        if total == 0 {
            return 0.0;
        }

        let hits = self
            .query_performance_history
            .iter()
            .filter(|q| q.from_cache)
            .count();
        hits as f64 / total as f64
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: usize,
    pub memory_usage: usize,
    pub memory_limit: usize,
    pub hit_rate: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_queries: usize,
    pub cached_queries: usize,
    pub cache_hit_rate: f64,
    pub average_execution_time_ms: f64,
    pub last_execution_time: Duration,
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
    sort: RefCell<SortState>,
    selection: RefCell<SelectionState>,

    // Widget states
    widgets: WidgetStates,

    // UI states
    cache_list: CacheListState,
    column_stats: ColumnStatsState,
    jump_to_row: JumpToRowState,
    navigation: RefCell<NavigationState>,

    // History
    command_history: CommandHistory,
    key_press_history: RefCell<KeyPressHistory>,

    // Results state (centralized query results management)
    results: RefCell<ResultsState>,

    // Clipboard/Yank state
    clipboard: RefCell<ClipboardState>,

    // Legacy results cache (to be deprecated)
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
            sort: RefCell::new(SortState::new()),
            selection: RefCell::new(SelectionState::new()),
            widgets,
            cache_list: CacheListState::new(),
            column_stats: ColumnStatsState::new(),
            jump_to_row: JumpToRowState::new(),
            navigation: RefCell::new(NavigationState::new()),
            command_history,
            key_press_history: RefCell::new(KeyPressHistory::new(50)), // Keep last 50 key presses
            results: RefCell::new(ResultsState::new()),
            clipboard: RefCell::new(ClipboardState::new()),
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

    // Column search operations with logging

    /// Start column search with pattern
    pub fn start_column_search(&self, pattern: String) {
        let mut column_search = self.column_search.borrow_mut();
        let old_pattern = column_search.pattern.clone();
        let old_active = column_search.is_active;

        column_search.pattern = pattern.clone();
        column_search.is_active = true;
        column_search.last_search_time = Some(Instant::now());

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "ColumnSearch",
                format!(
                    "Starting column search: '{}' (was: '{}', active: {})",
                    pattern, old_pattern, old_active
                ),
            );
        }
    }

    /// Update column search matches
    pub fn update_column_search_matches(
        &self,
        columns: &[(String, usize)],
        pattern: &str,
    ) -> Vec<(usize, String)> {
        let pattern_lower = pattern.to_lowercase();
        let mut matches = Vec::new();

        for (name, index) in columns {
            if name.to_lowercase().contains(&pattern_lower) {
                matches.push((*index, name.clone()));
            }
        }

        let mut column_search = self.column_search.borrow_mut();
        column_search.set_matches(matches.clone());

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "ColumnSearch",
                format!(
                    "Found {} columns matching '{}': {:?}",
                    matches.len(),
                    pattern,
                    matches.iter().map(|(_, name)| name).collect::<Vec<_>>()
                ),
            );
        }

        matches
    }

    /// Navigate to next column match
    pub fn next_column_match(&self) -> Option<(usize, String)> {
        let mut column_search = self.column_search.borrow_mut();
        if let Some((idx, name)) = column_search.next_match() {
            let current = column_search.current_match;
            let total = column_search.matching_columns.len();

            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "ColumnSearch",
                    format!(
                        "Navigate to next column: {}/{} - '{}' (index {})",
                        current + 1,
                        total,
                        name,
                        idx
                    ),
                );
            }

            Some((idx, name))
        } else {
            None
        }
    }

    /// Navigate to previous column match
    pub fn previous_column_match(&self) -> Option<(usize, String)> {
        let mut column_search = self.column_search.borrow_mut();
        if let Some((idx, name)) = column_search.prev_match() {
            let current = column_search.current_match;
            let total = column_search.matching_columns.len();

            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "ColumnSearch",
                    format!(
                        "Navigate to previous column: {}/{} - '{}' (index {})",
                        current + 1,
                        total,
                        name,
                        idx
                    ),
                );
            }

            Some((idx, name))
        } else {
            None
        }
    }

    /// Clear column search
    pub fn clear_column_search(&self) {
        let mut column_search = self.column_search.borrow_mut();
        let had_matches = column_search.matching_columns.len();
        let had_pattern = column_search.pattern.clone();

        column_search.clear();

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "ColumnSearch",
                format!(
                    "Cleared column search (had pattern: '{}', {} matches)",
                    had_pattern, had_matches
                ),
            );
        }
    }

    /// Accept current column match
    pub fn accept_column_match(&self) -> Option<(usize, String)> {
        let column_search = self.column_search.borrow();
        if let Some((idx, name)) = column_search.current_match() {
            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "ColumnSearch",
                    format!("Accepted column: '{}' at index {}", name, idx),
                );
            }
            Some((idx, name))
        } else {
            None
        }
    }

    // Sort operations with logging

    /// Sort by column
    pub fn sort_by_column(&self, column_index: usize, column_name: String, row_count: usize) {
        let mut sort_state = self.sort.borrow_mut();

        // Get the next sort order for this column
        let new_order = sort_state.get_next_order(column_index);

        let old_column = sort_state.column;
        let old_order = sort_state.order.clone();

        if new_order == SortOrder::None {
            // Clear sort - return to original order
            sort_state.clear_sort();

            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "Sort",
                    format!(
                        "Cleared sort on column {} ({}), returning to original order",
                        column_index, column_name
                    ),
                );
            }
        } else {
            // Apply sort
            sort_state.set_sort(
                column_index,
                column_name.clone(),
                new_order.clone(),
                row_count,
            );

            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "Sort",
                    format!(
                        "Sorted column {} ({}) {}, {} rows (was: column {:?} {})",
                        column_index,
                        column_name,
                        match new_order {
                            SortOrder::Ascending => "ascending ↑",
                            SortOrder::Descending => "descending ↓",
                            SortOrder::None => "none",
                        },
                        row_count,
                        old_column,
                        match old_order {
                            SortOrder::Ascending => "↑",
                            SortOrder::Descending => "↓",
                            SortOrder::None => "-",
                        }
                    ),
                );
            }
        }
    }

    /// Clear all sorting
    pub fn clear_sort(&self) {
        let mut sort_state = self.sort.borrow_mut();
        let had_sort = sort_state.column.is_some();
        let old_column = sort_state.column;
        let old_name = sort_state.column_name.clone();

        sort_state.clear_sort();

        if had_sort {
            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "Sort",
                    format!(
                        "Cleared all sorting (was: column {:?} - {})",
                        old_column,
                        old_name.unwrap_or_else(|| "unknown".to_string())
                    ),
                );
            }
        }
    }

    /// Get current sort state
    pub fn sort(&self) -> std::cell::Ref<SortState> {
        self.sort.borrow()
    }

    /// Get next sort order for a column
    pub fn get_next_sort_order(&self, column_index: usize) -> SortOrder {
        self.sort.borrow().get_next_order(column_index)
    }

    /// Advance the sort state for a column
    pub fn advance_sort_state(
        &self,
        column_index: usize,
        column_name: Option<String>,
        new_order: SortOrder,
    ) {
        self.sort
            .borrow_mut()
            .advance_sort_state(column_index, column_name, new_order);
    }

    /// Perform sorting on the results data and return sorted results
    pub fn sort_results_data(
        &self,
        column_index: usize,
        sort_order: SortOrder,
    ) -> Option<QueryResponse> {
        let results = self.results.borrow();
        let original_results = results.current_results.as_ref()?;

        if sort_order == SortOrder::None {
            // Return original unsorted data
            return Some(original_results.clone());
        }

        // Get column name from first row
        let first_row = original_results.data.first()?;
        let obj = first_row.as_object()?;
        let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();

        if column_index >= headers.len() {
            return Some(original_results.clone());
        }

        let column_name = headers[column_index];

        // Create a vector of (original_json_row, row_index) pairs for sorting
        let mut indexed_rows: Vec<(serde_json::Value, usize)> = original_results
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
                (Some(serde_json::Value::Number(a)), Some(serde_json::Value::Number(b))) => {
                    // Numeric comparison
                    let a_f64 = a.as_f64().unwrap_or(0.0);
                    let b_f64 = b.as_f64().unwrap_or(0.0);
                    a_f64.partial_cmp(&b_f64).unwrap_or(Ordering::Equal)
                }
                (Some(serde_json::Value::String(a)), Some(serde_json::Value::String(b))) => {
                    // String comparison
                    a.cmp(&b)
                }
                (Some(serde_json::Value::Bool(a)), Some(serde_json::Value::Bool(b))) => {
                    // Boolean comparison (false < true)
                    a.cmp(&b)
                }
                (Some(serde_json::Value::Null), Some(serde_json::Value::Null)) => Ordering::Equal,
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

            match sort_order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
                SortOrder::None => Ordering::Equal,
            }
        });

        // Rebuild the QueryResponse with sorted data
        let sorted_data: Vec<serde_json::Value> =
            indexed_rows.into_iter().map(|(row, _)| row).collect();

        let mut sorted_results = original_results.clone();
        sorted_results.data = sorted_data;

        // Update sort state
        let row_count = sorted_results.data.len();
        self.sort_by_column(column_index, column_name.to_string(), row_count);

        Some(sorted_results)
    }

    // Selection operations with logging

    /// Get current selection state (read-only)
    pub fn selection(&self) -> std::cell::Ref<SelectionState> {
        self.selection.borrow()
    }

    /// Get current selection state (mutable)
    pub fn selection_mut(&self) -> std::cell::RefMut<SelectionState> {
        self.selection.borrow_mut()
    }

    /// Set selection mode
    pub fn set_selection_mode(&self, mode: SelectionMode) {
        let mut selection = self.selection.borrow_mut();
        let old_mode = selection.mode.clone();
        selection.set_mode(mode.clone());

        if old_mode != mode {
            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "Selection",
                    format!("Mode changed: {:?} → {:?}", old_mode, mode),
                );
            }
        }
    }

    /// Select a row
    pub fn select_row(&self, row: Option<usize>) {
        let mut selection = self.selection.borrow_mut();
        let old_row = selection.selected_row;
        selection.select_row(row);

        if old_row != row {
            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "Selection",
                    format!("Row selection: {:?} → {:?}", old_row, row),
                );
            }
        }
    }

    /// Select a column
    pub fn select_column(&self, column: usize) {
        let mut selection = self.selection.borrow_mut();
        let old_column = selection.selected_column;
        selection.select_column(column);

        if old_column != column {
            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info(
                    "Selection",
                    format!("Column selection: {} → {}", old_column, column),
                );
            }
        }
    }

    /// Select a cell
    pub fn select_cell(&self, row: usize, column: usize) {
        self.selection.borrow_mut().select_cell(row, column);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info("Selection", format!("Cell selected: [{}, {}]", row, column));
        }
    }

    /// Toggle selection mode between Row/Cell/Column
    pub fn toggle_selection_mode(&self) {
        let mut selection = self.selection.borrow_mut();
        let new_mode = match selection.mode {
            SelectionMode::Row => SelectionMode::Cell,
            SelectionMode::Cell => SelectionMode::Column,
            SelectionMode::Column => SelectionMode::Row,
        };
        let old_mode = selection.mode.clone();
        selection.set_mode(new_mode.clone());

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Selection",
                format!("Mode toggled: {:?} → {:?}", old_mode, new_mode),
            );
        }
    }

    /// Clear all selections
    pub fn clear_selections(&self) {
        let mut selection = self.selection.borrow_mut();
        let had_selections = !selection.selected_cells.is_empty();
        selection.clear_selections();

        if had_selections {
            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info("Selection", "Cleared all selections".to_string());
            }
        }
    }

    /// Get current selection mode
    pub fn get_selection_mode(&self) -> SelectionMode {
        self.selection.borrow().mode.clone()
    }

    /// Get selected row
    pub fn get_selected_row(&self) -> Option<usize> {
        self.selection.borrow().selected_row
    }

    /// Get selected column
    pub fn get_selected_column(&self) -> usize {
        self.selection.borrow().selected_column
    }

    // History search operations (Ctrl+R)
    pub fn start_history_search(&self, original_input: String) {
        eprintln!(
            "[DEBUG] start_history_search called with input: '{}'",
            original_input
        );

        let mut history_search = self.history_search.borrow_mut();
        history_search.query.clear();
        history_search.matches.clear();
        history_search.selected_index = 0;
        history_search.is_active = true;
        history_search.original_input = original_input;

        // Initialize with all history entries
        let all_entries = self.command_history.get_all();
        eprintln!(
            "[DEBUG] Got {} entries from command_history.get_all()",
            all_entries.len()
        );

        history_search.matches = all_entries
            .iter()
            .cloned()
            .map(|entry| crate::history::HistoryMatch {
                entry,
                indices: Vec::new(),
                score: 0,
            })
            .collect();

        eprintln!(
            "[DEBUG] Created {} matches in history_search",
            history_search.matches.len()
        );

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

    // Navigation operations with logging (V16 implementation)
    pub fn navigate_to(&self, row: usize, col: usize) {
        let mut navigation = self.navigation.borrow_mut();
        let old_row = navigation.selected_row;
        let old_col = navigation.selected_column;

        // Update position
        navigation.selected_row = row.min(navigation.total_rows.saturating_sub(1));
        navigation.selected_column = col.min(navigation.total_columns.saturating_sub(1));

        let new_row = navigation.selected_row;
        let new_col = navigation.selected_column;

        // Add to history
        navigation.add_to_history(new_row, new_col);

        // Ensure position is visible
        navigation.ensure_visible(new_row, new_col);

        let scroll_offset = navigation.scroll_offset;
        drop(navigation);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.log(
                "Navigation",
                DebugLevel::Info,
                format!(
                    "Navigate: ({}, {}) -> ({}, {}), scroll: {:?}",
                    old_row, old_col, new_row, new_col, scroll_offset
                ),
                Some("navigate_to".to_string()),
            );
        }
    }

    pub fn navigate_relative(&self, delta_row: i32, delta_col: i32) {
        let navigation = self.navigation.borrow();
        let current_row = navigation.selected_row;
        let current_col = navigation.selected_column;
        drop(navigation);

        let new_row = if delta_row >= 0 {
            current_row.saturating_add(delta_row as usize)
        } else {
            current_row.saturating_sub(delta_row.abs() as usize)
        };

        let new_col = if delta_col >= 0 {
            current_col.saturating_add(delta_col as usize)
        } else {
            current_col.saturating_sub(delta_col.abs() as usize)
        };

        self.navigate_to(new_row, new_col);
    }

    pub fn navigate_to_row(&self, row: usize) {
        let navigation = self.navigation.borrow();
        let current_col = navigation.selected_column;
        drop(navigation);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.log(
                "Navigation",
                DebugLevel::Info,
                format!("Jump to row: {}", row),
                Some("navigate_to_row".to_string()),
            );
        }

        self.navigate_to(row, current_col);
    }

    pub fn navigate_to_column(&self, col: usize) {
        let navigation = self.navigation.borrow();
        let current_row = navigation.selected_row;
        drop(navigation);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.log(
                "Navigation",
                DebugLevel::Info,
                format!("Jump to column: {}", col),
                Some("navigate_to_column".to_string()),
            );
        }

        self.navigate_to(current_row, col);
    }

    pub fn update_data_size(&self, rows: usize, columns: usize) {
        let mut navigation = self.navigation.borrow_mut();
        let old_totals = (navigation.total_rows, navigation.total_columns);
        navigation.update_totals(rows, columns);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.log(
                "Navigation",
                DebugLevel::Info,
                format!(
                    "Data size updated: {:?} -> ({}, {}), position: ({}, {})",
                    old_totals, rows, columns, navigation.selected_row, navigation.selected_column
                ),
                Some("update_data_size".to_string()),
            );
        }
    }

    pub fn set_viewport_size(&self, rows: usize, columns: usize) {
        let mut navigation = self.navigation.borrow_mut();
        let old_viewport = (navigation.viewport_rows, navigation.viewport_columns);
        let selected_row = navigation.selected_row;
        let selected_column = navigation.selected_column;

        navigation.set_viewport_size(rows, columns);

        // Ensure current position is still visible with new viewport
        navigation.ensure_visible(selected_row, selected_column);

        let scroll_offset = navigation.scroll_offset;
        drop(navigation);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.log(
                "Navigation",
                DebugLevel::Info,
                format!(
                    "Viewport size updated: {:?} -> ({}, {}), scroll adjusted: {:?}",
                    old_viewport, rows, columns, scroll_offset
                ),
                Some("set_viewport_size".to_string()),
            );
        }
    }

    pub fn toggle_viewport_lock(&self) {
        let mut navigation = self.navigation.borrow_mut();
        navigation.viewport_lock = !navigation.viewport_lock;

        if navigation.viewport_lock {
            navigation.viewport_lock_row = Some(navigation.selected_row);
        } else {
            navigation.viewport_lock_row = None;
        }

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.log(
                "Navigation",
                DebugLevel::Info,
                format!(
                    "Viewport lock: {} at row {:?}",
                    navigation.viewport_lock, navigation.viewport_lock_row
                ),
                Some("toggle_viewport_lock".to_string()),
            );
        }
    }

    pub fn toggle_cursor_lock(&self) {
        let mut navigation = self.navigation.borrow_mut();
        navigation.cursor_lock = !navigation.cursor_lock;

        if navigation.cursor_lock {
            // Calculate visual position (position within viewport)
            let visual_position = navigation
                .selected_row
                .saturating_sub(navigation.scroll_offset.0);
            navigation.cursor_lock_position = Some(visual_position);
        } else {
            navigation.cursor_lock_position = None;
        }

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.log(
                "Navigation",
                DebugLevel::Info,
                format!(
                    "Cursor lock: {} at visual position {:?}",
                    navigation.cursor_lock, navigation.cursor_lock_position
                ),
                Some("toggle_cursor_lock".to_string()),
            );
        }
    }

    pub fn is_cursor_locked(&self) -> bool {
        self.navigation.borrow().cursor_lock
    }

    // Navigation state access
    pub fn navigation(&self) -> std::cell::Ref<'_, NavigationState> {
        self.navigation.borrow()
    }

    pub fn navigation_mut(&self) -> std::cell::RefMut<'_, NavigationState> {
        self.navigation.borrow_mut()
    }

    pub fn get_current_position(&self) -> (usize, usize) {
        let navigation = self.navigation.borrow();
        (navigation.selected_row, navigation.selected_column)
    }

    pub fn get_scroll_offset(&self) -> (usize, usize) {
        self.navigation.borrow().scroll_offset
    }

    pub fn is_viewport_locked(&self) -> bool {
        self.navigation.borrow().viewport_lock
    }

    // Results state methods (V17 implementation)
    /// Set query results with comprehensive logging and performance tracking
    pub fn set_results(
        &self,
        results: QueryResponse,
        execution_time: Duration,
        from_cache: bool,
    ) -> Result<()> {
        let query_text = results.query.select.join(", ");
        let row_count = results.count;

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.log(
                "ResultsState",
                DebugLevel::Info,
                format!(
                    "[RESULTS] Setting results: query='{}', rows={}, time={}ms, cached={}",
                    query_text.chars().take(50).collect::<String>(),
                    row_count,
                    execution_time.as_millis(),
                    from_cache
                ),
                Some("set_results".to_string()),
            );
        }

        self.results
            .borrow_mut()
            .set_results(results, execution_time, from_cache)?;

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            let stats = self.results.borrow().get_performance_stats();
            debug_service.log(
                "ResultsState",
                DebugLevel::Info,
                format!(
                    "[RESULTS] Performance stats: total_queries={}, cache_hit_rate={:.2}%, avg_time={:.2}ms",
                    stats.total_queries,
                    stats.cache_hit_rate * 100.0,
                    stats.average_execution_time_ms
                ),
                Some("performance_stats".to_string()),
            );
        }

        Ok(())
    }

    /// Get current query results
    pub fn get_results(&self) -> Option<QueryResponse> {
        self.results.borrow().get_results().cloned()
    }

    /// Cache query results with logging
    pub fn cache_results(&self, query_key: String, results: QueryResponse) -> Result<()> {
        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.log(
                "ResultsCache",
                DebugLevel::Info,
                format!(
                    "[RESULTS] Caching results: key='{}', rows={}",
                    query_key.chars().take(30).collect::<String>(),
                    results.count
                ),
                Some("cache_results".to_string()),
            );
        }

        let result = self
            .results
            .borrow_mut()
            .cache_results(query_key.clone(), results);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            let cache_stats = self.results.borrow().get_cache_stats();
            debug_service.log(
                "ResultsCache",
                DebugLevel::Info,
                format!(
                    "[RESULTS] Cache stats: entries={}, memory={}MB, hit_rate={:.2}%",
                    cache_stats.entry_count,
                    cache_stats.memory_usage / (1024 * 1024),
                    cache_stats.hit_rate * 100.0
                ),
                Some("cache_stats".to_string()),
            );
        }

        result
    }

    /// Get cached results with access tracking
    pub fn get_cached_results(&self, query_key: &str) -> Option<QueryResponse> {
        if let Some(result) = self.results.borrow_mut().get_cached_results(query_key) {
            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.log(
                    "ResultsCache",
                    DebugLevel::Trace,
                    format!(
                        "[RESULTS] Cache HIT for key: '{}'",
                        query_key.chars().take(30).collect::<String>()
                    ),
                    Some("cache_hit".to_string()),
                );
            }
            Some(result.clone())
        } else {
            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.log(
                    "ResultsCache",
                    DebugLevel::Trace,
                    format!(
                        "[RESULTS] Cache MISS for key: '{}'",
                        query_key.chars().take(30).collect::<String>()
                    ),
                    Some("cache_miss".to_string()),
                );
            }
            None
        }
    }

    /// Clear results cache
    pub fn clear_results_cache(&self) {
        let before_count = self.results.borrow().get_cache_stats().entry_count;
        self.results.borrow_mut().clear_cache();

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.log(
                "ResultsCache",
                DebugLevel::Info,
                format!("[RESULTS] Cache cleared: removed {} entries", before_count),
                Some("clear_cache".to_string()),
            );
        }
    }

    // Clipboard operations with logging

    /// Get clipboard state (read-only)
    pub fn clipboard(&self) -> std::cell::Ref<'_, ClipboardState> {
        self.clipboard.borrow()
    }

    /// Get clipboard state (mutable)
    pub fn clipboard_mut(&self) -> std::cell::RefMut<'_, ClipboardState> {
        self.clipboard.borrow_mut()
    }

    /// Yank a cell to clipboard
    pub fn yank_cell(&self, row: usize, column: usize, value: String, preview: String) {
        let description = format!("cell at [{}, {}]", row, column);
        let size_bytes = value.len();

        let item = YankedItem {
            description: description.clone(),
            full_value: value.clone(),
            preview: preview.clone(),
            yank_type: YankType::Cell { row, column },
            yanked_at: Local::now(),
            size_bytes,
        };

        self.clipboard.borrow_mut().add_yank(item);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Clipboard",
                format!(
                    "Yanked {}: '{}' ({} bytes)",
                    description,
                    if preview.len() > 50 {
                        format!("{}...", &preview[..50])
                    } else {
                        preview
                    },
                    size_bytes
                ),
            );
        }
    }

    /// Yank a row to clipboard
    pub fn yank_row(&self, row: usize, value: String, preview: String) {
        let description = format!("row {}", row);
        let size_bytes = value.len();

        let item = YankedItem {
            description: description.clone(),
            full_value: value.clone(),
            preview: preview.clone(),
            yank_type: YankType::Row { row },
            yanked_at: Local::now(),
            size_bytes,
        };

        self.clipboard.borrow_mut().add_yank(item);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Clipboard",
                format!(
                    "Yanked {}: {} columns ({} bytes)",
                    description,
                    value.split('\t').count(),
                    size_bytes
                ),
            );
        }
    }

    /// Yank a column to clipboard
    pub fn yank_column(
        &self,
        column_name: String,
        column_index: usize,
        value: String,
        preview: String,
    ) {
        let description = format!("column '{}'", column_name);
        let size_bytes = value.len();
        let row_count = value.lines().count();

        let item = YankedItem {
            description: description.clone(),
            full_value: value.clone(),
            preview: preview.clone(),
            yank_type: YankType::Column {
                name: column_name.clone(),
                index: column_index,
            },
            yanked_at: Local::now(),
            size_bytes,
        };

        self.clipboard.borrow_mut().add_yank(item);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Clipboard",
                format!(
                    "Yanked {}: {} rows ({} bytes)",
                    description, row_count, size_bytes
                ),
            );
        }
    }

    /// Yank all data to clipboard
    pub fn yank_all(&self, value: String, preview: String) {
        let size_bytes = value.len();
        let row_count = value.lines().count();

        let item = YankedItem {
            description: "all data".to_string(),
            full_value: value.clone(),
            preview: preview.clone(),
            yank_type: YankType::All,
            yanked_at: Local::now(),
            size_bytes,
        };

        self.clipboard.borrow_mut().add_yank(item);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Clipboard",
                format!("Yanked all data: {} rows ({} bytes)", row_count, size_bytes),
            );
        }
    }

    /// Yank a test case to clipboard
    pub fn yank_test_case(&self, value: String) {
        let size_bytes = value.len();
        let line_count = value.lines().count();

        let item = YankedItem {
            description: "Test Case".to_string(),
            full_value: value.clone(),
            preview: format!("{} lines of test case", line_count),
            yank_type: YankType::TestCase,
            yanked_at: Local::now(),
            size_bytes,
        };

        self.clipboard.borrow_mut().add_yank(item);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Clipboard",
                format!(
                    "Yanked test case: {} lines ({} bytes)",
                    line_count, size_bytes
                ),
            );
        }
    }

    /// Yank debug context to clipboard
    pub fn yank_debug_context(&self, value: String) {
        let size_bytes = value.len();
        let line_count = value.lines().count();

        let item = YankedItem {
            description: "Debug Context".to_string(),
            full_value: value.clone(),
            preview: "Query context with data for test creation".to_string(),
            yank_type: YankType::DebugContext,
            yanked_at: Local::now(),
            size_bytes,
        };

        self.clipboard.borrow_mut().add_yank(item);

        if let Some(ref debug_service) = *self.debug_service.borrow() {
            debug_service.info(
                "Clipboard",
                format!(
                    "Yanked debug context: {} lines ({} bytes)",
                    line_count, size_bytes
                ),
            );
        }
    }

    /// Clear clipboard
    pub fn clear_clipboard(&self) {
        let had_item = self.clipboard.borrow().last_yanked.is_some();
        self.clipboard.borrow_mut().clear();

        if had_item {
            if let Some(ref debug_service) = *self.debug_service.borrow() {
                debug_service.info("Clipboard", "Clipboard cleared".to_string());
            }
        }
    }

    /// Get clipboard statistics for debug display
    pub fn get_clipboard_stats(&self) -> String {
        self.clipboard.borrow().get_stats()
    }

    /// Get comprehensive results statistics
    pub fn get_results_stats(&self) -> (CacheStats, PerformanceStats) {
        let results = self.results.borrow();
        (results.get_cache_stats(), results.get_performance_stats())
    }

    /// Check if current results are from cache
    pub fn is_results_from_cache(&self) -> bool {
        self.results.borrow().from_cache
    }

    /// Get last query execution time
    pub fn get_last_execution_time(&self) -> Duration {
        self.results.borrow().last_execution_time
    }

    /// Get memory usage information
    pub fn get_results_memory_usage(&self) -> (usize, usize) {
        let cache_stats = self.results.borrow().get_cache_stats();
        (cache_stats.memory_usage, cache_stats.memory_limit)
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
                | KeyCode::Char(':')
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
        dump.push_str("FILTER STATE:\n");
        let filter = self.filter.borrow();
        if filter.is_active {
            dump.push_str(&format!("  Pattern: '{}'\n", filter.pattern));
            dump.push_str(&format!(
                "  Filtered Rows: {}\n",
                filter.filtered_indices.len()
            ));
            dump.push_str(&format!(
                "  Case Insensitive: {}\n",
                filter.case_insensitive
            ));
            if let Some(ref last_time) = filter.last_filter_time {
                dump.push_str(&format!("  Last Filter: {:?} ago\n", last_time.elapsed()));
            }
        } else {
            dump.push_str("  [Inactive]\n");
        }
        dump.push_str(&format!("  Total Filters: {}\n", filter.total_filters));
        dump.push_str(&format!("  History Items: {}\n", filter.history.len()));
        if !filter.history.is_empty() {
            dump.push_str("  Recent filters:\n");
            for (i, entry) in filter.history.iter().take(5).enumerate() {
                dump.push_str(&format!(
                    "    {}. '{}' ({} matches) at {}\n",
                    i + 1,
                    if entry.pattern.len() > 30 {
                        format!("{}...", &entry.pattern[..30])
                    } else {
                        entry.pattern.clone()
                    },
                    entry.match_count,
                    entry.timestamp.format("%H:%M:%S")
                ));
            }
        }
        dump.push_str("\n");

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

        // Navigation state with enhanced viewport information
        let navigation = self.navigation.borrow();
        dump.push_str("NAVIGATION STATE:\n");
        dump.push_str(&format!(
            "  Cursor Position: row={}, col={}\n",
            navigation.selected_row, navigation.selected_column
        ));
        dump.push_str(&format!(
            "  Scroll Offset: row={}, col={}\n",
            navigation.scroll_offset.0, navigation.scroll_offset.1
        ));
        dump.push_str(&format!(
            "  Viewport Dimensions: {} rows x {} cols\n",
            navigation.viewport_rows, navigation.viewport_columns
        ));
        dump.push_str(&format!(
            "  Data Size: {} rows x {} cols\n",
            navigation.total_rows, navigation.total_columns
        ));

        // Viewport boundary analysis
        dump.push_str("\nVIEWPORT BOUNDARIES:\n");
        let at_top = navigation.selected_row == 0;
        let at_bottom = navigation.selected_row == navigation.total_rows.saturating_sub(1);
        let at_left = navigation.selected_column == 0;
        let at_right = navigation.selected_column == navigation.total_columns.saturating_sub(1);

        dump.push_str(&format!("  At Top Edge: {}\n", at_top));
        dump.push_str(&format!("  At Bottom Edge: {}\n", at_bottom));
        dump.push_str(&format!("  At Left Edge: {}\n", at_left));
        dump.push_str(&format!("  At Right Edge: {}\n", at_right));

        // Scrolling state
        let viewport_bottom = navigation.scroll_offset.0 + navigation.viewport_rows;
        let viewport_right = navigation.scroll_offset.1 + navigation.viewport_columns;
        let should_scroll_down = navigation.selected_row >= viewport_bottom.saturating_sub(1);
        let should_scroll_up = navigation.selected_row < navigation.scroll_offset.0;
        let should_scroll_right = navigation.selected_column >= viewport_right.saturating_sub(1);
        let should_scroll_left = navigation.selected_column < navigation.scroll_offset.1;

        dump.push_str("\nSCROLLING STATE:\n");
        dump.push_str(&format!(
            "  Visible Row Range: {} to {}\n",
            navigation.scroll_offset.0,
            viewport_bottom.min(navigation.total_rows).saturating_sub(1)
        ));
        dump.push_str(&format!(
            "  Visible Col Range: {} to {}\n",
            navigation.scroll_offset.1,
            viewport_right
                .min(navigation.total_columns)
                .saturating_sub(1)
        ));
        dump.push_str(&format!(
            "  Should Scroll Down: {} (cursor at {}, viewport bottom at {})\n",
            should_scroll_down,
            navigation.selected_row,
            viewport_bottom.saturating_sub(1)
        ));
        dump.push_str(&format!(
            "  Should Scroll Up: {} (cursor at {}, viewport top at {})\n",
            should_scroll_up, navigation.selected_row, navigation.scroll_offset.0
        ));
        dump.push_str(&format!("  Should Scroll Right: {}\n", should_scroll_right));
        dump.push_str(&format!("  Should Scroll Left: {}\n", should_scroll_left));

        dump.push_str(&format!(
            "\n  Viewport Lock: {} at row {:?}\n",
            navigation.viewport_lock, navigation.viewport_lock_row
        ));
        dump.push_str(&format!(
            "  Cursor Lock: {} at visual position {:?}\n",
            navigation.cursor_lock, navigation.cursor_lock_position
        ));

        if !navigation.selection_history.is_empty() {
            dump.push_str("\n  Recent positions:\n");
            for (i, &(row, col)) in navigation
                .selection_history
                .iter()
                .rev()
                .take(5)
                .enumerate()
            {
                dump.push_str(&format!("    {}. ({}, {})\n", i + 1, row, col));
            }
        }
        dump.push_str("\n");

        // Column search state
        dump.push_str("COLUMN SEARCH STATE:\n");
        let column_search = self.column_search.borrow();
        if column_search.is_active {
            dump.push_str(&format!("  Pattern: '{}'\n", column_search.pattern));
            dump.push_str(&format!(
                "  Matches: {} columns found\n",
                column_search.matching_columns.len()
            ));
            if !column_search.matching_columns.is_empty() {
                dump.push_str(&format!(
                    "  Current: {} of {}\n",
                    column_search.current_match + 1,
                    column_search.matching_columns.len()
                ));
                dump.push_str("  Matching columns:\n");
                for (i, (idx, name)) in column_search.matching_columns.iter().enumerate() {
                    dump.push_str(&format!(
                        "    {}[{}] {} (index {})\n",
                        if i == column_search.current_match {
                            "*"
                        } else {
                            " "
                        },
                        i + 1,
                        name,
                        idx
                    ));
                }
            }
            if let Some(ref last_time) = column_search.last_search_time {
                dump.push_str(&format!("  Search time: {:?}\n", last_time.elapsed()));
            }
        } else {
            dump.push_str("  [Inactive]\n");
        }
        dump.push_str(&format!(
            "  Total searches: {}\n",
            column_search.total_searches
        ));
        dump.push_str(&format!(
            "  History items: {}\n",
            column_search.history.len()
        ));
        if !column_search.history.is_empty() {
            dump.push_str("  Recent searches:\n");
            for (i, entry) in column_search.history.iter().take(5).enumerate() {
                dump.push_str(&format!(
                    "    {}. '{}' ({} matches) at {}\n",
                    i + 1,
                    entry.pattern,
                    entry.match_count,
                    entry.timestamp.format("%H:%M:%S")
                ));
            }
        }
        dump.push_str("\n");

        // Sort state
        dump.push_str("SORT STATE:\n");
        let sort = self.sort.borrow();
        if let (Some(col), Some(name)) = (sort.column, &sort.column_name) {
            dump.push_str(&format!(
                "  Current: Column {} ({}) {}\n",
                col,
                name,
                match sort.order {
                    SortOrder::Ascending => "Ascending ↑",
                    SortOrder::Descending => "Descending ↓",
                    SortOrder::None => "None",
                }
            ));
        } else {
            dump.push_str("  Current: No sorting applied\n");
        }
        if let Some(ref last_time) = sort.last_sort_time {
            dump.push_str(&format!("  Last sort: {:?} ago\n", last_time.elapsed()));
        }
        dump.push_str(&format!("  Total sorts: {}\n", sort.total_sorts));
        dump.push_str(&format!("  History items: {}\n", sort.history.len()));
        if !sort.history.is_empty() {
            dump.push_str("  Recent sorts:\n");
            for (i, entry) in sort.history.iter().rev().take(5).enumerate() {
                dump.push_str(&format!(
                    "    {}. Column {} ({}) {} - {} rows\n",
                    i + 1,
                    entry.column_index,
                    entry.column_name,
                    match entry.order {
                        SortOrder::Ascending => "↑",
                        SortOrder::Descending => "↓",
                        SortOrder::None => "-",
                    },
                    entry.row_count
                ));
            }
        }
        dump.push_str("\n");

        // Selection state
        dump.push_str("SELECTION STATE:\n");
        let selection = self.selection.borrow();
        dump.push_str(&format!("  Mode: {:?}\n", selection.mode));
        if let Some(row) = selection.selected_row {
            dump.push_str(&format!("  Selected Row: {}\n", row));
        } else {
            dump.push_str("  Selected Row: None\n");
        }
        dump.push_str(&format!(
            "  Selected Column: {}\n",
            selection.selected_column
        ));
        if !selection.selected_cells.is_empty() {
            dump.push_str(&format!(
                "  Selected Cells: {} cells\n",
                selection.selected_cells.len()
            ));
            if selection.selected_cells.len() <= 5 {
                for (row, col) in &selection.selected_cells {
                    dump.push_str(&format!("    - ({}, {})\n", row, col));
                }
            } else {
                for (row, col) in selection.selected_cells.iter().take(3) {
                    dump.push_str(&format!("    - ({}, {})\n", row, col));
                }
                dump.push_str(&format!(
                    "    ... and {} more\n",
                    selection.selected_cells.len() - 3
                ));
            }
        }
        if let Some((row, col)) = selection.selection_anchor {
            dump.push_str(&format!("  Selection Anchor: ({}, {})\n", row, col));
        }
        dump.push_str(&format!(
            "  Total Selections: {}\n",
            selection.total_selections
        ));
        if let Some(ref last_time) = selection.last_selection_time {
            dump.push_str(&format!(
                "  Last Selection: {:?} ago\n",
                last_time.elapsed()
            ));
        }
        dump.push_str(&format!("  History Items: {}\n", selection.history.len()));
        if !selection.history.is_empty() {
            dump.push_str("  Recent selections:\n");
            for (i, entry) in selection.history.iter().rev().take(5).enumerate() {
                dump.push_str(&format!(
                    "    {}. {:?} mode at {}\n",
                    i + 1,
                    entry.mode,
                    entry.timestamp.format("%H:%M:%S")
                ));
            }
        }
        dump.push_str("\n");

        // Clipboard state
        dump.push_str("CLIPBOARD STATE:\n");
        let clipboard = self.clipboard.borrow();
        if let Some(ref yanked) = clipboard.last_yanked {
            dump.push_str(&format!("  Last Yanked: {}\n", yanked.description));
            dump.push_str(&format!("  Type: {:?}\n", yanked.yank_type));
            dump.push_str(&format!("  Size: {} bytes\n", yanked.size_bytes));
            dump.push_str(&format!(
                "  Preview: {}\n",
                if yanked.preview.len() > 60 {
                    format!("{}...", &yanked.preview[..60])
                } else {
                    yanked.preview.clone()
                }
            ));
            dump.push_str(&format!(
                "  Yanked at: {}\n",
                yanked.yanked_at.format("%H:%M:%S")
            ));
        } else {
            dump.push_str("  [Empty]\n");
        }
        dump.push_str(&format!("  Total yanks: {}\n", clipboard.total_yanks));
        dump.push_str(&format!(
            "  History items: {}\n",
            clipboard.yank_history.len()
        ));
        if !clipboard.yank_history.is_empty() {
            dump.push_str("  Recent yanks:\n");
            for (i, item) in clipboard.yank_history.iter().take(5).enumerate() {
                dump.push_str(&format!(
                    "    {}. {} ({} bytes) at {}\n",
                    i + 1,
                    item.description,
                    item.size_bytes,
                    item.yanked_at.format("%H:%M:%S")
                ));
            }
        }
        dump.push_str("\n");

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
