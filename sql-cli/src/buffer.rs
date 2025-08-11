use crate::api_client::QueryResponse;
use crate::csv_datasource::CsvApiClient;
use crate::cursor_operations::CursorOperations;
use crate::hybrid_parser::HybridParser;
use crate::input_manager::{create_from_input, create_single_line, InputManager};
use anyhow::Result;
use crossterm::event::KeyEvent;
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::style::Color;
use ratatui::widgets::TableState;
use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tui_input::Input;

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

#[derive(Clone, PartialEq, Debug)]
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
    pub matching_columns: Vec<(usize, String)>, // (index, column_name)
    pub current_match: usize,
}

#[derive(Clone, Debug)]
pub enum ColumnType {
    String,
    Numeric,
    Mixed,
}

#[derive(Clone)]
pub struct ColumnStatistics {
    pub column_name: String,
    pub column_type: ColumnType,
    // For all columns
    pub total_count: usize,
    pub null_count: usize,
    pub unique_count: usize,
    // For categorical/string columns
    pub frequency_map: Option<BTreeMap<String, usize>>,
    // For numeric columns
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub sum: Option<f64>,
    pub mean: Option<f64>,
    pub median: Option<f64>,
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

// pub type ColumnStatistics = std::collections::BTreeMap<String, String>; // Replaced with struct

/// BufferAPI trait - defines the interface for interacting with buffer state
/// This abstraction allows the TUI to work with buffer state without knowing
/// the implementation details, enabling gradual migration and testing
pub trait BufferAPI {
    // --- Identity ---
    fn get_id(&self) -> usize;
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
    fn get_edit_mode(&self) -> EditMode;
    fn set_edit_mode(&mut self, mode: EditMode);
    fn get_status_message(&self) -> String;
    fn set_status_message(&mut self, message: String);

    // --- Table Navigation ---
    fn get_selected_row(&self) -> Option<usize>;
    fn set_selected_row(&mut self, row: Option<usize>);
    fn get_current_column(&self) -> usize;
    fn set_current_column(&mut self, col: usize);
    fn get_scroll_offset(&self) -> (usize, usize);
    fn set_scroll_offset(&mut self, offset: (usize, usize));
    fn get_last_results_row(&self) -> Option<usize>;
    fn set_last_results_row(&mut self, row: Option<usize>);
    fn get_last_scroll_offset(&self) -> (usize, usize);
    fn set_last_scroll_offset(&mut self, offset: (usize, usize));

    // --- Filtering ---
    fn get_filter_pattern(&self) -> String;
    fn set_filter_pattern(&mut self, pattern: String);
    fn is_filter_active(&self) -> bool;
    fn set_filter_active(&mut self, active: bool);
    fn get_filtered_data(&self) -> Option<&Vec<Vec<String>>>;
    fn set_filtered_data(&mut self, data: Option<Vec<Vec<String>>>);

    // --- Fuzzy Filter ---
    fn get_fuzzy_filter_pattern(&self) -> String;
    fn set_fuzzy_filter_pattern(&mut self, pattern: String);
    fn is_fuzzy_filter_active(&self) -> bool;
    fn set_fuzzy_filter_active(&mut self, active: bool);
    fn get_fuzzy_filter_indices(&self) -> &Vec<usize>;
    fn set_fuzzy_filter_indices(&mut self, indices: Vec<usize>);
    fn clear_fuzzy_filter(&mut self);

    // --- Search ---
    fn get_search_pattern(&self) -> String;
    fn set_search_pattern(&mut self, pattern: String);
    fn get_search_matches(&self) -> Vec<(usize, usize)>;
    fn set_search_matches(&mut self, matches: Vec<(usize, usize)>);
    fn get_current_match(&self) -> Option<(usize, usize)>;
    fn set_current_match(&mut self, match_pos: Option<(usize, usize)>);
    fn get_search_match_index(&self) -> usize;
    fn set_search_match_index(&mut self, index: usize);
    fn clear_search_state(&mut self);

    // --- Column Search ---
    fn get_column_search_pattern(&self) -> String;
    fn set_column_search_pattern(&mut self, pattern: String);
    fn get_column_search_matches(&self) -> &Vec<(usize, String)>;
    fn set_column_search_matches(&mut self, matches: Vec<(usize, String)>);
    fn get_column_search_current_match(&self) -> usize;
    fn set_column_search_current_match(&mut self, index: usize);
    fn clear_column_search(&mut self);

