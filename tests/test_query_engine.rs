use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("users");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("age"));
    table.add_column(DataColumn::new("city"));

    // Add rows
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("Alice".to_string()),
            DataValue::Integer(30),
            DataValue::String("New York".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("Bob".to_string()),
            DataValue::Integer(25),
            DataValue::String("London".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("Charlie".to_string()),
            DataValue::Integer(35),
            DataValue::String("Paris".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(4),
            DataValue::String("David".to_string()),
            DataValue::Integer(28),
            DataValue::String("London".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(5),
            DataValue::String("Eve".to_string()),
            DataValue::Integer(32),
            DataValue::String("New York".to_string()),
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
    assert_eq!(view.row_count(), 5);
    assert_eq!(view.column_count(), 4);

    // Check column names
    let columns = view.column_names();
    assert_eq!(columns, vec!["id", "name", "age", "city"]);
}

#[test]
fn test_select_specific_columns() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT name, age FROM users")
        .unwrap();
    assert_eq!(view.row_count(), 5);
    assert_eq!(view.column_count(), 2);

    // Check column names
    let columns = view.column_names();
    assert_eq!(columns, vec!["name", "age"]);

    // Check first row data
    let row = view.get_row(0).unwrap();
    assert_eq!(row.values.len(), 2);
    assert_eq!(row.values[0], DataValue::String("Alice".to_string()));
    assert_eq!(row.values[1], DataValue::Integer(30));
}

#[test]
fn test_select_with_limit() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM users LIMIT 3")
        .unwrap();
    assert_eq!(view.row_count(), 3);
    assert_eq!(view.column_count(), 4);
}

#[test]
fn test_select_with_limit_offset() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM users LIMIT 2 OFFSET 2")
        .unwrap();
    assert_eq!(view.row_count(), 2);

    // Check that we get the right rows (Charlie and David)
    let row = view.get_row(0).unwrap();
    assert_eq!(row.values[1], DataValue::String("Charlie".to_string()));

    let row = view.get_row(1).unwrap();
    assert_eq!(row.values[1], DataValue::String("David".to_string()));
}

#[test]
fn test_data_view_direct_usage() {
    let table = create_test_table();

    // Test DataView directly
    let view = DataView::new(table.clone());
    assert_eq!(view.row_count(), 5);
    assert_eq!(view.column_count(), 4);

    // Test with column projection
    let view = view.with_columns(vec![1, 2]); // name and age
    assert_eq!(view.column_count(), 2);

    // Test with limit
    let view = view.with_limit(2, 0);
    assert_eq!(view.row_count(), 2);
}

#[test]
fn test_data_view_sorting() {
    let table = create_test_table();

    // Sort by age ascending
    let view = DataView::new(table.clone())
        .sort_by(2, true) // age column
        .unwrap();

    // Check that rows are sorted by age
    let row = view.get_row(0).unwrap();
    assert_eq!(row.values[2], DataValue::Integer(25)); // Bob is youngest

    let row = view.get_row(4).unwrap();
    assert_eq!(row.values[2], DataValue::Integer(35)); // Charlie is oldest
}

#[test]
fn test_select_with_where_equals() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM users WHERE city = 'London'")
        .unwrap();
    assert_eq!(view.row_count(), 2); // Bob and David

    // Check that we get the right rows
    let row = view.get_row(0).unwrap();
    assert_eq!(row.values[1], DataValue::String("Bob".to_string()));

    let row = view.get_row(1).unwrap();
    assert_eq!(row.values[1], DataValue::String("David".to_string()));
}

#[test]
fn test_select_with_where_greater_than() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM users WHERE age > 30")
        .unwrap();
    assert_eq!(view.row_count(), 2); // Charlie (35) and Eve (32)
}

#[test]
fn test_select_columns_with_where() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT name, age FROM users WHERE age >= 30")
        .unwrap();
    assert_eq!(view.row_count(), 3); // Alice (30), Charlie (35), Eve (32)
    assert_eq!(view.column_count(), 2); // Only name and age columns

    let columns = view.column_names();
    assert_eq!(columns, vec!["name", "age"]);
}

#[test]
fn test_select_with_like() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT * FROM users WHERE name LIKE '%e%'")
        .unwrap();
    // Should match Alice, Charlie, Eve (all contain 'e')
    assert_eq!(view.row_count(), 3);
}

#[test]
fn test_select_with_in() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM users WHERE city IN ('London', 'Paris')",
        )
        .unwrap();
    // Should match Bob (London), Charlie (Paris), David (London)
    assert_eq!(view.row_count(), 3);
}

