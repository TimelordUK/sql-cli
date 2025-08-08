#!/bin/bash

# Test script to verify history filtering with schema information

echo "Testing history filtering with schema information..."
echo ""

# Create a test CSV file with specific columns
cat > test_trades.csv << 'EOF'
symbol,price,quantity,timestamp
BTC,50000,1.5,2024-01-01
ETH,3000,10,2024-01-02
BTC,51000,2.0,2024-01-03
EOF

cat > test_cities.csv << 'EOF'
name,population,country
New York,8000000,USA
London,9000000,UK
Tokyo,14000000,Japan
EOF

echo "Created test CSV files:"
echo "- test_trades.csv (columns: symbol, price, quantity, timestamp)"
echo "- test_cities.csv (columns: name, population, country)"
echo ""

echo "To test the history filtering:"
echo "1. Run: ./target/release/sql-cli test_trades.csv"
echo "2. Execute some queries like:"
echo "   - SELECT * FROM trades WHERE symbol = 'BTC'"
echo "   - SELECT symbol, price FROM trades"
echo "3. Press Ctrl+R to open history search"
echo "4. Then switch to test_cities.csv in a new session"
echo "5. Press Ctrl+R - you should see history filtered for the current file's schema"
echo ""
echo "The history should prioritize queries that match the current file's columns."