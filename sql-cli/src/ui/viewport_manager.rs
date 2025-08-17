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
use tracing::debug;

use crate::data::data_view::DataView;
use crate::data::datatable::DataRow;

/// Result of a navigation operation
#[derive(Debug, Clone)]
pub struct NavigationResult {
    /// The new column position
    pub column_position: usize,
    /// The new scroll offset
    pub scroll_offset: usize,
    /// Human-readable description of the operation
    pub description: String,
    /// Whether the operation changed the viewport
    pub viewport_changed: bool,
}

/// Result of a row navigation operation (Page Up/Down, etc.)
#[derive(Debug, Clone)]
pub struct RowNavigationResult {
    /// The new row position
    pub row_position: usize,
    /// The new viewport scroll offset for rows
    pub row_scroll_offset: usize,
    /// Human-readable description of the operation
    pub description: String,
    /// Whether the operation changed the viewport
    pub viewport_changed: bool,
}

/// Result of a column reordering operation
#[derive(Debug, Clone)]
pub struct ColumnReorderResult {
    /// The new column position after reordering
    pub new_column_position: usize,
    /// Human-readable description of the operation
    pub description: String,
    /// Whether the reordering was successful
    pub success: bool,
}

/// Minimum column width in characters
const MIN_COL_WIDTH: u16 = 3;
/// Maximum column width in characters  
const MAX_COL_WIDTH: u16 = 50;
/// Default column width if no data
const DEFAULT_COL_WIDTH: u16 = 15;
/// Padding to add to column widths
const COLUMN_PADDING: u16 = 2;
/// Max ratio of header width to data width (to prevent huge columns for long headers with short data)
const MAX_HEADER_TO_DATA_RATIO: f32 = 1.5;

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

    /// Crosshair position in visual coordinates (row, col)
    /// This is the single source of truth for crosshair position
    crosshair_row: usize,
    crosshair_col: usize,
}

impl ViewportManager {
    /// Get the current viewport column range
    pub fn get_viewport_range(&self) -> std::ops::Range<usize> {
        self.viewport_cols.clone()
    }

    /// Get the current viewport row range
    pub fn get_viewport_rows(&self) -> std::ops::Range<usize> {
        self.viewport_rows.clone()
    }

    /// Set crosshair position in visual coordinates
    pub fn set_crosshair(&mut self, row: usize, col: usize) {
        self.crosshair_row = row;
        self.crosshair_col = col;
        debug!(target: "viewport_manager", 
               "Crosshair set to visual position: row={}, col={}", row, col);
    }

    /// Get crosshair column position in visual coordinates
    pub fn get_crosshair_col(&self) -> usize {
        self.crosshair_col
    }

    /// Get crosshair row position in visual coordinates  
    pub fn get_crosshair_row(&self) -> usize {
        self.crosshair_row
    }

    /// Get crosshair position relative to current viewport for rendering
    /// Returns (row_offset, col_offset) within the viewport, or None if outside
    pub fn get_crosshair_viewport_position(&self) -> Option<(usize, usize)> {
        // Check if crosshair is within the current viewport
        if self.crosshair_row >= self.viewport_rows.start
            && self.crosshair_row < self.viewport_rows.end
            && self.crosshair_col >= self.viewport_cols.start
            && self.crosshair_col < self.viewport_cols.end
        {
            Some((
                self.crosshair_row - self.viewport_rows.start,
                self.crosshair_col - self.viewport_cols.start,
            ))
        } else {
            None
        }
    }

    /// Navigate up one row
    pub fn navigate_row_up(&mut self) -> RowNavigationResult {
        let total_rows = self.dataview.row_count();

        // Vim-like behavior: don't wrap, stay at boundary
        if self.crosshair_row == 0 {
            // Already at first row, don't move
            return RowNavigationResult {
                row_position: 0,
                row_scroll_offset: self.viewport_rows.start,
                description: "Already at first row".to_string(),
                viewport_changed: false,
            };
        }

        let new_row = self.crosshair_row - 1;
        self.crosshair_row = new_row;

        // Adjust viewport if needed
        let viewport_changed = if new_row < self.viewport_rows.start {
            self.viewport_rows = new_row..self.viewport_rows.end.saturating_sub(1);
            true
        } else {
            false
        };

        RowNavigationResult {
            row_position: new_row,
            row_scroll_offset: self.viewport_rows.start,
            description: format!("Move to row {}", new_row + 1),
            viewport_changed,
        }
    }

    /// Navigate down one row
    pub fn navigate_row_down(&mut self) -> RowNavigationResult {
        let total_rows = self.dataview.row_count();

        // Vim-like behavior: don't wrap, stay at boundary
        if self.crosshair_row + 1 >= total_rows {
            // Already at last row, don't move
            let last_row = total_rows.saturating_sub(1);
            return RowNavigationResult {
                row_position: last_row,
                row_scroll_offset: self.viewport_rows.start,
                description: "Already at last row".to_string(),
                viewport_changed: false,
            };
        }

        let new_row = self.crosshair_row + 1;
        self.crosshair_row = new_row;

        // Adjust viewport if needed
        let viewport_changed = if new_row >= self.viewport_rows.end {
            let viewport_height = self.viewport_rows.end - self.viewport_rows.start;
            self.viewport_rows = (new_row + 1).saturating_sub(viewport_height)..(new_row + 1);
            true
        } else {
            false
        };

        RowNavigationResult {
            row_position: new_row,
            row_scroll_offset: self.viewport_rows.start,
            description: format!("Move to row {}", new_row + 1),
            viewport_changed,
        }
    }

