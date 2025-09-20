#!/bin/bash

# Test stats_examples.sql with the AAPL data
echo "Testing stats_examples.sql..."

# Change to examples directory to get correct relative paths
cd examples || exit 1

# Execute the SQL file
../target/release/sql-cli ../data/AAPL_data.csv < stats_examples.sql

echo "Test completed"