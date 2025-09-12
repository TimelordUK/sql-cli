#!/bin/bash

# CASE WHEN Evaluation Test Suite
# Tests comprehensive CASE expression functionality

echo "=== CASE WHEN Evaluation Test Suite ==="
echo

# Build first
echo "Building..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo "✅ Build successful"
echo

CLI="./target/release/sql-cli"
TEST_FILE="data/test_simple_math.csv"

echo "=== Test 1: Basic CASE WHEN with numbers ==="
$CLI $TEST_FILE -q "SELECT id, a, CASE WHEN a > 5 THEN 'High' WHEN a > 2 THEN 'Medium' ELSE 'Low' END as level FROM test_simple_math WHERE id <= 8" -o csv
echo

echo "=== Test 2: CASE WHEN with arithmetic expressions ==="
$CLI $TEST_FILE -q "SELECT id, a, b, CASE WHEN a * 2 > b THEN 'A Doubled Wins' WHEN a + 5 > b THEN 'A Plus 5 Wins' ELSE 'B Wins' END as comparison FROM test_simple_math WHERE id <= 5" -o csv
echo

echo "=== Test 3: Nested CASE expressions (using MOD function) ==="
$CLI $TEST_FILE -q "SELECT id, a, CASE WHEN a > 10 THEN 'Big' WHEN a > 5 THEN CASE WHEN MOD(a, 2) = 0 THEN 'Medium Even' ELSE 'Medium Odd' END ELSE 'Small' END as category FROM test_simple_math WHERE id <= 12" -o csv
echo

echo "=== Test 4: Multiple CASE expressions in same query ==="
$CLI $TEST_FILE -q "SELECT id, a, b, CASE WHEN a > 5 THEN 'High A' ELSE 'Low A' END as a_level, CASE WHEN b > 50 THEN 'High B' ELSE 'Low B' END as b_level FROM test_simple_math WHERE id <= 6" -o csv
echo

echo "=== Test 5: CASE with different comparison operators ==="
$CLI $TEST_FILE -q "SELECT id, a, CASE WHEN a >= 10 THEN 'GTE10' WHEN a <= 3 THEN 'LTE3' WHEN a = 5 THEN 'EQ5' WHEN a != 7 THEN 'NE7' ELSE 'Other' END as operator_test FROM test_simple_math WHERE id <= 15" -o csv
echo

echo "=== Test 6: CASE in WHERE clause (not supported - using equivalent WHERE) ==="
echo "# Note: CASE expressions in WHERE clauses are not yet implemented"
echo "# Using equivalent: WHERE a > 8 AND id <= 15"
$CLI $TEST_FILE -q "SELECT id, a, b FROM test_simple_math WHERE a > 8 AND id <= 15" -o csv
echo

echo "=== Test 7: CASE with mathematical functions ==="
$CLI $TEST_FILE -q "SELECT id, c, CASE WHEN ROUND(c, 0) > 5 THEN 'Rounded High' WHEN FLOOR(c) < 2 THEN 'Floor Low' ELSE 'Middle' END as math_case FROM test_simple_math WHERE id <= 8" -o csv
echo

echo "=== Test 8: CASE without ELSE clause ==="
$CLI $TEST_FILE -q "SELECT id, a, CASE WHEN a = 5 THEN 'Five' WHEN a = 10 THEN 'Ten' END as special_numbers FROM test_simple_math WHERE id <= 15" -o csv
echo

echo "=== Test 9: Complex nested CASE with arithmetic ==="
$CLI $TEST_FILE -q "SELECT id, a, b, CASE WHEN a > 10 THEN CASE WHEN b > 100 THEN 'Big Both' ELSE 'Big A Only' END WHEN a < 5 THEN CASE WHEN b < 30 THEN 'Small Both' ELSE 'Small A Big B' END ELSE 'Medium A' END as complex_case FROM test_simple_math WHERE id <= 12" -o csv
echo

echo "=== Test 10: CASE with string comparisons (CAST not supported - using string literals) ==="
echo "# Note: CAST function is not yet implemented"
echo "# Testing with string literal comparisons instead"
$CLI $TEST_FILE -q "SELECT id, a, CASE WHEN a = 5 THEN 'Is Five' WHEN a = 10 THEN 'Is Ten' ELSE 'Other Number' END as number_test FROM test_simple_math WHERE id <= 12" -o csv
echo

echo "=== All CASE evaluation tests completed ==="