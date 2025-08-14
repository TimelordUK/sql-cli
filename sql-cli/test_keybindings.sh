#!/bin/bash
# Test script for hide column keybindings

cat > test_keybindings.csv << 'EOF'
col1,col2,col3,col4,col5
A,B,C,D,E
1,2,3,4,5
X,Y,Z,W,V
EOF

echo "Hide Column Feature - Keybinding Test"
echo "======================================"
echo ""
echo "Multiple keybindings are available:"
echo "  • Ctrl+H    - Hide current column (may not work in some terminals)"
echo "  • Alt+H     - Hide current column (alternative)"
echo "  • -         - Hide current column (minus key)"
echo "  • + or =    - Unhide all columns"
echo "  • Ctrl+Shift+H - Unhide all columns"
echo ""
echo "Test Instructions:"
echo "1. Run: SELECT * FROM data"
echo "2. Try pressing the minus (-) key to hide the current column"
echo "3. Use arrow keys to navigate to other columns"
echo "4. Press minus (-) again to hide more columns"
echo "5. Press plus (+) or equals (=) to unhide all"
echo ""
echo "Starting SQL-CLI with debug logging..."

RUST_LOG=debug ./target/release/sql-cli test_keybindings.csv 2>&1 | tee test_keybindings.log &
PID=$!

echo ""
echo "SQL-CLI is running with PID $PID"
echo "Debug output is being saved to test_keybindings.log"
echo ""
echo "After testing, you can:"
echo "  • Check the log: grep -E '(hide_current_column|Minus key|Alt\+H)' test_keybindings.log"
echo "  • Kill the process: kill $PID"

wait $PID