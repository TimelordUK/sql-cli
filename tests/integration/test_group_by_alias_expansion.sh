#!/bin/bash
# Integration tests for GROUP BY clause alias expansion feature

set -e  # Exit on first error

CLI="./target/release/sql-cli"
DATA_FILE="data/test_simple_math.csv"

echo "=== GROUP BY Alias Expansion Integration Tests ==="
echo

# Test 1: Basic alias expansion with modulo
echo "Test 1: Basic SELECT alias in GROUP BY"
result=$($CLI -q "SELECT value % 3 as grp, COUNT(*) FROM range(10) GROUP BY grp" -o csv 2>&1)
if echo "$result" | grep -q "grp,expr_2"; then
    echo "✓ Test 1 passed"
else
    echo "✗ Test 1 failed"
    echo "$result"
    exit 1
fi

# Test 2: Alias with expression in SELECT and GROUP BY
echo "Test 2: Complex expression with division"
result=$($CLI "$DATA_FILE" -q "SELECT b / 10 as tens, COUNT(*) FROM test_simple_math GROUP BY tens" -o csv 2>&1)
if echo "$result" | grep -q "tens,expr_2"; then
    echo "✓ Test 2 passed"
else
    echo "✗ Test 2 failed"
    echo "$result"
    exit 1
fi

# Test 3: GROUP BY alias with HAVING (combo of transformers)
echo "Test 3: GROUP BY alias with HAVING clause"
result=$($CLI "$DATA_FILE" -q "SELECT id % 3 as grp, COUNT(*), SUM(b) FROM test_simple_math GROUP BY grp HAVING COUNT(*) > 2 AND SUM(b) > 100" -o csv 2>&1)
if echo "$result" | grep -q "grp,expr_2,expr_3"; then
    echo "✓ Test 3 passed"
else
    echo "✗ Test 3 failed"
    echo "$result"
    exit 1
fi

# Test 4: Multiple aliases in GROUP BY
echo "Test 4: Multiple aliases in GROUP BY"
result=$($CLI -q "SELECT value % 2 as even_odd, value % 3 as mod3, COUNT(*) FROM range(12) GROUP BY even_odd, mod3" -o csv 2>&1)
if echo "$result" | grep -q "even_odd,mod3,expr_3"; then
    echo "✓ Test 4 passed"
else
    echo "✗ Test 4 failed"
    echo "$result"
    exit 1
fi

# Test 5: Alias with ORDER BY (all three transformers working together)
echo "Test 5: GROUP BY alias with WHERE and ORDER BY"
result=$($CLI -q "SELECT value % 3 as grp, COUNT(*) as cnt FROM range(20) WHERE value > 5 GROUP BY grp ORDER BY cnt DESC" -o csv 2>&1)
if echo "$result" | grep -q "grp,cnt"; then
    echo "✓ Test 5 passed"
else
    echo "✗ Test 5 failed"
    echo "$result"
    exit 1
fi

# Test 6: Verify preprocessing shows GroupByAliasExpander
echo "Test 6: Verify GroupByAliasExpander runs in pipeline"
result=$($CLI -q "SELECT value % 3 as grp, COUNT(*) FROM range(10) GROUP BY grp" -o csv --show-preprocessing 2>&1)
if echo "$result" | grep -q "GroupByAliasExpander"; then
    echo "✓ Test 6 passed"
else
    echo "✗ Test 6 failed"
    echo "$result"
    exit 1
fi

# Test 7: Function alias in GROUP BY
echo "Test 7: Function result aliased in GROUP BY"
result=$($CLI "data/test_simple_strings.csv" -q "SELECT UPPER(name) as upper_name, COUNT(*) FROM test_simple_strings GROUP BY upper_name" -o csv 2>&1)
if echo "$result" | grep -q "upper_name,expr_2"; then
    echo "✓ Test 7 passed"
else
    echo "✗ Test 7 failed"
    echo "$result"
    exit 1
fi

# Test 8: Arithmetic expression alias
echo "Test 8: Arithmetic expression alias in GROUP BY"
result=$($CLI "$DATA_FILE" -q "SELECT a * 2 as double_a, COUNT(*) FROM test_simple_math GROUP BY double_a LIMIT 5" -o csv 2>&1)
if echo "$result" | grep -q "double_a,expr_2"; then
    echo "✓ Test 8 passed"
else
    echo "✗ Test 8 failed"
    echo "$result"
    exit 1
fi

echo
echo "=== All GROUP BY alias expansion tests passed! ==="
