#!/bin/bash

# Test search navigation with TableWidgetManager
echo "Testing search navigation with TableWidgetManager..."

# Create test data with "emerging" in specific rows
cat > test_search_nav.csv << 'CSV'
id,book,product
1,Fixed Income,Corporate
2,Commodities,Energy
3,Equities,Tech
4,Forex,EUR/USD
5,Derivatives,Options
6,Fixed Income,emerging
7,Commodities,Gold
8,Fixed Income,emerging
9,Equities,emerging
CSV

# Run with debug logging for search and table widget manager
RUST_LOG=sql_cli::ui::table_widget_manager=debug,search=info timeout 3 ./sql-cli/target/release/sql-cli test_search_nav.csv -e "select * from data" 2>&1 | grep -E "(TableWidgetManager|search|MATCH|Navigate)" | head -30

echo ""
echo "Test complete - check logs for TableWidgetManager navigation updates"

# Clean up
rm -f test_search_nav.csv
