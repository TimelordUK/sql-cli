#!/bin/bash

# Test pinning columns
echo "Testing column pinning..."

# Create test input: navigate to column 2 (book), pin it, then check visibility
cat > test_input.txt << 'EOF'
l
l
p
q
EOF

echo "Running test..."
RUST_LOG=debug timeout 2s ./target/release/sql-cli test_orders.csv < test_input.txt 2>&1 | grep -E "(viewport_manager|render|Pin|visible_indices|display_columns)" | tail -100

echo "Test complete"