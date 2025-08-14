use anyhow::Result;
use std::sync::Arc;

use crate::data::datatable::{DataRow, DataTable, DataValue};

/// A view over a DataTable that can filter, sort, and project columns
/// without modifying the underlying data
#[derive(Clone)]
pub struct DataView {
    /// The underlying immutable data source
    source: Arc<DataTable>,

    /// Row indices that are visible (after filtering)
    visible_rows: Vec<usize>,

    /// Column indices that are visible (after projection)
    visible_columns: Vec<usize>,

    /// Limit and offset for pagination
    limit: Option<usize>,
    offset: usize,
}

impl DataView {
    /// Create a new view showing all data from the table
    pub fn new(source: Arc<DataTable>) -> Self {
        let row_count = source.row_count();
        let col_count = source.column_count();

        Self {
            source,
            visible_rows: (0..row_count).collect(),
            visible_columns: (0..col_count).collect(),
            limit: None,
            offset: 0,
        }
    }

    /// Create a view with specific columns
    pub fn with_columns(mut self, columns: Vec<usize>) -> Self {
        self.visible_columns = columns;
        self
    }

    /// Hide a column by index
    pub fn hide_column(&mut self, column_index: usize) {
        self.visible_columns.retain(|&idx| idx != column_index);
    }

    /// Hide a column by name
    pub fn hide_column_by_name(&mut self, column_name: &str) {
        if let Some(col_idx) = self.source.get_column_index(column_name) {
            self.hide_column(col_idx);
        }
    }

    /// Move a column left in the view (swap with previous column)
    /// With wraparound: moving left from first position moves to last
    pub fn move_column_left(&mut self, visible_column_index: usize) -> bool {
        if visible_column_index >= self.visible_columns.len() {
            return false;
        }

        if visible_column_index == 0 {
            // Wraparound: move first column to end
            let col = self.visible_columns.remove(0);
            self.visible_columns.push(col);
        } else {
            // Normal swap with previous
            self.visible_columns
                .swap(visible_column_index - 1, visible_column_index);
        }
        true
    }

    /// Move a column right in the view (swap with next column)
    /// With wraparound: moving right from last position moves to first
    pub fn move_column_right(&mut self, visible_column_index: usize) -> bool {
        let len = self.visible_columns.len();
        if visible_column_index >= len {
            return false;
        }

        if visible_column_index == len - 1 {
            // Wraparound: move last column to beginning
            let col = self.visible_columns.pop().unwrap();
            self.visible_columns.insert(0, col);
        } else {
            // Normal swap with next
            self.visible_columns
                .swap(visible_column_index, visible_column_index + 1);
        }
        true
    }

    /// Move a column by name to the left
    pub fn move_column_left_by_name(&mut self, column_name: &str) -> bool {
        if let Some(source_idx) = self.source.get_column_index(column_name) {
            if let Some(visible_idx) = self
                .visible_columns
                .iter()
                .position(|&idx| idx == source_idx)
            {
                return self.move_column_left(visible_idx);
            }
        }
        false
    }

    /// Move a column by name to the right
    pub fn move_column_right_by_name(&mut self, column_name: &str) -> bool {
        if let Some(source_idx) = self.source.get_column_index(column_name) {
            if let Some(visible_idx) = self
                .visible_columns
                .iter()
                .position(|&idx| idx == source_idx)
            {
                return self.move_column_right(visible_idx);
            }
        }
        false
    }

    /// Create a view with specific rows
    pub fn with_rows(mut self, rows: Vec<usize>) -> Self {
        self.visible_rows = rows;
        self
    }

    /// Apply limit and offset
    pub fn with_limit(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = offset;
        self
    }

    /// Filter rows based on a predicate
    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&DataTable, usize) -> bool,
    {
        self.visible_rows = self
            .visible_rows
            .into_iter()
            .filter(|&row_idx| predicate(&self.source, row_idx))
            .collect();
        self
    }

    /// Sort rows by a column
    pub fn sort_by(mut self, column_index: usize, ascending: bool) -> Result<Self> {
        if column_index >= self.source.column_count() {
            return Err(anyhow::anyhow!(
                "Column index {} out of bounds",
                column_index
            ));
        }

        let source = &self.source;
        self.visible_rows.sort_by(|&a, &b| {
            let val_a = source.get_value(a, column_index);
            let val_b = source.get_value(b, column_index);

            let cmp = match (val_a, val_b) {
                (Some(DataValue::Integer(a)), Some(DataValue::Integer(b))) => a.cmp(&b),
                (Some(DataValue::Float(a)), Some(DataValue::Float(b))) => {
                    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Some(DataValue::String(a)), Some(DataValue::String(b))) => a.cmp(&b),
                (Some(DataValue::Boolean(a)), Some(DataValue::Boolean(b))) => a.cmp(&b),
                (Some(DataValue::DateTime(a)), Some(DataValue::DateTime(b))) => a.cmp(&b),
                (Some(DataValue::Null), Some(DataValue::Null)) => std::cmp::Ordering::Equal,
                (Some(DataValue::Null), _) => std::cmp::Ordering::Less,
                (_, Some(DataValue::Null)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            };

            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });

        Ok(self)
    }

    /// Get the number of visible rows
    pub fn row_count(&self) -> usize {
        let count = self.visible_rows.len();

        // Apply limit if set
        if let Some(limit) = self.limit {
            let available = count.saturating_sub(self.offset);
            available.min(limit)
        } else {
            count.saturating_sub(self.offset)
        }
    }

    /// Get the number of visible columns
    pub fn column_count(&self) -> usize {
        self.visible_columns.len()
    }

    /// Get column names for visible columns
    pub fn column_names(&self) -> Vec<String> {
        let all_columns = self.source.column_names();
        self.visible_columns
            .iter()
            .filter_map(|&idx| all_columns.get(idx).cloned())
            .collect()
    }

    /// Get a row by index (respecting limit/offset)
    pub fn get_row(&self, index: usize) -> Option<DataRow> {
        let actual_index = index + self.offset;

        // Check if within limit
        if let Some(limit) = self.limit {
            if index >= limit {
                return None;
            }
        }

        // Get the actual row index
        let row_idx = *self.visible_rows.get(actual_index)?;

        // Build a row with only visible columns
        let mut values = Vec::new();
        for &col_idx in &self.visible_columns {
            let value = self
                .source
                .get_value(row_idx, col_idx)
                .cloned()
                .unwrap_or(DataValue::Null);
            values.push(value);
        }

        Some(DataRow::new(values))
    }

    /// Get all visible rows (respecting limit/offset)
    pub fn get_rows(&self) -> Vec<DataRow> {
        let count = self.row_count();
        (0..count).filter_map(|i| self.get_row(i)).collect()
    }

    /// Get the source DataTable
    pub fn source(&self) -> &DataTable {
        &self.source
    }

    /// Check if a column index is visible
    pub fn is_column_visible(&self, index: usize) -> bool {
        self.visible_columns.contains(&index)
    }

    /// Get visible column indices
    pub fn visible_column_indices(&self) -> &[usize] {
        &self.visible_columns
    }

    /// Get visible row indices (before limit/offset)
    pub fn visible_row_indices(&self) -> &[usize] {
        &self.visible_rows
    }
}
