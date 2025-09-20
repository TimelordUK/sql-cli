use crate::data::datatable::{DataTable, DataValue};
use crate::sql::parser::ast::ColumnRef;
use crate::sql::recursive_parser::SqlExpression;
use std::sync::Arc;

/// Represents a column in a computed view - either original or derived
#[derive(Debug, Clone)]
pub enum ViewColumn {
    /// Direct reference to a column in the original table
    Original {
        source_index: usize, // Index in the original DataTable
        name: String,        // May be aliased from original
    },
    /// Computed/derived column with cached results
    Derived {
        name: String,
        expression: SqlExpression,
        cached_values: Vec<DataValue>, // Pre-computed for all visible rows
    },
}

/// A view over a `DataTable` that can contain both original and computed columns
/// This is query-scoped - exists only for the duration of one query result
#[derive(Debug, Clone)]
pub struct ComputedDataView {
    /// Reference to the original source table (never modified)
    source_table: Arc<DataTable>,

    /// Column definitions (mix of original and derived)
    columns: Vec<ViewColumn>,

    /// Which rows from the source table are visible (after WHERE clause filtering)
    /// Indices refer to rows in `source_table`
    visible_rows: Vec<usize>,
}

impl ComputedDataView {
    /// Create a new computed view with specified columns and visible rows
    #[must_use]
    pub fn new(
        source_table: Arc<DataTable>,
        columns: Vec<ViewColumn>,
        visible_rows: Vec<usize>,
    ) -> Self {
        Self {
            source_table,
            columns,
            visible_rows,
        }
    }

    /// Get the number of visible rows
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.visible_rows.len()
    }

    /// Get the number of columns (original + derived)
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get column names
    #[must_use]
    pub fn column_names(&self) -> Vec<String> {
        self.columns
            .iter()
            .map(|col| match col {
                ViewColumn::Original { name, .. } => name.clone(),
                ViewColumn::Derived { name, .. } => name.clone(),
            })
            .collect()
    }

    /// Get a value at a specific row and column
    #[must_use]
    pub fn get_value(&self, row_idx: usize, col_idx: usize) -> Option<DataValue> {
        // Check bounds
        if row_idx >= self.visible_rows.len() || col_idx >= self.columns.len() {
            return None;
        }

        match &self.columns[col_idx] {
            ViewColumn::Original { source_index, .. } => {
                // Get the actual row index in the source table
                let source_row_idx = self.visible_rows[row_idx];

                // Get value from original table
                self.source_table
                    .get_row(source_row_idx)
                    .and_then(|row| row.get(*source_index))
                    .cloned()
            }
            ViewColumn::Derived { cached_values, .. } => {
                // Return pre-computed value
                cached_values.get(row_idx).cloned()
            }
        }
    }

    /// Get all values for a row
    #[must_use]
    pub fn get_row_values(&self, row_idx: usize) -> Option<Vec<DataValue>> {
        if row_idx >= self.visible_rows.len() {
            return None;
        }

        let mut values = Vec::new();
        for col_idx in 0..self.columns.len() {
            values.push(self.get_value(row_idx, col_idx)?);
        }
        Some(values)
    }

    /// Get the underlying source table (for reference, not modification)
    #[must_use]
    pub fn source_table(&self) -> &Arc<DataTable> {
        &self.source_table
    }

    /// Get the visible row indices (useful for debugging)
    #[must_use]
    pub fn visible_rows(&self) -> &[usize] {
        &self.visible_rows
    }

    /// Check if a column is derived
    #[must_use]
    pub fn is_derived_column(&self, col_idx: usize) -> bool {
        matches!(self.columns.get(col_idx), Some(ViewColumn::Derived { .. }))
    }

    /// Create a simple view showing all columns from source (no computations)
    #[must_use]
    pub fn from_source_all_columns(source: Arc<DataTable>) -> Self {
        let columns: Vec<ViewColumn> = source
            .column_names()
            .into_iter()
            .enumerate()
            .map(|(idx, name)| ViewColumn::Original {
                source_index: idx,
                name,
            })
            .collect();

        let visible_rows: Vec<usize> = (0..source.row_count()).collect();

        Self::new(source, columns, visible_rows)
    }

    /// Create a view with filtered rows (WHERE clause applied)
    #[must_use]
    pub fn with_filtered_rows(mut self, row_indices: Vec<usize>) -> Self {
        self.visible_rows = row_indices;

        // Update cached values for derived columns
        for col in &mut self.columns {
            if let ViewColumn::Derived { cached_values, .. } = col {
                // Filter cached values to match new row set
                let mut new_cache = Vec::new();
                for &row_idx in &self.visible_rows {
                    if row_idx < cached_values.len() {
                        new_cache.push(cached_values[row_idx].clone());
                    }
                }
                *cached_values = new_cache;
            }
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow};

    fn create_test_table() -> Arc<DataTable> {
        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("a"));
        table.add_column(DataColumn::new("b"));

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(10),
                DataValue::Float(2.5),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(20),
                DataValue::Float(3.5),
            ]))
            .unwrap();

        Arc::new(table)
    }

    #[test]
    fn test_original_columns_view() {
        let table = create_test_table();
        let view = ComputedDataView::from_source_all_columns(table);

        assert_eq!(view.row_count(), 2);
        assert_eq!(view.column_count(), 2);
        assert_eq!(view.column_names(), vec!["a", "b"]);

        // Check values
        assert_eq!(view.get_value(0, 0), Some(DataValue::Integer(10)));
        assert_eq!(view.get_value(0, 1), Some(DataValue::Float(2.5)));
        assert_eq!(view.get_value(1, 0), Some(DataValue::Integer(20)));
    }

    #[test]
    fn test_mixed_columns() {
        let table = create_test_table();

        // Create a view with original column 'a' and derived column 'doubled'
        let columns = vec![
            ViewColumn::Original {
                source_index: 0,
                name: "a".to_string(),
            },
            ViewColumn::Derived {
                name: "doubled".to_string(),
                expression: SqlExpression::Column(ColumnRef::unquoted("a".to_string())), // Placeholder
                cached_values: vec![
                    DataValue::Integer(20), // 10 * 2
                    DataValue::Integer(40), // 20 * 2
                ],
            },
        ];

        let view = ComputedDataView::new(table, columns, vec![0, 1]);

        assert_eq!(view.column_count(), 2);
        assert_eq!(view.column_names(), vec!["a", "doubled"]);

        // Original column
        assert_eq!(view.get_value(0, 0), Some(DataValue::Integer(10)));

        // Derived column
        assert_eq!(view.get_value(0, 1), Some(DataValue::Integer(20)));
        assert_eq!(view.get_value(1, 1), Some(DataValue::Integer(40)));
    }

    #[test]
    fn test_filtered_rows() {
        let table = create_test_table();
        let view = ComputedDataView::from_source_all_columns(table).with_filtered_rows(vec![1]); // Only second row visible

        assert_eq!(view.row_count(), 1);
        assert_eq!(view.get_value(0, 0), Some(DataValue::Integer(20)));
    }
}
