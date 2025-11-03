#!/bin/bash
# Window Function Performance Benchmark Script
#
# This script measures window function performance across different dataset sizes
# and captures baseline metrics before implementing batch evaluation.
#
# Usage:
#   ./benchmark_window_functions.sh                    # Run all benchmarks
#   ./benchmark_window_functions.sh --quick            # Run 10k only (fast)
#   ./benchmark_window_functions.sh --capture          # Save baseline to file

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BINARY="$PROJECT_ROOT/target/release/sql-cli"
DATA_DIR="$PROJECT_ROOT/data"
RESULTS_DIR="$PROJECT_ROOT/docs/benchmarks"

# Ensure binary is built
if [ ! -f "$BINARY" ]; then
    echo "Building sql-cli in release mode..."
    cd "$PROJECT_ROOT"
    cargo build --release
fi

# Create results directory
mkdir -p "$RESULTS_DIR"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
QUICK_MODE=false
CAPTURE_MODE=false
for arg in "$@"; do
    case $arg in
        --quick)
            QUICK_MODE=true
            ;;
        --capture)
            CAPTURE_MODE=true
            ;;
    esac
done

# Function to generate test data
generate_test_data() {
    local rows=$1
    local output_file=$2

    echo -e "${BLUE}Generating $rows rows of test data...${NC}"

    # Header
    echo "id,value,category,amount" > "$output_file"

    # Generate rows
    for ((i=1; i<=rows; i++)); do
        local category=$((i % 5))  # 5 categories (0-4)
        local amount=$((RANDOM % 1000 + 1))
        echo "$i,$amount,cat_$category,$amount" >> "$output_file"
    done

    echo -e "${GREEN}✓ Generated $output_file${NC}"
}

# Function to run a timed query
run_benchmark() {
    local name=$1
    local data_file=$2
    local query=$3
    local rows=$4

    echo -e "${YELLOW}Running: $name (${rows} rows)${NC}" >&2

    # Run query 3 times and take the best time (to reduce variance)
    local best_time=999999
    local total_time=0
    local runs=3

    for ((i=1; i<=runs; i++)); do
        # Measure time in milliseconds
        local start=$(date +%s%3N)
        "$BINARY" "$data_file" -q "$query" -o csv > /dev/null 2>&1
        local end=$(date +%s%3N)
        local elapsed=$((end - start))

        total_time=$((total_time + elapsed))

        if [ $elapsed -lt $best_time ]; then
            best_time=$elapsed
        fi

        echo "  Run $i: ${elapsed}ms" >&2
    done

    local avg_time=$((total_time / runs))

    echo -e "${GREEN}  Best: ${best_time}ms, Average: ${avg_time}ms${NC}" >&2
    echo "" >&2

    # Return best time for capture mode (stdout only)
    echo "$best_time"
}

# Benchmark suite
run_benchmark_suite() {
    local rows=$1
    local data_file="$DATA_DIR/bench_window_${rows}.csv"

    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}Benchmark Suite: ${rows} rows${NC}"
    echo -e "${BLUE}================================${NC}"
    echo ""

    # Generate data if needed
    if [ ! -f "$data_file" ]; then
        generate_test_data $rows "$data_file"
    else
        echo -e "${GREEN}Using existing data file: $data_file${NC}"
    fi

    # Array to store results
    declare -A results

    # Benchmark 1: LAG without PARTITION BY
    results[lag_no_partition]=$(run_benchmark \
        "LAG without PARTITION BY" \
        "$data_file" \
        "SELECT id, value, LAG(value) OVER (ORDER BY id) as prev_value FROM bench_window_${rows}" \
        "$rows")

    # Benchmark 2: LAG with PARTITION BY
    results[lag_with_partition]=$(run_benchmark \
        "LAG with PARTITION BY" \
        "$data_file" \
        "SELECT id, category, value, LAG(value) OVER (PARTITION BY category ORDER BY id) as prev_value FROM bench_window_${rows}" \
        "$rows")

    # Benchmark 3: ROW_NUMBER without PARTITION BY
    results[row_number_no_partition]=$(run_benchmark \
        "ROW_NUMBER without PARTITION BY" \
        "$data_file" \
        "SELECT id, value, ROW_NUMBER() OVER (ORDER BY id) as row_num FROM bench_window_${rows}" \
        "$rows")

    # Benchmark 4: ROW_NUMBER with PARTITION BY
    results[row_number_with_partition]=$(run_benchmark \
        "ROW_NUMBER with PARTITION BY" \
        "$data_file" \
        "SELECT id, category, value, ROW_NUMBER() OVER (PARTITION BY category ORDER BY id) as row_num FROM bench_window_${rows}" \
        "$rows")

    # Benchmark 5: LEAD without PARTITION BY
    results[lead_no_partition]=$(run_benchmark \
        "LEAD without PARTITION BY" \
        "$data_file" \
        "SELECT id, value, LEAD(value) OVER (ORDER BY id) as next_value FROM bench_window_${rows}" \
        "$rows")

    # Benchmark 6: Multiple window functions (LAG, LEAD, ROW_NUMBER)
    results[multiple_functions]=$(run_benchmark \
        "Multiple window functions" \
        "$data_file" \
        "SELECT id, category, value, LAG(value) OVER (PARTITION BY category ORDER BY id) as lag_val, LEAD(value) OVER (PARTITION BY category ORDER BY id) as lead_val, ROW_NUMBER() OVER (PARTITION BY category ORDER BY id) as row_num FROM bench_window_${rows}" \
        "$rows")

    # Print summary
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}Summary for ${rows} rows:${NC}"
    echo -e "${BLUE}================================${NC}"
    printf "%-35s %10s\n" "Benchmark" "Time (ms)"
    echo "------------------------------------------------"
    for key in "${!results[@]}"; do
        printf "%-35s %10d\n" "$key" "${results[$key]}"
    done
    echo ""

    # Save results if in capture mode
    if [ "$CAPTURE_MODE" = true ]; then
        local results_file="$RESULTS_DIR/baseline_${rows}_rows.txt"
        echo "Window Function Baseline - ${rows} rows" > "$results_file"
        echo "Date: $(date)" >> "$results_file"
        echo "Binary: $BINARY" >> "$results_file"
        echo "" >> "$results_file"
        for key in "${!results[@]}"; do
            echo "$key: ${results[$key]}ms" >> "$results_file"
        done
        echo -e "${GREEN}✓ Saved baseline to $results_file${NC}"
    fi
}

# Main execution
echo -e "${GREEN}=====================================${NC}"
echo -e "${GREEN}Window Function Performance Benchmark${NC}"
echo -e "${GREEN}=====================================${NC}"
echo ""

if [ "$CAPTURE_MODE" = true ]; then
    echo -e "${YELLOW}Running in CAPTURE mode - will save baseline metrics${NC}"
    echo ""
fi

# Run benchmarks based on mode
if [ "$QUICK_MODE" = true ]; then
    run_benchmark_suite 10000
else
    run_benchmark_suite 10000
    run_benchmark_suite 50000
    run_benchmark_suite 100000
fi

echo -e "${GREEN}=====================================${NC}"
echo -e "${GREEN}Benchmark complete!${NC}"
echo -e "${GREEN}=====================================${NC}"

if [ "$CAPTURE_MODE" = true ]; then
    echo ""
    echo -e "${YELLOW}Baseline metrics saved to: $RESULTS_DIR${NC}"
    echo "Use these as reference when testing batch evaluation"
fi