    /// Create a new ViewportManager for a DataView
    pub fn new(dataview: Arc<DataView>) -> Self {
        // Get the actual visible column count (after hiding)
        let display_columns = dataview.get_display_columns();
        let visible_col_count = display_columns.len();
        let total_col_count = dataview.source().column_count(); // Total DataTable columns for width array
        let total_rows = dataview.row_count();

        // Initialize viewport in visual coordinate space
        let initial_viewport_cols = if visible_col_count > 0 {
            0..visible_col_count.min(20) // Show up to 20 visual columns initially
        } else {
            0..0
        };

        // Initialize viewport rows to show first page of data
        // Default terminal height is 24, reserve ~10 rows for UI chrome
        let default_visible_rows = 14usize;
        let initial_viewport_rows = if total_rows > 0 {
            0..total_rows.min(default_visible_rows)
        } else {
            0..0
        };

        Self {
            dataview,
            viewport_rows: initial_viewport_rows,
            viewport_cols: initial_viewport_cols,
            terminal_width: 80,
            terminal_height: 24,
            column_widths: vec![DEFAULT_COL_WIDTH; total_col_count], // Size for all DataTable columns
            visible_row_cache: Vec::new(),
            cache_signature: 0,
            cache_dirty: true,
            crosshair_row: 0,
            crosshair_col: 0,
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

        // For columns, we need to work in visual space (visible columns only)
        let display_columns = self.dataview.get_display_columns();
        let visual_column_count = display_columns.len();
        let new_cols = col_offset
            ..col_offset
                .saturating_add(width as usize)
                .min(visual_column_count);

        // Check if viewport actually changed
        if new_rows != self.viewport_rows || new_cols != self.viewport_cols {
            self.viewport_rows = new_rows;
            self.viewport_cols = new_cols;
            self.terminal_width = width;
            self.terminal_height = height;
            self.cache_dirty = true;
        }
    }

    /// Update viewport size based on terminal dimensions
    /// Returns the calculated visible rows for the results area
    pub fn update_terminal_size(&mut self, terminal_width: u16, terminal_height: u16) -> usize {
        // The terminal_height passed here is already the results area height
        // (after input and status areas have been subtracted)
        // So we only need to subtract the borders and header
        // - 1 row for top border
        // - 1 row for header
        // - 1 row for bottom border
        let visible_rows = (terminal_height as usize).saturating_sub(3).max(10);

        let old_viewport = self.viewport_rows.clone();

        // Update our stored terminal dimensions
        self.terminal_width = terminal_width;
        self.terminal_height = terminal_height;

        // Only adjust viewport if terminal size actually changed AND we need to
        // Don't reset the viewport on every render!
        let total_rows = self.dataview.row_count();

        // Check if viewport needs adjustment for the new terminal size
        let viewport_size = self.viewport_rows.end - self.viewport_rows.start;
        if viewport_size != visible_rows && total_rows > 0 {
            // Terminal size changed - adjust viewport to maintain crosshair position
            // Make sure crosshair stays visible in the viewport
            if self.crosshair_row < self.viewport_rows.start {
                // Crosshair is above viewport - scroll up
                self.viewport_rows =
                    self.crosshair_row..(self.crosshair_row + visible_rows).min(total_rows);
            } else if self.crosshair_row >= self.viewport_rows.start + visible_rows {
                // Crosshair is below viewport - scroll down
                let start = self.crosshair_row.saturating_sub(visible_rows - 1);
                self.viewport_rows = start..(start + visible_rows).min(total_rows);
            } else {
                // Crosshair is in viewport - just resize the viewport
                self.viewport_rows = self.viewport_rows.start
                    ..(self.viewport_rows.start + visible_rows).min(total_rows);
            }
        }

        if old_viewport != self.viewport_rows {
            debug!(target: "navigation",
                "ViewportManager::update_terminal_size - viewport changed from {:?} to {:?}, crosshair={}, visible_rows={}",
                old_viewport, self.viewport_rows, self.crosshair_row, visible_rows
            );
        }

        visible_rows
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
    /// Prioritizes data width over header width to maximize visible information
    fn recalculate_column_widths(&mut self) {
        let col_count = self.dataview.column_count();
        self.column_widths.resize(col_count, DEFAULT_COL_WIDTH);

        // Get column headers for width calculation
        let headers = self.dataview.column_names();

        // Calculate width for each column based on header and visible data
        for col_idx in 0..col_count {
            // Track header width separately
            let header_width = headers.get(col_idx).map(|h| h.len() as u16).unwrap_or(0);

            // Track actual data width
            let mut max_data_width = 0u16;
            let mut total_data_width = 0u64;
            let mut data_samples = 0u32;

            // Sample visible rows (limit sampling for performance)
            let sample_size = 100.min(self.viewport_rows.len());
            let sample_step = if self.viewport_rows.len() > sample_size {
                self.viewport_rows.len() / sample_size
            } else {
                1
            };

            for (i, row_idx) in self.viewport_rows.clone().enumerate() {
                // Sample every nth row for performance
                if i % sample_step != 0 && i != 0 && i != self.viewport_rows.len() - 1 {
                    continue;
                }

                if let Some(row) = self.dataview.get_row(row_idx) {
                    if col_idx < row.values.len() {
                        let cell_str = row.values[col_idx].to_string();
                        let cell_width = cell_str.len() as u16;

                        max_data_width = max_data_width.max(cell_width);
                        total_data_width += cell_width as u64;
                        data_samples += 1;

                        // Early exit if we hit max width
                        if max_data_width >= MAX_COL_WIDTH {
                            break;
                        }
                    }
                }
            }

            // Calculate optimal width
            let optimal_width = if data_samples > 0 {
                // Use the maximum data width we found
                let data_based_width = max_data_width + COLUMN_PADDING;

                // If header is significantly longer than data, cap it
                if header_width > max_data_width {
                    let max_allowed_header =
                        (max_data_width as f32 * MAX_HEADER_TO_DATA_RATIO) as u16;
                    data_based_width.max(header_width.min(max_allowed_header))
                } else {
                    // Header fits within data width, use data width
                    data_based_width.max(header_width)
                }
            } else {
                // No data, use header width with some default
                header_width.max(DEFAULT_COL_WIDTH)
            };

            // Apply constraints
            self.column_widths[col_idx] = optimal_width.clamp(MIN_COL_WIDTH, MAX_COL_WIDTH);
        }

        self.cache_dirty = false;
    }

    /// Calculate optimal column layout for available width
    /// Returns a RANGE of visual column indices (0..n) that should be displayed
    /// This works entirely in visual coordinate space - no DataTable indices!
    pub fn calculate_visible_column_indices(&mut self, available_width: u16) -> Vec<usize> {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }

        // Get the display columns from DataView (these are DataTable indices for visible columns)
        let display_columns = self.dataview.get_display_columns();
        let total_visual_columns = display_columns.len();

        if total_visual_columns == 0 {
            return Vec::new();
        }

        let mut used_width = 0u16;
        let separator_width = 1u16;

        // Work in visual coordinate space!
        // Visual indices are 0, 1, 2, 3... (contiguous, no gaps)
        let mut visual_start = self.viewport_cols.start.min(total_visual_columns);
        let mut visual_end = visual_start;

        debug!(target: "viewport_manager",
               "calculate_visible_column_indices: available_width={}, total_visual_columns={}, viewport_start={}",
               available_width, total_visual_columns, visual_start);

        // Calculate how many visual columns we can fit starting from visual_start
        for visual_idx in visual_start..total_visual_columns {
            // Get the DataTable index for this visual position
            let datatable_idx = display_columns[visual_idx];

            let width = self
                .column_widths
                .get(datatable_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);

            if used_width + width + separator_width <= available_width {
                used_width += width + separator_width;
                visual_end = visual_idx + 1;
            } else {
                break;
            }
        }

        // If we couldn't fit anything, ensure we show at least one column
        if visual_end == visual_start && visual_start < total_visual_columns {
            visual_end = visual_start + 1;
        }

        // Now we need to return DataTable indices for compatibility with the renderer
        // (until we fully refactor the renderer to work in visual space)
        let mut result = Vec::new();
        for visual_idx in visual_start..visual_end {
            if visual_idx < display_columns.len() {
                result.push(display_columns[visual_idx]);
            }
        }

        debug!(target: "viewport_manager",
               "calculate_visible_column_indices RESULT: visual range {}..{} -> DataTable indices {:?}",
               visual_start, visual_end, result);

        result

        // Removed the complex optimization logic - we now work with simple ranges
    }

    /// Calculate how many columns we can fit starting from a given column index
    /// This helps determine optimal scrolling positions
    pub fn calculate_columns_that_fit(&mut self, start_col: usize, available_width: u16) -> usize {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }

        let mut used_width = 0u16;
        let mut column_count = 0usize;
        let separator_width = 1u16;

        for col_idx in start_col..self.dataview.column_count() {
            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);
            if used_width + width + separator_width <= available_width {
                used_width += width + separator_width;
                column_count += 1;
            } else {
                break;
            }
        }

