#!/bin/bash

echo "Testing navigation with pinned columns..."

# Create test data with many columns to force scrolling
cat > test_nav_many_cols.csv << 'EOF'
id,name,book,category,price,quantity,status,location,description,notes,timestamp,user,external_id,fees,total
1,Alice,Book1,Fiction,19.99,5,Available,StoreA,Great story,Bestseller,2024-01-01,admin,EXT001,2.50,22.49
2,Bob,Book2,Science,29.99,3,Available,StoreB,Educational,Popular,2024-01-02,user1,EXT002,3.00,32.99
3,Charlie,Book3,History,24.99,7,Sold,StoreC,Interesting,Classic,2024-01-03,user2,EXT003,2.75,27.74
EOF

echo "Running with debug logging to trace navigation..."
echo -e "p\nlllllllllll\nq" | RUST_LOG=sql_cli::ui::viewport_manager=debug timeout 3 ./target/release/sql-cli test_nav_many_cols.csv -e "select * from data" 2>&1 | grep -E "(navigate_column_right|column_position|display_pos|datatable_col|visual/display)" | tail -20

echo ""
echo "Checking specific issue at external_id column..."
# Pin book column, navigate to external_id, then press l to see if it goes to fees or back to book
echo -e "p\nlllllllllll\nq" | timeout 3 ./target/release/sql-cli test_nav_many_cols.csv -e "select * from data" 2>&1 | grep -E "Navigate|column '|selected" | tail -10

rm test_nav_many_cols.csv