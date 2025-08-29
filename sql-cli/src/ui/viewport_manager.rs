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
use crate::ui::viewport::column_width_calculator::{
    COLUMN_PADDING, DEFAULT_COL_WIDTH, MAX_COL_WIDTH, MAX_COL_WIDTH_DATA_FOCUS,
    MAX_HEADER_TO_DATA_RATIO, MIN_COL_WIDTH, MIN_HEADER_WIDTH_DATA_FOCUS,
};
use crate::ui::viewport::{ColumnPackingMode, ColumnWidthCalculator};

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

/// Unified result for all column operations
#[derive(Debug, Clone)]
pub struct ColumnOperationResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Human-readable description for status message
    pub description: String,
    /// Updated DataView after the operation (if changed)
    pub updated_dataview: Option<DataView>,
    /// New column position (for move/navigation operations)
    pub new_column_position: Option<usize>,
    /// New viewport range (if changed)
    pub new_viewport: Option<std::ops::Range<usize>>,
    /// Number of columns affected (for hide/unhide operations)
    pub affected_count: Option<usize>,
}

impl ColumnOperationResult {
    /// Create a failure result with a description
    pub fn failure(description: impl Into<String>) -> Self {
        Self {
            success: false,
            description: description.into(),
            updated_dataview: None,
            new_column_position: None,
            new_viewport: None,
            affected_count: None,
        }
    }

    /// Create a success result with a description
    pub fn success(description: impl Into<String>) -> Self {
        Self {
            success: true,
            description: description.into(),
            updated_dataview: None,
            new_column_position: None,
            new_viewport: None,
            affected_count: None,
        }
    }
}

/// Column packing mode for optimizing data display
/// Default column width if no data (used as fallback)

/// Number of rows used by the table widget chrome (header + borders)
/// This includes:
/// - 1 row for the header
/// - 1 row for the top border  
/// - 1 row for the bottom border
const TABLE_CHROME_ROWS: usize = 3;

