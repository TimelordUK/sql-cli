#!/bin/bash

# SQL CLI Complete Test Suite Runner
# Runs both Rust and Python tests

set -e  # Exit on error

echo "========================================="
echo "SQL CLI Complete Test Suite"
echo "========================================="
echo ""

# Track overall success
all_passed=true

# Run Rust tests
echo "1. Running Rust tests..."
echo "-----------------------------------------"
if cargo test --quiet; then
    echo "✅ Rust tests passed"
else
    echo "❌ Rust tests failed"
    all_passed=false
fi
echo ""

# Run Python tests
echo "2. Running Python tests..."
echo "-----------------------------------------"
if ./run_python_tests.sh > /tmp/python_test_output.txt 2>&1; then
    tail -5 /tmp/python_test_output.txt
    echo "✅ Python tests passed"
else
    cat /tmp/python_test_output.txt
    echo "❌ Python tests failed"
    all_passed=false
fi
echo ""

# Summary
echo "========================================="
echo "Test Summary"
echo "========================================="
if [ "$all_passed" = true ]; then
    echo "✅ All tests passed successfully!"
    exit 0
else
    echo "❌ Some tests failed. Please review the output above."
    exit 1
fi