#!/bin/bash

# Test script for vim-like search functionality
echo "Testing vim-like search functionality..."

# Create a test CSV file with some data
cat > test_vim_search.csv << EOF
id,name,age,city,status
1,Alice,30,New York,active
2,Bob,25,Los Angeles,inactive
3,Charlie,35,Chicago,active
4,David,28,Houston,pending
5,Eve,32,Phoenix,active
6,Frank,29,Philadelphia,inactive
7,Grace,31,San Antonio,active
8,Helen,27,San Diego,pending
9,Ivan,33,Dallas,active
10,Julia,26,San Jose,inactive
EOF

echo "Created test_vim_search.csv with sample data"
echo ""
echo "To test the vim search feature:"
echo "1. Run: ./target/release/sql-cli test_vim_search.csv"
echo "2. Execute a query: select * from data"
echo "3. Press / to start search mode"
echo "4. Type 'active' and watch it navigate to first match dynamically"
echo "5. Press Enter to confirm search and enter navigation mode"
echo "6. Press n to go to next match (wraps around at end)"
echo "7. Press N to go to previous match (wraps around at beginning)"
echo "8. Press ESC to exit search mode and stay in Results"
echo "9. Press ESC again to return to Command mode"
echo ""
echo "Expected behavior:"
echo "- Search is case-insensitive by default ('active' matches 'Active')"
echo "- As you type, cursor jumps to first match immediately"
echo "- The viewport scrolls to keep matches visible"
echo "- After Enter, you're in vim search navigation mode"
echo "- Status bar shows 'Match X/Y at (row, col)'"
echo "- ESC exits search mode but keeps you in Results mode"
echo "- Second ESC takes you back to Command mode"

# Run with debug logging enabled
echo ""
echo "Running with vim_search debug logging enabled..."
echo "Watch for 'viewport-relative' coordinates in the logs to verify the fix"
RUST_LOG=vim_search=debug timeout 30 ./target/release/sql-cli test_vim_search.csv -e "select * from data" 2>&1 | tee vim_search_test.log || true

echo ""
echo "Test logs saved to vim_search_test.log"
echo "You can grep for 'vim_search' to see the search behavior"