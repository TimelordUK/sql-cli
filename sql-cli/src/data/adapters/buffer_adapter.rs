//! Adapter to make Buffer implement DataProvider trait
//!
//! This adapter allows the existing Buffer to work with the new DataProvider
//! trait system without modifying the Buffer code itself.

use crate::buffer::{Buffer, BufferAPI};
use crate::data::data_provider::{DataProvider, DataType};
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Adapter that makes Buffer implement DataProvider
pub struct BufferAdapter<'a> {
    buffer: &'a Buffer,
    /// Cached column types (lazy initialized, thread-safe)
    column_types: Arc<Mutex<Option<Vec<DataType>>>>,
}

impl<'a> BufferAdapter<'a> {
    /// Create a new BufferAdapter wrapping a Buffer
    pub fn new(buffer: &'a Buffer) -> Self {
        Self {
            buffer,
            column_types: Arc::new(Mutex::new(None)),
        }
    }

    /// Detect column types by sampling data
    fn detect_column_types(&self) -> Vec<DataType> {
        let column_count = self.get_column_count();
        let mut column_types = vec![DataType::Unknown; column_count];

        // Sample first 100 rows to determine types
        let sample_size = 100.min(self.get_row_count());
        if sample_size == 0 {
            return column_types;
        }

        // Count type occurrences for each column
        let mut type_counts: Vec<std::collections::HashMap<DataType, usize>> =
            vec![std::collections::HashMap::new(); column_count];

        for row_idx in 0..sample_size {
            if let Some(row) = self.get_row(row_idx) {
                for (col_idx, value) in row.iter().enumerate() {
                    if col_idx >= column_count {
                        break;
                    }

                    let detected_type = Self::detect_value_type(value);
                    *type_counts[col_idx].entry(detected_type).or_insert(0) += 1;
                }
            }
        }

        // Determine column type based on majority
        for (col_idx, counts) in type_counts.iter().enumerate() {
            if counts.is_empty() {
                continue;
            }

            // If more than 90% of values are the same type, use that type
            let total: usize = counts.values().sum();
            let mut best_type = DataType::Mixed;

            for (dtype, count) in counts {
                if *count as f64 / total as f64 > 0.9 {
                    best_type = *dtype;
                    break;
                }
            }

            // Special case: if we have both Integer and Float, use Float
            if counts.contains_key(&DataType::Integer) && counts.contains_key(&DataType::Float) {
                best_type = DataType::Float;
            }

            column_types[col_idx] = best_type;
        }

        column_types
    }

    /// Detect the type of a single value
    fn detect_value_type(value: &str) -> DataType {
        if value.is_empty() {
            return DataType::Unknown;
        }

        // Check boolean
        if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
            return DataType::Boolean;
        }

        // Check integer
        if value.parse::<i64>().is_ok() {
            return DataType::Integer;
        }

        // Check float
        if value.parse::<f64>().is_ok() {
            return DataType::Float;
        }

        // Check date (simple heuristic - contains dash or slash with numbers)
        if value.len() >= 8 && value.len() <= 30 {
            if (value.contains('-') || value.contains('/'))
                && value.chars().any(|c| c.is_ascii_digit())
            {
                // More thorough date check could go here
                return DataType::Date;
            }
        }

        DataType::Text
    }

    /// Helper to convert JSON value to Vec<String>
    fn json_to_row(&self, json_value: &serde_json::Value) -> Vec<String> {
        if let Some(obj) = json_value.as_object() {
            // Get column names to ensure consistent ordering
            let columns = self.get_column_names();
            columns
                .iter()
                .map(|col| {
                    obj.get(col)
                        .map(|v| {
                            // Convert JSON value to string
                            match v {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Null => String::new(),
                                other => other.to_string(),
                            }
                        })
                        .unwrap_or_default()
                })
                .collect()
        } else {
            // Fallback for non-object JSON values
            vec![json_value.to_string()]
        }
    }
}

