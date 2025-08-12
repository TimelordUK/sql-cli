use crate::api_client::QueryResponse;
use crate::app_state_container::ColumnSearchState;
use crate::buffer::{
    AppMode, BufferAPI, ColumnStatistics, EditMode, FilterState, FuzzyFilterState, SearchState,
    SortOrder, SortState,
};
use crate::csv_datasource::CsvApiClient;
use crate::datatable::DataTable;
use crate::datatable_view::{DataTableView, SortOrder as ViewSortOrder};
use crate::input_manager::{create_single_line, InputManager};
use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::style::Color;
use ratatui::widgets::TableState;
use std::path::PathBuf;
use tui_input::Input;

/// A Buffer implementation backed by DataTable
/// This allows us to integrate our clean DataTable architecture with the existing enhanced TUI
pub struct DataTableBuffer {
    // --- Identity ---
    id: usize,
    file_path: Option<PathBuf>,
    name: String,
    modified: bool,

    // --- DataTable Backend ---
    view: DataTableView,

    // --- UI State (compatible with existing Buffer) ---
    mode: AppMode,
    edit_mode: EditMode,
    input: Input, // Legacy - kept for compatibility
    input_manager: Box<dyn InputManager>,
    table_state: TableState,
    last_results_row: Option<usize>,
    last_scroll_offset: (usize, usize),

    // --- Query State ---
    last_query: String,
    status_message: String,

    // --- Filter/Search State ---
    sort_state: SortState,
    filter_state: FilterState,
    fuzzy_filter_state: FuzzyFilterState,
    search_state: SearchState,
    column_search_state: ColumnSearchState,
    column_stats: Option<ColumnStatistics>,
    filtered_data: Option<Vec<Vec<String>>>, // Cache for compatibility

    // --- View State ---
    column_widths: Vec<u16>,
    scroll_offset: (usize, usize),
    current_column: usize,
    pinned_columns: Vec<usize>,
    compact_mode: bool,
    viewport_lock: bool,
    viewport_lock_row: Option<usize>,
    show_row_numbers: bool,
    case_insensitive: bool,

    // --- Misc State ---
    undo_stack: Vec<(String, usize)>,
    redo_stack: Vec<(String, usize)>,
    kill_ring: String,
    last_visible_rows: usize,
    last_query_source: Option<String>,

    // --- Syntax Highlighting ---
    highlighted_text_cache: Option<Vec<(String, Color)>>,
    last_highlighted_text: String,

    // --- Input State Stack ---
    saved_input_state: Option<(String, usize)>,
}

impl DataTableBuffer {
    /// Create a new DataTableBuffer from a DataTable
    pub fn new(id: usize, table: DataTable) -> Self {
        let name = table.name.clone();
        let view = DataTableView::new(table);

        Self {
            // --- Identity ---
            id,
            file_path: None,
            name,
            modified: false,

            // --- DataTable Backend ---
            view,

            // --- UI State ---
            mode: AppMode::Command,
            edit_mode: EditMode::SingleLine,
            input: Input::default(),
            input_manager: create_single_line(String::new()),
            table_state: TableState::default(),
            last_results_row: None,
            last_scroll_offset: (0, 0),

            // --- Query State ---
            last_query: String::new(),
            status_message: String::new(),

            // --- Filter/Search State ---
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

            // --- View State ---
            column_widths: Vec::new(),
            scroll_offset: (0, 0),
            current_column: 0,
            pinned_columns: Vec::new(),
            compact_mode: false,
            viewport_lock: false,
            viewport_lock_row: None,
            show_row_numbers: false,
            case_insensitive: false,

            // --- Misc State ---
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            kill_ring: String::new(),
            last_visible_rows: 0,
            last_query_source: None,

            // --- Syntax Highlighting ---
            highlighted_text_cache: None,
            last_highlighted_text: String::new(),

            // --- Input State Stack ---
            saved_input_state: None,
        }
    }

    /// Create from file path
    pub fn from_file(id: usize, file_path: PathBuf) -> anyhow::Result<Self> {
        let table = crate::datatable_loaders::load_json_to_datatable(&file_path, "data")?;
        let mut buffer = Self::new(id, table);
        buffer.file_path = Some(file_path.clone());
        buffer.name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data")
            .to_string();
        Ok(buffer)
    }

