use crate::api_client::QueryResponse;
use crate::csv_datasource::CsvApiClient;
use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::widgets::TableState;
use regex::Regex;
use serde_json::Value;
use std::path::PathBuf;
use tui_input::Input;
use tui_textarea::TextArea;

// Re-define the types we need (these should eventually be moved to a common module)
#[derive(Clone, Debug, PartialEq)]
pub enum AppMode {
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
pub enum EditMode {
    SingleLine,
    MultiLine,
}

#[derive(Clone, PartialEq, Copy, Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
    None,
}

#[derive(Clone)]
pub struct SortState {
    pub column: Option<usize>,
    pub order: SortOrder,
}

#[derive(Clone)]
pub struct FilterState {
    pub pattern: String,
    pub regex: Option<Regex>,
    pub active: bool,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            regex: None,
            active: false,
        }
    }
}

pub struct FuzzyFilterState {
    pub pattern: String,
    pub active: bool,
    pub matcher: SkimMatcherV2,
    pub filtered_indices: Vec<usize>,
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

impl Default for FuzzyFilterState {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            active: false,
            matcher: SkimMatcherV2::default(),
            filtered_indices: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct SearchState {
    pub pattern: String,
    pub current_match: Option<(usize, usize)>,
    pub matches: Vec<(usize, usize)>,
    pub match_index: usize,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            current_match: None,
            matches: Vec::new(),
            match_index: 0,
        }
    }
}

#[derive(Clone)]
pub struct ColumnSearchState {
    pub pattern: String,
    pub matching_columns: Vec<usize>,
    pub current_match: usize,
}

impl Default for ColumnSearchState {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            matching_columns: Vec::new(),
            current_match: 0,
        }
    }
}

pub type ColumnStatistics = std::collections::BTreeMap<String, String>;

/// BufferAPI trait - defines the interface for interacting with buffer state
/// This abstraction allows the TUI to work with buffer state without knowing
/// the implementation details, enabling gradual migration and testing
pub trait BufferAPI {
    // --- Query and Results ---
    fn get_query(&self) -> String;
    fn set_query(&mut self, query: String);
    fn get_results(&self) -> Option<&QueryResponse>;
    fn set_results(&mut self, results: Option<QueryResponse>);
    fn get_last_query(&self) -> String;
    fn set_last_query(&mut self, query: String);

    // --- Mode and Status ---
    fn get_mode(&self) -> AppMode;
    fn set_mode(&mut self, mode: AppMode);
    fn get_status_message(&self) -> String;
    fn set_status_message(&mut self, message: String);

    // --- Table Navigation ---
    fn get_selected_row(&self) -> Option<usize>;
    fn set_selected_row(&mut self, row: Option<usize>);
    fn get_current_column(&self) -> usize;
    fn set_current_column(&mut self, col: usize);
    fn get_scroll_offset(&self) -> (usize, usize);
    fn set_scroll_offset(&mut self, offset: (usize, usize));

    // --- Filtering ---
    fn get_filter_pattern(&self) -> String;
    fn set_filter_pattern(&mut self, pattern: String);
    fn is_filter_active(&self) -> bool;
    fn set_filter_active(&mut self, active: bool);
    fn get_filtered_data(&self) -> Option<&Vec<Vec<String>>>;
    fn set_filtered_data(&mut self, data: Option<Vec<Vec<String>>>);

    // --- Search ---
    fn get_search_pattern(&self) -> String;
    fn set_search_pattern(&mut self, pattern: String);
    fn get_search_matches(&self) -> Vec<(usize, usize)>;
    fn set_search_matches(&mut self, matches: Vec<(usize, usize)>);
    fn get_current_match(&self) -> Option<(usize, usize)>;
    fn set_current_match(&mut self, match_pos: Option<(usize, usize)>);

    // --- Sorting ---
    fn get_sort_column(&self) -> Option<usize>;
    fn set_sort_column(&mut self, column: Option<usize>);
    fn get_sort_order(&self) -> SortOrder;
    fn set_sort_order(&mut self, order: SortOrder);

    // --- Display Options ---
    fn is_compact_mode(&self) -> bool;
    fn set_compact_mode(&mut self, compact: bool);
    fn is_show_row_numbers(&self) -> bool;
    fn set_show_row_numbers(&mut self, show: bool);
    fn get_pinned_columns(&self) -> &Vec<usize>;
    fn add_pinned_column(&mut self, col: usize);
    fn remove_pinned_column(&mut self, col: usize);
    fn clear_pinned_columns(&mut self);

    // --- Buffer Metadata ---
    fn get_name(&self) -> String;
    fn get_file_path(&self) -> Option<&PathBuf>;
    fn is_modified(&self) -> bool;
    fn set_modified(&mut self, modified: bool);