    // --- Column Statistics ---
    fn get_column_stats(&self) -> Option<&ColumnStatistics>;
    fn set_column_stats(&mut self, stats: Option<ColumnStatistics>);

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
    fn is_viewport_lock(&self) -> bool;
    fn set_viewport_lock(&mut self, locked: bool);
    fn get_viewport_lock_row(&self) -> Option<usize>;
    fn set_viewport_lock_row(&mut self, row: Option<usize>);
    fn get_pinned_columns(&self) -> &Vec<usize>;
    fn add_pinned_column(&mut self, col: usize);
    fn remove_pinned_column(&mut self, col: usize);
    fn clear_pinned_columns(&mut self);
    fn get_column_widths(&self) -> &Vec<u16>;
    fn set_column_widths(&mut self, widths: Vec<u16>);
    fn is_case_insensitive(&self) -> bool;
    fn set_case_insensitive(&mut self, case_insensitive: bool);

    // --- Buffer Metadata ---
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn get_file_path(&self) -> Option<&PathBuf>;
    fn set_file_path(&mut self, path: Option<String>);
    fn is_modified(&self) -> bool;
    fn set_modified(&mut self, modified: bool);
    fn get_last_query_source(&self) -> Option<String>;
    fn set_last_query_source(&mut self, source: Option<String>);

    // --- CSV/Data Source ---
    fn get_csv_client(&self) -> Option<&CsvApiClient>;
    fn get_csv_client_mut(&mut self) -> Option<&mut CsvApiClient>;
    fn set_csv_client(&mut self, client: Option<CsvApiClient>);
    fn is_csv_mode(&self) -> bool;
    fn set_csv_mode(&mut self, csv_mode: bool);
    fn get_table_name(&self) -> String;
    fn set_table_name(&mut self, table_name: String);
    fn is_cache_mode(&self) -> bool;
    fn set_cache_mode(&mut self, cache_mode: bool);
    fn get_cached_data(&self) -> Option<&Vec<Value>>;
    fn set_cached_data(&mut self, data: Option<Vec<Value>>);
    fn has_cached_data(&self) -> bool;

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

    // --- Edit State ---
    fn get_undo_stack(&self) -> &Vec<(String, usize)>;
    fn push_undo(&mut self, state: (String, usize));
    fn pop_undo(&mut self) -> Option<(String, usize)>;
    fn get_redo_stack(&self) -> &Vec<(String, usize)>;
    fn push_redo(&mut self, state: (String, usize));
    fn pop_redo(&mut self) -> Option<(String, usize)>;
    fn clear_redo(&mut self);
    fn get_kill_ring(&self) -> String;
    fn set_kill_ring(&mut self, text: String);
    fn is_kill_ring_empty(&self) -> bool;

    // High-level undo/redo operations
    fn perform_undo(&mut self) -> bool;
    fn perform_redo(&mut self) -> bool;
    fn save_state_for_undo(&mut self);

    // --- Viewport State ---
    fn get_last_visible_rows(&self) -> usize;
    fn set_last_visible_rows(&mut self, rows: usize);

    // --- Debug ---
    fn debug_dump(&self) -> String;

    // --- Input Management ---
    fn get_input_text(&self) -> String;
    fn set_input_text(&mut self, text: String);
    fn handle_input_key(&mut self, event: KeyEvent) -> bool;
    fn switch_input_mode(&mut self, multiline: bool);
    fn get_input_cursor_position(&self) -> usize;
    fn set_input_cursor_position(&mut self, position: usize);
    fn is_input_multiline(&self) -> bool;

    // --- History Navigation ---
    fn navigate_history_up(&mut self, history: &[String]) -> bool;
    fn navigate_history_down(&mut self, history: &[String]) -> bool;
    fn reset_history_navigation(&mut self);

    // --- Results Management ---
    fn clear_results(&mut self);
}

/// Represents a single buffer/tab with its own independent state
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
    pub cache_mode: bool,
    pub results: Option<QueryResponse>,
    pub cached_data: Option<Vec<serde_json::Value>>,

    // --- UI State ---
    pub mode: AppMode,
    pub edit_mode: EditMode,
    pub input: Input, // Legacy - kept for compatibility during migration
    pub input_manager: Box<dyn InputManager>, // New unified input management
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
    pub column_stats: Option<ColumnStatistics>,
    pub filtered_data: Option<Vec<Vec<String>>>,

    // --- View State ---
    pub column_widths: Vec<u16>,
    pub scroll_offset: (usize, usize),
    pub current_column: usize,
    pub pinned_columns: Vec<usize>,
    pub compact_mode: bool,
    pub viewport_lock: bool,
    pub viewport_lock_row: Option<usize>,
    pub show_row_numbers: bool,
    pub case_insensitive: bool,

