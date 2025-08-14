#!/bin/bash

echo "Testing Direct DataTable Loading vs JSON Loading"
echo "================================================"

# Create test CSV if it doesn't exist
if [ ! -f test_20k.csv ]; then
    echo "Creating test_20k.csv..."
    python3 -c "
import csv
import random
with open('test_20k.csv', 'w', newline='') as f:
    writer = csv.writer(f)
    writer.writerow(['id', 'name', 'value', 'status', 'timestamp'])
    for i in range(20000):
        writer.writerow([i, f'item_{i}', random.random()*1000, 
                        random.choice(['active', 'inactive']), 
                        f'2024-01-{(i%28)+1:02d}'])
"
fi

echo ""
echo "1. Testing with JSON intermediate (legacy mode):"
echo "-------------------------------------------------"
unset DIRECT_DATATABLE
timeout 5 ./target/debug/sql-cli test_20k.csv <<EOF
SELECT * FROM test_20k
:exit
EOF

echo ""
echo "2. Testing with DIRECT DataTable (no JSON):"
echo "--------------------------------------------"
export DIRECT_DATATABLE=1
timeout 5 ./target/debug/sql-cli test_20k.csv <<EOF
SELECT * FROM test_20k
:exit
EOF

echo ""
echo "Memory comparison will be visible in F5 debug output"