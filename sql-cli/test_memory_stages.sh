#!/bin/bash
# Test memory usage at different stages

echo "Memory Usage Investigation"
echo "=========================="
echo ""

# Function to get memory usage of a process
get_memory() {
    ps aux | grep "$1" | grep -v grep | awk '{print $6}'
}

# Create a test CSV if it doesn't exist
if [ ! -f "test_memory_10k.csv" ]; then
    echo "Creating test CSV with 10k rows..."
    cat > test_memory_10k.csv << 'EOF'
id,symbol,price,quantity,side,trader,timestamp,exchange,orderType,status
EOF
    
    for i in $(seq 1 10000); do
        echo "$i,AAPL,150.$((RANDOM % 100)),1000,BUY,Trader$((RANDOM % 100)),2024-01-01T10:00:00,NYSE,LIMIT,FILLED" >> test_memory_10k.csv
    done
    echo "Created test_memory_10k.csv"
fi

echo "Test 1: Load CSV and immediately exit"
echo "--------------------------------------"
echo "This will show memory for just loading the CSV"
echo ""

# We can't easily test this in bash, but we can show how to test manually
echo "To test memory usage:"
echo "1. Run: ./target/release/sql-cli test_memory_10k.csv"
echo "2. In another terminal: ps aux | grep sql-cli"
echo "3. Note the RSS (6th column) - that's memory in KB"
echo "4. Press Ctrl+C to exit"
echo ""

echo "Test 2: Check what's in memory"
echo "------------------------------"
echo "Let's check the actual data structures:"
echo ""

# Create a debug script
cat > debug_memory.txt << 'EOF'
When sql-cli is running with a CSV loaded:

1. Check process memory:
   ps aux | grep sql-cli | grep -v grep

2. In the TUI, press F5 for debug mode to see:
   - Buffer state
   - Number of rows loaded
   - Current mode
   
3. Press F6 to create DataTable and see memory comparison

4. The memory is likely coming from:
   a) CsvDataSource: Vec<Value> (JSON objects)
   b) QueryResponse: Another Vec<Value> 
   c) DataTable: Typed values
   d) Ratatui: String allocations per frame
   
5. For 10k rows with 10 columns:
   - Raw data: ~10k * 100 bytes = 1MB
   - JSON with field names: ~10k * 500 bytes = 5MB
   - Multiple copies: 5MB * 3 = 15MB
   - String allocations: Could be 10x more!
EOF

cat debug_memory.txt

echo ""
echo "Hypothesis: Memory is consumed by:"
echo "1. Multiple copies of data (JSON + DataTable + filtered)"
echo "2. String allocations (field names repeated 10k times)"
echo "3. Ratatui creating Row/Cell objects (even for non-visible rows?)"
echo "4. Rust's allocator overhead and fragmentation"
echo ""
echo "Solutions to explore:"
echo "1. Remove JSON storage completely (V50)"
echo "2. Use string interning for field names"
echo "3. Implement true virtual scrolling (only create visible rows)"
echo "4. Use a custom allocator optimized for this pattern"