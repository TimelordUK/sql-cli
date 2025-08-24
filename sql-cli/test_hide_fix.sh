#!/bin/bash

echo "Testing column hide fix..."
echo ""
echo "Instructions:"
echo "1. Navigate to any column with h/l keys"
echo "2. Press '-' to hide the current column"
echo "3. Should NOT panic - column should be hidden"
echo "4. Press Ctrl+Shift+H to unhide all columns"
echo "5. Press 'q' to quit"
echo ""
echo "Starting application..."

./target/release/sql-cli test_hide_column.csv -e "select * from data"