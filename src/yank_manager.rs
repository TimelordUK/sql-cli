use crate::app_state_container::AppStateContainer;
use crate::buffer::BufferAPI;
use crate::data_exporter::DataExporter;
use anyhow::{anyhow, Result};
use serde_json::Value;
use tracing::trace;

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
        // Prefer DataView when available (handles filtering)
        let (value, header, actual_row_index) = if let Some(dataview) = buffer.get_dataview() {
            trace!(
                "yank_cell: Using DataView for cell at visual_row={}, col={}",
                row_index,
                column_index
            );

            // The row_index here is the visual row index (e.g., row 0 in filtered view)
            // DataView's get_cell_value already handles this correctly
            let value = dataview
                .get_cell_value(row_index, column_index)
                .unwrap_or_else(|| "NULL".to_string());

            let headers = dataview.column_names();
            let header = headers
                .get(column_index)
                .ok_or_else(|| anyhow!("Column index out of bounds"))?
                .clone();

            // Get the actual data row index from the filtered view
            // When filtered, we need to translate visual row to actual data row
            let actual_row =
                if let Some(filtered_idx) = dataview.visible_row_indices().get(row_index) {
                    *filtered_idx
                } else {
                    row_index
                };

            (value, header, actual_row)
        } else if let Some(datatable) = buffer.get_datatable() {
            trace!(
                "yank_cell: Using DataTable for cell at row={}, col={}",
                row_index,
                column_index
            );

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

            (value, header, row_index)
        } else {
            return Err(anyhow!("No data available"));
        };

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
            actual_row_index,
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
        // Prefer DataView when available (handles filtering)
        let (row_data, actual_row_index) = if let Some(dataview) = buffer.get_dataview() {
            trace!("yank_row: Using DataView for row {}", row_index);
            let data = dataview
                .get_row_values(row_index)
                .ok_or_else(|| anyhow!("Row index out of bounds"))?;

            // Get the actual data row index from the filtered view
            let actual_row =
                if let Some(filtered_idx) = dataview.visible_row_indices().get(row_index) {
                    *filtered_idx
                } else {
                    row_index
                };

            (data, actual_row)
        } else if let Some(datatable) = buffer.get_datatable() {
            trace!("yank_row: Using DataTable for row {}", row_index);
            let data = datatable
                .get_row_as_strings(row_index)
                .ok_or_else(|| anyhow!("Row index out of bounds"))?;
            (data, row_index)
        } else {
            return Err(anyhow!("No data available"));
        };

        // Convert row to tab-separated text
        let row_text = row_data.join("\t");

        // Count values for preview
        let num_values = row_data.len();

        // Copy to clipboard using AppStateContainer
        let clipboard_len = row_text.len();
        state_container.yank_row(
            actual_row_index,
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
        // Prefer DataView when available (handles filtering)
        let (column_values, header) = if let Some(dataview) = buffer.get_dataview() {
            let headers = dataview.column_names();
            let header = headers
                .get(column_index)
                .ok_or_else(|| anyhow!("Column index out of bounds"))?
                .clone();

            trace!(
                "yank_column: Using DataView for column {} ({}), visible rows: {}",
                column_index,
                header,
                dataview.row_count()
            );

            let values = dataview.get_column_values(column_index);
            (values, header)
        } else if let Some(datatable) = buffer.get_datatable() {
            // Fall back to DataTable for legacy buffers
            let headers = datatable.column_names();
            let header = headers
                .get(column_index)
                .ok_or_else(|| anyhow!("Column index out of bounds"))?
                .clone();

            trace!(
                "yank_column: Using DataTable for column {} ({}), total rows: {}",
                column_index,
                header,
                datatable.row_count()
            );

            // Check if fuzzy filter is active
            let mut column_values = Vec::new();
            if buffer.is_fuzzy_filter_active() {
                let filtered_indices = buffer.get_fuzzy_filter_indices();
                trace!(
                    "yank_column: Filter active, yanking {} filtered rows",
                    filtered_indices.len()
                );

                for &row_idx in filtered_indices {
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
            } else {
                trace!(
                    "yank_column: No filter, yanking all {} rows",
                    datatable.row_count()
                );

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
            }

            (column_values, header)
        } else {
            return Err(anyhow!("No data available"));
        };

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
        // Prefer DataView when available (which handles filtering/sorting)
        let tsv_text = if let Some(dataview) = buffer.get_dataview() {
            // Use DataView's built-in TSV export
            dataview.to_tsv()?
        } else if let Some(datatable) = buffer.get_datatable() {
            // Fall back to DataTable for legacy buffers
            let data = Self::datatable_to_json(datatable)?;
            DataExporter::generate_tsv_text(&data)
                .ok_or_else(|| anyhow!("Failed to generate TSV"))?
        } else {
            return Err(anyhow!("No data available"));
        };

        // Copy to clipboard using AppStateContainer
        let clipboard_len = tsv_text.len();

        // Create preview based on what data source we used
        let (row_count, col_count, filter_info) = if let Some(dataview) = buffer.get_dataview() {
            // Get counts from DataView
            let rows = dataview.row_count();
            let cols = dataview.column_count();
            let filtered = dataview.has_filter();
            (rows, cols, if filtered { " (filtered)" } else { "" })
        } else if let Some(datatable) = buffer.get_datatable() {
            // Get counts from DataTable
            let rows = datatable.row_count();
            let cols = datatable.column_count();
            (rows, cols, "")
        } else {
            (0, 0, "")
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
}
