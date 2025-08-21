use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use std::sync::Arc;

#[test]
fn test_sort_with_interned_strings() {
    // Create a table with mixed String and InternedString values
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("category"));

    // Add rows with InternedString values for category (simulating memory optimization)
    let cat_a = Arc::new("Alpha".to_string());
    let cat_b = Arc::new("Beta".to_string());
    let cat_c = Arc::new("Charlie".to_string());

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("Item1".to_string()),
            DataValue::InternedString(cat_b.clone()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::String("Item2".to_string()),
            DataValue::InternedString(cat_c.clone()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("Item3".to_string()),
            DataValue::InternedString(cat_a.clone()),
        ]))
        .unwrap();

    // Create a DataView
    let mut view = DataView::new(Arc::new(table));

    // Sort by category (InternedString column) - ascending
    view.apply_sort(2, true).unwrap();

    // Verify the sort order
    let row0 = view.get_row(0).unwrap();
    let row1 = view.get_row(1).unwrap();
    let row2 = view.get_row(2).unwrap();

    // After sorting by category ascending, order should be: Alpha, Beta, Charlie
    assert_eq!(row0.values[0], DataValue::Integer(3)); // Item3 with Alpha
    assert_eq!(row1.values[0], DataValue::Integer(1)); // Item1 with Beta
    assert_eq!(row2.values[0], DataValue::Integer(2)); // Item2 with Charlie

    // Sort by category descending
    view.apply_sort(2, false).unwrap();

    let row0 = view.get_row(0).unwrap();
    let row1 = view.get_row(1).unwrap();
    let row2 = view.get_row(2).unwrap();

    // After sorting by category descending, order should be: Charlie, Beta, Alpha
    assert_eq!(row0.values[0], DataValue::Integer(2)); // Item2 with Charlie
    assert_eq!(row1.values[0], DataValue::Integer(1)); // Item1 with Beta
    assert_eq!(row2.values[0], DataValue::Integer(3)); // Item3 with Alpha
}

#[test]
fn test_sort_mixed_string_and_interned() {
    // Create a table with both String and InternedString values in the same column
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("value"));

    // Mix regular strings and interned strings
    let interned_b = Arc::new("Bravo".to_string());
    let interned_d = Arc::new("Delta".to_string());

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::String("Charlie".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(2),
            DataValue::InternedString(interned_b.clone()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("Alpha".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(4),
            DataValue::InternedString(interned_d.clone()),
        ]))
        .unwrap();

    // Create a DataView
    let mut view = DataView::new(Arc::new(table));

    // Sort by value column (mixed String/InternedString) - ascending
    view.apply_sort(1, true).unwrap();

    // Verify the sort order
    let row0 = view.get_row(0).unwrap();
    let row1 = view.get_row(1).unwrap();
    let row2 = view.get_row(2).unwrap();
    let row3 = view.get_row(3).unwrap();

    // After sorting ascending: Alpha, Bravo, Charlie, Delta
    assert_eq!(row0.values[0], DataValue::Integer(3)); // Alpha
    assert_eq!(row1.values[0], DataValue::Integer(2)); // Bravo (interned)
    assert_eq!(row2.values[0], DataValue::Integer(1)); // Charlie
    assert_eq!(row3.values[0], DataValue::Integer(4)); // Delta (interned)
}

#[test]
fn test_sort_with_nulls_and_interned() {
    // Test sorting with NULL values mixed with InternedString
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("value"));

    let interned = Arc::new("Middle".to_string());

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(1),
            DataValue::InternedString(interned.clone()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![DataValue::Integer(2), DataValue::Null]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::Integer(3),
            DataValue::String("Zebra".to_string()),
        ]))
        .unwrap();

    // Create a DataView
    let mut view = DataView::new(Arc::new(table));

    // Sort by value column - ascending (NULLs should come first)
    view.apply_sort(1, true).unwrap();

    let row0 = view.get_row(0).unwrap();
    let row1 = view.get_row(1).unwrap();
    let row2 = view.get_row(2).unwrap();

    // After sorting ascending: NULL, Middle, Zebra
    assert_eq!(row0.values[0], DataValue::Integer(2)); // NULL
    assert_eq!(row1.values[0], DataValue::Integer(1)); // Middle (interned)
    assert_eq!(row2.values[0], DataValue::Integer(3)); // Zebra
}
