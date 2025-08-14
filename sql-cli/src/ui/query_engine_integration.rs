use crate::api_client::{QueryInfo, QueryResponse};
use crate::data::data_view::DataView;
use crate::data::datatable::{DataTable, DataValue};
use crate::data::query_engine::QueryEngine;
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;

/// Integration layer between QueryEngine and enhanced_tui
/// Handles conversion between DataView and QueryResponse formats
pub struct QueryEngineIntegration;

impl QueryEngineIntegration {
    /// Execute a query using QueryEngine and convert to QueryResponse format
    pub fn execute_query(table: &DataTable, query: &str) -> Result<QueryResponse> {
        // Need an Arc for QueryEngine
        let table_arc = Arc::new(table.clone());
        let engine = QueryEngine;
        let view = engine.execute(table_arc, query)?;

        // Convert DataView to QueryResponse format
        Self::dataview_to_query_response(view)
    }

    /// Execute a query with hidden columns
    pub fn execute_query_with_hidden_columns(
        table: &DataTable,
        query: &str,
        hidden_columns: &[String],
    ) -> Result<QueryResponse> {
        // Need an Arc for QueryEngine
        let table_arc = Arc::new(table.clone());
        let engine = QueryEngine;
        let mut view = engine.execute(table_arc, query)?;

        // Hide the specified columns
        for col_name in hidden_columns {
            view.hide_column_by_name(col_name);
        }

        // Convert DataView to QueryResponse format
        Self::dataview_to_query_response(view)
    }

    /// Convert a DataView to QueryResponse format for TUI compatibility
    fn dataview_to_query_response(view: DataView) -> Result<QueryResponse> {
        let mut rows = Vec::new();
        let column_names = view.column_names();

        // Get all rows from the view
        let data_rows = view.get_rows();

        // Convert each DataRow to JSON
        for row in data_rows {
            let mut row_obj = serde_json::Map::new();

            for (col_idx, col_name) in column_names.iter().enumerate() {
                let value = row.values.get(col_idx);
                let json_value = match value {
                    Some(data_value) => Self::datavalue_to_json(data_value),
                    None => Value::Null,
                };
                row_obj.insert(col_name.clone(), json_value);
            }

            rows.push(Value::Object(row_obj));
        }

        Ok(QueryResponse {
            data: rows.clone(),
            count: rows.len(),
            query: QueryInfo {
                select: vec![],
                where_clause: None,
                order_by: None,
            },
            source: Some("QueryEngine".to_string()),
            table: None,
            cached: Some(false),
        })
    }

    /// Execute query and return DataView directly (for future optimized path)
    pub fn execute_to_view(table: Arc<DataTable>, query: &str) -> Result<DataView> {
        let engine = QueryEngine;
        engine.execute(table, query)
    }

    /// Convert DataValue to JSON Value
    fn datavalue_to_json(value: &DataValue) -> Value {
        match value {
            DataValue::String(s) => json!(s),
            DataValue::Integer(i) => json!(i),
            DataValue::Float(f) => json!(f),
            DataValue::Boolean(b) => json!(b),
            DataValue::DateTime(dt) => json!(dt),
            DataValue::Null => Value::Null,
        }
    }
}
