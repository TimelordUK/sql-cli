use anyhow::Result;
use std::sync::Arc;
use tracing::debug;

use crate::data::data_view::DataView;
use crate::data::datatable::DataTable;
use crate::data::recursive_where_evaluator::RecursiveWhereEvaluator;
use crate::sql::recursive_parser::{OrderByColumn, Parser, SelectStatement, SortDirection};

/// Query engine that executes SQL directly on DataTable
pub struct QueryEngine {
    case_insensitive: bool,
}

impl QueryEngine {
    pub fn new() -> Self {
        Self {
            case_insensitive: false,
        }
    }

    pub fn with_case_insensitive(case_insensitive: bool) -> Self {
        Self { case_insensitive }
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
                debug!("QueryEngine: About to create RecursiveWhereEvaluator for row {} (case_insensitive={})", row_idx, self.case_insensitive);
                let evaluator = RecursiveWhereEvaluator::with_case_insensitive(table, self.case_insensitive);
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

    #[test]
    fn test_not_in_clause() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("country"));

        // Add test data
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("CA".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("US".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("UK".to_string()),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing NOT IN clause ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let country = table.get_value(i, 1);
            println!("Row {}: country = {:?}", i, country);
        }

        // Test NOT IN clause - should exclude CA, return US and UK (2 rows)
        println!("\n--- Test: country NOT IN ('CA') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE country NOT IN ('CA')",
        );
        match result {
            Ok(view) => {
                println!("SUCCESS: Found {} rows not in ('CA')", view.row_count());
                assert_eq!(view.row_count(), 2); // Should find US and UK
            }
            Err(e) => {
                panic!("NOT IN query failed: {}", e);
            }
        }

        println!("\n=== NOT IN test complete! ===");
    }

