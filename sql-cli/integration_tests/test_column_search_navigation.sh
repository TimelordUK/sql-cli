#!/bin/bash

echo "Testing Column Search Navigation Fix"
echo "====================================="
echo ""

# Create test CSV with many columns
cat > test_column_search_nav.csv << 'EOF'
id,name,orderid,value,status,externalOrderId,parentOrderId,platformOrderId,description
1,Row1,ORD001,100,active,EXT001,PAR001,PLAT001,Test description 1
2,Row2,ORD002,200,inactive,EXT002,PAR002,PLAT002,Test description 2
3,Row3,ORD003,300,active,EXT003,PAR003,PLAT003,Test description 3
4,Row4,ORD004,400,pending,EXT004,PAR004,PLAT004,Test description 4
5,Row5,ORD005,500,active,EXT005,PAR005,PLAT005,Test description 5
EOF

echo "Test data created: test_column_search_nav.csv"
echo ""
echo "Test Instructions:"
echo "1. Run: ./target/release/sql-cli test_column_search_nav.csv -e \"select * from data\""
echo "2. Press '\' to start column search"
echo "3. Type 'order' and press Enter"
echo "4. Should automatically navigate to 'orderid' column (first match)"
echo "5. Press Tab to navigate to 'externalOrderId' (second match)"
echo "6. Press Tab again for 'parentOrderId' (third match)"
echo "7. Press Tab again for 'platformOrderId' (fourth match)"
echo "8. Shift+Tab should cycle backwards"
echo ""
echo "Expected behavior:"
echo "- When searching for 'order', viewport should scroll to show 'orderid' immediately"
echo "- Crosshair should be on the matched column"
echo "- Tab/Shift+Tab should navigate between all matches"
echo ""
echo "Debug mode (to see visual indices):"
echo "RUST_LOG=search=debug,navigation=debug ./target/release/sql-cli test_column_search_nav.csv -e \"select * from data\" 2>&1 | grep -E 'column search|visual index'"