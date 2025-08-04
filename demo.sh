#!/bin/bash

echo "Starting Trade API and SQL CLI Demo"
echo "==================================="

# Start the API in background
echo "Starting C# Trade API on port 5000..."
cd TradeApi
dotnet run --urls "http://localhost:5000" &
API_PID=$!

# Wait for API to start
echo "Waiting for API to start..."
sleep 5

# Test API is running
echo "Testing API endpoint..."
curl -s http://localhost:5000/api/trade/sample | jq '.[0]' || echo "API test failed"

echo ""
echo "API is running. In another terminal, run:"
echo "  cd sql-cli"
echo "  TRADE_API_URL=http://localhost:5000 cargo run"
echo ""
echo "Example queries to try:"
echo "  SELECT * FROM trade_deal"
echo "  SELECT dealId, platformOrderId, price FROM trade_deal WHERE price > 100"
echo "  SELECT * FROM trade_deal WHERE ticker = 'AAPL'"
echo "  SELECT * FROM trade_deal WHERE counterparty.Contains('Goldman')"
echo "  SELECT * FROM trade_deal ORDER BY price DESC"
echo ""
echo "Press Ctrl+C to stop the API"

# Wait for interrupt
wait $API_PID