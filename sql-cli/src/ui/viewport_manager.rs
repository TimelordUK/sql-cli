use std::collections::HashMap;
/// ViewportManager - A window into the DataView
///
/// This manages the visible portion of data for rendering, handling:
/// - Column width calculations based on visible data
/// - Row/column windowing for virtual scrolling
/// - Caching of expensive calculations
/// - Rendering optimizations
///
/// Architecture:
/// DataTable (immutable storage)
///     → DataView (filtered/sorted/projected data)
///         → ViewportManager (visible window)
///             → Renderer (pixels on screen)
use std::ops::Range;
use std::sync::Arc;

use crate::data::data_view::DataView;
use crate::data::datatable::DataRow;

/// Minimum column width in characters
const MIN_COL_WIDTH: u16 = 3;
/// Maximum column width in characters  
const MAX_COL_WIDTH: u16 = 50;
/// Default column width if no data
const DEFAULT_COL_WIDTH: u16 = 15;

/// Manages the visible viewport into a DataView
pub struct ViewportManager {
    /// The underlying data view
    dataview: Arc<DataView>,

    /// Current viewport bounds
    viewport_rows: Range<usize>,
    viewport_cols: Range<usize>,

    /// Terminal dimensions
    terminal_width: u16,
    terminal_height: u16,

    /// Cached column widths for current viewport
    column_widths: Vec<u16>,

    /// Cache of visible row indices (for efficient scrolling)
    visible_row_cache: Vec<usize>,

    /// Hash of current state for cache invalidation
    cache_signature: u64,

    /// Whether cache needs recalculation
    cache_dirty: bool,
}

impl ViewportManager {
    /// Create a new ViewportManager for a DataView
    pub fn new(dataview: Arc<DataView>) -> Self {
        let col_count = dataview.column_count();

        Self {
            dataview,
            viewport_rows: 0..0,
            viewport_cols: 0..0,
            terminal_width: 80,
            terminal_height: 24,
            column_widths: vec![DEFAULT_COL_WIDTH; col_count],
            visible_row_cache: Vec::new(),
            cache_signature: 0,
            cache_dirty: true,
        }
    }

    /// Update the underlying DataView
    pub fn set_dataview(&mut self, dataview: Arc<DataView>) {
        self.dataview = dataview;
        self.invalidate_cache();
    }

    /// Update viewport position and size
    pub fn set_viewport(&mut self, row_offset: usize, col_offset: usize, width: u16, height: u16) {
        let new_rows = row_offset
            ..row_offset
                .saturating_add(height as usize)
                .min(self.dataview.row_count());
        let new_cols = col_offset
            ..col_offset
                .saturating_add(width as usize)
                .min(self.dataview.column_count());

        // Check if viewport actually changed
        if new_rows != self.viewport_rows || new_cols != self.viewport_cols {
            self.viewport_rows = new_rows;
            self.viewport_cols = new_cols;
            self.terminal_width = width;
            self.terminal_height = height;
            self.cache_dirty = true;
        }
    }

    /// Scroll viewport by relative amount
    pub fn scroll_by(&mut self, row_delta: isize, col_delta: isize) {
        let new_row_start = (self.viewport_rows.start as isize + row_delta).max(0) as usize;
        let new_col_start = (self.viewport_cols.start as isize + col_delta).max(0) as usize;

        self.set_viewport(
            new_row_start,
            new_col_start,
            self.terminal_width,
            self.terminal_height,
        );
    }

