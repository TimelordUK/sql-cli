#!/bin/bash

echo "Testing hide column with ViewportManager crosshair"
echo "==================================================="
echo ""
echo "This test verifies that hiding a column uses the visual position"
echo "from ViewportManager's crosshair, not Buffer's DataTable index."
echo ""
echo "Test scenario:"
echo "1. Load data with columns: comments (index 4), dV01 (index 14)"
echo "2. Hide 'comments' column"
echo "3. Navigate to 'dV01' (now at visual position 13)"
echo "4. Press '-' to hide column"
echo "5. Should hide 'dV01', not 'dealId'"
echo ""

# Create test CSV with known columns
cat > test_hide_fix.csv << 'EOF'
accruedInterest,allocationStatus,book,clearingHouse,comments,commission,confirmationStatus,counterparty,counterpartyCountry,counterpartyId,counterpartyType,createdDate,currency,cusip,dV01,dealId,delta,desk,duration,exchange
100.5,Pending,Trading,ICE,Note1,50.25,Confirmed,ABC,US,123,Corp,2024-01-01,USD,12345,0.05,DEAL001,0.1,Trading,2.5,NYSE
200.75,Allocated,Options,CME,Note2,75.50,Pending,XYZ,UK,456,Bank,2024-01-02,EUR,67890,0.08,DEAL002,0.2,Options,3.5,LSE
EOF

echo "Test data created: test_hide_fix.csv"
echo ""
echo "To test manually:"
echo "1. ./target/release/sql-cli test_hide_fix.csv"
echo "2. Press 'H' and hide 'comments' column"
echo "3. Navigate right with 'l' until you reach 'dV01'"
echo "4. Verify status line shows: Col: dV01 [V:0,13]"
echo "5. Press '-' to hide column"
echo "6. Verify 'dV01' is hidden (not 'dealId')"
echo ""
echo "Running with debug logging..."
RUST_LOG=sql_cli::ui::enhanced_tui=debug,viewport_manager=debug ./target/release/sql-cli test_hide_fix.csv -e "select * from data"