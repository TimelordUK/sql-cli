use crate::api_client::QueryResponse;
use crate::csv_datasource::CsvApiClient;
use crate::datasource_trait::{DataSource, DataSourceQueryResponse};
use anyhow::Result;
use std::collections::HashMap;

/// Adapter to make CsvApiClient implement DataSource trait
pub struct CsvDataSourceAdapter {
    client: CsvApiClient,
}

impl CsvDataSourceAdapter {
    pub fn new(client: CsvApiClient) -> Self {
        Self { client }
    }

    pub fn from_csv_path(path: &str, table_name: &str) -> Result<Self> {
        let mut client = CsvApiClient::new();
        client.load_csv(path, table_name)?;
        Ok(Self { client })
    }

    pub fn from_json_path(path: &str, table_name: &str) -> Result<Self> {
        let mut client = CsvApiClient::new();
        client.load_json(path, table_name)?;
        Ok(Self { client })
    }

    /// Get access to the underlying CSV client if needed
    pub fn inner(&self) -> &CsvApiClient {
        &self.client
    }

    pub fn inner_mut(&mut self) -> &mut CsvApiClient {
        &mut self.client
    }
}

impl DataSource for CsvDataSourceAdapter {
    fn query(&self, sql: &str) -> Result<DataSourceQueryResponse> {
        let response = self.client.query_csv(sql)?;
        Ok(convert_query_response(response))
    }

    fn query_with_options(
        &self,
        sql: &str,
        case_insensitive: bool,
    ) -> Result<DataSourceQueryResponse> {
        // Temporarily set case insensitive for this query
        let mut temp_client = self.client.clone();
        temp_client.set_case_insensitive(case_insensitive);
        let response = temp_client.query_csv(sql)?;
        Ok(convert_query_response(response))
    }

    fn get_schema(&self) -> Option<HashMap<String, Vec<String>>> {
        self.client.get_schema()
    }

    fn get_table_name(&self) -> String {
        // Get first table name from schema, or default
        self.get_schema()
            .and_then(|schema| schema.keys().next().cloned())
            .unwrap_or_else(|| "data".to_string())
    }

    fn get_row_count(&self) -> usize {
        // This is a bit hacky but works for now
        // In the future, CsvApiClient should expose row count directly
        self.query("SELECT * FROM data")
            .map(|r| r.count)
            .unwrap_or(0)
    }

    fn is_case_insensitive(&self) -> bool {
        // CsvApiClient doesn't expose this, so we track it separately
        // For now, return false as default
        false
    }

    fn set_case_insensitive(&mut self, case_insensitive: bool) {
        self.client.set_case_insensitive(case_insensitive);
    }

    fn clone_box(&self) -> Box<dyn DataSource> {
        Box::new(Self {
            client: self.client.clone(),
        })
    }
}

/// Convert CsvApiClient's QueryResponse to our DataSourceQueryResponse
fn convert_query_response(response: QueryResponse) -> DataSourceQueryResponse {
    // Extract columns from the first row if available
    let columns = if let Some(first_row) = response.data.first() {
        if let Some(obj) = first_row.as_object() {
            obj.keys().cloned().collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    DataSourceQueryResponse {
        data: response.data,
        count: response.count,
        columns,
        table_name: "data".to_string(), // Default table name
    }
}
