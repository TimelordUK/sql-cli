#!/bin/bash

# Test script to verify numeric sorting works correctly

# Create a test CSV with numeric values
cat > /tmp/test_numeric_sort.csv << 'EOF'
name,value,price
Item1,100,1.5
Item2,20,10.99
Item3,3,100.00
Item4,1000,0.99
Item5,5,50.50
Item6,0.5,1000.0
Item7,0.01,0.01
Item8,999999,999999.99
EOF

echo "Created test CSV with numeric values:"
cat /tmp/test_numeric_sort.csv
echo ""

# Build the application
echo "Building application..."
cargo build --release

# Run the application and test sorting
echo "Testing sorting on 'value' column (should sort numerically):"
echo "Expected order: 0.01, 0.5, 3, 5, 20, 100, 1000, 999999"
echo ""

# Use the application to view the data
./target/release/sql-cli /tmp/test_numeric_sort.csv

echo ""
echo "Instructions:"
echo "1. Press 's' to sort the 'value' column"
echo "2. Use arrow keys to move to the 'value' column first"
echo "3. Check if values are sorted numerically (0.01 < 0.5 < 3 < 5 < 20 < 100 < 1000 < 999999)"
echo "4. Also test sorting the 'price' column"
echo "5. Press 'q' to quit"