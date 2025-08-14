use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("users");

    // Add columns
    table.add_column(DataColumn::new("id".to_string(), vec![]));
    table.add_column(DataColumn::new("name".to_string(), vec![]));
    table.add_column(DataColumn::new("age".to_string(), vec![]));
    table.add_column(DataColumn::new("city".to_string(), vec![]));

    // Add rows
    table.add_row(DataRow::new(vec![
        DataValue::Integer(1),
        DataValue::String("Alice".to_string()),
        DataValue::Integer(30),
        DataValue::String("New York".to_string()),
    ]));

    table.add_row(DataRow::new(vec![
        DataValue::Integer(2),
        DataValue::String("Bob".to_string()),
        DataValue::Integer(25),
        DataValue::String("London".to_string()),
    ]));

    table.add_row(DataRow::new(vec![
        DataValue::Integer(3),
        DataValue::String("Charlie".to_string()),
        DataValue::Integer(35),
        DataValue::String("Paris".to_string()),
    ]));

    table.add_row(DataRow::new(vec![
        DataValue::Integer(4),
        DataValue::String("David".to_string()),
        DataValue::Integer(28),
        DataValue::String("London".to_string()),
    ]));

    table.add_row(DataRow::new(vec![
        DataValue::Integer(5),
        DataValue::String("Eve".to_string()),
        DataValue::Integer(32),
        DataValue::String("New York".to_string()),
    ]));

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
