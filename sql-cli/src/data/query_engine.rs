use anyhow::Result;
use std::sync::Arc;

use crate::data::data_view::DataView;
use crate::data::datatable::DataTable;
use crate::sql::recursive_parser::{Parser, SelectStatement};

/// Query engine that executes SQL directly on DataTable
pub struct QueryEngine {
    parser: Parser,
}

impl QueryEngine {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    /// Execute a SQL query on a DataTable and return a DataView
    pub fn execute(&self, table: Arc<DataTable>, sql: &str) -> Result<DataView> {
        // Parse the SQL query
        let statement = self.parser.parse(sql)?;
        
        // Convert SelectStatement to DataView operations
        self.build_view(table, statement)
    }

    /// Build a DataView from a parsed SQL statement
    fn build_view(&self, table: Arc<DataTable>, statement: SelectStatement) -> Result<DataView> {
        // Start with a view that shows all data
        let mut view = DataView::new(table.clone());

        // Apply column projection (SELECT clause)
        if !statement.columns.is_empty() && statement.columns[0] != "*" {
            let column_indices = self.resolve_column_indices(&table, &statement.columns)?;
            view = view.with_columns(column_indices);
        }

        // Apply WHERE clause filtering
        if let Some(where_clause) = &statement.where_clause {
            view = self.apply_where_clause(view, where_clause)?;
        }

        // Apply ORDER BY sorting
        if let Some(order_by) = &statement.order_by {
            view = self.apply_order_by(view, order_by)?;
        }

        // Apply LIMIT/OFFSET
        if let Some(limit) = statement.limit {
            let offset = statement.offset.unwrap_or(0);
            view = view.with_limit(limit, offset);
        }

        Ok(view)
    }

    /// Resolve column names to indices
    fn resolve_column_indices(&self, table: &DataTable, columns: &[String]) -> Result<Vec<usize>> {
        let mut indices = Vec::new();
        let table_columns = table.column_names();
        
        for col_name in columns {
            let index = table_columns
                .iter()
                .position(|c| c.eq_ignore_ascii_case(col_name))
                .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", col_name))?;
            indices.push(index);
        }
        
        Ok(indices)
    }

    /// Apply WHERE clause filtering to the view
    fn apply_where_clause(&self, view: DataView, where_clause: &str) -> Result<DataView> {
        // For now, we'll use the existing WHERE clause evaluator
        // This will be implemented to filter rows based on conditions
        
        // TODO: Parse WHERE clause and create filter function
        // For Phase 1, we'll just return the view unchanged
        Ok(view)
    }

    /// Apply ORDER BY sorting to the view
    fn apply_order_by(&self, view: DataView, order_by: &str) -> Result<DataView> {
        // Parse ORDER BY clause and apply sorting
        // TODO: Implement sorting based on column and direction
        
        // For Phase 1, we'll just return the view unchanged
        Ok(view)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow, DataValue};

    fn create_test_table() -> Arc<DataTable> {
        let columns = vec![
            DataColumn::new("id".to_string(), vec![
                DataValue::Integer(1),
                DataValue::Integer(2),
                DataValue::Integer(3),
            ]),
            DataColumn::new("name".to_string(), vec![
                DataValue::String("Alice".to_string()),
                DataValue::String("Bob".to_string()),
                DataValue::String("Charlie".to_string()),
            ]),
            DataColumn::new("age".to_string(), vec![
                DataValue::Integer(30),
                DataValue::Integer(25),
                DataValue::Integer(35),
            ]),
        ];
        
        Arc::new(DataTable::new(columns))
    }

    #[test]
    fn test_select_all() {
        let table = create_test_table();
        let engine = QueryEngine::new();
        
        let view = engine.execute(table.clone(), "SELECT * FROM users").unwrap();
        assert_eq!(view.row_count(), 3);
        assert_eq!(view.column_count(), 3);
    }

    #[test]
    fn test_select_columns() {
        let table = create_test_table();
        let engine = QueryEngine::new();
        
        let view = engine.execute(table.clone(), "SELECT name, age FROM users").unwrap();
        assert_eq!(view.row_count(), 3);
        assert_eq!(view.column_count(), 2);
    }

    #[test]
    fn test_select_with_limit() {
        let table = create_test_table();
        let engine = QueryEngine::new();
        
        let view = engine.execute(table.clone(), "SELECT * FROM users LIMIT 2").unwrap();
        assert_eq!(view.row_count(), 2);
    }
}