    // --- Misc State ---
    pub undo_stack: Vec<(String, usize)>,
    pub redo_stack: Vec<(String, usize)>,
    pub kill_ring: String,
    pub last_visible_rows: usize,
    pub last_query_source: Option<String>,

    // --- Syntax Highlighting ---
    pub highlighted_text_cache: Option<Vec<(String, Color)>>, // Cache of highlighted tokens
    pub last_highlighted_text: String, // Track what text was highlighted to detect changes

    // --- Input State Stack (for search/filter modes) ---
    pub saved_input_state: Option<(String, usize)>, // Save input when entering search/filter
}

// Implement BufferAPI for Buffer
impl BufferAPI for Buffer {
    // --- Identity ---
    fn get_id(&self) -> usize {
        self.id
    }

    // --- Query and Results ---
    fn get_query(&self) -> String {
        // Use InputManager if available, fallback to legacy input
        self.input_manager.get_text()
    }

    fn set_query(&mut self, query: String) {
        // Update both InputManager and legacy field for compatibility
        self.input_manager.set_text(query.clone());
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

    fn get_edit_mode(&self) -> EditMode {
        self.edit_mode.clone()
    }

    fn set_edit_mode(&mut self, mode: EditMode) {
        self.edit_mode = mode;
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

    fn get_last_results_row(&self) -> Option<usize> {
        self.last_results_row
    }

    fn set_last_results_row(&mut self, row: Option<usize>) {
        self.last_results_row = row;
    }

    fn get_last_scroll_offset(&self) -> (usize, usize) {
        self.last_scroll_offset
    }

    fn set_last_scroll_offset(&mut self, offset: (usize, usize)) {
        self.last_scroll_offset = offset;
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

    // --- Fuzzy Filter ---
    fn get_fuzzy_filter_pattern(&self) -> String {
        self.fuzzy_filter_state.pattern.clone()
    }

    fn set_fuzzy_filter_pattern(&mut self, pattern: String) {
        self.fuzzy_filter_state.pattern = pattern;
    }

    fn is_fuzzy_filter_active(&self) -> bool {
        self.fuzzy_filter_state.active
    }

    fn set_fuzzy_filter_active(&mut self, active: bool) {
        self.fuzzy_filter_state.active = active;
    }

    fn get_fuzzy_filter_indices(&self) -> &Vec<usize> {
        &self.fuzzy_filter_state.filtered_indices
    }

    fn set_fuzzy_filter_indices(&mut self, indices: Vec<usize>) {
        self.fuzzy_filter_state.filtered_indices = indices;
    }

    fn clear_fuzzy_filter(&mut self) {
        self.fuzzy_filter_state.pattern.clear();
        self.fuzzy_filter_state.active = false;
        self.fuzzy_filter_state.filtered_indices.clear();
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

    fn get_search_match_index(&self) -> usize {
        self.search_state.match_index
    }

    fn set_search_match_index(&mut self, index: usize) {
        self.search_state.match_index = index;
    }

    fn clear_search_state(&mut self) {
        self.search_state.pattern.clear();
        self.search_state.matches.clear();
        self.search_state.current_match = None;
        self.search_state.match_index = 0;
    }

    // --- Column Search ---
    fn get_column_search_pattern(&self) -> String {
        self.column_search_state.pattern.clone()
    }

    fn set_column_search_pattern(&mut self, pattern: String) {
        self.column_search_state.pattern = pattern;
    }

    fn get_column_search_matches(&self) -> &Vec<(usize, String)> {
        &self.column_search_state.matching_columns
    }

    fn set_column_search_matches(&mut self, matches: Vec<(usize, String)>) {
        self.column_search_state.matching_columns = matches;
    }

    fn get_column_search_current_match(&self) -> usize {
        self.column_search_state.current_match
    }

    fn set_column_search_current_match(&mut self, index: usize) {
        self.column_search_state.current_match = index;
    }

    fn clear_column_search(&mut self) {
        self.column_search_state.pattern.clear();
        self.column_search_state.matching_columns.clear();
        self.column_search_state.current_match = 0;
    }

    fn get_column_stats(&self) -> Option<&ColumnStatistics> {
        self.column_stats.as_ref()
    }

    fn set_column_stats(&mut self, stats: Option<ColumnStatistics>) {
        self.column_stats = stats;
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

    fn is_viewport_lock(&self) -> bool {
        self.viewport_lock
    }

    fn set_viewport_lock(&mut self, locked: bool) {
        self.viewport_lock = locked;
    }

    fn get_viewport_lock_row(&self) -> Option<usize> {
        self.viewport_lock_row
    }

    fn set_viewport_lock_row(&mut self, row: Option<usize>) {
        self.viewport_lock_row = row;
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

    fn get_column_widths(&self) -> &Vec<u16> {
        &self.column_widths
    }

    fn set_column_widths(&mut self, widths: Vec<u16>) {
        self.column_widths = widths;
    }

    fn is_case_insensitive(&self) -> bool {
        self.case_insensitive
    }

    fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.case_insensitive = case_insensitive;
    }

    // --- Buffer Metadata ---
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn get_file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    fn set_file_path(&mut self, path: Option<String>) {
        self.file_path = path.map(PathBuf::from);
    }

    fn is_modified(&self) -> bool {
        self.modified
    }

    fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    fn get_last_query_source(&self) -> Option<String> {
        self.last_query_source.clone()
    }

    fn set_last_query_source(&mut self, source: Option<String>) {
        self.last_query_source = source;
    }

    // --- CSV/Data Source ---
    fn get_csv_client(&self) -> Option<&CsvApiClient> {
        self.csv_client.as_ref()
    }

    fn get_csv_client_mut(&mut self) -> Option<&mut CsvApiClient> {
        self.csv_client.as_mut()
    }

    fn set_csv_client(&mut self, client: Option<CsvApiClient>) {
        self.csv_client = client;
    }

    fn is_csv_mode(&self) -> bool {
        self.csv_mode
    }

    fn set_csv_mode(&mut self, csv_mode: bool) {
        self.csv_mode = csv_mode;
    }

    fn get_table_name(&self) -> String {
        self.csv_table_name.clone()
    }

    fn set_table_name(&mut self, table_name: String) {
        self.csv_table_name = table_name;
    }

    fn is_cache_mode(&self) -> bool {
        self.cache_mode
    }

    fn set_cache_mode(&mut self, cache_mode: bool) {
        self.cache_mode = cache_mode;
    }

    fn get_cached_data(&self) -> Option<&Vec<Value>> {
        self.cached_data.as_ref()
    }

    fn set_cached_data(&mut self, data: Option<Vec<Value>>) {
        self.cached_data = data;
    }

    fn has_cached_data(&self) -> bool {
        self.cached_data.is_some()
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

    // --- Edit State ---
    fn get_undo_stack(&self) -> &Vec<(String, usize)> {
        &self.undo_stack
    }

    fn push_undo(&mut self, state: (String, usize)) {
        self.undo_stack.push(state);
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
    }

    fn pop_undo(&mut self) -> Option<(String, usize)> {
        self.undo_stack.pop()
    }

    fn get_redo_stack(&self) -> &Vec<(String, usize)> {
        &self.redo_stack
    }

    fn push_redo(&mut self, state: (String, usize)) {
        self.redo_stack.push(state);
    }

    fn pop_redo(&mut self) -> Option<(String, usize)> {
        self.redo_stack.pop()
    }

    fn clear_redo(&mut self) {
        self.redo_stack.clear();
    }

    fn perform_undo(&mut self) -> bool {
        if let Some((prev_text, prev_cursor)) = self.pop_undo() {
            // Save current state to redo stack
            let current_state = (self.get_input_text(), self.get_input_cursor_position());
            self.push_redo(current_state);

            // Restore previous state
            self.set_input_text(prev_text);
            self.set_input_cursor_position(prev_cursor);
            true
        } else {
            false
        }
    }

    fn perform_redo(&mut self) -> bool {
        if let Some((next_text, next_cursor)) = self.pop_redo() {
            // Save current state to undo stack
            let current_state = (self.get_input_text(), self.get_input_cursor_position());
            self.push_undo(current_state);

            // Restore next state
            self.set_input_text(next_text);
            self.set_input_cursor_position(next_cursor);
            true
        } else {
            false
        }
    }

    fn save_state_for_undo(&mut self) {
        let current_state = (self.get_input_text(), self.get_input_cursor_position());
        self.push_undo(current_state);
        self.clear_redo();
    }

    fn get_kill_ring(&self) -> String {
        self.kill_ring.clone()
    }

    fn set_kill_ring(&mut self, text: String) {
        self.kill_ring = text;
    }

    fn is_kill_ring_empty(&self) -> bool {
        self.kill_ring.is_empty()
    }

    // --- Viewport State ---
    fn get_last_visible_rows(&self) -> usize {
        self.last_visible_rows
    }

    fn set_last_visible_rows(&mut self, rows: usize) {
        self.last_visible_rows = rows;
    }

    fn debug_dump(&self) -> String {
        let mut output = String::new();
        output.push_str("=== BUFFER DEBUG DUMP ===\n");
        output.push_str(&format!("Buffer ID: {}\n", self.id));
        output.push_str(&format!("Name: {}\n", self.name));
        output.push_str(&format!("File Path: {:?}\n", self.file_path));
        output.push_str(&format!("Modified: {}\n", self.modified));
        output.push_str("\n--- Modes ---\n");
        output.push_str(&format!("App Mode: {:?}\n", self.mode));
        output.push_str(&format!("Edit Mode: {:?}\n", self.edit_mode));
        output.push_str("\n--- Query State ---\n");
        output.push_str(&format!("Current Input: '{}'\n", self.input.value()));
        output.push_str(&format!("Input Cursor: {}\n", self.input.cursor()));
        output.push_str(&format!("Last Query: '{}'\n", self.last_query));
        output.push_str(&format!("Status Message: '{}'\n", self.status_message));
        output.push_str(&format!(
            "Last Query Source: {:?}\n",
            self.last_query_source
        ));
        output.push_str("\n--- Results ---\n");
        output.push_str(&format!("Has Results: {}\n", self.results.is_some()));
        output.push_str(&format!("Row Count: {}\n", self.get_row_count()));
        output.push_str(&format!("Column Count: {}\n", self.get_column_count()));
        output.push_str(&format!(
            "Selected Row: {:?}\n",
            self.table_state.selected()
        ));
        output.push_str(&format!("Current Column: {}\n", self.current_column));
        output.push_str(&format!("Scroll Offset: {:?}\n", self.scroll_offset));
        output.push_str("\n--- Filtering ---\n");
        output.push_str(&format!("Filter Active: {}\n", self.filter_state.active));
        output.push_str(&format!(
            "Filter Pattern: '{}'\n",
            self.filter_state.pattern
        ));
        output.push_str(&format!(
            "Has Filtered Data: {}\n",
            self.filtered_data.is_some()
        ));
        output.push_str(&format!(
            "Fuzzy Filter Active: {}\n",
            self.fuzzy_filter_state.active
        ));
        output.push_str(&format!(
            "Fuzzy Pattern: '{}'\n",
            self.fuzzy_filter_state.pattern
        ));
        output.push_str("\n--- Search ---\n");
        output.push_str(&format!(
            "Search Pattern: '{}'\n",
            self.search_state.pattern
        ));
        output.push_str(&format!(
            "Search Matches: {} found\n",
            self.search_state.matches.len()
        ));
        output.push_str(&format!(
            "Current Match: {:?}\n",
            self.search_state.current_match
        ));
        output.push_str(&format!("Match Index: {}\n", self.search_state.match_index));
        output.push_str("\n--- Column Search ---\n");
        output.push_str(&format!(
            "Column Search Pattern: '{}'\n",
            self.column_search_state.pattern
        ));
        output.push_str(&format!(
            "Matching Columns: {:?}\n",
            self.column_search_state.matching_columns
        ));
        output.push_str("\n--- Sorting ---\n");
        output.push_str(&format!("Sort Column: {:?}\n", self.sort_state.column));
        output.push_str(&format!("Sort Order: {:?}\n", self.sort_state.order));
        output.push_str("\n--- Display Options ---\n");
        output.push_str(&format!("Compact Mode: {}\n", self.compact_mode));
        output.push_str(&format!("Show Row Numbers: {}\n", self.show_row_numbers));
        output.push_str(&format!("Case Insensitive: {}\n", self.case_insensitive));
        output.push_str(&format!("Pinned Columns: {:?}\n", self.pinned_columns));
        output.push_str(&format!("Column Widths: {:?}\n", self.column_widths));
        output.push_str(&format!("Viewport Lock: {}\n", self.viewport_lock));
        output.push_str(&format!(
            "Viewport Lock Row: {:?}\n",
            self.viewport_lock_row
        ));
        output.push_str("\n--- CSV/Data Source ---\n");
        output.push_str(&format!("CSV Mode: {}\n", self.csv_mode));
        output.push_str(&format!("CSV Table Name: '{}'\n", self.csv_table_name));
        output.push_str(&format!("Cache Mode: {}\n", self.cache_mode));
        output.push_str(&format!("Has CSV Client: {}\n", self.csv_client.is_some()));
        output.push_str(&format!(
            "Has Cached Data: {}\n",
            self.cached_data.is_some()
        ));
        output.push_str("\n--- Undo/Redo ---\n");
        output.push_str(&format!("Undo Stack Size: {}\n", self.undo_stack.len()));
        output.push_str(&format!("Redo Stack Size: {}\n", self.redo_stack.len()));
        output.push_str(&format!(
            "Kill Ring: '{}'\n",
            if self.kill_ring.len() > 50 {
                format!(
                    "{}... ({} chars)",
                    &self.kill_ring[..50],
                    self.kill_ring.len()
                )
            } else {
                self.kill_ring.clone()
            }
        ));
        output.push_str("\n--- Stats ---\n");
        output.push_str(&format!(
            "Has Column Stats: {}\n",
            self.column_stats.is_some()
        ));
        output.push_str(&format!("Last Visible Rows: {}\n", self.last_visible_rows));
        output.push_str(&format!("Last Results Row: {:?}\n", self.last_results_row));
        output.push_str(&format!(
            "Last Scroll Offset: {:?}\n",
            self.last_scroll_offset
        ));
        output.push_str("\n=== END BUFFER DEBUG ===\n");
        output
    }

    // --- Input Management ---
    fn get_input_text(&self) -> String {
        self.input_manager.get_text()
    }

    fn set_input_text(&mut self, text: String) {
        self.input_manager.set_text(text.clone());
        // Sync with legacy fields for compatibility
        self.input = Input::new(text.clone()).with_cursor(text.len());
    }

    fn handle_input_key(&mut self, event: KeyEvent) -> bool {
        let result = self.input_manager.handle_key_event(event);
        // Sync with legacy fields after key handling
        self.sync_from_input_manager();
        result
    }

    fn switch_input_mode(&mut self, _multiline: bool) {
        let current_text = self.input_manager.get_text();
        let cursor_pos = self.input_manager.get_cursor_position();

        // Always use single-line mode
        self.edit_mode = EditMode::SingleLine;
        self.input_manager = create_single_line(current_text.clone());
        // Update legacy input
        self.input =
            Input::new(current_text.clone()).with_cursor(cursor_pos.min(current_text.len()));

        // Try to restore cursor position
        self.input_manager.set_cursor_position(cursor_pos);
    }

    fn get_input_cursor_position(&self) -> usize {
        self.input_manager.get_cursor_position()
    }

    fn set_input_cursor_position(&mut self, position: usize) {
        self.input_manager.set_cursor_position(position);
        // Sync with legacy fields
        if self.edit_mode == EditMode::SingleLine {
            let text = self.input.value().to_string();
            self.input = Input::new(text).with_cursor(position);
        }
    }

    fn is_input_multiline(&self) -> bool {
        self.input_manager.is_multiline()
    }

    // --- History Navigation ---
    fn navigate_history_up(&mut self, history: &[String]) -> bool {
        // Set history if not already set
        self.input_manager.set_history(history.to_vec());
        let navigated = self.input_manager.history_previous();
        if navigated {
            // Sync to legacy fields
            self.sync_from_input_manager();
        }
        navigated
    }

    fn navigate_history_down(&mut self, history: &[String]) -> bool {
        // Set history if not already set
        self.input_manager.set_history(history.to_vec());
        let navigated = self.input_manager.history_next();
        if navigated {
            // Sync to legacy fields
            self.sync_from_input_manager();
        }
        navigated
    }

    fn reset_history_navigation(&mut self) {
        self.input_manager.reset_history_position();
    }

    // --- Results Management ---
    fn clear_results(&mut self) {
        self.results = None;
        self.filtered_data = None;
        self.table_state.select(None);
        self.last_results_row = None;
        self.scroll_offset = (0, 0);
        self.last_scroll_offset = (0, 0);
        self.column_widths.clear();
        self.status_message = "Results cleared".to_string();
        // Reset search/filter states
        self.filter_state.active = false;
        self.filter_state.pattern.clear();
        self.search_state.pattern.clear();
        self.search_state.matches.clear();
        self.search_state.current_match = None;
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
            cache_mode: false,
            results: None,
            cached_data: None,

            mode: AppMode::Command,
            edit_mode: EditMode::SingleLine,
            input: Input::default(),
            input_manager: create_single_line(String::new()),
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
            column_stats: None,
            filtered_data: None,

            column_widths: Vec::new(),
            scroll_offset: (0, 0),
            current_column: 0,
            pinned_columns: Vec::new(),
            compact_mode: false,
            viewport_lock: false,
            viewport_lock_row: None,
            show_row_numbers: false,
            case_insensitive: false,

            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            kill_ring: String::new(),
            last_visible_rows: 30,
            last_query_source: None,

            highlighted_text_cache: None,
            last_highlighted_text: String::new(),
            saved_input_state: None,
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

    /// Sync from InputManager to legacy fields (for compatibility during migration)
    fn sync_from_input_manager(&mut self) {
        let text = self.input_manager.get_text();
        let cursor_pos = self.input_manager.get_cursor_position();

        // Always sync to single-line input
        let text_len = text.len();
        self.input = Input::new(text).with_cursor(cursor_pos.min(text_len));
    }

    /// Sync from legacy fields to InputManager (for compatibility during migration)
    fn sync_to_input_manager(&mut self) {
        // Always sync from single-line input
        let _text = self.input.value().to_string();
        self.input_manager = create_from_input(self.input.clone());
    }

    // --- Cursor Movement Operations ---
    // These use the CursorOperations helper to provide intelligent
    // SQL-aware cursor movement and text manipulation

    /// Move cursor to previous word boundary
    pub fn move_cursor_word_backward(&mut self) {
        let text = self.input_manager.get_text();
        let cursor_pos = self.input_manager.get_cursor_position();
        let new_pos = CursorOperations::find_word_boundary_backward(&text, cursor_pos);
        self.input_manager.set_cursor_position(new_pos);
        self.sync_from_input_manager();
        self.status_message = format!("Moved to position {} (word boundary)", new_pos);
    }

    /// Move cursor to next word boundary
    pub fn move_cursor_word_forward(&mut self) {
        let text = self.input_manager.get_text();
        let cursor_pos = self.input_manager.get_cursor_position();
        let new_pos = CursorOperations::find_word_boundary_forward(&text, cursor_pos);
        self.input_manager.set_cursor_position(new_pos);
        self.sync_from_input_manager();
    }

    /// Delete word backward from cursor
    pub fn delete_word_backward(&mut self) {
        let text = self.input_manager.get_text();
        let cursor_pos = self.input_manager.get_cursor_position();
        let (new_text, new_cursor) = CursorOperations::delete_word_backward(&text, cursor_pos);

        // Store deleted text in kill ring
        if cursor_pos > new_cursor {
            self.kill_ring = text[new_cursor..cursor_pos].to_string();
        }

        self.input_manager.set_text(new_text);
        self.input_manager.set_cursor_position(new_cursor);
        self.sync_from_input_manager();
    }

    /// Delete word forward from cursor
    pub fn delete_word_forward(&mut self) {
        let text = self.input_manager.get_text();
        let cursor_pos = self.input_manager.get_cursor_position();
        let (new_text, new_cursor) = CursorOperations::delete_word_forward(&text, cursor_pos);

        // Store deleted text in kill ring
        let word_end = CursorOperations::find_word_boundary_forward(&text, cursor_pos);
        if word_end > cursor_pos {
            self.kill_ring = text[cursor_pos..word_end].to_string();
        }

        self.input_manager.set_text(new_text);
        self.input_manager.set_cursor_position(new_cursor);
        self.sync_from_input_manager();
    }

    /// Kill line from cursor to end
    pub fn kill_line(&mut self) {
        let text = self.input_manager.get_text();
        let cursor_pos = self.input_manager.get_cursor_position();
        let (new_text, killed) = CursorOperations::kill_line(&text, cursor_pos);

        self.kill_ring = killed;
        self.input_manager.set_text(new_text);
        self.sync_from_input_manager();
    }

    /// Kill line from start to cursor
    pub fn kill_line_backward(&mut self) {
        let text = self.input_manager.get_text();
        let cursor_pos = self.input_manager.get_cursor_position();
        let (new_text, killed, new_cursor) =
            CursorOperations::kill_line_backward(&text, cursor_pos);

        self.kill_ring = killed;
        self.input_manager.set_text(new_text);
        self.input_manager.set_cursor_position(new_cursor);
        self.sync_from_input_manager();
    }

    /// Jump to previous SQL token
    pub fn jump_to_prev_token(&mut self) {
        let text = self.input_manager.get_text();
        let cursor_pos = self.input_manager.get_cursor_position();
        let new_pos = CursorOperations::jump_to_prev_token(&text, cursor_pos);
        self.input_manager.set_cursor_position(new_pos);
        self.sync_from_input_manager();
    }

    /// Jump to next SQL token
    pub fn jump_to_next_token(&mut self) {
        let text = self.input_manager.get_text();
        let cursor_pos = self.input_manager.get_cursor_position();
        let new_pos = CursorOperations::jump_to_next_token(&text, cursor_pos);
        self.input_manager.set_cursor_position(new_pos);
        self.sync_from_input_manager();
    }

    /// Yank (paste) from kill ring
    pub fn yank(&mut self) {
        if !self.kill_ring.is_empty() {
            self.save_state_for_undo();

            let text = self.input_manager.get_text();
            let cursor_pos = self.input_manager.get_cursor_position();

            // Insert kill ring content at cursor position
            let before = text.chars().take(cursor_pos).collect::<String>();
            let after = text.chars().skip(cursor_pos).collect::<String>();
            let new_text = format!("{}{}{}", before, &self.kill_ring, after);
            let new_cursor = cursor_pos + self.kill_ring.len();

            self.input_manager.set_text(new_text);
            self.input_manager.set_cursor_position(new_cursor);
            self.sync_from_input_manager();
        }
    }

    /// Expand SELECT * to column names using schema information
    pub fn expand_asterisk(&mut self, parser: &HybridParser) -> bool {
        let query = self.input_manager.get_text();
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
                        let columns = parser.get_table_columns(table_name);

                        if !columns.is_empty() {
                            // Build the replacement with all columns
                            let columns_str = columns.join(", ");

                            // Replace * with the column list
                            let before_star = &query[..star_abs_pos];
                            let after_star = &query[star_abs_pos + 1..];
                            let new_query = format!("{}{}{}", before_star, columns_str, after_star);

                            // Update the input
                            self.input_manager.set_text(new_query.clone());
                            self.input_manager.set_cursor_position(new_query.len());
                            self.sync_from_input_manager();

                            self.status_message =
                                format!("Expanded * to {} columns", columns.len());
                            return true;
                        } else {
                            self.status_message =
                                format!("No columns found for table '{}'", table_name);
                        }
                    }
                }
            }
        }

        self.status_message = "No SELECT * pattern found to expand".to_string();
        false
    }
}

// Manual Clone implementation for Buffer due to Box<dyn InputManager>
impl Clone for Buffer {
    fn clone(&self) -> Self {
        // Always clone as single-line mode
        let input_manager = create_from_input(self.input.clone());

        Self {
            id: self.id,
            file_path: self.file_path.clone(),
            name: self.name.clone(),
            modified: self.modified,
            csv_client: self.csv_client.clone(),
            csv_mode: self.csv_mode,
            csv_table_name: self.csv_table_name.clone(),
            cache_mode: self.cache_mode,
            results: self.results.clone(),
            cached_data: self.cached_data.clone(),
            mode: self.mode.clone(),
            edit_mode: self.edit_mode.clone(),
            input: self.input.clone(),
            input_manager,
            table_state: self.table_state.clone(),
            last_results_row: self.last_results_row,
            last_scroll_offset: self.last_scroll_offset,
            last_query: self.last_query.clone(),
            status_message: self.status_message.clone(),
            sort_state: self.sort_state.clone(),
            filter_state: self.filter_state.clone(),
            fuzzy_filter_state: self.fuzzy_filter_state.clone(),
            search_state: self.search_state.clone(),
            column_search_state: self.column_search_state.clone(),
            filtered_data: self.filtered_data.clone(),
            column_widths: self.column_widths.clone(),
            scroll_offset: self.scroll_offset,
            current_column: self.current_column,
            pinned_columns: self.pinned_columns.clone(),
            column_stats: self.column_stats.clone(),
            compact_mode: self.compact_mode,
            viewport_lock: self.viewport_lock,
            viewport_lock_row: self.viewport_lock_row,
            show_row_numbers: self.show_row_numbers,
            case_insensitive: self.case_insensitive,
            undo_stack: self.undo_stack.clone(),
            redo_stack: self.redo_stack.clone(),
            kill_ring: self.kill_ring.clone(),
            last_visible_rows: self.last_visible_rows,
            last_query_source: self.last_query_source.clone(),
            highlighted_text_cache: self.highlighted_text_cache.clone(),
            last_highlighted_text: self.last_highlighted_text.clone(),
            saved_input_state: self.saved_input_state.clone(),
        }
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

    /// Clear all buffers (used when loading a new file)
    pub fn clear_all(&mut self) {
        self.buffers.clear();
        self.current_buffer_index = 0;
    }
}
