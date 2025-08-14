#!/bin/bash
# Test script for hide column functionality

# Create test data with multiple columns
cat > test_hide.csv << 'EOF'
id,name,age,city,country
1,Alice,30,NYC,USA
2,Bob,25,London,UK
3,Charlie,35,Paris,France
4,Diana,28,Tokyo,Japan
5,Eve,32,Berlin,Germany
EOF

echo "Testing hide column functionality (Ctrl+H)"
echo "==========================================="
echo "Test data created: test_hide.csv"
echo ""
echo "Instructions:"
echo "1. Run a query: SELECT * FROM data"
echo "2. Press Ctrl+H to hide the current column"
echo "3. Navigate with arrow keys and hide more columns"
echo "4. Press Ctrl+Shift+H to unhide all columns"
echo ""
echo "Starting SQL-CLI..."

./target/release/sql-cli test_hide.csv