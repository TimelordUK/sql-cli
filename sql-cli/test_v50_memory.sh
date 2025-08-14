#!/bin/bash

echo "V50 DataTable Memory & Performance Test"
echo "========================================"
echo ""
echo "This test validates the direct CSV-to-DataTable loading"
echo "and compares memory usage between JSON and direct modes."
echo ""

# Build in release mode for accurate memory measurements
echo "Building release version..."
cargo build --release 2>/dev/null || {
    echo "Build failed, trying debug mode..."
    cargo build || exit 1
    BINARY="./target/debug/sql-cli"
}
BINARY=${BINARY:-"./target/release/sql-cli"}

# Create test CSV with varying sizes
create_test_csv() {
    local rows=$1
    local file=$2
    echo "Creating $file with $rows rows..."
    python3 -c "
import csv
import random
import datetime

with open('$file', 'w', newline='') as f:
    writer = csv.writer(f)
    writer.writerow(['id', 'name', 'value', 'status', 'category', 'timestamp', 'description'])
    for i in range($rows):
        writer.writerow([
            i, 
            f'item_{i}', 
            round(random.random()*10000, 2),
            random.choice(['active', 'inactive', 'pending']),
            random.choice(['A', 'B', 'C', 'D', 'E']),
            (datetime.datetime(2024, 1, 1) + datetime.timedelta(days=i%365)).strftime('%Y-%m-%d'),
            f'Description for item {i} with some text'
        ])
"
}

# Test function
run_memory_test() {
    local mode=$1
    local file=$2
    local rows=$3
    
    echo ""
    echo "Testing $mode mode with $rows rows:"
    echo "-----------------------------------"
    
    if [ "$mode" = "direct" ]; then
        export DIRECT_DATATABLE=1
        echo "DIRECT_DATATABLE=1 (Direct CSV to DataTable)"
    else
        unset DIRECT_DATATABLE
        echo "JSON mode (Legacy path through QueryResponse)"
    fi
    
    # Run with memory tracking
    RUST_LOG=sql_cli=info timeout 10 $BINARY "$file" -e "SELECT * FROM data" 2>&1 | grep -E "(MEMORY\[|Direct CSV load|Memory History|Current Memory)" | tail -20
    
    # Also try with F5 to get memory history
    timeout 2 $BINARY "$file" <<EOF 2>&1 | grep -E "(Memory History|MEMORY\[|rows|columns|MB)" | tail -30
SELECT * FROM data
EOF
}

# Create test files if needed
echo ""
echo "Preparing test data..."
[ ! -f test_1k.csv ] && create_test_csv 1000 test_1k.csv
[ ! -f test_5k.csv ] && create_test_csv 5000 test_5k.csv
[ ! -f test_20k.csv ] && create_test_csv 20000 test_20k.csv

# Run comparison tests
echo ""
echo "======================================"
echo "MEMORY COMPARISON TESTS"
echo "======================================"

# Small dataset
echo ""
echo "TEST 1: Small dataset (1,000 rows)"
echo "======================================"
run_memory_test "json" "test_1k.csv" "1k"
run_memory_test "direct" "test_1k.csv" "1k"

# Medium dataset
echo ""
echo "TEST 2: Medium dataset (5,000 rows)"
echo "======================================"
run_memory_test "json" "test_5k.csv" "5k"
run_memory_test "direct" "test_5k.csv" "5k"

# Large dataset
echo ""
echo "TEST 3: Large dataset (20,000 rows)"
echo "======================================"
run_memory_test "json" "test_20k.csv" "20k"
run_memory_test "direct" "test_20k.csv" "20k"

echo ""
echo "======================================"
echo "SUMMARY"
echo "======================================"
echo "Direct DataTable loading (DIRECT_DATATABLE=1) should show:"
echo "- 80-90% memory reduction compared to JSON mode"
echo "- Near-instantaneous load times"
echo "- No QueryResponse allocations in memory tracking"
echo ""
echo "Expected memory usage for 20k rows:"
echo "- JSON mode: 300-700 MB (includes 3x QueryResponse clones)"
echo "- Direct mode: 100-150 MB (DataTable only)"
echo ""
echo "To enable direct mode permanently:"
echo "  export DIRECT_DATATABLE=1"
echo ""
echo "To see detailed memory tracking in app:"
echo "  Press F5 while running"