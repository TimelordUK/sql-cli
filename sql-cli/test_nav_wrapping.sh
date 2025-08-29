#!/bin/bash

echo "Testing navigation wrapping issue at specific columns..."

# Create test data with enough columns to reproduce the issue
cat > test_wrap_issue.csv << 'EOF'
col1,col2,col3,col4,col5,col6,col7,col8,col9,col10,col11,col12,col13,col14,col15,col16,col17,col18,col19,col20,col21,col22,col23,col24,col25
a1,a2,a3,a4,a5,a6,a7,a8,a9,a10,a11,a12,a13,a14,a15,a16,a17,a18,a19,a20,a21,a22,a23,a24,a25
b1,b2,b3,b4,b5,b6,b7,b8,b9,b10,b11,b12,b13,b14,b15,b16,b17,b18,b19,b20,b21,b22,b23,b24,b25
EOF

echo "Test 1: Navigate to column 20-21 and press right arrow"
echo "Expected: Move to column 21-22"
echo "Actual behavior will be shown in logs..."
echo ""

# Navigate right 20 times to get to column 20/21, then press right again
navigation_sequence=""
for i in {1..21}; do
    navigation_sequence="${navigation_sequence}l"
done
navigation_sequence="${navigation_sequence}q"

echo -e "$navigation_sequence" | RUST_LOG=sql_cli::ui::viewport_manager=debug timeout 3 ./target/release/sql-cli test_wrap_issue.csv -e "select * from data" 2>&1 | grep -E "(navigate_column_right|WARNING|RESULT|Movement:)" | tail -30

rm test_wrap_issue.csv