#!/bin/bash
# Test script to verify StateCoordinator refactoring
# Tests the refactored state coordination methods

echo "Testing StateCoordinator refactoring..."
echo ""
echo "This test verifies that the refactored methods work correctly:"
echo "1. add_dataview - Load CSV files and switch between buffers"
echo "2. set_sql_query - Pre-populate SQL queries" 
echo "3. handle_execute_query - Execute queries and special commands"
echo "4. Navigation methods - goto_first_row, goto_last_row, goto_row"
echo ""
echo "Steps to test:"
echo "1. Load multiple CSV files"
echo "2. Switch between buffers with <leader>n and <leader>p"
echo "3. Check SQL completion suggestions match the current buffer"
echo "4. Execute :help command to test special command handling"
echo "5. Press 'g' to go to first row, 'G' to go to last row"
echo "6. Use vim search '/' and test 'g' resets to first match"
echo ""
echo "Starting sql-cli with StateCoordinator debug logging..."

# Enable debug logging for StateCoordinator
export RUST_LOG=sql_cli::ui::state_coordinator=debug,sql_cli::ui::enhanced_tui=info

# Run with sample files
./target/release/sql-cli ../trades_20k.csv ../data/instruments.csv