#!/bin/bash

# Test that 'n' key now works after search

echo "Testing 'n' navigation after search..."

# Create test data
cat > test_nav.csv << 'CSV'
id,book,product,status
1,Fixed Income,Corporate,active
2,Commodities,Energy,emerging
3,Equities,Tech,active
4,Forex,EUR/USD,pending
5,Derivatives,Options,active
6,Fixed Income,emerging,pending
7,Commodities,Gold,active
8,Fixed Income,Government,emerging
9,Equities,emerging,active
10,Bonds,Corporate,emerging
CSV

# Run with debug logging
echo "Running test (will timeout after 3 seconds)..."
RUST_LOG=vim_search=info,search=info timeout 3 ./target/release/sql-cli test_nav.csv -e "select * from data" 2>&1 | grep -E "(Syncing|vim_search|n' to work)" | head -20

echo ""
echo "Test complete - check if VimSearchManager received matches"

# Clean up
rm -f test_nav.csv
