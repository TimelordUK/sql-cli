#!/bin/bash

# Test script to check if numbers with commas and spaces are properly handled

# Create a test CSV with quoted numbers containing commas
cat > /tmp/test_proper_edge.csv << 'EOF'
name,plain_num,with_comma,with_spaces
Item1,1000,"1,000"," 100 "
Item2,5,"5,000","  5  "
Item3,200,"200,000"," 200"
Item4,10,"10","10 "
Item5,50000,"50,000"," 50000"
Item6,3,"3,333","3"
EOF

echo "Created test CSV with formatted numbers:"
cat /tmp/test_proper_edge.csv
echo ""

# Build
cargo build --release 2>/dev/null

# Create a test program
cat > /tmp/analyze_csv.rs << 'EOF'
use sql_cli::data::csv_datasource::CsvDataSource;
use sql_cli::data::data_view::DataView;
use sql_cli::data::datatable::DataValue;
use std::sync::Arc;

fn main() {
    // Load the CSV
    let csv_source = CsvDataSource::load_from_file("/tmp/test_proper_edge.csv", "test")
        .expect("Failed to load CSV");
    let datatable = csv_source.to_datatable();
    
    println!("=== Column Analysis ===");
    for (i, col) in datatable.columns.iter().enumerate() {
        println!("Column {}: '{}' - Type: {:?}", i, col.name, col.data_type);
    }
    
    println!("\n=== Value Analysis ===");
    println!("Checking how values are parsed:");
    for (row_idx, row) in datatable.rows.iter().enumerate().take(3) {
        println!("\nRow {}:", row_idx);
        for (col_idx, val) in row.values.iter().enumerate() {
            let col_name = &datatable.columns[col_idx].name;
            let type_str = match val {
                DataValue::String(s) => format!("String('{}')", s),
                DataValue::Integer(i) => format!("Integer({})", i),
                DataValue::Float(f) => format!("Float({})", f),
                _ => "Other".to_string(),
            };
            println!("  {}: {}", col_name, type_str);
        }
    }
    
    // Test sorting
    println!("\n=== Sorting Test ===");
    let mut view = DataView::new(Arc::new(datatable.clone()));
    
    // Sort by plain_num (should work correctly)
    println!("\nSorting by 'plain_num' column (index 1):");
    view.apply_sort(1, true).unwrap();
    print_sorted_values(&view, 1, "plain_num");
    
    // Sort by with_comma (will likely be sorted as strings)
    println!("\nSorting by 'with_comma' column (index 2):");
    view.apply_sort(2, true).unwrap();
    print_sorted_values(&view, 2, "with_comma");
    
    // Sort by with_spaces (might be trimmed and parsed)
    println!("\nSorting by 'with_spaces' column (index 3):");
    view.apply_sort(3, true).unwrap();  
    print_sorted_values(&view, 3, "with_spaces");
}

fn print_sorted_values(view: &DataView, col_idx: usize, col_name: &str) {
    println!("Sorted {} values:", col_name);
    for i in 0..view.row_count() {
        if let Some(row) = view.get_row(i) {
            if let Some(val) = row.values.get(col_idx) {
                println!("  {}: {:?}", i, val);
            }
        }
    }
}
EOF

# Compile the test
echo "Compiling test program..."
rustc --edition 2021 \
    -L target/release/deps \
    /tmp/analyze_csv.rs \
    -o /tmp/analyze_csv \
    --extern sql_cli=target/release/libsql_cli.rlib \
    $(find target/release/deps -name "*.rlib" | sed 's/^/--extern /' | tr '\n' ' ') 2>/dev/null

if [ -f /tmp/analyze_csv ]; then
    echo "Running analysis..."
    /tmp/analyze_csv
else
    echo "Could not compile test program, running TUI instead..."
    ./target/release/sql-cli /tmp/test_proper_edge.csv
fi