#!/bin/bash

echo "Testing Goto Line Navigation"
echo "============================"
echo ""

# Create test CSV with numbered rows
cat > test_goto.csv << 'EOF'
id,name,value
EOF

for i in {1..100}; do
    echo "$i,Row$(printf %03d $i),$((i * 100))" >> test_goto.csv
done

echo "Test data created: test_goto.csv (100 rows)"
echo ""
echo "Test Instructions:"
echo "1. Press ':' to enter goto line mode"
echo "2. Type '50' and press Enter - should jump to row 50 with crosshair"
echo "3. Type ':1' and Enter - should jump back to row 1"
echo "4. Type ':100' and Enter - should jump to last row"
echo ""
echo "Manual test:"
echo "  ./target/release/sql-cli test_goto.csv -e \"select * from data\""
echo ""
echo "Debug mode:"
echo "  RUST_LOG=sql_cli::ui::viewport_manager=debug,navigation=info ./target/release/sql-cli test_goto.csv -e \"select * from data\" 2>&1 | grep -E 'goto_line|Jump-to-row'"