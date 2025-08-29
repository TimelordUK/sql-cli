use crate::data::data_view::DataView;

// Constants moved from viewport_manager.rs
pub const DEFAULT_COL_WIDTH: u16 = 15;
pub const MIN_COL_WIDTH: u16 = 3;
pub const MAX_COL_WIDTH: u16 = 50;
pub const MAX_COL_WIDTH_DATA_FOCUS: u16 = 100;
pub const COLUMN_PADDING: u16 = 2;
pub const MIN_HEADER_WIDTH_DATA_FOCUS: u16 = 5;
pub const MAX_HEADER_TO_DATA_RATIO: f32 = 1.5;

/// Column packing modes for different width calculation strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnPackingMode {
    /// Focus on showing full data values (up to reasonable limit)
    /// Headers may be truncated if needed to show more data
    DataFocus,
    /// Focus on showing full headers
    /// Data may be truncated if needed to show complete column names
    HeaderFocus,
    /// Balanced approach - compromise between header and data visibility
    Balanced,
}

impl ColumnPackingMode {
    /// Cycle to the next mode
    pub fn cycle(&self) -> Self {
        match self {
            ColumnPackingMode::Balanced => ColumnPackingMode::DataFocus,
            ColumnPackingMode::DataFocus => ColumnPackingMode::HeaderFocus,
            ColumnPackingMode::HeaderFocus => ColumnPackingMode::Balanced,
        }
    }

    /// Get display name for the mode
    pub fn display_name(&self) -> &'static str {
        match self {
            ColumnPackingMode::Balanced => "Balanced",
            ColumnPackingMode::DataFocus => "Data Focus",
            ColumnPackingMode::HeaderFocus => "Header Focus",
        }
    }
}

/// Debug information for column width calculations
pub type ColumnWidthDebugInfo = (String, u16, u16, u16, u32);

/// Handles all column width calculations for the viewport
/// Extracted from ViewportManager to improve maintainability and testability
pub struct ColumnWidthCalculator {
    /// Cached column widths for current viewport
    column_widths: Vec<u16>,
    /// Column packing mode for width calculation
    packing_mode: ColumnPackingMode,
    /// Debug info for column width calculations
    /// (column_name, header_width, max_data_width_sampled, final_width, sample_count)
    column_width_debug: Vec<ColumnWidthDebugInfo>,
    /// Whether cache needs recalculation
    cache_dirty: bool,
}

impl ColumnWidthCalculator {
    /// Create a new column width calculator
    pub fn new() -> Self {
        Self {
            column_widths: Vec::new(),
            packing_mode: ColumnPackingMode::Balanced,
            column_width_debug: Vec::new(),
            cache_dirty: true,
        }
    }

    /// Get current packing mode
    pub fn get_packing_mode(&self) -> ColumnPackingMode {
        self.packing_mode
    }

    /// Set packing mode and mark cache as dirty
    pub fn set_packing_mode(&mut self, mode: ColumnPackingMode) {
        if self.packing_mode != mode {
            self.packing_mode = mode;
            self.cache_dirty = true;
        }
    }

    /// Cycle to the next packing mode
    pub fn cycle_packing_mode(&mut self) {
        self.set_packing_mode(self.packing_mode.cycle());
    }

    /// Get debug information about column width calculations
    pub fn get_debug_info(&self) -> &[ColumnWidthDebugInfo] {
        &self.column_width_debug
    }

    /// Mark cache as dirty (needs recalculation)
    pub fn mark_dirty(&mut self) {
        self.cache_dirty = true;
    }

    /// Get cached column width for a specific DataTable column index
    pub fn get_column_width(
        &mut self,
        dataview: &DataView,
        viewport_rows: &std::ops::Range<usize>,
        col_idx: usize,
    ) -> u16 {
        if self.cache_dirty {
            self.recalculate_column_widths(dataview, viewport_rows);
        }

        self.column_widths
            .get(col_idx)
            .copied()
            .unwrap_or(DEFAULT_COL_WIDTH)
    }

    /// Get all cached column widths, ensuring they're up to date
    pub fn get_all_column_widths(
        &mut self,
        dataview: &DataView,
        viewport_rows: &std::ops::Range<usize>,
    ) -> &[u16] {
        if self.cache_dirty {
            self.recalculate_column_widths(dataview, viewport_rows);
        }

        &self.column_widths
    }

