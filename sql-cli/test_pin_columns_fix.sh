#!/bin/bash

echo "Testing Pin Columns Feature"
echo "============================"
echo ""

# Create test data with many columns
cat > test_pin_wide.csv << EOF
id,name,category,price,quantity,status,date,location,description,notes
1,Apple,Fruit,2.50,100,Available,2024-01-01,Store A,Fresh apples,Good quality
2,Banana,Fruit,1.25,150,Available,2024-01-02,Store B,Yellow bananas,Ripe
3,Carrot,Vegetable,0.75,200,Available,2024-01-03,Store C,Orange carrots,Fresh
4,Desk,Furniture,150.00,10,Available,2024-01-04,Store D,Wooden desk,Sturdy
5,Eggs,Dairy,3.99,50,Low Stock,2024-01-05,Store A,Dozen eggs,Organic
EOF

echo "Test 1: Pin a column and verify it stays visible when scrolling"
echo "----------------------------------------------------------------"
echo "1. Load CSV with multiple columns"
echo "2. Navigate to 'name' column (press 'l' once)"
echo "3. Pin the column (press 'p')"
echo "4. Scroll right multiple times (press right arrow)"
echo "5. The 'name' column should remain visible on the left with pin indicator"
echo ""

# Run with debug logging for pin operations
RUST_LOG=sql_cli::data::data_view=debug,sql_cli::ui::viewport_manager=debug timeout 3 ./target/release/sql-cli test_pin_wide.csv -e "select * from data" 2>&1 | grep -i "pin" | head -10

echo ""
echo "Test 2: Check viewport calculation with pinned columns"
echo "------------------------------------------------------"
RUST_LOG=viewport_manager=debug timeout 2 ./target/release/sql-cli test_pin_wide.csv -e "select * from data" 2>&1 | grep "calculate_visible_column" | head -5

echo ""
echo "Test 3: Visual indicators"
echo "-------------------------"
echo "Expected visual features:"
echo "- Pinned columns have 📌 indicator in header"
echo "- Darker blue background for pinned columns"
echo "- Vertical separator │ between pinned and scrollable columns"
echo "- Pinned columns stay fixed when scrolling horizontally"
echo ""

echo "Manual Test Instructions:"
echo "1. Run: ./target/release/sql-cli test_pin_wide.csv"
echo "2. Press 'l' to move to 'name' column"
echo "3. Press 'p' to pin the column"
echo "4. Press right arrow multiple times to scroll"
echo "5. Verify 'name' column stays visible on left"
echo "6. Press 'P' (Shift+P) to clear all pins"
echo ""

# Clean up
rm -f test_pin_wide.csv

echo "Test complete!"