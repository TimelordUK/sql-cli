#!/bin/bash

echo "=== SQL CLI Enhanced TUI Test ==="
echo ""
echo "This will test the csvlens-style enhanced TUI features."
echo ""
echo "API Server should be running on port 5193."
echo "Testing connection..."

# Test API connection
if curl -s http://localhost:5193/api/trade/sample > /dev/null 2>&1; then
    echo "✅ API server is responding"
else
    echo "❌ API server not responding. Start it with:"
    echo "   cd TradeApi && dotnet run"
    exit 1
fi

echo ""
echo "🚀 Starting Enhanced TUI..."
echo ""
echo "📖 Quick Guide:"
echo "  1. Type SQL query and press Enter"
echo "  2. Press ↓ to enter navigation mode"
echo "  3. Use j/k or ↓/↑ to navigate rows"
echo "  4. Press / to search, F to filter"
echo "  5. Press s to sort current column"
echo "  6. Press F1 or ? for comprehensive help"
echo "  7. Press q to quit from navigation mode"
echo ""
echo "📝 Try these queries:"
echo '  SELECT * FROM trade_deal'
echo '  SELECT * FROM trade_deal WHERE platformOrderId.Contains("200")'
echo '  SELECT DealId, Ticker, Price FROM trade_deal WHERE Price > 100'
echo ""
read -p "Press Enter to launch the Enhanced TUI..."

cd sql-cli
cargo run