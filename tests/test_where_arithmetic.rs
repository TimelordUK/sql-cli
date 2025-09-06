use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

/// Helper to get a value from a `DataView`
fn get_value(view: &DataView, row_idx: usize, col_idx: usize) -> DataValue {
    view.get_row(row_idx).unwrap().get(col_idx).unwrap().clone()
}

/// Create a test table for arithmetic WHERE tests
fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test_arithmetic");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("quantity"));
    table.add_column(DataColumn::new("price"));
    table.add_column(DataColumn::new("discount"));

    // Add test rows
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::Integer(10),
            DataValue::Float(25.50),
            DataValue::Float(2.50),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::Integer(5),
            DataValue::Float(100.00),
            DataValue::Float(10.00),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::Integer(3),
            DataValue::Integer(15),
            DataValue::Float(1.50),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(4),
            DataValue::Integer(8),
            DataValue::Float(45.75),
            DataValue::Float(5.00),
        ]))
        .unwrap();

    Arc::new(table)
}

#[test]
fn test_where_with_multiplication() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test WHERE with multiplication expression
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, quantity, price FROM test_arithmetic WHERE quantity * price > 200",
        )
        .unwrap();

    // Expected rows where quantity * price > 200:
    // Row 1: 10 * 25.50 = 255.0 ✓
    // Row 2: 5 * 100.00 = 500.0 ✓
    // Row 3: 3 * 15 = 45.0 ✗
    // Row 4: 8 * 45.75 = 366.0 ✓
    assert_eq!(
        view.row_count(),
        3,
        "Should return 3 rows where quantity * price > 200"
    );

    // Verify the IDs of returned rows
    assert_eq!(get_value(&view, 0, 0), DataValue::Integer(1));
    assert_eq!(get_value(&view, 1, 0), DataValue::Integer(2));
    assert_eq!(get_value(&view, 2, 0), DataValue::Integer(4));
}

#[test]
fn test_where_with_subtraction() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test WHERE with subtraction expression
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, price, discount FROM test_arithmetic WHERE price - discount < 50",
        )
        .unwrap();

    // Expected rows where price - discount < 50:
    // Row 1: 25.50 - 2.50 = 23.0 ✓
    // Row 2: 100.00 - 10.00 = 90.0 ✗
    // Row 3: 15 - 1.50 = 13.50 ✓
    // Row 4: 45.75 - 5.00 = 40.75 ✓
    assert_eq!(
        view.row_count(),
        3,
        "Should return 3 rows where price - discount < 50"
    );
}

#[test]
fn test_where_with_addition() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test WHERE with addition expression
    let view = engine
        .execute(
            table.clone(),
            "SELECT id FROM test_arithmetic WHERE price + discount > 100",
        )
        .unwrap();

    // Expected rows where price + discount > 100:
    // Row 1: 25.50 + 2.50 = 28.0 ✗
    // Row 2: 100.00 + 10.00 = 110.0 ✓
    // Row 3: 15 + 1.50 = 16.50 ✗
    // Row 4: 45.75 + 5.00 = 50.75 ✗
    assert_eq!(
        view.row_count(),
        1,
        "Should return 1 row where price + discount > 100"
    );
    assert_eq!(get_value(&view, 0, 0), DataValue::Integer(2));
}

#[test]
fn test_where_with_division() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test WHERE with division expression
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, price, quantity FROM test_arithmetic WHERE price / quantity > 10",
        )
        .unwrap();

    // Expected rows where price / quantity > 10:
    // Row 1: 25.50 / 10 = 2.55 ✗
    // Row 2: 100.00 / 5 = 20.0 ✓
    // Row 3: 15 / 3 = 5.0 ✗
    // Row 4: 45.75 / 8 = 5.71875 ✗
    assert_eq!(
        view.row_count(),
        1,
        "Should return 1 row where price / quantity > 10"
    );
    assert_eq!(get_value(&view, 0, 0), DataValue::Integer(2));
}

#[test]
fn test_where_with_complex_expression() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test WHERE with complex nested expression
    let view = engine
        .execute(
            table.clone(),
            "SELECT id FROM test_arithmetic WHERE (quantity * price) - discount > 250",
        )
        .unwrap();

    // Expected rows where (quantity * price) - discount > 250:
    // Row 1: (10 * 25.50) - 2.50 = 252.50 ✓
    // Row 2: (5 * 100.00) - 10.00 = 490.0 ✓
    // Row 3: (3 * 15) - 1.50 = 43.50 ✗
    // Row 4: (8 * 45.75) - 5.00 = 361.0 ✓
    assert_eq!(
        view.row_count(),
        3,
        "Should return 3 rows where (quantity * price) - discount > 250"
    );
}

#[test]
fn test_where_expression_with_computed_select() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test combining WHERE expression with computed SELECT columns
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, quantity * price as total, price - discount as net_price 
             FROM test_arithmetic 
             WHERE quantity * price > 200",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Verify computed columns are calculated correctly for filtered rows
    // Row 1: id=1, total=255.0, net_price=23.0
    assert_eq!(get_value(&view, 0, 0), DataValue::Integer(1));
    assert_eq!(get_value(&view, 0, 1), DataValue::Float(255.0));
    assert_eq!(get_value(&view, 0, 2), DataValue::Float(23.0));

    // Row 2: id=2, total=500.0, net_price=90.0
    assert_eq!(get_value(&view, 1, 0), DataValue::Integer(2));
    assert_eq!(get_value(&view, 1, 1), DataValue::Float(500.0));
    assert_eq!(get_value(&view, 1, 2), DataValue::Float(90.0));
}

#[test]
fn test_where_expression_with_order_by() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test WHERE expression with ORDER BY computed column
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, quantity * price as total 
             FROM test_arithmetic 
             WHERE quantity * price > 100
             ORDER BY total DESC",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Verify order: 500.0, 366.0, 255.0
    assert_eq!(get_value(&view, 0, 0), DataValue::Integer(2)); // id=2, total=500.0
    assert_eq!(get_value(&view, 1, 0), DataValue::Integer(4)); // id=4, total=366.0
    assert_eq!(get_value(&view, 2, 0), DataValue::Integer(1)); // id=1, total=255.0
}
