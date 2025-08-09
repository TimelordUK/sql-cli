use serde_json::Value;

/// Manages column-related operations for table display
pub struct ColumnManager;

impl ColumnManager {
    /// Calculate optimal column widths based on data content
    pub fn calculate_optimal_widths(data: &[Value]) -> Vec<u16> {
        if data.is_empty() {
            return Vec::new();
        }

        let first_row = match data.first() {
            Some(row) => row,
            None => return Vec::new(),
        };

        let obj = match first_row.as_object() {
            Some(obj) => obj,
            None => return Vec::new(),
        };

        let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        let mut widths = Vec::new();

        // For large datasets, sample rows instead of checking all
        const MAX_ROWS_TO_CHECK: usize = 100;
        let total_rows = data.len();

        // Determine which rows to sample
        let rows_to_check: Vec<usize> = if total_rows <= MAX_ROWS_TO_CHECK {
            // Check all rows for small datasets
            (0..total_rows).collect()
        } else {
            // Sample evenly distributed rows for large datasets
            let step = total_rows / MAX_ROWS_TO_CHECK;
            (0..MAX_ROWS_TO_CHECK)
                .map(|i| (i * step).min(total_rows - 1))
                .collect()
        };

        for header in &headers {
            // Start with header width
            let mut max_width = header.len();

            // Check only sampled rows for this column
            for &row_idx in &rows_to_check {
                if let Some(row) = data.get(row_idx) {
                    if let Some(obj) = row.as_object() {
                        if let Some(value) = obj.get(*header) {
                            let display_len = match value {
                                Value::String(s) => s.len(),
                                Value::Number(n) => n.to_string().len(),
                                Value::Bool(b) => b.to_string().len(),
                                Value::Null => 4, // "null".len()
                                _ => value.to_string().len(),
                            };
                            max_width = max_width.max(display_len);
                        }
                    }
                }
            }

            // Add some padding and set reasonable limits
            let optimal_width = (max_width + 2).max(4).min(50); // 4-50 char range with 2 char padding
            widths.push(optimal_width as u16);
        }

        widths
    }

    /// Calculate column widths for filtered data (string arrays)
    pub fn calculate_widths_for_filtered(headers: &[String], data: &[Vec<String>]) -> Vec<u16> {
        let mut widths = Vec::new();

        // For large datasets, sample rows instead of checking all
        const MAX_ROWS_TO_CHECK: usize = 100;
        let total_rows = data.len();

        // Determine which rows to sample
        let rows_to_check: Vec<usize> = if total_rows <= MAX_ROWS_TO_CHECK {
            (0..total_rows).collect()
        } else {
            let step = total_rows / MAX_ROWS_TO_CHECK;
            (0..MAX_ROWS_TO_CHECK)
                .map(|i| (i * step).min(total_rows - 1))
                .collect()
        };

        for (col_idx, header) in headers.iter().enumerate() {
            // Start with header width
            let mut max_width = header.len();

            // Check only sampled rows for this column
            for &row_idx in &rows_to_check {
                if let Some(row) = data.get(row_idx) {
                    if let Some(value) = row.get(col_idx) {
                        max_width = max_width.max(value.len());
                    }
                }
            }

            // Add some padding and set reasonable limits
            let optimal_width = (max_width + 2).max(4).min(50);
            widths.push(optimal_width as u16);
        }

        widths
    }

    /// Get display width for a single value
    pub fn get_value_display_width(value: &Value) -> usize {
        match value {
            Value::String(s) => s.len(),
            Value::Number(n) => n.to_string().len(),
            Value::Bool(b) => b.to_string().len(),
            Value::Null => 4, // "null"
            _ => value.to_string().len(),
        }
    }
}
