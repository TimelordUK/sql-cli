#!/bin/bash

# Test GROUP BY performance with different row counts
echo "Testing GROUP BY performance phases..."
echo "=================================="

for rows in 1000 5000 10000 20000 30000 50000; do
    echo ""
    echo "Testing with $rows rows:"
    echo "-------------------------"

    # Simple GROUP BY with one column
    echo "1. Simple GROUP BY (10 groups):"
    ./target/release/sql-cli -q "WITH nums AS (SELECT value as n, value % 10 as category FROM RANGE(1, $rows)) SELECT category, COUNT(*) FROM nums GROUP BY category" --execution-plan 2>&1 | grep -E "GROUP_BY.*rows:|Group time:"

    # GROUP BY with multiple aggregates
    echo ""
    echo "2. GROUP BY with multiple aggregates (50 groups):"
    ./target/release/sql-cli -q "WITH nums AS (SELECT value as n, value % 50 as category FROM RANGE(1, $rows)) SELECT category, COUNT(*), SUM(n), AVG(n), MIN(n), MAX(n) FROM nums GROUP BY category" --execution-plan 2>&1 | grep -E "GROUP_BY.*rows:|Group time:"

    # Multi-column GROUP BY
    echo ""
    echo "3. Multi-column GROUP BY (100 groups):"
    ./target/release/sql-cli -q "WITH nums AS (SELECT value as n, value % 10 as cat1, value % 10 as cat2 FROM RANGE(1, $rows)) SELECT cat1, cat2, COUNT(*) FROM nums GROUP BY cat1, cat2" --execution-plan 2>&1 | grep -E "GROUP_BY.*rows:|Group time:"
done

echo ""
echo "=================================="
echo "Analysis complete"