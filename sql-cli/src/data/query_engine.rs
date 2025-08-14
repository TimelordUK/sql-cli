use anyhow::Result;
use std::sync::Arc;
use tracing::debug;

use crate::data::data_view::DataView;
use crate::data::datatable::DataTable;
use crate::data::recursive_where_evaluator::RecursiveWhereEvaluator;
use crate::sql::recursive_parser::{OrderByColumn, Parser, SelectStatement, SortDirection};

/// Query engine that executes SQL directly on DataTable
pub struct QueryEngine;

impl QueryEngine {
    pub fn new() -> Self {
        Self
    }

    /// Execute a SQL query on a DataTable and return a DataView
    pub fn execute(&self, table: Arc<DataTable>, sql: &str) -> Result<DataView> {
        // Parse the SQL query
        let mut parser = Parser::new(sql);
        let statement = parser
            .parse()
            .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

        // Convert SelectStatement to DataView operations
        self.build_view(table, statement)
    }

    /// Build a DataView from a parsed SQL statement
    fn build_view(&self, table: Arc<DataTable>, statement: SelectStatement) -> Result<DataView> {
        // Start with a view that shows all data
        let mut view = DataView::new(table.clone());

        // Apply WHERE clause filtering using recursive evaluator
        if let Some(where_clause) = &statement.where_clause {
            let total_rows = table.row_count();
            debug!("QueryEngine: Applying WHERE clause to {} rows", total_rows);
            debug!("QueryEngine: WHERE clause = {:?}", where_clause);

            view = view.filter(|table, row_idx| {
                debug!("QueryEngine: About to create RecursiveWhereEvaluator for row {}", row_idx);
                let evaluator = RecursiveWhereEvaluator::new(table);
                debug!("QueryEngine: Created RecursiveWhereEvaluator, about to call evaluate() for row {}", row_idx);
                match evaluator.evaluate(where_clause, row_idx) {
                    Ok(result) => {
                        if row_idx < 5 {
                            debug!("QueryEngine: Row {} WHERE result: {}", row_idx, result);
                        }
                        result
                    }
                    Err(e) => {
                        debug!(
                            "QueryEngine: WHERE evaluation error for row {}: {}",
                            row_idx, e
                        );
                        false
                    }
                }
            });

            debug!(
                "QueryEngine: After WHERE filtering, {} rows remain",
                view.row_count()
            );
        }

        // Apply column projection (SELECT clause) - do this AFTER filtering
        if !statement.columns.is_empty() && statement.columns[0] != "*" {
            let column_indices = self.resolve_column_indices(view.source(), &statement.columns)?;
            view = view.with_columns(column_indices);
        }

        // Apply ORDER BY sorting
        if let Some(order_by_columns) = &statement.order_by {
            if !order_by_columns.is_empty() {
                view = self.apply_order_by(view, &order_by_columns[0])?;
            }
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

    /// Apply ORDER BY sorting to the view
    fn apply_order_by(&self, view: DataView, order_by: &OrderByColumn) -> Result<DataView> {
        // Get column index
        let col_index = view
            .source()
            .get_column_index(&order_by.column)
            .ok_or_else(|| anyhow::anyhow!("Column '{}' not found", order_by.column))?;

        // Apply sorting
        let ascending = matches!(order_by.direction, SortDirection::Asc);
        view.sort_by(col_index, ascending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::datatable::{DataColumn, DataRow, DataValue};

    fn create_test_table() -> Arc<DataTable> {
        let mut table = DataTable::new("test");

        // Add columns
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("name"));
        table.add_column(DataColumn::new("age"));

        // Add rows
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Alice".to_string()),
                DataValue::Integer(30),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Bob".to_string()),
                DataValue::Integer(25),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Charlie".to_string()),
                DataValue::Integer(35),
            ]))
            .unwrap();

        Arc::new(table)
    }

    #[test]
    fn test_select_all() {
        let table = create_test_table();
        let engine = QueryEngine::new();

        let view = engine
            .execute(table.clone(), "SELECT * FROM users")
            .unwrap();
        assert_eq!(view.row_count(), 3);
        assert_eq!(view.column_count(), 3);
    }

    #[test]
    fn test_select_columns() {
        let table = create_test_table();
        let engine = QueryEngine::new();

        let view = engine
            .execute(table.clone(), "SELECT name, age FROM users")
            .unwrap();
        assert_eq!(view.row_count(), 3);
        assert_eq!(view.column_count(), 2);
    }

    #[test]
    fn test_select_with_limit() {
        let table = create_test_table();
        let engine = QueryEngine::new();

        let view = engine
            .execute(table.clone(), "SELECT * FROM users LIMIT 2")
            .unwrap();
        assert_eq!(view.row_count(), 2);
    }

    #[test]
    fn test_type_coercion_contains() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("status"));
        table.add_column(DataColumn::new("price"));

        // Add test data with mixed types
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Pending".to_string()),
                DataValue::Float(99.99),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Confirmed".to_string()),
                DataValue::Float(150.50),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Pending".to_string()),
                DataValue::Float(75.00),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing WHERE clause with Contains ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let status = table.get_value(i, 1);
            println!("Row {}: status = {:?}", i, status);
        }

        // Test 1: Basic string contains (should work)
        println!("\n--- Test 1: status.Contains('pend') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE status.Contains('pend')",
        );
        match result {
            Ok(view) => {
                println!("SUCCESS: Found {} matching rows", view.row_count());
                assert_eq!(view.row_count(), 2); // Should find both Pending rows
            }
            Err(e) => {
                panic!("Query failed: {}", e);
            }
        }

        // Test 2: Numeric contains (should work with type coercion)
        println!("\n--- Test 2: price.Contains('9') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE price.Contains('9')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} matching rows with price containing '9'",
                    view.row_count()
                );
                // Should find 99.99 row
                assert!(view.row_count() >= 1);
            }
            Err(e) => {
                panic!("Numeric coercion query failed: {}", e);
            }
        }

        println!("\n=== All tests passed! ===");
    }
}
