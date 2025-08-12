use crate::datatable::{DataTable, DataValue};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

/// Represents how data should be sorted
#[derive(Debug, Clone)]
pub struct SortConfig {
    pub column_index: usize,
    pub order: SortOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Filter configuration for the view
#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub pattern: String,
    pub column_index: Option<usize>, // None = search all columns
    pub case_sensitive: bool,
}

/// Search state within the current view
#[derive(Debug, Clone)]
pub struct SearchState {
    pub pattern: String,
    pub current_match: Option<(usize, usize)>, // (row, col)
    pub matches: Vec<(usize, usize)>,
    pub case_sensitive: bool,
}

/// Current view mode for input handling
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Normal,    // Normal navigation
    Filtering, // User is typing a filter
    Searching, // User is typing a search
    Sorting,   // User is selecting sort column
}

/// Simple input for view operations (not full query input)
#[derive(Debug, Clone)]
pub struct SimpleInput {
    pub text: String,
    pub cursor_position: usize,
}

impl SimpleInput {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
    }

    pub fn insert_char(&mut self, ch: char) {
        self.text.insert(self.cursor_position, ch);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.text.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.text.len();
    }
}

/// A view of a DataTable with presentation logic
#[derive(Clone)]
pub struct DataTableView {
    /// The underlying data
    table: DataTable,

    /// Current view mode
    mode: ViewMode,

    /// View state
    sort: Option<SortConfig>,
    filter: Option<FilterConfig>,
    search: Option<SearchState>,

    /// View-specific inputs
    filter_input: SimpleInput,
    search_input: SimpleInput,

    /// Derived/cached view data
    pub visible_rows: Vec<usize>, // Row indices after filtering/sorting
    pub column_widths: Vec<u16>, // Calculated column widths

    /// Navigation state
    pub selected_row: usize,
    pub selected_col: usize,
    pub scroll_offset: usize,     // First visible row (vertical)
    pub horizontal_scroll: usize, // First visible column (horizontal)
    pub page_size: usize,         // How many rows visible at once
    pub visible_col_start: usize, // First visible column index
    pub visible_col_end: usize,   // Last visible column index
}

impl DataTableView {
    /// Create a new view from a DataTable
    pub fn new(table: DataTable) -> Self {
        let visible_rows: Vec<usize> = (0..table.row_count()).collect();
        let column_widths = Self::calculate_column_widths(&table, &visible_rows);
        let column_count = table.column_count();

        Self {
            table,
            mode: ViewMode::Normal,
            sort: None,
            filter: None,
            search: None,
            filter_input: SimpleInput::new(),
            search_input: SimpleInput::new(),
            visible_rows,
            column_widths,
            selected_row: 0,
            selected_col: 0,
            scroll_offset: 0,
            horizontal_scroll: 0,
            page_size: 30, // Show more rows like enhanced TUI
            visible_col_start: 0,
            visible_col_end: column_count, // Start by showing all, will adjust dynamically
        }
    }

    /// Get the underlying table
    pub fn table(&self) -> &DataTable {
        &self.table
    }

    /// Update visible columns based on terminal width and height
    pub fn update_viewport(&mut self, terminal_width: u16, terminal_height: u16) {
        // Calculate how many columns we can fit
        let mut total_width = 0u16;
        let mut end_col = self.visible_col_start;

        for i in self.visible_col_start..self.column_widths.len() {
            let col_width = self.column_widths[i];
            if total_width + col_width + 1 > terminal_width.saturating_sub(2) {
                break; // Won't fit
            }
            total_width += col_width + 1;
            end_col = i + 1;
        }

        // Ensure we show at least one column
        if end_col == self.visible_col_start && self.visible_col_start < self.column_widths.len() {
            end_col = self.visible_col_start + 1;
        }

        self.visible_col_end = end_col;

        // Update page size based on terminal height
        // Typically we have 3 lines for query, 3 for status, 1 for help, 2 for borders = 9 lines of UI
        self.page_size = (terminal_height.saturating_sub(9) as usize).max(10);
    }

    /// Get current view mode
    pub fn mode(&self) -> ViewMode {
        self.mode.clone()
    }

    /// Get visible row count after filtering
    pub fn visible_row_count(&self) -> usize {
        self.visible_rows.len()
    }

