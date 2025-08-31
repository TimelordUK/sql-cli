#!/bin/bash

# Test string functions: MID, UPPER, LOWER, TRIM

# Create test data
cat > /tmp/test_strings.csv << EOF
id,name,description
1,Alice,"  Hello World  "
2,Bob,"Testing 123"
3,Charlie,"SQL Functions"
4,David,"   Spaces   "
5,Eve,"lowercase text"
6,Frank,"UPPERCASE TEXT"
EOF

echo "Testing string functions with SQL CLI"
echo "====================================="
echo

echo "1. MID function - Extract substring (Excel-compatible, 1-based indexing):"
echo "   Query: SELECT name, MID(name, 1, 3) as first_three FROM test_strings WHERE id <= 3"
./target/release/sql-cli /tmp/test_strings.csv -q "SELECT name, MID(name, 1, 3) as first_three FROM test_strings WHERE id <= 3" -o table
echo

echo "2. UPPER function - Convert to uppercase:"
echo "   Query: SELECT name, UPPER(name) as upper_name FROM test_strings WHERE id IN (1, 5)"
./target/release/sql-cli /tmp/test_strings.csv -q "SELECT name, UPPER(name) as upper_name FROM test_strings WHERE id IN (1, 5)" -o table
echo

echo "3. LOWER function - Convert to lowercase:"
echo "   Query: SELECT description, LOWER(description) as lower_desc FROM test_strings WHERE id IN (3, 6)"
./target/release/sql-cli /tmp/test_strings.csv -q "SELECT description, LOWER(description) as lower_desc FROM test_strings WHERE id IN (3, 6)" -o table
echo

echo "4. TRIM function - Remove leading/trailing spaces:"
echo "   Query: SELECT description, TRIM(description) as trimmed FROM test_strings WHERE id IN (1, 4)"
./target/release/sql-cli /tmp/test_strings.csv -q "SELECT description, TRIM(description) as trimmed FROM test_strings WHERE id IN (1, 4)" -o table
echo

echo "5. Combined functions - UPPER(TRIM()):"
echo "   Query: SELECT description, UPPER(TRIM(description)) as processed FROM test_strings WHERE id IN (1, 4)"
./target/release/sql-cli /tmp/test_strings.csv -q "SELECT description, UPPER(TRIM(description)) as processed FROM test_strings WHERE id IN (1, 4)" -o table
echo

echo "6. Complex MID with expressions:"
echo "   Query: SELECT name, MID(UPPER(name), 2, 3) as mid_upper FROM test_strings WHERE id <= 3"
./target/release/sql-cli /tmp/test_strings.csv -q "SELECT name, MID(UPPER(name), 2, 3) as mid_upper FROM test_strings WHERE id <= 3" -o table