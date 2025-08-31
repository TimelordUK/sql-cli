//! Adapter to make CsvApiClient implement DataProvider trait
//!
//! This adapter allows the existing CsvApiClient to work with the new DataProvider
//! trait system without modifying the CsvApiClient code itself.

use crate::api_client::QueryResponse;
use crate::data::csv_datasource::CsvApiClient;
use crate::data::data_provider::DataProvider;
use std::fmt::Debug;

/// Adapter that makes CsvApiClient implement DataProvider
/// Note: This adapter requires querying the data first since CsvApiClient
/// doesn't store results internally - it generates them on query
pub struct CsvClientAdapter<'a> {
    client: &'a CsvApiClient,
    cached_response: Option<QueryResponse>,
}

impl<'a> CsvClientAdapter<'a> {
    /// Create a new CsvClientAdapter wrapping a CsvApiClient
    /// You should call execute_query() to populate data before using DataProvider methods
    pub fn new(client: &'a CsvApiClient) -> Self {
        Self {
            client,
            cached_response: None,
        }
    }

    /// Execute a query and cache the results for DataProvider access
    pub fn execute_query(&mut self, sql: &str) -> anyhow::Result<()> {
        let response = self.client.query_csv(sql)?;
        self.cached_response = Some(response);
        Ok(())
    }
}

impl<'a> Debug for CsvClientAdapter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CsvClientAdapter")
            .field("row_count", &self.get_row_count())
            .field("column_count", &self.get_column_count())
            .field("has_data", &self.cached_response.is_some())
            .finish()
    }
}

impl<'a> DataProvider for CsvClientAdapter<'a> {
    fn get_row(&self, index: usize) -> Option<Vec<String>> {
        self.cached_response.as_ref().and_then(|response| {
            response.data.get(index).map(|json_value| {
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
        // Try to get from cached response first
        if let Some(ref response) = self.cached_response {
            // Extract column names from first data row if available
            if let Some(first_row) = response.data.first() {
                if let Some(obj) = first_row.as_object() {
                    return obj.keys().map(|k| k.to_string()).collect();
                }
            }
        }

        // Fallback to schema if no data
        self.client
            .get_schema()
            .and_then(|schema| schema.values().next().map(|headers| headers.clone()))
            .unwrap_or_default()
    }

    fn get_row_count(&self) -> usize {
        self.cached_response
            .as_ref()
            .map(|r| r.data.len())
            .unwrap_or(0)
    }

    fn get_column_count(&self) -> usize {
        self.get_column_names().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::csv_datasource::CsvApiClient;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_csv_client_adapter_basic() {
        // Create a CsvApiClient with test data
        let temp_dir = tempdir().unwrap();
        let json_path = temp_dir.path().join("test_data.json");

        let test_data = json!([
            {
                "id": 1,
                "name": "Alice",
                "age": 30
            },
            {
                "id": 2,
                "name": "Bob",
                "age": 25
            },
            {
                "id": 3,
                "name": "Charlie",
                "age": 35
            }
        ]);

        fs::write(&json_path, serde_json::to_string(&test_data).unwrap()).unwrap();

        let mut client = CsvApiClient::new();
        client.load_json(&json_path, "test").unwrap();

        // Create adapter and execute query
        let mut adapter = CsvClientAdapter::new(&client);
        adapter.execute_query("SELECT * FROM test").unwrap();

        // Test DataProvider methods
        assert_eq!(adapter.get_row_count(), 3);
        assert_eq!(adapter.get_column_count(), 3);

        let col_names = adapter.get_column_names();
        assert!(col_names.contains(&"id".to_string()));
        assert!(col_names.contains(&"name".to_string()));
        assert!(col_names.contains(&"age".to_string()));

        // Test getting rows
        let row = adapter.get_row(0).unwrap();
        assert!(row.contains(&"1".to_string()));
        assert!(row.contains(&"Alice".to_string()));
        assert!(row.contains(&"30".to_string()));

        let row = adapter.get_row(2).unwrap();
        assert!(row.contains(&"3".to_string()));
        assert!(row.contains(&"Charlie".to_string()));
        assert!(row.contains(&"35".to_string()));

        // Test out of bounds
        assert!(adapter.get_row(3).is_none());
    }

    #[test]
    fn test_csv_client_adapter_empty() {
        let client = CsvApiClient::new();
        let adapter = CsvClientAdapter::new(&client);

        // Without executing a query, should return empty data
        assert_eq!(adapter.get_row_count(), 0);
        assert_eq!(adapter.get_column_count(), 0);
        assert!(adapter.get_row(0).is_none());
    }

    #[test]
    fn test_csv_client_adapter_with_filter() {
        // Create a CsvApiClient with test data
        let temp_dir = tempdir().unwrap();
        let json_path = temp_dir.path().join("test_data.json");

        let test_data = json!([
            {
                "id": 1,
                "name": "Alice",
                "status": "active"
            },
            {
                "id": 2,
                "name": "Bob",
                "status": "inactive"
            },
            {
                "id": 3,
                "name": "Charlie",
                "status": "active"
            }
        ]);

        fs::write(&json_path, serde_json::to_string(&test_data).unwrap()).unwrap();

        let mut client = CsvApiClient::new();
        client.load_json(&json_path, "test").unwrap();

        // Create adapter and execute filtered query
        let mut adapter = CsvClientAdapter::new(&client);
        adapter
            .execute_query("SELECT * FROM test WHERE status = 'active'")
            .unwrap();

        // Should only have 2 active rows
        assert_eq!(adapter.get_row_count(), 2);

        // Check that both rows are active status
        for i in 0..adapter.get_row_count() {
            let row = adapter.get_row(i).unwrap();
            assert!(row.contains(&"active".to_string()));
        }
    }
}
