#!/bin/bash

echo "Testing memory optimization with 20k row file..."
echo ""

# Run the application and capture the log file
OUTPUT=$(./target/release/sql-cli ~/dev/sql-cli/trades_20k.csv 2>&1 | head -5)
LOG_FILE=$(echo "$OUTPUT" | grep "Debug logs will be written to:" | sed 's/.*Debug logs will be written to://' | xargs)

if [ -z "$LOG_FILE" ]; then
    echo "Error: Could not find log file"
    exit 1
fi

echo "Log file: $LOG_FILE"
echo ""

# Extract memory tracking information
echo "Memory Usage Timeline:"
grep "MEMORY\[" "$LOG_FILE" | grep -E "(before|after|complete)" | tail -5

echo ""
echo "Key Memory Points:"
BEFORE=$(grep "before_arc_share" "$LOG_FILE" | awk -F': ' '{print $NF}' | awk '{print $1}')
AFTER=$(grep "after_arc_share" "$LOG_FILE" | awk -F': ' '{print $NF}' | awk '{print $1}')

echo "  Before Arc sharing: $BEFORE MB"
echo "  After Arc sharing:  $AFTER MB"

# Calculate difference
if [ ! -z "$BEFORE" ] && [ ! -z "$AFTER" ]; then
    DIFF=$((AFTER - BEFORE))
    echo "  Memory increase: $DIFF MB (should be ~1 MB, not 100+ MB)"
    echo ""
    
    if [ $DIFF -le 5 ]; then
        echo "✅ SUCCESS: Memory optimization is working! Only $DIFF MB increase."
    else
        echo "❌ FAILURE: Memory still being duplicated. $DIFF MB increase."
    fi
else
    echo "Could not calculate memory difference"
fi

echo ""
echo "Process Memory Info:"
grep "Process Total:" "$LOG_FILE" | tail -1