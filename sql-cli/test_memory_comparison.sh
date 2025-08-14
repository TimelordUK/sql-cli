#!/bin/bash

echo "Memory Usage Comparison: JSON vs Direct DataTable"
echo "=================================================="

# Use existing test file
if [ ! -f test_20k.csv ]; then
    echo "Please run with an existing CSV file"
    exit 1
fi

echo ""
echo "Starting memory: $(ps aux | grep sql-cli | grep -v grep | awk '{print $6}') KB"

echo ""
echo "1. Legacy JSON mode:"
echo "--------------------"
echo "Run: ./target/debug/sql-cli test_20k.csv"
echo "Then execute: SELECT * FROM test_20k"
echo "Press F5 to see memory usage"
echo ""

echo "2. Direct DataTable mode:"
echo "-------------------------"
echo "Run: DIRECT_DATATABLE=1 ./target/debug/sql-cli test_20k.csv"
echo "Then execute: SELECT * FROM test_20k"
echo "Press F5 to see memory usage"
echo ""

echo "Expected results:"
echo "- JSON mode: ~300-500 MB for 20k rows"
echo "- Direct mode: ~100-200 MB for 20k rows (no JSON overhead)"