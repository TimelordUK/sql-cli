#!/bin/bash

# Test basic navigation
echo "Testing basic navigation keys..."

# Create test file
cat > test_nav.csv << 'EOF'
id,name,value
1,Alice,100
2,Bob,200
3,Charlie,300
4,Diana,400
5,Eve,500
EOF

# Test with debug output
RUST_LOG=debug timeout 2 ./target/debug/sql-cli test_nav.csv -e "select * from data" 2>&1 | grep -E "handle_results_input|next_row|previous_row|goto" | head -20

echo "Done"