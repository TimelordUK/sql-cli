/// Column utilities extracted from enhanced_tui
/// Contains column statistics, width calculations, and column data extraction
use crate::buffer::ColumnStatistics;
use crate::data::data_provider::DataProvider;
use crate::data_analyzer::{self, DataAnalyzer};
use std::collections::HashMap;

/// Calculate statistics for a specific column
pub fn calculate_column_statistics(
    provider: &dyn DataProvider,
    analyzer: &mut DataAnalyzer,
    column_index: usize,
) -> Option<ColumnStatistics> {
    let headers = provider.get_column_names();
    if headers.is_empty() || column_index >= headers.len() {
        return None;
    }

    let column_name = headers[column_index].clone();
    let row_count = provider.get_row_count();

    // Extract column data
    let mut column_data = Vec::with_capacity(row_count);
    for row_idx in 0..row_count {
        if let Some(row) = provider.get_row(row_idx) {
            if column_index < row.len() {
                column_data.push(row[column_index].clone());
            } else {
                column_data.push(String::new());
            }
        }
    }

    // Convert to references for the analyzer
    let data_refs: Vec<&str> = column_data.iter().map(|s| s.as_str()).collect();

    // Calculate statistics
    let analyzer_stats = analyzer.calculate_column_statistics(&column_name, &data_refs);

    // Convert to buffer's ColumnStatistics format
    Some(ColumnStatistics {
        column_name: analyzer_stats.column_name,
        column_type: match analyzer_stats.data_type {
            data_analyzer::ColumnType::Integer | data_analyzer::ColumnType::Float => {
                crate::buffer::ColumnType::Numeric
            }
            data_analyzer::ColumnType::String
            | data_analyzer::ColumnType::Boolean
            | data_analyzer::ColumnType::Date
            | data_analyzer::ColumnType::Unknown => crate::buffer::ColumnType::String,
            data_analyzer::ColumnType::Mixed => crate::buffer::ColumnType::Mixed,
        },
        total_count: analyzer_stats.total_values,
        null_count: analyzer_stats.null_values,
        unique_count: analyzer_stats.unique_values,
        frequency_map: analyzer_stats.frequency_map,
        min: analyzer_stats.min_value.and_then(|s| s.parse::<f64>().ok()),
        max: analyzer_stats.max_value.and_then(|s| s.parse::<f64>().ok()),
        sum: analyzer_stats.sum_value,
        mean: analyzer_stats.avg_value,
        median: analyzer_stats.median_value,
    })
}

/// Calculate optimal column widths based on content
pub fn calculate_optimal_column_widths(
    provider: &dyn DataProvider,
    max_sample_rows: usize,
) -> Vec<u16> {
    let column_count = provider.get_column_count();
    let row_count = provider.get_row_count();
    let sample_size = row_count.min(max_sample_rows);

    let mut column_widths = vec![0u16; column_count];

    // Start with header widths
    let headers = provider.get_column_names();
    for (i, header) in headers.iter().enumerate() {
        if i < column_widths.len() {
            column_widths[i] = header.len() as u16;
        }
    }

    // Sample rows to find max widths
    for row_idx in 0..sample_size {
        if let Some(row) = provider.get_row(row_idx) {
            for (col_idx, cell) in row.iter().enumerate() {
                if col_idx < column_widths.len() {
                    column_widths[col_idx] = column_widths[col_idx].max(cell.len() as u16);
                }
            }
        }
    }

    // Apply constraints
    const MIN_WIDTH: u16 = 5;
    const MAX_WIDTH: u16 = 50;

    for width in &mut column_widths {
        *width = (*width).clamp(MIN_WIDTH, MAX_WIDTH);
    }

    column_widths
}

/// Calculate column widths for a specific viewport range
pub fn calculate_viewport_column_widths(
    provider: &dyn DataProvider,
    viewport_start: usize,
    viewport_end: usize,
    max_sample_rows: usize,
) -> HashMap<usize, u16> {
    let mut widths = HashMap::new();
    let row_count = provider.get_row_count();
    let sample_size = row_count.min(max_sample_rows);

    // Get headers
    let headers = provider.get_column_names();

    // Initialize with header widths for viewport columns
    for col_idx in viewport_start..viewport_end.min(headers.len()) {
        widths.insert(col_idx, headers[col_idx].len() as u16);
    }

    // Sample rows to find max widths
    for row_idx in 0..sample_size {
        if let Some(row) = provider.get_row(row_idx) {
            for col_idx in viewport_start..viewport_end.min(row.len()) {
                let current_width = widths.get(&col_idx).copied().unwrap_or(0);
                let cell_width = row[col_idx].len() as u16;
                widths.insert(col_idx, current_width.max(cell_width));
            }
        }
    }

    // Apply constraints
    const MIN_WIDTH: u16 = 5;
    const MAX_WIDTH: u16 = 50;

    for width in widths.values_mut() {
        *width = (*width).clamp(MIN_WIDTH, MAX_WIDTH);
    }

    widths
}

/// Extract all values from a specific column
pub fn extract_column_values(provider: &dyn DataProvider, column_index: usize) -> Vec<String> {
    let row_count = provider.get_row_count();
    let mut values = Vec::with_capacity(row_count);

    for row_idx in 0..row_count {
        if let Some(row) = provider.get_row(row_idx) {
            if column_index < row.len() {
                values.push(row[column_index].clone());
            } else {
                values.push(String::new());
            }
        }
    }

    values
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    #[derive(Debug)]
    struct MockDataProvider {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    }

    impl DataProvider for MockDataProvider {
        fn get_column_count(&self) -> usize {
            self.headers.len()
        }

        fn get_row_count(&self) -> usize {
            self.rows.len()
        }

        fn get_column_names(&self) -> Vec<String> {
            self.headers.clone()
        }

        fn get_row(&self, index: usize) -> Option<Vec<String>> {
            self.rows.get(index).cloned()
        }

        fn get_cell_value(&self, row: usize, col: usize) -> Option<String> {
            self.rows.get(row).and_then(|r| r.get(col)).cloned()
        }
    }

    #[test]
    fn test_calculate_optimal_widths() {
        let provider = MockDataProvider {
            headers: vec![
                "ID".to_string(),
                "Name".to_string(),
                "Description".to_string(),
            ],
            rows: vec![
                vec!["1".to_string(), "Alice".to_string(), "Short".to_string()],
                vec![
                    "2".to_string(),
                    "Bob".to_string(),
                    "A very long description that should be clamped".to_string(),
                ],
            ],
        };

        let widths = calculate_optimal_column_widths(&provider, 10);
        assert_eq!(widths.len(), 3);
        assert!(widths[0] >= 2); // ID
        assert!(widths[1] >= 4); // Name
        assert!(widths[2] <= 50); // Description should be clamped
    }
}
