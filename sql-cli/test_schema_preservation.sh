#!/bin/bash

# Test that original schema is preserved after computed queries

echo "Creating test CSV with 10 columns..."
cat > /tmp/test_schema.csv << 'EOF'
id,val1,val2,val3,val4,val5,val6,val7,val8,val9
1,10,20,30,40,50,60,70,80,90
2,11,21,31,41,51,61,71,81,91
3,12,22,32,42,52,62,72,82,92
EOF

echo "Test file created with 10 columns"
echo ""
echo "Running test queries..."
echo "1. SELECT id, id*2 as id2 FROM test_schema (should show 2 columns)"
echo "2. SELECT * FROM test_schema (should show all 10 original columns)"
echo ""

# Create a test script that sends commands
cat > /tmp/test_commands.txt << 'EOF'
SELECT id, id * 2 as id2 FROM test_schema
SELECT * FROM test_schema
EOF

echo "Check the logs for QueryExecutionService and QueryOrchestrator messages:"
echo "tail -f ~/.local/share/sql-cli/logs/latest.log | grep -E 'QueryExecutionService:|QueryOrchestrator:'"