    // --- CSV/Data Source ---
    fn get_csv_client(&self) -> Option<&CsvApiClient>;
    fn get_csv_client_mut(&mut self) -> Option<&mut CsvApiClient>;
    fn is_csv_mode(&self) -> bool;
    fn get_table_name(&self) -> String;

    // --- Input State ---
    fn get_input_value(&self) -> String;
    fn set_input_value(&mut self, value: String);
    fn get_input_cursor(&self) -> usize;
    fn set_input_cursor(&mut self, pos: usize);

    // --- Advanced Operations ---
    fn apply_filter(&mut self) -> Result<()>;
    fn apply_sort(&mut self) -> Result<()>;
    fn search(&mut self) -> Result<()>;
    fn clear_filters(&mut self);
    fn get_row_count(&self) -> usize;
    fn get_column_count(&self) -> usize;
}

/// Represents a single buffer/tab with its own independent state
#[derive(Clone)]
pub struct Buffer {
    /// Unique identifier for this buffer
    pub id: usize,

    /// File path if loaded from file
    pub file_path: Option<PathBuf>,

    /// Display name (filename or "untitled")
    pub name: String,

    /// Whether this buffer has unsaved changes
    pub modified: bool,

    // --- Data State ---
    pub csv_client: Option<CsvApiClient>,
    pub csv_mode: bool,
    pub csv_table_name: String,
    pub results: Option<QueryResponse>,
    pub cached_data: Option<Vec<serde_json::Value>>,

    // --- UI State ---
    pub mode: AppMode,
    pub edit_mode: EditMode,
    pub input: Input,
    pub textarea: TextArea<'static>,
    pub table_state: TableState,
    pub last_results_row: Option<usize>,
    pub last_scroll_offset: (usize, usize),

    // --- Query State ---
    pub last_query: String,
    pub status_message: String,

    // --- Filter/Search State ---
    pub sort_state: SortState,
    pub filter_state: FilterState,
    pub fuzzy_filter_state: FuzzyFilterState,
    pub search_state: SearchState,
    pub column_search_state: ColumnSearchState,
    pub filtered_data: Option<Vec<Vec<String>>>,

    // --- View State ---
    pub column_widths: Vec<u16>,
    pub scroll_offset: (usize, usize),
    pub current_column: usize,
    pub pinned_columns: Vec<usize>,
    pub column_stats: Option<ColumnStatistics>,
    pub compact_mode: bool,
    pub viewport_lock: bool,
    pub viewport_lock_row: Option<usize>,
    pub show_row_numbers: bool,
    pub case_insensitive: bool,

    // --- Misc State ---
    pub undo_stack: Vec<String>,
    pub redo_stack: Vec<String>,
    pub kill_ring: String,
    pub last_visible_rows: usize,
}

// Implement BufferAPI for Buffer
impl BufferAPI for Buffer {
    // --- Query and Results ---
    fn get_query(&self) -> String {
        self.input.value().to_string()
    }

    fn set_query(&mut self, query: String) {
        self.input = Input::new(query.clone()).with_cursor(query.len());
    }

    fn get_results(&self) -> Option<&QueryResponse> {
        self.results.as_ref()
    }

    fn set_results(&mut self, results: Option<QueryResponse>) {
        self.results = results;
    }

    fn get_last_query(&self) -> String {
        self.last_query.clone()
    }

    fn set_last_query(&mut self, query: String) {
        self.last_query = query;
    }

    // --- Mode and Status ---
    fn get_mode(&self) -> AppMode {
        self.mode.clone()
    }

    fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    fn get_status_message(&self) -> String {
        self.status_message.clone()
    }

    fn set_status_message(&mut self, message: String) {
        self.status_message = message;
    }

    // --- Table Navigation ---
    fn get_selected_row(&self) -> Option<usize> {
        self.table_state.selected()
    }

    fn set_selected_row(&mut self, row: Option<usize>) {
        if let Some(r) = row {
            self.table_state.select(Some(r));
        } else {
            self.table_state.select(None);
        }
    }

    fn get_current_column(&self) -> usize {
        self.current_column
    }

    fn set_current_column(&mut self, col: usize) {
        self.current_column = col;
    }

    fn get_scroll_offset(&self) -> (usize, usize) {
        self.scroll_offset
    }

    fn set_scroll_offset(&mut self, offset: (usize, usize)) {
        self.scroll_offset = offset;
    }

    // --- Filtering ---
    fn get_filter_pattern(&self) -> String {
        self.filter_state.pattern.clone()
    }

    fn set_filter_pattern(&mut self, pattern: String) {
        self.filter_state.pattern = pattern;
    }

    fn is_filter_active(&self) -> bool {
        self.filter_state.active
    }

