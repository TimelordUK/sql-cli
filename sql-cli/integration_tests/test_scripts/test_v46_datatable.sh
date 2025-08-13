#!/bin/bash

# Test script for V46: DataTable Introduction

echo "Testing V46: DataTable Introduction"
echo "===================================="

# Build the project
echo "Building project..."
cargo build --release 2>&1 | grep -E "error|warning|Finished"

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "Test: DataTable Conversion Demo"
echo "--------------------------------"
echo "1. Load a CSV file"
echo "2. Press F6 to convert current results to DataTable"
echo "3. Check status message for memory comparison"
echo "4. Check debug logs for column type information"
echo ""

# Create test data
cat > test_datatable.csv << EOF
id,name,age,salary,active,joined_date
1,Alice,30,75000.50,true,2020-01-15
2,Bob,25,60000.00,false,2021-03-20
3,Charlie,35,85000.75,true,2019-06-01
4,Diana,28,70000.25,true,2022-02-10
5,Eve,32,,false,2020-11-30
EOF

echo "Test data created: test_datatable.csv"
echo ""
echo "Running tests..."

# Test DataTable conversion
cargo test --lib data::datatable::tests::test_from_query_response --nocapture 2>&1 | grep -E "test result|V46"

echo ""
echo "Instructions for manual testing:"
echo "1. Run: RUST_LOG=debug ./target/release/sql-cli test_datatable.csv"
echo "2. After data loads, press F6"
echo "3. Look for 'V46: DataTable created!' in status bar"
echo "4. Check debug logs (F5) for detailed column information"
echo ""
echo "Expected behavior:"
echo "- Status shows memory comparison (JSON vs DataTable)"
echo "- Debug logs show column types (Integer, String, Float, Boolean, DateTime)"
echo "- Memory usage should be lower for DataTable"
echo ""
echo "===================================="
echo "V46 DataTable Introduction Test Complete!"