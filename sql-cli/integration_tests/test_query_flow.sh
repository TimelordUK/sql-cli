#!/bin/bash

# Test script to verify SQL query preservation through mode transitions

echo "Testing SQL query preservation through mode transitions..."

# Create a test CSV file
cat > test_data.csv << EOF
id,name,value
1,Alice,100
2,Bob,200
3,Charlie,300
EOF

echo "1. Created test_data.csv"

# Test the flow
echo "2. Testing flow: Command -> Execute -> Results -> Fuzzy Filter -> Back to Results"
echo ""
echo "Expected behavior:"
echo "  - Type SQL query in Command mode"
echo "  - Execute query (switches to Results mode)"
echo "  - Press 'f' to enter Fuzzy Filter mode"
echo "  - Type filter pattern"
echo "  - Press Enter to apply filter and return to Results"
echo "  - Press 'c' to return to Command mode"
echo "  - The original SQL query should be restored"
echo ""
echo "Run the following test:"
echo "  ./target/release/sql-cli test_data.csv"
echo ""
echo "Then:"
echo "  1. Execute: SELECT * FROM test_data"
echo "  2. Press 'f' for fuzzy filter"
echo "  3. Type 'Alice' and press Enter"
echo "  4. Press 'c' to return to Command mode"
echo "  5. Verify the SQL 'SELECT * FROM test_data' is restored"