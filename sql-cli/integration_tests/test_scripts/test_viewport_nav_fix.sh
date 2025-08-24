#!/bin/bash
# Test script to verify viewport navigation fixes

set -e

echo "Testing viewport navigation (H, L, M keys)..."
echo "Creating test data with 100 rows..."

# Create test CSV with 100 rows
cat > test_nav_fix.csv << 'EOF'
id,name,value
EOF

for i in {1..100}; do
    echo "$i,row_$i,$((i * 10))" >> test_nav_fix.csv
done

echo "Running test..."
echo "Press:"
echo "  - Shift+L to go to bottom of viewport (should go to row 78)"
echo "  - Shift+M to go to middle (should go to row 39)"
echo "  - Shift+H to go to top (should go to row 0)"
echo "  - j to go down from row 78 (should stop at 78, not go beyond)"
echo ""

RUST_LOG=sql_cli::ui::viewport_manager=debug timeout 10 ./target/release/sql-cli test_nav_fix.csv -e "select * from data" 2>&1 | grep -E "navigate_to_viewport|crosshair.*->" | tail -20

rm -f test_nav_fix.csv

echo "Test complete!"