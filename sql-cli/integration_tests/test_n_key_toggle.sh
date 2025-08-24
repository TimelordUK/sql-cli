#!/bin/bash
# Test that N key toggles line numbers correctly after search mode

set -e

echo "Testing N key toggle after search mode..."

# Create test data
cat > test_n_key.csv << EOF
id,name,age
1,Alice,30
2,Bob,25
3,Charlie,35
4,David,40
5,Eve,28
EOF

# Test script using expect
cat > test_n_key.exp << 'EXPECT_SCRIPT'
#!/usr/bin/expect -f

set timeout 2
set test_failed 0

spawn ./target/release/sql-cli test_n_key.csv -e "select * from data"

# Wait for initial display
expect {
    timeout { 
        puts "FAIL: Timeout waiting for initial display"
        exit 1 
    }
    "rows" { 
        puts "✓ Initial display loaded" 
    }
}

# Test 1: N key should toggle line numbers initially
send "N"
expect {
    timeout { 
        puts "FAIL: N key didn't toggle line numbers initially"
        set test_failed 1
    }
    "│ 1 │" {
        puts "✓ Line numbers shown after first N press"
    }
}

# Test 2: Press N again to hide line numbers
send "N"
sleep 0.1
send "\r"  # Force refresh
expect {
    timeout {
        puts "FAIL: N key didn't hide line numbers"
        set test_failed 1
    }
    -re "│\\s+1\\s+│\\s+Alice" {
        puts "✓ Line numbers hidden after second N press"
    }
}

# Test 3: Enter search mode
send "/"
expect {
    timeout {
        puts "FAIL: Couldn't enter search mode"
        set test_failed 1
    }
    "/" {
        puts "✓ Entered search mode"
    }
}

# Type search pattern
send "Alice"
sleep 0.1

# Exit search mode with Escape
send "\033"
expect {
    timeout {
        puts "FAIL: Couldn't exit search mode"
        set test_failed 1
    }
    "rows" {
        puts "✓ Exited search mode"
    }
}

# Test 4: N key should still toggle line numbers (THE CRITICAL TEST)
send "N"
expect {
    timeout {
        puts "FAIL: N key doesn't work after search mode - BUG REPRODUCED!"
        set test_failed 1
    }
    "│ 1 │" {
        puts "✓ SUCCESS: N key toggles line numbers after search mode!"
    }
}

# Exit
send "q"
expect eof

if { $test_failed == 1 } {
    puts "\n❌ TEST FAILED: N key issue detected"
    exit 1
} else {
    puts "\n✅ TEST PASSED: N key works correctly"
    exit 0
}
EXPECT_SCRIPT

chmod +x test_n_key.exp

# Run the test
if ./test_n_key.exp; then
    echo "✅ N key toggle test PASSED"
    rm -f test_n_key.csv test_n_key.exp
    exit 0
else
    echo "❌ N key toggle test FAILED"
    rm -f test_n_key.csv test_n_key.exp
    exit 1
fi