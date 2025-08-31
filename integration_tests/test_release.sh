#!/bin/bash

# Test script for release binary

echo "Testing SQL CLI Release Binary..."
echo "================================"

BINARY="./target/release/sql-cli"

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo "Error: Release binary not found at $BINARY"
    echo "Run: cargo build --release"
    exit 1
fi

echo "Binary found at: $BINARY"
echo "Size: $(ls -lh $BINARY | awk '{print $5}')"
echo ""

# Test basic functionality
echo "1. Testing help..."
$BINARY --help > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "   ✓ Help command works"
else
    echo "   ✗ Help command failed"
fi

# Test cache mode with sample data
if [ -f "sample_trades.json" ]; then
    echo ""
    echo "2. Testing cache mode..."
    echo "   Run: SELECT id, price FROM trade_deal ORDER BY price"
    echo "   Expected: Results sorted by price (95.25, 150.5, 175.75, 200.0)"
    echo ""
    echo "   Press Ctrl+C to exit the test"
    $BINARY cache sample_trades.json --classic
else
    echo ""
    echo "2. Skipping cache mode test (sample_trades.json not found)"
fi

echo ""
echo "3. To install system-wide:"
echo "   sudo cp $BINARY /usr/local/bin/"
echo "   or"
echo "   cp $BINARY ~/.local/bin/"  # Make sure ~/.local/bin is in PATH

echo ""
echo "4. For distribution:"
echo "   - The binary at $BINARY is self-contained"
echo "   - You can copy it to any Linux system with same architecture"
echo "   - Consider using 'strip $BINARY' to reduce size further"
echo "   - Consider using 'upx --best $BINARY' for compression"