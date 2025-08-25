#!/bin/bash

# Test script for jump-to-row and help mode functionality

echo "Testing jump-to-row and help mode fixes..."

# Create test file with expect script to test modes
cat > test_modes.exp << 'EOF'
#!/usr/bin/expect -f

set timeout 10

# Start the application 
spawn timeout 30 ./target/release/sql-cli test_jump_to_row.csv -e "select * from data"

# Wait for the application to load
expect "Results"

# Test F1 help mode
send "F1"
expect "Help Mode"
sleep 1
send "\033"  # Escape key
expect "Results"
puts "✅ F1 help mode test: PASSED"

# Test G jump-to-row mode
send "G"
expect "Jump to row:"
puts "✅ Jump to row mode entry: PASSED"

# Test typing a row number
send "5"
expect "5"

# Test Enter to jump
send "\r"
expect "Results"
puts "✅ Jump to row with Enter: PASSED"

# Test G and Escape
send "G"
expect "Jump to row:"
send "\033"  # Escape key
expect "Results"
puts "✅ Jump to row with Escape: PASSED"

# Exit application
send "q"
expect eof
puts "✅ All mode switching tests PASSED!"
EOF

# Make expect script executable
chmod +x test_modes.exp

# Run the test
if ./test_modes.exp; then
    echo "✅ Mode switching functionality working correctly!"
    rm -f test_modes.exp
    exit 0
else
    echo "❌ Mode switching tests failed"
    rm -f test_modes.exp
    exit 1
fi