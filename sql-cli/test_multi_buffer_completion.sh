#!/bin/bash

# Test multi-buffer completion fix
# This script tests that column completions work correctly when switching between buffers

echo "Testing multi-buffer completion fix..."
echo "======================================"
echo ""
echo "1. Start sql-cli with two CSV files:"
echo "   - trades_20k.csv (with PlatformOrderId column)"
echo "   - data/instruments.csv (different schema)"
echo ""
echo "2. In command mode, type 'SELECT plat' and press Tab"
echo "   - Should complete to 'PlatformOrderId' in buffer 1"
echo ""
echo "3. Switch to buffer 2 (Alt+2 or :b2)"
echo "   - Verify you're looking at instruments.csv data"
echo ""
echo "4. Switch back to buffer 1 (Alt+1 or :b1)"
echo ""
echo "5. In command mode, type 'SELECT plat' and press Tab again"
echo "   - Should still complete to 'PlatformOrderId' (not instrument columns)"
echo ""
echo "Running: ./target/release/sql-cli ../trades_20k.csv ../data/instruments.csv"
echo ""

# Run the command
./target/release/sql-cli ../trades_20k.csv ../data/instruments.csv