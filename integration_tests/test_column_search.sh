#!/bin/bash

# Test column search functionality with hidden columns

echo "Testing column search with hidden columns..."

# Create test input: load CSV, hide some columns, then search for "order"
cat > test_input.txt << 'EOF'
-
-
-
\
order

q
EOF

echo "Running test..."
timeout 2s ./target/release/sql-cli test_orders.csv < test_input.txt 2>&1 | tail -50

echo "Test complete"