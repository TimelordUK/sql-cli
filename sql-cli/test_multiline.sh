#!/bin/bash

# Test script to demonstrate multi-line mode with auto-formatting

echo "SQL CLI Multi-line Mode Demo"
echo "==========================="
echo ""
echo "This demo shows:"
echo "1. F3 toggles between single-line and multi-line modes"
echo "2. When switching to multi-line, queries are auto-formatted"
echo "3. The syntax preview shows the single-line version with colors"
echo "4. Vim mode is available in multi-line mode"
echo ""
echo "Example query to try:"
echo "SELECT id, name, department, salary FROM employees WHERE department = 'Engineering' AND salary > 100000 ORDER BY salary DESC"
echo ""
echo "Press F3 to see it formatted nicely!"
echo ""

# Run the enhanced TUI with a sample CSV
cargo run --bin sql-cli -- --enhanced