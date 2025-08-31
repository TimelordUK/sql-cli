#!/bin/bash

# Compare ALGO vs CLIENT orders
# This script shows different queries to compare order types

# Navigate to project root
cd "$(dirname "$0")/.." || exit 1

# Check if binary exists
if [ ! -f "./target/release/sql-cli-chart" ]; then
    echo "Error: sql-cli-chart binary not found."
    echo "Please run 'cargo build --release' first."
    exit 1
fi

echo "=== VWAP Order Type Comparison ==="
echo ""
echo "Choose which view to display:"
echo "1) CLIENT orders only (clean progression)"
echo "2) ALGO_PARENT orders (algorithm's parent orders)"
echo "3) ALGO_SLICE orders (algorithm's child slices)"
echo "4) All orders (Christmas tree pattern)"
echo ""
read -p "Enter choice (1-4): " choice

case $choice in
    1)
        QUERY="SELECT snapshot_time, filled_quantity FROM production_vwap_final WHERE order_type LIKE '%CLIENT%'"
        TITLE="CLIENT Orders - Fill Progression"
        ;;
    2)
        QUERY="SELECT snapshot_time, filled_quantity FROM production_vwap_final WHERE order_type = 'ALGO_PARENT'"
        TITLE="ALGO Parent Orders - Fill Progression"
        ;;
    3)
        QUERY="SELECT snapshot_time, filled_quantity FROM production_vwap_final WHERE order_type = 'ALGO_SLICE'"
        TITLE="ALGO Slice Orders - Fill Progression"
        ;;
    4)
        QUERY="SELECT snapshot_time, filled_quantity FROM production_vwap_final WHERE filled_quantity > 0"
        TITLE="All Orders - Christmas Tree Pattern"
        ;;
    *)
        echo "Invalid choice. Exiting."
        exit 1
        ;;
esac

echo ""
echo "Executing: $QUERY"
echo ""

./target/release/sql-cli-chart data/production_vwap_final.csv \
  -q "$QUERY" \
  -x snapshot_time \
  -y filled_quantity \
  -t "$TITLE"

echo ""
echo "Chart closed."