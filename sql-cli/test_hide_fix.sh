#!/bin/bash

echo "Testing column hiding with QueryEngine..."

# Create test data with more columns
cat > test_hide.csv << EOF
id,name,amount,status,date
1,Alice,100.50,Active,2025-01-15
2,Bob,200.25,Pending,2025-02-20
3,Charlie,150.00,Active,2025-03-10
EOF

# Test with debug logging to see column operations
echo -e "\n=== Testing column hiding with WHERE clause ==="
RUST_LOG=debug timeout 2 ./target/release/sql-cli test_hide.csv -e "SELECT * FROM data WHERE status = 'Active'" 2>&1 | grep -E "(Buffer has|hidden|visible columns:|column_count|Stored QueryEngine)" | head -20

echo -e "\n=== Test complete ==="
rm test_hide.csv