#!/bin/bash
# Verify cache is working

echo "Testing query cache functionality..."
echo "This will execute the same query twice to check if cache is used"
echo ""

# Start sql-cli and send commands
cat << 'EOF' | RUST_LOG=query=info,results=info,sql_cli::app_state_container::ResultsCache=info ./target/release/sql-cli test_cache_simple.csv 2>&1 | tee cache_output.log
SELECT * FROM test_cache_simple WHERE status = 'active'
SELECT * FROM test_cache_simple WHERE status = 'active'
SELECT * FROM test_cache_simple WHERE category = 'A'
SELECT * FROM test_cache_simple WHERE category = 'A'
q
EOF

echo ""
echo "=== Analyzing cache behavior ==="
echo ""

# Check for cache hits
echo "Cache HITS (should see some after first execution of each query):"
grep -E "Cache HIT|Found cached|Using cached" cache_output.log

echo ""
echo "Cache MISSES (should see these for first execution):"
grep "Cache MISS" cache_output.log

echo ""
echo "Query executions:"
grep "Executing query:" cache_output.log

echo ""
echo "Cache operations:"
grep "Caching results" cache_output.log

# Clean up
rm -f cache_output.log