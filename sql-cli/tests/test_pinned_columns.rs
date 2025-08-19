// Test pinned columns functionality in DataView

use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use std::sync::Arc;

fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test");

    // Add 6 columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("amount"));
    table.add_column(DataColumn::new("category"));
    table.add_column(DataColumn::new("status"));
    table.add_column(DataColumn::new("date"));

    // Add sample data
    for i in 1..=5 {
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(i),
                DataValue::String(format!("Item{}", i)),
                DataValue::Float(100.0 * i as f64),
                DataValue::String(if i % 2 == 0 { "A" } else { "B" }.to_string()),
                DataValue::String("Active".to_string()),
                DataValue::String("2024-01-01".to_string()),
            ]))
            .unwrap();
    }

    Arc::new(table)
}

#[test]
fn test_pin_column_basic() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Initially no pinned columns
    assert_eq!(view.get_pinned_columns().len(), 0);
    assert_eq!(view.column_count(), 6);
    assert_eq!(
        view.column_names(),
        vec!["id", "name", "amount", "category", "status", "date"]
    );

    // Pin the "id" column
    view.pin_column(0).unwrap();
    assert_eq!(view.get_pinned_columns(), &[0]);
    assert_eq!(view.column_count(), 6); // Still 6 columns total

    // Column names should have id first (pinned), then others
    assert_eq!(
        view.column_names(),
        vec!["id", "name", "amount", "category", "status", "date"]
    );

    // Pin "amount" column
    view.pin_column(2).unwrap();
    assert_eq!(view.get_pinned_columns(), &[0, 2]);

    // Display order: pinned first, then remaining visible
    assert_eq!(
        view.column_names(),
        vec!["id", "amount", "name", "category", "status", "date"]
    );
}

#[test]
fn test_pin_column_by_name() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Pin by name
    view.pin_column_by_name("name").unwrap();
    assert_eq!(view.get_pinned_column_names(), vec!["name"]);

    view.pin_column_by_name("status").unwrap();
    assert_eq!(view.get_pinned_column_names(), vec!["name", "status"]);

    // Try to pin non-existent column
    assert!(view.pin_column_by_name("nonexistent").is_err());
}

#[test]
fn test_max_pinned_columns() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Default max is 4
    view.pin_column(0).unwrap();
    view.pin_column(1).unwrap();
    view.pin_column(2).unwrap();
    view.pin_column(3).unwrap();

    // Try to pin a 5th column - should fail
    assert!(view.pin_column(4).is_err());
    assert_eq!(view.get_pinned_columns().len(), 4);

    // Change max to 2
    view.set_max_pinned_columns(2);
    // Should automatically unpin the last 2
    assert_eq!(view.get_pinned_columns().len(), 2);
    assert_eq!(view.get_pinned_columns(), &[0, 1]);
}

#[test]
#[ignore = "Test needs update: pin/hide operations now use display indices, not source indices"]
fn test_cannot_hide_pinned_column() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Pin the "name" column
    view.pin_column(1).unwrap();

    // Try to hide it - should fail
    let hidden = view.hide_column(1);
    assert!(!hidden);

    // Column should still be visible
    assert!(view.is_column_visible(1));
    assert!(view.column_names().contains(&"name".to_string()));

    // Hide a non-pinned column - should work
    let hidden = view.hide_column(3);
    assert!(hidden);
    assert!(!view.column_names().contains(&"category".to_string()));
}

#[test]
fn test_unpin_column() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Pin some columns
    view.pin_column(0).unwrap();
    view.pin_column(2).unwrap();
    assert_eq!(view.get_pinned_columns().len(), 2);

    // Unpin one
    assert!(view.unpin_column(0));
    assert_eq!(view.get_pinned_columns(), &[2]);

    // It should be back in visible columns
    assert!(view.is_column_visible(0));

    // Unpin by name
    assert!(view.unpin_column_by_name("amount"));
    assert_eq!(view.get_pinned_columns().len(), 0);
}

#[test]
fn test_move_pinned_columns() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Pin 3 columns
    view.pin_column(0).unwrap(); // id
    view.pin_column(1).unwrap(); // name
    view.pin_column(2).unwrap(); // amount

    assert_eq!(view.get_pinned_column_names(), vec!["id", "name", "amount"]);

    // Move within pinned area - move "name" left (swaps with "id")
    view.move_column_left(1);
    assert_eq!(view.get_pinned_column_names(), vec!["name", "id", "amount"]);

    // Move "amount" right (wraps to first position)
    view.move_column_right(2);
    assert_eq!(view.get_pinned_column_names(), vec!["amount", "name", "id"]);

    // Moving at boundary between pinned and visible should work correctly
    let all_names = view.column_names();
    assert_eq!(all_names[0..3], ["amount", "name", "id"]);
    assert_eq!(all_names[3], "category"); // First unpinned column
}

