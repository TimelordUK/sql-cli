#!/bin/bash

# Test F5 dump with schema display

echo "Testing F5 dump with DataTable schema..."

# Create test data with various types
cat > test_schema.csv << EOF
name,age,salary,active,joined_date
Alice,25,50000.50,true,2023-01-15
Bob,30,75000.00,false,2022-06-20
Charlie,35,90000.75,true,2021-03-10
David,,60000.00,true,2023-11-05
Eve,28,,false,2022-09-18
EOF

# Test with debug output
echo "Running query and testing F5 dump..."
timeout 2 ./target/release/sql-cli test_schema.csv -e "select * from data where age > 20" 2>&1 | head -50

echo ""
echo "Test complete. The DataTable schema should be visible in F5 dump mode."
echo "In interactive mode, press F5 to see the schema information."

# Clean up
rm -f test_schema.csv