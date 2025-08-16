// Test column search functionality with hidden columns

use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};
use std::sync::Arc;

fn create_test_table() -> Arc<DataTable> {
    let mut table = DataTable::new("test");

    // Add columns with "order" in their names
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("orderid"));
    table.add_column(DataColumn::new("externalOrderId"));
    table.add_column(DataColumn::new("parentOrderId"));
    table.add_column(DataColumn::new("platformOrderId"));
    table.add_column(DataColumn::new("status"));
    table.add_column(DataColumn::new("date"));

    // Add sample data
    for i in 1..=5 {
        table
            .add_row(DataRow::new(vec![
                DataValue::Integer(i),
                DataValue::String(format!("Item{}", i)),
                DataValue::String(format!("ORD{:03}", i)),
                DataValue::String(format!("EXT{:03}", i)),
                DataValue::String(format!("PAR{:03}", i)),
                DataValue::String(format!("PLAT{:03}", i)),
                DataValue::String("Active".to_string()),
                DataValue::String(format!("2024-01-{:02}", i)),
            ]))
            .unwrap();
    }

    Arc::new(table)
}

#[test]
fn test_column_search_basic() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Search for columns containing "order"
    view.search_columns("order");

    // Should find 4 columns: orderid, externalOrderId, parentOrderId, platformOrderId
    let matches = view.get_matching_columns();
    assert_eq!(matches.len(), 4);

    // Check the column names
    let match_names: Vec<String> = matches.iter().map(|(_, name)| name.clone()).collect();
    assert!(match_names.contains(&"orderid".to_string()));
    assert!(match_names.contains(&"externalOrderId".to_string()));
    assert!(match_names.contains(&"parentOrderId".to_string()));
    assert!(match_names.contains(&"platformOrderId".to_string()));

    // Check that get_current_column_match returns a valid display index
    let current_match = view.get_current_column_match();
    assert!(current_match.is_some());
    let idx = current_match.unwrap();
    // The index should be within the visible columns range
    assert!(idx < view.column_count());
}

#[test]
fn test_column_search_with_hidden_columns() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Hide some columns (by display index):
    // 0=id, 1=name, 2=orderid, 3=externalOrderId, 4=parentOrderId, 5=platformOrderId, 6=status, 7=date
    view.hide_column(1); // Hide "name"
    view.hide_column(6); // Hide "status" (now at index 5 after hiding name)
    view.hide_column(5); // Hide "date" (now at index 5 after hiding status)

    // Now visible columns should be: id, orderid, externalOrderId, parentOrderId, platformOrderId
    assert_eq!(view.column_count(), 5);
    let visible_names = view.get_display_column_names();
    assert_eq!(
        visible_names,
        vec![
            "id",
            "orderid",
            "externalOrderId",
            "parentOrderId",
            "platformOrderId"
        ]
    );

    // Search for "order" - should still find the 4 order columns
    view.search_columns("order");
    let matches = view.get_matching_columns();
    assert_eq!(matches.len(), 4);

    // Get current match - should be a display index
    let current_match = view.get_current_column_match();
    assert!(current_match.is_some());
    let idx = current_match.unwrap();

    // The index should be 1 (orderid is at display index 1 after hiding name)
    assert_eq!(idx, 1);

    // Navigate to next match
    view.next_column_match();
    let next_idx = view.get_current_column_match().unwrap();
    assert_eq!(next_idx, 2); // externalOrderId at display index 2

    // Navigate through all matches
    view.next_column_match();
    assert_eq!(view.get_current_column_match().unwrap(), 3); // parentOrderId

    view.next_column_match();
    assert_eq!(view.get_current_column_match().unwrap(), 4); // platformOrderId

    // Should wrap around to first match
    view.next_column_match();
    assert_eq!(view.get_current_column_match().unwrap(), 1); // back to orderid
}

#[test]
fn test_column_search_navigation() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Search for "order"
    view.search_columns("order");
    let matches = view.get_matching_columns();
    assert_eq!(matches.len(), 4);

    // Test navigation
    let first = view.get_current_column_match().unwrap();
    assert_eq!(first, 2); // orderid at display index 2

    // Next
    view.next_column_match();
    assert_eq!(view.get_current_column_match().unwrap(), 3); // externalOrderId

    // Previous
    view.prev_column_match();
    assert_eq!(view.get_current_column_match().unwrap(), 2); // back to orderid

    // Previous from first should wrap to last
    view.prev_column_match();
    assert_eq!(view.get_current_column_match().unwrap(), 5); // platformOrderId (last match)
}

#[test]
fn test_column_search_case_insensitive() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Search with different cases
    view.search_columns("ORDER");
    assert_eq!(view.get_matching_columns().len(), 4);

    view.search_columns("Order");
    assert_eq!(view.get_matching_columns().len(), 4);

    view.search_columns("OrDeR");
    assert_eq!(view.get_matching_columns().len(), 4);
}

#[test]
fn test_column_search_clear() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Search for something
    view.search_columns("order");
    assert_eq!(view.get_matching_columns().len(), 4);
    assert!(view.has_column_search());

    // Clear search
    view.search_columns("");
    assert_eq!(view.get_matching_columns().len(), 0);
    assert!(!view.has_column_search());
    assert!(view.get_current_column_match().is_none());
}

#[test]
fn test_column_search_no_matches() {
    let table = create_test_table();
    let mut view = DataView::new(table);

    // Search for something that doesn't exist
    view.search_columns("xyz");
    assert_eq!(view.get_matching_columns().len(), 0);
    assert!(view.get_current_column_match().is_none());
}