    /// Apply a filter to the view
    pub fn apply_filter(
        &mut self,
        pattern: String,
        column_index: Option<usize>,
        case_sensitive: bool,
    ) {
        self.filter = Some(FilterConfig {
            pattern: pattern.clone(),
            column_index,
            case_sensitive,
        });

        self.update_visible_rows();
        self.selected_row = 0; // Reset selection
        self.scroll_offset = 0;
    }

    /// Clear the current filter
    pub fn clear_filter(&mut self) {
        self.filter = None;
        self.update_visible_rows();
        self.selected_row = 0;
        self.scroll_offset = 0;
    }

    /// Apply sorting to the view
    pub fn apply_sort(&mut self, column_index: usize, order: SortOrder) {
        self.sort = Some(SortConfig {
            column_index,
            order,
        });
        self.update_visible_rows();
        self.selected_row = 0;
        self.scroll_offset = 0;
    }

    /// Clear sorting
    pub fn clear_sort(&mut self) {
        self.sort = None;
        self.update_visible_rows();
    }

    /// Start a search within the view
    pub fn start_search(&mut self, pattern: String, case_sensitive: bool) {
        let matches = self.find_matches(&pattern, case_sensitive);
        let current_match = matches.first().copied();

        self.search = Some(SearchState {
            pattern,
            current_match,
            matches,
            case_sensitive,
        });

        // Navigate to first match if found
        if let Some((row_idx, _)) = current_match {
            if let Some(visible_pos) = self.visible_rows.iter().position(|&r| r == row_idx) {
                self.selected_row = visible_pos;
                self.ensure_row_visible(visible_pos);
            }
        }
    }

    /// Navigate to next search match
    pub fn next_search_match(&mut self) {
        if let Some(ref mut search) = self.search {
            if let Some(current) = search.current_match {
                if let Some(current_idx) = search.matches.iter().position(|&m| m == current) {
                    let next_idx = (current_idx + 1) % search.matches.len();
                    search.current_match = search.matches.get(next_idx).copied();

                    if let Some((row_idx, _)) = search.current_match {
                        if let Some(visible_pos) =
                            self.visible_rows.iter().position(|&r| r == row_idx)
                        {
                            self.selected_row = visible_pos;
                            self.ensure_row_visible(visible_pos);
                        }
                    }
                }
            }
        }
    }

    /// Navigate to previous search match
    pub fn prev_search_match(&mut self) {
        if let Some(ref mut search) = self.search {
            if let Some(current) = search.current_match {
                if let Some(current_idx) = search.matches.iter().position(|&m| m == current) {
                    let prev_idx = if current_idx == 0 {
                        search.matches.len() - 1
                    } else {
                        current_idx - 1
                    };
                    search.current_match = search.matches.get(prev_idx).copied();

                    if let Some((row_idx, _)) = search.current_match {
                        if let Some(visible_pos) =
                            self.visible_rows.iter().position(|&r| r == row_idx)
                        {
                            self.selected_row = visible_pos;
                            self.ensure_row_visible(visible_pos);
                        }
                    }
                }
            }
        }
    }

    /// Clear search
    pub fn clear_search(&mut self) {
        self.search = None;
    }

    /// Enter filter mode
    pub fn enter_filter_mode(&mut self) {
        self.mode = ViewMode::Filtering;
        self.filter_input.clear();
    }

    /// Enter search mode  
    pub fn enter_search_mode(&mut self) {
        self.mode = ViewMode::Searching;
        self.search_input.clear();
    }

    /// Exit special modes back to normal
    pub fn exit_special_mode(&mut self) {
        self.mode = ViewMode::Normal;
    }

