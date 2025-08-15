#!/bin/bash

echo "Testing basic SQL execution with editing..."

# Test 1: Basic query execution
echo "Test 1: Basic SELECT query"
./target/release/sql-cli test_edit.csv -e "SELECT * FROM data" --classic 2>&1 | grep -q "Alice" && echo "✓ Basic query works" || echo "✗ Basic query failed"

# Test 2: WHERE clause
echo "Test 2: WHERE clause"
./target/release/sql-cli test_edit.csv -e "SELECT * FROM data WHERE id = 2" --classic 2>&1 | grep -q "Bob" && echo "✓ WHERE clause works" || echo "✗ WHERE clause failed"

# Test 3: Column selection
echo "Test 3: Column selection"
./target/release/sql-cli test_edit.csv -e "SELECT name FROM data" --classic 2>&1 | grep -q "Charlie" && echo "✓ Column selection works" || echo "✗ Column selection failed"

# Test 4: Check row count
echo "Test 4: Row count"
./target/release/sql-cli test_edit.csv -e "SELECT * FROM data" --classic 2>&1 | grep -q "3 rows" && echo "✓ Row count correct" || echo "✗ Row count failed"

echo ""
echo "Basic tests complete. For interactive testing, run:"
echo "./target/release/sql-cli test_edit.csv"
echo ""
echo "Key editing commands to test interactively:"
echo "- Type text to insert"
echo "- Backspace to delete"
echo "- Ctrl+A/E for home/end"
echo "- Ctrl+W to delete word"
echo "- Ctrl+U to clear line"
echo "- Ctrl+Z to undo"
echo "- F2 to switch modes"
echo "- 'i' to return to command mode (vim-style)"
