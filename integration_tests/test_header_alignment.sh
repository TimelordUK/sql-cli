#!/bin/bash

# Test script to verify header alignment with hidden columns

echo "Testing header alignment when columns are hidden..."

# Create test data with clear, distinct column names
cat > test_headers.csv << 'EOF'
A_Column,B_Column,C_Column,D_Column,E_Column,F_Column
A1_data,B1_data,C1_data,D1_data,E1_data,F1_data
A2_data,B2_data,C2_data,D2_data,E2_data,F2_data
A3_data,B3_data,C3_data,D3_data,E3_data,F3_data
A4_data,B4_data,C4_data,D4_data,E4_data,F4_data
EOF

echo "Test data created with columns: A_Column, B_Column, C_Column, D_Column, E_Column, F_Column"
echo ""
echo "Test procedure:"
echo "1. The app will start with all columns visible"
echo "2. Navigate to C_Column using 'l' key twice"
echo "3. Press '-' to hide C_Column"
echo "4. Verify that:"
echo "   - Headers now show: A_Column, B_Column, D_Column, E_Column, F_Column"
echo "   - Data under D_Column header shows 'D*_data' values (not C*_data)"
echo "   - The crosshair highlights the correct column matching the status line"
echo ""
echo "Running with debug logging..."

RUST_LOG=render=debug,sql_cli::ui::viewport_manager=debug \
    ./target/release/sql-cli test_headers.csv -e "select * from data"