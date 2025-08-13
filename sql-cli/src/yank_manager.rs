use crate::app_state_container::AppStateContainer;
use crate::buffer::BufferAPI;
use crate::data::datatable::DataTable;
use crate::data_exporter::DataExporter;
use anyhow::{anyhow, Result};
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
        state_container: &AppStateContainer,
        row_index: usize,
        column_index: usize,
    ) -> Result<YankResult> {
        let datatable = buffer
            .get_datatable()
            .ok_or_else(|| anyhow!("No DataTable available"))?;

        let row_data = datatable
            .get_row_as_strings(row_index)
            .ok_or_else(|| anyhow!("Row index out of bounds"))?;

        let headers = datatable.column_names();
        let header = headers
            .get(column_index)
            .ok_or_else(|| anyhow!("Column index out of bounds"))?
            .clone();

        let value = row_data
            .get(column_index)
            .cloned()
            .unwrap_or_else(|| "NULL".to_string());

        // Prepare display value
        let col_name = header.to_string();
        let display_value = if value.len() > 20 {
            format!("{}...", &value[..17])
        } else {
            value.clone()
        };

        // Copy to clipboard using AppStateContainer
        let clipboard_len = value.len();
        state_container.yank_cell(
            row_index,
            column_index,
            value.clone(),
            display_value.clone(),
        )?;

        Ok(YankResult {
            description: format!("{} ({} chars)", col_name, clipboard_len),
            preview: display_value,
            full_value: value,
        })
    }

    /// Yank an entire row as tab-separated values
    pub fn yank_row(
        buffer: &dyn BufferAPI,
        state_container: &AppStateContainer,
        row_index: usize,
    ) -> Result<YankResult> {
        let datatable = buffer
            .get_datatable()
            .ok_or_else(|| anyhow!("No DataTable available"))?;

        let row_data = datatable
            .get_row_as_strings(row_index)
            .ok_or_else(|| anyhow!("Row index out of bounds"))?;

        // Convert row to tab-separated text
        let row_text = row_data.join("\t");

        // Count values for preview
        let num_values = row_data.len();

        // Copy to clipboard using AppStateContainer
        let clipboard_len = row_text.len();
        state_container.yank_row(
            row_index,
            row_text.clone(),
            format!("{} values", num_values),
        )?;

        Ok(YankResult {
            description: format!("Row {} ({} chars)", row_index + 1, clipboard_len),
            preview: format!("{} values", num_values),
            full_value: row_text,
        })
    }

    /// Yank an entire column
    pub fn yank_column(
        buffer: &dyn BufferAPI,
        state_container: &AppStateContainer,
        column_index: usize,
    ) -> Result<YankResult> {
        let datatable = buffer
            .get_datatable()
            .ok_or_else(|| anyhow!("No DataTable available"))?;

        // Get header name
        let headers = datatable.column_names();
        let header = headers
            .get(column_index)
            .ok_or_else(|| anyhow!("Column index out of bounds"))?
            .clone();

        // Collect all values from the column
        let mut column_values = Vec::new();
        for row_idx in 0..datatable.row_count() {
            if let Some(row_data) = datatable.get_row_as_strings(row_idx) {
                let value = row_data
                    .get(column_index)
                    .cloned()
                    .unwrap_or_else(|| "NULL".to_string())
                    .replace('\t', "    ")
                    .replace('\n', " ")
                    .replace('\r', "");
                column_values.push(value);
            }
        }

        // Use Windows-compatible line endings (\r\n) for better clipboard compatibility
        let column_text = column_values.join("\r\n");

        let preview = if column_values.len() > 5 {
            format!("{} values", column_values.len())
        } else {
            column_values.join(", ")
        };

        // Copy to clipboard using AppStateContainer
        let clipboard_len = column_text.len();
        state_container.yank_column(
            header.to_string(),
            column_index,
            column_text.clone(),
            preview.clone(),
        )?;

        Ok(YankResult {
            description: format!("Column '{}' ({} chars)", header, clipboard_len),
            preview,
            full_value: column_text,
        })
    }

    /// Yank all data as TSV (Tab-Separated Values) for better Windows clipboard compatibility
    pub fn yank_all(
        buffer: &dyn BufferAPI,
        state_container: &AppStateContainer,
    ) -> Result<YankResult> {
        // Get the DataTable
        let datatable = buffer
            .get_datatable()
            .ok_or_else(|| anyhow!("No DataTable available"))?;

        // Determine what data to use
        let data = if buffer.is_filter_active() || buffer.is_fuzzy_filter_active() {
            // Use filtered data if available
            if let Some(filtered) = buffer.get_filtered_data() {
                // Convert string data back to JSON for TSV generation
                // This is a bit inefficient but maintains compatibility
                Self::convert_filtered_to_json(datatable, filtered)?
            } else {
                // Convert DataTable to JSON for TSV generation
                Self::datatable_to_json(datatable)?
            }
        } else {
            // Convert DataTable to JSON for TSV generation
            Self::datatable_to_json(datatable)?
        };

        // Generate TSV text for better Windows/Excel compatibility
        let tsv_text = DataExporter::generate_tsv_text(&data)
            .ok_or_else(|| anyhow!("Failed to generate TSV"))?;

        // Copy to clipboard using AppStateContainer
        let clipboard_len = tsv_text.len();

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

        // Call AppStateContainer's yank_all
        let preview = format!("{} rows × {} columns", row_count, col_count);
        state_container.yank_all(tsv_text.clone(), preview.clone())?;

        Ok(YankResult {
            description: format!("All data{} as TSV ({} chars)", filter_info, clipboard_len),
            preview,
            full_value: tsv_text,
        })
    }

    /// Helper to convert DataTable to JSON for TSV generation
    fn datatable_to_json(datatable: &crate::data::datatable::DataTable) -> Result<Vec<Value>> {
        let headers = datatable.column_names();
        let mut json_data = Vec::new();

        for row_idx in 0..datatable.row_count() {
            if let Some(row_data) = datatable.get_row_as_strings(row_idx) {
                let mut obj = serde_json::Map::new();
                for (i, header) in headers.iter().enumerate() {
                    if let Some(value) = row_data.get(i) {
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
                json_data.push(Value::Object(obj));
            }
        }

        Ok(json_data)
    }

    /// Helper to convert filtered string data back to JSON
    fn convert_filtered_to_json(
        datatable: &crate::data::datatable::DataTable,
        filtered_data: &[Vec<String>],
    ) -> Result<Vec<Value>> {
        // Get headers from DataTable
        let headers = datatable.column_names();

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
