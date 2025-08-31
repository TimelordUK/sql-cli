#!/bin/bash

# Test script for multi-file buffer support

echo "Testing multi-file buffer support..."
echo ""
echo "Created test files:"
echo "  /tmp/test1.csv - Contains: id, name, value data"
echo "  /tmp/test2.csv - Contains: product, price, quantity data"
echo ""
echo "To test:"
echo "1. Run: cargo run /tmp/test1.csv /tmp/test2.csv"
echo "2. You should see both files loaded into separate buffers"
echo "3. Use Alt+Tab to switch between buffers"
echo "4. Each buffer should show a comment with the filename"
echo "5. Use Alt+B to list all buffers"
echo ""
echo "Key commands:"
echo "  Alt+Tab       - Next buffer"
echo "  Alt+Shift+Tab - Previous buffer"
echo "  Alt+N         - New empty buffer"
echo "  Alt+W         - Close current buffer"
echo "  Alt+B         - List all buffers"
echo ""
echo "Starting TUI with both files..."
cargo run /tmp/test1.csv /tmp/test2.csv