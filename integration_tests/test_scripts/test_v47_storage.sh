#!/bin/bash
# Test V47: DataTable storage alongside JSON

echo "Testing V47: DataTable storage alongside JSON"
echo "=============================================="

# Create test CSV
cat > test_v47.csv << 'EOF'
id,name,value,active
1,Alice,100.5,true
2,Bob,200.75,false
3,Carol,300.25,true
4,Dave,400.0,true
5,Eve,500.5,false
EOF

# Run query and capture debug output
echo "Running query to populate DataTable..."
RUST_LOG=debug timeout 2 ./target/release/sql-cli test_v47.csv \
    -e "select * from data where active = 'true'" \
    --classic 2>&1 | tee test_v47_output.log | grep -E "V47|Converting.*DataTable|Stored DataTable"

echo ""
echo "Checking log for V47 messages..."
grep -E "V47:|DataTable" test_v47_output.log | head -10

# Clean up
rm -f test_v47.csv test_v47_output.log

echo ""
echo "Test completed"