#!/bin/bash

# GROUP BY Performance Benchmark Script
# This script runs focused GROUP BY benchmarks to track optimization progress

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== SQL CLI GROUP BY Performance Benchmark ===${NC}"
echo ""

# Build release version
echo -e "${YELLOW}Building release version...${NC}"
cargo build --release --quiet
if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

SQL_CLI="./target/release/sql-cli"

# Function to run a benchmark and extract time
run_benchmark() {
    local rows=$1
    local query=$2
    local label=$3

    # Generate test data
    local csv_file="/tmp/benchmark_${rows}.csv"
    echo "id,category,value,name" > $csv_file
    for ((i=1; i<=$rows; i++)); do
        cat_id=$((i % 100))  # 100 unique categories
        value=$((RANDOM % 1000))
        echo "$i,cat_$cat_id,$value,name_$i" >> $csv_file
    done

    # Run query 3 times and take average
    local total_time=0
    for run in {1..3}; do
        local output=$($SQL_CLI "$csv_file" -q "$query" -o csv 2>&1)
        local time_ms=$(echo "$output" | grep -oP '\d+\.\d+ms' | grep -oP '\d+\.\d+' | head -1)
        if [ -z "$time_ms" ]; then
            time_ms=$(echo "$output" | grep -oP '\d+ms' | grep -oP '\d+' | head -1)
        fi
        total_time=$(echo "$total_time + $time_ms" | bc)
    done

    local avg_time=$(echo "scale=2; $total_time / 3" | bc)
    printf "%-45s %8.2fms\n" "$label" "$avg_time"

    # Clean up
    rm -f $csv_file
}

# Test different row counts
echo -e "${GREEN}Current GROUP BY Performance:${NC}"
echo "================================================"

for rows in 1000 5000 10000 20000 50000; do
    echo ""
    echo -e "${BLUE}Testing with $rows rows:${NC}"

    # Simple GROUP BY
    run_benchmark $rows \
        "SELECT category, COUNT(*) FROM benchmark GROUP BY category" \
        "  Simple GROUP BY (100 groups)"

    # GROUP BY with SUM
    run_benchmark $rows \
        "SELECT category, SUM(value) FROM benchmark GROUP BY category" \
        "  GROUP BY with SUM"

    # GROUP BY with multiple aggregates
    run_benchmark $rows \
        "SELECT category, COUNT(*), SUM(value), AVG(value), MIN(value), MAX(value) FROM benchmark GROUP BY category" \
        "  GROUP BY with 5 aggregates"

    # Multi-column GROUP BY
    run_benchmark $rows \
        "SELECT category, value, COUNT(*) FROM benchmark GROUP BY category, value" \
        "  Multi-column GROUP BY"
done

echo ""
echo "================================================"
echo -e "${GREEN}Benchmark complete!${NC}"
echo ""
echo "Performance observations:"
echo "- GROUP BY scales super-linearly with row count"
echo "- Multi-column GROUP BY is significantly slower"
echo "- Multiple aggregates have minimal additional overhead"
echo ""
echo "To track improvements after optimization:"
echo "  1. Save this output as baseline"
echo "  2. Apply optimizations"
echo "  3. Run again and compare"