    /// Get calculated column widths for current viewport
    pub fn get_column_widths(&mut self) -> &[u16] {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }
        &self.column_widths
    }

    /// Get column width for a specific column
    pub fn get_column_width(&mut self, col_idx: usize) -> u16 {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }
        self.column_widths
            .get(col_idx)
            .copied()
            .unwrap_or(DEFAULT_COL_WIDTH)
    }

    /// Get visible rows in the current viewport
    pub fn get_visible_rows(&self) -> Vec<DataRow> {
        let mut rows = Vec::with_capacity(self.viewport_rows.len());

        for row_idx in self.viewport_rows.clone() {
            if let Some(row) = self.dataview.get_row(row_idx) {
                rows.push(row);
            }
        }

        rows
    }

    /// Get a specific visible row by viewport-relative index
    pub fn get_visible_row(&self, viewport_row: usize) -> Option<DataRow> {
        let absolute_row = self.viewport_rows.start + viewport_row;
        if absolute_row < self.viewport_rows.end {
            self.dataview.get_row(absolute_row)
        } else {
            None
        }
    }

    /// Get visible column headers
    pub fn get_visible_columns(&self) -> Vec<String> {
        let all_columns = self.dataview.column_names();
        let mut visible = Vec::with_capacity(self.viewport_cols.len());

        for col_idx in self.viewport_cols.clone() {
            if col_idx < all_columns.len() {
                visible.push(all_columns[col_idx].clone());
            }
        }

        visible
    }

    /// Get the current viewport row range
    pub fn viewport_rows(&self) -> Range<usize> {
        self.viewport_rows.clone()
    }

    /// Get the current viewport column range
    pub fn viewport_cols(&self) -> Range<usize> {
        self.viewport_cols.clone()
    }

    /// Check if a row is visible in the viewport
    pub fn is_row_visible(&self, row_idx: usize) -> bool {
        self.viewport_rows.contains(&row_idx)
    }

    /// Check if a column is visible in the viewport
    pub fn is_column_visible(&self, col_idx: usize) -> bool {
        self.viewport_cols.contains(&col_idx)
    }

    /// Get total row count from underlying view
    pub fn total_rows(&self) -> usize {
        self.dataview.row_count()
    }

    /// Get total column count from underlying view
    pub fn total_columns(&self) -> usize {
        self.dataview.column_count()
    }

    /// Force cache recalculation on next access
    pub fn invalidate_cache(&mut self) {
        self.cache_dirty = true;
    }

    /// Recalculate column widths based on visible data
    fn recalculate_column_widths(&mut self) {
        let col_count = self.dataview.column_count();
        self.column_widths.resize(col_count, DEFAULT_COL_WIDTH);

        // Get column headers for width calculation
        let headers = self.dataview.column_names();

        // Calculate width for each column based on header and visible data
        for col_idx in 0..col_count {
            // Start with header width
            let header_width = headers.get(col_idx).map(|h| h.len() as u16).unwrap_or(0);

            let mut max_width = header_width;

            // Sample visible rows (limit sampling for performance)
            let sample_size = 100.min(self.viewport_rows.len());
            let sample_step = if self.viewport_rows.len() > sample_size {
                self.viewport_rows.len() / sample_size
            } else {
                1
            };

            for (i, row_idx) in self.viewport_rows.clone().enumerate() {
                // Sample every nth row for performance
                if i % sample_step != 0 && i != 0 {
                    continue;
                }

                if let Some(row) = self.dataview.get_row(row_idx) {
                    if col_idx < row.values.len() {
                        let cell_str = row.values[col_idx].to_string();
                        let cell_width = cell_str.len() as u16;
                        max_width = max_width.max(cell_width);

                        // Early exit if we hit max width
                        if max_width >= MAX_COL_WIDTH {
                            break;
                        }
                    }
                }
            }

            // Apply constraints
            self.column_widths[col_idx] = max_width.clamp(MIN_COL_WIDTH, MAX_COL_WIDTH);
        }

        self.cache_dirty = false;
    }

    /// Calculate optimal column layout for available width
    pub fn calculate_visible_column_indices(&mut self, available_width: u16) -> Vec<usize> {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }

        let mut visible_indices = Vec::new();
        let mut used_width = 0u16;

        // First, add pinned columns
        let pinned = self.dataview.get_pinned_columns();
        for &col_idx in pinned {
            let width = self.column_widths[col_idx];
            if used_width + width + 1 <= available_width {
                // +1 for separator
                visible_indices.push(col_idx);
                used_width += width + 1;
            }
        }

        // Then add regular columns from viewport
        for col_idx in self.viewport_cols.clone() {
            // Skip if already added as pinned
            if pinned.contains(&col_idx) {
                continue;
            }

            let width = self.column_widths[col_idx];
            if used_width + width + 1 <= available_width {
                visible_indices.push(col_idx);
                used_width += width + 1;
            } else {
                break; // No more space
            }
        }

        visible_indices
    }

    /// Get a reference to the underlying DataView
    pub fn dataview(&self) -> &DataView {
        &self.dataview
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataTable, DataValue};

    fn create_test_dataview() -> Arc<DataView> {
        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("name"));
        table.add_column(DataColumn::new("amount"));

        for i in 0..100 {
            table
                .add_row(DataRow::new(vec![
                    DataValue::Integer(i),
                    DataValue::String(format!("Item {}", i)),
                    DataValue::Float(i as f64 * 10.5),
                ]))
                .unwrap();
        }

        Arc::new(DataView::new(Arc::new(table)))
    }

    #[test]
    fn test_viewport_basic() {
        let dataview = create_test_dataview();
        let mut viewport = ViewportManager::new(dataview);

        viewport.set_viewport(0, 0, 80, 24);

        assert_eq!(viewport.viewport_rows(), 0..24);
        assert_eq!(viewport.viewport_cols(), 0..3);

        let visible_rows = viewport.get_visible_rows();
        assert_eq!(visible_rows.len(), 24);
    }

    #[test]
    fn test_column_width_calculation() {
        let dataview = create_test_dataview();
        let mut viewport = ViewportManager::new(dataview);

        viewport.set_viewport(0, 0, 80, 10);

        let widths = viewport.get_column_widths();
        assert_eq!(widths.len(), 3);

        // "id" column should be narrow
        assert!(widths[0] < 10);
        // "name" column should be wider
        assert!(widths[1] > widths[0]);
    }

    #[test]
    fn test_viewport_scrolling() {
        let dataview = create_test_dataview();
        let mut viewport = ViewportManager::new(dataview);

        viewport.set_viewport(0, 0, 80, 24);
        viewport.scroll_by(10, 0);

        assert_eq!(viewport.viewport_rows(), 10..34);

        viewport.scroll_by(-5, 1);
        assert_eq!(viewport.viewport_rows(), 5..29);
        assert_eq!(viewport.viewport_cols(), 1..3);
    }
}
