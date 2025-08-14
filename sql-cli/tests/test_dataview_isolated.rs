// Isolated DataView test that can run independently
// This test file can be run directly in RustRover:
// Right-click → Run 'test_dataview_isolated'

// Include the modules directly to bypass lib compilation
#[path = "../src/data/datatable.rs"]
mod datatable;

#[path = "../src/data/data_view.rs"]
mod data_view;

#[path = "../src/data/data_provider.rs"]
mod data_provider;

use data_view::DataView;
use datatable::{DataColumn, DataRow, DataTable, DataValue};
use std::sync::Arc;

#[test]
fn test_column_search() {
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("amount"));
    table.add_column(DataColumn::new("category"));

    let mut view = DataView::new(Arc::new(table));

    // Test searching for 'a'
    view.search_columns("a");
    let matches = view.get_matching_columns();
    assert_eq!(matches.len(), 3); // name, amount, category
    assert_eq!(matches[0].1, "name");
    assert_eq!(matches[1].1, "amount");
    assert_eq!(matches[2].1, "category");

    // Test next match
    let first = view.get_current_column_match();
    assert_eq!(first, Some(1)); // index of 'name'

    let next = view.next_column_match();
    assert_eq!(next, Some(2)); // index of 'amount'

    // Clear search
    view.clear_column_search();
    assert!(!view.has_column_search());
    assert_eq!(view.get_matching_columns().len(), 0);
}

#[test]
fn test_text_filtering() {
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("category"));

    // Add test data
    table
        .add_row(DataRow::new(vec![
            DataValue::String("Alice".to_string()),
            DataValue::String("Sales".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Bob".to_string()),
            DataValue::String("Marketing".to_string()),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Charlie".to_string()),
            DataValue::String("Sales".to_string()),
        ]))
        .unwrap();

    let mut view = DataView::new(Arc::new(table));
    assert_eq!(view.row_count(), 3);

    // Filter for "Sales"
    view.apply_text_filter("Sales", false);
    assert_eq!(view.row_count(), 2);

    // Check the filtered rows
    let row1 = view.get_row(0).unwrap();
    assert_eq!(row1.values[1].to_string(), "Sales");

    // Clear filter
    view.clear_filter();
    assert_eq!(view.row_count(), 3);
}

#[test]
fn test_column_visibility() {
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("amount"));
    table.add_column(DataColumn::new("active"));

    let mut view = DataView::new(Arc::new(table));
    assert_eq!(view.column_count(), 4);

    // Hide a column
    view.hide_column_by_name("active");
    assert_eq!(view.column_count(), 3);
    assert_eq!(view.column_names(), vec!["id", "name", "amount"]);

    let hidden = view.get_hidden_column_names();
    assert_eq!(hidden, vec!["active"]);

    // Unhide all
    view.unhide_all_columns();
    assert_eq!(view.column_count(), 4);
}

#[test]
fn test_sorting() {
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("amount"));

    // Add test data
    table
        .add_row(DataRow::new(vec![
            DataValue::String("Alice".to_string()),
            DataValue::Float(100.0),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Bob".to_string()),
            DataValue::Float(300.0),
        ]))
        .unwrap();

    table
        .add_row(DataRow::new(vec![
            DataValue::String("Charlie".to_string()),
            DataValue::Float(200.0),
        ]))
        .unwrap();

    let mut view = DataView::new(Arc::new(table));

    // Sort by amount descending
    view.apply_sort(1, false).unwrap();

    // Check order
    let row1 = view.get_row(0).unwrap();
    if let DataValue::Float(amount) = &row1.values[1] {
        assert_eq!(*amount, 300.0); // Bob has highest
    }

    let row2 = view.get_row(1).unwrap();
    if let DataValue::Float(amount) = &row2.values[1] {
        assert_eq!(*amount, 200.0); // Charlie is second
    }
}

#[test]
fn test_combined_operations() {
    let mut table = DataTable::new("test");
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("amount"));
    table.add_column(DataColumn::new("category"));

    // Add test data
    let data = vec![
        ("Alice", 100.0, "Sales"),
        ("Bob", 300.0, "Marketing"),
        ("Charlie", 200.0, "Sales"),
        ("David", 150.0, "Sales"),
    ];

    for (name, amount, category) in data {
        table
            .add_row(DataRow::new(vec![
                DataValue::String(name.to_string()),
                DataValue::Float(amount),
                DataValue::String(category.to_string()),
            ]))
            .unwrap();
    }

    let mut view = DataView::new(Arc::new(table));

    // 1. Sort by amount descending
    view.apply_sort(1, false).unwrap();

    // 2. Filter for Sales
    view.apply_text_filter("Sales", false);
    assert_eq!(view.row_count(), 3); // Only Sales people

    // 3. Search columns for "name"
    view.search_columns("name");
    assert_eq!(view.get_matching_columns().len(), 1);

    // Check that Charlie is first (highest Sales amount)
    let first_row = view.get_row(0).unwrap();
    assert_eq!(first_row.values[0].to_string(), "Charlie");

    // Clear filter - sort should remain
    view.clear_filter();
    assert_eq!(view.row_count(), 4);

    // Bob should be first now (highest overall)
    let first_row = view.get_row(0).unwrap();
    assert_eq!(first_row.values[0].to_string(), "Bob");
}

// Main function for debugging - set breakpoints here!
#[test]
fn debug_dataview() {
    println!("\n=== DataView Debug Test ===");
    println!("Set breakpoints in this function to debug!");

    let mut table = DataTable::new("debug_test");
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("category"));

    // Add sample data
    for i in 1..=5 {
        table
            .add_row(DataRow::new(vec![
                DataValue::String(i.to_string()),
                DataValue::String(format!("Item{}", i)),
                DataValue::String(if i % 2 == 0 { "Even" } else { "Odd" }.to_string()),
            ]))
            .unwrap();
    }

    let mut view = DataView::new(Arc::new(table));

    // BREAKPOINT HERE - Inspect initial state
    println!(
        "Initial: {} rows, {} columns",
        view.row_count(),
        view.column_count()
    );

    // BREAKPOINT HERE - Watch column search
    view.search_columns("a");
    println!("Column matches: {:?}", view.get_matching_columns());

    // BREAKPOINT HERE - Watch filtering
    view.apply_text_filter("Even", false);
    println!("Filtered: {} rows", view.row_count());

    // BREAKPOINT HERE - Inspect final state
    view.clear_filter();
    println!("Final: {} rows", view.row_count());
}
