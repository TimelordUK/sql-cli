#!/bin/bash
# Test V49: Direct DataTable creation from CSV

echo "Testing V49: Direct DataTable creation from CSV"
echo "==============================================="

# Create test CSV
cat > test_v49.csv << 'EOF'
id,name,value,active,date
1,Alice,100.5,true,2024-01-01
2,Bob,200.75,false,2024-01-02
3,Carol,300.25,true,2024-01-03
EOF

echo "Test CSV created"
echo ""
echo "When you load this CSV, DataTable should be created directly from CsvApiClient"
echo "Look for 'V49: Setting DataTable directly' in the logs"
echo ""
echo "To test:"
echo "1. Run: RUST_LOG=debug ./target/release/sql-cli test_v49.csv"
echo "2. Check logs: tail -f ~/.local/share/sql-cli/logs/sql-cli_*.log | grep V49"
echo "3. Should see: 'V49: Converting CsvDataSource to DataTable'"
echo "4. Should see: 'V49: Setting DataTable directly from CsvApiClient'"
echo ""
echo "This means DataTable was created WITHOUT going through JSON conversion!"