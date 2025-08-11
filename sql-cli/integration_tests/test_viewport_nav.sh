#!/bin/bash

# Test script for viewport navigation (H/M/L) behavior
echo "Testing viewport navigation commands (H/M/L)..."
echo ""
echo "This test will verify the new viewport navigation commands:"
echo "- H: Jump to top of viewport"
echo "- M: Jump to middle of viewport"
echo "- L: Jump to bottom of viewport"
echo ""
echo "These are different from:"
echo "- gg (or g): Jump to first row of data"
echo "- G: Jump to last row of data"
echo ""
echo "Starting sql-cli with navigation logging..."
echo "Try scrolling down to middle of data, then:"
echo "1. Press 'H' to jump to top of current viewport"
echo "2. Press 'M' to jump to middle of current viewport"
echo "3. Press 'L' to jump to bottom of current viewport"
echo "4. Press F5 to see viewport details in debug dump"
echo ""

# Run with navigation logging enabled
RUST_LOG=navigation=info ./target/release/sql-cli demos/trades_10k.csv