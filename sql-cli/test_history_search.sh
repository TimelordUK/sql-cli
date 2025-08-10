#!/bin/bash
# Test script for history search functionality

echo "Testing history search (Ctrl+R) functionality..."
echo "Run the application and:"
echo "1. Execute a few queries to build history:"
echo "   SELECT * FROM trades_100000"
echo "   SELECT * FROM trades_100000 WHERE price > 100"
echo "   SELECT * FROM trades_100000 ORDER BY quantity DESC"
echo ""
echo "2. Press Ctrl+R to start history search"
echo "3. Check that you see history entries"
echo "4. Type to filter history"
echo "5. Use Up/Down arrows to navigate"
echo "6. Press Enter to select or Esc to cancel"
echo ""
echo "Starting SQL CLI with test data..."

./target/release/sql-cli ../data/trades_100000.csv