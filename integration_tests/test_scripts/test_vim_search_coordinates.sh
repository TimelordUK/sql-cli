#!/bin/bash
# Test vim search with coordinate updates

set -e

echo "Creating test data..."
cat > test_vim_search.csv << 'EOF'
id,name,department,status,location
1,Alice,Engineering,Active,London
2,Bob,Marketing,Inactive,Paris
3,Charlie,Engineering,Active,Berlin
4,David,Sales,Active,London
5,Eve,Engineering,Inactive,Tokyo
6,Frank,Marketing,Active,Paris
7,Grace,Engineering,Active,London
8,Henry,Sales,Inactive,Berlin
9,Ivy,Engineering,Active,Tokyo
10,Jack,Marketing,Active,London
EOF

echo "Test data created. Testing vim search..."
echo ""
echo "Instructions for manual testing:"
echo "1. Run: ./target/release/sql-cli test_vim_search.csv"
echo "2. Press '/' to start search"
echo "3. Type 'engineering' and watch cursor move to first match"
echo "4. Press Enter to confirm search"
echo "5. Press 'n' to navigate to next match (should move to next row with Engineering)"
echo "6. Press 'N' to navigate to previous match"
echo "7. Press F5 to verify crosshair coordinates match the cell containing 'Engineering'"
echo ""
echo "Expected behavior:"
echo "- First match should be row 1, column 2 (Alice's Engineering)"
echo "- Next match should be row 3, column 2 (Charlie's Engineering)"
echo "- Crosshair should visually appear on cells containing 'Engineering'"
echo "- No horizontal jumping between columns"

# Run with debug logging
echo ""
echo "Running with debug logging enabled..."
RUST_LOG=vim_search=debug,viewport_manager=debug timeout 2 ./target/release/sql-cli test_vim_search.csv -e "select * from data" 2>&1 | grep -E "vim_search|viewport_manager|Crosshair" | head -20 || true

echo ""
echo "Test setup complete. Please run the manual test."