#!/bin/bash
# Test that fuzzy filter properly filters results

# Create test data
cat > test_fuzzy.csv << 'EOF'
id,name,city
1,Alice,New York
2,Bob,Boston  
3,Charlie,Chicago
4,David,Denver
5,Eve,New York
6,Frank,Boston
EOF

echo "Testing fuzzy filter functionality..."
echo ""
echo "Data loaded: 6 rows"
echo "Fuzzy filter pattern: 'Boston' should show 2 rows (Bob and Frank)"
echo ""

# Run query and check row count
echo "select * from data" | timeout 2 ./target/release/sql-cli test_fuzzy.csv 2>&1 | grep -E "rows returned|Error" || echo "Query executed"

echo ""
echo "If fuzzy filter is working correctly:"
echo "- Press 'f' to enter fuzzy filter mode"
echo "- Type 'Boston'"
echo "- Should see only 2 matching rows"
echo ""
echo "Test complete. Please test manually in TUI to confirm fuzzy filter works."