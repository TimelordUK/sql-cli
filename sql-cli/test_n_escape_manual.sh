#!/bin/bash

# Manual test script for N key toggle fix
echo "=== Manual Test for N Key Toggle After Search ==="
echo ""
echo "Creating test data..."

cat > test_n_manual.csv << 'EOF'
id,name,type,status
1,Alice,derivatives,active
2,Bob,futures,pending
3,Charlie,derivatives,active
4,David,options,inactive
5,Eve,derivatives,pending
EOF

echo "Test data created: test_n_manual.csv"
echo ""
echo "INSTRUCTIONS:"
echo "1. Run: RUST_LOG=info ./target/release/sql-cli test_n_manual.csv -e \"select * from data\""
echo "2. Press '/' to enter search mode"
echo "3. Type 'derivatives' and press Enter"
echo "4. Press 'n' a few times to navigate search results"
echo "5. Press Escape to clear search"
echo "6. Press 'N' - it should toggle line numbers, NOT navigate search"
echo ""
echo "Look for these log messages:"
echo "- 'VimSearchAdapter: ESCAPE pressed with active search - clearing'"
echo "- 'Buffer.set_search_pattern: 'derivatives' -> '''"
echo "- 'Search cleared'"
echo ""
echo "Starting application with logging..."
echo ""

RUST_LOG=info ./target/release/sql-cli test_n_manual.csv -e "select * from data" 2>&1 | grep -E "VimSearchAdapter|set_search_pattern|Search cleared|Toggled line numbers|KeyPressed"