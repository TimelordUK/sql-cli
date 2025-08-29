#!/bin/bash

echo "Testing exact navigation issue..."
echo "1. Navigate to column 20 (col20)"
echo "2. Press 'l' to go right"
echo "3. Should go to column 21 (col21), not back to column 0"
echo ""

# Navigate to column 20, then press l
seq=""
for i in {1..20}; do
    seq="${seq}l"
done
seq="${seq}lq"  # One more 'l' to trigger the issue, then quit

echo -e "$seq" | RUST_LOG=sql_cli::ui::viewport_manager=debug timeout 2 ./target/release/sql-cli test_nav_simple.csv -e "select * from data" 2>&1 | grep -E "(navigate_column_right|Movement:|Returning:|WARNING)" | tail -10

echo ""
echo "Without pinned columns:"
echo -e "$seq" | timeout 2 ./target/release/sql-cli test_nav_simple.csv -e "select * from data" 2>&1 | grep -i "column" | tail -5