    fn set_filter_active(&mut self, active: bool) {
        self.filter_state.active = active;
    }

    fn get_filtered_data(&self) -> Option<&Vec<Vec<String>>> {
        self.filtered_data.as_ref()
    }

    fn set_filtered_data(&mut self, data: Option<Vec<Vec<String>>>) {
        self.filtered_data = data;
    }

    // --- Search ---
    fn get_search_pattern(&self) -> String {
        self.search_state.pattern.clone()
    }

    fn set_search_pattern(&mut self, pattern: String) {
        self.search_state.pattern = pattern;
    }

    fn get_search_matches(&self) -> Vec<(usize, usize)> {
        self.search_state.matches.clone()
    }

    fn set_search_matches(&mut self, matches: Vec<(usize, usize)>) {
        self.search_state.matches = matches;
    }

    fn get_current_match(&self) -> Option<(usize, usize)> {
        self.search_state.current_match
    }

    fn set_current_match(&mut self, match_pos: Option<(usize, usize)>) {
        self.search_state.current_match = match_pos;
    }

    // --- Sorting ---
    fn get_sort_column(&self) -> Option<usize> {
        self.sort_state.column
    }

    fn set_sort_column(&mut self, column: Option<usize>) {
        self.sort_state.column = column;
    }

    fn get_sort_order(&self) -> SortOrder {
        self.sort_state.order
    }

    fn set_sort_order(&mut self, order: SortOrder) {
        self.sort_state.order = order;
    }

    // --- Display Options ---
    fn is_compact_mode(&self) -> bool {
        self.compact_mode
    }

    fn set_compact_mode(&mut self, compact: bool) {
        self.compact_mode = compact;
    }

    fn is_show_row_numbers(&self) -> bool {
        self.show_row_numbers
    }

    fn set_show_row_numbers(&mut self, show: bool) {
        self.show_row_numbers = show;
    }

    fn get_pinned_columns(&self) -> &Vec<usize> {
        &self.pinned_columns
    }

    fn add_pinned_column(&mut self, col: usize) {
        if !self.pinned_columns.contains(&col) {
            self.pinned_columns.push(col);
            self.pinned_columns.sort();
        }
    }

    fn remove_pinned_column(&mut self, col: usize) {
        self.pinned_columns.retain(|&c| c != col);
    }

    fn clear_pinned_columns(&mut self) {
        self.pinned_columns.clear();
    }

    // --- Buffer Metadata ---
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    fn is_modified(&self) -> bool {
        self.modified
    }

    fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    // --- CSV/Data Source ---
    fn get_csv_client(&self) -> Option<&CsvApiClient> {
        self.csv_client.as_ref()
    }

    fn get_csv_client_mut(&mut self) -> Option<&mut CsvApiClient> {
        self.csv_client.as_mut()
    }

    fn is_csv_mode(&self) -> bool {
        self.csv_mode
    }

    fn get_table_name(&self) -> String {
        self.csv_table_name.clone()
    }

    // --- Input State ---
    fn get_input_value(&self) -> String {
        self.input.value().to_string()
    }

    fn set_input_value(&mut self, value: String) {
        let cursor = value.len();
        self.input = Input::new(value).with_cursor(cursor);
    }

    fn get_input_cursor(&self) -> usize {
        self.input.cursor()
    }

    fn set_input_cursor(&mut self, pos: usize) {
        let value = self.input.value().to_string();
        self.input = Input::new(value).with_cursor(pos);
    }

    // --- Advanced Operations ---
    fn apply_filter(&mut self) -> Result<()> {
        // TODO: Implement actual filtering logic
        Ok(())
    }

    fn apply_sort(&mut self) -> Result<()> {
        // TODO: Implement actual sorting logic
        Ok(())
    }

    fn search(&mut self) -> Result<()> {
        // TODO: Implement actual search logic
        Ok(())
    }

