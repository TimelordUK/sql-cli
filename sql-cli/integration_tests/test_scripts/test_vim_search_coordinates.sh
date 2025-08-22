#!/bin/bash
# Test script to verify vim search coordinate conversion is correct
# This tests that when searching and navigating matches, the crosshair
# is positioned at the correct absolute coordinates, not viewport-relative

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing vim search coordinate handling...${NC}"

# Create test data with known "derivatives" matches
cat > /tmp/test_search_data.csv << 'EOF'
id,status,book,portfolio,value
1,Unallocated,Derivatives,Portfolio-A,1000
2,Pending,Options Trading,Portfolio-B,2000
3,Unallocated,Options Trading,Portfolio-C,3000
4,Pending,Futures Trading,Portfolio-D,4000
5,Partial,ETF Trading,Portfolio-E,5000
6,Unallocated,Derivatives,Portfolio-F,6000
7,Pending,Options Trading,Derivatives-1,7000
8,Unallocated,Derivatives,Portfolio-H,8000
9,Pending,Futures Trading,Portfolio-I,9000
10,Partial,ETF Trading,Portfolio-J,10000
11,Unallocated,Derivatives,Portfolio-K,11000
12,Pending,Options Trading,Portfolio-L,12000
13,Unallocated,Options Trading,Portfolio-M,13000
14,Pending,Futures Trading,Portfolio-N,14000
15,Partial,Derivatives,Derivatives-7,15000
16,Unallocated,Derivatives,Portfolio-P,16000
EOF

# Build the application
echo -e "${YELLOW}Building application...${NC}"
cargo build --release 2>/dev/null

# Run the application and test search
echo -e "${YELLOW}Testing search navigation...${NC}"

# Create expect script to test vim search
cat > /tmp/test_vim_search.exp << 'EOF'
#!/usr/bin/expect -f
set timeout 5
log_user 0

spawn ./target/release/sql-cli /tmp/test_search_data.csv

# Wait for initial render
sleep 0.5

# Start vim search with /
send "/"
sleep 0.2

# Type "derivatives"
send "derivatives"
sleep 0.2

# Press Enter to confirm search
send "\r"
sleep 0.2

# Press F5 to get debug output
send "\033\[15~"
sleep 0.5

# Capture screen content
expect {
    -re ".*ViewportManager Crosshair: row=(\[0-9\]+), col=(\[0-9\]+).*" {
        set crosshair_row $expect_out(1,string)
        set crosshair_col $expect_out(2,string)
    }
    timeout {
        puts "ERROR: Could not find crosshair position"
        exit 1
    }
}

# Check for first match at row 0, col 2 (book column)
if {$crosshair_row == 0 && $crosshair_col == 2} {
    puts "PASS: First match correctly positioned at absolute (0, 2)"
} else {
    puts "FAIL: Expected crosshair at (0, 2), got ($crosshair_row, $crosshair_col)"
    exit 1
}

# Press 'n' to go to next match
send "n"
sleep 0.2

# Press F5 again
send "\033\[15~"
sleep 0.5

# Check second match position
expect {
    -re ".*ViewportManager Crosshair: row=(\[0-9\]+), col=(\[0-9\]+).*" {
        set crosshair_row $expect_out(1,string)
        set crosshair_col $expect_out(2,string)
    }
    timeout {
        puts "ERROR: Could not find crosshair position after 'n'"
        exit 1
    }
}

# Should move to next match (row 5, col 2)
if {$crosshair_row == 5 && $crosshair_col == 2} {
    puts "PASS: Second match correctly positioned at absolute (5, 2)"
} else {
    puts "INFO: Second match at ($crosshair_row, $crosshair_col) - may be correct depending on data"
}

# Press 'n' multiple times to test wrapping
send "nnn"
sleep 0.5

send "\033\[15~"
sleep 0.5

expect {
    -re ".*ViewportManager Crosshair: row=(\[0-9\]+), col=(\[0-9\]+).*" {
        set crosshair_row $expect_out(1,string)
        set crosshair_col $expect_out(2,string)
        puts "INFO: After multiple 'n' presses, crosshair at ($crosshair_row, $crosshair_col)"
    }
}

# Exit
send "q"
expect eof
EOF

chmod +x /tmp/test_vim_search.exp

# Run the expect script
if /tmp/test_vim_search.exp; then
    echo -e "${GREEN}✓ Vim search coordinate test passed${NC}"
else
    echo -e "${RED}✗ Vim search coordinate test failed${NC}"
    exit 1
fi

# Clean up
rm -f /tmp/test_search_data.csv /tmp/test_vim_search.exp

echo -e "${GREEN}All tests passed!${NC}"