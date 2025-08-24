#!/bin/bash

echo "=== Testing Simple Escape Fix ==="
echo ""
echo "Creating test data..."

cat > test_escape.csv << 'EOF'
id,name,type,status
1,Alice,derivatives,active
2,Bob,futures,pending
3,Charlie,derivatives,active
4,David,options,inactive
5,Eve,derivatives,pending
EOF

echo ""
echo "TEST INSTRUCTIONS:"
echo "1. Press '/' to enter search mode"
echo "2. Type 'derivatives' and press Enter"
echo "3. Press 'n' to navigate forward (should work)"
echo "4. Press 'N' to navigate backward (should work)"
echo "5. Press Escape to clear search"
echo "6. Press 'N' - should toggle line numbers (NOT navigate)"
echo ""
echo "Starting application..."
echo ""

./target/release/sql-cli test_escape.csv -e "select * from data"