#!/bin/bash

# Chart VWAP Fill Volume Progression (CLIENT orders only)
# This script visualizes how the filled quantity accumulates for client orders

# Navigate to project root
cd "$(dirname "$0")/.." || exit 1

# Check if binary exists
if [ ! -f "./target/release/sql-cli-chart" ]; then
    echo "Error: sql-cli-chart binary not found."
    echo "Please run 'cargo build --release' first."
    exit 1
fi

echo "=== VWAP Fill Volume Chart ==="
echo "Visualizing CLIENT order fill quantity progression"
echo "Expected: Smooth upward trend starting from 0"
echo ""

./target/release/sql-cli-chart data/production_vwap_final.csv \
  -q "SELECT snapshot_time, average_price, filled_quantity FROM production_vwap_final WHERE order_type LIKE '%CLIENT%'" \
  -x snapshot_time \
  -y filled_quantity \
  -t "CLIENT Order Fill Volume Progression"

echo ""
echo "Chart closed."