use crate::buffer::BufferAPI;
use crate::data::data_provider::DataProvider;
use anyhow::{anyhow, Result};
use chrono::Local;
use serde_json::Value;
use std::fs::File;
use std::io::Write;

/// Handles exporting data from buffers to various formats
pub struct DataExporter;

impl DataExporter {
    /// Export data to CSV format using DataProvider trait
    pub fn export_provider_to_csv(provider: &dyn DataProvider) -> Result<String> {
        let row_count = provider.get_row_count();
        if row_count == 0 {
            return Err(anyhow!("No data to export"));
        }

        // Generate filename with timestamp
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("query_results_{}.csv", timestamp);

        let mut file = File::create(&filename)?;

        // Write headers
        let headers = provider.get_column_names();
        let header_line = headers.join(",");
        writeln!(file, "{}", header_line)?;

        // Write data rows
        for i in 0..row_count {
            if let Some(row) = provider.get_row(i) {
                let escaped_row: Vec<String> = row
                    .iter()
                    .map(|field| Self::escape_csv_field(field))
                    .collect();
                let row_line = escaped_row.join(",");
                writeln!(file, "{}", row_line)?;
            }
        }

        Ok(format!(
            "✓ Exported {} rows to CSV file: {}",
            row_count, filename
        ))
    }

    /// Export data to JSON format using DataProvider trait
    pub fn export_provider_to_json(provider: &dyn DataProvider) -> Result<String> {
        let row_count = provider.get_row_count();
        if row_count == 0 {
            return Err(anyhow!("No data to export"));
        }

        // Generate filename with timestamp
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("query_results_{}.json", timestamp);

        // Build JSON array
        let headers = provider.get_column_names();
        let mut json_array = Vec::new();

        for i in 0..row_count {
            if let Some(row) = provider.get_row(i) {
                let mut json_obj = serde_json::Map::new();
                for (j, value) in row.iter().enumerate() {
                    if j < headers.len() {
                        json_obj.insert(headers[j].clone(), Value::String(value.clone()));
                    }
                }
                json_array.push(Value::Object(json_obj));
            }
        }

        let file = File::create(&filename)?;
        serde_json::to_writer_pretty(file, &json_array)?;

        Ok(format!(
            "✓ Exported {} rows to JSON file: {}",
            row_count, filename
        ))
    }

    /// V50: Export buffer results to CSV format using DataTable
    pub fn export_to_csv(buffer: &dyn BufferAPI) -> Result<String> {
        let datatable = buffer
            .get_datatable()
            .ok_or_else(|| anyhow!("No results to export - run a query first"))?;

        if datatable.row_count() == 0 {
            return Err(anyhow!("No data to export"));
        }

        // Generate filename with timestamp
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("query_results_{}.csv", timestamp);

        let mut file = File::create(&filename)?;

        // Write headers
        let headers = datatable.column_names();
        let header_line = headers.join(",");
        writeln!(file, "{}", header_line)?;

        // Write data rows
        let mut row_count = 0;
        for row_data in datatable.to_string_table() {
            let row: Vec<String> = row_data.iter().map(|s| Self::escape_csv_field(s)).collect();
            let row_line = row.join(",");
            writeln!(file, "{}", row_line)?;
            row_count += 1;
        }

        Ok(format!(
            "✓ Exported {} rows to CSV file: {}",
            row_count, filename
        ))
    }

    /// Export buffer results to JSON format
    pub fn export_to_json(buffer: &dyn BufferAPI, include_filtered: bool) -> Result<String> {
        // V50: For now, convert DataTable back to JSON for export
        // This will be optimized later
        let datatable = buffer
            .get_datatable()
            .ok_or_else(|| anyhow!("No results to export - run a query first"))?;

        // Convert DataTable to JSON values for export
        let data = Self::datatable_to_json_values(datatable);

        // Determine what data to export
        let data_to_export = if include_filtered && buffer.is_filter_active() {
            Self::get_filtered_data(buffer)?
        } else if include_filtered && buffer.is_fuzzy_filter_active() {
            Self::get_fuzzy_filtered_data(buffer)?
        } else {
            data.clone()
        };

        // Generate filename with timestamp
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("query_results_{}.json", timestamp);

        let file = File::create(&filename)?;
        serde_json::to_writer_pretty(file, &data_to_export)?;

        let filter_info =
            if include_filtered && (buffer.is_filter_active() || buffer.is_fuzzy_filter_active()) {
                " (filtered)"
            } else {
                ""
            };

        Ok(format!(
            "✓ Exported{} {} rows to JSON file: {}",
            filter_info,
            data_to_export.len(),
            filename
        ))
    }