    /// Calculate optimal column widths for all columns
    /// This is the core method extracted from ViewportManager
    fn recalculate_column_widths(
        &mut self,
        dataview: &DataView,
        viewport_rows: &std::ops::Range<usize>,
    ) {
        let col_count = dataview.column_count();
        self.column_widths.resize(col_count, DEFAULT_COL_WIDTH);

        // Clear debug info
        self.column_width_debug.clear();

        // Get column headers for width calculation
        let headers = dataview.column_names();

        // Calculate width for each column based on header and visible data
        for col_idx in 0..col_count {
            // Track header width separately
            let header_width = headers.get(col_idx).map(|h| h.len() as u16).unwrap_or(0);

            // Track actual data width
            let mut max_data_width = 0u16;
            let mut total_data_width = 0u64;
            let mut data_samples = 0u32;

            // Sample visible rows (limit sampling for performance)
            let sample_size = 100.min(viewport_rows.len());
            let sample_step = if viewport_rows.len() > sample_size {
                viewport_rows.len() / sample_size
            } else {
                1
            };

            for (i, row_idx) in viewport_rows.clone().enumerate() {
                // Sample every nth row for performance
                if i % sample_step != 0 && i != 0 && i != viewport_rows.len() - 1 {
                    continue;
                }

                if let Some(row) = dataview.get_row(row_idx) {
                    if col_idx < row.values.len() {
                        let cell_str = row.values[col_idx].to_string();
                        let cell_width = cell_str.len() as u16;

                        max_data_width = max_data_width.max(cell_width);
                        total_data_width += cell_width as u64;
                        data_samples += 1;

                        // Early exit if we hit max width (depends on mode)
                        let mode_max = match self.packing_mode {
                            ColumnPackingMode::DataFocus => MAX_COL_WIDTH_DATA_FOCUS,
                            _ => MAX_COL_WIDTH,
                        };
                        if max_data_width >= mode_max {
                            break;
                        }
                    }
                }
            }

            // Calculate optimal width based on packing mode
            let optimal_width =
                self.calculate_optimal_width_for_mode(header_width, max_data_width, data_samples);

            // Apply constraints based on mode
            let (min_width, max_width) = match self.packing_mode {
                ColumnPackingMode::DataFocus => (MIN_COL_WIDTH, MAX_COL_WIDTH_DATA_FOCUS),
                _ => (MIN_COL_WIDTH, MAX_COL_WIDTH),
            };

            let final_width = optimal_width.clamp(min_width, max_width);
            self.column_widths[col_idx] = final_width;

            // Store debug info
            let column_name = headers
                .get(col_idx)
                .map(|s| s.clone())
                .unwrap_or_else(|| format!("col_{}", col_idx));
            self.column_width_debug.push((
                column_name,
                header_width,
                max_data_width,
                final_width,
                data_samples,
            ));
        }

        self.cache_dirty = false;
    }

