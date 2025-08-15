#!/bin/bash

# Create simple test CSV
cat > test_action_demo.csv << EOF
id,name
1,Alice
2,Bob
3,Carol
EOF

echo "Testing action system navigation..."
echo "The following keys should work through the new action system:"
echo "  j/k - down/up"
echo "  h/l - left/right"
echo "  5j - move down 5 (vim count)"
echo "  v - toggle selection mode"
echo "  p - pin column"
echo "  s - sort"
echo ""
echo "Starting SQL CLI with debug logging..."
echo "Watch for '✓ Action system:' messages in the log"
echo ""

# Run with info-level logging to see our action system messages
RUST_LOG=sql_cli::ui::enhanced_tui=info ./target/debug/sql-cli test_action_demo.csv -e "select * from data" 2>&1 | tee action_test.log &
LOG_PID=$!

# Give it a moment to start
sleep 1

# Wait for user to test
echo "Test the keys above, then press 'q' to quit."
wait $LOG_PID

echo ""
echo "=== Action System Activity Summary ==="
if grep -q "✓ Action system:" action_test.log; then
    echo "SUCCESS: Action system is working!"
    echo ""
    echo "Actions handled:"
    grep "✓ Action system:" action_test.log | head -10
else
    echo "WARNING: No action system activity detected. Check if keys are being handled by legacy code."
fi

# Cleanup
rm -f test_action_demo.csv action_test.log