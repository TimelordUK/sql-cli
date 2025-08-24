#!/bin/bash

echo "Testing final fix for N key toggle after search..."
echo ""
echo "Creating test data..."

cat > test_final.csv << 'EOF'
id,name,type,status
1,Alice,derivatives,active
2,Bob,futures,pending
3,Charlie,derivatives,active
4,David,options,inactive
5,Eve,derivatives,pending
EOF

echo ""
echo "=== TEST INSTRUCTIONS ==="
echo "1. App will start with test data"
echo "2. Press '/' to enter search mode"
echo "3. Type 'derivatives' and press Enter"
echo "4. Press 'n' to navigate forward (should work)"
echo "5. Press 'N' to navigate backward (should work)"
echo "6. Press Escape to clear search"
echo "7. Press 'N' - should toggle line numbers (NOT navigate)"
echo ""
echo "Starting app..."

./target/release/sql-cli test_final.csv -e "select * from data"