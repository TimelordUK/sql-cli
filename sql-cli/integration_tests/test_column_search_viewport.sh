#!/bin/bash
echo "Testing column search viewport scrolling..."

# Create test CSV with many columns
cat > test_search.csv << 'EOF'
col1,col2,col3,col4,col5,col6,col7,col8,col9,col10,col11,col12,col13,col14,col15,col16,col17,col18,col19,col20,col21,col22,col23,col24,col25,col26,col27,col28,col29,col30,col31,col32,platformOrderId,col34,col35,col36,col37,col38,col39,col40
a1,a2,a3,a4,a5,a6,a7,a8,a9,a10,a11,a12,a13,a14,a15,a16,a17,a18,a19,a20,a21,a22,a23,a24,a25,a26,a27,a28,a29,a30,a31,a32,ORDER123,a34,a35,a36,a37,a38,a39,a40
b1,b2,b3,b4,b5,b6,b7,b8,b9,b10,b11,b12,b13,b14,b15,b16,b17,b18,b19,b20,b21,b22,b23,b24,b25,b26,b27,b28,b29,b30,b31,b32,ORDER456,b34,b35,b36,b37,b38,b39,b40
EOF

echo "Created test CSV with platformOrderId at column 32 (visual index 32)"
echo ""
echo "To test:"
echo "1. Run: ./target/release/sql-cli test_search.csv -e \"select * from data\""
echo "2. Press \\ to start column search"
echo "3. Type 'order' to search for columns containing 'order'"
echo "4. Press Tab to navigate to platformOrderId"
echo "5. The viewport should scroll to show column 32"
echo ""
echo "Expected: platformOrderId column should be visible on screen after Tab"