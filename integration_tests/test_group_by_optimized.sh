#!/bin/bash

echo "Testing GROUP BY performance after optimization..."
echo "=================================================="
echo ""

for rows in 1000 5000 10000 30000 50000; do
    echo "Testing with $rows rows (10 groups):"

    time_output=$(./target/release/sql-cli -q "WITH nums AS (SELECT value as n, value % 10 as category FROM RANGE(1, $rows)) SELECT category, COUNT(*), SUM(n), AVG(n) FROM nums GROUP BY category" --execution-plan 2>&1 | grep -E "Phase 1 - Group Building:|Phase 2 - Aggregation:|Total GROUP BY time:")

    echo "$time_output"
    echo ""
done

echo "=================================================="
echo "Compare with previous results:"
echo "30k rows BEFORE: Group Building: 2191ms (90.5%)"
echo "50k rows BEFORE: Group Building: 3596ms (94.1%)"