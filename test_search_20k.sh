#!/bin/bash

# Test search on the large dataset
echo "Testing search navigation on 20k rows..."

# Run with timeout and grep for relevant logs
RUST_LOG=sql_cli::ui::table_widget_manager=info,search=info timeout 5 ./sql-cli/target/release/sql-cli ../trades_20k.csv -e "select * from trades_20k" 2>&1 &
PID=$!

# Give it time to load
sleep 2

# Send search commands via echo (simulating input)
echo "/" | nc -q 0 localhost 0 2>/dev/null || true
sleep 0.5
echo "emerging" | nc -q 0 localhost 0 2>/dev/null || true

# Wait a bit for processing
sleep 1

# Kill the process
kill $PID 2>/dev/null || true

wait $PID 2>/dev/null

echo "Test terminated"