    /// Update the filtered data cache from the DataTableView
    fn update_filtered_data_cache(&mut self) {
        // Convert the view's data to the Vec<Vec<String>> format expected by the TUI
        self.filtered_data = Some(self.view.table().to_string_table());

        // Update column widths based on the data
        self.column_widths = self.calculate_column_widths();
    }

    /// Calculate column widths from the DataTable
    fn calculate_column_widths(&self) -> Vec<u16> {
        let table = self.view.table();
        let mut widths = Vec::new();

        for (col_idx, column) in table.columns.iter().enumerate() {
            let mut max_width = column.name.len() as u16;

            // Sample some rows to determine width
            let sample_size = 50.min(table.row_count());
            for row_idx in 0..sample_size {
                if let Some(value) = table.get_value(row_idx, col_idx) {
                    let value_len = value.to_string().len() as u16;
                    max_width = max_width.max(value_len);
                }
            }

            // Add some padding and limit maximum width
            widths.push((max_width + 2).min(50));
        }

        widths
    }

    /// Sync sort state between buffer and view
    fn sync_sort_to_view(&mut self) {
        if let Some(column) = self.sort_state.column {
            let view_order = match self.sort_state.order {
                SortOrder::Ascending => ViewSortOrder::Ascending,
                SortOrder::Descending => ViewSortOrder::Descending,
                SortOrder::None => return, // Don't sort
            };

            self.view.apply_sort(column, view_order);
            self.update_filtered_data_cache();
        }
    }

    /// Sync filter state to view
    fn sync_filter_to_view(&mut self) {
        if self.filter_state.active && !self.filter_state.pattern.is_empty() {
            self.view.apply_filter(
                self.filter_state.pattern.clone(),
                None, // Search all columns for now
                !self.case_insensitive,
            );
            self.update_filtered_data_cache();
        } else {
            self.view.clear_filter();
            self.update_filtered_data_cache();
        }
    }

    /// Get the underlying DataTable (read-only access)
    pub fn table(&self) -> &DataTable {
        self.view.table()
    }

    /// Get the DataTableView (read-only access)  
    pub fn view(&self) -> &DataTableView {
        &self.view
    }
}

impl BufferAPI for DataTableBuffer {
    // --- Identity ---
    fn get_id(&self) -> usize {
        self.id
    }

    // --- Query and Results ---
    fn get_query(&self) -> String {
        self.input_manager.get_text()
    }

    fn set_query(&mut self, query: String) {
        self.input_manager.set_text(query.clone());
        self.input = Input::new(query.clone()).with_cursor(query.len());
    }

    fn get_results(&self) -> Option<&QueryResponse> {
        // DataTableBuffer doesn't use QueryResponse - it has DataTable directly
        // Return None to indicate this is not an API-based buffer
        None
    }

    fn set_results(&mut self, _results: Option<QueryResponse>) {
        // DataTableBuffer manages its own data through DataTable
        // This is a no-op for compatibility
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
        self.last_results_row
    }

    fn set_selected_row(&mut self, row: Option<usize>) {
        self.last_results_row = row;
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
        self.sync_filter_to_view();
    }

    fn is_filter_active(&self) -> bool {
        self.filter_state.active
    }

    fn set_filter_active(&mut self, active: bool) {
        self.filter_state.active = active;
        self.sync_filter_to_view();
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
        // TODO: Implement fuzzy filtering in DataTableView if needed
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
        self.fuzzy_filter_state.active = false;
        self.fuzzy_filter_state.pattern.clear();
        self.fuzzy_filter_state.filtered_indices.clear();
    }

    // --- Search ---
    fn get_search_pattern(&self) -> String {
        self.search_state.pattern.clone()
    }

    fn set_search_pattern(&mut self, pattern: String) {
        self.search_state.pattern = pattern.clone();
        // Sync with DataTableView search
        if !pattern.is_empty() {
            self.view.start_search(pattern, !self.case_insensitive);
        }
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
        self.view.clear_search();
    }

