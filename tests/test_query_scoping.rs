use sql_cli::buffer::{Buffer, BufferAPI};
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

/// Create a test table with numeric data
fn create_test_table() -> DataTable {
    let mut table = DataTable::new("test_data");

    // Add columns
    table.add_column(DataColumn::new("a"));
    table.add_column(DataColumn::new("b"));
    table.add_column(DataColumn::new("c"));

    // Add test rows
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(10),
            DataValue::Integer(20),
            DataValue::Integer(30),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(5),
            DataValue::Integer(15),
            DataValue::Integer(25),
        ]))
        .unwrap();

    table
}

#[test]
fn test_query_scoping_preserves_original() {
    // Create initial table and set it in a buffer
    let table = create_test_table();
    let mut buffer = Buffer::new(1);
    buffer.set_datatable(Some(Arc::new(table)));

    // First query: SELECT with computed column
    let engine = QueryEngine::new();
    let table_arc = Arc::new(buffer.get_datatable().unwrap().clone());

    let view1 = engine
        .execute(
            table_arc.clone(),
            "SELECT a, a * 2 as double_a FROM test_data",
        )
        .unwrap();

    assert_eq!(view1.row_count(), 2);
    assert_eq!(view1.column_count(), 2);
    assert_eq!(view1.column_names(), vec!["a", "double_a"]);

    // Second query: SELECT different columns (should still work from original)
    let view2 = engine
        .execute(table_arc.clone(), "SELECT b, c FROM test_data")
        .unwrap();

    assert_eq!(view2.row_count(), 2);
    assert_eq!(view2.column_count(), 2);
    assert_eq!(view2.column_names(), vec!["b", "c"]);

    // Third query: SELECT all original columns (should all be available)
    let view3 = engine
        .execute(table_arc.clone(), "SELECT * FROM test_data")
        .unwrap();

    assert_eq!(view3.row_count(), 2);
    assert_eq!(view3.column_count(), 3);
    assert_eq!(view3.column_names(), vec!["a", "b", "c"]);

    // Verify original data is intact
    let row = view3.get_row(0).unwrap();
    assert_eq!(row.get(0).unwrap(), &DataValue::Integer(10));
    assert_eq!(row.get(1).unwrap(), &DataValue::Integer(20));
    assert_eq!(row.get(2).unwrap(), &DataValue::Integer(30));
}

#[test]
fn test_derived_columns_are_query_scoped() {
    let table = create_test_table();
    let engine = QueryEngine::new();
    let table_arc = Arc::new(table);

    // Query 1: Create a derived column "total"
    let view1 = engine
        .execute(
            table_arc.clone(),
            "SELECT a, b, a + b as total FROM test_data",
        )
        .unwrap();

    assert_eq!(view1.column_names(), vec!["a", "b", "total"]);

    // Query 2: Different query without "total" - it shouldn't exist
    let view2 = engine
        .execute(table_arc.clone(), "SELECT a, b, c FROM test_data")
        .unwrap();

    assert_eq!(view2.column_names(), vec!["a", "b", "c"]);
    // "total" column should not exist in this query
    assert!(!view2.column_names().contains(&"total".to_string()));

    // Query 3: Create a different derived column "product"
    let view3 = engine
        .execute(
            table_arc.clone(),
            "SELECT a, a * b as product FROM test_data",
        )
        .unwrap();

    assert_eq!(view3.column_names(), vec!["a", "product"]);
    // "total" from query 1 should not exist here
    assert!(!view3.column_names().contains(&"total".to_string()));
}

#[test]
fn test_original_source_preserved_in_buffer() {
    // Create a buffer with initial data
    let table = create_test_table();
    let mut buffer = Buffer::new(1);
    buffer.set_datatable(Some(Arc::new(table.clone())));

    // Check that original source is preserved
    let original = buffer.get_original_source().unwrap();
    assert_eq!(original.column_names(), vec!["a", "b", "c"]);
    assert_eq!(original.row_count(), 2);

    // Even after modifying the datatable, original should remain
    let mut modified_table = table.clone();
    modified_table.add_column(DataColumn::new("computed"));
    buffer.set_datatable(Some(Arc::new(modified_table)));

    // Original source should still have only 3 columns
    let original_after = buffer.get_original_source().unwrap();
    assert_eq!(original_after.column_names(), vec!["a", "b", "c"]);
    assert_eq!(original_after.row_count(), 2);
}
