#!/bin/bash

echo "Testing Tab key mode switching..."
echo ""
echo "Creating test data..."
cat > test_tab.csv << EOF
id,name,value
1,Alice,100
2,Bob,200
3,Carol,300
EOF

echo "Instructions:"
echo "1. Query will auto-execute showing results in Results mode"
echo "2. Press Tab → Should switch to Command mode"
echo "3. Press Tab again → Should switch back to Results mode"
echo "4. In Results: arrows/hjkl navigate data"
echo "5. In Command: type SQL queries"
echo "6. Press 'q' in Results mode to quit"
echo ""
echo "Starting SQL CLI..."

./target/debug/sql-cli test_tab.csv -e "select * from data"

rm -f test_tab.csv
echo "Test complete!"