    /// Handle navigation keys in normal mode
    pub fn handle_navigation(&mut self, key: KeyEvent) -> bool {
        if self.mode != ViewMode::Normal {
            return false;
        }

        match key.code {
            KeyCode::Up => {
                if self.selected_row > 0 {
                    self.selected_row -= 1;
                    self.ensure_row_visible(self.selected_row);
                }
                true
            }
            KeyCode::Down => {
                if self.selected_row + 1 < self.visible_rows.len() {
                    self.selected_row += 1;
                    self.ensure_row_visible(self.selected_row);
                }
                true
            }
            KeyCode::Left => {
                if self.selected_col > 0 {
                    self.selected_col -= 1;
                    self.ensure_column_visible(self.selected_col);
                }
                true
            }
            KeyCode::Right => {
                if self.selected_col + 1 < self.table.column_count() {
                    self.selected_col += 1;
                    self.ensure_column_visible(self.selected_col);
                }
                true
            }
            KeyCode::PageUp => {
                let jump = self.page_size.min(self.selected_row);
                self.selected_row -= jump;
                self.ensure_row_visible(self.selected_row);
                true
            }
            KeyCode::PageDown => {
                let jump = self
                    .page_size
                    .min(self.visible_rows.len() - self.selected_row - 1);
                self.selected_row += jump;
                self.ensure_row_visible(self.selected_row);
                true
            }
            KeyCode::Home => {
                self.selected_row = 0;
                self.scroll_offset = 0;
                true
            }
            KeyCode::End => {
                if !self.visible_rows.is_empty() {
                    self.selected_row = self.visible_rows.len() - 1;
                    self.ensure_row_visible(self.selected_row);
                }
                true
            }
            _ => false,
        }
    }

    /// Handle filter input
    pub fn handle_filter_input(&mut self, key: KeyEvent) -> bool {
        if self.mode != ViewMode::Filtering {
            return false;
        }

        match key.code {
            KeyCode::Char(c) => {
                self.filter_input.insert_char(c);
                true
            }
            KeyCode::Backspace => {
                self.filter_input.delete_char();
                true
            }
            KeyCode::Left => {
                self.filter_input.move_cursor_left();
                true
            }
            KeyCode::Right => {
                self.filter_input.move_cursor_right();
                true
            }
            KeyCode::Home => {
                self.filter_input.move_cursor_home();
                true
            }
            KeyCode::End => {
                self.filter_input.move_cursor_end();
                true
            }
            KeyCode::Enter => {
                // Apply the filter
                self.apply_filter(self.filter_input.text.clone(), None, false);
                self.exit_special_mode();
                true
            }
            KeyCode::Esc => {
                self.exit_special_mode();
                true
            }
            _ => false,
        }
    }

    /// Handle search input
    pub fn handle_search_input(&mut self, key: KeyEvent) -> bool {
        if self.mode != ViewMode::Searching {
            return false;
        }

        match key.code {
            KeyCode::Char(c) => {
                self.search_input.insert_char(c);
                true
            }
            KeyCode::Backspace => {
                self.search_input.delete_char();
                true
            }
            KeyCode::Left => {
                self.search_input.move_cursor_left();
                true
            }
            KeyCode::Right => {
                self.search_input.move_cursor_right();
                true
            }
            KeyCode::Home => {
                self.search_input.move_cursor_home();
                true
            }
            KeyCode::End => {
                self.search_input.move_cursor_end();
                true
            }
            KeyCode::Enter => {
                // Apply the search
                self.start_search(self.search_input.text.clone(), false);
                self.exit_special_mode();
                true
            }
            KeyCode::Esc => {
                self.exit_special_mode();
                true
            }
            _ => false,
        }
    }

    /// Get the currently selected cell value
    pub fn get_selected_value(&self) -> Option<&DataValue> {
        let visible_row = *self.visible_rows.get(self.selected_row)?;
        self.table.get_value(visible_row, self.selected_col)
    }

    /// Get the currently selected column index
    pub fn get_selected_column(&self) -> usize {
        self.selected_col
    }

    /// Get status information for display
    pub fn get_status_info(&self) -> String {
        let total_rows = self.table.row_count();
        let visible_rows = self.visible_rows.len();
        let current_row = self.selected_row + 1;

        let mut status = format!("Row {}/{}", current_row, visible_rows);

        if visible_rows != total_rows {
            status.push_str(&format!(" (filtered from {})", total_rows));
        }

        if let Some(ref filter) = self.filter {
            status.push_str(&format!(" | Filter: '{}'", filter.pattern));
        }

        if let Some(ref search) = self.search {
            status.push_str(&format!(
                " | Search: '{}' ({} matches)",
                search.pattern,
                search.matches.len()
            ));
        }

        if let Some(ref sort) = self.sort {
            let col_name = &self.table.columns[sort.column_index].name;
            let order = match sort.order {
                SortOrder::Ascending => "↑",
                SortOrder::Descending => "↓",
            };
            status.push_str(&format!(" | Sort: {} {}", col_name, order));
        }

        status
    }

