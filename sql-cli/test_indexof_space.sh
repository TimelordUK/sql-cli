#!/bin/bash

# Create test CSV with books that have and don't have spaces
cat > test_books.csv << EOF
id,book,description
1,derivatives,No spaces in this book name
2,equity trading,Has a space in the middle
3,FX,Short name no space
4, leading,Space at the beginning
5,trailing ,Space at the end
6,multi word book,Multiple spaces in name
EOF

echo "Testing IndexOf with space character..."
echo ""
echo "Test data:"
cat test_books.csv
echo ""
echo "Running query: SELECT * FROM test_books WHERE book.IndexOf(' ') = 0"
echo "Expected: Only row 4 (' leading') should match as it has space at position 0"
echo ""

# Run the query
./target/release/sql-cli test_books.csv << 'SQL'
SELECT * FROM test_books WHERE book.IndexOf(' ') = 0
SQL

echo ""
echo "Running query: SELECT * FROM test_books WHERE book.IndexOf(' ') = -1"
echo "Expected: Rows 1 (derivatives) and 3 (FX) should match as they have no spaces"
echo ""

./target/release/sql-cli test_books.csv << 'SQL'
SELECT * FROM test_books WHERE book.IndexOf(' ') = -1
SQL

echo ""
echo "Running query: SELECT * FROM test_books WHERE book.IndexOf(' ') > 0"
echo "Expected: Rows 2, 5, 6 should match (space not at beginning but present)"
echo ""

./target/release/sql-cli test_books.csv << 'SQL'
SELECT * FROM test_books WHERE book.IndexOf(' ') > 0
SQL

# Cleanup
rm -f test_books.csv