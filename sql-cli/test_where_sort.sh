#!/bin/bash

# Test that WHERE clause filtering is preserved when clearing sort

echo "Testing WHERE clause preservation during sort cycling..."

# Create a test binary that will:
# 1. Execute a SELECT with WHERE clause
# 2. Apply sorting
# 3. Clear sorting
# 4. Verify the filtered rows are preserved

cat << 'EOF' > test_where_sort_bug.rs
use sql_cli::data::{datatable::DataTable, data_view::DataView, query_engine::QueryEngine};
use std::sync::Arc;

fn main() {
    // Load test data
    let csv_content = include_str!("test_where_sort.csv");
    let table = DataTable::from_csv(csv_content).unwrap();
    let table_arc = Arc::new(table);
    
    // Create query engine and execute a WHERE clause query
    let engine = QueryEngine::new();
    let query = "SELECT * FROM data WHERE age > 30";
    
    println!("Executing query: {}", query);
    let result = engine.execute_query(table_arc.clone(), query).unwrap();
    let mut view = result;
    
    println!("Initial filtered rows (age > 30): {} rows", view.row_count());
    
    // Get the initial row indices (should be filtered)
    let initial_rows: Vec<usize> = (0..view.row_count())
        .map(|i| view.get_row_index(i).unwrap())
        .collect();
    println!("Initial row indices: {:?}", initial_rows);
    
    // Apply sorting on the 'name' column
    println!("\nApplying sort on 'name' column...");
    view.cycle_sort(1).unwrap(); // name is column 1
    
    let sorted_rows: Vec<usize> = (0..view.row_count())
        .map(|i| view.get_row_index(i).unwrap())
        .collect();
    println!("After sorting - row count: {}, indices: {:?}", view.row_count(), sorted_rows);
    
    // Cycle sort again (descending)
    println!("\nCycling to descending sort...");
    view.cycle_sort(1).unwrap();
    
    // Cycle sort again to clear (None)
    println!("\nCycling to clear sort (None)...");
    view.cycle_sort(1).unwrap();
    
    let final_rows: Vec<usize> = (0..view.row_count())
        .map(|i| view.get_row_index(i).unwrap())
        .collect();
    println!("After clearing sort - row count: {}, indices: {:?}", view.row_count(), final_rows);
    
    // Verify the WHERE clause is still applied
    if view.row_count() == 5 {
        println!("\n✓ SUCCESS: WHERE clause preserved! Still have 5 filtered rows (age > 30)");
        
        // Double-check by looking at actual ages
        for i in 0..view.row_count() {
            let row_idx = view.get_row_index(i).unwrap();
            let age_val = table_arc.get_cell_value(row_idx, 2); // age is column 2
            println!("  Row {}: age = {}", row_idx, age_val);
        }
    } else {
        println!("\n✗ FAILURE: WHERE clause NOT preserved! Have {} rows instead of 5", view.row_count());
        std::process::exit(1);
    }
}
EOF

# Compile and run the test
echo "Compiling test..."
rustc --edition 2021 -L target/release/deps test_where_sort_bug.rs -o test_where_sort_bug --extern sql_cli=target/release/libsql_cli.rlib

echo "Running test..."
./test_where_sort_bug

# Clean up
rm -f test_where_sort_bug.rs test_where_sort_bug

echo "Test complete!"