        column_count.max(1) // Always show at least one column
    }

    /// Get calculated widths for specific columns
    /// This is useful for rendering when we know which columns will be displayed
    pub fn get_column_widths_for(&mut self, column_indices: &[usize]) -> Vec<u16> {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }

        column_indices
            .iter()
            .map(|&idx| {
                self.column_widths
                    .get(idx)
                    .copied()
                    .unwrap_or(DEFAULT_COL_WIDTH)
            })
            .collect()
    }

    /// Update viewport for column scrolling
    /// This recalculates column widths based on newly visible columns
    pub fn update_column_viewport(&mut self, start_col: usize, available_width: u16) {
        let col_count = self.calculate_columns_that_fit(start_col, available_width);
        let end_col = (start_col + col_count).min(self.dataview.column_count());

        if self.viewport_cols.start != start_col || self.viewport_cols.end != end_col {
            self.viewport_cols = start_col..end_col;
            self.cache_dirty = true;
        }
    }

    /// Get a reference to the underlying DataView
    pub fn dataview(&self) -> &DataView {
        &self.dataview
    }

    /// Get a cloned copy of the underlying DataView (for syncing with Buffer)
    /// This is a temporary solution until we refactor Buffer to use Arc<DataView>
    pub fn clone_dataview(&self) -> DataView {
        (*self.dataview).clone()
    }

    /// Calculate the optimal scroll offset to show the last column
    /// This backtracks from the end to find the best viewport position
    /// Returns the scroll offset in terms of scrollable columns (excluding pinned)
    pub fn calculate_optimal_offset_for_last_column(&mut self, available_width: u16) -> usize {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }

        // Get the display columns (visible columns only, in display order)
        let display_columns = self.dataview.get_display_columns();
        if display_columns.is_empty() {
            return 0;
        }

        let pinned = self.dataview.get_pinned_columns();
        let pinned_count = pinned.len();

        // Calculate how much width is used by pinned columns
        let mut pinned_width = 0u16;
        let separator_width = 1u16;
        for &col_idx in pinned {
            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);
            pinned_width += width + separator_width;
        }

        // Available width for scrollable columns
        let available_for_scrollable = available_width.saturating_sub(pinned_width);

        // Get scrollable columns only (display columns minus pinned)
        let scrollable_columns: Vec<usize> = display_columns
            .iter()
            .filter(|&&col| !pinned.contains(&col))
            .copied()
            .collect();

        if scrollable_columns.is_empty() {
            return 0;
        }

        // Get the last scrollable column
        let last_col_idx = *scrollable_columns.last().unwrap();
        let last_col_width = self
            .column_widths
            .get(last_col_idx)
            .copied()
            .unwrap_or(DEFAULT_COL_WIDTH);

        tracing::debug!(
            "Starting calculation: last_col_idx={}, width={}w, available={}w, scrollable_cols={}",
            last_col_idx,
            last_col_width,
            available_for_scrollable,
            scrollable_columns.len()
        );

        let mut accumulated_width = last_col_width + separator_width;
        let mut best_offset = scrollable_columns.len() - 1; // Start with last scrollable column

        // Now work backwards through scrollable columns to find how many more we can fit
        for (idx, &col_idx) in scrollable_columns.iter().enumerate().rev().skip(1) {
            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);

            let width_with_separator = width + separator_width;

            if accumulated_width + width_with_separator <= available_for_scrollable {
                // This column fits, keep going backwards
                accumulated_width += width_with_separator;
                best_offset = idx; // Use the index in scrollable_columns
                tracing::trace!(
                    "Column {} (idx {}) fits ({}w), accumulated={}w, new offset={}",
                    col_idx,
                    idx,
                    width,
                    accumulated_width,
                    best_offset
                );
            } else {
                // This column doesn't fit, we found our optimal offset
                // The offset should be the next column (since this one doesn't fit)
                best_offset = idx + 1;
                tracing::trace!(
                    "Column {} doesn't fit ({}w would make {}w total), stopping at offset {}",
                    col_idx,
                    width,
                    accumulated_width + width_with_separator,
                    best_offset
                );
                break;
            }
        }

        // best_offset is now the index within scrollable_columns
        // We need to return it as is (it's already the scroll offset for scrollable columns)

        // Now verify that starting from best_offset, we can actually see the last column
        // This is the critical check we were missing!
        let mut test_width = 0u16;
        let mut can_see_last = false;
        for idx in best_offset..scrollable_columns.len() {
            let col_idx = scrollable_columns[idx];
            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);
            test_width += width + separator_width;

            if test_width > available_for_scrollable {
                // We can't fit all columns from best_offset to last
                // Need to adjust offset forward
                tracing::warn!(
                    "Offset {} doesn't show last column! Need {}w but have {}w",
                    best_offset,
                    test_width,
                    available_for_scrollable
                );
                // Move offset forward until last column fits
                best_offset = best_offset + 1;
                can_see_last = false;
                break;
            }
            if idx == scrollable_columns.len() - 1 {
                can_see_last = true;
            }
        }

        // If we still can't see the last column, keep adjusting
        while !can_see_last && best_offset < scrollable_columns.len() {
            test_width = 0;
            for idx in best_offset..scrollable_columns.len() {
                let col_idx = scrollable_columns[idx];
                let width = self
                    .column_widths
                    .get(col_idx)
                    .copied()
                    .unwrap_or(DEFAULT_COL_WIDTH);
                test_width += width + separator_width;

                if test_width > available_for_scrollable {
                    best_offset = best_offset + 1;
                    break;
                }
                if idx == scrollable_columns.len() - 1 {
                    can_see_last = true;
                }
            }
        }

        // best_offset is already in terms of scrollable columns
        tracing::debug!(
            "Final offset for last column: scrollable_offset={}, fits {} columns, last col width: {}w, verified last col visible: {}",
            best_offset,
            scrollable_columns.len() - best_offset,
            last_col_width,
            can_see_last
        );

        best_offset
    }

    /// Debug dump of ViewportManager state for F5 diagnostics
    pub fn debug_dump(&mut self, available_width: u16) -> String {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }

        let mut output = String::new();
        output.push_str("========== VIEWPORT MANAGER DEBUG ==========\n");

        let total_cols = self.dataview.column_count();
        let pinned = self.dataview.get_pinned_columns();
        let pinned_count = pinned.len();

        output.push_str(&format!("Total columns: {}\n", total_cols));
        output.push_str(&format!("Pinned columns: {:?}\n", pinned));
        output.push_str(&format!("Available width: {}w\n", available_width));
        output.push_str(&format!("Current viewport: {:?}\n", self.viewport_cols));
        output.push_str("\n");

        // Show column widths
        output.push_str("Column widths:\n");
        for (idx, &width) in self.column_widths.iter().enumerate() {
            if idx >= 20 && idx < total_cols - 10 {
                if idx == 20 {
                    output.push_str("  ... (showing only first 20 and last 10)\n");
                }
                continue;
            }
            output.push_str(&format!("  [{}] {}w\n", idx, width));
        }
        output.push_str("\n");

        // Test optimal offset calculation step by step
        output.push_str("=== OPTIMAL OFFSET CALCULATION ===\n");
        let last_col_idx = total_cols - 1;
        let last_col_width = self.column_widths.get(last_col_idx).copied().unwrap_or(15);

        // Calculate available width for scrollable columns
        let separator_width = 1u16;
        let mut pinned_width = 0u16;
        for &col_idx in pinned {
            let width = self.column_widths.get(col_idx).copied().unwrap_or(15);
            pinned_width += width + separator_width;
        }
        let available_for_scrollable = available_width.saturating_sub(pinned_width);

        output.push_str(&format!(
            "Last column: {} (width: {}w)\n",
            last_col_idx, last_col_width
        ));
        output.push_str(&format!("Pinned width: {}w\n", pinned_width));
        output.push_str(&format!(
            "Available for scrollable: {}w\n",
            available_for_scrollable
        ));
        output.push_str("\n");

        // Simulate the calculation
        let mut accumulated_width = last_col_width + separator_width;
        let mut best_offset = last_col_idx;

        output.push_str("Backtracking from last column:\n");
        output.push_str(&format!(
            "  Start: column {} = {}w (accumulated: {}w)\n",
            last_col_idx, last_col_width, accumulated_width
        ));

        for col_idx in (pinned_count..last_col_idx).rev() {
            let width = self.column_widths.get(col_idx).copied().unwrap_or(15);
            let width_with_sep = width + separator_width;

            if accumulated_width + width_with_sep <= available_for_scrollable {
                accumulated_width += width_with_sep;
                best_offset = col_idx;
                output.push_str(&format!(
                    "  Column {} fits: {}w (accumulated: {}w, offset: {})\n",
                    col_idx, width, accumulated_width, best_offset
                ));
            } else {
                output.push_str(&format!(
                    "  Column {} doesn't fit: {}w (would make {}w > {}w)\n",
                    col_idx,
                    width,
                    accumulated_width + width_with_sep,
                    available_for_scrollable
                ));
                best_offset = col_idx + 1;
                break;
            }
        }

        output.push_str(&format!(
            "\nCalculated offset: {} (absolute)\n",
            best_offset
        ));

        // Now verify this offset actually works
        output.push_str("\n=== VERIFICATION ===\n");
        let mut verify_width = 0u16;
        let mut can_show_last = true;

        for test_idx in best_offset..=last_col_idx {
            let width = self.column_widths.get(test_idx).copied().unwrap_or(15);
            verify_width += width + separator_width;

            output.push_str(&format!(
                "  Column {}: {}w (running total: {}w)\n",
                test_idx, width, verify_width
            ));

            if verify_width > available_for_scrollable {
                output.push_str(&format!(
                    "    ❌ EXCEEDS LIMIT! {}w > {}w\n",
                    verify_width, available_for_scrollable
                ));
                if test_idx == last_col_idx {
                    can_show_last = false;
                    output.push_str("    ❌ LAST COLUMN NOT VISIBLE!\n");
                }
                break;
            }

            if test_idx == last_col_idx {
                output.push_str("    ✅ LAST COLUMN VISIBLE!\n");
            }
        }

        output.push_str(&format!(
            "\nVerification result: last column visible = {}\n",
            can_show_last
        ));

        // Show what the current viewport actually shows
        output.push_str("\n=== CURRENT VIEWPORT RESULT ===\n");
        let visible_indices = self.calculate_visible_column_indices(available_width);
        output.push_str(&format!("Visible columns: {:?}\n", visible_indices));
        output.push_str(&format!(
            "Last visible column: {}\n",
            visible_indices.last().copied().unwrap_or(0)
        ));
        output.push_str(&format!(
            "Shows last column ({}): {}\n",
            last_col_idx,
            visible_indices.contains(&last_col_idx)
        ));

        output.push_str("============================================\n");
        output
    }

    /// Get column names in DataView's preferred order (pinned first, then display order)
    /// This should be the single source of truth for column ordering from TUI perspective
    pub fn get_column_names_ordered(&self) -> Vec<String> {
        self.dataview.column_names()
    }

    /// Get structured information about visible columns for rendering
    /// Returns (visible_indices, pinned_indices, scrollable_indices)
    pub fn get_visible_columns_info(
        &mut self,
        available_width: u16,
    ) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        debug!(target: "viewport_manager", 
               "get_visible_columns_info CALLED with width={}, current_viewport={:?}", 
               available_width, self.viewport_cols);

        // Get all visible column indices - use viewport-aware method
        let viewport_indices = self.calculate_visible_column_indices(available_width);

        // Sort visible indices according to DataView's display order (pinned first)
        let display_order = self.dataview.get_display_columns();
        let mut visible_indices = Vec::new();

        // Add columns in DataView's preferred order, but only if they're in the viewport
        for &col_idx in &display_order {
            if viewport_indices.contains(&col_idx) {
                visible_indices.push(col_idx);
            }
        }

        // Get pinned column indices from DataView
        let pinned_columns = self.dataview.get_pinned_columns();

        // Split visible columns into pinned and scrollable
        let mut pinned_visible = Vec::new();
        let mut scrollable_visible = Vec::new();

        for &idx in &visible_indices {
            if pinned_columns.contains(&idx) {
                pinned_visible.push(idx);
            } else {
                scrollable_visible.push(idx);
            }
        }

        debug!(target: "viewport_manager", 
               "get_visible_columns_info: viewport={:?} -> ordered={:?} ({} pinned, {} scrollable)",
               viewport_indices, visible_indices, pinned_visible.len(), scrollable_visible.len());

        debug!(target: "viewport_manager", 
               "RENDERER DEBUG: viewport_indices={:?}, display_order={:?}, visible_indices={:?}",
               viewport_indices, display_order, visible_indices);

        (visible_indices, pinned_visible, scrollable_visible)
    }

    /// Calculate the actual X positions in terminal coordinates for visible columns
    /// Returns (column_indices, x_positions) where x_positions[i] is the starting x position for column_indices[i]
    pub fn calculate_column_x_positions(&mut self, available_width: u16) -> (Vec<usize>, Vec<u16>) {
        let visible_indices = self.calculate_visible_column_indices(available_width);
        let mut x_positions = Vec::new();
        let mut current_x = 0u16;
        let separator_width = 1u16;

        for &col_idx in &visible_indices {
            x_positions.push(current_x);
            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);
            current_x += width + separator_width;
        }

        (visible_indices, x_positions)
    }

    /// Get the X position in terminal coordinates for a specific column (if visible)
    pub fn get_column_x_position(&mut self, column: usize, available_width: u16) -> Option<u16> {
        let (indices, positions) = self.calculate_column_x_positions(available_width);
        indices
            .iter()
            .position(|&idx| idx == column)
            .and_then(|pos| positions.get(pos).copied())
    }

    /// Get visible column indices that fit in available width, preserving DataView's order
    pub fn calculate_visible_column_indices_ordered(&mut self, available_width: u16) -> Vec<usize> {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }

        // Get DataView's preferred column order (pinned first)
        let ordered_columns = self.dataview.get_display_columns();
        let mut visible_indices = Vec::new();
        let mut used_width = 0u16;
        let separator_width = 1u16;

        tracing::trace!(
            "ViewportManager: Starting ordered column layout. Available width: {}w, DataView order: {:?}",
            available_width,
            ordered_columns
        );

        // Process columns in DataView's order (pinned first, then display order)
        for &col_idx in &ordered_columns {
            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);

            if used_width + width + separator_width <= available_width {
                visible_indices.push(col_idx);
                used_width += width + separator_width;
                tracing::trace!(
                    "Added column {} in DataView order: {}w (total used: {}w)",
                    col_idx,
                    width,
                    used_width
                );
            } else {
                tracing::trace!(
                    "Skipped column {} ({}w) - would exceed available width",
                    col_idx,
                    width
                );
                break; // Stop when we run out of space, maintaining order
            }
        }

        tracing::trace!(
            "Final ordered layout: {} columns visible {:?}, {}w used of {}w",
            visible_indices.len(),
            visible_indices,
            used_width,
            available_width
        );

        visible_indices
    }

    /// Convert a DataTable column index to its display position within the current visible columns
    /// Returns None if the column is not currently visible
    pub fn get_display_position_for_datatable_column(
        &mut self,
        datatable_column: usize,
        available_width: u16,
    ) -> Option<usize> {
        let visible_columns_info = self.get_visible_columns_info(available_width);
        let visible_indices = visible_columns_info.0;

        // Find the position of the datatable column in the visible columns list
        let position = visible_indices
            .iter()
            .position(|&col| col == datatable_column);

        debug!(target: "viewport_manager",
               "get_display_position_for_datatable_column: datatable_column={}, visible_indices={:?}, position={:?}",
               datatable_column, visible_indices, position);

        position
    }

    /// Get the exact crosshair column position for rendering
    /// This is the single source of truth for which column should be highlighted
    /// For now, current_column is still a DataTable index (due to Buffer storing DataTable indices)
    /// This converts it to the correct display position
    pub fn get_crosshair_column(
        &mut self,
        current_datatable_column: usize,
        available_width: u16,
    ) -> Option<usize> {
        // Get visible columns
        let visible_columns_info = self.get_visible_columns_info(available_width);
        let visible_indices = visible_columns_info.0;

        // Find where this DataTable column appears in the visible columns
        let position = visible_indices
            .iter()
            .position(|&col| col == current_datatable_column);

        debug!(target: "viewport_manager",
               "CROSSHAIR: current_datatable_column={}, visible_indices={:?}, crosshair_position={:?}",
               current_datatable_column, visible_indices, position);

        position
    }

    /// Get the complete visual display data for rendering
    /// Returns (headers, rows, widths) where everything is in visual order with no gaps
    /// This method works entirely in visual coordinate space
    pub fn get_visual_display(
        &mut self,
        available_width: u16,
        _row_indices: &[usize], // DEPRECATED - using internal viewport_rows instead
    ) -> (Vec<String>, Vec<Vec<String>>, Vec<u16>) {
        // Use our internal viewport_rows to determine what rows to display
        let row_indices: Vec<usize> = (self.viewport_rows.start..self.viewport_rows.end).collect();

        debug!(target: "viewport_manager",
               "get_visual_display: Using viewport_rows {:?} -> row_indices: {:?} (first 5)",
               self.viewport_rows,
               row_indices.iter().take(5).collect::<Vec<_>>());
        // Get ALL visual columns from DataView (already filtered for hidden columns)
        let all_headers = self.dataview.get_display_column_names();
        let total_visual_columns = all_headers.len();

        debug!(target: "viewport_manager",
               "get_visual_display: {} total visual columns, viewport: {:?}",
               total_visual_columns, self.viewport_cols);

        // Determine visual range to display
        let visual_start = self.viewport_cols.start.min(total_visual_columns);
        let visual_end = self.viewport_cols.end.min(total_visual_columns);

        debug!(target: "viewport_manager",
               "Showing visual columns {}..{} (of {})",
               visual_start, visual_end, total_visual_columns);

        // Get headers for the visual range
        let headers: Vec<String> = all_headers[visual_start..visual_end].to_vec();

        // Get data from DataView in visual column order
        // IMPORTANT: row_indices contains display row indices (0-based positions in the result set)
        let visual_rows: Vec<Vec<String>> = row_indices
            .iter()
            .filter_map(|&display_row_idx| {
                // Get the full row in visual column order from DataView
                // display_row_idx is the position in the filtered/sorted result set
                let row_data = self.dataview.get_row_visual_values(display_row_idx);
                if let Some(ref full_row) = row_data {
                    // Debug first few and last few rows to track what we're actually getting
                    if display_row_idx < 5 || display_row_idx >= 19900 {
                        debug!(target: "viewport_manager",
                               "DATAVIEW FETCH: display_row_idx {} -> data: {:?} (first 3 cols)",
                               display_row_idx,
                               full_row.iter().take(3).collect::<Vec<_>>());
                    }
                }
                row_data.map(|full_row| {
                    // Slice to just the visible columns
                    full_row[visual_start..visual_end.min(full_row.len())].to_vec()
                })
            })
            .collect();

        // For now, use default widths - we'll fix width calculation separately
        let widths: Vec<u16> = vec![DEFAULT_COL_WIDTH; headers.len()];

        debug!(target: "viewport_manager",
               "get_visual_display RESULT: {} headers, {} rows",
               headers.len(), visual_rows.len());
        if let Some(first_row) = visual_rows.first() {
            debug!(target: "viewport_manager",
                   "Alignment check (FIRST ROW): {:?}",
                   headers.iter().zip(first_row).take(5)
                       .map(|(h, v)| format!("{}: {}", h, v)).collect::<Vec<_>>());
        }
        if let Some(last_row) = visual_rows.last() {
            debug!(target: "viewport_manager",
                   "Alignment check (LAST ROW): {:?}",
                   headers.iter().zip(last_row).take(5)
                       .map(|(h, v)| format!("{}: {}", h, v)).collect::<Vec<_>>());
        }

        (headers, visual_rows, widths)
    }

    /// Get the column headers for the visible columns in the correct order
    /// This ensures headers align with the data columns when columns are hidden
    pub fn get_visible_column_headers(&self, visible_indices: &[usize]) -> Vec<String> {
        let mut headers = Vec::new();

        // Get the column names directly from the DataTable source
        // The visible_indices are DataTable column indices, so we can use them directly
        let source = self.dataview.source();
        let all_column_names = source.column_names();

        for &datatable_idx in visible_indices {
            if datatable_idx < all_column_names.len() {
                headers.push(all_column_names[datatable_idx].clone());
            } else {
                // Fallback for invalid indices
                headers.push(format!("Column_{}", datatable_idx));
            }
        }

        debug!(target: "viewport_manager", 
               "get_visible_column_headers: indices={:?} -> headers={:?}", 
               visible_indices, headers);

        headers
    }

    /// Get crosshair column position for rendering when given a display position
    /// This is for the new architecture where Buffer stores display positions
    pub fn get_crosshair_column_for_display(
        &mut self,
        current_display_position: usize,
        available_width: u16,
    ) -> Option<usize> {
        // Get the display columns order from DataView
        let display_columns = self.dataview.get_display_columns();

        // Validate the display position
        if current_display_position >= display_columns.len() {
            debug!(target: "viewport_manager",
                   "CROSSHAIR DISPLAY: display_position {} out of bounds (max {})",
                   current_display_position, display_columns.len());
            return None;
        }

        // Get the DataTable column index for this display position
        let datatable_column = display_columns[current_display_position];

        // Get visible columns for rendering
        let visible_columns_info = self.get_visible_columns_info(available_width);
        let visible_indices = visible_columns_info.0;

        // Find where this DataTable column appears in the visible columns
        let position = visible_indices
            .iter()
            .position(|&col| col == datatable_column);

        debug!(target: "viewport_manager",
               "CROSSHAIR DISPLAY: display_pos={} -> datatable_col={} -> visible_indices={:?} -> crosshair_pos={:?}",
               current_display_position, datatable_column, visible_indices, position);

        position
    }

    /// Calculate viewport efficiency metrics
    pub fn calculate_efficiency_metrics(&mut self, available_width: u16) -> ViewportEfficiency {
        // Get the visible columns
        let visible_indices = self.calculate_visible_column_indices(available_width);

        // Calculate total width used
        let mut used_width = 0u16;
        let separator_width = 1u16;

        for &col_idx in &visible_indices {
            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);
            used_width += width + separator_width;
        }

        // Remove the last separator since it's not needed after the last column
        if !visible_indices.is_empty() {
            used_width = used_width.saturating_sub(separator_width);
        }

        let wasted_space = available_width.saturating_sub(used_width);

        // Find the next column that didn't fit
        let next_column_width = if !visible_indices.is_empty() {
            let last_visible = *visible_indices.last().unwrap();
            if last_visible + 1 < self.column_widths.len() {
                Some(self.column_widths[last_visible + 1])
            } else {
                None
            }
        } else {
            None
        };

        // Find ALL columns that COULD fit in the wasted space
        let mut columns_that_could_fit = Vec::new();
        if wasted_space > MIN_COL_WIDTH + separator_width {
            for (idx, &width) in self.column_widths.iter().enumerate() {
                // Skip already visible columns
                if !visible_indices.contains(&idx) {
                    if width + separator_width <= wasted_space {
                        columns_that_could_fit.push((idx, width));
                    }
                }
            }
        }

        let efficiency_percent = if available_width > 0 {
            ((used_width as f32 / available_width as f32) * 100.0) as u8
        } else {
            0
        };

        ViewportEfficiency {
            available_width,
            used_width,
            wasted_space,
            efficiency_percent,
            visible_columns: visible_indices.len(),
            column_widths: visible_indices
                .iter()
                .map(|&idx| {
                    self.column_widths
                        .get(idx)
                        .copied()
                        .unwrap_or(DEFAULT_COL_WIDTH)
                })
                .collect(),
            next_column_width,
            columns_that_could_fit,
        }
    }

    /// Navigate to the first column (first scrollable column after pinned columns)
    /// This centralizes the logic for first column navigation
    pub fn navigate_to_first_column(&mut self) -> NavigationResult {
        // Get pinned column count from dataview
        let pinned_count = self.dataview.get_pinned_columns().len();
        let pinned_names = self.dataview.get_pinned_column_names();

        // First scrollable column is at index = pinned_count
        let first_scrollable_column = pinned_count;

        // Reset viewport to beginning (scroll offset = 0)
        let new_scroll_offset = 0;
        let old_scroll_offset = self.viewport_cols.start;

        // Update our internal viewport state
        self.viewport_cols = new_scroll_offset..self.viewport_cols.end;

        // Create description
        let description = if pinned_count > 0 {
            format!(
                "First scrollable column selected (after {} pinned: {:?})",
                pinned_count, pinned_names
            )
        } else {
            "First column selected".to_string()
        };

        let viewport_changed = old_scroll_offset != new_scroll_offset;

        debug!(target: "viewport_manager", 
               "navigate_to_first_column: pinned={}, first_scrollable={}, scroll_offset={}->{}",
               pinned_count, first_scrollable_column, old_scroll_offset, new_scroll_offset);

        NavigationResult {
            column_position: first_scrollable_column,
            scroll_offset: new_scroll_offset,
            description,
            viewport_changed,
        }
    }

    /// Navigate to the last column (rightmost visible column)
    /// This centralizes the logic for last column navigation
    pub fn navigate_to_last_column(&mut self) -> NavigationResult {
        // Get the display columns (visual order)
        let display_columns = self.dataview.get_display_columns();
        let total_visual_columns = display_columns.len();

        if total_visual_columns == 0 {
            return NavigationResult {
                column_position: 0,
                scroll_offset: 0,
                description: "No columns available".to_string(),
                viewport_changed: false,
            };
        }

        // Last column is at visual index total_visual_columns - 1
        let last_visual_column = total_visual_columns - 1;

        // Update crosshair to last visual column
        self.crosshair_col = last_visual_column;

        // Calculate the appropriate scroll offset to make the last column visible
        // We need to ensure the last column fits within the viewport
        let available_width = self.terminal_width;
        let pinned_count = self.dataview.get_pinned_columns().len();

        // Calculate pinned width
        let mut pinned_width = 0u16;
        for i in 0..pinned_count {
            let col_idx = display_columns[i];
            let width = self.column_widths.get(col_idx).copied().unwrap_or(15);
            pinned_width += width + 3; // separator width
        }

        let available_for_scrollable = available_width.saturating_sub(pinned_width);

        // Calculate the optimal scroll offset to show the last column
        let mut accumulated_width = 0u16;
        let mut new_scroll_offset = last_visual_column;

        // Work backwards from the last column to find the best scroll position
        for visual_idx in (pinned_count..=last_visual_column).rev() {
            let col_idx = display_columns[visual_idx];
            let width = self.column_widths.get(col_idx).copied().unwrap_or(15);
            accumulated_width += width + 3; // separator width

            if accumulated_width > available_for_scrollable {
                // We've exceeded available width, use the next column as scroll start
                new_scroll_offset = visual_idx + 1;
                break;
            }
            new_scroll_offset = visual_idx;
        }

        // Ensure scroll offset doesn't go below pinned columns
        new_scroll_offset = new_scroll_offset.max(pinned_count);

        let old_scroll_offset = self.viewport_cols.start;
        let viewport_changed = old_scroll_offset != new_scroll_offset;

        // Update our internal viewport state
        self.viewport_cols = new_scroll_offset..self.viewport_cols.end;

        debug!(target: "viewport_manager", 
               "navigate_to_last_column: last_visual={}, scroll_offset={}->{}",
               last_visual_column, old_scroll_offset, new_scroll_offset);

        NavigationResult {
            column_position: last_visual_column,
            scroll_offset: new_scroll_offset,
            description: format!("Last column selected (column {})", last_visual_column + 1),
            viewport_changed,
        }
    }

    /// Navigate one column to the left with intelligent wrapping and scrolling
    /// This method handles everything: column movement, viewport tracking, and scrolling
    /// IMPORTANT: current_display_position is a logical display position (0,1,2,3...), NOT a DataTable index
    pub fn navigate_column_left(&mut self, current_display_position: usize) -> NavigationResult {
        // Get the DataView's display order (pinned columns first, then others)
        let display_columns = self.dataview.get_display_columns();
        let total_display_columns = display_columns.len();

        debug!(target: "viewport_manager", 
               "navigate_column_left: current_display_pos={}, total_display={}, display_order={:?}", 
               current_display_position, total_display_columns, display_columns);

        // Validate current position
        let current_display_index = if current_display_position < total_display_columns {
            current_display_position
        } else {
            0 // Reset to first if out of bounds
        };

        debug!(target: "viewport_manager", 
               "navigate_column_left: using display_index={}", 
               current_display_index);

        // Calculate new display position (move left in display order)
        // Vim-like behavior: don't wrap, stay at boundary
        if current_display_index == 0 {
            // Already at first column, don't move
            let first_datatable_column = display_columns.get(0).copied().unwrap_or(0);
            return NavigationResult {
                column_position: first_datatable_column,
                scroll_offset: self.viewport_cols.start,
                description: "Already at first column".to_string(),
                viewport_changed: false,
            };
        }

        let new_display_index = current_display_index - 1;

        // Get the actual DataTable column index from display order for internal operations
        let new_datatable_column = display_columns
            .get(new_display_index)
            .copied()
            .unwrap_or_else(|| {
                display_columns
                    .get(current_display_index)
                    .copied()
                    .unwrap_or(0)
            });

        let old_scroll_offset = self.viewport_cols.start;

        // Don't pre-extend viewport - let set_current_column handle all viewport adjustments
        // This avoids the issue where we extend the viewport, then set_current_column thinks
        // the column is already visible and doesn't scroll
        debug!(target: "viewport_manager", 
               "navigate_column_left: moving to datatable_column={}, current viewport={:?}", 
               new_datatable_column, self.viewport_cols);

        // Use set_current_column to handle viewport adjustment automatically (this takes DataTable index)
        let viewport_changed = self.set_current_column(new_datatable_column);

        // Update crosshair to the new visual position
        self.crosshair_col = new_display_index;

        let column_names = self.dataview.column_names();
        let column_name = display_columns
            .get(new_display_index)
            .and_then(|&dt_idx| column_names.get(dt_idx))
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        let description = format!(
            "Navigate left to column '{}' ({})",
            column_name,
            new_display_index + 1
        );

        debug!(target: "viewport_manager", 
               "navigate_column_left: display_pos {}→{}, datatable_col: {}, scroll: {}→{}, viewport_changed={}", 
               current_display_index, new_display_index, new_datatable_column,
               old_scroll_offset, self.viewport_cols.start, viewport_changed);

        NavigationResult {
            column_position: new_datatable_column, // Return DataTable index for Buffer
            scroll_offset: self.viewport_cols.start,
            description,
            viewport_changed,
        }
    }

    /// Convert a DataTable column index to logical display index
    pub fn datatable_to_display_index(&self, datatable_column: usize) -> Option<usize> {
        let display_columns = self.dataview.get_display_columns();
        display_columns
            .iter()
            .position(|&col| col == datatable_column)
    }

    /// Convert a logical display index to DataTable column index
    pub fn display_index_to_datatable(&self, display_index: usize) -> Option<usize> {
        let display_columns = self.dataview.get_display_columns();
        display_columns.get(display_index).copied()
    }

    /// Navigate one column to the right with intelligent wrapping and scrolling
    /// IMPORTANT: current_display_position is a logical display position (0,1,2,3...), NOT a DataTable index
    pub fn navigate_column_right(&mut self, current_display_position: usize) -> NavigationResult {
        let display_columns = self.dataview.get_display_columns();
        let total_display_columns = display_columns.len();

        debug!(target: "viewport_manager", 
               "navigate_column_right ENTRY: current_display_pos={}, display_columns={:?}", 
               current_display_position, display_columns);

        // Validate current position
        let current_display_index = if current_display_position < total_display_columns {
            current_display_position
        } else {
            0 // Reset to first if out of bounds
        };

        debug!(target: "viewport_manager", 
               "navigate_column_right: using display_index={}", 
               current_display_index);

        // Calculate new display position (move right without wrapping)
        // Vim-like behavior: don't wrap, stay at boundary
        if current_display_index + 1 >= total_display_columns {
            // Already at last column, don't move
            let last_display_index = total_display_columns.saturating_sub(1);
            let last_datatable_column = display_columns
                .get(last_display_index)
                .copied()
                .unwrap_or(0);
            return NavigationResult {
                column_position: last_datatable_column,
                scroll_offset: self.viewport_cols.start,
                description: "Already at last column".to_string(),
                viewport_changed: false,
            };
        }

        let new_display_index = current_display_index + 1;

        // Get the actual DataTable column index for the new position (for internal operations)
        let new_datatable_column = display_columns
            .get(new_display_index)
            .copied()
            .unwrap_or_else(|| {
                // Fallback: if something goes wrong, use first column
                display_columns.get(0).copied().unwrap_or(0)
            });

        debug!(target: "viewport_manager", 
               "navigate_column_right: display_pos {}→{}, datatable_column={}", 
               current_display_index, new_display_index, new_datatable_column);

        let old_scroll_offset = self.viewport_cols.start;

        // Ensure the viewport includes the target column before checking visibility
        // This fixes the range issue where column N is not included in range start..N
        // Don't pre-extend viewport - let set_current_column handle all viewport adjustments
        // This avoids the issue where we extend the viewport, then set_current_column thinks
        // the column is already visible and doesn't scroll
        debug!(target: "viewport_manager", 
               "navigate_column_right: moving to datatable_column={}, current viewport={:?}", 
               new_datatable_column, self.viewport_cols);

        // Use set_current_column to handle viewport adjustment automatically (this takes DataTable index)
        debug!(target: "viewport_manager", 
               "navigate_column_right: before set_current_column({}), viewport={:?}", 
               new_datatable_column, self.viewport_cols);
        let viewport_changed = self.set_current_column(new_datatable_column);
        debug!(target: "viewport_manager", 
               "navigate_column_right: after set_current_column({}), viewport={:?}, changed={}", 
               new_datatable_column, self.viewport_cols, viewport_changed);

        // Update crosshair to the new visual position
        self.crosshair_col = new_display_index;

        let column_names = self.dataview.column_names();
        let column_name = display_columns
            .get(new_display_index)
            .and_then(|&dt_idx| column_names.get(dt_idx))
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        let description = format!(
            "Navigate right to column '{}' ({})",
            column_name,
            new_display_index + 1
        );

        debug!(target: "viewport_manager", 
               "navigate_column_right EXIT: display_pos {}→{}, datatable_col: {}, viewport: {:?}, scroll: {}→{}, viewport_changed={}", 
               current_display_index, new_display_index, new_datatable_column,
               self.viewport_cols, old_scroll_offset, self.viewport_cols.start, viewport_changed);

        NavigationResult {
            column_position: new_datatable_column, // Return DataTable index for Buffer
            scroll_offset: self.viewport_cols.start,
            description,
            viewport_changed,
        }
    }

    /// Navigate one page down in the data
    pub fn page_down(&mut self, current_row: usize, total_rows: usize) -> RowNavigationResult {
        // Calculate visible rows (viewport height)
        let visible_rows = self.terminal_height.saturating_sub(6) as usize; // Account for headers, borders, status

        debug!(target: "viewport_manager", 
               "page_down: current_row={}, total_rows={}, visible_rows={}, current_viewport_rows={:?}", 
               current_row, total_rows, visible_rows, self.viewport_rows);

        // Calculate new row position (move down by one page)
        let new_row = (current_row + visible_rows).min(total_rows.saturating_sub(1));

        // Calculate new scroll offset to keep new position visible
        let old_scroll_offset = self.viewport_rows.start;
        let new_scroll_offset = if new_row >= self.viewport_rows.start + visible_rows {
            // Need to scroll down
            (new_row + 1).saturating_sub(visible_rows)
        } else {
            // Keep current scroll
            old_scroll_offset
        };

        // Update viewport
        self.viewport_rows = new_scroll_offset..(new_scroll_offset + visible_rows).min(total_rows);
        let viewport_changed = new_scroll_offset != old_scroll_offset;

        let description = format!(
            "Page down: row {} → {} (of {})",
            current_row + 1,
            new_row + 1,
            total_rows
        );

        debug!(target: "viewport_manager", 
               "page_down result: new_row={}, scroll_offset={}→{}, viewport_changed={}", 
               new_row, old_scroll_offset, new_scroll_offset, viewport_changed);

        RowNavigationResult {
            row_position: new_row,
            row_scroll_offset: new_scroll_offset,
            description,
            viewport_changed,
        }
    }

    /// Navigate one page up in the data
    pub fn page_up(&mut self, current_row: usize, _total_rows: usize) -> RowNavigationResult {
        // Calculate visible rows (viewport height)
        let visible_rows = self.terminal_height.saturating_sub(6) as usize; // Account for headers, borders, status

        debug!(target: "viewport_manager", 
               "page_up: current_row={}, visible_rows={}, current_viewport_rows={:?}", 
               current_row, visible_rows, self.viewport_rows);

        // Calculate new row position (move up by one page)
        let new_row = current_row.saturating_sub(visible_rows);

        // Calculate new scroll offset to keep new position visible
        let old_scroll_offset = self.viewport_rows.start;
        let new_scroll_offset = if new_row < self.viewport_rows.start {
            // Need to scroll up
            new_row
        } else {
            // Keep current scroll
            old_scroll_offset
        };

        // Update viewport
        self.viewport_rows = new_scroll_offset..(new_scroll_offset + visible_rows);
        let viewport_changed = new_scroll_offset != old_scroll_offset;

        let description = format!("Page up: row {} → {}", current_row + 1, new_row + 1);

        debug!(target: "viewport_manager", 
               "page_up result: new_row={}, scroll_offset={}→{}, viewport_changed={}", 
               new_row, old_scroll_offset, new_scroll_offset, viewport_changed);

        RowNavigationResult {
            row_position: new_row,
            row_scroll_offset: new_scroll_offset,
            description,
            viewport_changed,
        }
    }

    /// Navigate to the last row in the data (like vim 'G' command)
    pub fn navigate_to_last_row(&mut self, total_rows: usize) -> RowNavigationResult {
        if total_rows == 0 {
            return RowNavigationResult {
                row_position: 0,
                row_scroll_offset: 0,
                description: "No rows to navigate".to_string(),
                viewport_changed: false,
            };
        }

        // Get the actual visible rows from our current viewport
        let visible_rows = (self.terminal_height as usize).saturating_sub(3).max(10);

        // The last row index
        let last_row = total_rows - 1;

        // Calculate scroll offset to show the last row at the bottom of the viewport
        // We want the last row visible at the bottom, so start the viewport such that
        // the last row appears at the last position
        let new_scroll_offset = if total_rows > visible_rows {
            total_rows - visible_rows
        } else {
            0
        };

        debug!(target: "viewport_manager", 
               "navigate_to_last_row: total_rows={}, last_row={}, visible_rows={}, new_scroll_offset={}", 
               total_rows, last_row, visible_rows, new_scroll_offset);

        // Check if viewport actually changed
        let old_scroll_offset = self.viewport_rows.start;
        let viewport_changed = new_scroll_offset != old_scroll_offset;

        // Update viewport to show the last rows
        self.viewport_rows = new_scroll_offset..total_rows.min(new_scroll_offset + visible_rows);

        // Update crosshair to be at the last row
        // The crosshair position is the absolute row in the data
        self.crosshair_row = last_row;

        let description = format!("Jumped to last row ({}/{})", last_row + 1, total_rows);

        debug!(target: "viewport_manager", 
               "navigate_to_last_row result: row={}, crosshair_row={}, scroll_offset={}→{}, viewport_changed={}", 
               last_row, self.crosshair_row, old_scroll_offset, new_scroll_offset, viewport_changed);

        RowNavigationResult {
            row_position: last_row,
            row_scroll_offset: new_scroll_offset,
            description,
            viewport_changed,
        }
    }

    /// Navigate to the first row in the data (like vim 'gg' command)
    pub fn navigate_to_first_row(&mut self, total_rows: usize) -> RowNavigationResult {
        if total_rows == 0 {
            return RowNavigationResult {
                row_position: 0,
                row_scroll_offset: 0,
                description: "No rows to navigate".to_string(),
                viewport_changed: false,
            };
        }

        // Get the actual visible rows from our current viewport
        let visible_rows = (self.terminal_height as usize).saturating_sub(3).max(10);

        // First row is always 0
        let first_row = 0;

        // Scroll offset should be 0 to show the first row at the top
        let new_scroll_offset = 0;

        debug!(target: "viewport_manager", 
               "navigate_to_first_row: total_rows={}, visible_rows={}", 
               total_rows, visible_rows);

        // Check if viewport actually changed
        let old_scroll_offset = self.viewport_rows.start;
        let viewport_changed = new_scroll_offset != old_scroll_offset;

        // Update viewport to show the first rows
        self.viewport_rows = 0..visible_rows.min(total_rows);

        // Update crosshair to be at the first row
        self.crosshair_row = first_row;

        let description = format!("Jumped to first row (1/{})", total_rows);

        debug!(target: "viewport_manager", 
               "navigate_to_first_row result: row=0, crosshair_row={}, scroll_offset={}→0, viewport_changed={}", 
               self.crosshair_row, old_scroll_offset, viewport_changed);

        RowNavigationResult {
            row_position: first_row,
            row_scroll_offset: new_scroll_offset,
            description,
            viewport_changed,
        }
    }

    /// Navigate to the top of the current viewport (H in vim)
    pub fn navigate_to_viewport_top(&mut self) -> RowNavigationResult {
        let top_row = self.viewport_rows.start;
        let old_row = self.crosshair_row;

        // Move crosshair to top of viewport
        self.crosshair_row = top_row;

        let description = format!("Moved to viewport top (row {})", top_row + 1);

        debug!(target: "viewport_manager", 
               "navigate_to_viewport_top: crosshair {} -> {}", 
               old_row, self.crosshair_row);

        RowNavigationResult {
            row_position: self.crosshair_row,
            row_scroll_offset: self.viewport_rows.start,
            description,
            viewport_changed: false, // Viewport doesn't change, only crosshair moves
        }
    }

    /// Navigate to the middle of the current viewport (M in vim)
    pub fn navigate_to_viewport_middle(&mut self) -> RowNavigationResult {
        let viewport_height = self.viewport_rows.end - self.viewport_rows.start;
        let middle_offset = viewport_height / 2;
        let middle_row = self.viewport_rows.start + middle_offset;
        let old_row = self.crosshair_row;

        // Move crosshair to middle of viewport
        self.crosshair_row = middle_row;

        let description = format!("Moved to viewport middle (row {})", middle_row + 1);

        debug!(target: "viewport_manager", 
               "navigate_to_viewport_middle: crosshair {} -> {}", 
               old_row, self.crosshair_row);

        RowNavigationResult {
            row_position: self.crosshair_row,
            row_scroll_offset: self.viewport_rows.start,
            description,
            viewport_changed: false, // Viewport doesn't change, only crosshair moves
        }
    }

    /// Navigate to the bottom of the current viewport (L in vim)
    pub fn navigate_to_viewport_bottom(&mut self) -> RowNavigationResult {
        // Bottom row is the last visible row in the viewport
        let bottom_row = self.viewport_rows.end.saturating_sub(1);
        let old_row = self.crosshair_row;

        // Move crosshair to bottom of viewport
        self.crosshair_row = bottom_row;

        let description = format!("Moved to viewport bottom (row {})", bottom_row + 1);

        debug!(target: "viewport_manager", 
               "navigate_to_viewport_bottom: crosshair {} -> {}", 
               old_row, self.crosshair_row);

        RowNavigationResult {
            row_position: self.crosshair_row,
            row_scroll_offset: self.viewport_rows.start,
            description,
            viewport_changed: false, // Viewport doesn't change, only crosshair moves
        }
    }

    /// Move the current column left in the display order (swap with previous column)
    pub fn reorder_column_left(&mut self, current_column: usize) -> ColumnReorderResult {
        debug!(target: "viewport_manager",
            "reorder_column_left: current_column={}, viewport={:?}",
            current_column, self.viewport_cols
        );

        // Get the current column count
        let column_count = self.dataview.column_count();

        if current_column >= column_count {
            return ColumnReorderResult {
                new_column_position: current_column,
                description: "Invalid column position".to_string(),
                success: false,
            };
        }

        // Get pinned columns count to respect boundaries
        let pinned_count = self.dataview.get_pinned_columns().len();

        debug!(target: "viewport_manager",
            "Before move: column_count={}, pinned_count={}, current_column={}",
            column_count, pinned_count, current_column
        );

        // Delegate to DataView's move_column_left - it handles pinned column logic
        let dataview_mut = Arc::get_mut(&mut self.dataview)
            .expect("ViewportManager should have exclusive access to DataView during reordering");

        let success = dataview_mut.move_column_left(current_column);

        if success {
            self.invalidate_cache(); // Column order changed, need to recalculate widths

            // Determine new cursor position based on the move operation
            let wrapped_to_end =
                current_column == 0 || (current_column == pinned_count && pinned_count > 0);
            let new_position = if wrapped_to_end {
                // Column wrapped to end
                column_count - 1
            } else {
                // Normal swap with previous
                current_column - 1
            };

            let column_names = self.dataview.column_names();
            let column_name = column_names
                .get(new_position)
                .map(|s| s.as_str())
                .unwrap_or("?");

            debug!(target: "viewport_manager",
                "After move: new_position={}, wrapped_to_end={}, column_name={}",
                new_position, wrapped_to_end, column_name
            );

            // Adjust viewport to keep the moved column visible
            if wrapped_to_end {
                // Calculate optimal offset to show the last column
                let optimal_offset = self.calculate_optimal_offset_for_last_column(
                    self.terminal_width.saturating_sub(4),
                );
                debug!(target: "viewport_manager",
                    "Column wrapped to end! Adjusting viewport from {:?} to {}..{}",
                    self.viewport_cols, optimal_offset, self.dataview.column_count()
                );
                self.viewport_cols = optimal_offset..self.dataview.column_count();
            } else {
                // Check if the new position is outside the current viewport
                if !self.viewport_cols.contains(&new_position) {
                    // Column moved outside viewport, adjust to show it
                    let terminal_width = self.terminal_width.saturating_sub(4);

                    // Calculate how many columns we can fit starting from the new position
                    let columns_that_fit =
                        self.calculate_columns_that_fit(new_position, terminal_width);

                    // Adjust viewport to show the column at its new position
                    let new_start = if new_position < self.viewport_cols.start {
                        // Column moved to the left, scroll left
                        new_position
                    } else {
                        // Column moved to the right (shouldn't happen in move_left, but handle it)
                        new_position.saturating_sub(columns_that_fit - 1)
                    };

                    let new_end = (new_start + columns_that_fit).min(self.dataview.column_count());
                    self.viewport_cols = new_start..new_end;

                    debug!(target: "viewport_manager",
                        "Column moved outside viewport! Adjusting viewport to {}..{} to show column {} at position {}",
                        new_start, new_end, column_name, new_position
                    );
                }
            }

            ColumnReorderResult {
                new_column_position: new_position,
                description: format!("Moved column '{}' left", column_name),
                success: true,
            }
        } else {
            ColumnReorderResult {
                new_column_position: current_column,
                description: "Cannot move column left".to_string(),
                success: false,
            }
        }
    }

    /// Move the current column right in the display order (swap with next column)
    pub fn reorder_column_right(&mut self, current_column: usize) -> ColumnReorderResult {
        // Get the current column count
        let column_count = self.dataview.column_count();

        if current_column >= column_count {
            return ColumnReorderResult {
                new_column_position: current_column,
                description: "Invalid column position".to_string(),
                success: false,
            };
        }

        // Get pinned columns count to respect boundaries
        let pinned_count = self.dataview.get_pinned_columns().len();

        // Delegate to DataView's move_column_right - it handles pinned column logic
        let dataview_mut = Arc::get_mut(&mut self.dataview)
            .expect("ViewportManager should have exclusive access to DataView during reordering");

        let success = dataview_mut.move_column_right(current_column);

        if success {
            self.invalidate_cache(); // Column order changed, need to recalculate widths

            // Determine new cursor position and if wrapping occurred
            let wrapped_to_beginning = current_column == column_count - 1
                || (current_column == pinned_count - 1 && pinned_count > 0);

            let new_position = if current_column == column_count - 1 {
                // Column wrapped to beginning
                if pinned_count > 0 {
                    pinned_count // First non-pinned column
                } else {
                    0 // No pinned columns, go to start
                }
            } else if current_column == pinned_count - 1 && pinned_count > 0 {
                // Last pinned column wrapped to first pinned
                0
            } else {
                // Normal swap with next
                current_column + 1
            };

            let column_names = self.dataview.column_names();
            let column_name = column_names
                .get(new_position)
                .map(|s| s.as_str())
                .unwrap_or("?");

            // Adjust viewport to keep the moved column visible
            if wrapped_to_beginning {
                // Reset viewport to start
                self.viewport_cols = 0..self.dataview.column_count().min(20); // Show first ~20 columns or all if less
                debug!(target: "viewport_manager",
                    "Column wrapped to beginning, resetting viewport to show column {} at position {}",
                    column_name, new_position
                );
            } else {
                // Check if the new position is outside the current viewport
                if !self.viewport_cols.contains(&new_position) {
                    // Column moved outside viewport, adjust to show it
                    let terminal_width = self.terminal_width.saturating_sub(4);

                    // Calculate how many columns we can fit
                    let columns_that_fit =
                        self.calculate_columns_that_fit(new_position, terminal_width);

                    // Adjust viewport to show the column at its new position
                    let new_start = if new_position > self.viewport_cols.end {
                        // Column moved to the right, scroll right
                        new_position.saturating_sub(columns_that_fit - 1)
                    } else {
                        // Column moved to the left (shouldn't happen in move_right, but handle it)
                        new_position
                    };

                    let new_end = (new_start + columns_that_fit).min(self.dataview.column_count());
                    self.viewport_cols = new_start..new_end;

                    debug!(target: "viewport_manager",
                        "Column moved outside viewport! Adjusting viewport to {}..{} to show column {} at position {}",
                        new_start, new_end, column_name, new_position
                    );
                }
            }

            ColumnReorderResult {
                new_column_position: new_position,
                description: format!("Moved column '{}' right", column_name),
                success: true,
            }
        } else {
            ColumnReorderResult {
                new_column_position: current_column,
                description: "Cannot move column right".to_string(),
                success: false,
            }
        }
    }

    /// Hide the specified column
    /// Returns true if the column was hidden, false if it couldn't be hidden
    pub fn hide_column(&mut self, column_index: usize) -> bool {
        debug!(target: "viewport_manager", "hide_column: column_index={}", column_index);

        // Get mutable access to DataView
        let dataview_mut = Arc::get_mut(&mut self.dataview).expect(
            "ViewportManager should have exclusive access to DataView during column operations",
        );

        // Hide the column in DataView
        let success = dataview_mut.hide_column(column_index);

        if success {
            self.invalidate_cache(); // Column visibility changed, need to recalculate widths

            // Adjust viewport if necessary
            let column_count = self.dataview.column_count();
            if self.viewport_cols.end > column_count {
                self.viewport_cols.end = column_count;
            }
            if self.viewport_cols.start >= column_count && column_count > 0 {
                self.viewport_cols.start = column_count - 1;
            }

            debug!(target: "viewport_manager", "Column {} hidden successfully", column_index);
        } else {
            debug!(target: "viewport_manager", "Failed to hide column {} (might be pinned)", column_index);
        }

        success
    }

    /// Hide a column by name
    /// Returns true if the column was hidden, false if it couldn't be hidden
    pub fn hide_column_by_name(&mut self, column_name: &str) -> bool {
        debug!(target: "viewport_manager", "hide_column_by_name: column_name={}", column_name);

        // Get mutable access to DataView
        let dataview_mut = Arc::get_mut(&mut self.dataview).expect(
            "ViewportManager should have exclusive access to DataView during column operations",
        );

        // Hide the column in DataView
        let success = dataview_mut.hide_column_by_name(column_name);

        if success {
            self.invalidate_cache(); // Column visibility changed, need to recalculate widths

            // Adjust viewport if necessary
            let column_count = self.dataview.column_count();
            if self.viewport_cols.end > column_count {
                self.viewport_cols.end = column_count;
            }
            if self.viewport_cols.start >= column_count && column_count > 0 {
                self.viewport_cols.start = column_count - 1;
            }

            debug!(target: "viewport_manager", "Column '{}' hidden successfully", column_name);
        } else {
            debug!(target: "viewport_manager", "Failed to hide column '{}' (might be pinned or not found)", column_name);
        }

        success
    }

    /// Hide all empty columns
    /// Returns the number of columns hidden
    pub fn hide_empty_columns(&mut self) -> usize {
        debug!(target: "viewport_manager", "hide_empty_columns called");

        // Get mutable access to DataView
        let dataview_mut = Arc::get_mut(&mut self.dataview).expect(
            "ViewportManager should have exclusive access to DataView during column operations",
        );

        // Hide empty columns in DataView
        let count = dataview_mut.hide_empty_columns();

        if count > 0 {
            self.invalidate_cache(); // Column visibility changed, need to recalculate widths

            // Adjust viewport if necessary
            let column_count = self.dataview.column_count();
            if self.viewport_cols.end > column_count {
                self.viewport_cols.end = column_count;
            }
            if self.viewport_cols.start >= column_count && column_count > 0 {
                self.viewport_cols.start = column_count - 1;
            }

            debug!(target: "viewport_manager", "Hidden {} empty columns", count);
        }

        count
    }

    /// Unhide all columns
    pub fn unhide_all_columns(&mut self) {
        debug!(target: "viewport_manager", "unhide_all_columns called");

        // Get mutable access to DataView
        let dataview_mut = Arc::get_mut(&mut self.dataview).expect(
            "ViewportManager should have exclusive access to DataView during column operations",
        );

        // Unhide all columns in DataView
        dataview_mut.unhide_all_columns();

        self.invalidate_cache(); // Column visibility changed, need to recalculate widths

        // Reset viewport to show first columns
        let column_count = self.dataview.column_count();
        self.viewport_cols = 0..column_count.min(20); // Show first ~20 columns or all if less

        debug!(target: "viewport_manager", "All columns unhidden, viewport reset to {:?}", self.viewport_cols);
    }

    /// Update the current column position and automatically adjust viewport if needed
    /// This takes a VISUAL column index (0, 1, 2... in display order)
    pub fn set_current_column(&mut self, visual_column: usize) -> bool {
        let terminal_width = self.terminal_width.saturating_sub(4); // Account for borders
        let total_visual_columns = self.dataview.get_display_columns().len();

        debug!(target: "viewport_manager", 
               "set_current_column ENTRY: visual_column={}, current_viewport={:?}, terminal_width={}, total_visual={}", 
               visual_column, self.viewport_cols, terminal_width, total_visual_columns);

        // Validate the visual column
        if visual_column >= total_visual_columns {
            debug!(target: "viewport_manager", "Visual column {} out of bounds (max {})", visual_column, total_visual_columns);
            return false;
        }

        // Check if we're in optimal layout mode (all columns fit)
        // This needs to calculate based on visual columns
        let display_columns = self.dataview.get_display_columns();
        let mut total_width_needed = 0u16;
        for &dt_idx in &display_columns {
            let width = self
                .column_widths
                .get(dt_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);
            total_width_needed += width + 1; // +1 for separator
        }

        if total_width_needed <= terminal_width {
            // All columns fit - no viewport adjustment needed, all columns are visible
            debug!(target: "viewport_manager", 
                   "Visual column {} in optimal layout mode (all columns fit), no adjustment needed", visual_column);
            return false;
        }

        // Check if the visual column is already visible in the viewport
        let is_visible = self.viewport_cols.contains(&visual_column);

        debug!(target: "viewport_manager", 
               "set_current_column CHECK: visual_column={}, viewport={:?}, is_visible={}", 
               visual_column, self.viewport_cols, is_visible);

        if is_visible {
            debug!(target: "viewport_manager", "Visual column {} already visible in viewport {:?}, no adjustment needed", 
                   visual_column, self.viewport_cols);
            return false;
        }

        // Column is not visible, need to adjust viewport
        debug!(target: "viewport_manager", "Visual column {} NOT visible, calculating new offset", visual_column);
        let new_scroll_offset = self.calculate_scroll_offset_for_visual_column(visual_column);
        let old_scroll_offset = self.viewport_cols.start;

        debug!(target: "viewport_manager", "Calculated new_scroll_offset={}, old_scroll_offset={}", 
               new_scroll_offset, old_scroll_offset);

        if new_scroll_offset != old_scroll_offset {
            // Calculate how many columns fit from the new offset
            let display_columns = self.dataview.get_display_columns();
            let mut new_end = new_scroll_offset;
            let mut used_width = 0u16;
            let separator_width = 1u16;

            for visual_idx in new_scroll_offset..display_columns.len() {
                let dt_idx = display_columns[visual_idx];
                let width = self
                    .column_widths
                    .get(dt_idx)
                    .copied()
                    .unwrap_or(DEFAULT_COL_WIDTH);
                if used_width + width + separator_width <= terminal_width {
                    used_width += width + separator_width;
                    new_end = visual_idx + 1;
                } else {
                    break;
                }
            }

            self.viewport_cols = new_scroll_offset..new_end;
            self.cache_dirty = true; // Mark cache as dirty since viewport changed

            debug!(target: "viewport_manager", 
                   "Adjusted viewport for visual column {}: offset {}→{} (viewport: {:?})", 
                   visual_column, old_scroll_offset, new_scroll_offset, self.viewport_cols);

            return true;
        }

        false
    }

    /// Calculate visible columns with a specific scroll offset (for viewport tracking)
    /// Returns visual column indices that would be visible with the given offset
    fn calculate_visible_column_indices_with_offset(
        &mut self,
        available_width: u16,
        scroll_offset: usize,
    ) -> Vec<usize> {
        // Temporarily update viewport to calculate with new offset
        let original_viewport = self.viewport_cols.clone();
        let total_visual_columns = self.dataview.get_display_columns().len();
        self.viewport_cols = scroll_offset..(scroll_offset + 50).min(total_visual_columns);

        let visible_columns = self.calculate_visible_column_indices(available_width);

        // Restore original viewport
        self.viewport_cols = original_viewport;

        visible_columns
    }

    /// Calculate the optimal scroll offset to keep a visual column visible
    fn calculate_scroll_offset_for_visual_column(&mut self, visual_column: usize) -> usize {
        let current_offset = self.viewport_cols.start;
        let terminal_width = self.terminal_width.saturating_sub(4); // Account for borders

        // Calculate how many columns fit from current offset
        let display_columns = self.dataview.get_display_columns();
        let mut columns_that_fit = 0;
        let mut used_width = 0u16;
        let separator_width = 1u16;

        for visual_idx in current_offset..display_columns.len() {
            let dt_idx = display_columns[visual_idx];
            let width = self
                .column_widths
                .get(dt_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);
            if used_width + width + separator_width <= terminal_width {
                used_width += width + separator_width;
                columns_that_fit += 1;
            } else {
                break;
            }
        }

        // Smart scrolling logic in visual space
        if visual_column < current_offset {
            // Column is to the left of viewport, scroll left to show it
            visual_column
        } else if columns_that_fit > 0 && visual_column >= current_offset + columns_that_fit {
            // Column is to the right of viewport, scroll right to show it
            visual_column.saturating_sub(columns_that_fit - 1)
        } else {
            // Column is already visible, keep current offset
            current_offset
        }
    }
}

