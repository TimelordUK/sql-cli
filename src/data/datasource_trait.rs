use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Response from a query operation
#[derive(Debug, Clone)]
pub struct DataSourceQueryResponse {
    pub data: Vec<Value>,
    pub count: usize,
    pub columns: Vec<String>,
    pub table_name: String,
}

/// Trait for abstracting data sources (CSV, JSON, Database, etc.)
/// This allows the TUI to work with data without knowing the specific source
pub trait DataSource: Send + Sync {
    /// Execute a SQL-like query against the data source
    fn query(&self, sql: &str) -> Result<DataSourceQueryResponse>;

    /// Execute a query with case-insensitive matching
    fn query_with_options(
        &self,
        sql: &str,
        case_insensitive: bool,
    ) -> Result<DataSourceQueryResponse>;

    /// Get the schema (table names and their columns)
    fn get_schema(&self) -> Option<HashMap<String, Vec<String>>>;

    /// Get the primary table name
    fn get_table_name(&self) -> String;

    /// Get total row count (unfiltered)
    fn get_row_count(&self) -> usize;

    /// Check if data source is case-insensitive
    fn is_case_insensitive(&self) -> bool;

    /// Set case sensitivity
    fn set_case_insensitive(&mut self, case_insensitive: bool);

    /// Clone the data source into a boxed trait object
    fn clone_box(&self) -> Box<dyn DataSource>;
}

/// Helper trait for converting to DataTable
pub trait ToDataTable {
    /// Convert the data source to a DataTable structure
    fn to_datatable(&self) -> Result<crate::datatable::DataTable>;
}
