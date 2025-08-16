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
}

impl ViewportManager {
    /// Get the current viewport range
    pub fn get_viewport_range(&self) -> std::ops::Range<usize> {
        self.viewport_cols.clone()
    }

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

    /// Update viewport size based on terminal dimensions
    /// Returns the calculated visible rows for the results area
    pub fn update_terminal_size(&mut self, terminal_width: u16, terminal_height: u16) -> usize {
        // Match the actual layout calculation:
        // - Input area: 3 rows
        // - Status bar: 3 rows
        // - Results area gets the rest
        let input_height = 3;
        let status_height = 3;
        let results_area_height =
            (terminal_height as usize).saturating_sub(input_height + status_height);

        // Now match EXACTLY what the render function does:
        // - 1 row for top border
        // - 1 row for header
        // - 1 row for bottom border
        let visible_rows = results_area_height.saturating_sub(3).max(10);

        // Update our stored terminal dimensions
        self.terminal_width = terminal_width;
        self.terminal_height = terminal_height;

        debug!(target: "navigation",
            "ViewportManager::update_terminal_size - terminal: {}x{}, visible_rows: {}",
            terminal_width, terminal_height, visible_rows
        );

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
    /// This is the key method that determines how many columns we can fit
    pub fn calculate_visible_column_indices(&mut self, available_width: u16) -> Vec<usize> {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }

        let mut visible_indices = Vec::new();
        let mut used_width = 0u16;
        let separator_width = 1u16; // Width of column separator

        tracing::trace!(
            "ViewportManager: Starting column layout calculation. Available width: {}w, viewport cols: {:?}, total cols: {}",
            available_width,
            self.viewport_cols,
            self.dataview.column_count()
        );

        // Check if all columns can fit by calculating total width
        let total_cols = self.dataview.column_count();
        let mut total_width_needed = 0u16;
        for col_idx in 0..total_cols {
            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);
            total_width_needed += width + separator_width;
        }

        // Determine the range of columns to process
        let process_range = if total_width_needed <= available_width {
            // All columns fit! Use optimal layout instead of restricted viewport
            tracing::trace!(
                "All columns fit ({}w needed, {}w available) - using optimal layout 0..{}",
                total_width_needed,
                available_width,
                total_cols
            );

            // Update viewport state to match optimal layout
            self.viewport_cols = 0..total_cols;
            tracing::trace!("Updated viewport_cols to optimal range: 0..{}", total_cols);

            0..total_cols
        } else {
            // Not all columns fit, use current viewport
            tracing::trace!(
                "Using viewport range {:?} = columns {} to {}",
                self.viewport_cols,
                self.viewport_cols.start,
                self.viewport_cols.end.saturating_sub(1)
            );
            self.viewport_cols.clone()
        };

