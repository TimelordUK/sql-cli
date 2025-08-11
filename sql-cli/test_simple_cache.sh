#!/bin/bash
# Simple test to verify cache functionality

echo "Creating test CSV file..."
cat > test_cache_data.csv << EOF
id,name,amount
1,Alice,100
2,Bob,200
3,Charlie,300
EOF

echo "Starting sql-cli with test data..."
# Run with query/results logging enabled
RUST_LOG=query=info,results=info ./target/release/sql-cli test_cache_data.csv 2>cache_test.log &
PID=$!

# Give it time to start
sleep 2

# Send commands using xdotool or similar if available
# For now, let's just kill and check what we got
sleep 3
kill $PID 2>/dev/null

echo "=== Checking logs for cache behavior ==="
if [ -f cache_test.log ]; then
    echo "Query execution logs:"
    grep -E "(Executing query|Found cached|ResultsCache)" cache_test.log
    echo ""
    echo "All query-related logs:"
    grep "query" cache_test.log | head -10
else
    echo "No log file generated"
fi

# Clean up
rm -f test_cache_data.csv cache_test.log