    // --- Column Search ---
    // Column search methods: MIGRATED to AppStateContainer

    // --- Column Statistics ---
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
        self.sync_sort_to_view();
    }

    fn get_sort_order(&self) -> SortOrder {
        self.sort_state.order
    }

    fn set_sort_order(&mut self, order: SortOrder) {
        self.sort_state.order = order;
        self.sync_sort_to_view();
    }

    // --- Buffer Management ---
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

    // --- CSV/Data Source (Not applicable for DataTable, but required for compatibility) ---
    fn get_csv_client(&self) -> Option<&CsvApiClient> {
        None
    }

    fn get_csv_client_mut(&mut self) -> Option<&mut CsvApiClient> {
        None
    }

    fn set_csv_client(&mut self, _client: Option<CsvApiClient>) {
        // No-op for DataTableBuffer
    }

    fn is_csv_mode(&self) -> bool {
        false
    }

    fn set_csv_mode(&mut self, _csv_mode: bool) {
        // No-op for DataTableBuffer
    }

    fn get_table_name(&self) -> String {
        self.view.table().name.clone()
    }

    fn set_table_name(&mut self, _table_name: String) {
        // DataTable name is immutable in this implementation
    }

    fn is_cache_mode(&self) -> bool {
        false
    }

    fn set_cache_mode(&mut self, _cache_mode: bool) {
        // No-op for DataTableBuffer
    }

    fn get_cached_data(&self) -> Option<&Vec<serde_json::Value>> {
        None
    }

    fn set_cached_data(&mut self, _data: Option<Vec<serde_json::Value>>) {
        // No-op for DataTableBuffer
    }

    // --- View State ---
    fn get_column_widths(&self) -> &Vec<u16> {
        &self.column_widths
    }

    fn set_column_widths(&mut self, widths: Vec<u16>) {
        self.column_widths = widths;
    }

    fn get_pinned_columns(&self) -> &Vec<usize> {
        &self.pinned_columns
    }

    fn add_pinned_column(&mut self, col: usize) {
        if !self.pinned_columns.contains(&col) {
            self.pinned_columns.push(col);
        }
    }

    fn remove_pinned_column(&mut self, col: usize) {
        self.pinned_columns.retain(|&x| x != col);
    }

    fn clear_pinned_columns(&mut self) {
        self.pinned_columns.clear();
    }

    fn is_compact_mode(&self) -> bool {
        self.compact_mode
    }

    fn set_compact_mode(&mut self, compact: bool) {
        self.compact_mode = compact;
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

    fn is_show_row_numbers(&self) -> bool {
        self.show_row_numbers
    }

    fn set_show_row_numbers(&mut self, show: bool) {
        self.show_row_numbers = show;
    }

    fn is_case_insensitive(&self) -> bool {
        self.case_insensitive
    }

    fn set_case_insensitive(&mut self, insensitive: bool) {
        self.case_insensitive = insensitive;
    }

    // --- Input State ---
    fn get_input_value(&self) -> String {
        self.input_manager.get_text()
    }

    fn set_input_value(&mut self, value: String) {
        self.input_manager.set_text(value.clone());
        self.input = Input::new(value.clone()).with_cursor(value.len());
    }

    fn get_input_cursor(&self) -> usize {
        self.input_manager.get_cursor_position()
    }

    fn set_input_cursor(&mut self, pos: usize) {
        self.input_manager.set_cursor_position(pos);
    }

    // --- Misc ---
    fn get_kill_ring(&self) -> String {
        self.kill_ring.clone()
    }

    fn set_kill_ring(&mut self, text: String) {
        self.kill_ring = text;
    }

    fn get_last_visible_rows(&self) -> usize {
        self.last_visible_rows
    }

    fn set_last_visible_rows(&mut self, rows: usize) {
        self.last_visible_rows = rows;
    }

    // --- Advanced Operations ---
    fn apply_filter(&mut self) -> Result<()> {
        self.sync_filter_to_view();
        Ok(())
    }

    fn apply_sort(&mut self) -> Result<()> {
        self.sync_sort_to_view();
        Ok(())
    }

    fn search(&mut self) -> Result<()> {
        // Search functionality handled by sync_filter_to_view for now
        Ok(())
    }

    fn clear_filters(&mut self) {
        self.filter_state.active = false;
        self.filter_state.pattern.clear();
        self.view.clear_filter();
        self.update_filtered_data_cache();
    }

    fn get_row_count(&self) -> usize {
        self.view.table().row_count()
    }

    fn get_column_count(&self) -> usize {
        self.view.table().column_count()
    }

    fn get_column_names(&self) -> Vec<String> {
        self.view
            .table()
            .columns
            .iter()
            .map(|col| col.name.clone())
            .collect()
    }

    fn has_cached_data(&self) -> bool {
        false // DataTableBuffer doesn't use cached_data
    }

    // --- Undo/Redo ---
    fn get_undo_stack(&self) -> &Vec<(String, usize)> {
        &self.undo_stack
    }

    fn push_undo(&mut self, state: (String, usize)) {
        self.undo_stack.push(state);
        // Limit undo stack size
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
        // Limit redo stack size
        if self.redo_stack.len() > 100 {
            self.redo_stack.remove(0);
        }
    }

    fn pop_redo(&mut self) -> Option<(String, usize)> {
        self.redo_stack.pop()
    }

    fn clear_redo(&mut self) {
        self.redo_stack.clear();
    }

    // --- Additional required methods ---
    fn is_kill_ring_empty(&self) -> bool {
        self.kill_ring.is_empty()
    }

    fn perform_undo(&mut self) -> bool {
        if let Some((text, cursor)) = self.pop_undo() {
            // Save current state for redo
            let current_state = (self.get_input_value(), self.get_input_cursor());
            self.push_redo(current_state);

            // Restore previous state
            self.set_input_value(text);
            self.set_input_cursor(cursor);
            true
        } else {
            false
        }
    }

    fn perform_redo(&mut self) -> bool {
        if let Some((text, cursor)) = self.pop_redo() {
            // Save current state for undo
            let current_state = (self.get_input_value(), self.get_input_cursor());
            self.push_undo(current_state);

            // Restore state
            self.set_input_value(text);
            self.set_input_cursor(cursor);
            true
        } else {
            false
        }
    }

    fn save_state_for_undo(&mut self) {
        let current_state = (self.get_input_value(), self.get_input_cursor());
        self.push_undo(current_state);
    }

    fn debug_dump(&self) -> String {
        format!(
            "DataTableBuffer {{ id: {}, name: \"{}\", rows: {}, cols: {} }}",
            self.id,
            self.name,
            self.get_row_count(),
            self.get_column_count()
        )
    }

    fn get_input_text(&self) -> String {
        self.input_manager.get_text()
    }

    fn set_input_text(&mut self, text: String) {
        self.input_manager.set_text(text.clone());
        self.input = Input::new(text.clone()).with_cursor(text.len());
    }

    fn handle_input_key(&mut self, event: KeyEvent) -> bool {
        self.input_manager.handle_key_event(event)
    }

    fn switch_input_mode(&mut self, _multiline: bool) {
        // Always use single-line mode
        self.edit_mode = EditMode::SingleLine;
        let current_text = self.input_manager.get_text();
        self.input_manager = create_single_line(current_text);
    }

    fn get_input_cursor_position(&self) -> usize {
        self.input_manager.get_cursor_position()
    }

    fn set_input_cursor_position(&mut self, position: usize) {
        self.input_manager.set_cursor_position(position);
    }

    fn is_input_multiline(&self) -> bool {
        matches!(self.edit_mode, EditMode::MultiLine)
    }

    fn navigate_history_up(&mut self, _history: &[String]) -> bool {
        // For now, delegate to input manager if it supports history
        // TODO: Implement proper history navigation
        false
    }

    fn navigate_history_down(&mut self, _history: &[String]) -> bool {
        // For now, delegate to input manager if it supports history
        // TODO: Implement proper history navigation
        false
    }

    fn reset_history_navigation(&mut self) {
        // TODO: Implement history navigation reset
    }

    fn clear_results(&mut self) {
        // Clear the DataTable or reset to empty state
        // For now, just clear filtered data cache
        self.filtered_data = None;
    }
}
