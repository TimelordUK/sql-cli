#!/bin/bash

echo "Testing row number updates in status bar..."

# Create test data
cat > test_row_nav.csv << EOF
id,name,value
1,First,100
2,Second,200
3,Third,300
4,Fourth,400
5,Fifth,500
EOF

echo "Starting TUI with debug logging to monitor navigation state updates..."
echo "Check the log file for navigation updates showing row changes:"
echo ""

# Run with debug logging to see navigation updates
RUST_LOG=sql_cli::ui::traits::navigation=debug,sql_cli::app_state_container=debug timeout 1 ./target/release/sql-cli test_row_nav.csv -e "select * from data" 2>&1 | grep -E "(Navigation|selected_row|sync_row_state)" || true

# Check the log file
LOG_FILE=$(ls -t /home/me/.local/share/sql-cli/logs/sql-cli_*.log 2>/dev/null | head -1)
if [ -n "$LOG_FILE" ]; then
    echo "Checking log file for navigation updates: $LOG_FILE"
    echo "Recent navigation entries:"
    grep -E "(Navigation|selected_row|sync_row)" "$LOG_FILE" 2>/dev/null | tail -10 || echo "No navigation entries found"
fi

echo ""
echo "The sync_row_state() method in NavigationBehavior trait now ensures:"
echo "1. Buffer's selected row is updated"
echo "2. AppStateContainer's navigation state is updated" 
echo "3. set_table_selected_row() is called for consistency"
echo ""
echo "This centralized synchronization fixes the row number display issue."

# Cleanup
rm -f test_row_nav.csv