    /// Calculate optimal width for a column based on the current packing mode
    fn calculate_optimal_width_for_mode(
        &self,
        header_width: u16,
        max_data_width: u16,
        data_samples: u32,
    ) -> u16 {
        match self.packing_mode {
            ColumnPackingMode::DataFocus => {
                // Aggressively prioritize showing full data values
                if data_samples > 0 {
                    // ULTRA AGGRESSIVE for very short data (2-3 chars)
                    // This handles currency codes (USD), country codes (US), etc.
                    if max_data_width <= 3 {
                        // For 2-3 char data, just use data width + padding
                        // Don't enforce minimum header width - let it truncate heavily
                        max_data_width + COLUMN_PADDING
                    } else if max_data_width <= 10 && header_width > max_data_width * 2 {
                        // Short data (4-10 chars) with long header - still aggressive
                        // but ensure at least 5 chars for some header visibility
                        (max_data_width + COLUMN_PADDING).max(MIN_HEADER_WIDTH_DATA_FOCUS)
                    } else {
                        // Normal data - use full width but don't exceed limit
                        let data_width =
                            (max_data_width + COLUMN_PADDING).min(MAX_COL_WIDTH_DATA_FOCUS);

                        // Ensure at least minimum header visibility
                        data_width.max(MIN_HEADER_WIDTH_DATA_FOCUS)
                    }
                } else {
                    // No data samples - use header width but constrain it
                    header_width
                        .min(DEFAULT_COL_WIDTH)
                        .max(MIN_HEADER_WIDTH_DATA_FOCUS)
                }
            }
            ColumnPackingMode::HeaderFocus => {
                // Prioritize showing full headers
                let header_with_padding = header_width + COLUMN_PADDING;

                if data_samples > 0 {
                    // Ensure we show the full header, but respect data if it's wider
                    header_with_padding.max(max_data_width.min(MAX_COL_WIDTH))
                } else {
                    header_with_padding
                }
            }
            ColumnPackingMode::Balanced => {
                // Original balanced approach
                if data_samples > 0 {
                    let data_based_width = max_data_width + COLUMN_PADDING;

                    if header_width > max_data_width {
                        let max_allowed_header =
                            (max_data_width as f32 * MAX_HEADER_TO_DATA_RATIO) as u16;
                        data_based_width.max(header_width.min(max_allowed_header))
                    } else {
                        data_based_width.max(header_width)
                    }
                } else {
                    header_width.max(DEFAULT_COL_WIDTH)
                }
            }
        }
    }
}

impl Default for ColumnWidthCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
    use std::sync::Arc;

    fn create_test_dataview() -> DataView {
        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("short"));
        table.add_column(DataColumn::new("very_long_header_name"));
        table.add_column(DataColumn::new("normal"));

        // Add test data
        for i in 0..5 {
            let values = vec![
                DataValue::String("A".to_string()),       // Short data
                DataValue::String("X".to_string()),       // Short data, long header
                DataValue::String(format!("Value{}", i)), // Normal data
            ];
            table.add_row(DataRow::new(values)).unwrap();
        }

        DataView::new(Arc::new(table))
    }

    #[test]
    fn test_column_width_calculator_creation() {
        let calculator = ColumnWidthCalculator::new();
        assert_eq!(calculator.get_packing_mode(), ColumnPackingMode::Balanced);
        assert!(calculator.cache_dirty);
    }

    #[test]
    fn test_packing_mode_cycle() {
        let mut calculator = ColumnWidthCalculator::new();

        assert_eq!(calculator.get_packing_mode(), ColumnPackingMode::Balanced);

        calculator.cycle_packing_mode();
        assert_eq!(calculator.get_packing_mode(), ColumnPackingMode::DataFocus);

        calculator.cycle_packing_mode();
        assert_eq!(
            calculator.get_packing_mode(),
            ColumnPackingMode::HeaderFocus
        );

        calculator.cycle_packing_mode();
        assert_eq!(calculator.get_packing_mode(), ColumnPackingMode::Balanced);
    }

    #[test]
    fn test_width_calculation_different_modes() {
        let dataview = create_test_dataview();
        let viewport_rows = 0..5;
        let mut calculator = ColumnWidthCalculator::new();

        // Test balanced mode
        calculator.set_packing_mode(ColumnPackingMode::Balanced);
        let balanced_widths = calculator
            .get_all_column_widths(&dataview, &viewport_rows)
            .to_vec();

        // Test data focus mode
        calculator.set_packing_mode(ColumnPackingMode::DataFocus);
        let data_focus_widths = calculator
            .get_all_column_widths(&dataview, &viewport_rows)
            .to_vec();

        // Test header focus mode
        calculator.set_packing_mode(ColumnPackingMode::HeaderFocus);
        let header_focus_widths = calculator
            .get_all_column_widths(&dataview, &viewport_rows)
            .to_vec();

        // Verify we get different widths for different modes
        // (exact values depend on the algorithm, but they should differ)
        assert_eq!(balanced_widths.len(), 3);
        assert_eq!(data_focus_widths.len(), 3);
        assert_eq!(header_focus_widths.len(), 3);

        // Column 1 has a very long header but short data
        // HeaderFocus should be wider than DataFocus for this column
        assert!(header_focus_widths[1] >= data_focus_widths[1]);
    }
}
