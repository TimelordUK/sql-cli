#!/bin/bash

echo "Testing buffer switching and tab completion..."

# Create test CSV files
echo "customer_id,name,country" > test_customers.csv
echo "1,Alice,USA" >> test_customers.csv
echo "2,Bob,Canada" >> test_customers.csv

echo "order_id,customer_id,amount" > test_orders.csv  
echo "101,1,99.99" >> test_orders.csv
echo "102,2,149.99" >> test_orders.csv

echo "Test files created. Now you can:"
echo "1. Run: cargo run --release --bin sql-cli -- test_customers.csv test_orders.csv"
echo "2. Switch between buffers with Alt+1 and Alt+2"
echo "3. Type 'select * from test_customers where ' and press Tab"
echo "4. You should see column completions (customer_id, name, country)"
echo "5. Switch to buffer 2 and type 'select * from test_orders where ' and press Tab"
echo "6. You should see different columns (order_id, customer_id, amount)"