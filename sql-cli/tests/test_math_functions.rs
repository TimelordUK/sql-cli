use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

/// Helper to get a value from a DataView
fn get_value(view: &DataView, row_idx: usize, col_idx: usize) -> DataValue {
    view.get_row(row_idx).unwrap().get(col_idx).unwrap().clone()
}

/// Create a test table for math function tests
fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test_math");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("quantity"));
    table.add_column(DataColumn::new("price"));
    table.add_column(DataColumn::new("discount"));
    table.add_column(DataColumn::new("negative"));

    // Add test rows with various numeric values
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::Integer(10),
            DataValue::Float(25.456),
            DataValue::Float(2.5),
            DataValue::Float(-15.789),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::Integer(7),
            DataValue::Float(99.999),
            DataValue::Float(10.0),
            DataValue::Integer(-42),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::Integer(3),
            DataValue::Float(15.234),
            DataValue::Float(1.567),
            DataValue::Float(-3.14159),
        ]))
        .unwrap();

    Arc::new(table)
}

#[test]
fn test_round_basic() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test ROUND with no decimals (default to 0)
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, ROUND(price) as rounded_price FROM test_math",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Row 1: ROUND(25.456) = 25
    assert_eq!(get_value(&view, 0, 1), DataValue::Integer(25));

    // Row 2: ROUND(99.999) = 100
    assert_eq!(get_value(&view, 1, 1), DataValue::Integer(100));

    // Row 3: ROUND(15.234) = 15
    assert_eq!(get_value(&view, 2, 1), DataValue::Integer(15));
}

#[test]
fn test_round_with_decimals() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test ROUND with 2 decimal places
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, ROUND(price, 2) as rounded_price FROM test_math",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Row 1: ROUND(25.456, 2) = 25.46
    assert_eq!(get_value(&view, 0, 1), DataValue::Float(25.46));

    // Row 2: ROUND(99.999, 2) = 100.00
    assert_eq!(get_value(&view, 1, 1), DataValue::Float(100.0));

    // Row 3: ROUND(15.234, 2) = 15.23
    assert_eq!(get_value(&view, 2, 1), DataValue::Float(15.23));
}

#[test]
fn test_round_with_nested_expression() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test ROUND with arithmetic expression: ROUND(quantity * price / 100, 3)
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, ROUND(quantity * price / 100, 3) as result FROM test_math",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Row 1: ROUND(10 * 25.456 / 100, 3) = ROUND(2.5456, 3) = 2.546
    assert_eq!(get_value(&view, 0, 1), DataValue::Float(2.546));

    // Row 2: ROUND(7 * 99.999 / 100, 3) = ROUND(6.99993, 3) = 7.0
    assert_eq!(get_value(&view, 1, 1), DataValue::Float(7.0));

    // Row 3: ROUND(3 * 15.234 / 100, 3) = ROUND(0.45702, 3) = 0.457
    assert_eq!(get_value(&view, 2, 1), DataValue::Float(0.457));
}

#[test]
fn test_abs_function() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test ABS function
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, ABS(negative) as abs_value FROM test_math",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Row 1: ABS(-15.789) = 15.789
    assert_eq!(get_value(&view, 0, 1), DataValue::Float(15.789));

    // Row 2: ABS(-42) = 42
    assert_eq!(get_value(&view, 1, 1), DataValue::Integer(42));

    // Row 3: ABS(-3.14159) = 3.14159
    assert_eq!(get_value(&view, 2, 1), DataValue::Float(3.14159));
}

#[test]
fn test_floor_function() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test FLOOR function
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, FLOOR(price) as floor_price FROM test_math",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Row 1: FLOOR(25.456) = 25
    assert_eq!(get_value(&view, 0, 1), DataValue::Integer(25));

    // Row 2: FLOOR(99.999) = 99
    assert_eq!(get_value(&view, 1, 1), DataValue::Integer(99));

    // Row 3: FLOOR(15.234) = 15
    assert_eq!(get_value(&view, 2, 1), DataValue::Integer(15));
}

#[test]
fn test_ceiling_function() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test CEILING function
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, CEILING(price) as ceil_price FROM test_math",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Row 1: CEILING(25.456) = 26
    assert_eq!(get_value(&view, 0, 1), DataValue::Integer(26));

    // Row 2: CEILING(99.999) = 100
    assert_eq!(get_value(&view, 1, 1), DataValue::Integer(100));

    // Row 3: CEILING(15.234) = 16
    assert_eq!(get_value(&view, 2, 1), DataValue::Integer(16));
}

