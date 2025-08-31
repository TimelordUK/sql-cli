use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

fn create_simple_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("a"));
    table.add_column(DataColumn::new("b"));

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(10),
            DataValue::Float(2.5),
        ]))
        .unwrap();

    Arc::new(table)
}

#[test]
fn test_basic_multiplication() {
    let table = create_simple_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT a * b as result FROM test")
        .unwrap();

    assert_eq!(view.row_count(), 1);
    assert_eq!(view.column_count(), 1);

    let columns = view.column_names();
    assert_eq!(columns, vec!["result"]);

    // Get the result value
    let row = view.get_row(0).unwrap();
    let result_value = row.get(0).unwrap().clone();

    // Should be 10 * 2.5 = 25.0
    assert_eq!(result_value, DataValue::Float(25.0));
}

#[test]
fn test_literal_arithmetic() {
    let table = create_simple_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT 2 + 3 * 4 as result FROM test")
        .unwrap();

    assert_eq!(view.row_count(), 1);

    let row = view.get_row(0).unwrap();
    let result_value = row.get(0).unwrap().clone();

    // Should be 2 + (3 * 4) = 2 + 12 = 14
    assert_eq!(result_value, DataValue::Integer(14));
}

#[test]
fn test_mixed_columns_and_literals() {
    let table = create_simple_test_table();
    let engine = QueryEngine::new();

    let view = engine
        .execute(table.clone(), "SELECT a, a * 2 as doubled FROM test")
        .unwrap();

    assert_eq!(view.row_count(), 1);
    assert_eq!(view.column_count(), 2);

    let row = view.get_row(0).unwrap();
    let a_value = row.get(0).unwrap().clone();
    let doubled_value = row.get(1).unwrap().clone();

    assert_eq!(a_value, DataValue::Integer(10));
    assert_eq!(doubled_value, DataValue::Integer(20)); // 10 * 2
}
