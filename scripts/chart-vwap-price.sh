#!/bin/bash

# Chart VWAP Average Price Over Time (CLIENT orders only)
# This script visualizes how the VWAP average price evolves for client orders

# Navigate to project root
cd "$(dirname "$0")/.." || exit 1

# Check if binary exists
if [ ! -f "./target/release/sql-cli-chart" ]; then
    echo "Error: sql-cli-chart binary not found."
    echo "Please run 'cargo build --release' first."
    exit 1
fi

echo "=== VWAP Average Price Chart ==="
echo "Visualizing CLIENT order average price progression over time"
echo ""

./target/release/sql-cli-chart data/production_vwap_final.csv \
  -q "SELECT snapshot_time, average_price, filled_quantity FROM production_vwap_final WHERE order_type LIKE '%CLIENT%'" \
  -x snapshot_time \
  -y average_price \
  -t "CLIENT Order VWAP Price Over Time"

echo ""
echo "Chart closed."