/// Number of columns used by table borders (left + right + padding)
const TABLE_BORDER_WIDTH: u16 = 4;

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

    /// Column width calculator (extracted subsystem)
    width_calculator: ColumnWidthCalculator,

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

    /// Cursor lock state - when true, crosshair stays at same viewport position while scrolling
    cursor_lock: bool,
    /// The relative position of crosshair within viewport when locked (0 = top, viewport_height-1 = bottom)
    cursor_lock_position: Option<usize>,

    /// Viewport lock state - when true, prevents scrolling and constrains cursor to current viewport
    viewport_lock: bool,
    /// The viewport boundaries when locked (prevents scrolling beyond these)
    viewport_lock_boundaries: Option<std::ops::Range<usize>>,
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

    /// Get selected row (alias for crosshair_row for compatibility)
    pub fn get_selected_row(&self) -> usize {
        self.crosshair_row
    }

    /// Get selected column (alias for crosshair_col for compatibility)
    pub fn get_selected_column(&self) -> usize {
        self.crosshair_col
    }

    /// Get scroll offset as (row_offset, col_offset)
    pub fn get_scroll_offset(&self) -> (usize, usize) {
        (self.viewport_rows.start, self.viewport_cols.start)
    }

    /// Set scroll offset and update viewport accordingly
    pub fn set_scroll_offset(&mut self, row_offset: usize, col_offset: usize) {
        let viewport_height = self.viewport_rows.end - self.viewport_rows.start;
        let viewport_width = self.viewport_cols.end - self.viewport_cols.start;

        // Update viewport ranges based on new scroll offset
        self.viewport_rows = row_offset..(row_offset + viewport_height);
        self.viewport_cols = col_offset..(col_offset + viewport_width);

        // Ensure crosshair stays within new viewport
        if self.crosshair_row < self.viewport_rows.start {
            self.crosshair_row = self.viewport_rows.start;
        } else if self.crosshair_row >= self.viewport_rows.end {
            self.crosshair_row = self.viewport_rows.end.saturating_sub(1);
        }

        if self.crosshair_col < self.viewport_cols.start {
            self.crosshair_col = self.viewport_cols.start;
        } else if self.crosshair_col >= self.viewport_cols.end {
            self.crosshair_col = self.viewport_cols.end.saturating_sub(1);
        }

        self.cache_dirty = true;
    }

    /// Get crosshair position relative to current viewport for rendering
    /// Returns (row_offset, col_offset) within the viewport, or None if outside
    pub fn get_crosshair_viewport_position(&self) -> Option<(usize, usize)> {
        // Check if crosshair is within the current viewport
        // For rows, standard check
        if self.crosshair_row < self.viewport_rows.start
            || self.crosshair_row >= self.viewport_rows.end
        {
            return None;
        }

        // For columns, we need to account for pinned columns
        let pinned_count = self.dataview.get_pinned_columns().len();

        // If crosshair is on a pinned column, it's always visible
        if self.crosshair_col < pinned_count {
            return Some((
                self.crosshair_row - self.viewport_rows.start,
                self.crosshair_col, // Pinned columns are always at the start
            ));
        }

        // For scrollable columns, check if it's in the viewport
        // Convert visual column to scrollable column index
        let scrollable_col = self.crosshair_col - pinned_count;
        if scrollable_col >= self.viewport_cols.start && scrollable_col < self.viewport_cols.end {
            // Calculate the visual position in the rendered output
            // Pinned columns come first, then the visible scrollable columns
            let visual_col_in_viewport = pinned_count + (scrollable_col - self.viewport_cols.start);
            return Some((
                self.crosshair_row - self.viewport_rows.start,
                visual_col_in_viewport,
            ));
        }

        None
    }

    /// Navigate up one row
    pub fn navigate_row_up(&mut self) -> RowNavigationResult {
        let total_rows = self.dataview.row_count();

        // Check viewport lock first - prevent scrolling entirely
        if self.viewport_lock {
            debug!(target: "viewport_manager", 
                   "navigate_row_up: Viewport locked, crosshair={}, viewport={:?}",
                   self.crosshair_row, self.viewport_rows);
            // In viewport lock mode, just move cursor up within current viewport
            if self.crosshair_row > self.viewport_rows.start {
                self.crosshair_row -= 1;
                return RowNavigationResult {
                    row_position: self.crosshair_row,
                    row_scroll_offset: self.viewport_rows.start,
                    description: "Moved within locked viewport".to_string(),
                    viewport_changed: false,
                };
            } else {
                // Already at top of locked viewport
                return RowNavigationResult {
                    row_position: self.crosshair_row,
                    row_scroll_offset: self.viewport_rows.start,
                    description: "Moved within locked viewport".to_string(),
                    viewport_changed: false,
                };
            }
        }

        // Handle cursor lock mode
        if self.cursor_lock {
            if let Some(lock_position) = self.cursor_lock_position {
                // In cursor lock mode, scroll the viewport but keep crosshair at same relative position
                if self.viewport_rows.start == 0 {
                    // Can't scroll further up
                    return RowNavigationResult {
                        row_position: self.crosshair_row,
                        row_scroll_offset: self.viewport_rows.start,
                        description: "At top of data".to_string(),
                        viewport_changed: false,
                    };
                }

                let viewport_height = self.viewport_rows.end - self.viewport_rows.start;
                let new_viewport_start = self.viewport_rows.start.saturating_sub(1);

                // Update viewport
                self.viewport_rows =
                    new_viewport_start..(new_viewport_start + viewport_height).min(total_rows);

                // Update crosshair to maintain relative position
                self.crosshair_row = (self.viewport_rows.start + lock_position)
                    .min(self.viewport_rows.end.saturating_sub(1));

                return RowNavigationResult {
                    row_position: self.crosshair_row,
                    row_scroll_offset: self.viewport_rows.start,
                    description: format!(
                        "Scrolled up (locked at viewport row {})",
                        lock_position + 1
                    ),
                    viewport_changed: true,
                };
            }
        }

        // Normal navigation (not locked)
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

        // Check viewport lock first - prevent scrolling entirely
        if self.viewport_lock {
            debug!(target: "viewport_manager", 
                   "navigate_row_down: Viewport locked, crosshair={}, viewport={:?}",
                   self.crosshair_row, self.viewport_rows);
            // In viewport lock mode, just move cursor down within current viewport
            if self.crosshair_row < self.viewport_rows.end - 1
                && self.crosshair_row < total_rows - 1
            {
                self.crosshair_row += 1;
                return RowNavigationResult {
                    row_position: self.crosshair_row,
                    row_scroll_offset: self.viewport_rows.start,
                    description: "Moved within locked viewport".to_string(),
                    viewport_changed: false,
                };
            } else {
                // Already at bottom of locked viewport or end of data
                return RowNavigationResult {
                    row_position: self.crosshair_row,
                    row_scroll_offset: self.viewport_rows.start,
                    description: "Moved within locked viewport".to_string(),
                    viewport_changed: false,
                };
            }
        }

        // Handle cursor lock mode
        if self.cursor_lock {
            if let Some(lock_position) = self.cursor_lock_position {
                // In cursor lock mode, scroll the viewport but keep crosshair at same relative position
                let viewport_height = self.viewport_rows.end - self.viewport_rows.start;
                let new_viewport_start =
                    (self.viewport_rows.start + 1).min(total_rows.saturating_sub(viewport_height));

                if new_viewport_start == self.viewport_rows.start {
                    // Can't scroll further
                    return RowNavigationResult {
                        row_position: self.crosshair_row,
                        row_scroll_offset: self.viewport_rows.start,
                        description: "At bottom of data".to_string(),
                        viewport_changed: false,
                    };
                }

                // Update viewport
                self.viewport_rows =
                    new_viewport_start..(new_viewport_start + viewport_height).min(total_rows);

                // Update crosshair to maintain relative position
                self.crosshair_row = (self.viewport_rows.start + lock_position)
                    .min(self.viewport_rows.end.saturating_sub(1));

                return RowNavigationResult {
                    row_position: self.crosshair_row,
                    row_scroll_offset: self.viewport_rows.start,
                    description: format!(
                        "Scrolled down (locked at viewport row {})",
                        lock_position + 1
                    ),
                    viewport_changed: true,
                };
            }
        }

        // Normal navigation (not locked)
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
        // viewport_rows now correctly represents only data rows (no table chrome)
        let viewport_changed = if new_row >= self.viewport_rows.end {
            // Need to scroll - cursor is at or past the end of viewport
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
        // Start with a reasonable default that will be updated when terminal size is known
        let default_visible_rows = 50usize; // Start larger, will be adjusted by update_terminal_size
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
            width_calculator: ColumnWidthCalculator::new(),
            visible_row_cache: Vec::new(),
            cache_signature: 0,
            cache_dirty: true,
            crosshair_row: 0,
            crosshair_col: 0,
            cursor_lock: false,
            cursor_lock_position: None,
            viewport_lock: false,
            viewport_lock_boundaries: None,
        }
    }

    /// Update the underlying DataView
    pub fn set_dataview(&mut self, dataview: Arc<DataView>) {
        self.dataview = dataview;
        self.invalidate_cache();
    }

    /// Reset crosshair position to origin (0, 0)
    pub fn reset_crosshair(&mut self) {
        self.crosshair_row = 0;
        self.crosshair_col = 0;
        self.cursor_lock = false;
        self.cursor_lock_position = None;
    }

    /// Get the current column packing mode
    pub fn get_packing_mode(&self) -> ColumnPackingMode {
        self.width_calculator.get_packing_mode()
    }

    /// Set the column packing mode and recalculate widths
    pub fn set_packing_mode(&mut self, mode: ColumnPackingMode) {
        self.width_calculator.set_packing_mode(mode);
        self.invalidate_cache();
    }

    /// Cycle to the next packing mode
    pub fn cycle_packing_mode(&mut self) -> ColumnPackingMode {
        self.width_calculator.cycle_packing_mode();
        self.invalidate_cache();
        self.width_calculator.get_packing_mode()
    }

    /// Update viewport position and size
    pub fn set_viewport(&mut self, row_offset: usize, col_offset: usize, width: u16, height: u16) {
        let new_rows = row_offset
            ..row_offset
                .saturating_add(height as usize)
                .min(self.dataview.row_count());

        // For columns, we need to calculate how many columns actually fit in the width
        // Don't use width directly as column count - it's terminal width in characters!
        let display_columns = self.dataview.get_display_columns();
        let visual_column_count = display_columns.len();

        // Calculate how many columns we can actually fit in the available width
        let columns_that_fit = self.calculate_columns_that_fit(col_offset, width);
        let new_cols = col_offset
            ..col_offset
                .saturating_add(columns_that_fit)
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
        // The terminal_height passed here should already be the number of data rows available
        // The caller should have already accounted for any UI chrome
        let visible_rows = (terminal_height as usize).max(10);

        debug!(target: "viewport_manager",
            "update_terminal_size: terminal_height={}, calculated visible_rows={}",
            terminal_height, visible_rows
        );

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

        // Also update column viewport based on new terminal width
        // This is crucial for showing all columns that fit when first loading
        let visible_column_count = self.dataview.get_display_columns().len();
        if visible_column_count > 0 {
            // Calculate how many columns we can fit with the new terminal width
            // Calculate how many columns we can fit with the new terminal width
            // Subtract 2 for left and right table borders
            let columns_that_fit = self.calculate_columns_that_fit(
                self.viewport_cols.start,
                terminal_width.saturating_sub(2), // Left + right table borders
            );

            let new_col_viewport_end = self
                .viewport_cols
                .start
                .saturating_add(columns_that_fit)
                .min(visible_column_count);

            let old_col_viewport = self.viewport_cols.clone();
            self.viewport_cols = self.viewport_cols.start..new_col_viewport_end;

            if old_col_viewport != self.viewport_cols {
                debug!(target: "viewport_manager",
                    "update_terminal_size - column viewport changed from {:?} to {:?}, terminal_width={}",
                    old_col_viewport, self.viewport_cols, terminal_width
                );
                self.cache_dirty = true;
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
        self.width_calculator
            .get_all_column_widths(&self.dataview, &self.viewport_rows)
    }

    /// Get column width for a specific column
    pub fn get_column_width(&mut self, col_idx: usize) -> u16 {
        self.width_calculator
            .get_column_width(&self.dataview, &self.viewport_rows, col_idx)
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

    /// Get terminal width in characters
    pub fn get_terminal_width(&self) -> u16 {
        self.terminal_width
    }

    /// Get terminal height in rows
    pub fn get_terminal_height(&self) -> usize {
        self.terminal_height as usize
    }

    /// Force cache recalculation on next access
    pub fn invalidate_cache(&mut self) {
        self.cache_dirty = true;
        self.width_calculator.mark_dirty();
    }

    /// Calculate optimal column layout for available width
    /// Returns a RANGE of visual column indices (0..n) that should be displayed
    /// This works entirely in visual coordinate space - no DataTable indices!
    pub fn calculate_visible_column_indices(&mut self, available_width: u16) -> Vec<usize> {
        // Width calculation is now handled by ColumnWidthCalculator

        // Get the display columns from DataView (these are DataTable indices for visible columns)
        let display_columns = self.dataview.get_display_columns();
        let total_visual_columns = display_columns.len();

        if total_visual_columns == 0 {
            return Vec::new();
        }

        // Get pinned columns - they're always visible
        let pinned_columns = self.dataview.get_pinned_columns();
        let pinned_count = pinned_columns.len();

        let mut used_width = 0u16;
        let separator_width = 1u16;
        let mut result = Vec::new();

        tracing::debug!("[PIN_DEBUG] === calculate_visible_column_indices ===");
        tracing::debug!(
            "[PIN_DEBUG] available_width={}, total_visual_columns={}",
            available_width,
            total_visual_columns
        );
        tracing::debug!(
            "[PIN_DEBUG] pinned_columns={:?} (count={})",
            pinned_columns,
            pinned_count
        );
        tracing::debug!("[PIN_DEBUG] viewport_cols={:?}", self.viewport_cols);
        tracing::debug!("[PIN_DEBUG] display_columns={:?}", display_columns);

        debug!(target: "viewport_manager",
               "calculate_visible_column_indices: available_width={}, total_visual_columns={}, pinned_count={}, viewport_start={}",
               available_width, total_visual_columns, pinned_count, self.viewport_cols.start);

        // First, always add all pinned columns (they're at the beginning of display_columns)
        for visual_idx in 0..pinned_count {
            if visual_idx >= display_columns.len() {
                break;
            }

            let datatable_idx = display_columns[visual_idx];
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                datatable_idx,
            );

            // Always include pinned columns, even if they exceed available width
            used_width += width + separator_width;
            result.push(datatable_idx);
            tracing::debug!(
                "[PIN_DEBUG] Added pinned column: visual_idx={}, datatable_idx={}, width={}",
                visual_idx,
                datatable_idx,
                width
            );
        }

        // IMPORTANT FIX: viewport_cols represents SCROLLABLE column indices (0-based, excluding pinned)
        // To get the visual column index, we need to add pinned_count to the scrollable index
        let scrollable_start = self.viewport_cols.start;
        let visual_start = scrollable_start + pinned_count;

        tracing::debug!(
            "[PIN_DEBUG] viewport_cols.start={} is SCROLLABLE index",
            self.viewport_cols.start
        );
        tracing::debug!(
            "[PIN_DEBUG] visual_start={} (scrollable_start {} + pinned_count {})",
            visual_start,
            scrollable_start,
            pinned_count
        );

        let visual_start = visual_start.min(total_visual_columns);

        // Calculate how many columns we can fit from the viewport
        for visual_idx in visual_start..total_visual_columns {
            // Get the DataTable index for this visual position
            let datatable_idx = display_columns[visual_idx];

            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                datatable_idx,
            );

            if used_width + width + separator_width <= available_width {
                used_width += width + separator_width;
                result.push(datatable_idx);
                tracing::debug!("[PIN_DEBUG] Added scrollable column: visual_idx={}, datatable_idx={}, width={}", visual_idx, datatable_idx, width);
            } else {
                tracing::debug!(
                    "[PIN_DEBUG] Stopped at visual_idx={} - would exceed width",
                    visual_idx
                );
                break;
            }
        }

        // If we couldn't fit any scrollable columns but have pinned columns, that's okay
        // If we have no columns at all, ensure we show at least one column
        if result.is_empty() && total_visual_columns > 0 {
            result.push(display_columns[0]);
        }

        tracing::debug!("[PIN_DEBUG] Final result: {:?}", result);
        tracing::debug!("[PIN_DEBUG] === End calculate_visible_column_indices ===");
        debug!(target: "viewport_manager",
               "calculate_visible_column_indices RESULT: pinned={}, viewport_start={}, visual_start={} -> DataTable indices {:?}",
               pinned_count, self.viewport_cols.start, visual_start, result);

        result
    }

    /// Calculate how many columns we can fit starting from a given column index
    /// This helps determine optimal scrolling positions
    pub fn calculate_columns_that_fit(&mut self, start_col: usize, available_width: u16) -> usize {
        // Width calculation is now handled by ColumnWidthCalculator

        let mut used_width = 0u16;
        let mut column_count = 0usize;
        let separator_width = 1u16;

        for col_idx in start_col..self.dataview.column_count() {
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );
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
        column_indices
            .iter()
            .map(|&idx| {
                self.width_calculator
                    .get_column_width(&self.dataview, &self.viewport_rows, idx)
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
        // Width calculation is now handled by ColumnWidthCalculator

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
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );
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
        let last_col_width = self.width_calculator.get_column_width(
            &self.dataview,
            &self.viewport_rows,
            last_col_idx,
        );

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
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );

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
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );
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
                let width = self.width_calculator.get_column_width(
                    &self.dataview,
                    &self.viewport_rows,
                    col_idx,
                );
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
        // Width calculation is now handled by ColumnWidthCalculator

        let mut output = String::new();
        output.push_str("========== VIEWPORT MANAGER DEBUG ==========\n");

        let total_cols = self.dataview.column_count();
        let pinned = self.dataview.get_pinned_columns();
        let pinned_count = pinned.len();

        output.push_str(&format!("Total columns: {}\n", total_cols));
        output.push_str(&format!("Pinned columns: {:?}\n", pinned));
        output.push_str(&format!("Available width: {}w\n", available_width));
        output.push_str(&format!("Current viewport: {:?}\n", self.viewport_cols));
        output.push_str(&format!(
            "Packing mode: {} (Alt+S to cycle)\n",
            self.width_calculator.get_packing_mode().display_name()
        ));
        output.push_str("\n");

        // Show detailed column width calculations
        output.push_str("=== COLUMN WIDTH CALCULATIONS ===\n");
        output.push_str(&format!(
            "Mode: {}\n",
            self.width_calculator.get_packing_mode().display_name()
        ));

        // Show debug info for visible columns in viewport
        let debug_info = self.width_calculator.get_debug_info();
        if !debug_info.is_empty() {
            output.push_str("Visible columns in viewport:\n");

            // Only show columns that are currently visible
            let mut visible_count = 0;
            for col_idx in self.viewport_cols.clone() {
                if col_idx < debug_info.len() {
                    let (ref col_name, header_width, max_data_width, final_width, sample_count) =
                        debug_info[col_idx];

                    // Determine why this width was chosen
                    let reason = match self.width_calculator.get_packing_mode() {
                        ColumnPackingMode::DataFocus => {
                            if max_data_width <= 3 {
                                format!("Ultra aggressive (data:{}≤3 chars)", max_data_width)
                            } else if max_data_width <= 10 && header_width > max_data_width * 2 {
                                format!(
                                    "Aggressive truncate (data:{}≤10, header:{}>{} )",
                                    max_data_width,
                                    header_width,
                                    max_data_width * 2
                                )
                            } else if final_width == MAX_COL_WIDTH_DATA_FOCUS {
                                "Max width reached".to_string()
                            } else {
                                "Data-based width".to_string()
                            }
                        }
                        ColumnPackingMode::HeaderFocus => {
                            if final_width == header_width + COLUMN_PADDING {
                                "Full header shown".to_string()
                            } else if final_width == MAX_COL_WIDTH {
                                "Max width reached".to_string()
                            } else {
                                "Header priority".to_string()
                            }
                        }
                        ColumnPackingMode::Balanced => {
                            if header_width > max_data_width && final_width < header_width {
                                "Header constrained by ratio".to_string()
                            } else {
                                "Balanced".to_string()
                            }
                        }
                    };

                    output.push_str(&format!(
                        "  [{}] \"{}\":\n    Header: {}w, Data: {}w → Final: {}w ({}, {} samples)\n",
                        col_idx, col_name, header_width, max_data_width, final_width, reason, sample_count
                    ));

                    visible_count += 1;

                    // Stop after showing 10 columns to avoid clutter
                    if visible_count >= 10 {
                        let remaining = self.viewport_cols.end - self.viewport_cols.start - 10;
                        if remaining > 0 {
                            output.push_str(&format!("  ... and {} more columns\n", remaining));
                        }
                        break;
                    }
                }
            }
        }

        output.push_str("\n");

        // Show column widths summary
        output.push_str("Column width summary (all columns):\n");
        let all_widths = self
            .width_calculator
            .get_all_column_widths(&self.dataview, &self.viewport_rows);
        for (idx, &width) in all_widths.iter().enumerate() {
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
        let last_col_width = self.width_calculator.get_column_width(
            &self.dataview,
            &self.viewport_rows,
            last_col_idx,
        );

        // Calculate available width for scrollable columns
        let separator_width = 1u16;
        let mut pinned_width = 0u16;
        for &col_idx in pinned {
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );
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
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );
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
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                test_idx,
            );
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
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );
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
        // Width calculation is now handled by ColumnWidthCalculator

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
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );

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
        // IMPORTANT: Use calculate_visible_column_indices to get the correct columns
        // This properly handles pinned columns that should always be visible
        let visible_column_indices = self.calculate_visible_column_indices(available_width);

        tracing::debug!(
            "[RENDER_DEBUG] visible_column_indices from calculate: {:?}",
            visible_column_indices
        );

        // Get ALL visual columns from DataView (already filtered for hidden columns)
        let all_headers = self.dataview.get_display_column_names();
        let display_columns = self.dataview.get_display_columns();
        let total_visual_columns = all_headers.len();

        debug!(target: "viewport_manager",
               "get_visual_display: {} total visual columns, viewport: {:?}",
               total_visual_columns, self.viewport_cols);

        // Build headers from the visible column indices (DataTable indices)
        let headers: Vec<String> = visible_column_indices
            .iter()
            .filter_map(|&dt_idx| {
                // Find the visual position for this DataTable index
                display_columns
                    .iter()
                    .position(|&x| x == dt_idx)
                    .and_then(|visual_idx| all_headers.get(visual_idx).cloned())
            })
            .collect();

        tracing::debug!("[RENDER_DEBUG] headers: {:?}", headers);

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
                    // Extract the columns we need based on visible_column_indices
                    visible_column_indices
                        .iter()
                        .filter_map(|&dt_idx| {
                            // Find the visual position for this DataTable index
                            display_columns
                                .iter()
                                .position(|&x| x == dt_idx)
                                .and_then(|visual_idx| full_row.get(visual_idx).cloned())
                        })
                        .collect()
                })
            })
            .collect();

        // Get the actual calculated widths for the visible columns
        let widths: Vec<u16> = visible_column_indices
            .iter()
            .map(|&dt_idx| {
                Some(self.width_calculator.get_column_width(
                    &self.dataview,
                    &self.viewport_rows,
                    dt_idx,
                ))
                .unwrap_or(DEFAULT_COL_WIDTH)
            })
            .collect();

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

        for &visual_idx in visible_indices {
            if visual_idx < all_column_names.len() {
                headers.push(all_column_names[visual_idx].clone());
            } else {
                // Fallback for invalid indices
                headers.push(format!("Column_{}", visual_idx));
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
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );
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
            if last_visible + 1 < self.dataview.column_count() {
                Some(self.width_calculator.get_column_width(
                    &self.dataview,
                    &self.viewport_rows,
                    last_visible + 1,
                ))
            } else {
                None
            }
        } else {
            None
        };

        // Find ALL columns that COULD fit in the wasted space
        let mut columns_that_could_fit = Vec::new();
        if wasted_space > MIN_COL_WIDTH + separator_width {
            let all_widths = self
                .width_calculator
                .get_all_column_widths(&self.dataview, &self.viewport_rows);
            for (idx, &width) in all_widths.iter().enumerate() {
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
                    Some(self.width_calculator.get_column_width(
                        &self.dataview,
                        &self.viewport_rows,
                        idx,
                    ))
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
        // Check viewport lock - prevent scrolling
        if self.viewport_lock {
            // In viewport lock mode, just move to leftmost visible column
            self.crosshair_col = self.viewport_cols.start;
            return NavigationResult {
                column_position: self.crosshair_col,
                scroll_offset: self.viewport_cols.start,
                description: "Moved to first visible column (viewport locked)".to_string(),
                viewport_changed: false,
            };
        }
        // Get pinned column count from dataview
        let pinned_count = self.dataview.get_pinned_columns().len();
        let pinned_names = self.dataview.get_pinned_column_names();

        // First scrollable column is at index = pinned_count
        let first_scrollable_column = pinned_count;

        // Reset viewport to beginning (scroll offset = 0)
        let new_scroll_offset = 0;
        let old_scroll_offset = self.viewport_cols.start;

        // Recalculate the entire viewport to show columns starting from new_scroll_offset
        let visible_indices = self
            .calculate_visible_column_indices_with_offset(self.terminal_width, new_scroll_offset);
        let viewport_end = if let Some(&last_idx) = visible_indices.last() {
            last_idx + 1
        } else {
            new_scroll_offset + 1
        };

        // Update our internal viewport state
        self.viewport_cols = new_scroll_offset..viewport_end;

        // Update crosshair to first scrollable column
        self.crosshair_col = first_scrollable_column;

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
               "navigate_to_first_column: pinned={}, first_scrollable={}, crosshair_col={}, scroll_offset={}->{}",
               pinned_count, first_scrollable_column, self.crosshair_col, old_scroll_offset, new_scroll_offset);

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
        // Check viewport lock - prevent scrolling
        if self.viewport_lock {
            // In viewport lock mode, just move to rightmost visible column
            self.crosshair_col = self.viewport_cols.end.saturating_sub(1);
            return NavigationResult {
                column_position: self.crosshair_col,
                scroll_offset: self.viewport_cols.start,
                description: "Moved to last visible column (viewport locked)".to_string(),
                viewport_changed: false,
            };
        }
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
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );
            pinned_width += width + 3; // separator width
        }

        let available_for_scrollable = available_width.saturating_sub(pinned_width);

        // Calculate the optimal scroll offset to show the last column
        let mut accumulated_width = 0u16;
        let mut new_scroll_offset = last_visual_column;

        // Work backwards from the last column to find the best scroll position
        for visual_idx in (pinned_count..=last_visual_column).rev() {
            let col_idx = display_columns[visual_idx];
            let width = self.width_calculator.get_column_width(
                &self.dataview,
                &self.viewport_rows,
                col_idx,
            );
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

        // Recalculate the entire viewport to show columns starting from new_scroll_offset
        let visible_indices = self
            .calculate_visible_column_indices_with_offset(self.terminal_width, new_scroll_offset);
        let viewport_end = if let Some(&last_idx) = visible_indices.last() {
            last_idx + 1
        } else {
            new_scroll_offset + 1
        };

        // Update our internal viewport state
        self.viewport_cols = new_scroll_offset..viewport_end;

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
        // Check viewport lock first - prevent scrolling entirely
        if self.viewport_lock {
            debug!(target: "viewport_manager", 
                   "navigate_column_left: Viewport locked, crosshair_col={}, viewport={:?}",
                   self.crosshair_col, self.viewport_cols);

            // In viewport lock mode, just move cursor left within current viewport
            if self.crosshair_col > self.viewport_cols.start {
                self.crosshair_col -= 1;
                return NavigationResult {
                    column_position: self.crosshair_col,
                    scroll_offset: self.viewport_cols.start,
                    description: "Moved within locked viewport".to_string(),
                    viewport_changed: false,
                };
            } else {
                // Already at left edge of locked viewport
                return NavigationResult {
                    column_position: self.crosshair_col,
                    scroll_offset: self.viewport_cols.start,
                    description: "At left edge of locked viewport".to_string(),
                    viewport_changed: false,
                };
            }
        }

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
            // Already at first column, return visual index 0
            return NavigationResult {
                column_position: 0, // Visual position, not DataTable index
                scroll_offset: self.viewport_cols.start,
                description: "Already at first column".to_string(),
                viewport_changed: false,
            };
        }

        let new_display_index = current_display_index - 1;

        // Get the actual DataTable column index from display order for internal operations
        let new_visual_column = display_columns
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
               new_visual_column, self.viewport_cols);

        // Use set_current_column to handle viewport adjustment automatically (this takes DataTable index)
        let viewport_changed = self.set_current_column(new_display_index);

        // crosshair_col is already updated by set_current_column, no need to set it again

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
               current_display_index, new_display_index, new_visual_column,
               old_scroll_offset, self.viewport_cols.start, viewport_changed);

        NavigationResult {
            column_position: new_display_index, // Return visual/display index
            scroll_offset: self.viewport_cols.start,
            description,
            viewport_changed,
        }
    }

    /// Navigate one column to the right with intelligent wrapping and scrolling
    /// IMPORTANT: current_display_position is a logical display position (0,1,2,3...), NOT a DataTable index
    pub fn navigate_column_right(&mut self, current_display_position: usize) -> NavigationResult {
        debug!(target: "viewport_manager",
               "=== CRITICAL DEBUG: navigate_column_right CALLED ===");
        debug!(target: "viewport_manager",
               "Input current_display_position: {}", current_display_position);
        debug!(target: "viewport_manager",
               "Current crosshair_col: {}", self.crosshair_col);
        debug!(target: "viewport_manager",
               "Current viewport_cols: {:?}", self.viewport_cols);
        // Check viewport lock first - prevent scrolling entirely
        if self.viewport_lock {
            debug!(target: "viewport_manager", 
                   "navigate_column_right: Viewport locked, crosshair_col={}, viewport={:?}",
                   self.crosshair_col, self.viewport_cols);

            // In viewport lock mode, just move cursor right within current viewport
            if self.crosshair_col < self.viewport_cols.end - 1 {
                self.crosshair_col += 1;
                return NavigationResult {
                    column_position: self.crosshair_col,
                    scroll_offset: self.viewport_cols.start,
                    description: "Moved within locked viewport".to_string(),
                    viewport_changed: false,
                };
            } else {
                // Already at right edge of locked viewport
                return NavigationResult {
                    column_position: self.crosshair_col,
                    scroll_offset: self.viewport_cols.start,
                    description: "At right edge of locked viewport".to_string(),
                    viewport_changed: false,
                };
            }
        }

        let display_columns = self.dataview.get_display_columns();
        let total_display_columns = display_columns.len();
        let column_names = self.dataview.column_names();

        // Enhanced logging to debug the external_id issue
        debug!(target: "viewport_manager", 
               "=== navigate_column_right DETAILED DEBUG ===");
        debug!(target: "viewport_manager", 
               "ENTRY: current_display_pos={}, total_display_columns={}", 
               current_display_position, total_display_columns);
        debug!(target: "viewport_manager",
               "display_columns (DataTable indices): {:?}", display_columns);

        // Log column names at each position
        if current_display_position < display_columns.len() {
            let current_dt_idx = display_columns[current_display_position];
            let current_name = column_names
                .get(current_dt_idx)
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            debug!(target: "viewport_manager",
                   "Current position {} -> column '{}' (dt_idx={})", 
                   current_display_position, current_name, current_dt_idx);
        }

        if current_display_position + 1 < display_columns.len() {
            let next_dt_idx = display_columns[current_display_position + 1];
            let next_name = column_names
                .get(next_dt_idx)
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            debug!(target: "viewport_manager",
                   "Next position {} -> column '{}' (dt_idx={})", 
                   current_display_position + 1, next_name, next_dt_idx);
        }

        // Validate current position
        let current_display_index = if current_display_position < total_display_columns {
            current_display_position
        } else {
            debug!(target: "viewport_manager",
                   "WARNING: current_display_position {} >= total_display_columns {}, resetting to 0",
                   current_display_position, total_display_columns);
            0 // Reset to first if out of bounds
        };

        debug!(target: "viewport_manager", 
               "Validated: current_display_index={}", 
               current_display_index);

        // Calculate new display position (move right without wrapping)
        // Vim-like behavior: don't wrap, stay at boundary
        if current_display_index + 1 >= total_display_columns {
            // Already at last column, don't move
            let last_display_index = total_display_columns.saturating_sub(1);
            debug!(target: "viewport_manager",
                   "At last column boundary: current={}, total={}, returning last_display_index={}",
                   current_display_index, total_display_columns, last_display_index);
            return NavigationResult {
                column_position: last_display_index, // Return visual/display index
                scroll_offset: self.viewport_cols.start,
                description: "Already at last column".to_string(),
                viewport_changed: false,
            };
        }

        let new_display_index = current_display_index + 1;

        // Get the actual DataTable column index for the new position (for internal operations)
        let new_visual_column = display_columns
            .get(new_display_index)
            .copied()
            .unwrap_or_else(|| {
                // This fallback should never be hit since we already checked bounds
                tracing::error!(
                    "[NAV_ERROR] Failed to get display column at index {}, total={}",
                    new_display_index,
                    display_columns.len()
                );
                // Return the current column instead of wrapping to first
                display_columns
                    .get(current_display_index)
                    .copied()
                    .unwrap_or(0)
            });

        debug!(target: "viewport_manager", 
               "navigate_column_right: display_pos {}→{}, new_visual_column={}",
               current_display_index, new_display_index, new_visual_column);

        let old_scroll_offset = self.viewport_cols.start;

        // Ensure the viewport includes the target column before checking visibility
        // This fixes the range issue where column N is not included in range start..N
        // Don't pre-extend viewport - let set_current_column handle all viewport adjustments
        // This avoids the issue where we extend the viewport, then set_current_column thinks
        // the column is already visible and doesn't scroll
        debug!(target: "viewport_manager", 
               "navigate_column_right: moving to datatable_column={}, current viewport={:?}", 
               new_visual_column, self.viewport_cols);

        // Use set_current_column to handle viewport adjustment automatically
        // IMPORTANT: set_current_column expects a VISUAL index, and we're passing new_display_index which IS a visual index
        debug!(target: "viewport_manager", 
               "navigate_column_right: before set_current_column(visual_idx={}), viewport={:?}", 
               new_display_index, self.viewport_cols);
        let viewport_changed = self.set_current_column(new_display_index);
        debug!(target: "viewport_manager", 
               "navigate_column_right: after set_current_column(visual_idx={}), viewport={:?}, changed={}", 
               new_display_index, self.viewport_cols, viewport_changed);

        // crosshair_col is already updated by set_current_column, no need to set it again

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

        // Final logging with clear indication of what we're returning
        debug!(target: "viewport_manager", 
               "=== navigate_column_right RESULT ===");
        debug!(target: "viewport_manager",
               "Returning: column_position={} (visual/display index)", new_display_index);
        debug!(target: "viewport_manager",
               "Movement: {} -> {} (visual indices)", current_display_index, new_display_index);
        debug!(target: "viewport_manager",
               "Viewport: {:?}, changed={}", self.viewport_cols, viewport_changed);
        debug!(target: "viewport_manager",
               "Description: {}", description);

        tracing::debug!("[NAV_DEBUG] Final result: column_position={} (visual/display idx), viewport_changed={}", 
                       new_display_index, viewport_changed);
        debug!(target: "viewport_manager", 
               "navigate_column_right EXIT: display_pos {}→{}, datatable_col: {}, viewport: {:?}, scroll: {}→{}, viewport_changed={}", 
               current_display_index, new_display_index, new_visual_column,
               self.viewport_cols, old_scroll_offset, self.viewport_cols.start, viewport_changed);

        NavigationResult {
            column_position: new_display_index, // Return visual/display index
            scroll_offset: self.viewport_cols.start,
            description,
            viewport_changed,
        }
    }

    /// Navigate one page down in the data
    pub fn page_down(&mut self) -> RowNavigationResult {
        let total_rows = self.dataview.row_count();
        // Calculate visible rows (viewport height)
        let visible_rows = self.terminal_height.saturating_sub(6) as usize; // Account for headers, borders, status

        debug!(target: "viewport_manager", 
               "page_down: crosshair_row={}, total_rows={}, visible_rows={}, current_viewport_rows={:?}", 
               self.crosshair_row, total_rows, visible_rows, self.viewport_rows);

        // Check viewport lock first - prevent scrolling entirely
        if self.viewport_lock {
            debug!(target: "viewport_manager", 
                   "page_down: Viewport locked, moving within current viewport");
            // In viewport lock mode, move to bottom of current viewport
            let new_row = self
                .viewport_rows
                .end
                .saturating_sub(1)
                .min(total_rows.saturating_sub(1));
            self.crosshair_row = new_row;
            return RowNavigationResult {
                row_position: new_row,
                row_scroll_offset: self.viewport_rows.start,
                description: format!(
                    "Page down within locked viewport: row {} → {}",
                    self.crosshair_row + 1,
                    new_row + 1
                ),
                viewport_changed: false,
            };
        }

        // Check cursor lock - scroll viewport but keep cursor at same relative position
        if self.cursor_lock {
            if let Some(lock_position) = self.cursor_lock_position {
                debug!(target: "viewport_manager", 
                       "page_down: Cursor locked at position {}", lock_position);

                // Calculate new viewport position
                let old_scroll_offset = self.viewport_rows.start;
                let max_scroll = total_rows.saturating_sub(visible_rows);
                let new_scroll_offset = (old_scroll_offset + visible_rows).min(max_scroll);

                if new_scroll_offset == old_scroll_offset {
                    // Can't scroll further
                    return RowNavigationResult {
                        row_position: self.crosshair_row,
                        row_scroll_offset: old_scroll_offset,
                        description: "Already at bottom".to_string(),
                        viewport_changed: false,
                    };
                }

                // Update viewport
                self.viewport_rows =
                    new_scroll_offset..(new_scroll_offset + visible_rows).min(total_rows);

                // Keep crosshair at same relative position
                self.crosshair_row =
                    (new_scroll_offset + lock_position).min(total_rows.saturating_sub(1));

                return RowNavigationResult {
                    row_position: self.crosshair_row,
                    row_scroll_offset: new_scroll_offset,
                    description: format!(
                        "Page down with cursor lock (viewport {} → {})",
                        old_scroll_offset + 1,
                        new_scroll_offset + 1
                    ),
                    viewport_changed: true,
                };
            }
        }

        // Normal page down behavior
        // Calculate new row position (move down by one page) using ViewportManager's crosshair
        let new_row = (self.crosshair_row + visible_rows).min(total_rows.saturating_sub(1));
        self.crosshair_row = new_row;

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
            self.crosshair_row + 1,
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
    pub fn page_up(&mut self) -> RowNavigationResult {
        let total_rows = self.dataview.row_count();
        // Calculate visible rows (viewport height)
        let visible_rows = self.terminal_height.saturating_sub(6) as usize; // Account for headers, borders, status

        debug!(target: "viewport_manager", 
               "page_up: crosshair_row={}, visible_rows={}, current_viewport_rows={:?}", 
               self.crosshair_row, visible_rows, self.viewport_rows);

        // Check viewport lock first - prevent scrolling entirely
        if self.viewport_lock {
            debug!(target: "viewport_manager", 
                   "page_up: Viewport locked, moving within current viewport");
            // In viewport lock mode, move to top of current viewport
            let new_row = self.viewport_rows.start;
            self.crosshair_row = new_row;
            return RowNavigationResult {
                row_position: new_row,
                row_scroll_offset: self.viewport_rows.start,
                description: format!(
                    "Page up within locked viewport: row {} → {}",
                    self.crosshair_row + 1,
                    new_row + 1
                ),
                viewport_changed: false,
            };
        }

        // Check cursor lock - scroll viewport but keep cursor at same relative position
        if self.cursor_lock {
            if let Some(lock_position) = self.cursor_lock_position {
                debug!(target: "viewport_manager", 
                       "page_up: Cursor locked at position {}", lock_position);

                // Calculate new viewport position
                let old_scroll_offset = self.viewport_rows.start;
                let new_scroll_offset = old_scroll_offset.saturating_sub(visible_rows);

                if new_scroll_offset == old_scroll_offset {
                    // Can't scroll further
                    return RowNavigationResult {
                        row_position: self.crosshair_row,
                        row_scroll_offset: old_scroll_offset,
                        description: "Already at top".to_string(),
                        viewport_changed: false,
                    };
                }

                // Update viewport
                self.viewport_rows =
                    new_scroll_offset..(new_scroll_offset + visible_rows).min(total_rows);

                // Keep crosshair at same relative position
                self.crosshair_row = new_scroll_offset + lock_position;

                return RowNavigationResult {
                    row_position: self.crosshair_row,
                    row_scroll_offset: new_scroll_offset,
                    description: format!(
                        "Page up with cursor lock (viewport {} → {})",
                        old_scroll_offset + 1,
                        new_scroll_offset + 1
                    ),
                    viewport_changed: true,
                };
            }
        }

        // Normal page up behavior
        // Calculate new row position (move up by one page) using ViewportManager's crosshair
        let new_row = self.crosshair_row.saturating_sub(visible_rows);
        self.crosshair_row = new_row;

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
        self.viewport_rows = new_scroll_offset..(new_scroll_offset + visible_rows).min(total_rows);
        let viewport_changed = new_scroll_offset != old_scroll_offset;

        let description = format!("Page up: row {} → {}", self.crosshair_row + 1, new_row + 1);

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

    /// Navigate half page down in the data
    pub fn half_page_down(&mut self) -> RowNavigationResult {
        let total_rows = self.dataview.row_count();
        // Calculate visible rows (viewport height)
        let visible_rows = self.terminal_height.saturating_sub(6) as usize; // Account for headers, borders, status
        let half_page = visible_rows / 2;

        debug!(target: "viewport_manager", 
               "half_page_down: crosshair_row={}, total_rows={}, half_page={}, current_viewport_rows={:?}", 
               self.crosshair_row, total_rows, half_page, self.viewport_rows);

        // Check viewport lock first - prevent scrolling entirely
        if self.viewport_lock {
            debug!(target: "viewport_manager", 
                   "half_page_down: Viewport locked, moving within current viewport");
            // In viewport lock mode, move to bottom of current viewport
            let new_row = self
                .viewport_rows
                .end
                .saturating_sub(1)
                .min(total_rows.saturating_sub(1));
            self.crosshair_row = new_row;
            return RowNavigationResult {
                row_position: new_row,
                row_scroll_offset: self.viewport_rows.start,
                description: format!(
                    "Half page down within locked viewport: row {} → {}",
                    self.crosshair_row + 1,
                    new_row + 1
                ),
                viewport_changed: false,
            };
        }

        // Check cursor lock - scroll viewport but keep cursor at same relative position
        if self.cursor_lock {
            if let Some(lock_position) = self.cursor_lock_position {
                debug!(target: "viewport_manager", 
                       "half_page_down: Cursor locked at position {}", lock_position);

                // Calculate new viewport position
                let old_scroll_offset = self.viewport_rows.start;
                let max_scroll = total_rows.saturating_sub(visible_rows);
                let new_scroll_offset = (old_scroll_offset + half_page).min(max_scroll);

                if new_scroll_offset == old_scroll_offset {
                    // Can't scroll further
                    return RowNavigationResult {
                        row_position: self.crosshair_row,
                        row_scroll_offset: old_scroll_offset,
                        description: "Already at bottom".to_string(),
                        viewport_changed: false,
                    };
                }

                // Update viewport
                self.viewport_rows =
                    new_scroll_offset..(new_scroll_offset + visible_rows).min(total_rows);

                // Keep crosshair at same relative position
                self.crosshair_row =
                    (new_scroll_offset + lock_position).min(total_rows.saturating_sub(1));

                return RowNavigationResult {
                    row_position: self.crosshair_row,
                    row_scroll_offset: new_scroll_offset,
                    description: format!(
                        "Half page down with cursor lock (viewport {} → {})",
                        old_scroll_offset + 1,
                        new_scroll_offset + 1
                    ),
                    viewport_changed: true,
                };
            }
        }

        // Normal half page down behavior
        // Calculate new row position (move down by half page) using ViewportManager's crosshair
        let new_row = (self.crosshair_row + half_page).min(total_rows.saturating_sub(1));
        self.crosshair_row = new_row;

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
            "Half page down: row {} → {} (of {})",
            self.crosshair_row + 1 - half_page.min(self.crosshair_row),
            new_row + 1,
            total_rows
        );

        debug!(target: "viewport_manager", 
               "half_page_down result: new_row={}, scroll_offset={}→{}, viewport_changed={}", 
               new_row, old_scroll_offset, new_scroll_offset, viewport_changed);

        RowNavigationResult {
            row_position: new_row,
            row_scroll_offset: new_scroll_offset,
            description,
            viewport_changed,
        }
    }

    /// Navigate half page up in the data
    pub fn half_page_up(&mut self) -> RowNavigationResult {
        let total_rows = self.dataview.row_count();
        // Calculate visible rows (viewport height)
        let visible_rows = self.terminal_height.saturating_sub(6) as usize; // Account for headers, borders, status
        let half_page = visible_rows / 2;

        debug!(target: "viewport_manager", 
               "half_page_up: crosshair_row={}, half_page={}, current_viewport_rows={:?}", 
               self.crosshair_row, half_page, self.viewport_rows);

        // Check viewport lock first - prevent scrolling entirely
        if self.viewport_lock {
            debug!(target: "viewport_manager", 
                   "half_page_up: Viewport locked, moving within current viewport");
            // In viewport lock mode, move to top of current viewport
            let new_row = self.viewport_rows.start;
            self.crosshair_row = new_row;
            return RowNavigationResult {
                row_position: new_row,
                row_scroll_offset: self.viewport_rows.start,
                description: format!(
                    "Half page up within locked viewport: row {} → {}",
                    self.crosshair_row + 1,
                    new_row + 1
                ),
                viewport_changed: false,
            };
        }

        // Check cursor lock - scroll viewport but keep cursor at same relative position
        if self.cursor_lock {
            if let Some(lock_position) = self.cursor_lock_position {
                debug!(target: "viewport_manager", 
                       "half_page_up: Cursor locked at position {}", lock_position);

                // Calculate new viewport position
                let old_scroll_offset = self.viewport_rows.start;
                let new_scroll_offset = old_scroll_offset.saturating_sub(half_page);

                if new_scroll_offset == old_scroll_offset {
                    // Can't scroll further
                    return RowNavigationResult {
                        row_position: self.crosshair_row,
                        row_scroll_offset: old_scroll_offset,
                        description: "Already at top".to_string(),
                        viewport_changed: false,
                    };
                }

                // Update viewport
                self.viewport_rows =
                    new_scroll_offset..(new_scroll_offset + visible_rows).min(total_rows);

                // Keep crosshair at same relative position
                self.crosshair_row = new_scroll_offset + lock_position;

                return RowNavigationResult {
                    row_position: self.crosshair_row,
                    row_scroll_offset: new_scroll_offset,
                    description: format!(
                        "Half page up with cursor lock (viewport {} → {})",
                        old_scroll_offset + 1,
                        new_scroll_offset + 1
                    ),
                    viewport_changed: true,
                };
            }
        }

        // Normal half page up behavior
        // Calculate new row position (move up by half page) using ViewportManager's crosshair
        let new_row = self.crosshair_row.saturating_sub(half_page);
        self.crosshair_row = new_row;

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
        self.viewport_rows = new_scroll_offset..(new_scroll_offset + visible_rows).min(total_rows);
        let viewport_changed = new_scroll_offset != old_scroll_offset;

        let description = format!(
            "Half page up: row {} → {}",
            self.crosshair_row + half_page + 1,
            new_row + 1
        );

        debug!(target: "viewport_manager", 
               "half_page_up result: new_row={}, scroll_offset={}→{}, viewport_changed={}", 
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
        // Check viewport lock - prevent scrolling
        if self.viewport_lock {
            // In viewport lock mode, just move to bottom of current viewport
            let last_visible = self
                .viewport_rows
                .end
                .saturating_sub(1)
                .min(total_rows.saturating_sub(1));
            self.crosshair_row = last_visible;
            return RowNavigationResult {
                row_position: self.crosshair_row,
                row_scroll_offset: self.viewport_rows.start,
                description: "Moved to last visible row (viewport locked)".to_string(),
                viewport_changed: false,
            };
        }
        if total_rows == 0 {
            return RowNavigationResult {
                row_position: 0,
                row_scroll_offset: 0,
                description: "No rows to navigate".to_string(),
                viewport_changed: false,
            };
        }

        // Get the actual visible rows from our current viewport
        // terminal_height should already account for UI chrome
        let visible_rows = (self.terminal_height as usize).max(10);

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
        // Check viewport lock - prevent scrolling
        if self.viewport_lock {
            // In viewport lock mode, just move to top of current viewport
            self.crosshair_row = self.viewport_rows.start;
            return RowNavigationResult {
                row_position: self.crosshair_row,
                row_scroll_offset: self.viewport_rows.start,
                description: "Moved to first visible row (viewport locked)".to_string(),
                viewport_changed: false,
            };
        }
        if total_rows == 0 {
            return RowNavigationResult {
                row_position: 0,
                row_scroll_offset: 0,
                description: "No rows to navigate".to_string(),
                viewport_changed: false,
            };
        }

        // Get the actual visible rows from our current viewport
        // terminal_height should already account for UI chrome
        let visible_rows = (self.terminal_height as usize).max(10);

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
        // Calculate the middle of the viewport (viewport now only contains data rows)
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
        // viewport_rows now represents only data rows (no table chrome)
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

    /// Toggle viewport lock - when locked, crosshair stays at same viewport position while scrolling
    /// Toggle cursor lock - cursor stays at same viewport position while scrolling
    pub fn toggle_cursor_lock(&mut self) -> (bool, String) {
        self.cursor_lock = !self.cursor_lock;

        if self.cursor_lock {
            // Calculate and store the relative position within viewport
            let relative_position = self.crosshair_row.saturating_sub(self.viewport_rows.start);
            self.cursor_lock_position = Some(relative_position);

            let description = format!(
                "Cursor lock: ON (locked at viewport position {})",
                relative_position + 1
            );
            debug!(target: "viewport_manager", 
                   "Cursor lock enabled: crosshair at viewport position {}", 
                   relative_position);
            (true, description)
        } else {
            self.cursor_lock_position = None;
            let description = "Cursor lock: OFF".to_string();
            debug!(target: "viewport_manager", "Cursor lock disabled");
            (false, description)
        }
    }

    /// Toggle viewport lock - prevents scrolling and constrains cursor to current viewport
    pub fn toggle_viewport_lock(&mut self) -> (bool, String) {
        self.viewport_lock = !self.viewport_lock;

        if self.viewport_lock {
            // Store current viewport boundaries
            self.viewport_lock_boundaries = Some(self.viewport_rows.clone());

            let description = format!(
                "Viewport lock: ON (no scrolling, cursor constrained to rows {}-{})",
                self.viewport_rows.start + 1,
                self.viewport_rows.end
            );
            debug!(target: "viewport_manager", 
                   "VIEWPORT LOCK ENABLED: boundaries {:?}, crosshair={}, viewport={:?}", 
                   self.viewport_lock_boundaries, self.crosshair_row, self.viewport_rows);
            (true, description)
        } else {
            self.viewport_lock_boundaries = None;
            let description = "Viewport lock: OFF (normal scrolling)".to_string();
            debug!(target: "viewport_manager", "VIEWPORT LOCK DISABLED");
            (false, description)
        }
    }

    /// Check if cursor is locked
    pub fn is_cursor_locked(&self) -> bool {
        self.cursor_lock
    }

    /// Check if viewport is locked
    pub fn is_viewport_locked(&self) -> bool {
        self.viewport_lock
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

        // Clone the DataView, modify it, and replace the Arc
        let mut new_dataview = (*self.dataview).clone();

        // Delegate to DataView's move_column_left - it handles pinned column logic
        let success = new_dataview.move_column_left(current_column);

        if success {
            // Replace the Arc with the modified DataView
            self.dataview = Arc::new(new_dataview);
        }

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
                    self.terminal_width.saturating_sub(TABLE_BORDER_WIDTH),
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
                    let terminal_width = self.terminal_width.saturating_sub(TABLE_BORDER_WIDTH);

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

            // Update crosshair to follow the moved column
            self.crosshair_col = new_position;

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

        // Clone the DataView, modify it, and replace the Arc
        let mut new_dataview = (*self.dataview).clone();

        // Delegate to DataView's move_column_right - it handles pinned column logic
        let success = new_dataview.move_column_right(current_column);

        if success {
            // Replace the Arc with the modified DataView
            self.dataview = Arc::new(new_dataview);
        }

        if success {
            self.invalidate_cache(); // Column order changed, need to recalculate widths

            // Determine new cursor position and if wrapping occurred
            let wrapped_to_beginning = current_column == column_count - 1
                || (pinned_count > 0 && current_column == pinned_count - 1);

            let new_position = if current_column == column_count - 1 {
                // Column wrapped to beginning
                if pinned_count > 0 {
                    pinned_count // First non-pinned column
                } else {
                    0 // No pinned columns, go to start
                }
            } else if pinned_count > 0 && current_column == pinned_count - 1 {
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
                    let terminal_width = self.terminal_width.saturating_sub(TABLE_BORDER_WIDTH);

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

            // Update crosshair to follow the moved column
            self.crosshair_col = new_position;

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

        // Clone the DataView, modify it, and replace the Arc
        let mut new_dataview = (*self.dataview).clone();

        // Hide the column in the cloned DataView
        let success = new_dataview.hide_column(column_index);

        if success {
            // Replace the Arc with the modified DataView
            self.dataview = Arc::new(new_dataview);
            self.invalidate_cache(); // Column visibility changed, need to recalculate widths

            // Adjust viewport if necessary
            let column_count = self.dataview.column_count();
            if self.viewport_cols.end > column_count {
                self.viewport_cols.end = column_count;
            }
            if self.viewport_cols.start >= column_count && column_count > 0 {
                self.viewport_cols.start = column_count - 1;
            }

            // Adjust crosshair if necessary
            // If we hid the column the crosshair was on, or a column before it, adjust
            if column_index == self.crosshair_col {
                // We hid the current column
                if column_count > 0 {
                    // If we were at the last column and it's now hidden, move to the new last column
                    // Otherwise, stay at the same index (which now points to the next column)
                    if self.crosshair_col >= column_count {
                        self.crosshair_col = column_count - 1;
                    }
                    // Note: if crosshair_col < column_count, we keep the same index,
                    // which naturally moves us to the next column
                } else {
                    self.crosshair_col = 0;
                }
                debug!(target: "viewport_manager", "Crosshair was on hidden column, moved to {}", self.crosshair_col);
            } else if column_index < self.crosshair_col {
                // We hid a column before the crosshair - decrement crosshair position
                self.crosshair_col = self.crosshair_col.saturating_sub(1);
                debug!(target: "viewport_manager", "Hidden column was before crosshair, adjusted crosshair to {}", self.crosshair_col);
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

        // Clone the DataView, modify it, and replace the Arc
        let mut new_dataview = (*self.dataview).clone();

        // Hide the column in DataView
        let success = new_dataview.hide_column_by_name(column_name);

        if success {
            // Replace the Arc with the modified DataView
            self.dataview = Arc::new(new_dataview);
        }

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

            // Ensure crosshair stays within bounds after hiding
            if self.crosshair_col >= column_count && column_count > 0 {
                self.crosshair_col = column_count - 1;
                debug!(target: "viewport_manager", "Adjusted crosshair to {} after hiding column", self.crosshair_col);
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

        // Clone the DataView, modify it, and replace the Arc
        let mut new_dataview = (*self.dataview).clone();

        // Hide empty columns in DataView
        let count = new_dataview.hide_empty_columns();

        if count > 0 {
            // Replace the Arc with the modified DataView
            self.dataview = Arc::new(new_dataview);
        }

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

        // Clone the DataView, modify it, and replace the Arc
        let mut new_dataview = (*self.dataview).clone();

        // Unhide all columns in the cloned DataView
        new_dataview.unhide_all_columns();

        // Replace the Arc with the modified DataView
        self.dataview = Arc::new(new_dataview);

        self.invalidate_cache(); // Column visibility changed, need to recalculate widths

        // Reset viewport to show first columns
        let column_count = self.dataview.column_count();
        self.viewport_cols = 0..column_count.min(20); // Show first ~20 columns or all if less

        debug!(target: "viewport_manager", "All columns unhidden, viewport reset to {:?}", self.viewport_cols);
    }

    /// Update the current column position and automatically adjust viewport if needed
    /// This takes a VISUAL column index (0, 1, 2... in display order)
    pub fn set_current_column(&mut self, visual_column: usize) -> bool {
        let terminal_width = self.terminal_width.saturating_sub(TABLE_BORDER_WIDTH); // Account for borders
        let total_visual_columns = self.dataview.get_display_columns().len();

        tracing::debug!("[PIN_DEBUG] === set_current_column ===");
        tracing::debug!(
            "[PIN_DEBUG] visual_column={}, viewport_cols={:?}",
            visual_column,
            self.viewport_cols
        );
        tracing::debug!(
            "[PIN_DEBUG] terminal_width={}, total_visual_columns={}",
            terminal_width,
            total_visual_columns
        );

        debug!(target: "viewport_manager", 
               "set_current_column ENTRY: visual_column={}, current_viewport={:?}, terminal_width={}, total_visual={}", 
               visual_column, self.viewport_cols, terminal_width, total_visual_columns);

        // Validate the visual column
        if visual_column >= total_visual_columns {
            debug!(target: "viewport_manager", "Visual column {} out of bounds (max {})", visual_column, total_visual_columns);
            tracing::debug!(
                "[PIN_DEBUG] Column {} out of bounds (max {})",
                visual_column,
                total_visual_columns
            );
            return false;
        }

        // Update the crosshair position
        self.crosshair_col = visual_column;
        debug!(target: "viewport_manager", "Updated crosshair_col to {}", visual_column);
        tracing::debug!("[PIN_DEBUG] Updated crosshair_col to {}", visual_column);

        // Check if we're in optimal layout mode (all columns fit)
        // This needs to calculate based on visual columns
        let display_columns = self.dataview.get_display_columns();
        let mut total_width_needed = 0u16;
        for &dt_idx in &display_columns {
            let width =
                self.width_calculator
                    .get_column_width(&self.dataview, &self.viewport_rows, dt_idx);
            total_width_needed += width + 1; // +1 for separator
        }

        if total_width_needed <= terminal_width {
            // All columns fit - no viewport adjustment needed, all columns are visible
            debug!(target: "viewport_manager", 
                   "Visual column {} in optimal layout mode (all columns fit), no adjustment needed", visual_column);
            tracing::debug!("[PIN_DEBUG] All columns fit, no adjustment needed");
            tracing::debug!("[PIN_DEBUG] === End set_current_column (all fit) ===");
            return false;
        }

        // Check if the visual column is already visible in the viewport
        // We need to check what's ACTUALLY visible, not just what's in the viewport range
        let pinned_count = self.dataview.get_pinned_columns().len();
        tracing::debug!("[PIN_DEBUG] pinned_count={}", pinned_count);

        // Calculate which columns are actually visible with the current viewport
        let visible_columns = self.calculate_visible_column_indices(terminal_width);
        let display_columns = self.dataview.get_display_columns();

        // Check if the target visual column's DataTable index is in the visible set
        let target_dt_idx = if visual_column < display_columns.len() {
            display_columns[visual_column]
        } else {
            tracing::debug!("[PIN_DEBUG] Column {} out of bounds", visual_column);
            return false;
        };

        let is_visible = visible_columns.contains(&target_dt_idx);
        tracing::debug!(
            "[PIN_DEBUG] Column {} (dt_idx={}) visible check: visible_columns={:?}, is_visible={}",
            visual_column,
            target_dt_idx,
            visible_columns,
            is_visible
        );

        debug!(target: "viewport_manager", 
               "set_current_column CHECK: visual_column={}, viewport={:?}, is_visible={}", 
               visual_column, self.viewport_cols, is_visible);

        if is_visible {
            debug!(target: "viewport_manager", "Visual column {} already visible in viewport {:?}, no adjustment needed", 
                   visual_column, self.viewport_cols);
            tracing::debug!("[PIN_DEBUG] Column already visible, no adjustment");
            tracing::debug!("[PIN_DEBUG] === End set_current_column (no change) ===");
            return false;
        }

        // Column is not visible, need to adjust viewport
        debug!(target: "viewport_manager", "Visual column {} NOT visible, calculating new offset", visual_column);
        let new_scroll_offset = self.calculate_scroll_offset_for_visual_column(visual_column);
        let old_scroll_offset = self.viewport_cols.start;

        debug!(target: "viewport_manager", "Calculated new_scroll_offset={}, old_scroll_offset={}", 
               new_scroll_offset, old_scroll_offset);

        if new_scroll_offset != old_scroll_offset {
            // Calculate how many scrollable columns fit from the new offset
            // This is similar logic to calculate_visible_column_indices
            let display_columns = self.dataview.get_display_columns();
            let pinned_count = self.dataview.get_pinned_columns().len();
            let mut used_width = 0u16;
            let separator_width = 1u16;

            // First account for pinned column widths
            for visual_idx in 0..pinned_count {
                if visual_idx < display_columns.len() {
                    let dt_idx = display_columns[visual_idx];
                    let width = self.width_calculator.get_column_width(
                        &self.dataview,
                        &self.viewport_rows,
                        dt_idx,
                    );
                    used_width += width + separator_width;
                }
            }

            // Now calculate how many scrollable columns fit
            let mut scrollable_columns_that_fit = 0;
            let visual_start = pinned_count + new_scroll_offset;

            for visual_idx in visual_start..display_columns.len() {
                let dt_idx = display_columns[visual_idx];
                let width = self.width_calculator.get_column_width(
                    &self.dataview,
                    &self.viewport_rows,
                    dt_idx,
                );
                if used_width + width + separator_width <= terminal_width {
                    used_width += width + separator_width;
                    scrollable_columns_that_fit += 1;
                } else {
                    break;
                }
            }

            // viewport_cols represents scrollable columns only
            let new_end = new_scroll_offset + scrollable_columns_that_fit;
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
    /// Returns scroll offset in terms of scrollable columns (excluding pinned)
    fn calculate_scroll_offset_for_visual_column(&mut self, visual_column: usize) -> usize {
        debug!(target: "viewport_manager",
               "=== calculate_scroll_offset_for_visual_column ENTRY ===");
        debug!(target: "viewport_manager",
               "visual_column={}, current_viewport={:?}", visual_column, self.viewport_cols);

        let pinned_count = self.dataview.get_pinned_columns().len();
        debug!(target: "viewport_manager",
               "pinned_count={}", pinned_count);

        // If it's a pinned column, it's always visible, no scrolling needed
        if visual_column < pinned_count {
            debug!(target: "viewport_manager",
                   "Visual column {} is pinned, returning current offset {}", 
                   visual_column, self.viewport_cols.start);
            return self.viewport_cols.start; // Keep current offset
        }

        // Convert to scrollable column index
        let scrollable_column = visual_column - pinned_count;
        debug!(target: "viewport_manager",
               "Converted to scrollable_column={}", scrollable_column);

        let current_scroll_offset = self.viewport_cols.start;
        let terminal_width = self.terminal_width.saturating_sub(TABLE_BORDER_WIDTH);

        // Calculate how much width pinned columns use
        let display_columns = self.dataview.get_display_columns();
        let mut pinned_width = 0u16;
        let separator_width = 1u16;

        for visual_idx in 0..pinned_count {
            if visual_idx < display_columns.len() {
                let dt_idx = display_columns[visual_idx];
                let width = self.width_calculator.get_column_width(
                    &self.dataview,
                    &self.viewport_rows,
                    dt_idx,
                );
                pinned_width += width + separator_width;
            }
        }

        // Available width for scrollable columns
        let available_for_scrollable = terminal_width.saturating_sub(pinned_width);

        debug!(target: "viewport_manager",
               "Scroll offset calculation: target_scrollable_col={}, current_offset={}, available_width={}", 
               scrollable_column, current_scroll_offset, available_for_scrollable);

        // Smart scrolling logic in scrollable column space
        if scrollable_column < current_scroll_offset {
            // Column is to the left of viewport, scroll left to show it
            debug!(target: "viewport_manager", "Column {} is left of viewport, scrolling left to offset {}", 
                   scrollable_column, scrollable_column);
            scrollable_column
        } else {
            // Column is to the right of viewport, use MINIMAL scrolling to make it visible
            // Strategy: Try the current offset first, then increment by small steps

            debug!(target: "viewport_manager",
                   "Checking if column {} can be made visible with minimal scrolling from offset {}",
                   scrollable_column, current_scroll_offset);

            // Try starting from current offset and incrementing until target column fits
            let mut test_scroll_offset = current_scroll_offset;
            let max_scrollable_columns = display_columns.len().saturating_sub(pinned_count);

            while test_scroll_offset <= scrollable_column
                && test_scroll_offset < max_scrollable_columns
            {
                let mut used_width = 0u16;
                let mut target_column_fits = false;

                // Test columns from this scroll offset
                for test_scrollable_idx in test_scroll_offset..max_scrollable_columns {
                    let visual_idx = pinned_count + test_scrollable_idx;
                    if visual_idx < display_columns.len() {
                        let dt_idx = display_columns[visual_idx];
                        let width = self.width_calculator.get_column_width(
                            &self.dataview,
                            &self.viewport_rows,
                            dt_idx,
                        );

                        if used_width + width + separator_width <= available_for_scrollable {
                            used_width += width + separator_width;
                            if test_scrollable_idx == scrollable_column {
                                target_column_fits = true;
                                break; // Found it, no need to check more columns
                            }
                        } else {
                            break; // No more columns fit
                        }
                    }
                }

                debug!(target: "viewport_manager", 
                       "Testing scroll_offset={}: target_fits={}, used_width={}", 
                       test_scroll_offset, target_column_fits, used_width);

                if target_column_fits {
                    debug!(target: "viewport_manager", 
                           "Found minimal scroll offset {} for column {} (current was {})", 
                           test_scroll_offset, scrollable_column, current_scroll_offset);
                    return test_scroll_offset;
                }

                // If target column doesn't fit, try next offset (scroll one column right)
                test_scroll_offset += 1;
            }

            // If we couldn't find a minimal scroll, fall back to placing target column at start
            debug!(target: "viewport_manager", 
                   "Could not find minimal scroll, placing column {} at scroll offset {}", 
                   scrollable_column, scrollable_column);
            scrollable_column
        }
    }

    /// Jump to a specific line (row) with centering
    pub fn goto_line(&mut self, target_row: usize) -> RowNavigationResult {
        let total_rows = self.dataview.row_count();

        // Clamp target row to valid range
        let target_row = target_row.min(total_rows.saturating_sub(1));

        // Calculate visible rows
        let visible_rows = (self.terminal_height as usize).saturating_sub(6);

        // Calculate scroll offset to center the target row
        let centered_scroll_offset = if visible_rows > 0 {
            // Try to center the row in the viewport
            let half_viewport = visible_rows / 2;
            if target_row > half_viewport {
                // Can scroll up to center
                (target_row - half_viewport).min(total_rows.saturating_sub(visible_rows))
            } else {
                // Target is near the top, can't center
                0
            }
        } else {
            target_row
        };

        // Update viewport
        let old_scroll_offset = self.viewport_rows.start;
        self.viewport_rows =
            centered_scroll_offset..(centered_scroll_offset + visible_rows).min(total_rows);
        let viewport_changed = centered_scroll_offset != old_scroll_offset;

        // Update crosshair position
        self.crosshair_row = target_row;

        let description = format!(
            "Jumped to row {} (centered at viewport {})",
            target_row + 1,
            centered_scroll_offset + 1
        );

        debug!(target: "viewport_manager", 
               "goto_line: target_row={}, crosshair_row={}, scroll_offset={}→{}, viewport={:?}", 
               target_row, self.crosshair_row, old_scroll_offset, centered_scroll_offset, self.viewport_rows);

        RowNavigationResult {
            row_position: target_row,
            row_scroll_offset: centered_scroll_offset,
            description,
            viewport_changed,
        }
    }

    // ========== Column Operation Methods with Unified Results ==========

    /// Hide the current column (using crosshair position) and return unified result
    pub fn hide_current_column_with_result(&mut self) -> ColumnOperationResult {
        let visual_col_idx = self.get_crosshair_col();
        let columns = self.dataview.column_names();

        if visual_col_idx >= columns.len() {
            return ColumnOperationResult::failure("Invalid column position");
        }

        let col_name = columns[visual_col_idx].clone();
        let visible_count = columns.len();

        // Don't hide if it's the last visible column
        if visible_count <= 1 {
            return ColumnOperationResult::failure("Cannot hide the last visible column");
        }

        // Hide the column
        let success = self.hide_column(visual_col_idx);

        if success {
            let mut result =
                ColumnOperationResult::success(format!("Column '{}' hidden", col_name));
            result.updated_dataview = Some(self.clone_dataview());
            result.new_column_position = Some(self.get_crosshair_col());
            result.new_viewport = Some(self.viewport_cols.clone());
            result
        } else {
            ColumnOperationResult::failure(format!(
                "Cannot hide column '{}' (may be pinned)",
                col_name
            ))
        }
    }

    /// Unhide all columns and return unified result
    pub fn unhide_all_columns_with_result(&mut self) -> ColumnOperationResult {
        let hidden_columns = self.dataview.get_hidden_column_names();
        let count = hidden_columns.len();

        if count == 0 {
            return ColumnOperationResult::success("No hidden columns");
        }

        self.unhide_all_columns();

        let mut result = ColumnOperationResult::success(format!("Unhidden {} column(s)", count));
        result.updated_dataview = Some(self.clone_dataview());
        result.affected_count = Some(count);
        result.new_viewport = Some(self.viewport_cols.clone());
        result
    }

    /// Reorder column left and return unified result
    pub fn reorder_column_left_with_result(&mut self) -> ColumnOperationResult {
        let current_col = self.get_crosshair_col();
        let reorder_result = self.reorder_column_left(current_col);

        if reorder_result.success {
            let mut result = ColumnOperationResult::success(reorder_result.description);
            result.updated_dataview = Some(self.clone_dataview());
            result.new_column_position = Some(reorder_result.new_column_position);
            result.new_viewport = Some(self.viewport_cols.clone());
            result
        } else {
            ColumnOperationResult::failure(reorder_result.description)
        }
    }

    /// Reorder column right and return unified result
    pub fn reorder_column_right_with_result(&mut self) -> ColumnOperationResult {
        let current_col = self.get_crosshair_col();
        let reorder_result = self.reorder_column_right(current_col);

        if reorder_result.success {
            let mut result = ColumnOperationResult::success(reorder_result.description);
            result.updated_dataview = Some(self.clone_dataview());
            result.new_column_position = Some(reorder_result.new_column_position);
            result.new_viewport = Some(self.viewport_cols.clone());
            result
        } else {
            ColumnOperationResult::failure(reorder_result.description)
        }
    }

    // ========== COLUMN WIDTH CALCULATIONS ==========

    /// Calculate optimal column widths based on visible viewport rows
    /// This is a performance-optimized version that only examines visible data
    pub fn calculate_viewport_column_widths(
        &mut self,
        viewport_start: usize,
        viewport_end: usize,
        compact_mode: bool,
    ) -> Vec<u16> {
        let headers = self.dataview.column_names();
        let mut widths = Vec::with_capacity(headers.len());

        // Use compact mode settings
        let min_width = if compact_mode { 4 } else { 6 };
        let padding = if compact_mode { 1 } else { 2 };

        // Calculate dynamic max_width based on terminal size and column count
        let available_width = self.terminal_width.saturating_sub(10) as usize;
        let visible_cols = headers.len().min(12); // Estimate visible columns

        // Allow columns to use more space on wide terminals
        let dynamic_max = if visible_cols > 0 {
            (available_width / visible_cols).max(30).min(80)
        } else {
            30
        };

        let max_width = if compact_mode {
            dynamic_max.min(40)
        } else {
            dynamic_max
        };

        // PERF: Only convert viewport rows to strings, not entire table!
        let mut rows_to_check = Vec::new();
        let source_table = self.dataview.source();
        for i in viewport_start..viewport_end.min(source_table.row_count()) {
            if let Some(row_strings) = source_table.get_row_as_strings(i) {
                rows_to_check.push(row_strings);
            }
        }

        // Calculate width for each column
        for (col_idx, header) in headers.iter().enumerate() {
            // Start with header width
            let mut max_col_width = header.len();

            // Check only visible rows for this column
            for row in &rows_to_check {
                if let Some(value) = row.get(col_idx) {
                    let display_value = if value.is_empty() {
                        "NULL"
                    } else {
                        value.as_str()
                    };
                    max_col_width = max_col_width.max(display_value.len());
                }
            }

            // Apply min/max constraints and padding
            let width = (max_col_width + padding).clamp(min_width, max_width) as u16;
            widths.push(width);
        }

        widths
    }

    /// Calculate optimal column widths using smart viewport-based calculations
    /// Returns the calculated widths without modifying any state
    pub fn calculate_optimal_column_widths(&mut self) -> Vec<u16> {
        // Use the viewport's visible rows for calculation
        let viewport_start = self.viewport_rows.start;
        let viewport_end = self.viewport_rows.end;

        // For now, assume non-compact mode (this could be passed as a parameter)
        let compact_mode = false;

        self.calculate_viewport_column_widths(viewport_start, viewport_end, compact_mode)
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
            if next_width < self.wasted_space {
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
            self.column_widths.clone(),
            avg_width,
            efficiency_analysis
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};

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

    // Comprehensive navigation and column operation tests

    #[test]
    fn test_navigate_to_last_and_first_column() {
        let dataview = create_test_dataview();
        let mut vm = ViewportManager::new(dataview);
        vm.update_terminal_size(120, 40);

        // Navigate to last column
        let result = vm.navigate_to_last_column();
        assert_eq!(vm.get_crosshair_col(), 2); // We have 3 columns (0-2)
        assert_eq!(result.column_position, 2);

        // Navigate back to first column
        let result = vm.navigate_to_first_column();
        assert_eq!(vm.get_crosshair_col(), 0);
        assert_eq!(result.column_position, 0);
    }

    #[test]
    fn test_column_reorder_right_with_crosshair() {
        let dataview = create_test_dataview();
        let mut vm = ViewportManager::new(dataview);
        vm.update_terminal_size(120, 40);

        // Start at column 0 (id)
        vm.crosshair_col = 0;

        // Move column right (swap id with name)
        let result = vm.reorder_column_right(0);
        assert!(result.success);
        assert_eq!(result.new_column_position, 1);
        assert_eq!(vm.get_crosshair_col(), 1); // Crosshair follows the moved column

        // Verify column order changed
        let headers = vm.dataview.column_names();
        assert_eq!(headers[0], "name"); // name is now at position 0
        assert_eq!(headers[1], "id"); // id is now at position 1
    }

    #[test]
    fn test_column_reorder_left_with_crosshair() {
        let dataview = create_test_dataview();
        let mut vm = ViewportManager::new(dataview);
        vm.update_terminal_size(120, 40);

        // Start at column 1 (name)
        vm.crosshair_col = 1;

        // Move column left (swap name with id)
        let result = vm.reorder_column_left(1);
        assert!(result.success);
        assert_eq!(result.new_column_position, 0);
        assert_eq!(vm.get_crosshair_col(), 0); // Crosshair follows the moved column
    }

    #[test]
    fn test_hide_column_adjusts_crosshair() {
        let dataview = create_test_dataview();
        let mut vm = ViewportManager::new(dataview);
        vm.update_terminal_size(120, 40);

        // Test hiding column that crosshair is on
        vm.crosshair_col = 1; // On "name" column
        let success = vm.hide_column(1);
        assert!(success);
        // Crosshair stays at index 1, which now points to "amount"
        assert_eq!(vm.get_crosshair_col(), 1);
        assert_eq!(vm.dataview.column_count(), 2); // Only 2 visible columns now

        // Test hiding last column when crosshair is on it
        vm.crosshair_col = 1; // On last visible column now
        let success = vm.hide_column(1);
        assert!(success);
        assert_eq!(vm.get_crosshair_col(), 0); // Moved to previous column
        assert_eq!(vm.dataview.column_count(), 1); // Only 1 visible column
    }

    #[test]
    fn test_goto_line_centers_viewport() {
        let dataview = create_test_dataview();
        let mut vm = ViewportManager::new(dataview);
        vm.update_terminal_size(120, 40);

        // Jump to row 50
        let result = vm.goto_line(50);
        assert_eq!(result.row_position, 50);
        assert_eq!(vm.get_crosshair_row(), 50);

        // Verify viewport is centered around target row
        let visible_rows = 34; // 40 - 6 for headers/status
        let expected_offset = 50 - (visible_rows / 2);
        assert_eq!(result.row_scroll_offset, expected_offset);
    }

    #[test]
    fn test_page_navigation() {
        let dataview = create_test_dataview();
        let mut vm = ViewportManager::new(dataview);
        vm.update_terminal_size(120, 40);

        // Test page down
        let initial_row = vm.get_crosshair_row();
        let result = vm.page_down();
        assert!(result.row_position > initial_row);
        assert_eq!(vm.get_crosshair_row(), result.row_position);

        // Test page up to return
        vm.page_down(); // Go down more
        vm.page_down();
        let prev_position = vm.get_crosshair_row();
        let result = vm.page_up();
        assert!(result.row_position < prev_position); // Should have moved up
    }

    #[test]
    fn test_cursor_lock_mode() {
        let dataview = create_test_dataview();
        let mut vm = ViewportManager::new(dataview);
        vm.update_terminal_size(120, 40);

        // Enable cursor lock
        vm.toggle_cursor_lock();
        assert!(vm.is_cursor_locked());

        // Move down with cursor lock - viewport position should stay same
        let initial_viewport_position = vm.get_crosshair_row() - vm.viewport_rows.start;
        let result = vm.navigate_row_down();

        // With cursor lock, viewport should scroll but cursor stays at same viewport position
        if result.viewport_changed {
            let new_viewport_position = vm.get_crosshair_row() - vm.viewport_rows.start;
            assert_eq!(initial_viewport_position, new_viewport_position);
        }
    }

    #[test]
    fn test_viewport_lock_prevents_scrolling() {
        let dataview = create_test_dataview();
        let mut vm = ViewportManager::new(dataview);
        vm.update_terminal_size(120, 40);

        // Enable viewport lock
        vm.toggle_viewport_lock();
        assert!(vm.is_viewport_locked());

        // Try to navigate - viewport should not change
        let initial_viewport = vm.viewport_rows.clone();
        let result = vm.navigate_row_down();

        // Viewport should remain the same
        assert_eq!(vm.viewport_rows, initial_viewport);
        // Viewport lock should prevent scrolling
        assert!(!result.viewport_changed);
    }

    #[test]
    fn test_h_m_l_viewport_navigation() {
        let dataview = create_test_dataview();
        let mut vm = ViewportManager::new(dataview);
        vm.update_terminal_size(120, 40);

        // Move down to establish a viewport
        for _ in 0..20 {
            vm.navigate_row_down();
        }

        // Test H (top of viewport)
        let result = vm.navigate_to_viewport_top();
        assert_eq!(vm.get_crosshair_row(), vm.viewport_rows.start);

        // Test L (bottom of viewport)
        let result = vm.navigate_to_viewport_bottom();
        assert_eq!(vm.get_crosshair_row(), vm.viewport_rows.end - 1);

        // Test M (middle of viewport)
        let result = vm.navigate_to_viewport_middle();
        let expected_middle =
            vm.viewport_rows.start + (vm.viewport_rows.end - vm.viewport_rows.start) / 2;
        assert_eq!(vm.get_crosshair_row(), expected_middle);
    }

    #[test]
    fn test_out_of_order_column_navigation() {
        // Create a test dataview with 12 columns
        let mut table = DataTable::new("test");
        for i in 0..12 {
            table.add_column(DataColumn::new(&format!("col{}", i)));
        }

        // Add some test data
        for row in 0..10 {
            let mut values = Vec::new();
            for col in 0..12 {
                values.push(DataValue::String(format!("r{}c{}", row, col)));
            }
            table.add_row(DataRow::new(values)).unwrap();
        }

        // Create DataView with columns selected out of order
        // Select columns in order: col11, col0, col5, col3, col8, col1, col10, col2, col7, col4, col9, col6
        // This simulates a SQL query like: SELECT col11, col0, col5, ... FROM table
        let dataview =
            DataView::new(Arc::new(table)).with_columns(vec![11, 0, 5, 3, 8, 1, 10, 2, 7, 4, 9, 6]);

        let mut vm = ViewportManager::new(Arc::new(dataview));
        vm.update_terminal_size(200, 40); // Wide terminal to see all columns

        // Test that columns appear in the order we selected them
        let column_names = vm.dataview.column_names();
        assert_eq!(
            column_names[0], "col11",
            "First visual column should be col11"
        );
        assert_eq!(
            column_names[1], "col0",
            "Second visual column should be col0"
        );
        assert_eq!(
            column_names[2], "col5",
            "Third visual column should be col5"
        );

        // Start at first visual column (col11)
        vm.crosshair_col = 0;

        // Navigate right through all columns and verify crosshair moves sequentially
        let mut visual_positions = vec![0];
        let mut datatable_positions = vec![];

        // Record initial position
        let display_cols = vm.dataview.get_display_columns();
        datatable_positions.push(display_cols[0]);

        // Navigate right through all columns
        for i in 0..11 {
            let current_visual = vm.get_crosshair_col();
            let result = vm.navigate_column_right(current_visual);

            // Crosshair should move to next visual position
            let new_visual = vm.get_crosshair_col();
            assert_eq!(
                new_visual,
                current_visual + 1,
                "Crosshair should move from visual position {} to {}, but got {}",
                current_visual,
                current_visual + 1,
                new_visual
            );

            visual_positions.push(new_visual);
            // Get the actual DataTable index at this visual position
            let display_cols = vm.dataview.get_display_columns();
            datatable_positions.push(display_cols[new_visual]);
        }

        // Verify we visited columns in sequential visual order (0,1,2,3...11)
        assert_eq!(
            visual_positions,
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            "Crosshair should move through visual positions sequentially"
        );

        // Verify DataTable indices match our selection order
        assert_eq!(
            datatable_positions,
            vec![11, 0, 5, 3, 8, 1, 10, 2, 7, 4, 9, 6],
            "DataTable indices should match our column selection order"
        );

        // Navigate back left and verify sequential movement
        for _i in (0..11).rev() {
            let current_visual = vm.get_crosshair_col();
            let _result = vm.navigate_column_left(current_visual);

            // Crosshair should move to previous visual position
            let new_visual = vm.get_crosshair_col();
            assert_eq!(
                new_visual,
                current_visual - 1,
                "Crosshair should move from visual position {} to {}, but got {}",
                current_visual,
                current_visual - 1,
                new_visual
            );
        }

        // Should be back at first column
        assert_eq!(
            vm.get_crosshair_col(),
            0,
            "Should be back at first visual column"
        );

        // Test hiding a column and verifying navigation still works
        vm.hide_column(2); // Hide col5 (at visual position 2)

        // Navigate from position 0 to what was position 3 (now position 2)
        vm.crosshair_col = 0;
        let _result1 = vm.navigate_column_right(0);
        assert_eq!(vm.get_crosshair_col(), 1, "Should be at visual position 1");

        let _result2 = vm.navigate_column_right(1);
        assert_eq!(
            vm.get_crosshair_col(),
            2,
            "Should be at visual position 2 after hiding"
        );

        // The column at position 2 should now be what was at position 3 (col3)
        let visible_cols = vm.dataview.column_names();
        assert_eq!(
            visible_cols[2], "col3",
            "Column at position 2 should be col3 after hiding col5"
        );
    }
}
