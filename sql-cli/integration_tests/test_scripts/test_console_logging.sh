#!/bin/bash

echo "Testing console logging for state changes..."
echo ""

# Create test file
cat > logging_test.csv << EOF
A,B,C
1,2,3
4,5,6
EOF

# Run with debug logging to console
echo "Starting sql-cli with debug logging to console..."
echo "Try these actions:"
echo "  - Arrow keys (watch for Navigation logs)"
echo "  - 'v' key (watch for Selection mode toggle)"
echo "  - Ctrl+C to exit"
echo ""
echo "Running..."
echo "=========="

RUST_LOG=sql_cli::app_state_container=info timeout 10 ./target/release/sql-cli logging_test.csv 2>&1 | grep -E "(Navigation|Selection|Table row|Column selected|Mode toggled)" || true

echo ""
echo "Note: In a real terminal, you'd see these logs in the debug log file."
echo "The logs show our new centralized state management in action!"