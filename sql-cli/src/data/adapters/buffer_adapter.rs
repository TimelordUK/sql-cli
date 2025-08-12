//! Adapter to make Buffer implement DataProvider trait
//!
//! This adapter allows the existing Buffer to work with the new DataProvider
//! trait system without modifying the Buffer code itself.

use crate::buffer::{Buffer, BufferAPI};
use crate::data::data_provider::DataProvider;
use std::fmt::Debug;

/// Adapter that makes Buffer implement DataProvider
pub struct BufferAdapter<'a> {
    buffer: &'a Buffer,
}

impl<'a> BufferAdapter<'a> {
    /// Create a new BufferAdapter wrapping a Buffer
    pub fn new(buffer: &'a Buffer) -> Self {
        Self { buffer }
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
        // Get results from buffer
        self.buffer.get_results().and_then(|results| {
            results.data.get(index).map(|json_value| {
                // Convert JSON value to Vec<String>
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
            })
        })
    }

    fn get_column_names(&self) -> Vec<String> {
        self.buffer.get_column_names()
    }

    fn get_row_count(&self) -> usize {
        self.buffer.get_results().map(|r| r.data.len()).unwrap_or(0)
    }

    fn get_column_count(&self) -> usize {
        self.get_column_names().len()
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
