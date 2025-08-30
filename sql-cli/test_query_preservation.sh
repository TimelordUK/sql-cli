#!/bin/bash

# Test script to verify original data is preserved after computed queries

echo "Creating test CSV file..."
cat > /tmp/test_query_preserve.csv << 'EOF'
a,b,c,d,e
1,2,3,4,5
6,7,8,9,10
11,12,13,14,15
EOF

echo "Testing query preservation..."
echo ""
echo "1. Load CSV file"
echo "2. Execute: SELECT a, b * 2 as double_b FROM test"  
echo "3. Execute: SELECT * FROM test (should show all 5 columns)"
echo ""

# Run the SQL CLI with commands
(
    echo "SELECT a, b * 2 as double_b FROM test_query_preserve"
    sleep 2
    echo "SELECT * FROM test_query_preserve"
    sleep 2
    echo ":quit"
) | RUST_LOG=info ./target/release/sql-cli /tmp/test_query_preserve.csv 2>&1 | grep -E "QueryExecutionService:|columns:|DataTable"

echo ""
echo "Check if original 5 columns are preserved after computed query"