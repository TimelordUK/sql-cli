use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use sql_cli::data::query_engine::QueryEngine;
use std::sync::Arc;

#[test]
fn test_multi_column_order_by_sql() {
    // Create a table with trades data
    let mut table = DataTable::new("trades");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("trader"));
    table.add_column(DataColumn::new("price"));
    table.add_column(DataColumn::new("quantity"));

    // Add test data with duplicate trader names to test secondary sorting
    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("Alice".to_string()),
            DataValue::Float(100.0),
            DataValue::Integer(10),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("Bob".to_string()),
            DataValue::Float(150.0),
            DataValue::Integer(5),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("Alice".to_string()),
            DataValue::Float(200.0),
            DataValue::Integer(8),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(4),
            DataValue::String("Bob".to_string()),
            DataValue::Float(120.0),
            DataValue::Integer(15),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(5),
            DataValue::String("Alice".to_string()),
            DataValue::Float(150.0),
            DataValue::Integer(12),
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
    assert_eq!(view.row_count(), 5);

    // Alice's trades should come first, sorted by price (100, 150, 200)
    let row0 = view.get_row(0).unwrap();
    assert_eq!(row0.values[1], DataValue::String("Alice".to_string()));
    assert_eq!(row0.values[2], DataValue::Float(100.0));

    let row1 = view.get_row(1).unwrap();
    assert_eq!(row1.values[1], DataValue::String("Alice".to_string()));
    assert_eq!(row1.values[2], DataValue::Float(150.0));

    let row2 = view.get_row(2).unwrap();
    assert_eq!(row2.values[1], DataValue::String("Alice".to_string()));
    assert_eq!(row2.values[2], DataValue::Float(200.0));

    // Bob's trades should come next, sorted by price (120, 150)
    let row3 = view.get_row(3).unwrap();
    assert_eq!(row3.values[1], DataValue::String("Bob".to_string()));
    assert_eq!(row3.values[2], DataValue::Float(120.0));

    let row4 = view.get_row(4).unwrap();
    assert_eq!(row4.values[1], DataValue::String("Bob".to_string()));
    assert_eq!(row4.values[2], DataValue::Float(150.0));
}

