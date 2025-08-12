#!/bin/bash

# Test F5 debug dump with new states
echo "Testing F5 debug dump for V31 states..."

# Create a temporary expect script
cat > /tmp/test_f5.exp << 'EOF'
#!/usr/bin/expect -f
set timeout 10
spawn ./target/release/sql-cli data/instruments.csv
expect "*Command*"
send "\033\[16~"  ;# Send F5 key
expect "*DEBUG*"
send "q"           ;# Quit
expect eof
EOF

chmod +x /tmp/test_f5.exp

# Run the expect script
if command -v expect > /dev/null 2>&1; then
    expect /tmp/test_f5.exp 2>/dev/null | grep -A 100 "DEBUG DUMP" | head -150
else
    echo "expect not installed, testing with echo simulation"
    # Alternative: just compile and check the code compiles correctly
    cargo build --release 2>&1 | tail -5
fi

rm -f /tmp/test_f5.exp