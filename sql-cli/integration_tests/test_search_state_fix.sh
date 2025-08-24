#!/bin/bash

echo "Testing search state reset bug fix"
echo "=================================="
echo ""

# Create test data
cat > test_search_fix.csv << 'CSV'
id,name,status,amount
1,Alice,active,100
2,Bob,pending,200
3,Charlie,active,150
4,David,pending,300
5,Eve,active,250
CSV

echo "Test procedure:"
echo "1. Load file"  
echo "2. Press 'N' - should toggle line numbers (first time)"
echo "3. Press '/' and search for 'active'"
echo "4. Press 'n', 'N' to navigate search results"
echo "5. Press Escape to exit search mode"
echo "6. Press 'N' - should toggle line numbers again (not search)"
echo "7. Type new query: select * from test_search_fix where status = 'active'"
echo "8. Press Enter to run query"
echo "9. Press 'N' - should toggle line numbers (not search navigation)"
echo ""
echo "EXPECTED: After escape or running new query, 'N' should toggle line numbers"
echo "BUG WAS: 'N' stayed stuck trying to navigate old search results"
echo ""

# Start the application
echo "Starting application..."
./target/release/sql-cli test_search_fix.csv

# Clean up
rm -f test_search_fix.csv
echo "Test complete!"