    /// V50: Export selected rows to CSV
    pub fn export_selected_to_csv(
        buffer: &dyn BufferAPI,
        selected_rows: &[usize],
    ) -> Result<String> {
        let datatable = buffer
            .get_datatable()
            .ok_or_else(|| anyhow!("No results to export"))?;
        let data = Self::datatable_to_json_values(datatable);

        if selected_rows.is_empty() {
            return Err(anyhow!("No rows selected"));
        }

        // Get first valid row for headers
        let first_row_idx = selected_rows[0];
        let first_row = data
            .get(first_row_idx)
            .ok_or_else(|| anyhow!("Invalid row index"))?;

        let obj = first_row
            .as_object()
            .ok_or_else(|| anyhow!("Invalid data format"))?;

        // Generate filename with timestamp
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("selected_rows_{}.csv", timestamp);

        let mut file = File::create(&filename)?;

        // Write headers
        let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        let header_line = headers.join(",");
        writeln!(file, "{}", header_line)?;

        // Write selected rows
        let mut row_count = 0;
        for &row_idx in selected_rows {
            if let Some(item) = data.get(row_idx) {
                if let Some(obj) = item.as_object() {
                    let row: Vec<String> = headers
                        .iter()
                        .map(|&header| match obj.get(header) {
                            Some(Value::String(s)) => Self::escape_csv_field(s),
                            Some(Value::Number(n)) => n.to_string(),
                            Some(Value::Bool(b)) => b.to_string(),
                            Some(Value::Null) => String::new(),
                            Some(other) => Self::escape_csv_field(&other.to_string()),
                            None => String::new(),
                        })
                        .collect();

                    let row_line = row.join(",");
                    writeln!(file, "{}", row_line)?;
                    row_count += 1;
                }
            }
        }

        Ok(format!(
            "Exported {} selected rows to {}",
            row_count, filename
        ))
    }

    /// Helper to escape CSV fields that contain special characters
    fn escape_csv_field(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            // Escape quotes by doubling them and wrap field in quotes
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }

