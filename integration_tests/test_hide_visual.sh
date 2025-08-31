#!/bin/bash
echo "Testing column hiding and visual display alignment..."

# Create test CSV with known data
cat > test_visual.csv << CSV
Name,Age,City,Comments,Status
Alice,30,NYC,Long comment here,Active
Bob,25,LA,Another comment,Inactive
Charlie,35,Chicago,Third comment,Active
CSV

# Run with debug logging and hide columns
RUST_LOG=sql_cli::ui::viewport_manager=debug,sql_cli::data::data_view=debug timeout 3 ./target/release/sql-cli test_visual.csv -e "select * from data" 2>&1 | head -100

echo "Test complete"
