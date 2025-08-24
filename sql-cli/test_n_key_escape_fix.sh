#!/bin/bash

# Test script to verify N key toggle fix after search
# Bug: After entering search mode (/) and exiting, N continues to navigate search instead of toggling line numbers

echo "Testing N key toggle after search mode..."
echo ""

# Create test data
cat > test_n_escape.csv << 'EOF'
id,name,type,status
1,Alice,derivatives,active
2,Bob,futures,pending
3,Charlie,derivatives,active
4,David,options,inactive
5,Eve,derivatives,pending
EOF

# Create expect script to test the interaction
cat > test_n_escape.exp << 'EOF'
#!/usr/bin/expect -f
set timeout 5

# Start the application
spawn ./target/release/sql-cli test_n_escape.csv -e "select * from data"

# Wait for initial display
expect {
    timeout { puts "TIMEOUT: Initial display"; exit 1 }
    "rows*cols" { }
}

# Enter search mode with /
send "/"
expect {
    timeout { puts "TIMEOUT: Entering search mode"; exit 1 }
    "Search:" { }
}

# Type search pattern
send "derivatives"
expect {
    timeout { puts "TIMEOUT: Typing search pattern"; exit 1 }
    "derivatives" { }
}

# Apply search with Enter
send "\r"
expect {
    timeout { puts "TIMEOUT: Applying search"; exit 1 }
    -re "Match.*of" { }
}

# Navigate with n a few times
send "n"
expect {
    timeout { puts "TIMEOUT: First n navigation"; exit 1 }
    -re "Match.*of" { }
}

send "n"
expect {
    timeout { puts "TIMEOUT: Second n navigation"; exit 1 }
    -re "Match.*of" { }
}

# Press Escape to clear search
send "\033"
expect {
    timeout { puts "TIMEOUT: Escape to clear search"; exit 1 }
    "Search cleared" { puts "✓ Search cleared message appeared" }
    -re ".*" { }
}

# Brief pause
sleep 0.5

# Now test N key - should toggle line numbers, not navigate
send "N"
expect {
    timeout { puts "TIMEOUT: N key after Escape"; exit 1 }
    "Toggled line numbers" { puts "✓ SUCCESS: N key toggles line numbers after Escape!" }
    -re "Match.*of" { puts "✗ FAIL: N key still navigating search!"; exit 1 }
    -re ".*" { 
        # Check F5 debug to see state
        send "\[24~"
        expect {
            timeout { puts "TIMEOUT: F5 debug"; exit 1 }
            -re "pattern='(\[^']*)'.*active=(\[^ ]*)" {
                set pattern $expect_out(1,string)
                set active $expect_out(2,string)
                if {$pattern != ""} {
                    puts "✗ FAIL: Search pattern still present: '$pattern', active=$active"
                    exit 1
                } else {
                    puts "✓ Pattern cleared, checking for line number toggle..."
                }
            }
        }
    }
}

# Exit
send "q"
expect eof
EOF

chmod +x test_n_escape.exp

# Run the test
echo "Running test..."
RUST_LOG=info ./test_n_escape.exp 2>&1 | grep -E "✓|✗|FAIL|SUCCESS|TIMEOUT|VimSearchAdapter.*ESCAPE|Buffer.set_search_pattern|Search cleared"

echo ""
echo "Test complete. Check output above for results."