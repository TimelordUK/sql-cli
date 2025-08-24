#!/bin/bash

echo "Testing navigation triggers TableWidgetManager rendering"
echo "========================================================="
echo ""

# Create test data
cat > test_nav_render.csv << 'CSV'
id,book,product,status
1,Fixed Income,Corporate,active
2,Commodities,Energy,emerging
3,Equities,Tech,active
4,Forex,EUR/USD,pending
5,Derivatives,Options,active
CSV

echo "Running test with TableWidgetManager logging..."
echo "Look for 'TableWidgetManager: RENDERING TABLE' messages when pressing j/k/h/l"
echo ""

# Run with specific logging to see TableWidgetManager render logs
RUST_LOG=sql_cli::ui::table_widget_manager=info,navigation=info timeout 5 ./target/release/sql-cli test_nav_render.csv -e "select * from data" 2>&1 | grep -E "TableWidgetManager|navigation|sync_row_state" | head -30

echo ""
echo "Test complete. Check if navigation triggers TableWidgetManager renders."

# Clean up
rm -f test_nav_render.csv