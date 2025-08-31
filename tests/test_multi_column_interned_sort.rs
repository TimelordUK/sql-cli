use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

#[test]
fn test_multi_column_order_by_with_interned_trader() {
    // Create a table with trades data where trader column uses InternedString
    // This simulates real-world usage where repeated strings are interned for memory efficiency
    let mut table = DataTable::new("trades");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("trader"));
    table.add_column(DataColumn::new("price"));
    table.add_column(DataColumn::new("quantity"));

    // Create interned strings for trader names
    let alice_str = Arc::new("Alice".to_string());
    let bob_str = Arc::new("Bob".to_string());
    let charlie_str = Arc::new("Charlie".to_string());

    // Add test data with InternedString trader names
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::InternedString(alice_str.clone()),
            DataValue::Float(100.0),
            DataValue::Integer(10),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::InternedString(bob_str.clone()),
            DataValue::Float(150.0),
            DataValue::Integer(5),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::InternedString(alice_str.clone()),
            DataValue::Float(200.0),
            DataValue::Integer(8),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(4),
            DataValue::InternedString(bob_str.clone()),
            DataValue::Float(120.0),
            DataValue::Integer(15),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(5),
            DataValue::InternedString(alice_str.clone()),
            DataValue::Float(150.0),
            DataValue::Integer(12),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(6),
            DataValue::InternedString(charlie_str.clone()),
            DataValue::Float(175.0),
            DataValue::Integer(20),
        ]))
        .unwrap();

    let table_arc = Arc::new(table);
    let engine = QueryEngine::new();

    // Test ORDER BY trader, price
    let view = engine
        .execute(
            table_arc.clone(),
            "SELECT * FROM trades ORDER BY trader, price",
        )
        .unwrap();

    // Verify results are sorted by trader first, then by price
    assert_eq!(view.row_count(), 6);

    // Alice's trades should come first, sorted by price (100, 150, 200)
    let row0 = view.get_row(0).unwrap();
    assert_eq!(row0.values[1], DataValue::InternedString(alice_str.clone()));
    assert_eq!(row0.values[2], DataValue::Float(100.0));

    let row1 = view.get_row(1).unwrap();
    assert_eq!(row1.values[1], DataValue::InternedString(alice_str.clone()));
    assert_eq!(row1.values[2], DataValue::Float(150.0));

    let row2 = view.get_row(2).unwrap();
    assert_eq!(row2.values[1], DataValue::InternedString(alice_str.clone()));
    assert_eq!(row2.values[2], DataValue::Float(200.0));

    // Bob's trades should come next, sorted by price (120, 150)
    let row3 = view.get_row(3).unwrap();
    assert_eq!(row3.values[1], DataValue::InternedString(bob_str.clone()));
    assert_eq!(row3.values[2], DataValue::Float(120.0));

    let row4 = view.get_row(4).unwrap();
    assert_eq!(row4.values[1], DataValue::InternedString(bob_str.clone()));
    assert_eq!(row4.values[2], DataValue::Float(150.0));

    // Charlie's single trade should be last
    let row5 = view.get_row(5).unwrap();
    assert_eq!(
        row5.values[1],
        DataValue::InternedString(charlie_str.clone())
    );
    assert_eq!(row5.values[2], DataValue::Float(175.0));
}

