#!/bin/bash

# Window Function Profiling Benchmark
# Tests LAG/LEAD with execution plan output to analyze performance

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Window Function Profiling Benchmark ===${NC}"
echo "Testing LAG/LEAD performance with execution plan analysis"
echo ""

SQL_CLI="./target/release/sql-cli"

# Check if binary exists
if [ ! -f "$SQL_CLI" ]; then
    echo -e "${RED}Error: $SQL_CLI not found. Run 'cargo build --release' first.${NC}"
    exit 1
fi

# Function to generate test data
generate_test_data() {
    local rows=$1
    local file=$2

    echo "id,region,sales_amount,product,date" > $file
    for ((i=1; i<=$rows; i++)); do
        region="Region_$(($i % 4))"  # 4 regions for partitioning
        amount=$((RANDOM % 50000 + 10000))
        product="Product_$(($i % 100))"
        day=$(($i % 365 + 1))
        echo "$i,$region,$amount,$product,2024-$(printf "%02d" $(($day / 30 + 1)))-$(printf "%02d" $(($day % 30 + 1)))" >> $file
    done
}

# Function to extract window timing from execution plan
extract_window_timing() {
    local output="$1"
    # Extract the window function evaluation time
    echo "$output" | grep -E "WINDOW.*window function" | grep -oP '\[[\d.]+ms\]' | tr -d '[]ms'
}

# Function to run query with execution plan and extract timing
run_with_plan() {
    local file=$1
    local query=$2
    local label=$3

    echo -e "${YELLOW}$label${NC}"

    # Run query with execution plan
    local output=$($SQL_CLI "$file" -q "$query" --execution-plan 2>&1)

    # Extract window timing
    local window_time=$(echo "$output" | grep "Evaluation time:" | grep -oP '[\d.]+ms' | head -1)

    # Extract total query time
    local total_time=$(echo "$output" | grep "Query completed:" | grep -oP '[\d.]+ms' | head -1)

    # Show the window function step
    echo "$output" | grep -A4 "WINDOW Evaluate"

    echo ""
    echo "  Window Function Time: ${window_time:-N/A}"
    echo "  Total Query Time: ${total_time:-N/A}"

    if [ -n "$window_time" ] && [ -n "$total_time" ]; then
        # Calculate percentage (rough approximation)
        local window_ms=$(echo "$window_time" | tr -d 'ms')
        local total_ms=$(echo "$total_time" | tr -d 'ms')
        local percent=$(awk "BEGIN {printf \"%.1f\", ($window_ms / $total_ms) * 100}")
        echo "  Window % of Total: ${percent}%"
    fi

    echo ""
}

# Test different data sizes
for rows in 10000 20000 50000; do
    echo -e "${GREEN}========================================"
    echo -e "Testing with $rows rows"
    echo -e "========================================${NC}"
    echo ""

    # Generate test data
    TEST_FILE="/tmp/window_benchmark_$rows.csv"
    echo -e "${YELLOW}Generating test data...${NC}"
    generate_test_data $rows $TEST_FILE
    echo "Generated $rows rows"
    echo ""

    # Test 1: Simple LAG function
    run_with_plan "$TEST_FILE" \
        "SELECT region, sales_amount, LAG(sales_amount) OVER (ORDER BY id) as prev_sales FROM window_benchmark LIMIT 1000" \
        "Test 1: LAG function (no partition)"

    # Test 2: LEAD function
    run_with_plan "$TEST_FILE" \
        "SELECT region, sales_amount, LEAD(sales_amount) OVER (ORDER BY id) as next_sales FROM window_benchmark LIMIT 1000" \
        "Test 2: LEAD function (no partition)"

    # Test 3: LAG + LEAD combined
    run_with_plan "$TEST_FILE" \
        "SELECT region, sales_amount, LAG(sales_amount) OVER (ORDER BY id) as prev_sales, LEAD(sales_amount) OVER (ORDER BY id) as next_sales FROM window_benchmark LIMIT 1000" \
        "Test 3: LAG + LEAD combined"

    # Test 4: LAG with PARTITION BY
    run_with_plan "$TEST_FILE" \
        "SELECT region, sales_amount, LAG(sales_amount) OVER (PARTITION BY region ORDER BY id) as prev_sales FROM window_benchmark LIMIT 1000" \
        "Test 4: LAG with PARTITION BY region"

    # Test 5: ROW_NUMBER
    run_with_plan "$TEST_FILE" \
        "SELECT region, sales_amount, ROW_NUMBER() OVER (ORDER BY sales_amount) as rank FROM window_benchmark LIMIT 1000" \
        "Test 5: ROW_NUMBER function"

    # Test 6: Multiple window functions with partitioning
    run_with_plan "$TEST_FILE" \
        "SELECT region, sales_amount, LAG(sales_amount) OVER (PARTITION BY region ORDER BY sales_amount) as prev, LEAD(sales_amount) OVER (PARTITION BY region ORDER BY sales_amount) as next, ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount) as rank FROM window_benchmark LIMIT 1000" \
        "Test 6: Multiple functions with PARTITION BY"

    # Clean up
    rm -f "$TEST_FILE"

    echo ""
done

echo -e "${GREEN}Benchmark complete!${NC}"
echo ""
echo -e "${BLUE}Summary:${NC}"
echo "The execution plan now shows:"
echo "  • Window function evaluation time"
echo "  • Number of window functions evaluated"
echo "  • Input/output row counts"
echo ""
echo "Use this data to identify bottlenecks in window function performance."
echo "Next steps: Add detailed timing in arithmetic_evaluator.rs and window_context.rs"