    fn clear_filters(&mut self) {
        self.filter_state.active = false;
        self.filter_state.pattern.clear();
        self.fuzzy_filter_state.active = false;
        self.fuzzy_filter_state.pattern.clear();
        self.filtered_data = None;
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

    fn get_column_count(&self) -> usize {
        if let Some(results) = &self.results {
            if let Some(first_row) = results.data.first() {
                if let Some(obj) = first_row.as_object() {
                    return obj.len();
                }
            }
        }
        0
    }
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new(id: usize) -> Self {
        Self {
            id,
            file_path: None,
            name: format!("[Buffer {}]", id),
            modified: false,

            csv_client: None,
            csv_mode: false,
            csv_table_name: String::new(),
            results: None,
            cached_data: None,

            mode: AppMode::Command,
            edit_mode: EditMode::SingleLine,
            input: Input::default(),
            textarea: TextArea::default(),
            table_state: TableState::default(),
            last_results_row: None,
            last_scroll_offset: (0, 0),

            last_query: String::new(),
            status_message: String::new(),

            sort_state: SortState {
                column: None,
                order: SortOrder::None,
            },
            filter_state: FilterState::default(),
            fuzzy_filter_state: FuzzyFilterState::default(),
            search_state: SearchState::default(),
            column_search_state: ColumnSearchState::default(),
            filtered_data: None,

            column_widths: Vec::new(),
            scroll_offset: (0, 0),
            current_column: 0,
            pinned_columns: Vec::new(),
            column_stats: None,
            compact_mode: false,
            viewport_lock: false,
            viewport_lock_row: None,
            show_row_numbers: false,
            case_insensitive: false,

            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            kill_ring: String::new(),
            last_visible_rows: 30,
        }
    }

    /// Create a buffer from a CSV file
    pub fn from_csv(
        id: usize,
        path: PathBuf,
        csv_client: CsvApiClient,
        table_name: String,
    ) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.csv")
            .to_string();

        let mut buffer = Self::new(id);
        buffer.file_path = Some(path);
        buffer.name = name;
        buffer.csv_client = Some(csv_client);
        buffer.csv_mode = true;
        buffer.csv_table_name = table_name;

        buffer
    }

    /// Create a buffer from a JSON file
    pub fn from_json(
        id: usize,
        path: PathBuf,
        csv_client: CsvApiClient,
        table_name: String,
    ) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.json")
            .to_string();

        let mut buffer = Self::new(id);
        buffer.file_path = Some(path);
        buffer.name = name;
        buffer.csv_client = Some(csv_client);
        buffer.csv_mode = true;
        buffer.csv_table_name = table_name;

        buffer
    }

    /// Get display name for tab bar
    pub fn display_name(&self) -> String {
        if self.modified {
            format!("{}*", self.name)
        } else {
            self.name.clone()
        }
    }

    /// Get short name for tab bar (truncated if needed)
    pub fn short_name(&self, max_len: usize) -> String {
        let display = self.display_name();
        if display.len() <= max_len {
            display
        } else {
            format!("{}...", &display[..max_len.saturating_sub(3)])
        }
    }

    /// Check if buffer has a specific file open
    pub fn has_file(&self, path: &PathBuf) -> bool {
        self.file_path.as_ref() == Some(path)
    }
}

/// Manages multiple buffers and switching between them
pub struct BufferManager {
    buffers: Vec<Buffer>,
    current_buffer_index: usize,
    next_buffer_id: usize,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            current_buffer_index: 0,
            next_buffer_id: 1,
        }
    }

    /// Add a new buffer and make it current
    pub fn add_buffer(&mut self, mut buffer: Buffer) -> usize {
        buffer.id = self.next_buffer_id;
        self.next_buffer_id += 1;

        let index = self.buffers.len();
        self.buffers.push(buffer);
        self.current_buffer_index = index;
        index
    }

    /// Get current buffer
    pub fn current(&self) -> Option<&Buffer> {
        self.buffers.get(self.current_buffer_index)
    }

    /// Get current buffer mutably
    pub fn current_mut(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(self.current_buffer_index)
    }

    /// Switch to next buffer
    pub fn next_buffer(&mut self) {
        if !self.buffers.is_empty() {
            self.current_buffer_index = (self.current_buffer_index + 1) % self.buffers.len();
        }
    }

    /// Switch to previous buffer
    pub fn prev_buffer(&mut self) {
        if !self.buffers.is_empty() {
            if self.current_buffer_index == 0 {
                self.current_buffer_index = self.buffers.len() - 1;
            } else {
                self.current_buffer_index -= 1;
            }
        }
    }

    /// Switch to buffer by index
    pub fn switch_to(&mut self, index: usize) {
        if index < self.buffers.len() {
            self.current_buffer_index = index;
        }
    }

    /// Close current buffer
    pub fn close_current(&mut self) -> bool {
        if self.buffers.len() <= 1 {
            return false; // Don't close last buffer
        }

        self.buffers.remove(self.current_buffer_index);

        // Adjust current index if needed
        if self.current_buffer_index >= self.buffers.len() {
            self.current_buffer_index = self.buffers.len() - 1;
        }

        true
    }

    /// Find buffer by file path
    pub fn find_by_path(&self, path: &PathBuf) -> Option<usize> {
        self.buffers.iter().position(|b| b.has_file(path))
    }

    /// Get all buffers for display
    pub fn all_buffers(&self) -> &[Buffer] {
        &self.buffers
    }

    /// Get current buffer index
    pub fn current_index(&self) -> usize {
        self.current_buffer_index
    }

    /// Check if we have multiple buffers
    pub fn has_multiple(&self) -> bool {
        self.buffers.len() > 1
    }
}
