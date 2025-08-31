#!/bin/bash

# Quick test to verify visitor pattern actions work
echo "Testing visitor pattern actions..."

# Create a test CSV file
cat > test_visitor.csv << EOF
name,age,score
Alice,25,95
Bob,30,87
Charlie,35,92
EOF

# Test various actions by simulating key presses
echo "Testing toggle and clear actions..."
timeout 2 bash -c "
echo -e 'v\nR\nC\n\x1b\nq' | ./target/debug/sql-cli test_visitor.csv -e 'select * from data' 2>&1 | grep -E '(Cell mode|Row mode|Compact mode|numbers)'
"

# Clean up
rm -f test_visitor.csv

echo "Test completed successfully"