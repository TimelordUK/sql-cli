#!/bin/bash

# Test script to verify buffer switching updates parser context

echo "Testing buffer switching with parser context update..."

# Create test data files
cat > /tmp/test_trades.json << 'EOF'
[
  {"tradeId": 1, "symbol": "AAPL", "price": 150.00, "quantity": 100},
  {"tradeId": 2, "symbol": "GOOGL", "price": 2800.00, "quantity": 50}
]
EOF

cat > /tmp/test_customers.csv << 'EOF'
customerId,customerName,country,creditLimit
1,Acme Corp,USA,100000
2,Global Inc,UK,50000
EOF

echo "Created test files:"
echo "- /tmp/test_trades.json (columns: tradeId, symbol, price, quantity)"
echo "- /tmp/test_customers.csv (columns: customerId, customerName, country, creditLimit)"

echo ""
echo "To test buffer switching:"
echo "1. Run: sql-cli/target/release/sql-cli /tmp/test_trades.json /tmp/test_customers.csv"
echo "2. Press F5 to see debug info showing current buffer's columns"
echo "3. Press Ctrl+6 to switch buffers"
echo "4. Press F5 again to verify columns changed"
echo "5. Try typing 'SELECT * FROM ' and press Tab for completions"
echo ""
echo "Expected behavior:"
echo "- Buffer 1 should show trades columns (tradeId, symbol, price, quantity)"
echo "- Buffer 2 should show customers columns (customerId, customerName, country, creditLimit)"
echo "- Completions should match the current buffer's table and columns"