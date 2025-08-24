#!/bin/bash

echo "Testing if j key triggers TableWidgetManager"
echo "============================================"
echo ""

# Create simple test data
cat > test_j_debug.csv << 'CSV'
id,name
1,Alice
2,Bob
3,Charlie
4,David
5,Eve
CSV

echo "Running with specific TableWidgetManager and navigation logging..."
echo "Press 'j' a few times and look for TableWidgetManager logs"
echo ""

# Run with focused logging on TableWidgetManager and navigation
RUST_LOG=sql_cli::ui::table_widget_manager=info,navigation=info,sql_cli::ui::enhanced_tui=info timeout 5 ./target/release/sql-cli test_j_debug.csv -e "select * from data" 2>&1 | grep -E "(TableWidgetManager|sync_row_state|Navigate.*Down|next_row)" | head -20

echo ""
echo "Test complete. Check if j key shows TableWidgetManager activity."

# Clean up
rm -f test_j_debug.csv