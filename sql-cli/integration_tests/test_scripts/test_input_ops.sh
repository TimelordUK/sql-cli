#!/bin/bash
# Test script for InputBehavior trait implementation

echo "Testing input operations..."

# Create a test CSV file
cat > test_input.csv << EOF
id,name,age
1,Alice,30
2,Bob,25
3,Charlie,35
EOF

# Test that the app starts and accepts input
echo "Testing basic input operations..."
timeout 2 ./target/release/sql-cli test_input.csv -e "SELECT * FROM data" 2>&1 | head -20

echo ""
echo "Test completed. Check for any errors above."
echo "The InputBehavior trait methods should be working if no errors are shown."

# Clean up
rm -f test_input.csv