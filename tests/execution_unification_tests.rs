//! Integration tests for execution mode unification
//!
//! These tests verify that the new unified execution module works correctly
//! and is ready to be integrated into both script mode and single query mode.

use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataType, DataValue};
use sql_cli::execution::{ExecutionConfig, ExecutionContext, StatementExecutor};
use sql_cli::sql::recursive_parser::Parser;
use std::sync::Arc;

/// Helper to create a sample table for testing
fn create_sales_table() -> DataTable {
    let mut table = DataTable::new("sales");
    table.add_column(DataColumn::new("id").with_type(DataType::Integer));
    table.add_column(DataColumn::new("product").with_type(DataType::String));
    table.add_column(DataColumn::new("quantity").with_type(DataType::Integer));
    table.add_column(DataColumn::new("price").with_type(DataType::Float));

    let data = vec![
        (1, "Widget", 10, 19.99),
        (2, "Gadget", 5, 49.99),
        (3, "Doohickey", 15, 9.99),
        (4, "Thingamajig", 8, 29.99),
        (5, "Whatsit", 20, 14.99),
    ];

    for (id, product, quantity, price) in data {
        let _ = table.add_row(DataRow {
            values: vec![
                DataValue::Integer(id),
                DataValue::String(product.to_string()),
                DataValue::Integer(quantity),
                DataValue::Float(price),
            ],
        });
    }

    table
}

#[test]
fn test_simple_select_execution() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    let mut parser = Parser::new("SELECT id, product FROM sales WHERE id <= 3");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor
        .execute(stmt, &mut context)
        .expect("Execution failed");

    assert_eq!(result.dataview.row_count(), 3);
    assert_eq!(result.dataview.column_count(), 2);
    assert!(result.stats.total_time_ms >= 0.0);
}

#[test]
fn test_select_with_arithmetic() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    let mut parser = Parser::new("SELECT product, quantity * price as total FROM sales");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor
        .execute(stmt, &mut context)
        .expect("Execution failed");

    assert_eq!(result.dataview.row_count(), 5);
    assert_eq!(result.dataview.column_count(), 2);
}

#[test]
fn test_aggregation_query() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    let mut parser = Parser::new("SELECT COUNT(*) as total_count FROM sales");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor
        .execute(stmt, &mut context)
        .expect("Execution failed");

    assert_eq!(result.dataview.row_count(), 1);
    assert_eq!(result.dataview.column_count(), 1);

    // Verify the count
    let row = result.dataview.get_row(0).expect("No row");
    if let DataValue::Integer(count) = row.values[0] {
        assert_eq!(count, 5);
    } else {
        panic!("Expected integer count");
    }
}

#[test]
fn test_dual_table_expressions() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    // Query without FROM - uses DUAL
    let mut parser = Parser::new("SELECT 42 as answer, 'hello' as greeting");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor
        .execute(stmt, &mut context)
        .expect("Execution failed");

    assert_eq!(result.dataview.row_count(), 1);
    assert_eq!(result.dataview.column_count(), 2);
}

#[test]
fn test_temp_table_creation_and_query() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    // First query: filter data
    let mut parser1 = Parser::new("SELECT * FROM sales WHERE quantity > 10");
    let stmt1 = parser1.parse().expect("Parse failed");

    let result1 = executor
        .execute(stmt1, &mut context)
        .expect("Execution failed");

    assert_eq!(result1.dataview.row_count(), 2); // Doohickey and Whatsit

    // Store as temp table
    context
        .store_temp_table("#high_quantity".to_string(), result1.dataview.source_arc())
        .expect("Failed to store temp table");

    // Second query: query the temp table
    let mut parser2 = Parser::new("SELECT product FROM #high_quantity");
    let stmt2 = parser2.parse().expect("Parse failed");

    let result2 = executor
        .execute(stmt2, &mut context)
        .expect("Execution failed");

    assert_eq!(result2.dataview.row_count(), 2);
    assert_eq!(result2.dataview.column_count(), 1);
}

#[test]
fn test_case_insensitive_execution() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));

    let config = ExecutionConfig::new().with_case_insensitive(true);
    let executor = StatementExecutor::with_config(config);

    // Use uppercase column names
    let mut parser = Parser::new("SELECT PRODUCT, QUANTITY FROM sales");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor.execute(stmt, &mut context);

    // Should work with case insensitive mode
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.dataview.row_count(), 5);
}

