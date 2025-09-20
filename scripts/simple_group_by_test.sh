#!/bin/bash

echo "Simple GROUP BY Performance Test"
echo "================================"

# Generate test data
echo "id,category,value" > /tmp/group_test.csv
for i in {1..50000}; do
    cat=$((i % 100))
    val=$((RANDOM % 1000))
    echo "$i,cat_$cat,$val" >> /tmp/group_test.csv
done

echo "Test data: 50,000 rows, 100 categories"
echo ""

# Run query
time ./target/release/sql-cli /tmp/group_test.csv -q "SELECT category, COUNT(*), SUM(value) FROM group_test GROUP BY category" -o csv > /dev/null

rm /tmp/group_test.csv