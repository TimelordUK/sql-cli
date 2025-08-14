#!/bin/bash
# Test script for visual rendering of hidden columns

cat > test_hide_visual.csv << 'EOF'
id,name,age,city,country,email
1,Alice,30,NYC,USA,alice@example.com
2,Bob,25,London,UK,bob@example.com
3,Charlie,35,Paris,France,charlie@example.com
4,David,28,Berlin,Germany,david@example.com
5,Eve,32,Tokyo,Japan,eve@example.com
EOF

echo "Testing Visual Rendering of Hidden Columns"
echo "=========================================="
echo ""
echo "This test will verify that hidden columns disappear from the display."
echo ""
echo "Test Steps:"
echo "1. Run: SELECT * FROM data"
echo "2. Navigate to the 'age' column (3rd column)"
echo "3. Press '-' (minus) to hide it"
echo "4. Verify that 'age' column disappears from the display"
echo "5. Navigate to 'email' column and press '-' to hide it"
echo "6. Press '+' to unhide all columns"
echo ""
echo "Starting SQL-CLI with debug logging..."

RUST_LOG=sql_cli=debug,sql_cli::data::adapters::buffer_adapter=debug ./target/release/sql-cli test_hide_visual.csv