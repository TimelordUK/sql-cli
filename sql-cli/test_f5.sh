#!/bin/bash

echo "Testing F5 Debug Info..."
echo ""
echo "Creating test data..."
cat > test_debug.csv << EOF
id,name,value
1,Alice,100
2,Bob,200
3,Carol,300
EOF

echo "Instructions:"
echo "1. Press F5 to show debug info"
echo "2. You should see comprehensive debug dump including:"
echo "   - Buffer state"
echo "   - DataView state"
echo "   - Memory usage"
echo "   - Navigation timing"
echo "   - Key history"
echo "3. Press Esc or q to exit debug mode"
echo "4. Press q again to quit"
echo ""
echo "Starting SQL CLI..."

./target/debug/sql-cli test_debug.csv -e "select * from data"

rm -f test_debug.csv
echo "Test complete!"