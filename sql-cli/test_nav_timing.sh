#!/bin/bash

# Test navigation timing with different row counts

echo "Testing navigation timing with DataTable..."

# Test with 10k rows
echo "=== 10k rows ==="
head -10001 test_100k.csv > test_10k.csv
echo -e "SELECT * FROM test_10k\njjjjjq" | timeout 3 ./target/release/sql-cli test_10k.csv 2>&1 | grep -E "next_row timing|DataTable.*10000 rows" | head -10

# Test with 20k rows  
echo "=== 20k rows ==="
head -20001 test_100k.csv > test_20k.csv
echo -e "SELECT * FROM test_20k\njjjjjq" | timeout 3 ./target/release/sql-cli test_20k.csv 2>&1 | grep -E "next_row timing|DataTable.*20000 rows" | head -10

# Test with 100k rows
echo "=== 100k rows ==="
echo -e "SELECT * FROM test_100k\njjjjjq" | timeout 5 ./target/release/sql-cli test_100k.csv 2>&1 | grep -E "next_row timing|DataTable.*100000 rows" | head -10

echo "Done"