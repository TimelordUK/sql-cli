use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

/// Create a test table with numeric data
fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test_table");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("quantity"));
    table.add_column(DataColumn::new("price"));

    // Add test rows
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::Integer(10),
            DataValue::Float(25.50),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::Integer(5),
            DataValue::Float(100.00),
        ]))
        .unwrap();

    Arc::new(table)
}

#[test]
fn test_select_star_with_computed_column() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // This should expand * to all columns AND add the computed column
    let view = engine
        .execute(
            table.clone(),
            "SELECT *, quantity * price as total FROM test_table",
        )
        .unwrap();

    // Should have 4 columns: id, quantity, price, total
    assert_eq!(view.column_count(), 4);

    let columns = view.column_names();
    assert_eq!(columns, vec!["id", "quantity", "price", "total"]);

    // Check first row values
    let row = view.get_row(0).unwrap();
    assert_eq!(row.get(0).unwrap(), &DataValue::Integer(1)); // id
    assert_eq!(row.get(1).unwrap(), &DataValue::Integer(10)); // quantity
    assert_eq!(row.get(2).unwrap(), &DataValue::Float(25.50)); // price
    assert_eq!(row.get(3).unwrap(), &DataValue::Float(255.0)); // total = 10 * 25.50
}

#[test]
fn test_select_star_comma_arithmetic() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test the problematic case: SELECT *,quantity * price as total
    let result = engine.execute(
        table.clone(),
        "SELECT *,quantity * price as total FROM test_table",
    );

    match result {
        Ok(view) => {
            println!("Columns: {:?}", view.column_names());
            println!("Column count: {}", view.column_count());
            // Should have 4 columns: id, quantity, price, total
            assert_eq!(view.column_count(), 4);

            let columns = view.column_names();
            assert_eq!(columns, vec!["id", "quantity", "price", "total"]);

            // Check first row values
            let row = view.get_row(0).unwrap();
            assert_eq!(row.get(0).unwrap(), &DataValue::Integer(1)); // id
            assert_eq!(row.get(1).unwrap(), &DataValue::Integer(10)); // quantity
            assert_eq!(row.get(2).unwrap(), &DataValue::Float(25.50)); // price
            assert_eq!(row.get(3).unwrap(), &DataValue::Float(255.0)); // total
        }
        Err(e) => {
            panic!("Query failed: {e}");
        }
    }
}

#[test]
fn test_star_in_middle_of_select() {
    let table = create_test_table();
    let engine = QueryEngine::new();

    // Test: SELECT id, *, price * 2 as double_price
    let view = engine
        .execute(
            table.clone(),
            "SELECT id, *, price * 2 as double_price FROM test_table",
        )
        .unwrap();

    // Should have 5 columns: id (explicit), id (from *), quantity (from *), price (from *), double_price
    // But the first id might be deduplicated
    assert!(view.column_count() >= 4);

    let columns = view.column_names();
    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"quantity".to_string()));
    assert!(columns.contains(&"price".to_string()));
    assert!(columns.contains(&"double_price".to_string()));
}
