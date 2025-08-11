#\!/bin/bash
# Test column search functionality

echo "Testing column search..."

# Start the TUI in background
RUST_LOG=sql_cli::app_state_container=debug ./target/release/sql-cli test_columns.csv 2>column_search.log &
PID=$\!

# Wait for it to start
sleep 2

# Send column search commands using expect or similar
# For now, just kill it after a bit
sleep 3
kill $PID 2>/dev/null

# Check the logs
echo "=== Column Search Logs ==="
grep -i "column" column_search.log | grep -v "Column names" | head -20

echo "=== Test complete ==="
