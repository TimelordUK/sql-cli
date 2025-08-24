#!/bin/bash

echo "Testing column scrolling for off-screen search matches"
echo "======================================================"
echo ""

# Create test data with structured in column 18 (off-screen)
cat > test_column_scroll.csv << 'CSV'
id,book,product,status,field5,field6,field7,field8,field9,field10,field11,field12,field13,field14,field15,field16,field17,structuredProduct,field19,field20
1,Fixed Income,Corporate,active,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,StructuredNote,data15,data16
2,Commodities,Energy,emerging,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,Regular,data15,data16
3,Equities,Tech,active,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,Regular,data15,data16
4,Forex,EUR/USD,pending,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,StructuredBond,data15,data16
5,Derivatives,Options,active,data1,data2,data3,data4,data5,data6,data7,data8,data9,data10,data11,data12,data13,Regular,data15,data16
CSV

echo "Testing with debug logging for column scrolling..."
echo "Look for 'Match column' and 'scroll' messages in the logs"
echo ""

# Run with specific logging for search column scrolling
RUST_LOG=search=debug timeout 3 ./target/release/sql-cli test_column_scroll.csv -e "select * from data" 2>&1 | grep -E "Match column|col_offset|scroll" | head -20

echo ""
echo "If column scrolling is working:"
echo "- You should see 'Match column XX is right of viewport' messages"
echo "- You should see 'Set viewport to row_offset=X, col_offset=Y' with non-zero col_offset"
echo ""

# Clean up
rm -f test_column_scroll.csv