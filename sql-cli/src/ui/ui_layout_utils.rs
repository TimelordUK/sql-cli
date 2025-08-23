//! UI layout calculation utilities
//!
//! This module contains pure utility functions for calculating UI dimensions,
//! scrolling, and layout-related operations without coupling to the main TUI state.

// UI Layout Constants (from enhanced_tui.rs)
const TABLE_BORDER_WIDTH: u16 = 4; // Left border (1) + right border (1) + padding (2)
const INPUT_AREA_HEIGHT: u16 = 3; // Height of the command input area
const STATUS_BAR_HEIGHT: u16 = 3; // Height of the status bar
const TOTAL_UI_CHROME: u16 = INPUT_AREA_HEIGHT + STATUS_BAR_HEIGHT; // Total non-table UI height
const TABLE_CHROME_ROWS: u16 = 3; // Table header (1) + top border (1) + bottom border (1)

/// Calculate the number of data rows available for display in the terminal
/// This accounts for all UI chrome including input area, status bar, table header, and borders
pub fn calculate_available_data_rows(terminal_height: u16) -> u16 {
    terminal_height
        .saturating_sub(TOTAL_UI_CHROME) // Remove input area and status bar
        .saturating_sub(TABLE_CHROME_ROWS) // Remove table header and borders
}

/// Calculate the number of data rows available for a table area
/// This accounts only for table chrome (header, borders)
pub fn calculate_table_data_rows(table_area_height: u16) -> u16 {
    table_area_height.saturating_sub(TABLE_CHROME_ROWS)
}

/// Extract timing information from debug strings
/// Parses strings like "total=123µs" or "total=1.5ms" and returns milliseconds as f64
pub fn extract_timing_from_debug_string(s: &str) -> Option<f64> {
    if let Some(total_pos) = s.find("total=") {
        let after_total = &s[total_pos + 6..];
        let time_str =
            if let Some(end_pos) = after_total.find(',').or_else(|| after_total.find(')')) {
                &after_total[..end_pos]
            } else {
                after_total
            };

        if let Some(us_pos) = time_str.find("µs") {
            time_str[..us_pos].parse::<f64>().ok().map(|us| us / 1000.0)
        } else if let Some(ms_pos) = time_str.find("ms") {
            time_str[..ms_pos].parse::<f64>().ok()
        } else if let Some(s_pos) = time_str.find('s') {
            time_str[..s_pos].parse::<f64>().ok().map(|s| s * 1000.0)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_available_data_rows() {
        // Terminal height 50 - UI chrome (6) - table chrome (3) = 41 rows
        assert_eq!(calculate_available_data_rows(50), 41);

        // Edge case: very small terminal
        assert_eq!(calculate_available_data_rows(5), 0); // saturating_sub prevents underflow
    }

    #[test]
    fn test_calculate_table_data_rows() {
        // Table area height 20 - table chrome (3) = 17 rows
        assert_eq!(calculate_table_data_rows(20), 17);

        // Edge case: table area smaller than chrome
        assert_eq!(calculate_table_data_rows(2), 0);
    }

    #[test]
    fn test_extract_timing_from_debug_string() {
        // Test microseconds conversion to milliseconds
        assert_eq!(
            extract_timing_from_debug_string("query executed total=1500µs"),
            Some(1.5)
        );

        // Test milliseconds
        assert_eq!(
            extract_timing_from_debug_string("query executed total=2.5ms"),
            Some(2.5)
        );

        // Test seconds conversion to milliseconds
        assert_eq!(
            extract_timing_from_debug_string("query executed total=1.2s"),
            Some(1200.0)
        );

        // Test with comma separator
        assert_eq!(
            extract_timing_from_debug_string("query executed total=800µs, other=123"),
            Some(0.8)
        );

        // Test with parentheses
        assert_eq!(
            extract_timing_from_debug_string("query executed (total=300µs)"),
            Some(0.3)
        );

        // Test invalid input
        assert_eq!(extract_timing_from_debug_string("no timing info"), None);
        assert_eq!(extract_timing_from_debug_string("total=invalid"), None);
    }
}