        // First, add pinned columns
        let pinned = self.dataview.get_pinned_columns();
        for &col_idx in pinned {
            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);
            if used_width + width + separator_width <= available_width {
                visible_indices.push(col_idx);
                used_width += width + separator_width;
                tracing::trace!(
                    "Added pinned column {}: {}w (total used: {}w)",
                    col_idx,
                    width,
                    used_width
                );
            }
        }

        // Track which columns we've skipped due to width
        let mut skipped_columns: Vec<(usize, u16)> = Vec::new();

        // Then add regular columns from determined range
        for col_idx in process_range {
            // Skip if already added as pinned
            if pinned.contains(&col_idx) {
                continue;
            }

            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);

            if used_width + width + separator_width <= available_width {
                visible_indices.push(col_idx);
                used_width += width + separator_width;
                tracing::trace!(
                    "Added column {}: {}w (total used: {}w)",
                    col_idx,
                    width,
                    used_width
                );
            } else {
                // Track this skipped column
                skipped_columns.push((col_idx, width));
                tracing::trace!(
                    "Skipped column {} ({}w) - would exceed available width",
                    col_idx,
                    width
                );
            }
        }

        // Now check if we have significant unused space and can fit more columns
        let remaining_width = available_width.saturating_sub(used_width);

        tracing::trace!(
            "After initial pass: {}w used, {}w remaining. {} columns visible, {} skipped",
            used_width,
            remaining_width,
            visible_indices.len(),
            skipped_columns.len()
        );

        if remaining_width > MIN_COL_WIDTH + separator_width {
            // Look for ANY columns (not just from viewport) that could fit
            let mut candidates: Vec<(usize, u16)> = Vec::new();

            // First check skipped columns from viewport
            for &(col_idx, width) in &skipped_columns {
                if width + separator_width <= remaining_width {
                    candidates.push((col_idx, width));
                }
            }

            // DISABLED: Don't look beyond viewport for efficiency - it breaks column order
            // The optimization below would add column 42 when we should be showing columns in order
            // // Then check ALL columns beyond viewport that might fit
            // tracing::trace!(
            //     "Looking for additional columns from {} to {} (beyond viewport end)",
            //     self.viewport_cols.end,
            //     self.dataview.column_count() - 1
            // );
            // for col_idx in self.viewport_cols.end..self.dataview.column_count() {
            //     // Skip if already visible or pinned
            //     if visible_indices.contains(&col_idx) || pinned.contains(&col_idx) {
            //         tracing::trace!("Skipping column {} - already visible or pinned", col_idx);
            //         continue;
            //     }

            //     let width = self
            //         .column_widths
            //         .get(col_idx)
            //         .copied()
            //         .unwrap_or(DEFAULT_COL_WIDTH);

            //     if width + separator_width <= remaining_width {
            //         candidates.push((col_idx, width));
            //         tracing::trace!("Column {} ({}w) is candidate for extra space", col_idx, width);
            //     } else {
            //         tracing::trace!("Column {} ({}w) too wide for remaining {}w", col_idx, width, remaining_width);
            //     }
            // }

            if !candidates.is_empty() {
                tracing::trace!(
                    "Found {} candidate columns that could fit in {}w remaining space",
                    candidates.len(),
                    remaining_width
                );

                // Sort candidates by width (narrowest first) to maximize columns fitted
                candidates.sort_by_key(|&(_, width)| width);

                // Try to fit as many columns as possible
                let mut space_left = remaining_width;
                let mut added_columns = Vec::new();

                for (idx, width) in candidates {
                    if width + separator_width <= space_left {
                        visible_indices.push(idx);
                        used_width += width + separator_width;
                        space_left -= width + separator_width;
                        added_columns.push((idx, width));
                    }
                }

                if !added_columns.is_empty() {
                    tracing::trace!(
                        "Added {} extra columns to fill space: {:?}. Reduced waste from {}w to {}w",
                        added_columns.len(),
                        added_columns,
                        remaining_width,
                        space_left
                    );
                }
            } else {
                tracing::trace!(
                    "No columns found that could fit in {}w remaining space",
                    remaining_width
                );
            }
        }

        // Sort visible indices to maintain proper column order
        visible_indices.sort_unstable();

        tracing::trace!(
            "Final layout: {} columns visible {:?}, {}w used of {}w ({}% efficiency)",
            visible_indices.len(),
            visible_indices,
            used_width,
            available_width,
            (used_width as f32 / available_width as f32 * 100.0) as u8
        );

        visible_indices
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
    pub fn calculate_optimal_offset_for_last_column(&mut self, available_width: u16) -> usize {
        if self.cache_dirty {
            self.recalculate_column_widths();
        }

        let total_cols = self.dataview.column_count();
        if total_cols == 0 {
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

        // Start by including the last column
        let last_col_idx = total_cols - 1;
        let last_col_width = self
            .column_widths
            .get(last_col_idx)
            .copied()
            .unwrap_or(DEFAULT_COL_WIDTH);

        tracing::debug!(
            "Starting calculation: last_col_idx={}, width={}w, available={}w",
            last_col_idx,
            last_col_width,
            available_for_scrollable
        );

        let mut accumulated_width = last_col_width + separator_width;
        let mut best_offset = last_col_idx;

        // Now work backwards from the second-to-last column to find how many more we can fit
        for col_idx in (pinned_count..last_col_idx).rev() {
            let width = self
                .column_widths
                .get(col_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);

            let width_with_separator = width + separator_width;

            if accumulated_width + width_with_separator <= available_for_scrollable {
                // This column fits, keep going backwards
                accumulated_width += width_with_separator;
                best_offset = col_idx;
                tracing::trace!(
                    "Column {} fits ({}w), accumulated={}w, new offset={}",
                    col_idx,
                    width,
                    accumulated_width,
                    best_offset
                );
            } else {
                // This column doesn't fit, we found our optimal offset
                // The offset should be the next column (since this one doesn't fit)
                best_offset = col_idx + 1;
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

        // Ensure we don't go below the first scrollable column
        best_offset = best_offset.max(pinned_count);

        // Now verify that starting from best_offset, we can actually see the last column
        // This is the critical check we were missing!
        let mut test_width = 0u16;
        let mut can_see_last = false;
        for test_idx in best_offset..=last_col_idx {
            let width = self
                .column_widths
                .get(test_idx)
                .copied()
                .unwrap_or(DEFAULT_COL_WIDTH);
            test_width += width + separator_width;

            if test_width > available_for_scrollable {
                // We can't fit all columns from best_offset to last_col_idx
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
            if test_idx == last_col_idx {
                can_see_last = true;
            }
        }

        // If we still can't see the last column, keep adjusting
        while !can_see_last && best_offset < total_cols {
            test_width = 0;
            for test_idx in best_offset..=last_col_idx {
                let width = self
                    .column_widths
                    .get(test_idx)
                    .copied()
                    .unwrap_or(DEFAULT_COL_WIDTH);
                test_width += width + separator_width;

                if test_width > available_for_scrollable {
                    best_offset = best_offset + 1;
                    break;
                }
                if test_idx == last_col_idx {
                    can_see_last = true;
                }
            }
        }

        // Convert to scrollable column index (subtract pinned count)
        let scrollable_offset = best_offset.saturating_sub(pinned_count);

        tracing::debug!(
            "Final offset for last column: absolute={}, scrollable={}, fits {} columns, last col width: {}w, verified last col visible: {}",
            best_offset,
            scrollable_offset,
            total_cols - best_offset,
            last_col_width,
            can_see_last
        );

        scrollable_offset
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

    /// Navigate one column to the left with intelligent wrapping and scrolling
    /// This method handles everything: column movement, viewport tracking, and scrolling
    pub fn navigate_column_left(&mut self, current_column: usize) -> NavigationResult {
        // Get the DataView's display order (pinned columns first, then others)
        let display_columns = self.dataview.get_display_columns();
        let total_display_columns = display_columns.len();

        debug!(target: "viewport_manager", 
               "navigate_column_left: current={}, total_display={}, display_order={:?}", 
               current_column, total_display_columns, display_columns);

        // Find current column in the display order
        let current_display_index = display_columns
            .iter()
            .position(|&col_idx| col_idx == current_column)
            .unwrap_or(0);

        debug!(target: "viewport_manager", 
               "navigate_column_left: current_column={} found at display_index={}", 
               current_column, current_display_index);

        // Calculate new display position (move left in display order)
        let new_display_index = if current_display_index > 0 {
            current_display_index - 1
        } else {
            // Wrap to last column
            if total_display_columns > 0 {
                total_display_columns - 1
            } else {
                0
            }
        };

        // Get the actual column index from display order
        let new_column = display_columns
            .get(new_display_index)
            .copied()
            .unwrap_or(current_column);

        let old_scroll_offset = self.viewport_cols.start;

        // Don't pre-extend viewport - let set_current_column handle all viewport adjustments
        // This avoids the issue where we extend the viewport, then set_current_column thinks
        // the column is already visible and doesn't scroll
        debug!(target: "viewport_manager", 
               "navigate_column_left: moving to new_column={}, current viewport={:?}", 
               new_column, self.viewport_cols);

        // Use set_current_column to handle viewport adjustment automatically
        let viewport_changed = self.set_current_column(new_column);

        let column_names = self.dataview.column_names();
        let column_name = column_names
            .get(new_display_index)
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        let description = format!(
            "Navigate left to column '{}' ({})",
            column_name,
            new_display_index + 1
        );

        debug!(target: "viewport_manager", 
               "navigate_column_left: {}→{} (display_index: {}→{}), scroll: {}→{}, viewport_changed={}", 
               current_column, new_column, current_display_index, new_display_index,
               old_scroll_offset, self.viewport_cols.start, viewport_changed);

        NavigationResult {
            column_position: new_column,
            scroll_offset: self.viewport_cols.start,
            description,
            viewport_changed,
        }
    }

    /// Navigate one column to the right with intelligent wrapping and scrolling
    pub fn navigate_column_right(&mut self, current_column: usize) -> NavigationResult {
        // Get the DataView's display order (pinned columns first, then others)
        let display_columns = self.dataview.get_display_columns();
        let total_display_columns = display_columns.len();

        debug!(target: "viewport_manager", 
               "navigate_column_right ENTRY: current_col={}, display_columns={:?}, total={}", 
               current_column, display_columns, total_display_columns);

        // Find current column in the display order
        let current_display_index = display_columns
            .iter()
            .position(|&col_idx| col_idx == current_column)
            .unwrap_or(0);

        debug!(target: "viewport_manager", 
               "navigate_column_right: current_column={} maps to display_index={}", 
               current_column, current_display_index);

        // Calculate new display position (move right in display order)
        let new_display_index = if current_display_index + 1 < total_display_columns {
            current_display_index + 1
        } else {
            // Wrap to first column
            0
        };

        // Get the actual column index from display order
        let new_column = display_columns
            .get(new_display_index)
            .copied()
            .unwrap_or_else(|| {
                // Fallback: if something goes wrong, just wrap to first column
                display_columns.get(0).copied().unwrap_or(0)
            });

        let old_scroll_offset = self.viewport_cols.start;

        // Ensure the viewport includes the target column before checking visibility
        // This fixes the range issue where column N is not included in range start..N
        // Don't pre-extend viewport - let set_current_column handle all viewport adjustments
        // This avoids the issue where we extend the viewport, then set_current_column thinks
        // the column is already visible and doesn't scroll
        debug!(target: "viewport_manager", 
               "navigate_column_right: moving to new_column={}, current viewport={:?}", 
               new_column, self.viewport_cols);

        // Use set_current_column to handle viewport adjustment automatically
        let viewport_changed = self.set_current_column(new_column);

        let column_names = self.dataview.column_names();
        let column_name = column_names
            .get(new_display_index)
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        let description = format!(
            "Navigate right to column '{}' ({})",
            column_name,
            new_display_index + 1
        );

        debug!(target: "viewport_manager", 
               "navigate_column_right EXIT: {}→{} (display_index: {}→{}), viewport: {:?}, scroll: {}→{}, viewport_changed={}", 
               current_column, new_column, current_display_index, new_display_index,
               self.viewport_cols, old_scroll_offset, self.viewport_cols.start, viewport_changed);

        NavigationResult {
            column_position: new_column,
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

        // Calculate visible rows (viewport height)
        let visible_rows = self.terminal_height.saturating_sub(6) as usize; // Account for headers, borders, status

        // The last row index
        let last_row = total_rows - 1;

        // Calculate scroll offset to show the last row at the bottom of the viewport
        // We want the last row visible, so scroll to position it at the bottom
        let new_scroll_offset = last_row.saturating_sub(visible_rows - 1);

        debug!(target: "viewport_manager", 
               "navigate_to_last_row: total_rows={}, last_row={}, visible_rows={}, new_scroll_offset={}", 
               total_rows, last_row, visible_rows, new_scroll_offset);

        // Check if viewport actually changed
        let old_scroll_offset = self.viewport_rows.start;
        let viewport_changed = new_scroll_offset != old_scroll_offset;

        // Update viewport to show the last rows
        self.viewport_rows = new_scroll_offset..(new_scroll_offset + visible_rows).min(total_rows);

        let description = format!("Jumped to last row ({}/{})", last_row + 1, total_rows);

        debug!(target: "viewport_manager", 
               "navigate_to_last_row result: row={}, scroll_offset={}→{}, viewport_changed={}", 
               last_row, old_scroll_offset, new_scroll_offset, viewport_changed);

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

        // Calculate visible rows (viewport height)
        let visible_rows = self.terminal_height.saturating_sub(6) as usize; // Account for headers, borders, status

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

        let description = format!("Jumped to first row (1/{})", total_rows);

        debug!(target: "viewport_manager", 
               "navigate_to_first_row result: row=0, scroll_offset={}→0, viewport_changed={}", 
               old_scroll_offset, viewport_changed);

        RowNavigationResult {
            row_position: first_row,
            row_scroll_offset: new_scroll_offset,
            description,
            viewport_changed,
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
    /// This should be called whenever the user navigates to a new column
    pub fn set_current_column(&mut self, column: usize) -> bool {
        let pinned_count = self.dataview.get_pinned_columns().len();
        let terminal_width = self.terminal_width.saturating_sub(4); // Account for borders

        debug!(target: "viewport_manager", 
               "set_current_column: column={}, pinned_count={}, current_viewport={:?}", 
               column, pinned_count, self.viewport_cols);

        // Check if column is already visible
        if column < pinned_count {
            // Pinned column, always visible
            debug!(target: "viewport_manager", "Column {} is pinned, no viewport adjustment needed", column);
            return false;
        }

        let visible_columns = self.calculate_visible_column_indices(terminal_width);
        let is_visible = visible_columns.contains(&column);

        debug!(target: "viewport_manager", 
               "set_current_column: column={}, visible_columns={:?}, is_visible={}", 
               column, visible_columns, is_visible);

        if is_visible {
            debug!(target: "viewport_manager", "Column {} already visible in {:?}, no adjustment needed", column, self.viewport_cols);
            return false;
        }

        // Column is not visible, need to adjust viewport
        let new_scroll_offset = self.calculate_scroll_offset_for_column(column, pinned_count);
        let old_scroll_offset = self.viewport_cols.start;

        if new_scroll_offset != old_scroll_offset {
            // Update viewport to new position
            let visible_columns_at_offset = self
                .calculate_visible_column_indices_with_offset(terminal_width, new_scroll_offset);
            let new_end = if !visible_columns_at_offset.is_empty() {
                visible_columns_at_offset
                    .last()
                    .copied()
                    .unwrap_or(new_scroll_offset)
                    + 1
            } else {
                new_scroll_offset + 1
            };

            self.viewport_cols = new_scroll_offset..new_end;
            self.cache_dirty = true; // Mark cache as dirty since viewport changed

            debug!(target: "viewport_manager", 
                   "Adjusted viewport for column {}: {}→{} (viewport: {:?})", 
                   column, old_scroll_offset, new_scroll_offset, self.viewport_cols);

            return true;
        }

        false
    }

    /// Calculate visible columns with a specific scroll offset (for viewport tracking)
    fn calculate_visible_column_indices_with_offset(
        &mut self,
        available_width: u16,
        scroll_offset: usize,
    ) -> Vec<usize> {
        // Temporarily update viewport to calculate with new offset
        let original_viewport = self.viewport_cols.clone();
        self.viewport_cols = scroll_offset..scroll_offset + 50; // Temporary large range

        let visible_columns = self.calculate_visible_column_indices(available_width);

        // Restore original viewport
        self.viewport_cols = original_viewport;

        visible_columns
    }

    /// Calculate the optimal scroll offset to keep a column visible
    /// This replaces the hardcoded estimates in TUI with proper viewport calculations
    fn calculate_scroll_offset_for_column(&mut self, column: usize, pinned_count: usize) -> usize {
        if column < pinned_count {
            // Column is pinned, no scroll needed
            return 0;
        }

        let scrollable_column_index = column - pinned_count;
        let current_offset = self.viewport_cols.start;

        // Get actual visible column count from our viewport calculations
        let terminal_width = self.terminal_width.saturating_sub(4); // Account for borders
        let visible_columns = self.calculate_visible_column_indices(terminal_width);
        let visible_count = visible_columns.len();

        // Smart scrolling logic
        if scrollable_column_index < current_offset {
            // Column is to the left of viewport, scroll left to show it
            scrollable_column_index
        } else if visible_count > 0 && scrollable_column_index >= current_offset + visible_count {
            // Column is to the right of viewport, scroll right to show it
            scrollable_column_index.saturating_sub(visible_count - 1)
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