#[test]
fn test_select_with_between() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM users WHERE age BETWEEN 28 AND 32",
        )
        .unwrap();
    // Should match David (28), Alice (30), Eve (32)
    assert_eq!(view.row_count(), 3);
}

#[test]
fn test_select_with_and_condition() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM users WHERE age > 25 AND city = 'London'",
        )
        .unwrap();
    // Should match David (28, London) only - Bob is 25 not > 25
    assert_eq!(view.row_count(), 1);

    let row = view.get_row(0).unwrap();
    assert_eq!(row.values[1], DataValue::String("David".to_string()));
}

#[test]
fn test_select_with_or_condition() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM users WHERE city = 'Paris' OR age = 25",
        )
        .unwrap();
    // Should match Charlie (Paris) and Bob (age 25)
    assert_eq!(view.row_count(), 2);
}

#[test]
#[ignore = "Complex WHERE with parentheses not yet implemented"]
fn test_complex_where_and_or() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(
            table.clone(),
            "SELECT * FROM users WHERE (age > 30 AND city = 'New York') OR city = 'Paris'",
        )
        .unwrap();
    // Should match Eve (32, New York) and Charlie (35, Paris)
    assert_eq!(view.row_count(), 2);
}

#[test]
fn test_string_literal_with_aggregates_single_row() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: string literal + aggregate should return single row (not Cartesian product)
    let view = engine
        .execute(
            table.clone(),
            "SELECT 'User Count', COUNT(id), 'Average Age', AVG(age) FROM users",
        )
        .unwrap();

    // Should return exactly 1 row, not 5 rows (one per input row)
    assert_eq!(view.row_count(), 1);

    let row = view.get_row(0).unwrap();
    assert_eq!(row.values[0], DataValue::String("User Count".to_string()));
    assert_eq!(row.values[1], DataValue::Integer(5)); // COUNT(id)
    assert_eq!(row.values[2], DataValue::String("Average Age".to_string()));
    // AVG(age) = (30 + 25 + 35 + 28 + 32) / 5 = 30.0
    if let DataValue::Float(avg) = &row.values[3] {
        assert!((avg - 30.0).abs() < 0.001);
    } else {
        panic!("Expected float for AVG(age), got {:?}", row.values[3]);
    }
}

#[test]
fn test_number_literal_with_aggregates_single_row() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: number literals + aggregates should return single row
    let view = engine
        .execute(
            table.clone(),
            "SELECT 42, COUNT(id), 3.14, MIN(age) FROM users",
        )
        .unwrap();

    assert_eq!(view.row_count(), 1);

    let row = view.get_row(0).unwrap();
    assert_eq!(row.values[0], DataValue::Integer(42));
    assert_eq!(row.values[1], DataValue::Integer(5));
    assert_eq!(row.values[2], DataValue::Float(3.14));
    assert_eq!(row.values[3], DataValue::Integer(25)); // MIN(age)
}

#[test]
fn test_boolean_null_literal_with_aggregates_single_row() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: boolean and null literals + aggregates should return single row
    let view = engine
        .execute(
            table.clone(),
            "SELECT TRUE, COUNT(id), NULL, MAX(age) FROM users",
        )
        .unwrap();

    assert_eq!(view.row_count(), 1);

    let row = view.get_row(0).unwrap();
    assert_eq!(row.values[0], DataValue::Boolean(true));
    assert_eq!(row.values[1], DataValue::Integer(5));
    assert_eq!(row.values[2], DataValue::Null);
    assert_eq!(row.values[3], DataValue::Integer(35)); // MAX(age)
}

#[test]
fn test_column_with_aggregates_multiple_rows() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: column reference + aggregates should return multiple rows (Cartesian product)
    // This is the existing behavior that should remain unchanged
    let view = engine
        .execute(table.clone(), "SELECT name, COUNT(id) FROM users")
        .unwrap();

    // Should return 5 rows (Cartesian product), not 1 row
    assert_eq!(view.row_count(), 5);

    // All rows should have COUNT(id) = 5
    for i in 0..5 {
        let row = view.get_row(i).unwrap();
        assert_eq!(row.values[1], DataValue::Integer(5));
    }
}

#[test]
fn test_constant_arithmetic_with_aggregates_single_row() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: constant arithmetic expressions + aggregates should return single row
    let view = engine
        .execute(table.clone(), "SELECT 1 + 2, COUNT(id), 10 * 5 FROM users")
        .unwrap();

    assert_eq!(view.row_count(), 1);

    let row = view.get_row(0).unwrap();
    assert_eq!(row.values[0], DataValue::Integer(3)); // 1 + 2
    assert_eq!(row.values[1], DataValue::Integer(5)); // COUNT(id)
    assert_eq!(row.values[2], DataValue::Integer(50)); // 10 * 5
}
