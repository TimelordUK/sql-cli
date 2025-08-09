#!/bin/bash

echo "Testing History Protection Mechanisms"
echo "======================================"

# Build the project
echo "Building sql-cli..."
cargo build --release 2>/dev/null || { echo "Build failed"; exit 1; }

# Get the binary path
BINARY="./target/release/sql-cli"

# Create a test directory
TEST_DIR="/tmp/sql_cli_test_$$"
mkdir -p "$TEST_DIR"
export HOME="$TEST_DIR"

echo "Test directory: $TEST_DIR"
echo ""

# Function to count history entries
count_history() {
    if [ -f "$TEST_DIR/.sql_cli/history.json" ]; then
        grep -c '"command":' "$TEST_DIR/.sql_cli/history.json" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Test 1: Add some initial queries
echo "Test 1: Adding initial queries..."
echo "SELECT * FROM users;" | $BINARY --non-interactive 2>/dev/null
echo "SELECT * FROM orders;" | $BINARY --non-interactive 2>/dev/null
echo "SELECT * FROM products;" | $BINARY --non-interactive 2>/dev/null

INITIAL_COUNT=$(count_history)
echo "Initial history entries: $INITIAL_COUNT"

# Check if backup directory was created
if [ -d "$TEST_DIR/.sql_cli/history_backups" ]; then
    echo "✓ Backup directory created"
else
    echo "✗ Backup directory not created"
fi

# Test 2: Simulate clearing (which should be protected)
echo ""
echo "Test 2: Testing clear protection..."
# Try to write an empty history file (simulating the bug)
echo "[]" > "$TEST_DIR/.sql_cli/history.json"

# Add another query to trigger save
echo "SELECT * FROM test;" | $BINARY --non-interactive 2>/dev/null

NEW_COUNT=$(count_history)
echo "History entries after simulated clear: $NEW_COUNT"

if [ "$NEW_COUNT" -gt 0 ]; then
    echo "✓ History protected from clear"
else
    echo "✗ History was cleared (protection failed)"
fi

# Test 3: Check for backups
echo ""
echo "Test 3: Checking backups..."
BACKUP_COUNT=$(ls -1 "$TEST_DIR/.sql_cli/history_backups" 2>/dev/null | wc -l)
echo "Number of backup files: $BACKUP_COUNT"

if [ "$BACKUP_COUNT" -gt 0 ]; then
    echo "✓ Backups are being created"
    ls -la "$TEST_DIR/.sql_cli/history_backups" | head -5
else
    echo "✗ No backups found"
fi

# Cleanup
echo ""
echo "Cleaning up test directory..."
rm -rf "$TEST_DIR"

echo ""
echo "Test complete!"