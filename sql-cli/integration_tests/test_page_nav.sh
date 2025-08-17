#!/bin/bash

echo "Testing Page Up/Down Navigation"
echo "================================"
echo ""

# Create test CSV with 200 rows to test page navigation
cat > test_page_nav.csv << 'EOF'
id,name,value,status
EOF

for i in {1..200}; do
    echo "$i,Row$(printf %03d $i),$((i * 100)),status$((i % 4))" >> test_page_nav.csv
done

echo "Test data created: test_page_nav.csv (200 rows)"
echo ""
echo "Test 1: Normal Page Navigation"
echo "------------------------------"
echo "1. Press PageDown or Ctrl+F - should move down ~79 rows"
echo "2. Press PageUp or Ctrl+B - should move up ~79 rows"
echo ""
echo "Test 2: With Cursor Lock (x)"
echo "-----------------------------"
echo "1. Press 'x' to enable cursor lock"
echo "2. Press PageDown - viewport should scroll, cursor stays at same position"
echo "3. Press PageUp - viewport should scroll back"
echo ""
echo "Test 3: With Viewport Lock (Ctrl+Space)"
echo "----------------------------------------"
echo "1. Press Ctrl+Space to enable viewport lock"
echo "2. Press PageDown - cursor should jump to bottom of current viewport"
echo "3. Press PageUp - cursor should jump to top of current viewport"
echo "4. No scrolling should occur with viewport lock"
echo ""
echo "Manual test:"
echo "  ./target/release/sql-cli test_page_nav.csv -e \"select * from data\""
echo ""
echo "Debug mode (with logs):"
echo "  RUST_LOG=sql_cli::ui::viewport_manager=debug ./target/release/sql-cli test_page_nav.csv -e \"select * from data\" 2>&1 | grep page_"