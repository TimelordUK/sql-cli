#!/bin/bash

# Test the new architecture with DataLoaderService

echo "Testing new architecture..."

# Create a test CSV file
cat > test_architecture.csv << EOF
id,name,value
1,Alice,100
2,Bob,200
3,Charlie,300
EOF

# Run the application
timeout 2 ./target/release/sql-cli test_architecture.csv -e "select * from data" --classic 2>&1 | head -20

# Clean up
rm -f test_architecture.csv

echo "Test completed"