#[test]
fn test_multi_column_order_by_desc_with_interned() {
    // Test ORDER BY with DESC on InternedString column
    let mut table = DataTable::new("trades");
    table.add_column(DataColumn::new("trader"));
    table.add_column(DataColumn::new("price"));

    // Create interned strings
    let alice_str = Arc::new("Alice".to_string());
    let bob_str = Arc::new("Bob".to_string());

    // Add test data
    table
        .add_row(DataRow::new(vec![
            DataValue::InternedString(alice_str.clone()),
            DataValue::Float(100.0),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::InternedString(bob_str.clone()),
            DataValue::Float(200.0),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::InternedString(alice_str.clone()),
            DataValue::Float(150.0),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::InternedString(bob_str.clone()),
            DataValue::Float(100.0),
        ]))
        .unwrap();

    let table_arc = Arc::new(table);
    let engine = QueryEngine::new();

    // Test ORDER BY trader DESC, price ASC
    let view = engine
        .execute(
            table_arc.clone(),
            "SELECT * FROM trades ORDER BY trader DESC, price ASC",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);

    // Bob's trades should come first (DESC), sorted by price ASC (100, 200)
    let row0 = view.get_row(0).unwrap();
    assert_eq!(row0.values[0], DataValue::InternedString(bob_str.clone()));
    assert_eq!(row0.values[1], DataValue::Float(100.0));

    let row1 = view.get_row(1).unwrap();
    assert_eq!(row1.values[0], DataValue::InternedString(bob_str.clone()));
    assert_eq!(row1.values[1], DataValue::Float(200.0));

    // Alice's trades should come next, sorted by price ASC (100, 150)
    let row2 = view.get_row(2).unwrap();
    assert_eq!(row2.values[0], DataValue::InternedString(alice_str.clone()));
    assert_eq!(row2.values[1], DataValue::Float(100.0));

    let row3 = view.get_row(3).unwrap();
    assert_eq!(row3.values[0], DataValue::InternedString(alice_str.clone()));
    assert_eq!(row3.values[1], DataValue::Float(150.0));
}

#[test]
fn test_mixed_string_and_interned_multi_sort() {
    // Test multi-column sort with mixed String and InternedString values
    let mut table = DataTable::new("data");
    table.add_column(DataColumn::new("category"));
    table.add_column(DataColumn::new("trader"));
    table.add_column(DataColumn::new("value"));

    // Mix regular strings for category and interned strings for trader
    let alice_str = Arc::new("Alice".to_string());
    let bob_str = Arc::new("Bob".to_string());

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Sales".to_string()),
            DataValue::InternedString(bob_str.clone()),
            DataValue::Integer(100),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Tech".to_string()),
            DataValue::InternedString(alice_str.clone()),
            DataValue::Integer(200),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Sales".to_string()),
            DataValue::InternedString(alice_str.clone()),
            DataValue::Integer(150),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Tech".to_string()),
            DataValue::InternedString(bob_str.clone()),
            DataValue::Integer(100),
        ]))
        .unwrap();

    let table_arc = Arc::new(table);
    let engine = QueryEngine::new();

    // Test ORDER BY category, trader, value
    let view = engine
        .execute(
            table_arc.clone(),
            "SELECT * FROM data ORDER BY category, trader, value",
        )
        .unwrap();

    assert_eq!(view.row_count(), 4);

    // Sales category first
    let row0 = view.get_row(0).unwrap();
    assert_eq!(row0.values[0], DataValue::String("Sales".to_string()));
    assert_eq!(row0.values[1], DataValue::InternedString(alice_str.clone()));
    assert_eq!(row0.values[2], DataValue::Integer(150));

    let row1 = view.get_row(1).unwrap();
    assert_eq!(row1.values[0], DataValue::String("Sales".to_string()));
    assert_eq!(row1.values[1], DataValue::InternedString(bob_str.clone()));
    assert_eq!(row1.values[2], DataValue::Integer(100));

    // Tech category next
    let row2 = view.get_row(2).unwrap();
    assert_eq!(row2.values[0], DataValue::String("Tech".to_string()));
    assert_eq!(row2.values[1], DataValue::InternedString(alice_str.clone()));
    assert_eq!(row2.values[2], DataValue::Integer(200));

    let row3 = view.get_row(3).unwrap();
    assert_eq!(row3.values[0], DataValue::String("Tech".to_string()));
    assert_eq!(row3.values[1], DataValue::InternedString(bob_str.clone()));
    assert_eq!(row3.values[2], DataValue::Integer(100));
}
