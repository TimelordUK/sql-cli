#!/bin/bash

# Test column search focus issue
# We'll check if search_columns() is being called and if it's setting the correct column

echo "Testing column search focus issue..."
echo

# Create test CSV with known columns
cat > test_search.csv << 'EOF'
id,name,description,orderid,amount
1,Widget A,First widget,ORD001,100.50
2,Widget B,Second widget,ORD002,200.75
3,Gadget X,Cool gadget,ORD003,300.25
4,Tool Y,Useful tool,ORD004,400.00
5,Device Z,Smart device,ORD005,500.99
EOF

echo "Test data created with columns: id, name, description, orderid, amount"
echo

# Run sql-cli with debug logging specifically for search
echo "Starting sql-cli with search debug logging..."
RUST_LOG=search=debug timeout 2 ./target/release/sql-cli test_search.csv 2>&1 | grep -E "(search_columns|Setting current column|Found.*columns matching|Column search)" | head -20

echo
echo "Testing if column focus is updating correctly..."

# Create a test that simulates the column search
cat > test_column_search.rs << 'EOF'
// This would test that when we search for "order" it should find "orderid" at index 3
// and set both state_container.current_column and buffer.current_column to 3
EOF

echo "To debug the issue:"
echo "1. When entering column search mode with '\' + pattern"
echo "2. The debounced search should call execute_search_action"
echo "3. execute_search_action calls search_columns()"
echo "4. search_columns() should find matches and set current_column"
echo "5. The cursor should visually move to the matched column"
echo
echo "Check the log file for search debug messages:"
echo "tail -f ~/.local/share/sql-cli/logs/sql-cli_*.log | grep search"