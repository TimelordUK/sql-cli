#!/bin/bash

echo "Testing V27 State Management Logging..."
echo "======================================="
echo ""

# Create a test CSV with more data
cat > test_data.csv << 'EOF'
id,name,age,city,country
1,Alice,30,New York,USA
2,Bob,25,London,UK
3,Charlie,35,Paris,France
4,Diana,28,Tokyo,Japan
5,Eve,32,Berlin,Germany
6,Frank,29,Sydney,Australia
7,Grace,31,Toronto,Canada
8,Henry,27,Mumbai,India
EOF

echo "Test data created: test_data.csv"
echo ""

# Set up environment for debug logging
export SQL_CLI_DEBUG=1
export RUST_LOG=debug

echo "What to expect in the logs:"
echo "============================"
echo ""
echo "1. NavigationState logging:"
echo "   - 'Table row selected: X → Y' when moving between rows"
echo "   - 'Column selected: X → Y' when moving between columns"
echo "   - Navigation position updates"
echo ""
echo "2. SelectionState logging:"
echo "   - 'Mode toggled: Row → Cell → Column' when pressing 'v'"
echo "   - Selection sync operations"
echo ""
echo "3. Transaction-like blocks:"
echo "   - Fewer individual buffer operations"
echo "   - Grouped state updates"
echo ""
echo "4. In F5 Debug Mode, you should see:"
echo "   - Current position from NavigationState"
echo "   - Selection mode and stats"
echo "   - Synced state between Navigation and Selection"
echo ""

# Show log location
LOG_DIR="$HOME/.local/share/sql-cli/logs"
LATEST_LOG=$(ls -t $LOG_DIR/sql-cli_*.log 2>/dev/null | head -1)

echo "Logs will be written to:"
echo "  $LOG_DIR/"
echo ""
echo "To watch logs in real-time:"
echo "  tail -f $LOG_DIR/sql-cli_*.log | grep -E '(Navigation|Selection|Table row|Column selected|Mode toggled)'"
echo ""
echo "Key sequences to test:"
echo "======================"
echo "1. Arrow keys → Should log navigation updates"
echo "2. 'v' key → Should log selection mode changes"
echo "3. Page Up/Down → Should log batch navigation"
echo "4. F5 → Should show complete state dump"
echo "5. 'y' in different modes → Should show mode-aware yank behavior"
echo ""

# Clean up old logs to make it easier to see new ones
if [ -n "$LATEST_LOG" ]; then
    echo "Previous log: $(basename $LATEST_LOG)"
    echo "New log will be created when you run sql-cli"
fi

echo ""
echo "Run the TUI with:"
echo "  ./target/release/sql-cli test_data.csv"
echo ""
echo "Or with explicit debug output to console:"
echo "  RUST_LOG=sql_cli::app_state_container=debug ./target/release/sql-cli test_data.csv 2>&1 | tee debug.log"