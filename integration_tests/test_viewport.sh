#!/bin/bash

# Test script for viewport scrolling behavior
echo "Testing viewport scrolling with NavigationState..."
echo ""
echo "This test will:"
echo "1. Run sql-cli with navigation logging enabled"
echo "2. Load sample data (trades_10k.csv)"
echo "3. Monitor viewport behavior"
echo ""
echo "Key actions to test:"
echo "- Press 'j' to move down (should scroll at bottom of viewport)"
echo "- Press 'k' to move up (should scroll at top of viewport)"
echo "- Press 'G' to jump to last row"
echo "- Press 'gg' to jump to first row"
echo "- Press F5 to see viewport details in debug dump"
echo ""
echo "Starting sql-cli with navigation logging..."
echo ""

# Run with navigation logging enabled
RUST_LOG=navigation=info,buffer=info ./target/release/sql-cli demos/trades_10k.csv