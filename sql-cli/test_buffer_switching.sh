#!/bin/bash

# Test buffer switching functionality
echo "Testing buffer switching functionality..."

# Create test CSV files
echo "name,age,city" > test_buffer1.csv
echo "Alice,30,NYC" >> test_buffer1.csv
echo "Bob,25,LA" >> test_buffer1.csv

echo "product,price,stock" > test_buffer2.csv
echo "Laptop,999,10" >> test_buffer2.csv
echo "Mouse,25,100" >> test_buffer2.csv

echo "id,value,status" > test_buffer3.csv
echo "1,100,active" >> test_buffer3.csv
echo "2,200,inactive" >> test_buffer3.csv

# Build the project
echo "Building project..."
cargo build --release 2>&1 | tail -5

echo ""
echo "Test Instructions:"
echo "1. Run: ./target/release/sql-cli test_buffer1.csv test_buffer2.csv test_buffer3.csv"
echo "2. Press F12 or Ctrl+PgDn to switch to next buffer"
echo "3. Press F11 or Ctrl+PgUp to switch to previous buffer"
echo "4. Press Ctrl+6 to quick switch between last two buffers"
echo "5. Press Alt+1, Alt+2, Alt+3 to switch to specific buffer"
echo ""
echo "Expected behavior:"
echo "- Ctrl+6 should toggle between current and previous buffer"
echo "- F11/F12 should cycle through all buffers"
echo "- Alt+[number] should jump to specific buffer"