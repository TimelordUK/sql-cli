use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

/// Helper function to get a value from a DataView row by index
fn get_value(view: &DataView, row_idx: usize, col_idx: usize) -> DataValue {
    view.get_row(row_idx).unwrap().get(col_idx).unwrap().clone()
}

/// Create a test table with numeric data for arithmetic operations
fn create_arithmetic_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("arithmetic_test");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("quantity"));
    table.add_column(DataColumn::new("price"));
    table.add_column(DataColumn::new("cost"));
    table.add_column(DataColumn::new("discount"));

    // Add test rows with mixed integer and float values
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::Integer(10),
            DataValue::Float(25.50),
            DataValue::Float(20.00),
            DataValue::Float(2.50),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::Integer(5),
            DataValue::Float(100.00),
            DataValue::Float(80.00),
            DataValue::Float(10.00),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::Integer(3),
            DataValue::Integer(15), // Integer price
            DataValue::Integer(12), // Integer cost
            DataValue::Float(1.50),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(4),
            DataValue::Integer(8),
            DataValue::Float(45.75),
            DataValue::Float(30.25),
            DataValue::Float(5.00),
        ]))
        .unwrap();

    Arc::new(table)
}

#[test]
fn test_simple_multiplication() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT quantity * price as total FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);
    assert_eq!(view.column_count(), 1);

    let columns = view.column_names();
    assert_eq!(columns, vec!["total"]);

    // Check calculated values
    // Row 1: 10 * 25.50 = 255.0 (int * float = float)
    // Row 2: 5 * 100.00 = 500.0 (int * float = float)
    // Row 3: 3 * 15 = 45 (int * int = int)
    // Row 4: 8 * 45.75 = 366.0 (int * float = float)
    assert_eq!(get_value(&view, 0, 0), DataValue::Float(255.0));
    assert_eq!(get_value(&view, 1, 0), DataValue::Float(500.0));
    assert_eq!(get_value(&view, 2, 0), DataValue::Integer(45));
    assert_eq!(get_value(&view, 3, 0), DataValue::Float(366.0));
}

#[test]
fn test_multiplication_with_literal() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT price * 2 as double_price FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);
    assert_eq!(view.column_count(), 1);

    // Row 1: 25.50 * 2 = 51.0
    let row_data = view.get_row(0).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Float(51.0));

    // Row 3: 15 * 2 = 30 (int * int = int)
    let row_data = view.get_row(2).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Integer(30));
}

#[test]
fn test_addition_and_subtraction() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT price + discount as total_price, price - cost as profit FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);
    assert_eq!(view.column_count(), 2);

    let columns = view.column_names();
    assert_eq!(columns, vec!["total_price", "profit"]);

    // Row 1: price + discount = 25.50 + 2.50 = 28.0, price - cost = 25.50 - 20.00 = 5.50
    let row_data = view.get_row(0).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Float(28.0));
    assert_eq!(row_data.get(1).unwrap(), &DataValue::Float(5.50));
}

#[test]
fn test_division() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT price / quantity as unit_price FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);

    // Row 1: 25.50 / 10 = 2.55
    let row_data = view.get_row(0).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Float(2.55));

    // Row 2: 100.00 / 5 = 20.0
    let row_data = view.get_row(1).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Float(20.0));
}

#[test]
fn test_complex_expression_with_precedence() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    // Test PEMDAS: (price - cost) * quantity / 2
    let view = engine
        .execute(
            table.clone(),
            "SELECT (price - cost) * quantity / 2 as profit_per_two FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);

    // Row 1: (25.50 - 20.00) * 10 / 2 = 5.50 * 10 / 2 = 55.0 / 2 = 27.5
    let row_data = view.get_row(0).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Float(27.5));
}

#[test]
fn test_expression_with_multiple_operations() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    // Test: a * b / c (should be (a * b) / c due to left-to-right associativity)
    let view = engine
        .execute(
            table.clone(),
            "SELECT quantity * price / cost as efficiency FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);

    // Row 1: (10 * 25.50) / 20.00 = 255.0 / 20.0 = 12.75
    let row_data = view.get_row(0).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Float(12.75));
}

#[test]
fn test_mixed_columns_and_expressions() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT id, quantity, price, quantity * price as total FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);
    assert_eq!(view.column_count(), 4);

    let columns = view.column_names();
    assert_eq!(columns, vec!["id", "quantity", "price", "total"]);

    // Check first row
    let row_data = view.get_row(0).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Integer(1)); // id
    assert_eq!(row_data.get(1).unwrap(), &DataValue::Integer(10)); // quantity
    assert_eq!(row_data.get(2).unwrap(), &DataValue::Float(25.50)); // price
    assert_eq!(row_data.get(3).unwrap(), &DataValue::Float(255.0)); // quantity * price
}

#[test]
fn test_type_coercion() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    // Mix integer and float operations
    let view = engine
        .execute(
            table.clone(),
            "SELECT quantity + 1.5 as adjusted_quantity FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);

    // Row 1: 10 + 1.5 = 11.5 (integer + float = float)
    let row_data = view.get_row(0).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Float(11.5));
}

#[test]
fn test_division_by_zero_error() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    let result = engine.execute(
        table.clone(),
        "SELECT price / 0 as invalid FROM arithmetic_test",
    );

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("division by zero") || error_msg.contains("Division by zero"));
}

#[test]
fn test_literal_only_expressions() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT 2 + 3 * 4 as math_result FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);

    // Should be 2 + (3 * 4) = 2 + 12 = 14 (due to precedence)
    let row_data = view.get_row(0).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Integer(14));

    // All rows should have the same result since it's literal-only
    for i in 0..4 {
        let row_data = view.get_row(i).unwrap();
        assert_eq!(row_data.get(0).unwrap(), &DataValue::Integer(14));
    }
}

#[test]
fn test_parentheses_precedence() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT (2 + 3) * 4 as paren_result FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);

    // Should be (2 + 3) * 4 = 5 * 4 = 20
    let row_data = view.get_row(0).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Integer(20));
}

#[test]
fn test_negative_numbers() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT price - cost - 50 as loss FROM arithmetic_test",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);

    // Row 1: 25.50 - 20.00 - 50 = 5.50 - 50 = -44.50
    let row_data = view.get_row(0).unwrap();
    assert_eq!(row_data.get(0).unwrap(), &DataValue::Float(-44.50));
}

#[test]
#[ignore] // Arithmetic in WHERE clauses not yet implemented
fn test_where_clause_with_arithmetic() {
    let table = create_arithmetic_test_table();
    let engine = QueryEngine::new();

    // This tests that WHERE clauses can also use arithmetic (when we implement it)
    // For now, this might not work, but it's a good test case for future extension
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, quantity * price as total FROM arithmetic_test WHERE quantity * price > 200"
        )
        .unwrap();

    // Should return rows where total > 200:
    // Row 1: 10 * 25.50 = 255.0 ✓
    // Row 2: 5 * 100.00 = 500.0 ✓
    // Row 3: 3 * 15 = 45.0 ✗
    // Row 4: 8 * 45.75 = 366.0 ✓
    assert_eq!(view.row_count(), 3);
}
