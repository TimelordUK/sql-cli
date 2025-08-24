#!/bin/bash
# Test the N key toggle fix after search mode

set -e

echo "Testing N key toggle fix after search mode..."

# Create test data
cat > test_n_key.csv << EOF
name,age,city
Alice,30,New York
Bob,25,London
Charlie,35,Paris
David,40,Tokyo
Eve,28,Berlin
EOF

echo "✅ Created test data"

# Build the project
echo "Building project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"

echo ""
echo "🎯 MANUAL TEST STEPS:"
echo "1. Run: ./target/release/sql-cli test_n_key.csv -e \"select * from data\""
echo "2. Press 'N' (should toggle line numbers) ✅"
echo "3. Press '/' to enter search mode"
echo "4. Type 'Alice' or any search term"
echo "5. Press Escape to exit search mode"
echo "6. Press 'N' again (should toggle line numbers, NOT search navigation) ✅"
echo ""
echo "If step 6 toggles line numbers instead of doing search navigation,"
echo "then the fix is working correctly!"
echo ""
echo "🐛 Before fix: N would navigate to 'next search match' even after Escape"
echo "✅ After fix: N should toggle line numbers after Escape"
echo ""

# Clean up
rm -f test_n_key.csv

echo "Ready to test manually!"