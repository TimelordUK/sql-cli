#!/bin/bash

# HAVING Auto-Aliasing Test Suite
# Tests automatic aliasing of aggregate functions in HAVING clauses

echo "=== HAVING Auto-Aliasing Test Suite ==="
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

echo "=== Test 1: Basic COUNT(*) in HAVING (should auto-alias) ==="
echo "Query: SELECT a, COUNT(*) FROM test_simple_math GROUP BY a HAVING COUNT(*) >= 1"
$CLI $TEST_FILE -q "SELECT a, COUNT(*) FROM test_simple_math GROUP BY a HAVING COUNT(*) >= 1" --check-in-lifting --show-preprocessing -o csv 2>&1 | head -20
echo

echo "=== Test 2: Multiple aggregates in HAVING ==="
echo "Query: SELECT id % 3 as grp, COUNT(*), SUM(b) FROM test_simple_math GROUP BY grp HAVING COUNT(*) > 2 AND SUM(b) > 100"
$CLI $TEST_FILE -q "SELECT id % 3 as grp, COUNT(*), SUM(b) FROM test_simple_math GROUP BY grp HAVING COUNT(*) > 2 AND SUM(b) > 100" --check-in-lifting --show-preprocessing -o csv 2>&1 | head -25
echo

echo "=== Test 3: HAVING with existing alias (should not add new alias) ==="
echo "Query: SELECT a, COUNT(*) as cnt FROM test_simple_math GROUP BY a HAVING cnt >= 1"
$CLI $TEST_FILE -q "SELECT a, COUNT(*) as cnt FROM test_simple_math GROUP BY a HAVING cnt >= 1" --check-in-lifting -o csv 2>&1 | head -15
echo

echo "=== Test 4: Complex HAVING expression with SUM ==="
echo "Query: SELECT a % 2 as parity, SUM(b) FROM test_simple_math GROUP BY parity HAVING SUM(b) > 500"
$CLI $TEST_FILE -q "SELECT a % 2 as parity, SUM(b) FROM test_simple_math GROUP BY parity HAVING SUM(b) > 500" --check-in-lifting --show-preprocessing -o csv 2>&1 | head -20
echo

echo "=== Test 5: AVG in HAVING ==="
echo "Query: SELECT a % 3 as grp, AVG(b) FROM test_simple_math GROUP BY grp HAVING AVG(b) > 50"
$CLI $TEST_FILE -q "SELECT a % 3 as grp, AVG(b) FROM test_simple_math GROUP BY grp HAVING AVG(b) > 50" --check-in-lifting --show-preprocessing -o csv 2>&1 | head -20
echo

echo "=== Test 6: Verify preprocessing shows HavingAliasTransformer ==="
OUTPUT=$($CLI $TEST_FILE -q "SELECT a, COUNT(*) FROM test_simple_math GROUP BY a HAVING COUNT(*) > 0" --check-in-lifting --show-preprocessing -o csv 2>&1)
if echo "$OUTPUT" | grep -q "HavingAliasTransformer"; then
    echo "✅ PASSED: HavingAliasTransformer shown in preprocessing output"
else
    echo "❌ FAILED: HavingAliasTransformer not shown in output"
    echo "Output was:"
    echo "$OUTPUT"
    exit 1
fi
echo

echo "=== All HAVING auto-aliasing tests completed ==="