    /// Get filtered data based on current filter
    fn get_filtered_data(buffer: &dyn BufferAPI) -> Result<Vec<Value>> {
        let datatable = buffer
            .get_datatable()
            .ok_or_else(|| anyhow!("No results available"))?;
        let data = Self::datatable_to_json_values(datatable);

        let filter_pattern = buffer.get_filter_pattern();
        if filter_pattern.is_empty() {
            return Ok(data.clone());
        }

        let regex = regex::Regex::new(&filter_pattern)
            .map_err(|e| anyhow!("Invalid filter pattern: {}", e))?;

        let filtered: Vec<Value> = results
            .data
            .iter()
            .filter(|item| {
                if let Some(obj) = item.as_object() {
                    obj.values().any(|v| {
                        let text = match v {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => String::new(),
                        };
                        regex.is_match(&text)
                    })
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Get fuzzy filtered data based on current fuzzy filter
    fn get_fuzzy_filtered_data(buffer: &dyn BufferAPI) -> Result<Vec<Value>> {
        let datatable = buffer
            .get_datatable()
            .ok_or_else(|| anyhow!("No results available"))?;
        let data = Self::datatable_to_json_values(datatable);

        let indices = buffer.get_fuzzy_filter_indices();
        if indices.is_empty() {
            return Ok(data.clone());
        }

        let filtered: Vec<Value> = indices
            .iter()
            .filter_map(|&idx| data.get(idx).cloned())
            .collect();

        Ok(filtered)
    }

    /// Export a single value to clipboard-friendly format
    pub fn format_for_clipboard(value: &Value, _header: &str) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "NULL".to_string(),
            other => other.to_string(),
        }
    }

    /// Export row as tab-separated values for clipboard
    pub fn format_row_for_clipboard(row: &serde_json::Map<String, Value>) -> String {
        let values: Vec<String> = row
            .values()
            .map(|v| Self::format_for_clipboard(v, ""))
            .collect();
        values.join("\t")
    }

    /// Convert JSON query results to a 2D vector of strings for display
    pub fn convert_json_to_strings(data: &[Value]) -> Vec<Vec<String>> {
        if let Some(first_row) = data.first() {
            if let Some(obj) = first_row.as_object() {
                let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();

                data.iter()
                    .map(|item| {
                        if let Some(obj) = item.as_object() {
                            headers
                                .iter()
                                .map(|&header| match obj.get(header) {
                                    Some(Value::String(s)) => s.clone(),
                                    Some(Value::Number(n)) => n.to_string(),
                                    Some(Value::Bool(b)) => b.to_string(),
                                    Some(Value::Null) => "".to_string(),
                                    Some(other) => other.to_string(),
                                    None => "".to_string(),
                                })
                                .collect()
                        } else {
                            vec![]
                        }
                    })
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// Generate CSV text from JSON data for clipboard operations
    pub fn generate_csv_text(data: &[Value]) -> Option<String> {
        let first_row = data.first()?;
        let obj = first_row.as_object()?;
        let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();

        // Create CSV format
        let mut csv_text = headers.join(",") + "\n";

        for row in data {
            if let Some(obj) = row.as_object() {
                let values: Vec<String> = headers
                    .iter()
                    .map(|&header| match obj.get(header) {
                        Some(Value::String(s)) => Self::escape_csv_field(s),
                        Some(Value::Number(n)) => n.to_string(),
                        Some(Value::Bool(b)) => b.to_string(),
                        Some(Value::Null) => String::new(),
                        Some(other) => Self::escape_csv_field(&other.to_string()),
                        None => String::new(),
                    })
                    .collect();
                csv_text.push_str(&values.join(","));
                csv_text.push('\n');
            }
        }

        Some(csv_text)
    }

    /// Generate TSV (Tab-Separated Values) text from JSON data for better Windows clipboard compatibility
    pub fn generate_tsv_text(data: &[Value]) -> Option<String> {
        let first_row = data.first()?;
        let obj = first_row.as_object()?;
        let headers: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();

        // Create TSV format with headers
        let mut tsv_text = headers.join("\t") + "\r\n";

        for row in data {
            if let Some(obj) = row.as_object() {
                let values: Vec<String> = headers
                    .iter()
                    .map(|&header| match obj.get(header) {
                        Some(Value::String(s)) => {
                            s.replace('\t', "    ").replace('\n', " ").replace('\r', "")
                        }
                        Some(Value::Number(n)) => n.to_string(),
                        Some(Value::Bool(b)) => b.to_string(),
                        Some(Value::Null) => String::new(),
                        Some(other) => other
                            .to_string()
                            .replace('\t', "    ")
                            .replace('\n', " ")
                            .replace('\r', ""),
                        None => String::new(),
                    })
                    .collect();
                tsv_text.push_str(&values.join("\t"));
                tsv_text.push_str("\r\n");
            }
        }

        Some(tsv_text)
    }

    /// V50: Helper to convert DataTable to JSON Values for export compatibility
    fn datatable_to_json_values(datatable: &crate::data::datatable::DataTable) -> Vec<Value> {
        use serde_json::json;

        let headers = datatable.column_names();
        let mut result = Vec::new();

        for row_data in datatable.to_string_table() {
            let mut obj = serde_json::Map::new();
            for (i, header) in headers.iter().enumerate() {
                if let Some(value) = row_data.get(i) {
                    obj.insert(header.clone(), json!(value));
                }
            }
            result.push(Value::Object(obj));
        }

        result
    }
}