    /// Create a ratatui Table widget for rendering
    pub fn create_table_widget(&self) -> Table<'_> {
        // Create header for visible columns only
        let header = Row::new(
            self.table.columns[self.visible_col_start..self.visible_col_end]
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    let actual_col_idx = self.visible_col_start + i;
                    let mut style = Style::default().add_modifier(Modifier::BOLD);
                    if actual_col_idx == self.selected_col {
                        style = style.bg(Color::Blue);
                    }
                    // Just show column name since we display all columns
                    Cell::from(col.name.as_str()).style(style)
                }),
        )
        .style(Style::default().bg(Color::DarkGray));

        // Create visible rows
        let start = self.scroll_offset;
        let end = (start + self.page_size).min(self.visible_rows.len());

        let rows: Vec<Row> = (start..end)
            .map(|visible_idx| {
                let row_idx = self.visible_rows[visible_idx];
                let is_selected = visible_idx == self.selected_row;
                let is_search_match = self.is_search_match(row_idx);

                // Only show cells for visible columns
                let cells: Vec<Cell> = (self.visible_col_start..self.visible_col_end)
                    .map(|col_idx| {
                        let value = self
                            .table
                            .get_value(row_idx, col_idx)
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "".to_string());

                        let mut style = Style::default();

                        if is_selected && col_idx == self.selected_col {
                            style = style.bg(Color::Yellow).fg(Color::Black);
                        } else if is_selected {
                            style = style.bg(Color::Blue).fg(Color::White);
                        } else if is_search_match && self.is_cell_search_match(row_idx, col_idx) {
                            style = style.bg(Color::Green).fg(Color::Black);
                        }

                        Cell::from(value).style(style)
                    })
                    .collect();

                Row::new(cells)
            })
            .collect();

        // Calculate constraints based on visible column widths only
        let constraints: Vec<Constraint> = self.column_widths
            [self.visible_col_start..self.visible_col_end]
            .iter()
            .map(|&width| Constraint::Length(width))
            .collect();

        Table::new(rows, constraints)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Data"))
            .row_highlight_style(Style::default().bg(Color::Blue))
    }

    /// Create input widget for filter/search modes
    pub fn create_input_widget(&self) -> Option<Paragraph<'_>> {
        match self.mode {
            ViewMode::Filtering => Some(
                Paragraph::new(format!("Filter: {}", self.filter_input.text))
                    .block(Block::default().borders(Borders::ALL).title("Filter")),
            ),
            ViewMode::Searching => Some(
                Paragraph::new(format!("Search: {}", self.search_input.text))
                    .block(Block::default().borders(Borders::ALL).title("Search")),
            ),
            _ => None,
        }
    }

    // Private helper methods

    fn update_visible_rows(&mut self) {
        // Start with all rows
        let mut visible: Vec<usize> = (0..self.table.row_count()).collect();

        // Apply filter
        if let Some(ref filter) = self.filter {
            visible.retain(|&row_idx| self.matches_filter(row_idx, filter));
        }

        // Apply sort
        if let Some(ref sort) = self.sort {
            visible.sort_by(|&a, &b| self.compare_rows(a, b, sort));
        }

        self.visible_rows = visible;
        self.column_widths = Self::calculate_column_widths(&self.table, &self.visible_rows);
    }

    fn matches_filter(&self, row_idx: usize, filter: &FilterConfig) -> bool {
        let pattern = if filter.case_sensitive {
            filter.pattern.clone()
        } else {
            filter.pattern.to_lowercase()
        };

        if let Some(col_idx) = filter.column_index {
            // Filter specific column
            if let Some(value) = self.table.get_value(row_idx, col_idx) {
                let text = if filter.case_sensitive {
                    value.to_string()
                } else {
                    value.to_string().to_lowercase()
                };
                text.contains(&pattern)
            } else {
                false
            }
        } else {
            // Filter all columns
            (0..self.table.column_count()).any(|col_idx| {
                if let Some(value) = self.table.get_value(row_idx, col_idx) {
                    let text = if filter.case_sensitive {
                        value.to_string()
                    } else {
                        value.to_string().to_lowercase()
                    };
                    text.contains(&pattern)
                } else {
                    false
                }
            })
        }
    }

    fn compare_rows(&self, a: usize, b: usize, sort: &SortConfig) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        let val_a = self.table.get_value(a, sort.column_index);
        let val_b = self.table.get_value(b, sort.column_index);

        let result = match (val_a, val_b) {
            (Some(a), Some(b)) => self.compare_values(a, b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        };

        match sort.order {
            SortOrder::Ascending => result,
            SortOrder::Descending => result.reverse(),
        }
    }

    fn compare_values(&self, a: &DataValue, b: &DataValue) -> std::cmp::Ordering {
        use crate::datatable::DataValue;
        use std::cmp::Ordering;

        match (a, b) {
            (DataValue::Integer(a), DataValue::Integer(b)) => a.cmp(b),
            (DataValue::Float(a), DataValue::Float(b)) => {
                a.partial_cmp(b).unwrap_or(Ordering::Equal)
            }
            (DataValue::String(a), DataValue::String(b)) => a.cmp(b),
            (DataValue::Boolean(a), DataValue::Boolean(b)) => a.cmp(b),
            (DataValue::DateTime(a), DataValue::DateTime(b)) => a.cmp(b),
            (DataValue::Null, DataValue::Null) => Ordering::Equal,
            (DataValue::Null, _) => Ordering::Greater,
            (_, DataValue::Null) => Ordering::Less,
            // Mixed types - convert to strings for comparison
            (a, b) => a.to_string().cmp(&b.to_string()),
        }
    }

    fn find_matches(&self, pattern: &str, case_sensitive: bool) -> Vec<(usize, usize)> {
        let search_pattern = if case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        let mut matches = Vec::new();

        for &row_idx in &self.visible_rows {
            for col_idx in 0..self.table.column_count() {
                if let Some(value) = self.table.get_value(row_idx, col_idx) {
                    let text = if case_sensitive {
                        value.to_string()
                    } else {
                        value.to_string().to_lowercase()
                    };

                    if text.contains(&search_pattern) {
                        matches.push((row_idx, col_idx));
                    }
                }
            }
        }

        matches
    }

    fn is_search_match(&self, row_idx: usize) -> bool {
        if let Some(ref search) = self.search {
            search.matches.iter().any(|(r, _)| *r == row_idx)
        } else {
            false
        }
    }

    fn is_cell_search_match(&self, row_idx: usize, col_idx: usize) -> bool {
        if let Some(ref search) = self.search {
            search.matches.contains(&(row_idx, col_idx))
        } else {
            false
        }
    }

    fn ensure_row_visible(&mut self, row_idx: usize) {
        if row_idx < self.scroll_offset {
            self.scroll_offset = row_idx;
        } else if row_idx >= self.scroll_offset + self.page_size {
            self.scroll_offset = row_idx - self.page_size + 1;
        }
    }

    fn ensure_column_visible(&mut self, col_idx: usize) {
        // If column is already visible, nothing to do
        if col_idx >= self.visible_col_start && col_idx < self.visible_col_end {
            return;
        }

        if col_idx < self.visible_col_start {
            // Scrolling left - make this the first visible column
            self.visible_col_start = col_idx;
            // visible_col_end will be recalculated by update_viewport
        } else if col_idx >= self.visible_col_end {
            // Scrolling right - shift view to show this column
            self.visible_col_start =
                col_idx - (self.visible_col_end - self.visible_col_start - 1).min(col_idx);
            // visible_col_end will be recalculated by update_viewport
        }

        self.horizontal_scroll = self.visible_col_start;
    }

    fn calculate_column_widths(table: &DataTable, visible_rows: &[usize]) -> Vec<u16> {
        let mut widths = Vec::new();

        // Match enhanced TUI's constants exactly
        const MIN_WIDTH: u16 = 4; // Minimum column width (enhanced uses 4)
        const MAX_WIDTH: u16 = 50; // Maximum column width (enhanced uses 50)
        const PADDING: u16 = 2; // Padding (enhanced adds 2)
        const MAX_ROWS_TO_CHECK: usize = 100; // Sample size (enhanced uses 100)

        // Determine which rows to sample (like enhanced TUI)
        let total_rows = if visible_rows.is_empty() {
            table.row_count()
        } else {
            visible_rows.len()
        };

        let rows_to_check: Vec<usize> = if total_rows <= MAX_ROWS_TO_CHECK {
            // Check all rows for small datasets
            if visible_rows.is_empty() {
                (0..total_rows).collect()
            } else {
                visible_rows.iter().take(total_rows).copied().collect()
            }
        } else {
            // Sample evenly distributed rows for large datasets
            let step = total_rows / MAX_ROWS_TO_CHECK;
            (0..MAX_ROWS_TO_CHECK)
                .map(|i| {
                    let idx = (i * step).min(total_rows - 1);
                    if visible_rows.is_empty() {
                        idx
                    } else {
                        visible_rows[idx]
                    }
                })
                .collect()
        };

        for (col_idx, column) in table.columns.iter().enumerate() {
            // Start with header width
            let mut max_width = column.name.len();

            // Check only sampled rows for this column
            for &row_idx in &rows_to_check {
                if let Some(value) = table.get_value(row_idx, col_idx) {
                    let display_len = value.to_string().len();
                    max_width = max_width.max(display_len);
                }
            }

            // Add padding and set reasonable limits (exactly like enhanced TUI)
            let optimal_width = (max_width + PADDING as usize)
                .max(MIN_WIDTH as usize)
                .min(MAX_WIDTH as usize) as u16;

            widths.push(optimal_width);
        }

        widths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datatable::{DataColumn, DataRow, DataType, DataValue};
    use crossterm::event::KeyModifiers;

    fn create_test_table() -> DataTable {
        let mut table = DataTable::new("test");

        table.add_column(DataColumn::new("id").with_type(DataType::Integer));
        table.add_column(DataColumn::new("name").with_type(DataType::String));
        table.add_column(DataColumn::new("score").with_type(DataType::Float));

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Alice".to_string()),
                DataValue::Float(95.5),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Bob".to_string()),
                DataValue::Float(87.3),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Charlie".to_string()),
                DataValue::Float(92.1),
            ]))
            .unwrap();

        table
    }

    #[test]
    fn test_datatable_view_creation() {
        let table = create_test_table();
        let view = DataTableView::new(table);

        assert_eq!(view.visible_row_count(), 3);
        assert_eq!(view.mode(), ViewMode::Normal);
        assert!(view.filter.is_none());
        assert!(view.search.is_none());
        assert!(view.sort.is_none());
    }

    #[test]
    fn test_filter() {
        let table = create_test_table();
        let mut view = DataTableView::new(table);

        // Filter for names containing "li"
        view.apply_filter("li".to_string(), None, false);

        assert_eq!(view.visible_row_count(), 2); // Alice and Charlie
        assert!(view.filter.is_some());
    }

    #[test]
    fn test_sort() {
        let table = create_test_table();
        let mut view = DataTableView::new(table);

        // Sort by score descending
        view.apply_sort(2, SortOrder::Descending);

        assert_eq!(view.visible_row_count(), 3);

        // First visible row should have the highest score (Alice: 95.5)
        let first_visible_row = view.visible_rows[0];
        let first_value = view.table().get_value(first_visible_row, 1).unwrap();
        assert_eq!(first_value.to_string(), "Alice");
    }

    #[test]
    fn test_search() {
        let table = create_test_table();
        let mut view = DataTableView::new(table);

        view.start_search("Bob".to_string(), false);

        assert!(view.search.is_some());
        let search = view.search.as_ref().unwrap();
        assert_eq!(search.matches.len(), 1);
        assert_eq!(search.current_match, Some((1, 1))); // Row 1, column 1 (name)
    }

    #[test]
    fn test_navigation() {
        let table = create_test_table();
        let mut view = DataTableView::new(table);

        assert_eq!(view.selected_row, 0);
        assert_eq!(view.selected_col, 0);

        // Move down
        view.handle_navigation(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(view.selected_row, 1);

        // Move right
        view.handle_navigation(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(view.selected_col, 1);
    }
}
