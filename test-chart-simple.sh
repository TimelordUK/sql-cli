#!/bin/bash

echo "=== Testing Chart Tool Data Processing ==="

echo "1. Testing CSV loading and basic data processing..."
echo "   File: data/production_vwap_final.csv"
echo "   Expected: Should load 3320 rows, 20 columns successfully"
echo ""

# Test basic data loading (will fail at TUI step but show data loading works)
echo "Running chart tool (will exit at TUI due to no TTY):"
./target/release/sql-cli-chart data/production_vwap_final.csv \
  -q "SELECT snapshot_time, average_price FROM production_vwap_final WHERE filled_quantity > 0" \
  -x snapshot_time \
  -y average_price \
  -t "VWAP Price Over Time"

echo ""
echo "=== Success Indicators ==="
echo "✅ Data loading works if you see: 'Loaded 3320 rows, 20 columns'"  
echo "✅ Chart processing works if you see: 'Starting chart TUI...'"
echo "✅ TTY error is expected in this environment"
echo ""
echo "=== Next Steps ==="
echo "To test interactively, run in a proper terminal:"
echo "./target/release/sql-cli-chart data/production_vwap_final.csv -q \"SELECT snapshot_time, average_price FROM production_vwap_final WHERE filled_quantity > 0\" -x snapshot_time -y average_price -t \"VWAP Price Over Time\""