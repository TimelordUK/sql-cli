#!/bin/bash
# Test with comprehensive debugging to see exact state values

echo "🔍 COMPREHENSIVE DEBUG TEST"
echo "Testing with enhanced logging to see exact VimSearchAdapter state..."

# Create test data  
cat > test_debug_final.csv << EOF
name,age,city
Alice,30,New York
Bob,25,London
Charlie,35,Paris
EOF

echo ""
echo "🎯 RUN THIS COMMAND:"
echo "RUST_LOG=info,sql_cli::ui::enhanced_tui=debug,sql_cli::ui::vim_search_adapter=debug,vim_search=debug ./target/release/sql-cli test_debug_final.csv -e \"select * from data\""
echo ""

echo "📝 EXPECTED LOGS AFTER PRESSING N (after search exit):"
echo "- 'Action system: key Char('N') -> action PreviousSearchMatch'"  
echo "- '🎯 PreviousSearchMatch: is_active=?, has_pattern=?, pattern=?'"
echo "- Either 'N key -> vim search navigation' OR 'N key -> toggle row numbers'"
echo "- '🎯 ToggleActionHandler: Processing ToggleRowNumbers action' (if working)"
echo "- '🎯 TOGGLE_ROW_NUMBERS CALLED: current=?, will set to=?' (if working)"
echo ""

echo "🐛 THE ISSUE:"
echo "If you see 'Searching for ...' logs after pressing N (post-search),"
echo "it means VimSearchAdapter still has a pattern stored even after clear!"
echo ""

# Keep the test file
echo "Test file: test_debug_final.csv"