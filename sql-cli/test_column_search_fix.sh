#!/bin/bash

# Test column search with hidden columns
echo "Testing column search with hidden columns..."

# Create test data
cat > test_column_search.csv << 'EOF'
id,name,age,comments,orderNumber,externalOrderId,status,country,notes
1,Alice,30,Some comment,1001,EXT-001,Active,USA,Note 1
2,Bob,25,Another comment,1002,EXT-002,Pending,Canada,Note 2
3,Charlie,35,Third comment,1003,EXT-003,Active,UK,Note 3
EOF

echo "Created test data with columns: id, name, age, comments, orderNumber, externalOrderId, status, country, notes"
echo ""
echo "Test: Hide 'comments' column (index 3) and search for 'order'"
echo "Expected: Should highlight 'orderNumber' at visual position 3 (was 4 before hiding)"
echo ""
echo "Run with: ./target/release/sql-cli test_column_search.csv"
echo "Commands to test:"
echo "  1. Press 'H' to enter hide mode"
echo "  2. Navigate to 'comments' column and press Enter to hide it"
echo "  3. Press '/' to search columns"
echo "  4. Type 'order' and press Enter"
echo "  5. Verify that 'orderNumber' is highlighted (not the column to its left)"
echo ""
echo "You can also test in enhanced mode for visual debugging:"
echo "./target/release/sql-cli test_column_search.csv --enhanced"