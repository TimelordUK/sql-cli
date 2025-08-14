use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde_json::{json, Value};
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

    /// Base rows before any filtering (for restoring after clear filter)
    /// This allows us to clear filters without losing sort order
    base_rows: Vec<usize>,

    /// Base columns from the original projection (for restoring after unhide all)
    /// This preserves the original column selection if view was created with specific columns
    base_columns: Vec<usize>,

    /// Active filter pattern (if any)
    filter_pattern: Option<String>,
}

impl DataView {
    /// Create a new view showing all data from the table
    pub fn new(source: Arc<DataTable>) -> Self {
        let row_count = source.row_count();
        let col_count = source.column_count();
        let all_rows: Vec<usize> = (0..row_count).collect();
        let all_columns: Vec<usize> = (0..col_count).collect();

        Self {
            source,
            visible_rows: all_rows.clone(),
            visible_columns: all_columns.clone(),
            limit: None,
            offset: 0,
            base_rows: all_rows,
            base_columns: all_columns,
            filter_pattern: None,
        }
    }

    /// Create a view with specific columns
    pub fn with_columns(mut self, columns: Vec<usize>) -> Self {
        self.visible_columns = columns.clone();
        self.base_columns = columns; // Store as the base projection
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

    /// Unhide all columns (restore to the base column projection)
    /// This restores to the original column selection, not necessarily all source columns
    pub fn unhide_all_columns(&mut self) {
        self.visible_columns = self.base_columns.clone();
    }

    /// Hide all columns (clear all visible columns)
    pub fn hide_all_columns(&mut self) {
        self.visible_columns.clear();
    }

    /// Check if any columns are visible
    pub fn has_visible_columns(&self) -> bool {
        !self.visible_columns.is_empty()
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

    /// Get the names of hidden columns (columns in source but not visible)
    pub fn get_hidden_column_names(&self) -> Vec<String> {
        let all_columns = self.source.column_names();
        let visible_columns = self.column_names();

        all_columns
            .into_iter()
            .filter(|col| !visible_columns.contains(col))
            .collect()
    }

    /// Check if there are any hidden columns
    pub fn has_hidden_columns(&self) -> bool {
        self.visible_columns.len() < self.source.column_count()
    }

    /// Create a view with specific rows
    pub fn with_rows(mut self, rows: Vec<usize>) -> Self {
        self.visible_rows = rows.clone();
        self.base_rows = rows; // Update base_rows so clear_filter restores to this
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

    /// Apply a text filter to the view (filters visible rows)
    pub fn apply_text_filter(&mut self, pattern: &str, case_sensitive: bool) {
        if pattern.is_empty() {
            self.clear_filter();
            return;
        }

        // Store the filter pattern
        self.filter_pattern = Some(pattern.to_string());

        // Filter from base_rows (not visible_rows) to allow re-filtering
        let pattern_lower = if !case_sensitive {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        self.visible_rows = self
            .base_rows
            .iter()
            .copied()
            .filter(|&row_idx| {
                // Check if any cell in the row contains the pattern
                if let Some(row) = self.source.get_row(row_idx) {
                    for value in &row.values {
                        let text = value.to_string();
                        let text_to_match = if !case_sensitive {
                            text.to_lowercase()
                        } else {
                            text
                        };
                        if text_to_match.contains(&pattern_lower) {
                            return true;
                        }
                    }
                }
                false
            })
            .collect();
    }

    /// Clear the filter and restore all base rows
    pub fn clear_filter(&mut self) {
        self.filter_pattern = None;
        self.visible_rows = self.base_rows.clone();
    }

    /// Check if a filter is active
    pub fn has_filter(&self) -> bool {
        self.filter_pattern.is_some()
    }

    /// Get the current filter pattern
    pub fn get_filter_pattern(&self) -> Option<&str> {
        self.filter_pattern.as_deref()
    }

    /// Apply a fuzzy filter to the view
    /// Supports both fuzzy matching and exact matching (when pattern starts with ')
    pub fn apply_fuzzy_filter(&mut self, pattern: &str, case_insensitive: bool) {
        if pattern.is_empty() {
            self.clear_filter();
            return;
        }

        // Store the filter pattern
        self.filter_pattern = Some(pattern.to_string());

        // Check if pattern starts with ' for exact matching
        let use_exact = pattern.starts_with('\'');

        self.visible_rows = self
            .base_rows
            .iter()
            .copied()
            .filter(|&row_idx| {
                // Get all cell values as a single string for matching
                if let Some(row) = self.source.get_row(row_idx) {
                    // Concatenate all cell values with spaces
                    let row_text = row
                        .values
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");

                    if use_exact && pattern.len() > 1 {
                        // Exact substring matching (skip the leading ')
                        let exact_pattern = &pattern[1..];
                        if case_insensitive {
                            row_text
                                .to_lowercase()
                                .contains(&exact_pattern.to_lowercase())
                        } else {
                            row_text.contains(exact_pattern)
                        }
                    } else if !use_exact {
                        // Fuzzy matching
                        let matcher = if case_insensitive {
                            SkimMatcherV2::default().ignore_case()
                        } else {
                            SkimMatcherV2::default().respect_case()
                        };

                        // Check if there's a fuzzy match with score > 0
                        matcher
                            .fuzzy_match(&row_text, pattern)
                            .map_or(false, |score| score > 0)
                    } else {
                        // Just a single quote - no pattern to match
                        false
                    }
                } else {
                    false
                }
            })
            .collect();
    }

    /// Get indices of rows that match the fuzzy filter (for compatibility)
    pub fn get_fuzzy_filter_indices(&self) -> Vec<usize> {
        // Return indices relative to the base data, not the view indices
        self.visible_rows.clone()
    }

    /// Get the visible row indices
    pub fn get_visible_rows(&self) -> Vec<usize> {
        self.visible_rows.clone()
    }

    /// Sort rows by a column (consuming version - returns new Self)
    pub fn sort_by(mut self, column_index: usize, ascending: bool) -> Result<Self> {
        self.apply_sort(column_index, ascending)?;
        Ok(self)
    }

    /// Sort rows by a column (mutable version - modifies in place)
    pub fn apply_sort(&mut self, column_index: usize, ascending: bool) -> Result<()> {
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

        // Also update base_rows so that clearing filters preserves the sort
        self.base_rows = self.visible_rows.clone();

        Ok(())
    }

    /// Clear the current sort and restore original row order
    pub fn clear_sort(&mut self) {
        let row_count = self.source.row_count();
        self.base_rows = (0..row_count).collect();

        // Reapply any active filter
        if let Some(pattern) = self.filter_pattern.clone() {
            let case_insensitive = false; // Would need to track this
            self.apply_text_filter(&pattern, case_insensitive);
        } else {
            self.visible_rows = self.base_rows.clone();
        }
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

    /// Export the visible data as JSON
    /// Returns an array of objects where each object represents a row
    pub fn to_json(&self) -> Value {
        let column_names = self.column_names();
        let mut rows = Vec::new();

        // Iterate through visible rows
        for row_idx in 0..self.row_count() {
            if let Some(row) = self.get_row(row_idx) {
                let mut obj = serde_json::Map::new();
                for (col_idx, col_name) in column_names.iter().enumerate() {
                    if let Some(value) = row.values.get(col_idx) {
                        let json_value = match value {
                            DataValue::String(s) => json!(s),
                            DataValue::Integer(i) => json!(i),
                            DataValue::Float(f) => json!(f),
                            DataValue::Boolean(b) => json!(b),
                            DataValue::DateTime(dt) => json!(dt),
                            DataValue::Null => json!(null),
                        };
                        obj.insert(col_name.clone(), json_value);
                    }
                }
                rows.push(json!(obj));
            }
        }

        json!(rows)
    }

    /// Export the visible data as CSV string
    pub fn to_csv(&self) -> Result<String> {
        let mut csv_output = String::new();
        let column_names = self.column_names();

        // Write header
        csv_output.push_str(&column_names.join(","));
        csv_output.push('\n');

        // Write data rows
        for row_idx in 0..self.row_count() {
            if let Some(row) = self.get_row(row_idx) {
                let row_strings: Vec<String> = row
                    .values
                    .iter()
                    .map(|v| {
                        let s = v.to_string();
                        // Quote values that contain commas, quotes, or newlines
                        if s.contains(',') || s.contains('"') || s.contains('\n') {
                            format!("\"{}\"", s.replace('"', "\"\""))
                        } else {
                            s
                        }
                    })
                    .collect();
                csv_output.push_str(&row_strings.join(","));
                csv_output.push('\n');
            }
        }

        Ok(csv_output)
    }

    /// Export the visible data as TSV (Tab-Separated Values) string
    pub fn to_tsv(&self) -> Result<String> {
        let mut tsv_output = String::new();
        let column_names = self.column_names();

        // Write header
        tsv_output.push_str(&column_names.join("\t"));
        tsv_output.push('\n');

        // Write data rows
        for row_idx in 0..self.row_count() {
            if let Some(row) = self.get_row(row_idx) {
                let row_strings: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
                tsv_output.push_str(&row_strings.join("\t"));
                tsv_output.push('\n');
            }
        }

        Ok(tsv_output)
    }
}
