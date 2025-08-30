#!/bin/bash

# Test smart function completion
echo "Testing smart function completion..."

# Create a test CSV file
cat > test_data.csv << EOF
allocationStatus,comment,price
active,This is a test,100
pending,Another test,200
completed,Final test,300
EOF

# Test the completion with the updated binary
echo "Starting SQL CLI with test data..."
echo ""
echo "Test cases:"
echo "1. Type: WHERE allocationStatus.Len[TAB]"
echo "   Expected: WHERE allocationStatus.Length() with cursor after )"
echo ""
echo "2. Type: WHERE comment.Cont[TAB]"
echo "   Expected: WHERE comment.Contains('') with cursor between quotes"
echo ""
echo "3. Type: WHERE allocationStatus.ToLow[TAB]"
echo "   Expected: WHERE allocationStatus.ToLower() with cursor after )"
echo ""

./target/release/sql-cli test_data.csv

# Cleanup
rm -f test_data.csv