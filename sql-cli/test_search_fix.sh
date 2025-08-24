#!/usr/bin/expect -f

set timeout 10

# Run with debug logging
spawn env RUST_LOG=sql_cli::ui::table_widget_manager=debug,search=info,sql_cli::ui::enhanced_tui=info ./sql-cli/target/release/sql-cli test_emerging.csv -e "select * from data"

# Wait for initial load
sleep 1

# Enter search mode
send "/"
expect "Search Pattern"
sleep 0.2

# Type search pattern
send "emerging"
sleep 1

# Check the logs in background
exec sh -c "sleep 0.5 && pkill -f sql-cli" &

# Wait for output
expect {
    "TableWidgetManager: Navigate to search match" { puts "\n✓ TableWidgetManager navigation triggered"; exp_continue }
    "Updating TableWidgetManager" { puts "\n✓ TableWidgetManager update called"; exp_continue }
    "needs render" { puts "\n✓ Render check triggered"; exp_continue }
    "RENDERING TABLE" { puts "\n✓ Table actually rendering"; exp_continue }
    "match 1/" { puts "\n✓ Match found in status"; exp_continue }
    timeout { puts "\n✗ Timeout waiting for navigation"; exit 1 }
}

send "\x03"
expect eof