#[test]
fn test_ceil_alias_function() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test CEIL (alias for CEILING)
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, CEIL(discount) as ceil_discount FROM test_math",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Row 1: CEIL(2.5) = 3
    assert_eq!(get_value(&view, 0, 1), DataValue::Integer(3));

    // Row 2: CEIL(10.0) = 10
    assert_eq!(get_value(&view, 1, 1), DataValue::Integer(10));

    // Row 3: CEIL(1.567) = 2
    assert_eq!(get_value(&view, 2, 1), DataValue::Integer(2));
}

#[test]
fn test_functions_in_where_clause() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test using ROUND in WHERE clause
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, price FROM test_math WHERE ROUND(price) > 50",
        )
        .unwrap();

    // Only row 2 has ROUND(99.999) = 100 > 50
    assert_eq!(view.row_count(), 1);
    assert_eq!(get_value(&view, 0, 0), DataValue::Integer(2));
}

#[test]
fn test_nested_functions() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test nested functions: ROUND(ABS(negative), 1)
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, ROUND(ABS(negative), 1) as result FROM test_math",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Row 1: ROUND(ABS(-15.789), 1) = ROUND(15.789, 1) = 15.8
    assert_eq!(get_value(&view, 0, 1), DataValue::Float(15.8));

    // Row 2: ROUND(ABS(-42), 1) = ROUND(42, 1) = 42.0
    assert_eq!(get_value(&view, 1, 1), DataValue::Integer(42));

    // Row 3: ROUND(ABS(-3.14159), 1) = ROUND(3.14159, 1) = 3.1
    assert_eq!(get_value(&view, 2, 1), DataValue::Float(3.1));
}

#[test]
fn test_functions_with_order_by() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test ORDER BY with function result
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, ROUND(price - discount, 2) as net_price 
             FROM test_math 
             ORDER BY net_price DESC",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Results should be ordered by net_price descending:
    // Row 2: 99.999 - 10.0 = 90.0
    // Row 1: 25.456 - 2.5 = 22.96
    // Row 3: 15.234 - 1.567 = 13.67

    assert_eq!(get_value(&view, 0, 0), DataValue::Integer(2)); // id=2
    assert_eq!(get_value(&view, 0, 1), DataValue::Float(90.0));

    assert_eq!(get_value(&view, 1, 0), DataValue::Integer(1)); // id=1
    assert_eq!(get_value(&view, 1, 1), DataValue::Float(22.96));

    assert_eq!(get_value(&view, 2, 0), DataValue::Integer(3)); // id=3
    assert_eq!(get_value(&view, 2, 1), DataValue::Float(13.67));
}

#[test]
fn test_complex_expression_with_functions() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test complex expression: quantity * ROUND(price, 1) - ABS(negative)
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, quantity * ROUND(price, 1) - ABS(negative) as complex_calc 
             FROM test_math",
        )
        .unwrap();

    assert_eq!(view.row_count(), 3);

    // Row 1: 10 * ROUND(25.456, 1) - ABS(-15.789) = 10 * 25.5 - 15.789 = 239.211
    assert_eq!(get_value(&view, 0, 1), DataValue::Float(239.211));

    // Row 2: 7 * ROUND(99.999, 1) - ABS(-42) = 7 * 100.0 - 42 = 658.0
    assert_eq!(get_value(&view, 1, 1), DataValue::Float(658.0));

    // Row 3: 3 * ROUND(15.234, 1) - ABS(-3.14159) = 3 * 15.2 - 3.14159 = 42.45841
    let val = get_value(&view, 2, 1);
    if let DataValue::Float(f) = val {
        assert!((f - 42.45841).abs() < 0.00001);
    } else {
        panic!("Expected Float value");
    }
}

#[test]
fn test_functions_with_select_star() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test SELECT *, function_result pattern
    let view = engine
        .execute(
            table.clone(),
            "SELECT *, ROUND(quantity * price, 2) as total FROM test_math WHERE id = 1",
        )
        .unwrap();

    assert_eq!(view.row_count(), 1);
    // Should have all original columns plus the computed one
    assert_eq!(view.column_count(), 6); // 5 original + 1 computed

    // Verify the computed column
    assert_eq!(get_value(&view, 0, 5), DataValue::Float(254.56)); // 10 * 25.456
}
