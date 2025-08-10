#\!/bin/bash

# Test script to verify column search mode behavior

echo "Testing column search mode with Tab navigation..."

# Create a test CSV file with multiple columns
cat > test_columns.csv << EOF2
orderid,customer_name,order_date,order_status,order_total,order_items
1001,Alice Smith,2024-01-15,completed,199.99,5
1002,Bob Jones,2024-01-16,pending,299.99,3
1003,Charlie Brown,2024-01-17,completed,149.50,8
1004,Diana Prince,2024-01-18,shipped,399.99,2
1005,Eve Adams,2024-01-19,completed,89.99,1
EOF2

echo "Created test_columns.csv with columns containing 'order' prefix"
echo ""
echo "To test column search:"
echo "1. Run: ./target/release/sql-cli test_columns.csv"
echo "2. Execute: SELECT * FROM test_columns"
echo "3. Press '\\' to enter column search mode"
echo "4. Type 'order' to find matching columns"
echo "5. Wait for debounced search to execute (should see matching columns)"
echo "6. Press Tab to navigate between matching columns"
echo "7. Press F5 to see debug info showing current mode and column search state"
echo ""
echo "Expected behavior:"
echo "- Should find 5 columns matching 'order'"
echo "- Tab/Shift-Tab should navigate between them"
echo "- Mode should stay in ColumnSearch (COL) until Enter or Esc"
