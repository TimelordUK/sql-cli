#!/bin/bash

echo "Testing column search fix..."
echo

# Create test CSV
cat > test_columns.csv << 'EOF'
id,name,orderid,description,amount,status
1,Widget A,ORD001,First widget,100.50,active
2,Widget B,ORD002,Second widget,200.75,active
3,Gadget X,ORD003,Cool gadget,300.25,pending
4,Tool Y,ORD004,Useful tool,400.00,complete
5,Device Z,ORD005,Smart device,500.99,active
EOF

echo "Created test data with columns: id, name, orderid, description, amount, status"
echo
echo "To test column search:"
echo "1. Run: ./target/release/sql-cli test_columns.csv"
echo "2. Press Enter to go to Results mode"
echo "3. Press '\' to enter column search mode"
echo "4. Type 'order' - should find and focus on 'orderid' column (column 3)"
echo "5. Type 'amo' - should find and focus on 'amount' column (column 5)"
echo
echo "The cursor should move to the matched column when typing."
echo
echo "Debug: Check logs with:"
echo "tail -f ~/.local/share/sql-cli/logs/sql-cli_*.log | grep -E 'search|column'"