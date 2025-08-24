#!/bin/bash

# Test that empty fuzzy filter clears the filter and shows all rows

# Create test data with 10 rows
cat > test_fuzzy_clear.csv << EOF
id,status,name
1,pending,Alice
2,approved,Bob
3,rejected,Charlie
4,pending,David
5,approved,Eve
6,rejected,Frank
7,pending,Grace
8,approved,Henry
9,rejected,Ivan
10,pending,Jane
EOF

echo "Testing fuzzy filter clear behavior..."
echo "Created test file with 10 rows"
echo ""
echo "Instructions:"
echo "1. Press Shift+F to enter fuzzy filter mode"
echo "2. Type 'rejected' to filter to 3 rows"
echo "3. Press Enter to apply"
echo "4. Press Shift+F again"
echo "5. Press Enter with empty input"
echo "6. Check that all 10 rows are shown again"
echo ""
echo "Starting application..."

RUST_LOG=search=debug timeout 30 ./target/release/sql-cli test_fuzzy_clear.csv -e "select * from data"