//! Adapter to make CsvApiClient implement DataProvider trait
//!
//! This adapter allows the existing CsvApiClient to work with the new DataProvider
//! trait system without modifying the CsvApiClient code itself.

use crate::data::csv_datasource::CsvApiClient;
use crate::data::data_provider::DataProvider;
use std::fmt::Debug;

/// Adapter that makes CsvApiClient implement DataProvider
pub struct CsvClientAdapter<'a> {
    client: &'a CsvApiClient,
}

impl<'a> CsvClientAdapter<'a> {
    /// Create a new CsvClientAdapter wrapping a CsvApiClient
    pub fn new(client: &'a CsvApiClient) -> Self {
        Self { client }
    }
}

impl<'a> Debug for CsvClientAdapter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CsvClientAdapter")
            .field("row_count", &self.get_row_count())
            .field("column_count", &self.get_column_count())
            .finish()
    }
}

impl<'a> DataProvider for CsvClientAdapter<'a> {
    fn get_row(&self, index: usize) -> Option<Vec<String>> {
        // Get the current view (which respects filtering/sorting)
        let view = self.client.get_current_view();

        // Check if index is valid
        if index >= view.len() {
            return None;
        }

        // Get the actual row index from the view
        let actual_index = view[index];

        // Get the row data
        self.client.results.as_ref().and_then(|results| {
            results.data.get(actual_index).map(|json_value| {
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
        self.client
            .results
            .as_ref()
            .map(|r| r.columns.clone())
            .unwrap_or_default()
    }

    fn get_row_count(&self) -> usize {
        // Return the filtered count (what's visible)
        self.client.get_current_view().len()
    }

    fn get_column_count(&self) -> usize {
        self.get_column_names().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_client::{QueryInfo, QueryResponse};
    use crate::data::csv_datasource::CsvApiClient;
    use serde_json::json;

    #[test]
    fn test_csv_client_adapter_basic() {
        // Create a CsvApiClient with test data
        let mut client = CsvApiClient::new();

        // Create test query response
        let response = QueryResponse {
            query: QueryInfo {
                query: "SELECT * FROM test".to_string(),
                row_count: 3,
                execution_time: 0.1,
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
                json!({
                    "id": 3,
                    "name": "Charlie",
                    "age": 35
                }),
            ],
            columns: vec!["id".to_string(), "name".to_string(), "age".to_string()],
            total_count: 3,
        };

        client.results = Some(response);
        client.current_view = vec![0, 1, 2]; // All rows visible

        // Create adapter
        let adapter = CsvClientAdapter::new(&client);

        // Test DataProvider methods
        assert_eq!(adapter.get_row_count(), 3);
        assert_eq!(adapter.get_column_count(), 3);
        assert_eq!(adapter.get_column_names(), vec!["id", "name", "age"]);

        // Test getting rows
        let row = adapter.get_row(0).unwrap();
        assert_eq!(row, vec!["1", "Alice", "30"]);

        let row = adapter.get_row(2).unwrap();
        assert_eq!(row, vec!["3", "Charlie", "35"]);

        // Test out of bounds
        assert!(adapter.get_row(3).is_none());
    }

    #[test]
    fn test_csv_client_adapter_filtered() {
        // Create a CsvApiClient with test data
        let mut client = CsvApiClient::new();

        // Create test query response with 3 rows
        let response = QueryResponse {
            query: QueryInfo {
                query: "SELECT * FROM test".to_string(),
                row_count: 3,
                execution_time: 0.1,
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
                json!({
                    "id": 3,
                    "name": "Charlie",
                    "age": 35
                }),
            ],
            columns: vec!["id".to_string(), "name".to_string(), "age".to_string()],
            total_count: 3,
        };

        client.results = Some(response);
        // Simulate a filter - only rows 0 and 2 are visible
        client.current_view = vec![0, 2];

        // Create adapter
        let adapter = CsvClientAdapter::new(&client);

        // Test that we only see filtered rows
        assert_eq!(adapter.get_row_count(), 2);

        // First visible row is actually row 0
        let row = adapter.get_row(0).unwrap();
        assert_eq!(row, vec!["1", "Alice", "30"]);

        // Second visible row is actually row 2
        let row = adapter.get_row(1).unwrap();
        assert_eq!(row, vec!["3", "Charlie", "35"]);

        // Only 2 rows visible
        assert!(adapter.get_row(2).is_none());
    }
}
