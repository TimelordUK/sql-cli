#!/bin/bash
# Interactive TUI test for FilterState migration

echo "=== TUI FilterState Test ==="
echo "This will start the TUI - you can test filter functionality:"
echo ""
echo "To test FilterState migration:"
echo "1. Press 'f' to enter Filter mode"  
echo "2. Type 'New York' and press Enter"
echo "3. Should show 2 matching rows (Alice and Charlie)"
echo "4. Press Escape to clear filter"
echo "5. Press 'q' to quit"
echo ""
echo "Expected debug logs to show:"
echo "- 'apply_filter called with pattern: New York'"
echo "- 'Filter applied: 2 rows matched out of 5'"
echo ""
echo "Starting TUI in 3 seconds..."
sleep 3

RUST_LOG=filter=debug ./target/release/sql-cli test_filter_migration.csv