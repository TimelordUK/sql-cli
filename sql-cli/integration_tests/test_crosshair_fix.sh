#!/bin/bash

# Test script to verify crosshair alignment with hidden columns

echo "Testing crosshair alignment with hidden columns..."

# Create test data with clear column names
cat > test_crosshair.csv << 'EOF'
col_A,col_B,col_C,col_D,col_E
A1,B1,C1,D1,E1
A2,B2,C2,D2,E2
A3,B3,C3,D3,E3
EOF

echo "Test data created: col_A, col_B, col_C, col_D, col_E"
echo ""
echo "Instructions for manual testing:"
echo "1. Run: ./target/release/sql-cli test_crosshair.csv -e 'select * from data'"
echo "2. Use 'h' and 'l' to navigate columns - verify crosshair aligns with column headers"
echo "3. Press '-' to hide current column (e.g., hide col_B)"
echo "4. Navigate with 'h' and 'l' again - crosshair should still align correctly"
echo "5. The status line should show the correct column name"
echo ""
echo "Expected behavior after hiding col_B:"
echo "  - Navigation should skip from col_A to col_C"
echo "  - Crosshair should highlight the correct column"
echo "  - Status line should match the highlighted column"
echo ""
echo "Running with debug logging enabled..."

RUST_LOG=sql_cli::ui::viewport_manager=debug,render=debug,navigation=debug \
    ./target/release/sql-cli test_crosshair.csv -e "select * from data"