impl<'a> Debug for BufferAdapter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferAdapter")
            .field("row_count", &self.get_row_count())
            .field("column_count", &self.get_column_count())
            .finish()
    }
}

impl<'a> DataProvider for BufferAdapter<'a> {
    fn get_row(&self, index: usize) -> Option<Vec<String>> {
        // V48: Try to use DataTable first for better performance
        if let Some(datatable) = self.buffer.get_datatable() {
            debug!("V48: Using DataTable for get_row({})", index);

            // Check if fuzzy filter is active
            if self.buffer.is_fuzzy_filter_active() {
                let fuzzy_indices = self.buffer.get_fuzzy_filter_indices();
                // Map the display index to the actual data index
                let actual_index = fuzzy_indices.get(index).copied()?;

                // Get row from DataTable
                if actual_index < datatable.row_count() {
                    let row = &datatable.rows[actual_index];
                    return Some(row.values.iter().map(|v| v.to_string()).collect());
                }
            } else if let Some(filtered_data) = self.buffer.get_filtered_data() {
                // Regex filter is active - use filtered data (still string-based for now)
                return filtered_data.get(index).cloned();
            } else {
                // Normal path - get row directly from DataTable
                if index < datatable.row_count() {
                    let row = &datatable.rows[index];
                    return Some(row.values.iter().map(|v| v.to_string()).collect());
                }
            }
        }

        // Fallback to JSON if no DataTable
        debug!("V48: Falling back to JSON for get_row({})", index);

        // Check if fuzzy filter is active
        if self.buffer.is_fuzzy_filter_active() {
            let fuzzy_indices = self.buffer.get_fuzzy_filter_indices();
            // Map the display index to the actual data index
            let actual_index = fuzzy_indices.get(index).copied()?;

            // Get the row at the actual index
            self.buffer.get_results().and_then(|results| {
                results
                    .data
                    .get(actual_index)
                    .map(|json_value| self.json_to_row(json_value))
            })
        } else if let Some(filtered_data) = self.buffer.get_filtered_data() {
            // Regex filter is active - use filtered data
            filtered_data.get(index).cloned()
        } else {
            // Normal path - get row directly from results
            self.buffer.get_results().and_then(|results| {
                results
                    .data
                    .get(index)
                    .map(|json_value| self.json_to_row(json_value))
            })
        }
    }

    fn get_column_names(&self) -> Vec<String> {
        // V48: Use DataTable column names if available
        if let Some(datatable) = self.buffer.get_datatable() {
            debug!("V48: Using DataTable for column names");
            return datatable.column_names();
        }

        // Fallback to buffer's method (which uses JSON)
        self.buffer.get_column_names()
    }

    fn get_row_count(&self) -> usize {
        // If fuzzy filter is active, return the filtered count
        if self.buffer.is_fuzzy_filter_active() {
            return self.buffer.get_fuzzy_filter_indices().len();
        } else if let Some(filtered_data) = self.buffer.get_filtered_data() {
            // Regex filter is active - return filtered count
            return filtered_data.len();
        }

        // V48: Use DataTable row count if available
        if let Some(datatable) = self.buffer.get_datatable() {
            debug!("V48: Using DataTable for row count");
            return datatable.row_count();
        }

        // Fallback to JSON
        self.buffer.get_results().map(|r| r.data.len()).unwrap_or(0)
    }

    fn get_column_count(&self) -> usize {
        self.get_column_names().len()
    }

