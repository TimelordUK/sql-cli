#!/bin/bash

echo "Testing column name display with hidden columns"
echo "================================================"
echo ""
echo "This test verifies that the status line shows the correct column name"
echo "when using ViewportManager's crosshair position instead of Buffer's column."
echo ""
echo "Test scenario:"
echo "1. Load data with 'comments' at index 4"
echo "2. Hide 'comments' column"
echo "3. Navigate to visual position 4 (which is now 'commission')"
echo "4. Status line should show 'commission', not 'confirmationStatus'"
echo ""
echo "Visual mapping after hiding 'comments' (index 4):"
echo "  V[0] -> DT[0]: accruedInterest"
echo "  V[1] -> DT[1]: allocationStatus"
echo "  V[2] -> DT[2]: book"
echo "  V[3] -> DT[3]: clearingHouse"
echo "  V[4] -> DT[5]: commission        <-- Visual 4 maps to DataTable 5"
echo "  V[5] -> DT[6]: confirmationStatus"
echo ""

# Create test CSV with known column order
cat > test_column_display.csv << 'EOF'
accruedInterest,allocationStatus,book,clearingHouse,comments,commission,confirmationStatus,counterparty
100.5,Pending,Trading,ICE,Important note,50.25,Confirmed,ABC Corp
200.75,Allocated,Options,CME,Another comment,75.50,Pending,XYZ Inc
EOF

echo "Test data created: test_column_display.csv"
echo ""
echo "To test manually:"
echo "1. ./target/release/sql-cli test_column_display.csv"
echo "2. Press 'H' to enter hide mode"
echo "3. Navigate to 'comments' (column 4) and press Enter to hide it"
echo "4. Press 'h' to move left to column 4 (should be on 'commission')"
echo "5. Check status line shows: Col: commission [V:0,4]"
echo "6. Press F5 to verify ViewportManager Crosshair shows: row=0, col=4"
echo ""
echo "Running with debug logging..."
RUST_LOG=sql_cli::ui::viewport_manager=debug,render=debug ./target/release/sql-cli test_column_display.csv -e "select * from data"