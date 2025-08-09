use crate::buffer::BufferAPI;
use crate::data_exporter::DataExporter;
use anyhow::{anyhow, Result};
use arboard::Clipboard;
use serde_json::Value;

/// Manages clipboard operations for data yanking
pub struct YankManager;

/// Result of a yank operation
pub struct YankResult {
    pub description: String,
    pub preview: String,
    pub full_value: String,
}

impl YankManager {
    /// Yank a single cell value to clipboard
    pub fn yank_cell(
        buffer: &dyn BufferAPI,
        row_index: usize,
        column_index: usize,
    ) -> Result<YankResult> {
        let results = buffer
            .get_results()
            .ok_or_else(|| anyhow!("No results available"))?;

        let row_data = results
            .data
            .get(row_index)
            .ok_or_else(|| anyhow!("Row index out of bounds"))?;

        let obj = row_data
            .as_object()
            .ok_or_else(|| anyhow!("Invalid data format"))?;

        let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        let header = headers
            .get(column_index)
            .ok_or_else(|| anyhow!("Column index out of bounds"))?;

        let value = match obj.get(*header) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::Bool(b)) => b.to_string(),
            Some(Value::Null) => "NULL".to_string(),
            Some(other) => other.to_string(),
            None => String::new(),
        };

        // Copy to clipboard
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(&value)?;

        // Prepare result
        let col_name = header.to_string();
        let display_value = if value.len() > 20 {
            format!("{}...", &value[..17])
        } else {
            value.clone()
        };

        Ok(YankResult {
            description: col_name,
            preview: display_value,
            full_value: value,
        })
    }

    /// Yank an entire row as tab-separated values
    pub fn yank_row(buffer: &dyn BufferAPI, row_index: usize) -> Result<YankResult> {
        let results = buffer
            .get_results()
            .ok_or_else(|| anyhow!("No results available"))?;

        let row_data = results
            .data
            .get(row_index)
            .ok_or_else(|| anyhow!("Row index out of bounds"))?;

        let obj = row_data
            .as_object()
            .ok_or_else(|| anyhow!("Invalid data format"))?;

        // Use DataExporter's utility function
        let row_text = DataExporter::format_row_for_clipboard(obj);

        // Copy to clipboard
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(&row_text)?;

        // Count values for preview
        let num_values = obj.len();

        Ok(YankResult {
            description: format!("Row {}", row_index + 1),
            preview: format!("{} values", num_values),
            full_value: row_text,
        })
    }

    /// Yank an entire column
    pub fn yank_column(buffer: &dyn BufferAPI, column_index: usize) -> Result<YankResult> {
        let results = buffer
            .get_results()
            .ok_or_else(|| anyhow!("No results available"))?;

        // Get header name
        let first_row = results
            .data
            .first()
            .ok_or_else(|| anyhow!("No data available"))?;

        let obj = first_row
            .as_object()
            .ok_or_else(|| anyhow!("Invalid data format"))?;

        let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        let header = headers
            .get(column_index)
            .ok_or_else(|| anyhow!("Column index out of bounds"))?;

        // Collect all values from the column
        let mut column_values = Vec::new();
        for row in &results.data {
            if let Some(obj) = row.as_object() {
                let value = match obj.get(*header) {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Number(n)) => n.to_string(),
                    Some(Value::Bool(b)) => b.to_string(),
                    Some(Value::Null) => "NULL".to_string(),
                    Some(other) => other.to_string(),
                    None => String::new(),
                };
                column_values.push(value);
            }
        }

        let column_text = column_values.join("\n");

        // Copy to clipboard
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(&column_text)?;

        let preview = if column_values.len() > 5 {
            format!("{} values", column_values.len())
        } else {
            column_values.join(", ")
        };

        Ok(YankResult {
            description: format!("Column '{}'", header),
            preview,
            full_value: column_text,
        })
    }

    /// Yank all data as CSV
    pub fn yank_all(buffer: &dyn BufferAPI) -> Result<YankResult> {
        // Determine what data to use
        let data = if buffer.is_filter_active() || buffer.is_fuzzy_filter_active() {
            // Use filtered data if available
            if let Some(filtered) = buffer.get_filtered_data() {
                // Convert string data back to JSON for CSV generation
                // This is a bit inefficient but maintains compatibility
                Self::convert_filtered_to_json(buffer, filtered)?
            } else if let Some(results) = buffer.get_results() {
                results.data.clone()
            } else {
                return Err(anyhow!("No data available"));
            }
        } else if let Some(results) = buffer.get_results() {
            results.data.clone()
        } else {
            return Err(anyhow!("No data available"));
        };

        // Generate CSV text
        let csv_text = DataExporter::generate_csv_text(&data)
            .ok_or_else(|| anyhow!("Failed to generate CSV"))?;

        // Copy to clipboard
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(&csv_text)?;

        // Create preview
        let row_count = data.len();
        let col_count = if let Some(first) = data.first() {
            if let Some(obj) = first.as_object() {
                obj.len()
            } else {
                0
            }
        } else {
            0
        };

        let filter_info = if buffer.is_filter_active() || buffer.is_fuzzy_filter_active() {
            " (filtered)"
        } else {
            ""
        };

        Ok(YankResult {
            description: format!("All data{}", filter_info),
            preview: format!("{} rows × {} columns", row_count, col_count),
            full_value: csv_text,
        })
    }

    /// Helper to convert filtered string data back to JSON
    fn convert_filtered_to_json(
        buffer: &dyn BufferAPI,
        filtered_data: &[Vec<String>],
    ) -> Result<Vec<Value>> {
        let results = buffer
            .get_results()
            .ok_or_else(|| anyhow!("No results available"))?;

        // Get headers from original results
        let first_row = results
            .data
            .first()
            .ok_or_else(|| anyhow!("No data available"))?;

        let obj = first_row
            .as_object()
            .ok_or_else(|| anyhow!("Invalid data format"))?;

        let headers: Vec<String> = obj.keys().map(|k| k.to_string()).collect();

        // Convert filtered string data back to JSON
        let json_data: Vec<Value> = filtered_data
            .iter()
            .map(|row| {
                let mut obj = serde_json::Map::new();
                for (i, header) in headers.iter().enumerate() {
                    if let Some(value) = row.get(i) {
                        // Try to preserve original types
                        if value == "NULL" || value.is_empty() {
                            obj.insert(header.clone(), Value::Null);
                        } else if let Ok(n) = value.parse::<f64>() {
                            obj.insert(
                                header.clone(),
                                Value::Number(
                                    serde_json::Number::from_f64(n)
                                        .unwrap_or_else(|| serde_json::Number::from(0)),
                                ),
                            );
                        } else if value == "true" || value == "false" {
                            obj.insert(header.clone(), Value::Bool(value == "true"));
                        } else {
                            obj.insert(header.clone(), Value::String(value.clone()));
                        }
                    } else {
                        obj.insert(header.clone(), Value::Null);
                    }
                }
                Value::Object(obj)
            })
            .collect();

        Ok(json_data)
    }
}
