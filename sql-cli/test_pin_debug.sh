#!/bin/bash

echo "Testing pin columns with debug output..."

# Test with a simpler approach - just send 'p' key and look for any response
echo -e "llp\nq" | RUST_LOG=debug timeout 2 ./target/release/sql-cli test_pin_columns.csv -e "select * from data" 2>&1 | grep -E "(pin|Pin|toggle|Toggle|column action|handle_column)" | head -20

echo ""
echo "Checking viewport manager for pinned columns handling..."
grep -n "pinned" src/ui/viewport_manager.rs | head -10