/// Viewport efficiency metrics
#[derive(Debug, Clone)]
pub struct ViewportEfficiency {
    pub available_width: u16,
    pub used_width: u16,
    pub wasted_space: u16,
    pub efficiency_percent: u8,
    pub visible_columns: usize,
    pub column_widths: Vec<u16>,
    pub next_column_width: Option<u16>, // Width of the next column that didn't fit
    pub columns_that_could_fit: Vec<(usize, u16)>, // Columns that could fit in wasted space
}

impl ViewportEfficiency {
    /// Format as a compact status line message
    pub fn to_status_string(&self) -> String {
        format!(
            "Viewport: {}w used of {}w ({}% efficient, {} cols, {}w wasted)",
            self.used_width,
            self.available_width,
            self.efficiency_percent,
            self.visible_columns,
            self.wasted_space
        )
    }

    /// Format as detailed debug info
    pub fn to_debug_string(&self) -> String {
        let avg_width = if !self.column_widths.is_empty() {
            self.column_widths.iter().sum::<u16>() / self.column_widths.len() as u16
        } else {
            0
        };

        // Show what efficiency we could get by fitting more columns
        let mut efficiency_analysis = String::new();
        if let Some(next_width) = self.next_column_width {
            efficiency_analysis.push_str(&format!(
                "\n\n  Next column needs: {}w (+1 separator)",
                next_width
            ));
            if next_width + 1 <= self.wasted_space {
                efficiency_analysis.push_str(" ✓ FITS!");
            } else {
                efficiency_analysis.push_str(&format!(" ✗ Too wide (have {}w)", self.wasted_space));
            }
        }

        if !self.columns_that_could_fit.is_empty() {
            efficiency_analysis.push_str(&format!(
                "\n  Columns that COULD fit in {}w:",
                self.wasted_space
            ));
            for (idx, width) in
                &self.columns_that_could_fit[..self.columns_that_could_fit.len().min(5)]
            {
                efficiency_analysis.push_str(&format!("\n    - Column {}: {}w", idx, width));
            }
            if self.columns_that_could_fit.len() > 5 {
                efficiency_analysis.push_str(&format!(
                    "\n    ... and {} more",
                    self.columns_that_could_fit.len() - 5
                ));
            }
        }

        // Calculate hypothetical efficiencies
        efficiency_analysis.push_str("\n\n  Hypothetical efficiencies:");
        for extra in 1..=3 {
            let hypothetical_used =
                self.used_width + (extra * (avg_width + 1)).min(self.wasted_space);
            let hypothetical_eff =
                ((hypothetical_used as f32 / self.available_width as f32) * 100.0) as u8;
            let hypothetical_wasted = self.available_width.saturating_sub(hypothetical_used);
            efficiency_analysis.push_str(&format!(
                "\n    +{} cols ({}w each): {}% efficiency, {}w wasted",
                extra, avg_width, hypothetical_eff, hypothetical_wasted
            ));
        }

        format!(
            "Viewport Efficiency:\n  Available: {}w\n  Used: {}w\n  Wasted: {}w\n  Efficiency: {}%\n  Columns: {} visible\n  Widths: {:?}\n  Avg Width: {}w{}",
            self.available_width,
            self.used_width,
            self.wasted_space,
            self.efficiency_percent,
            self.visible_columns,
            self.column_widths,
            avg_width,
            efficiency_analysis
        )
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
