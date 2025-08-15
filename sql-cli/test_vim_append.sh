#!/bin/bash

# Test vim-style append modes

echo "Testing vim-style append modes..."
echo ""

# Create a test CSV file
cat > test_vim_append.csv << 'EOF'
id,name,age,country
1,Alice,30,USA
2,Bob,25,Canada
3,Charlie,35,UK
4,David,28,Australia
5,Eve,32,Germany
EOF

echo "Test data created in test_vim_append.csv"
echo ""
echo "Test the following vim-style commands in Results mode:"
echo "  i - Insert at current cursor position (existing)"
echo "  a - Append at end of query"
echo "  wa - Append after WHERE clause"
echo "  oa - Append after ORDER BY clause" 
echo "  sa - Append after SELECT clause"
echo "  ga - Append after GROUP BY clause"
echo ""
echo "Starting SQL CLI..."

./target/debug/sql-cli test_vim_append.csv

# Clean up
rm -f test_vim_append.csv