#[test]
fn test_clear_pinned_columns() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Pin some columns
    view.pin_column(0).unwrap();
    view.pin_column(2).unwrap();
    view.pin_column(4).unwrap();
    assert_eq!(view.get_pinned_columns().len(), 3);

    // Clear all
    view.clear_pinned_columns();
    assert_eq!(view.get_pinned_columns().len(), 0);

    // All columns should still be visible
    assert_eq!(view.column_count(), 6);
}

#[test]
fn test_pinned_columns_with_filtering() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Pin "id" and "name"
    view.pin_column(0).unwrap();
    view.pin_column(1).unwrap();

    // Apply a filter
    view.apply_text_filter("Item2", false);
    assert_eq!(view.row_count(), 1);

    // Check that pinned columns are still first
    let row = view.get_row(0).unwrap();
    assert_eq!(row.values[0], DataValue::Integer(2)); // id = 2
    assert_eq!(row.values[1], DataValue::String("Item2".to_string())); // name = Item2

    // Column order should be preserved
    assert_eq!(
        view.column_names(),
        vec!["id", "name", "amount", "category", "status", "date"]
    );
}

#[test]
fn test_pinned_columns_with_sorting() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Pin "name" column
    view.pin_column(1).unwrap();

    // Sort by amount (which is not pinned)
    view.apply_sort(2, false).unwrap(); // Sort by original index

    // First row should have highest amount
    let row = view.get_row(0).unwrap();
    // Column order: name (pinned), id, amount, category, status, date
    assert_eq!(row.values[0], DataValue::String("Item5".to_string())); // name
    assert_eq!(row.values[2], DataValue::Float(500.0)); // amount is now at index 2 in display

    // Verify column order
    assert_eq!(view.column_names()[0], "name"); // Pinned first
}

#[test]
fn test_combined_operations() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // 1. Pin some columns
    view.pin_column_by_name("id").unwrap();
    view.pin_column_by_name("amount").unwrap();

    // 2. Hide a non-pinned column
    view.hide_column_by_name("date");

    // 3. Apply sorting
    view.apply_sort(1, false).unwrap(); // Sort by amount descending (visible index 1)

    // 4. Apply filter
    view.apply_text_filter("Active", false);

    // Check results
    assert_eq!(view.row_count(), 5); // All rows have "Active"
    assert_eq!(view.column_count(), 5); // 6 - 1 hidden

    // Column order: id (pinned), amount (pinned), name, category, status
    let columns = view.column_names();
    assert_eq!(columns, vec!["id", "amount", "name", "category", "status"]);

    // First row should have highest amount
    let first_row = view.get_row(0).unwrap();
    assert_eq!(first_row.values[0], DataValue::Integer(5)); // id
    assert_eq!(first_row.values[1], DataValue::Float(500.0)); // amount
}

#[test]
fn test_get_display_columns() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Pin columns 1 and 3
    view.pin_column(1).unwrap(); // name
    view.pin_column(3).unwrap(); // category

    // Hide column 5
    view.hide_column(5); // date

    // Display columns should be: 1, 3 (pinned), then 0, 2, 4 (visible, excluding 5)
    let display = view.get_display_columns();
    assert_eq!(display, vec![1, 3, 0, 2, 4]);

    let names = view.get_display_column_names();
    assert_eq!(names, vec!["name", "category", "id", "amount", "status"]);
}

#[test]
#[ignore = "Test needs update: pin/hide operations now use display indices, not source indices"]
fn test_export_with_pinned_columns() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Pin "amount" and "name"
    view.pin_column(2).unwrap(); // amount
    view.pin_column(1).unwrap(); // name

    // Hide "date"
    view.hide_column(5);

    // Export to CSV
    let csv = view.to_csv().unwrap();
    let lines: Vec<&str> = csv.lines().collect();

    // Header should reflect display order
    assert_eq!(lines[0], "amount,name,id,category,status");

    // First data row
    assert_eq!(lines[1], "100,Item1,1,B,Active");
}

#[test]
fn test_wraparound_navigation() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Pin first two columns
    view.pin_column(0).unwrap(); // id
    view.pin_column(1).unwrap(); // name

    // Current order: id, name (pinned) | amount, category, status, date (visible)

    // Move first unpinned column (amount, display index 2) left
    // Should wrap to end of visible columns
    view.move_column_left(2);
    let names = view.column_names();
    assert_eq!(names[names.len() - 1], "amount"); // amount moved to end

    // Move last column right - should wrap to first unpinned position
    view.move_column_right(names.len() - 1);
    let names = view.column_names();
    assert_eq!(names[2], "amount"); // Back at first unpinned position
}
