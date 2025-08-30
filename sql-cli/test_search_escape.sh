#!/bin/bash
# Test script to verify search and Escape behavior

echo "Testing search and Escape behavior..."
echo ""
echo "Steps to test:"
echo "1. Load a CSV file"
echo "2. Press '/' to start vim search"
echo "3. Type a search term (e.g., 'unalloc')"
echo "4. Press Enter to apply search"
echo "5. Press 'n' - should navigate to next match"
echo "6. Press 'N' - should toggle line numbers (NOT navigate)"
echo "7. Press Escape - should clear search completely"
echo "8. Press 'n' - should do nothing (no search active)"
echo "9. Press 'N' - should toggle line numbers"
echo ""
echo "Starting sql-cli..."

# Enable debug logging for search and StateCoordinator
export RUST_LOG=sql_cli::ui::state_coordinator=debug,sql_cli::ui::vim_search_adapter=info,search=debug

# Run with a sample file
./target/release/sql-cli ../trades_20k.csv