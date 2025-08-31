#!/bin/bash

# Test script for V44: Sort operations infrastructure

echo "Testing V44: Sort operations infrastructure"
echo "==========================================="

# Build the project first
echo "Building project..."
cargo build --release 2>&1 | grep -E "error|Finished"

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "Test 1: Basic Sort (Press 's' on a column)"
echo "-------------------------------------------"
echo "Currently still uses AppStateContainer JSON sorting"
echo "But column names come from DataProvider::get_column_names()"
timeout 2 ./target/release/sql-cli test_columns.csv -e "select * from data" <<EOF
s
q
EOF

echo ""
echo "Test 2: Sort Cycling (Ascending -> Descending -> None)"
echo "-------------------------------------------------------"
echo "Press 's' three times to cycle through sort states"
timeout 3 ./target/release/sql-cli test_columns.csv -e "select * from data" <<EOF
s
s
s
q
EOF

echo ""
echo "Test 3: Numeric Sort (Press '1' to sort by first column)"
echo "---------------------------------------------------------"
echo "Should detect numeric values and sort numerically"
timeout 2 ./target/release/sql-cli test_columns.csv -e "select * from data" <<EOF
1
q
EOF

echo ""
echo "Test 4: Multi-column Sort (Press different number keys)"
echo "--------------------------------------------------------"
echo "Press '2' for second column, '3' for third"
timeout 3 ./target/release/sql-cli test_columns.csv -e "select * from data" <<EOF
2
3
q
EOF

echo ""
echo "==========================================="
echo "V44 Infrastructure Test Complete!"
echo ""
echo "Current state:"
echo "✓ DataViewProvider has sort methods (get_sorted_indices, is_sorted, get_sort_state)"
echo "✓ sort_via_provider() helper created (uses DataProvider::get_row)"
echo "✓ Column names come from DataProvider"
echo "✗ Actual sorting still uses JSON in AppStateContainer"
echo ""
echo "Next steps for full migration:"
echo "1. Implement get_sorted_indices in BufferAdapter"
echo "2. Update rendering to use sorted indices"
echo "3. Remove JSON sorting from AppStateContainer"