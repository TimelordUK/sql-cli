#!/bin/bash

echo "Testing search state reset in CLASSIC mode"
echo "=========================================="

# Create test data
cat > test_search_fix.csv << 'CSV'
id,name,status,amount
1,Alice,active,100
2,Bob,pending,200
3,Charlie,active,150
4,David,pending,300
5,Eve,active,250
CSV

echo "Test: Running with --classic flag"
echo "Note: In classic mode, we can't test interactive key behavior"
echo "But we can verify the application loads and runs queries correctly"
echo ""

./target/release/sql-cli test_search_fix.csv -e "select * from test_search_fix where status = 'active'" --classic

rm -f test_search_fix.csv
echo "Classic mode test complete!"