#[test]
fn test_preprocessing_statistics() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    let mut parser = Parser::new("SELECT * FROM sales WHERE id > 0");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor
        .execute(stmt, &mut context)
        .expect("Execution failed");

    // Should have preprocessing statistics
    assert!(result.stats.preprocessing_time_ms >= 0.0);
    assert!(result.stats.execution_time_ms >= 0.0);
    assert!(
        result.stats.total_time_ms
            >= result.stats.preprocessing_time_ms + result.stats.execution_time_ms
    );
}

#[test]
fn test_no_preprocessing_for_dual() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    // Query without FROM
    let mut parser = Parser::new("SELECT 1+1");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor
        .execute(stmt, &mut context)
        .expect("Execution failed");

    // No preprocessing should occur for queries without FROM
    assert!(!result.stats.preprocessing_applied);
}

#[test]
fn test_preprocessing_disabled() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));

    let config = ExecutionConfig::new().without_preprocessing();
    let executor = StatementExecutor::with_config(config);

    let mut parser = Parser::new("SELECT * FROM sales");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor
        .execute(stmt, &mut context)
        .expect("Execution failed");

    // Should still work even without preprocessing
    assert_eq!(result.dataview.row_count(), 5);
}

#[test]
fn test_multiple_statements_with_context() {
    // Simulate script mode: multiple statements sharing context
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    // Statement 1: Get high value items
    let mut parser1 = Parser::new("SELECT * FROM sales WHERE price > 30");
    let stmt1 = parser1.parse().expect("Parse 1 failed");
    let result1 = executor
        .execute(stmt1, &mut context)
        .expect("Execution 1 failed");

    assert_eq!(result1.dataview.row_count(), 1); // Gadget

    // Store in temp table
    context
        .store_temp_table("#high_value".to_string(), result1.dataview.source_arc())
        .expect("Store failed");

    // Statement 2: Get low quantity items
    let mut parser2 = Parser::new("SELECT * FROM sales WHERE quantity < 10");
    let stmt2 = parser2.parse().expect("Parse 2 failed");
    let result2 = executor
        .execute(stmt2, &mut context)
        .expect("Execution 2 failed");

    assert_eq!(result2.dataview.row_count(), 2); // Gadget, Thingamajig

    // Store in temp table
    context
        .store_temp_table("#low_quantity".to_string(), result2.dataview.source_arc())
        .expect("Store failed");

    // Statement 3: Query temp tables
    let mut parser3 = Parser::new("SELECT * FROM #high_value");
    let stmt3 = parser3.parse().expect("Parse 3 failed");
    let result3 = executor
        .execute(stmt3, &mut context)
        .expect("Execution 3 failed");

    assert_eq!(result3.dataview.row_count(), 1);

    // Verify temp table registry
    assert!(context.has_temp_table("#high_value"));
    assert!(context.has_temp_table("#low_quantity"));
    assert_eq!(context.temp_table_names().len(), 2);
}

#[test]
fn test_context_isolation() {
    // Verify that separate contexts don't share temp tables
    let table = create_sales_table();

    let mut context1 = ExecutionContext::new(Arc::new(table.clone()));
    let mut context2 = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    // Create temp table in context1
    let temp_table = DataTable::new("#temp");
    context1
        .store_temp_table("#temp".to_string(), Arc::new(temp_table))
        .expect("Store failed");

    // Verify isolation
    assert!(context1.has_temp_table("#temp"));
    assert!(!context2.has_temp_table("#temp"));
}

#[test]
fn test_order_by_execution() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    let mut parser = Parser::new("SELECT product, price FROM sales ORDER BY price DESC");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor
        .execute(stmt, &mut context)
        .expect("Execution failed");

    assert_eq!(result.dataview.row_count(), 5);

    // First row should be Gadget (highest price: 49.99)
    let first_row = result.dataview.get_row(0).expect("No first row");
    if let DataValue::String(product) = &first_row.values[0] {
        assert_eq!(product, "Gadget");
    } else {
        panic!("Expected string product name");
    }
}

#[test]
fn test_limit_clause() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    let mut parser = Parser::new("SELECT * FROM sales LIMIT 2");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor
        .execute(stmt, &mut context)
        .expect("Execution failed");

    assert_eq!(result.dataview.row_count(), 2);
    assert_eq!(result.dataview.column_count(), 4);
}

#[test]
fn test_execution_with_group_by() {
    let table = create_sales_table();
    let mut context = ExecutionContext::new(Arc::new(table));
    let executor = StatementExecutor::new();

    // This is a simple group by - more complex ones might need preprocessing
    let mut parser =
        Parser::new("SELECT product, SUM(quantity) as total FROM sales GROUP BY product");
    let stmt = parser.parse().expect("Parse failed");

    let result = executor.execute(stmt, &mut context);

    // Should succeed
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.dataview.row_count(), 5); // 5 different products
}
