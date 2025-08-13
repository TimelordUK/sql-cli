//! Data provider traits for abstracting data access
//!
//! This module defines the core traits that allow the TUI to work with
//! data without knowing the underlying implementation (Buffer, CSVClient, DataTable, etc.)

use std::fmt::Debug;
use std::hash::Hash;

/// Filter specification for DataView
#[derive(Debug, Clone)]
pub enum FilterSpec {
    /// SQL WHERE clause filter
    WhereClause(String),
    /// Fuzzy text search across all columns
    FuzzySearch(String),
    /// Column-specific filter
    ColumnFilter { column: usize, pattern: String },
    /// Custom filter function
    Custom(String),
}

/// Sort order for columns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Data type for columns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    Text,
    Integer,
    Float,
    Date,
    Boolean,
    Json,
    Mixed,
    Unknown,
}

/// Column statistics
#[derive(Debug, Clone)]
pub struct ColumnStats {
    pub null_count: usize,
    pub unique_count: usize,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub mean_value: Option<f64>,
}

/// Core trait for read-only data access
///
/// This trait defines the minimal interface that any data source must provide
/// to be usable by the TUI for rendering and display.
pub trait DataProvider: Send + Sync + Debug {
    /// Get a single row by index
    /// Returns None if the index is out of bounds
    fn get_row(&self, index: usize) -> Option<Vec<String>>;

    /// Get the column names/headers
    fn get_column_names(&self) -> Vec<String>;

    /// Get the total number of rows
    fn get_row_count(&self) -> usize;

    /// Get the total number of columns
    fn get_column_count(&self) -> usize;

    /// Get multiple rows for efficient rendering
    /// This is an optimization to avoid multiple get_row calls
    fn get_visible_rows(&self, start: usize, count: usize) -> Vec<Vec<String>> {
        let mut rows = Vec::new();
        let end = (start + count).min(self.get_row_count());

        for i in start..end {
            if let Some(row) = self.get_row(i) {
                rows.push(row);
            }
        }

        rows
    }

    /// Get the display width for each column
    /// Used for rendering column widths in the TUI
    fn get_column_widths(&self) -> Vec<usize> {
        // Default implementation: calculate from first 100 rows
        let mut widths = vec![0; self.get_column_count()];
        let sample_size = 100.min(self.get_row_count());

        // Start with column name widths
        for (i, name) in self.get_column_names().iter().enumerate() {
            if i < widths.len() {
                widths[i] = name.len();
            }
        }

        // Check first 100 rows for max width
        for row_idx in 0..sample_size {
            if let Some(row) = self.get_row(row_idx) {
                for (col_idx, value) in row.iter().enumerate() {
                    if col_idx < widths.len() {
                        widths[col_idx] = widths[col_idx].max(value.len());
                    }
                }
            }
        }

        widths
    }

    /// Get a single cell value
    /// Returns None if row or column index is out of bounds
    fn get_cell_value(&self, row: usize, col: usize) -> Option<String> {
        self.get_row(row).and_then(|r| r.get(col).cloned())
    }

    /// Get a display-formatted cell value
    /// Returns empty string if indices are out of bounds
    fn get_display_value(&self, row: usize, col: usize) -> String {
        self.get_cell_value(row, col).unwrap_or_default()
    }

    /// Get the data type of a specific column
    /// This should be cached/determined at load time, not computed on each call
    fn get_column_type(&self, column_index: usize) -> DataType {
        // Default implementation: Unknown
        // Implementations should override with actual type detection
        DataType::Unknown
    }

    /// Get data types for all columns
    /// Returns a vector where index corresponds to column index
    fn get_column_types(&self) -> Vec<DataType> {
        // Default implementation: all Unknown
        vec![DataType::Unknown; self.get_column_count()]
    }
}

/// Extended trait for data views that support filtering and sorting
///
/// This trait extends DataProvider with mutable operations that change
/// what data is visible without modifying the underlying data.
pub trait DataViewProvider: DataProvider {
    /// Apply a filter to the view
    /// The filter string format depends on the implementation
    fn apply_filter(&mut self, filter: &str) -> Result<(), String>;

    /// Clear all filters
    fn clear_filters(&mut self);

    /// Get the number of rows after filtering
    fn get_filtered_count(&self) -> usize {
        // Default: same as total count (no filtering)
        self.get_row_count()
    }

    /// Sort by a column
    fn sort_by(&mut self, column_index: usize, ascending: bool) -> Result<(), String>;

    /// Clear sorting and return to original order
    fn clear_sort(&mut self);

    /// Check if a row index is visible in the current view
    fn is_row_visible(&self, row_index: usize) -> bool {
        row_index < self.get_row_count()
    }

    /// Get sorted indices for a column (for read-only sorting)
    /// Returns a vector of indices in sorted order
    fn get_sorted_indices(&self, column_index: usize, ascending: bool) -> Vec<usize> {
        // Default implementation: return unsorted indices
        (0..self.get_row_count()).collect()
    }

    /// Check if data is currently sorted
    fn is_sorted(&self) -> bool {
        false
    }

    /// Get current sort state
    fn get_sort_state(&self) -> Option<(usize, bool)> {
        None // Returns (column_index, is_ascending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock implementation for testing
    #[derive(Debug)]
    struct MockDataProvider {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    }

    impl DataProvider for MockDataProvider {
        fn get_row(&self, index: usize) -> Option<Vec<String>> {
            self.rows.get(index).cloned()
        }

        fn get_column_names(&self) -> Vec<String> {
            self.columns.clone()
        }

        fn get_row_count(&self) -> usize {
            self.rows.len()
        }

        fn get_column_count(&self) -> usize {
            self.columns.len()
        }
    }

    #[test]
    fn test_data_provider_basics() {
        let provider = MockDataProvider {
            columns: vec!["ID".to_string(), "Name".to_string(), "Age".to_string()],
            rows: vec![
                vec!["1".to_string(), "Alice".to_string(), "30".to_string()],
                vec!["2".to_string(), "Bob".to_string(), "25".to_string()],
            ],
        };

        assert_eq!(provider.get_row_count(), 2);
        assert_eq!(provider.get_column_count(), 3);
        assert_eq!(provider.get_column_names(), vec!["ID", "Name", "Age"]);
        assert_eq!(
            provider.get_row(0),
            Some(vec!["1".to_string(), "Alice".to_string(), "30".to_string()])
        );
        assert_eq!(provider.get_cell_value(1, 1), Some("Bob".to_string()));
    }

    #[test]
    fn test_get_visible_rows() {
        let provider = MockDataProvider {
            columns: vec!["Col1".to_string()],
            rows: (0..10).map(|i| vec![format!("Row{}", i)]).collect(),
        };

        let visible = provider.get_visible_rows(2, 3);
        assert_eq!(visible.len(), 3);
        assert_eq!(visible[0], vec!["Row2"]);
        assert_eq!(visible[2], vec!["Row4"]);
    }

    #[test]
    fn test_column_widths() {
        let provider = MockDataProvider {
            columns: vec!["ID".to_string(), "LongColumnName".to_string()],
            rows: vec![
                vec!["123456".to_string(), "Short".to_string()],
                vec!["1".to_string(), "Value".to_string()],
            ],
        };

        let widths = provider.get_column_widths();
        assert_eq!(widths[0], 6); // "123456" is longest
        assert_eq!(widths[1], 14); // "LongColumnName" is longest
    }
}
