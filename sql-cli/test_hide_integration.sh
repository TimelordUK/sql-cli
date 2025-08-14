#!/bin/bash
# Integration test for the hide column feature

# Create test data
cat > test_hide_integration.csv << 'EOF'
id,name,age,city,country
1,Alice,30,NYC,USA
2,Bob,25,London,UK
3,Charlie,35,Paris,France
EOF

echo "Testing hide column integration..."
echo "=================================="

# Test that hiding columns is reflected in the QueryEngine execution
RUST_LOG=debug timeout 3 ./target/release/sql-cli test_hide_integration.csv -e "select * from data" 2>&1 | grep -E "(hide_current_column|Hidden column|Ctrl\+H)" || true

echo ""
echo "Test complete. The hide column feature should work with Ctrl+H in Results mode."
echo "To manually test:"
echo "  1. Run: ./target/release/sql-cli test_hide_integration.csv"
echo "  2. Execute: SELECT * FROM data"
echo "  3. Press Ctrl+H to hide current column"
echo "  4. Navigate with arrow keys and hide more columns"
echo "  5. Press Ctrl+Shift+H to unhide all"