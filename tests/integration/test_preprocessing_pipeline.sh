#!/bin/bash

# Preprocessing Pipeline Test Suite
# Tests AST transformation pipeline with --show-preprocessing flag

echo "=== Preprocessing Pipeline Test Suite ==="
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

echo "=== Test 1: Basic query with preprocessing flag ==="
echo "Should show preprocessing statistics"
$CLI -q "SELECT 1+1" --check-in-lifting --show-preprocessing -o csv
echo

echo "=== Test 2: IN expression lifting (11 values - should trigger) ==="
echo "InOperatorLifter should report rewriting"
$CLI $TEST_FILE -q "SELECT * FROM test_simple_math WHERE a IN (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)" --check-in-lifting --show-preprocessing -o csv 2>&1 | grep -E "(Preprocessing|transformer|applied)"
echo

echo "=== Test 3: Query without --check-in-lifting flag ==="
echo "Should NOT show preprocessing (pipeline disabled)"
RESULT=$($CLI $TEST_FILE -q "SELECT * FROM test_simple_math WHERE id <= 3" -o csv 2>&1)
if echo "$RESULT" | grep -q "Preprocessing"; then
    echo "❌ FAILED: Preprocessing should not run without --check-in-lifting flag"
    exit 1
else
    echo "✅ PASSED: No preprocessing output when flag not set"
fi
echo

echo "=== Test 4: Query with --show-preprocessing but without --check-in-lifting ==="
echo "Should NOT show preprocessing (pipeline not enabled)"
RESULT=$($CLI $TEST_FILE -q "SELECT * FROM test_simple_math WHERE id <= 3" --show-preprocessing -o csv 2>&1)
if echo "$RESULT" | grep -q "Preprocessing"; then
    echo "❌ FAILED: Preprocessing should not run without --check-in-lifting flag"
    exit 1
else
    echo "✅ PASSED: --show-preprocessing has no effect without --check-in-lifting"
fi
echo

echo "=== Test 5: Verify pipeline statistics format ==="
echo "Should show transformer names and timing"
OUTPUT=$($CLI -q "SELECT 1+1" --check-in-lifting --show-preprocessing -o csv 2>&1)
if echo "$OUTPUT" | grep -q "ExpressionLifter" && \
   echo "$OUTPUT" | grep -q "CTEHoister" && \
   echo "$OUTPUT" | grep -q "InOperatorLifter" && \
   echo "$OUTPUT" | grep -q "transformer(s) applied"; then
    echo "✅ PASSED: All expected transformers shown in output"
else
    echo "❌ FAILED: Missing expected transformer names in output"
    echo "Output was:"
    echo "$OUTPUT"
    exit 1
fi
echo

echo "=== Test 6: Pipeline with CTE hoisting ==="
echo "Nested CTEs should be hoisted to top level"
$CLI -q "WITH outer_cte AS (SELECT * FROM (SELECT 1 as x)) SELECT * FROM outer_cte" --check-in-lifting --show-preprocessing -o csv 2>&1 | grep -E "(Preprocessing|transformer)"
echo

echo "=== All preprocessing pipeline tests completed ==="
