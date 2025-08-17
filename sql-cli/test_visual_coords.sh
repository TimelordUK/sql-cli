#!/bin/bash

# Test that everything uses visual coordinates correctly

echo "Testing visual coordinate system with hidden columns..."

# Create test data with distinct values for each cell
cat > test_visual.csv << 'EOF'
Col_A,Col_B,Col_C,Col_D,Col_E,Col_F
A1,B1,C1,D1,E1,F1
A2,B2,C2,D2,E2,F2
A3,B3,C3,D3,E3,F3
EOF

echo "Created test data:"
echo "  Columns: Col_A, Col_B, Col_C, Col_D, Col_E, Col_F"
echo "  Data pattern: XN where X=column letter, N=row number"
echo ""
echo "Test procedure:"
echo "1. Start with all 6 columns visible"
echo "2. Navigate to Col_C (visual position 2)"
echo "3. Hide Col_C with '-' key"
echo "4. After hiding Col_C:"
echo "   - Headers should show: Col_A, Col_B, Col_D, Col_E, Col_F"
echo "   - Data should show: A1, B1, D1, E1, F1 (NOT C1!)"
echo "   - Visual position 2 should now be Col_D"
echo "   - Crosshair should highlight Col_D header and D1 data"
echo "5. Navigate right with 'l' - should move to Col_E (visual position 3)"
echo ""
echo "Key points to verify:"
echo "- Headers align with data columns"
echo "- Crosshair highlights the correct column"
echo "- Navigation uses visual positions (0,1,2,3,4) not DataTable indices"
echo "- Status line shows the correct column name"
echo ""
echo "Running application..."

RUST_LOG=render=debug,sql_cli::ui::viewport_manager=debug \
    ./target/release/sql-cli test_visual.csv -e "select * from data"