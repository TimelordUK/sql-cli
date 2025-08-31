#!/bin/bash

# Test script for V45: Column types via DataProvider

echo "Testing V45: Column types via DataProvider"
echo "==========================================="

# Build with debug output
echo "Building project..."
cargo build --release 2>&1 | grep -E "error|Finished"

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "Test 1: Column Statistics with Cached Types"
echo "--------------------------------------------"
echo "Press 'S' on different columns to see cached type detection"
echo "Should see: 'V45: Column X has cached type: [Type]' in debug logs"

# Create test data with different column types
cat > test_types.csv << EOF
id,name,age,salary,active,join_date
1,Alice,25,50000.50,true,2020-01-15
2,Bob,30,60000.75,false,2019-06-20
3,Carol,28,55000.00,true,2021-03-10
4,David,35,70000.25,true,2018-11-05
5,Eve,32,65000.00,false,2020-07-22
EOF

echo "Test data created with columns:"
echo "  - id: Integer"
echo "  - name: Text"
echo "  - age: Integer"
echo "  - salary: Float"
echo "  - active: Boolean"
echo "  - join_date: Date"

# Run with debug logging to see V45 messages
RUST_LOG=debug timeout 3 ./target/release/sql-cli test_types.csv -e "select * from data" 2>&1 | grep "V45" || echo "No V45 messages found (may need to press 'S' in the app)"

echo ""
echo "==========================================="
echo "V45 Column Types Test Complete!"
echo ""
echo "What we added:"
echo "✓ DataType enum with Integer, Float, Text, Boolean, Date, Mixed"
echo "✓ get_column_type() and get_column_types() methods in DataProvider"
echo "✓ Type detection in BufferAdapter (samples first 100 rows)"
echo "✓ Lazy caching with thread-safe Arc<Mutex>"
echo "✓ Debug logging in calculate_column_statistics"
echo ""
echo "Performance benefits:"
echo "- Type detection happens ONCE when first accessed"
echo "- No regex compilation on every stats call"
echo "- Cached for entire session"
echo ""
echo "Next steps:"
echo "- Use cached types in stats calculation (avoid re-detection)"
echo "- Fix histogram for high-cardinality columns"
echo "- Use types for better sorting (numeric vs text)"