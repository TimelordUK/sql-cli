#!/bin/bash

echo "Testing vim search with 'deriv' pattern..."
echo ""
echo "Instructions:"
echo "1. Press '/' to start vim search"
echo "2. Type 'deriv' and press Enter"
echo "3. Press 'n' multiple times to navigate through matches"
echo "4. Watch the terminal output for detailed logs"
echo "5. Press 'q' to quit when done"
echo ""
echo "Starting application with vim_search logging enabled..."
echo ""

RUST_LOG=vim_search=info ./target/release/sql-cli integration_tests/test_data/trades_20k.csv