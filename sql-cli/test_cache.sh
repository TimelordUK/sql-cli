#!/bin/bash
# Test query cache functionality

echo "Testing query cache..."

# Create a test CSV file
cat > test_cache.csv << EOF
id,name,value
1,Item1,100
2,Item2,200
3,Item3,300
4,Item4,400
5,Item5,500
EOF

# Start the TUI with debug logging for query and results
RUST_LOG=sql_cli::enhanced_tui=debug,query=info,results=info timeout 10 ./target/release/sql-cli test_cache.csv 2>cache_test.log << EOF
SELECT * FROM test_cache
SELECT * FROM test_cache

q
EOF

echo "=== Cache Test Results ==="
echo "Looking for cache hits in the log..."
grep -E "(Executing query|Found cached|cached=)" cache_test.log | head -20

echo ""
echo "=== Cache-related logs ==="
grep -i "cache" cache_test.log | grep -v "is_cache_mode" | head -20

# Clean up
rm -f test_cache.csv cache_test.log