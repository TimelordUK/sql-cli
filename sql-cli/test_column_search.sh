#!/bin/bash

echo "Testing column search preserves command text..."
echo ""
echo "Test steps:"
echo "1. Run: cargo run --release --bin sql-cli -- test.json"
echo "2. Type a query: 'select * from trades_10k where symbol = \"AAPL\"'"
echo "3. Press Enter to view results"
echo "4. Press backslash (\\) to enter column search mode"
echo "5. Type a column name like 'price' or 'symbol'"
echo "6. Press Enter or Esc to exit column search"
echo "7. The original query should be restored in the command window"
echo ""
echo "The same should work with regular search (/) as well"