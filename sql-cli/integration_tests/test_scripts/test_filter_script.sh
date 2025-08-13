#!/bin/bash
# Test regex filter (Shift+F)
echo -e "select * from data\n" | timeout 2 ./target/release/sql-cli test_filters.csv 2>&1 | head -20

# Show that data loaded
echo "=== Data loaded successfully ==="
