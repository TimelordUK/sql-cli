#!/bin/bash
# Test script for large table navigation in Neovim

echo "Creating test data with 500 rows..."

# Create a CSV file with 500 rows
cat > /tmp/test_large_table.csv << 'EOF'
id,name,value,status,timestamp
EOF

for i in {1..500}; do
    echo "$i,Item_$i,$((i * 100)),$([ $((i % 2)) -eq 0 ] && echo "active" || echo "inactive"),2025-01-$(printf "%02d" $((i % 30 + 1)))" >> /tmp/test_large_table.csv
done

echo "Test data created at /tmp/test_large_table.csv"
echo "Running query to display all 500 rows..."

# Run the query
./target/release/sql-cli /tmp/test_large_table.csv -q "SELECT * FROM test_large_table ORDER BY id" -o table

echo ""
echo "Test completed. The table above has 500 rows."
echo "To test in Neovim:"
echo "1. Copy the output to a buffer"
echo "2. Press \\sn to enter table navigation mode"
echo "3. Navigate down with 'j' - the header should remain visible or be scrollable back to"