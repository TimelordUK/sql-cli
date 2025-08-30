// Table rendering context that encapsulates all data needed for rendering
// This decouples the table renderer from TUI internals

use crate::app_state_container::SelectionMode;
use crate::buffer::AppMode;
use crate::data::data_view::SortState;
use std::ops::Range;

/// All the data needed to render a table, collected in one place
/// This allows the table renderer to be independent of TUI internals
#[derive(Debug, Clone)]
pub struct TableRenderContext {
    // ========== Data Source ==========
    /// Total number of rows in the dataset
    pub row_count: usize,

    /// Row indices to display (the visible viewport)
    pub visible_row_indices: Vec<usize>,

    /// The actual data to display (already formatted as strings)
    /// Outer vec is rows, inner vec is columns
    pub data_rows: Vec<Vec<String>>,

    // ========== Column Information ==========
    /// Column headers in visual order
    pub column_headers: Vec<String>,

    /// Column widths in visual order (matching column_headers)
    pub column_widths: Vec<u16>,

    /// Indices of pinned columns (in visual space)
    pub pinned_column_indices: Vec<usize>,

    /// Number of pinned columns (convenience field)
    pub pinned_count: usize,

    // ========== Selection & Navigation ==========
    /// Currently selected row (absolute index, not viewport-relative)
    pub selected_row: usize,

    /// Currently selected column (visual index)
    pub selected_column: usize,

    /// Row viewport range (start..end absolute indices)
    pub row_viewport: Range<usize>,

    /// Selection mode (Cell or Row)
    pub selection_mode: SelectionMode,

    // ========== Visual Indicators ==========
    /// Sort state (which column is sorted and how)
    pub sort_state: Option<SortState>,

    /// Whether to show row numbers
    pub show_row_numbers: bool,

    /// Current application mode (for title bar)
    pub app_mode: AppMode,

    // ========== Search & Filter ==========
    /// Fuzzy filter pattern if active
    pub fuzzy_filter_pattern: Option<String>,

    /// Whether filter is case insensitive
    pub case_insensitive: bool,

    // ========== Layout Information ==========
    /// Available width for the table (excluding borders)
    pub available_width: u16,

    /// Available height for the table (excluding borders)
    pub available_height: u16,
}

impl TableRenderContext {
    /// Check if a given row is the currently selected row
    pub fn is_selected_row(&self, viewport_row_index: usize) -> bool {
        let absolute_row = self.row_viewport.start + viewport_row_index;
        absolute_row == self.selected_row
    }

    /// Check if a given column is the currently selected column
    pub fn is_selected_column(&self, visual_column_index: usize) -> bool {
        visual_column_index == self.selected_column
    }

    /// Check if a column is pinned
    pub fn is_pinned_column(&self, visual_column_index: usize) -> bool {
        visual_column_index < self.pinned_count
    }

    /// Get the crosshair position (selected cell)
    pub fn get_crosshair(&self) -> (usize, usize) {
        (self.selected_row, self.selected_column)
    }

    /// Check if we're at a specific cell
    pub fn is_crosshair_cell(&self, viewport_row_index: usize, visual_column_index: usize) -> bool {
        self.is_selected_row(viewport_row_index) && self.is_selected_column(visual_column_index)
    }

    /// Get sort indicator for a column
    pub fn get_sort_indicator(&self, visual_column_index: usize) -> &str {
        if let Some(ref sort) = self.sort_state {
            if sort.column == Some(visual_column_index) {
                match sort.order {
                    crate::data::data_view::SortOrder::Ascending => " ↑",
                    crate::data::data_view::SortOrder::Descending => " ↓",
                    crate::data::data_view::SortOrder::None => "",
                }
            } else {
                ""
            }
        } else {
            ""
        }
    }

    /// Check if a cell value matches the fuzzy filter
    pub fn cell_matches_filter(&self, cell_value: &str) -> bool {
        if let Some(ref pattern) = self.fuzzy_filter_pattern {
            if pattern.starts_with('\'') && pattern.len() > 1 {
                // Exact match mode
                let search_pattern = &pattern[1..];
                if self.case_insensitive {
                    cell_value
                        .to_lowercase()
                        .contains(&search_pattern.to_lowercase())
                } else {
                    cell_value.contains(search_pattern)
                }
            } else if !pattern.is_empty() {
                // Fuzzy match mode
                use fuzzy_matcher::skim::SkimMatcherV2;
                use fuzzy_matcher::FuzzyMatcher;
                let matcher = if self.case_insensitive {
                    SkimMatcherV2::default().ignore_case()
                } else {
                    SkimMatcherV2::default().respect_case()
                };
                matcher
                    .fuzzy_match(cell_value, pattern)
                    .map(|score| score > 0)
                    .unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// Builder for TableRenderContext to make construction easier
pub struct TableRenderContextBuilder {
    context: TableRenderContext,
}

impl TableRenderContextBuilder {
    pub fn new() -> Self {
        Self {
            context: TableRenderContext {
                row_count: 0,
                visible_row_indices: Vec::new(),
                data_rows: Vec::new(),
                column_headers: Vec::new(),
                column_widths: Vec::new(),
                pinned_column_indices: Vec::new(),
                pinned_count: 0,
                selected_row: 0,
                selected_column: 0,
                row_viewport: 0..0,
                selection_mode: SelectionMode::Cell,
                sort_state: None,
                show_row_numbers: false,
                app_mode: AppMode::Results,
                fuzzy_filter_pattern: None,
                case_insensitive: false,
                available_width: 0,
                available_height: 0,
            },
        }
    }

    pub fn row_count(mut self, count: usize) -> Self {
        self.context.row_count = count;
        self
    }

    pub fn visible_rows(mut self, indices: Vec<usize>, data: Vec<Vec<String>>) -> Self {
        self.context.visible_row_indices = indices;
        self.context.data_rows = data;
        self
    }

    pub fn columns(mut self, headers: Vec<String>, widths: Vec<u16>) -> Self {
        self.context.column_headers = headers;
        self.context.column_widths = widths;
        self
    }

    pub fn pinned_columns(mut self, indices: Vec<usize>) -> Self {
        self.context.pinned_count = indices.len();
        self.context.pinned_column_indices = indices;
        self
    }

    pub fn selection(mut self, row: usize, column: usize, mode: SelectionMode) -> Self {
        self.context.selected_row = row;
        self.context.selected_column = column;
        self.context.selection_mode = mode;
        self
    }

    pub fn row_viewport(mut self, range: Range<usize>) -> Self {
        self.context.row_viewport = range;
        self
    }

    pub fn sort_state(mut self, state: Option<SortState>) -> Self {
        self.context.sort_state = state;
        self
    }

    pub fn display_options(mut self, show_row_numbers: bool, app_mode: AppMode) -> Self {
        self.context.show_row_numbers = show_row_numbers;
        self.context.app_mode = app_mode;
        self
    }

    pub fn filter(mut self, pattern: Option<String>, case_insensitive: bool) -> Self {
        self.context.fuzzy_filter_pattern = pattern;
        self.context.case_insensitive = case_insensitive;
        self
    }

    pub fn dimensions(mut self, width: u16, height: u16) -> Self {
        self.context.available_width = width;
        self.context.available_height = height;
        self
    }

    pub fn build(self) -> TableRenderContext {
        self.context
    }
}