    fn get_column_type(&self, column_index: usize) -> DataType {
        // V48: Use DataTable column types if available
        if let Some(datatable) = self.buffer.get_datatable() {
            if let Some(column) = datatable.columns.get(column_index) {
                debug!(
                    "V48: Using DataTable column type for column {}",
                    column_index
                );
                // Convert DataTable's DataType to DataProvider's DataType
                return match &column.data_type {
                    crate::data::datatable::DataType::String => DataType::Text,
                    crate::data::datatable::DataType::Integer => DataType::Integer,
                    crate::data::datatable::DataType::Float => DataType::Float,
                    crate::data::datatable::DataType::Boolean => DataType::Boolean,
                    crate::data::datatable::DataType::DateTime => DataType::Date,
                    crate::data::datatable::DataType::Null => DataType::Unknown,
                    crate::data::datatable::DataType::Mixed => DataType::Text,
                };
            }
        }

        // Fallback to detection
        let mut types_guard = self.column_types.lock().unwrap();
        if types_guard.is_none() {
            *types_guard = Some(self.detect_column_types());
        }

        types_guard
            .as_ref()
            .and_then(|types| types.get(column_index))
            .copied()
            .unwrap_or(DataType::Unknown)
    }

    fn get_column_types(&self) -> Vec<DataType> {
        // V48: Use DataTable column types if available
        if let Some(datatable) = self.buffer.get_datatable() {
            debug!("V48: Using DataTable for all column types");
            return datatable
                .columns
                .iter()
                .map(|column| match &column.data_type {
                    crate::data::datatable::DataType::String => DataType::Text,
                    crate::data::datatable::DataType::Integer => DataType::Integer,
                    crate::data::datatable::DataType::Float => DataType::Float,
                    crate::data::datatable::DataType::Boolean => DataType::Boolean,
                    crate::data::datatable::DataType::DateTime => DataType::Date,
                    crate::data::datatable::DataType::Null => DataType::Unknown,
                    crate::data::datatable::DataType::Mixed => DataType::Text,
                })
                .collect();
        }

        // Fallback to detection
        let mut types_guard = self.column_types.lock().unwrap();
        if types_guard.is_none() {
            *types_guard = Some(self.detect_column_types());
        }

        types_guard
            .as_ref()
            .cloned()
            .unwrap_or_else(|| vec![DataType::Unknown; self.get_column_count()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_client::{QueryInfo, QueryResponse};
    use crate::buffer::Buffer;
    use serde_json::json;

    #[test]
    fn test_buffer_adapter_basic() {
        // Create a buffer with test data
        let mut buffer = Buffer::new(0);

        // Create test query response
        let response = QueryResponse {
            query: QueryInfo {
                select: vec!["*".to_string()],
                where_clause: None,
                order_by: None,
            },
            data: vec![
                json!({
                    "id": 1,
                    "name": "Alice",
                    "age": 30
                }),
                json!({
                    "id": 2,
                    "name": "Bob",
                    "age": 25
                }),
            ],
            count: 2,
            source: Some("test".to_string()),
            table: Some("test".to_string()),
            cached: Some(false),
        };

        buffer.set_results(Some(response));

        // Create adapter
        let adapter = BufferAdapter::new(&buffer);

        // Test DataProvider methods
        assert_eq!(adapter.get_row_count(), 2);
        assert_eq!(adapter.get_column_count(), 3);

        // Check column names contain expected values (order may vary)
        let column_names = adapter.get_column_names();
        assert!(column_names.contains(&"id".to_string()));
        assert!(column_names.contains(&"name".to_string()));
        assert!(column_names.contains(&"age".to_string()));

        // Test getting a row - values should be present but order may vary
        let row = adapter.get_row(0).unwrap();
        assert_eq!(row.len(), 3);
        assert!(row.contains(&"1".to_string()));
        assert!(row.contains(&"Alice".to_string()));
        assert!(row.contains(&"30".to_string()));

        let row = adapter.get_row(1).unwrap();
        assert_eq!(row.len(), 3);
        assert!(row.contains(&"2".to_string()));
        assert!(row.contains(&"Bob".to_string()));
        assert!(row.contains(&"25".to_string()));

        // Test out of bounds
        assert!(adapter.get_row(2).is_none());
    }

    #[test]
    fn test_buffer_adapter_empty() {
        let buffer = Buffer::new(0);
        let adapter = BufferAdapter::new(&buffer);

        assert_eq!(adapter.get_row_count(), 0);
        assert_eq!(adapter.get_column_count(), 0);
        assert!(adapter.get_row(0).is_none());
    }
}
