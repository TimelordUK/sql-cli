#!/bin/bash
# Final test of the N key toggle fix with comprehensive logging

set -e

echo "🎯 FINAL N KEY TOGGLE TEST"

# Create test data
cat > test_n_final.csv << EOF
name,age,city
Alice,30,New York
Bob,25,London
Charlie,35,Paris
David,40,Tokyo
Eve,28,Berlin
EOF

echo "✅ Created test data"

echo ""
echo "🔍 RUN THIS COMMAND TO TEST WITH FULL DEBUG LOGGING:"
echo ""
echo "RUST_LOG=info,sql_cli::ui::vim_search_adapter=debug,sql_cli::ui::enhanced_tui=debug ./target/release/sql-cli test_n_final.csv -e \"select * from data\""
echo ""

echo "📋 TEST SEQUENCE:"
echo "1. Press 'N' → should toggle line numbers ✅"
echo "   Look for: 'N key -> toggle row numbers (search inactive)'"
echo ""
echo "2. Press '/' → enter search mode"
echo "   Look for: 'VimSearchAdapter: Activating for vim search'"
echo "   Look for: 'Manually notified VimSearchAdapter of SearchStarted event'"
echo ""
echo "3. Type 'Alice' and press Enter"
echo "   Look for: search results appearing"
echo ""
echo "4. Press Escape → exit search mode"
echo "   Look for: 'VimSearchAdapter: Search ended, clearing'"  
echo "   Look for: 'Manually notified VimSearchAdapter of SearchEnded event'"
echo ""
echo "5. Press 'N' → should toggle line numbers (NOT search navigation)"
echo "   Look for: 'PreviousSearchMatch: is_active=false, has_pattern=false'"
echo "   Look for: 'N key -> toggle row numbers (search inactive)'"
echo ""

echo "🐛 WHAT THE DEBUG LOGS WILL TELL US:"
echo "- If StateDispatcher events are being sent correctly"
echo "- If VimSearchAdapter is receiving and processing the events"
echo "- If is_active and get_pattern are returning the expected values"
echo "- Which branch the PreviousSearchMatch action takes"
echo ""

echo "✅ SUCCESS CRITERIA:"
echo "After step 5, you should see 'N key -> toggle row numbers (search inactive)'"
echo "NOT 'N key -> vim search navigation'"
echo ""

# Don't clean up - keep file for testing
echo "Test file: test_n_final.csv (not cleaning up for manual testing)"