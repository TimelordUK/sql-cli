// DataView Debug Binary - For testing DataView in isolation
// This binary is properly registered in Cargo.toml
// Run with: cargo run --bin dataview_debug

use std::sync::Arc;

// We'll import directly from the data modules to avoid TUI compilation issues
// This lets us test DataView while TUI is being refactored

fn main() {
    use sql_cli::data::data_view::DataView;
    use sql_cli::data::datatable::{DataColumn, DataRow, DataTable, DataValue};

    println!("=== DataView Debug (Cargo Binary) ===");
    println!("This version works with RustRover's debugger!");
    println!("Set breakpoints in src/data/data_view.rs\n");

    // Create test data
    let mut table = DataTable::new("test_data");

    // Add columns
    table.add_column(DataColumn::new("id"));
    table.add_column(DataColumn::new("name"));
    table.add_column(DataColumn::new("amount"));
    table.add_column(DataColumn::new("category"));
    table.add_column(DataColumn::new("active"));

    // Add sample rows
    let rows = vec![
        (1, "Alice", 100.50, "Sales", true),
        (2, "Bob", 200.75, "Marketing", false),
        (3, "Charlie", 150.25, "Sales", true),
        (4, "David", 300.00, "Engineering", true),
        (5, "Eve", 175.50, "Marketing", false),
        (6, "Frank", 250.00, "Sales", false),
        (7, "Grace", 180.00, "Engineering", false),
    ];

    for (id, name, amount, category, active) in rows {
        let row = DataRow::new(vec![
            DataValue::String(id.to_string()),
            DataValue::String(name.to_string()),
            DataValue::Float(amount),
            DataValue::String(category.to_string()),
            DataValue::Boolean(active),
        ]);
        table.add_row(row).unwrap();
    }

    let table_arc = Arc::new(table);
    let mut view = DataView::new(table_arc);

    println!("📋 Initial state:");
    println!("  Rows: {}", view.row_count());
    println!("  Columns: {}", view.column_count());
    println!("  Column names: {:?}\n", view.column_names());

    // Test 1: Column Search
    println!("Test 1: Column Search");
    println!("────────────────────");

    view.search_columns("a");
    println!("  Searching for 'a':");
    println!("    Matches: {:?}", view.get_matching_columns());
    println!("    Current: {:?}", view.get_current_column_match());

    view.next_column_match();
    println!("  After next_match:");
    println!("    Current: {:?}", view.get_current_column_match());

    view.clear_column_search();
    println!("  After clear:");
    println!("    Has search: {}\n", view.has_column_search());

    // Test 2: Text Filtering
    println!("Test 2: Text Filtering");
    println!("──────────────────────");

    view.apply_text_filter("Sales", false);
    println!("  Filter 'Sales': {} rows", view.row_count());

    for i in 0..view.row_count().min(3) {
        if let Some(row) = view.get_row(i) {
            let values: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
            println!("    Row {}: {}", i, values.join(" | "));
        }
    }

    view.clear_filter();
    println!("  After clear: {} rows\n", view.row_count());

    // Test 3: Fuzzy Filtering
    println!("Test 3: Fuzzy Filtering");
    println!("───────────────────────");

    view.apply_fuzzy_filter("mrk", true);
    println!("  Fuzzy 'mrk': {} rows", view.row_count());

    view.clear_filter();
    view.apply_fuzzy_filter("'Engineering", false);
    println!("  Exact 'Engineering': {} rows", view.row_count());

    for i in 0..view.row_count() {
        if let Some(row) = view.get_row(i) {
            let values: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
            println!("    {}", values.join(" | "));
        }
    }

    view.clear_filter();
    println!();

    // Test 4: Sorting
    println!("Test 4: Sorting");
    println!("───────────────");

    view.apply_sort(2, false).unwrap(); // Sort by amount descending
    println!("  Sorted by amount (DESC):");

    for i in 0..3 {
        if let Some(row) = view.get_row(i) {
            let values: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
            println!("    {}", values.join(" | "));
        }
    }
    println!();

    // Test 5: Column Visibility
    println!("Test 5: Column Visibility");
    println!("─────────────────────────");

    println!("  Visible: {:?}", view.column_names());

    view.hide_column_by_name("active");
    println!("  After hiding 'active': {:?}", view.column_names());
    println!("  Hidden: {:?}", view.get_hidden_column_names());

    view.move_column_left_by_name("amount");
    println!("  After moving 'amount' left: {:?}", view.column_names());

    view.unhide_all_columns();
    println!("  After unhide all: {:?}\n", view.column_names());

    // Test 6: Combined Operations
    println!("Test 6: Combined Operations");
    println!("───────────────────────────");

    view.apply_sort(2, false).unwrap();
    println!("  1. Sorted by amount DESC");

    view.apply_text_filter("Sales", false);
    println!("  2. Filtered for 'Sales': {} rows", view.row_count());

    view.search_columns("name");
    println!(
        "  3. Column search 'name': {:?}",
        view.get_matching_columns()
    );

    println!("\n  Final result (Sales people sorted by amount):");
    for i in 0..view.row_count() {
        if let Some(row) = view.get_row(i) {
            let values: Vec<String> = row.values.iter().map(|v| v.to_string()).collect();
            println!("    {}", values.join(" | "));
        }
    }

    view.clear_filter();
    println!(
        "\n  After clear filter (sort remains): {} rows",
        view.row_count()
    );
    if let Some(row) = view.get_row(0) {
        println!("  First row: {:?}", row.values.get(2)); // Should still be highest amount
    }

    // Test 7: Export Functions
    println!("\nTest 7: Export Functions");
    println!("────────────────────────");

    view.apply_text_filter("Engineering", false);
    view.hide_column_by_name("active");

    println!("  Setup: Filtered to Engineering, hidden 'active'");

    let csv = view.to_csv().unwrap();
    println!("  CSV (first 150 chars):");
    println!("    {}", &csv[..csv.len().min(150)]);

    let json = view.to_json();
    println!(
        "  JSON array length: {}",
        json.as_array().map(|a| a.len()).unwrap_or(0)
    );

    println!("\n✅ All DataView tests complete!");
    println!("\n🐛 To debug in RustRover:");
    println!("  1. Set breakpoints in src/data/data_view.rs");
    println!("  2. Right-click this file → Debug 'dataview_debug'");
    println!("  3. Or use the Run/Debug configurations dropdown");
}
