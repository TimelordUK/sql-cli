#!/bin/bash

echo "Testing sort functionality..."

# Create test data
cat > test_sort_data.csv << EOF
name,age,score
Alice,25,85
Bob,30,92
Charlie,22,78
EOF

echo "Created test data:"
cat test_sort_data.csv

echo ""
echo "Starting SQL CLI - press 's' to sort by current column..."
echo "Logs will be in ~/.local/share/sql-cli/logs/"

# Run with debug logging for sort and key mapping
RUST_LOG=sql_cli::ui::key_mapper=debug,sql_cli::ui::enhanced_tui=debug,sql_cli::ui::key_chord_handler=debug timeout 10 ./target/release/sql-cli test_sort_data.csv -e "select * from data"