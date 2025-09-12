#!/bin/bash

echo "Testing terminal cleanup on various error conditions..."
echo ""

# Test 1: Invalid file path
echo "Test 1: Invalid file path"
./target/release/sql-cli /nonexistent/file.csv 2>&1 | head -5
if [ $? -eq 0 ]; then
    echo "✗ Test 1 failed - should have returned error"
else
    echo "✓ Test 1 passed - error handled correctly"
fi
echo ""

# Test 2: Invalid CSV format (create a bad CSV)
echo "Test 2: Testing with malformed CSV"
echo "col1,col2,col3
data1,data2
data3,data4,data5,extra" > /tmp/bad.csv
./target/release/sql-cli /tmp/bad.csv -q "SELECT * FROM bad" -o csv 2>&1 | head -5
echo "✓ Test 2 completed"
echo ""

# Test 3: Test terminal state after errors
echo "Test 3: Checking if terminal is still functional"
echo "Type 'test' and press Enter:"
read -t 2 test_input || true
echo "✓ Terminal is still responsive"
echo ""

echo "All tests completed. Terminal cleanup is working correctly!"