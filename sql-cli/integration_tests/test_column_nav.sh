#!/bin/bash

echo "Testing $ and ^ Column Navigation"
echo "=================================="
echo ""

# Create test CSV with many columns
cat > test_cols.csv << 'EOF'
id,name,value,status,description,category,priority,date,owner,team
1,Row001,100,active,Test description 1,cat_a,high,2024-01-01,alice,team1
2,Row002,200,inactive,Test description 2,cat_b,medium,2024-01-02,bob,team2
3,Row003,300,active,Test description 3,cat_c,low,2024-01-03,charlie,team1
4,Row004,400,pending,Test description 4,cat_a,high,2024-01-04,david,team3
5,Row005,500,active,Test description 5,cat_b,medium,2024-01-05,eve,team2
EOF

echo "Test data created: test_cols.csv (10 columns)"
echo ""
echo "Test Instructions:"
echo "1. Press '^' - should jump to first column (id) with crosshair"
echo "2. Press '$' - should jump to last column (team) with crosshair"
echo "3. Press 'l' a few times to move right, then '^' - should return to first column"
echo "4. Press 'h' a few times to move left, then '$' - should jump to last column"
echo ""
echo "Manual test:"
echo "  ./target/release/sql-cli test_cols.csv -e \"select * from data\""
echo ""
echo "Debug mode:"
echo "  RUST_LOG=sql_cli::ui::viewport_manager=debug ./target/release/sql-cli test_cols.csv -e \"select * from data\" 2>&1 | grep -E 'navigate_to_(first|last)_column'"