#!/bin/bash

echo "Testing new action system integration..."

# Build in debug mode for logging
cargo build

# Create a test CSV if it doesn't exist
cat > test_action.csv << EOF
name,value
row1,10
row2,20
row3,30
EOF

# Test with debug logging to see if action system is triggered
echo -e "\n=== Testing navigation keys through action system ==="
echo "Press j (down), k (up), h (left), l (right), then q to quit"
echo "Watch for 'Action system mapped key to action' messages"

RUST_LOG=sql_cli::ui::enhanced_tui=debug timeout 10 ./target/debug/sql-cli test_action.csv -e "select * from data" 2>&1 | grep -E "Action system|Action handled|mapped key to action|new system" &

# Let user interact
./target/debug/sql-cli test_action.csv -e "select * from data"

echo -e "\n=== Test complete ==="
echo "If you saw 'Action system mapped key to action' messages, the new system is working!"