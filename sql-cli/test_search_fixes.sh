#!/bin/bash

echo "Testing search navigation fixes:"
echo "================================"
echo ""
echo "1. Testing 'g' key to reset search to first match"
echo "2. Testing column scrolling for off-screen matches"
echo ""

# Create test data with many columns
cat > test_search_fixes.csv << 'CSV'
id,book,product,status,field5,field6,field7,field8,field9,field10,field11,field12,field13,field14,field15,field16,field17,structuredProduct,field19,field20
1,Fixed Income,Corporate,active,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,StructuredNote,data15,data16
2,Commodities,Energy,emerging,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,Regular,data15,data16
3,Equities,Tech,active,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,Regular,data15,data16
4,Forex,EUR/USD,pending,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,Regular,data15,data16
5,Derivatives,Options,active,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,Regular,data15,data16
6,Fixed Income,emerging,pending,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,StructuredBond,data15,data16
7,Commodities,Gold,active,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,Regular,data15,data16
8,Fixed Income,Government,emerging,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,Regular,data15,data16
9,Equities,emerging,active,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,Regular,data15,data16
10,Bonds,Corporate,emerging,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,StructuredCredit,data15,data16
CSV

echo "Test data created with 20 columns"
echo ""
echo "Running test with search logging..."
echo ""
echo "Instructions for manual testing:"
echo "1. Press '/' to start search"
echo "2. Type 'emerging' and press Enter"
echo "3. Press 'n' a few times to navigate forward"
echo "4. Press 'g' to reset to first match"
echo "5. Try searching for 'Structured' (in column 18)"
echo ""
echo "Starting TUI (will timeout after 10 seconds)..."

RUST_LOG=vim_search=info,search=info timeout 10 ./target/release/sql-cli test_search_fixes.csv -e "select * from data" 2>&1 | grep -E "(Reset|Scroll|column|navigate)" | tail -30

echo ""
echo "Test complete. Check if:"
echo "1. 'g' key resets search to first match"
echo "2. Column scrolling works for off-screen matches"

# Clean up
rm -f test_search_fixes.csv