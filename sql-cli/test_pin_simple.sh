#!/bin/bash

echo "Testing pin columns feature..."

# Create a simple test with debug logging
echo "p" | RUST_LOG=sql_cli::data::data_view=debug,sql_cli::ui::viewport_manager=debug timeout 2 ./target/release/sql-cli test_pin_columns.csv -e "select * from data" 2>&1 | grep -i "pin"

echo ""
echo "Checking if pin action is registered..."
grep -r "pin_column" src/handlers/ src/action.rs src/ui/enhanced_tui.rs | head -10