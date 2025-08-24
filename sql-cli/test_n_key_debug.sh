#!/bin/bash
# Test the N key toggle fix with detailed debug logging

set -e

echo "Testing N key toggle fix with debug logging..."

# Create test data
cat > test_n_debug.csv << EOF
name,age,city
Alice,30,New York
Bob,25,London
Charlie,35,Paris
David,40,Tokyo
Eve,28,Berlin
Frank,33,Rome
Grace,27,Madrid
Henry,45,Dublin
EOF

echo "✅ Created test data with 8 rows"

# Build the project to ensure latest changes
cargo build --release

echo ""
echo "🎯 TESTING WITH DEBUG LOGS:"
echo "Run this command to see detailed logs:"
echo "RUST_LOG=info,sql_cli::ui::vim_search_adapter=debug,sql_cli::state=debug ./target/release/sql-cli test_n_debug.csv -e \"select * from data\""
echo ""
echo "Test sequence:"
echo "1. Press 'N' → should toggle line numbers"
echo "2. Press '/' → enter search mode" 
echo "3. Type 'Alice'"
echo "4. Press Escape → exit search mode (look for StateDispatcher event in logs)"
echo "5. Press 'N' → should toggle line numbers (NOT navigate search)"
echo ""
echo "Look for these log entries:"
echo "- 'VimSearchAdapter: should_handle_key? mode=Results, pattern='', active=false'"
echo "- 'StateDispatcher dispatching event: SearchEnded'"
echo "- 'VimSearchAdapter: Search ended, clearing'"
echo ""

# Don't clean up - keep file for testing
echo "Test file: test_n_debug.csv (not cleaning up for manual testing)"