#[test]
fn test_multi_column_order_by_desc_asc() {
    // Create a table with mixed data
    let mut table = DataTable::new("data");
    table.add_column(DataColumn::new("category"));
    table.add_column(DataColumn::new("value"));
    table.add_column(DataColumn::new("name"));

    // Add test data
    table
        .add_row(DataRow::new(vec![
            DataValue::String("A".to_string()),
            DataValue::Integer(100),
            DataValue::String("Item1".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("B".to_string()),
            DataValue::Integer(50),
            DataValue::String("Item2".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("A".to_string()),
            DataValue::Integer(200),
            DataValue::String("Item3".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("B".to_string()),
            DataValue::Integer(150),
            DataValue::String("Item4".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("A".to_string()),
            DataValue::Integer(100),
            DataValue::String("Item5".to_string()),
        ]))
        .unwrap();

    let table_arc = Arc::new(table);
    let engine = QueryEngine::new();

    // Test ORDER BY category ASC, value DESC
    let view = engine
        .execute(
            table_arc.clone(),
            "SELECT * FROM data ORDER BY category ASC, value DESC",
        )
        .unwrap();

    // Verify results
    assert_eq!(view.row_count(), 5);

    // Category A should come first, sorted by value DESC (200, 100, 100)
    let row0 = view.get_row(0).unwrap();
    assert_eq!(row0.values[0], DataValue::String("A".to_string()));
    assert_eq!(row0.values[1], DataValue::Integer(200));

    let row1 = view.get_row(1).unwrap();
    assert_eq!(row1.values[0], DataValue::String("A".to_string()));
    assert_eq!(row1.values[1], DataValue::Integer(100));

    let row2 = view.get_row(2).unwrap();
    assert_eq!(row2.values[0], DataValue::String("A".to_string()));
    assert_eq!(row2.values[1], DataValue::Integer(100));

    // Category B should come next, sorted by value DESC (150, 50)
    let row3 = view.get_row(3).unwrap();
    assert_eq!(row3.values[0], DataValue::String("B".to_string()));
    assert_eq!(row3.values[1], DataValue::Integer(150));

    let row4 = view.get_row(4).unwrap();
    assert_eq!(row4.values[0], DataValue::String("B".to_string()));
    assert_eq!(row4.values[1], DataValue::Integer(50));
}

#[test]
fn test_three_column_order_by() {
    // Create a table with three levels of sorting
    let mut table = DataTable::new("data");
    table.add_column(DataColumn::new("department"));
    table.add_column(DataColumn::new("role"));
    table.add_column(DataColumn::new("salary"));
    table.add_column(DataColumn::new("name"));

    // Engineering department
    table
        .add_row(DataRow::new(vec![
            DataValue::String("Engineering".to_string()),
            DataValue::String("Senior".to_string()),
            DataValue::Integer(120000),
            DataValue::String("Alice".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Engineering".to_string()),
            DataValue::String("Junior".to_string()),
            DataValue::Integer(80000),
            DataValue::String("Bob".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Engineering".to_string()),
            DataValue::String("Senior".to_string()),
            DataValue::Integer(130000),
            DataValue::String("Charlie".to_string()),
        ]))
        .unwrap();

    // Sales department
    table
        .add_row(DataRow::new(vec![
            DataValue::String("Sales".to_string()),
            DataValue::String("Senior".to_string()),
            DataValue::Integer(110000),
            DataValue::String("David".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Sales".to_string()),
            DataValue::String("Junior".to_string()),
            DataValue::Integer(70000),
            DataValue::String("Eve".to_string()),
        ]))
        .unwrap();

    let table_arc = Arc::new(table);
    let engine = QueryEngine::new();

    // Test ORDER BY department, role DESC, salary DESC
    let view = engine
        .execute(
            table_arc.clone(),
            "SELECT * FROM data ORDER BY department, role DESC, salary DESC",
        )
        .unwrap();

    // Verify the complex sorting
    assert_eq!(view.row_count(), 5);

    // Engineering department first
    // Senior roles first (DESC), then by salary DESC
    let row0 = view.get_row(0).unwrap();
    assert_eq!(row0.values[0], DataValue::String("Engineering".to_string()));
    assert_eq!(row0.values[1], DataValue::String("Senior".to_string()));
    assert_eq!(row0.values[2], DataValue::Integer(130000)); // Charlie

    let row1 = view.get_row(1).unwrap();
    assert_eq!(row1.values[0], DataValue::String("Engineering".to_string()));
    assert_eq!(row1.values[1], DataValue::String("Senior".to_string()));
    assert_eq!(row1.values[2], DataValue::Integer(120000)); // Alice

    // Junior role
    let row2 = view.get_row(2).unwrap();
    assert_eq!(row2.values[0], DataValue::String("Engineering".to_string()));
    assert_eq!(row2.values[1], DataValue::String("Junior".to_string()));
    assert_eq!(row2.values[2], DataValue::Integer(80000)); // Bob

    // Sales department next
    // Senior role first
    let row3 = view.get_row(3).unwrap();
    assert_eq!(row3.values[0], DataValue::String("Sales".to_string()));
    assert_eq!(row3.values[1], DataValue::String("Senior".to_string()));
    assert_eq!(row3.values[2], DataValue::Integer(110000)); // David

    // Junior role
    let row4 = view.get_row(4).unwrap();
    assert_eq!(row4.values[0], DataValue::String("Sales".to_string()));
    assert_eq!(row4.values[1], DataValue::String("Junior".to_string()));
    assert_eq!(row4.values[2], DataValue::Integer(70000)); // Eve
}

#[test]
fn test_direct_multi_sort_method() {
    // Test the DataView::apply_multi_sort method directly
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("col1"));
    table.add_column(DataColumn::new("col2"));
    table.add_column(DataColumn::new("col3"));

    // Add rows with specific patterns for testing
    table
        .add_row(DataRow::new(vec![
            DataValue::String("B".to_string()),
            DataValue::Integer(2),
            DataValue::Float(3.0),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("A".to_string()),
            DataValue::Integer(2),
            DataValue::Float(1.0),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("B".to_string()),
            DataValue::Integer(1),
            DataValue::Float(2.0),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("A".to_string()),
            DataValue::Integer(2),
            DataValue::Float(2.0),
        ]))
        .unwrap();

    let mut view = DataView::new(Arc::new(table));

    // Sort by col1 ASC, col2 DESC, col3 ASC
    view.apply_multi_sort(&[(0, true), (1, false), (2, true)])
        .unwrap();

    // Verify the sorting
    // First "A" rows (col1 ASC)
    let row0 = view.get_row(0).unwrap();
    assert_eq!(row0.values[0], DataValue::String("A".to_string()));
    assert_eq!(row0.values[1], DataValue::Integer(2)); // col2 DESC
    assert_eq!(row0.values[2], DataValue::Float(1.0)); // col3 ASC

    let row1 = view.get_row(1).unwrap();
    assert_eq!(row1.values[0], DataValue::String("A".to_string()));
    assert_eq!(row1.values[1], DataValue::Integer(2));
    assert_eq!(row1.values[2], DataValue::Float(2.0));

    // Then "B" rows
    let row2 = view.get_row(2).unwrap();
    assert_eq!(row2.values[0], DataValue::String("B".to_string()));
    assert_eq!(row2.values[1], DataValue::Integer(2)); // col2 DESC, so 2 before 1
    assert_eq!(row2.values[2], DataValue::Float(3.0));

    let row3 = view.get_row(3).unwrap();
    assert_eq!(row3.values[0], DataValue::String("B".to_string()));
    assert_eq!(row3.values[1], DataValue::Integer(1));
    assert_eq!(row3.values[2], DataValue::Float(2.0));
}
