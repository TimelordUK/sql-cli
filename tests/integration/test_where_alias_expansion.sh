#!/bin/bash
# Integration tests for WHERE clause alias expansion feature

set -e  # Exit on first error

CLI="./target/release/sql-cli"
DATA_FILE="data/test_simple_math.csv"

echo "=== WHERE Alias Expansion Integration Tests ==="
echo

# Test 1: Basic alias expansion
echo "Test 1: Basic SELECT alias in WHERE"
result=$($CLI "$DATA_FILE" -q "SELECT a, b, a * 2 as double_a FROM test_simple_math WHERE double_a > 10" -o csv 2>&1)
if echo "$result" | grep -q "6,60,12"; then
    echo "✓ Test 1 passed"
else
    echo "✗ Test 1 failed"
    echo "$result"
    exit 1
fi

# Test 2: Multiple aliases
echo "Test 2: Multiple aliases in WHERE"
result=$($CLI "$DATA_FILE" -q "SELECT a, a * 2 as double_a, a * 3 as triple_a FROM test_simple_math WHERE double_a > 10 AND triple_a < 25" -o csv 2>&1)
if echo "$result" | grep -q "6,12,18"; then
    echo "✓ Test 2 passed"
else
    echo "✗ Test 2 failed"
    echo "$result"
    exit 1
fi

# Test 3: Complex expression in alias
echo "Test 3: Complex expression with division"
result=$($CLI "$DATA_FILE" -q "SELECT a, b, b / 10 as tens FROM test_simple_math WHERE tens > 5" -o csv 2>&1)
if echo "$result" | grep -q "6,60,6"; then
    echo "✓ Test 3 passed"
else
    echo "✗ Test 3 failed"
    echo "$result"
    exit 1
fi

# Test 4: Alias in comparison with another column
echo "Test 4: Alias compared with column"
result=$($CLI "$DATA_FILE" -q "SELECT a, b, a * 10 as ten_a FROM test_simple_math WHERE ten_a = b LIMIT 5" -o csv 2>&1)
if echo "$result" | grep -q "1,10,10"; then
    echo "✓ Test 4 passed"
else
    echo "✗ Test 4 failed"
    echo "$result"
    exit 1
fi

# Test 5: Alias in OR condition
echo "Test 5: Alias in OR condition"
result=$($CLI "$DATA_FILE" -q "SELECT a, a * 2 as double_a FROM test_simple_math WHERE double_a = 4 OR double_a = 6" -o csv 2>&1)
if echo "$result" | grep -q "2,4"; then
    echo "✓ Test 5 passed"
else
    echo "✗ Test 5 failed"
    echo "$result"
    exit 1
fi

# Test 6: Verify preprocessing shows WhereAliasExpander
echo "Test 6: Verify WhereAliasExpander runs in pipeline"
result=$($CLI "$DATA_FILE" -q "SELECT a, a * 2 as double_a FROM test_simple_math WHERE double_a > 10" -o csv --show-preprocessing 2>&1)
if echo "$result" | grep -q "WhereAliasExpander"; then
    echo "✓ Test 6 passed"
else
    echo "✗ Test 6 failed"
    echo "$result"
    exit 1
fi

# Test 7: Alias with function
echo "Test 7: Alias with UPPER function"
result=$($CLI "data/test_simple_strings.csv" -q "SELECT name, UPPER(name) as upper_name FROM test_simple_strings WHERE upper_name = 'ALICE'" -o csv 2>&1)
if echo "$result" | grep -q "Alice,ALICE"; then
    echo "✓ Test 7 passed"
else
    echo "✗ Test 7 failed"
    echo "$result"
    exit 1
fi

# Test 8: Multiple uses of same alias
echo "Test 8: Same alias used multiple times in WHERE"
result=$($CLI "$DATA_FILE" -q "SELECT a, a * 2 as double_a FROM test_simple_math WHERE double_a > 10 AND double_a < 20" -o csv 2>&1)
if echo "$result" | grep -q "6,12"; then
    echo "✓ Test 8 passed"
else
    echo "✗ Test 8 failed"
    echo "$result"
    exit 1
fi

echo
echo "=== All WHERE alias expansion tests passed! ==="