    #[test]
    fn test_case_insensitive_in_and_not_in() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("country"));

        // Add test data with mixed case
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("CA".to_string()), // uppercase
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("us".to_string()), // lowercase
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("UK".to_string()), // uppercase
            ]))
            .unwrap();

        let table = Arc::new(table);

        println!("\n=== Testing Case-Insensitive IN clause ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let country = table.get_value(i, 1);
            println!("Row {}: country = {:?}", i, country);
        }

        // Test case-insensitive IN - should match 'CA' with 'ca'
        println!("\n--- Test: country IN ('ca') with case_insensitive=true ---");
        let engine = QueryEngine::with_case_insensitive(true);
        let result = engine.execute(table.clone(), "SELECT * FROM test WHERE country IN ('ca')");
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows matching 'ca' (case-insensitive)",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 1); // Should find CA row
            }
            Err(e) => {
                panic!("Case-insensitive IN query failed: {}", e);
            }
        }

        // Test case-insensitive NOT IN - should exclude 'CA' when searching for 'ca'
        println!("\n--- Test: country NOT IN ('ca') with case_insensitive=true ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE country NOT IN ('ca')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows not matching 'ca' (case-insensitive)",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find us and UK rows
            }
            Err(e) => {
                panic!("Case-insensitive NOT IN query failed: {}", e);
            }
        }

        // Test case-sensitive (default) - should NOT match 'CA' with 'ca'
        println!("\n--- Test: country IN ('ca') with case_insensitive=false ---");
        let engine_case_sensitive = QueryEngine::new(); // defaults to case_insensitive=false
        let result = engine_case_sensitive
            .execute(table.clone(), "SELECT * FROM test WHERE country IN ('ca')");
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows matching 'ca' (case-sensitive)",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 0); // Should find no rows (CA != ca)
            }
            Err(e) => {
                panic!("Case-sensitive IN query failed: {}", e);
            }
        }

        println!("\n=== Case-insensitive IN/NOT IN test complete! ===");
    }

    #[test]
    fn test_parentheses_in_where_clause() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("status"));
        table.add_column(DataColumn::new("priority"));

        // Add test data
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Pending".to_string()),
                DataValue::String("High".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Complete".to_string()),
                DataValue::String("High".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Pending".to_string()),
                DataValue::String("Low".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(4),
                DataValue::String("Complete".to_string()),
                DataValue::String("Low".to_string()),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing Parentheses in WHERE clause ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let status = table.get_value(i, 1);
            let priority = table.get_value(i, 2);
            println!(
                "Row {}: status = {:?}, priority = {:?}",
                i, status, priority
            );
        }

        // Test OR with parentheses - should get (Pending AND High) OR (Complete AND Low)
        println!("\n--- Test: (status = 'Pending' AND priority = 'High') OR (status = 'Complete' AND priority = 'Low') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE (status = 'Pending' AND priority = 'High') OR (status = 'Complete' AND priority = 'Low')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with parenthetical logic",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find rows 1 and 4
            }
            Err(e) => {
                panic!("Parentheses query failed: {}", e);
            }
        }

        println!("\n=== Parentheses test complete! ===");
    }

    #[test]
    fn test_numeric_type_coercion() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("price"));
        table.add_column(DataColumn::new("quantity"));

        // Add test data with different numeric types
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::Float(99.50), // Contains '.'
                DataValue::Integer(100),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::Float(150.0), // Contains '.' and '0'
                DataValue::Integer(200),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::Integer(75), // No decimal point
                DataValue::Integer(50),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing Numeric Type Coercion ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let price = table.get_value(i, 1);
            let quantity = table.get_value(i, 2);
            println!("Row {}: price = {:?}, quantity = {:?}", i, price, quantity);
        }

        // Test Contains on float values - should find rows with decimal points
        println!("\n--- Test: price.Contains('.') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE price.Contains('.')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with decimal points in price",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find 99.50 and 150.0
            }
            Err(e) => {
                panic!("Numeric Contains query failed: {}", e);
            }
        }

        // Test Contains on integer values converted to string
        println!("\n--- Test: quantity.Contains('0') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE quantity.Contains('0')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with '0' in quantity",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find 100 and 200
            }
            Err(e) => {
                panic!("Integer Contains query failed: {}", e);
            }
        }

        println!("\n=== Numeric type coercion test complete! ===");
    }

    #[test]
    fn test_datetime_comparisons() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("created_date"));

        // Add test data with date strings (as they would come from CSV)
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("2024-12-15".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("2025-01-15".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("2025-02-15".to_string()),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing DateTime Comparisons ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let date = table.get_value(i, 1);
            println!("Row {}: created_date = {:?}", i, date);
        }

        // Test DateTime constructor comparison - should find dates after 2025-01-01
        println!("\n--- Test: created_date > DateTime(2025,1,1) ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE created_date > DateTime(2025,1,1)",
        );
        match result {
            Ok(view) => {
                println!("SUCCESS: Found {} rows after 2025-01-01", view.row_count());
                assert_eq!(view.row_count(), 2); // Should find 2025-01-15 and 2025-02-15
            }
            Err(e) => {
                panic!("DateTime comparison query failed: {}", e);
            }
        }

        println!("\n=== DateTime comparison test complete! ===");
    }

    #[test]
    fn test_not_with_method_calls() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("status"));

        // Add test data
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Pending Review".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Complete".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Pending Approval".to_string()),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::with_case_insensitive(true);

        println!("\n=== Testing NOT with Method Calls ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let status = table.get_value(i, 1);
            println!("Row {}: status = {:?}", i, status);
        }

        // Test NOT with Contains - should exclude rows containing "pend"
        println!("\n--- Test: NOT status.Contains('pend') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE NOT status.Contains('pend')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows NOT containing 'pend'",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 1); // Should find only "Complete"
            }
            Err(e) => {
                panic!("NOT Contains query failed: {}", e);
            }
        }

        // Test NOT with StartsWith
        println!("\n--- Test: NOT status.StartsWith('Pending') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE NOT status.StartsWith('Pending')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows NOT starting with 'Pending'",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 1); // Should find only "Complete"
            }
            Err(e) => {
                panic!("NOT StartsWith query failed: {}", e);
            }
        }

        println!("\n=== NOT with method calls test complete! ===");
    }

    #[test]
    fn test_complex_logical_expressions() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("status"));
        table.add_column(DataColumn::new("priority"));
        table.add_column(DataColumn::new("assigned"));

        // Add comprehensive test data
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("Pending".to_string()),
                DataValue::String("High".to_string()),
                DataValue::String("John".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::String("Complete".to_string()),
                DataValue::String("High".to_string()),
                DataValue::String("Jane".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::String("Pending".to_string()),
                DataValue::String("Low".to_string()),
                DataValue::String("John".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(4),
                DataValue::String("In Progress".to_string()),
                DataValue::String("Medium".to_string()),
                DataValue::String("Jane".to_string()),
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing Complex Logical Expressions ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let status = table.get_value(i, 1);
            let priority = table.get_value(i, 2);
            let assigned = table.get_value(i, 3);
            println!(
                "Row {}: status = {:?}, priority = {:?}, assigned = {:?}",
                i, status, priority, assigned
            );
        }

        // Test complex AND/OR logic
        println!("\n--- Test: status = 'Pending' AND (priority = 'High' OR assigned = 'John') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE status = 'Pending' AND (priority = 'High' OR assigned = 'John')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with complex logic",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find rows 1 and 3 (both Pending, one High priority, both assigned to John)
            }
            Err(e) => {
                panic!("Complex logic query failed: {}", e);
            }
        }

        // Test NOT with complex expressions
        println!("\n--- Test: NOT (status.Contains('Complete') OR priority = 'Low') ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE NOT (status.Contains('Complete') OR priority = 'Low')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with NOT complex logic",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 2); // Should find rows 1 (Pending+High) and 4 (In Progress+Medium)
            }
            Err(e) => {
                panic!("NOT complex logic query failed: {}", e);
            }
        }

        println!("\n=== Complex logical expressions test complete! ===");
    }

    #[test]
    fn test_mixed_data_types_and_edge_cases() {
        // Initialize tracing for debug output
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let mut table = DataTable::new("test");
        table.add_column(DataColumn::new("id"));
        table.add_column(DataColumn::new("value"));
        table.add_column(DataColumn::new("nullable_field"));

        // Add test data with mixed types and edge cases
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(1),
                DataValue::String("123.45".to_string()),
                DataValue::String("present".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(2),
                DataValue::Float(678.90),
                DataValue::Null,
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(3),
                DataValue::Boolean(true),
                DataValue::String("also present".to_string()),
            ]))
            .unwrap();

        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(4),
                DataValue::String("false".to_string()),
                DataValue::Null,
            ]))
            .unwrap();

        let table = Arc::new(table);
        let engine = QueryEngine::new();

        println!("\n=== Testing Mixed Data Types and Edge Cases ===");
        println!("Table has {} rows", table.row_count());
        for i in 0..table.row_count() {
            let value = table.get_value(i, 1);
            let nullable = table.get_value(i, 2);
            println!(
                "Row {}: value = {:?}, nullable_field = {:?}",
                i, value, nullable
            );
        }

        // Test type coercion with boolean Contains
        println!("\n--- Test: value.Contains('true') (boolean to string coercion) ---");
        let result = engine.execute(
            table.clone(),
            "SELECT * FROM test WHERE value.Contains('true')",
        );
        match result {
            Ok(view) => {
                println!(
                    "SUCCESS: Found {} rows with boolean coercion",
                    view.row_count()
                );
                assert_eq!(view.row_count(), 1); // Should find the boolean true row
            }
            Err(e) => {
                panic!("Boolean coercion query failed: {}", e);
            }
        }

        // Test multiple IN values with mixed types
        println!("\n--- Test: id IN (1, 3) ---");
        let result = engine.execute(table.clone(), "SELECT * FROM test WHERE id IN (1, 3)");
        match result {
            Ok(view) => {
                println!("SUCCESS: Found {} rows with IN clause", view.row_count());
                assert_eq!(view.row_count(), 2); // Should find rows with id 1 and 3
            }
            Err(e) => {
                panic!("Multiple IN values query failed: {}", e);
            }
        }

        println!("\n=== Mixed data types test complete! ===");
    }

    #[test]
    fn test_not_in_parsing() {
        use crate::sql::recursive_parser::Parser;

        let query = "SELECT * FROM test WHERE country NOT IN ('CA')";
        println!("\n=== Testing NOT IN parsing ===");
        println!("Parsing query: {}", query);

        let mut parser = Parser::new(query);
        match parser.parse() {
            Ok(statement) => {
                println!("Parsed statement: {:#?}", statement);
                if let Some(where_clause) = statement.where_clause {
                    println!("WHERE conditions: {:#?}", where_clause.conditions);
                    if let Some(first_condition) = where_clause.conditions.first() {
                        println!("First condition expression: {:#?}", first_condition.expr);
                    }
                }
            }
            Err(e) => {
                panic!("Parse error: {}", e);
            }
        }
    }
}
