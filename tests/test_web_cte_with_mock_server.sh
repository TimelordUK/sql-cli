#!/bin/bash
# Test WEB CTE features with mock API server

set -e

echo "=================================="
echo "Testing WEB CTE Features"
echo "=================================="

# Start mock server in background
echo "Starting mock API server..."
uv run python tests/mock_api_server/server.py 5001 > /dev/null 2>&1 &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Function to run test
run_test() {
    local name=$1
    local query=$2
    echo -n "Testing $name... "
    if ./target/release/sql-cli -q "$query" -o csv > /dev/null 2>&1; then
        echo "✓"
    else
        echo "✗"
        ./target/release/sql-cli -q "$query" -o csv
    fi
}

# Test 1: Basic GET with JSON_PATH
run_test "GET with JSON_PATH" "
WITH WEB trades AS (
    URL 'http://localhost:5001/trades'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT COUNT(*) as total FROM trades"

# Test 2: POST with BODY
run_test "POST with BODY" "
WITH WEB filtered AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    BODY '{\"Where\": \"Symbol = \\\"AAPL\\\"\", \"Limit\": 10}'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT COUNT(*) as apple_trades FROM filtered"

# Test 3: Nested JSON paths
run_test "Nested JSON_PATH" "
WITH WEB nested AS (
    URL 'http://localhost:5001/nested/data'
    FORMAT JSON
    JSON_PATH 'data.trades.Result'
)
SELECT COUNT(*) as nested_count FROM nested"

# Test 4: Headers with environment variables
export TEST_API_TOKEN="test-token-123"
run_test "Headers with env vars" "
WITH WEB data AS (
    URL 'http://localhost:5001/trades'
    HEADERS (
        'Authorization': 'Bearer \${TEST_API_TOKEN}',
        'X-Custom': 'test'
    )
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT COUNT(*) as auth_count FROM data"

# Test 5: Complex POST query
run_test "Complex POST" "
WITH WEB market AS (
    URL 'http://localhost:5001/api/v2/market-data'
    METHOD POST
    BODY '{\"Limit\": 5}'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT
    COUNT(*) as total,
    AVG(volume) as avg_volume
FROM market"

# Test 6: CACHE with new features
run_test "CACHE clause" "
WITH WEB cached AS (
    URL 'http://localhost:5001/trades'
    METHOD GET
    FORMAT JSON
    JSON_PATH 'Result'
    CACHE 60
)
SELECT COUNT(*) FROM cached"

# Clean up
echo "Stopping mock server..."
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo "=================================="
echo "All tests completed!"
echo "=================================="