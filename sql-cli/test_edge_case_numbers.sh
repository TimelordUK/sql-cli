#!/bin/bash

# Test script to check edge case numeric formats that might break sorting

# Create a test CSV with various numeric formats
cat > /tmp/test_edge_numbers.csv << 'EOF'
name,formatted_num,scientific,with_spaces,negative,very_small
Item1,1000,1e3, 100 ,-50,0.000001
Item2,1,000,1e2,  200  ,-10,0.01
Item3,500,5e2,50,-100,0.0001
Item4,10000,1e4, 1000,-5,0.1
Item5,100,1e1,10 ,-200,0.00001
Item6,5000,5e3,500,-1,1.0
EOF

echo "Created test CSV with edge case numeric formats:"
cat /tmp/test_edge_numbers.csv
echo ""

# Build and run
cargo build --release 2>/dev/null
echo "Running the application with edge case numbers..."
echo ""

# Create an automated test
cat > /tmp/test_edge_sorting.rs << 'EOF'
use sql_cli::data::csv_datasource::CsvDataSource;
use std::sync::Arc;

fn main() {
    // Load the CSV
    let csv_source = CsvDataSource::load_from_file("/tmp/test_edge_numbers.csv", "test").unwrap();
    let datatable = csv_source.to_datatable();
    
    println!("Column types detected:");
    for (i, col) in datatable.columns.iter().enumerate() {
        println!("  Column {}: {} - Type: {:?}", i, col.name, col.data_type);
    }
    
    println!("\nFirst row values (to check parsing):");
    if let Some(row) = datatable.rows.get(0) {
        for (i, val) in row.values.iter().enumerate() {
            let type_name = match val {
                sql_cli::data::datatable::DataValue::String(_) => "String",
                sql_cli::data::datatable::DataValue::Integer(_) => "Integer", 
                sql_cli::data::datatable::DataValue::Float(_) => "Float",
                _ => "Other",
            };
            println!("  Value {}: {:?} ({})", i, val, type_name);
        }
    }
}
EOF

# Compile and run the test
rustc --edition 2021 -L target/release/deps /tmp/test_edge_sorting.rs -o /tmp/test_edge --extern sql_cli=target/release/libsql_cli.rlib 2>/dev/null || echo "Note: Direct test compilation failed, values with spaces/commas won't parse as numbers"

echo ""
echo "Testing with the actual application:"
./target/release/sql-cli /tmp/test_edge_numbers.csv