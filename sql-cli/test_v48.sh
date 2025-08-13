#!/bin/bash
# Test V48: DataTable rendering

echo "Testing V48: DataTable rendering"
echo "================================="

# Create test CSV with various data types
cat > test_v48.csv << 'EOF'
id,name,value,active,date
1,Alice,100.5,true,2024-01-01
2,Bob,200.75,false,2024-01-02
3,Carol,300.25,true,2024-01-03
4,Dave,400.0,true,2024-01-04
5,Eve,500.5,false,2024-01-05
EOF

echo "Test CSV created with mixed data types"
echo ""
echo "To test V48:"
echo "1. Run: RUST_LOG=debug ./target/release/sql-cli test_v48.csv 2>&1 | grep V48"
echo "2. Execute: select * from data"
echo "3. Look for 'V48: Using DataTable' messages in the log"
echo "4. Verify data displays correctly"
echo ""

# Run in classic mode to see debug output
echo "Running query in classic mode..."
RUST_LOG=debug timeout 2 ./target/release/sql-cli test_v48.csv \
    -e "select * from data" \
    --classic 2>&1 | grep -E "V48:|V47:" | head -20

echo ""
echo "If you see 'V48: Using DataTable' messages, the rendering is using DataTable!"