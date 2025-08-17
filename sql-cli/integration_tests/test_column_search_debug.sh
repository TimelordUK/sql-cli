#!/bin/bash

# Test column search with debug output
echo "Testing column search with hidden columns..."

# Create simple test data
cat > test_search_debug.csv << 'EOF'
a,b,c,d,e,order1,order2,f,g
1,2,3,4,5,6,7,8,9
EOF

echo "Test data columns: a,b,c,d,e,order1,order2,f,g"
echo ""
echo "Running with debug logging to see column search behavior..."
echo "Commands:"
echo "  1. Hide column 'd' (index 3)"
echo "  2. Search for 'order'"
echo ""
echo "Expected: Should find 'order1' at visual position 4 (was 5 before hiding)"
echo "          This should map to DataTable index 5"
echo ""

RUST_LOG=column_search=debug ./target/release/sql-cli test_search_debug.csv