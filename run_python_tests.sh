#!/bin/bash

# SQL CLI Python Test Runner
# This script runs all Python tests for the SQL CLI project

set -e  # Exit on error

echo "========================================="
echo "SQL CLI Python Test Suite"
echo "========================================="
echo ""

# Check if uv is installed
if ! command -v uv &> /dev/null; then
    echo "Error: uv is not installed. Please install uv first."
    exit 1
fi

# Build the SQL CLI if not already built
if [ ! -f "target/release/sql-cli" ]; then
    echo "Building SQL CLI in release mode..."
    cargo build --release
    echo "Build complete."
    echo ""
fi

# Generate test data if needed
if [ ! -f "data/test_simple_math.csv" ] || [ ! -f "data/test_simple_strings.csv" ]; then
    echo "Generating test data..."
    uv run python scripts/generate_simple_test.py
    echo "Test data generated."
    echo ""
fi

# Run the tests
echo "Running Python tests..."
echo "-----------------------------------------"

# Run with verbose output and show summary
uv run pytest tests/test_sql_engine_pytest.py tests/test_string_methods_comprehensive.py -v

# Exit code from pytest
exit_code=$?

echo ""
echo "========================================="
if [ $exit_code -eq 0 ]; then
    echo "✅ All Python tests passed!"
else
    echo "❌ Some tests failed. Please review the output above."
fi
echo "========================================="

exit $exit_code