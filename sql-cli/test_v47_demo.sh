#!/bin/bash
# Create a test CSV and instructions for testing V47

cat > test_v47.csv << 'EOF'
id,name,value,active,timestamp
1,Alice,100.5,true,2024-01-01
2,Bob,200.75,false,2024-01-02
3,Carol,300.25,true,2024-01-03
4,Dave,400.0,true,2024-01-04
5,Eve,500.5,false,2024-01-05
6,Frank,600.25,true,2024-01-06
7,Grace,700.5,false,2024-01-07
8,Henry,800.75,true,2024-01-08
9,Ivy,900.0,false,2024-01-09
10,Jack,1000.5,true,2024-01-10
EOF

echo "Test CSV created: test_v47.csv"
echo ""
echo "To test V47 DataTable storage:"
echo "1. Run: RUST_LOG=debug ./target/release/sql-cli test_v47.csv"
echo "2. Execute a query: select * from data"
echo "3. Press F6 to demo DataTable conversion"
echo "4. Check the status message for 'V47: DataTable stored!'"
echo "5. Check logs: tail -f ~/.local/share/sql-cli/logs/sql-cli_*.log | grep V47"
echo ""
echo "The DataTable should be automatically created and stored when results are set."