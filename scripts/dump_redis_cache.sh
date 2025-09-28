#!/bin/bash
# Dump entire Redis cache for sql-cli

echo "=== SQL-CLI Redis Cache Full Dump ==="
echo "Generated at: $(date)"
echo "========================================="
echo ""

# Check if Redis is running
if ! redis-cli ping > /dev/null 2>&1; then
    echo "❌ Redis is not running"
    exit 1
fi

# Count total entries
TOTAL=$(redis-cli --scan --pattern "sql-cli:*" 2>/dev/null | wc -l)
echo "📊 Total cached entries: $TOTAL"

if [ "$TOTAL" -eq 0 ]; then
    echo "No cache entries found"
    exit 0
fi

echo ""
echo "========================================="

# Iterate through each key
KEY_NUM=0
redis-cli --scan --pattern "sql-cli:*" 2>/dev/null | while read key; do
    KEY_NUM=$((KEY_NUM + 1))

    echo ""
    echo "🔑 Entry #$KEY_NUM"
    echo "----------------------------------------"
    echo "Key: $key"

    # Get TTL
    ttl=$(redis-cli TTL "$key" 2>/dev/null)
    if [ "$ttl" -eq -1 ]; then
        echo "TTL: Never expires"
    elif [ "$ttl" -eq -2 ]; then
        echo "TTL: Key doesn't exist (expired)"
    else
        # Convert to human readable
        if [ "$ttl" -gt 3600 ]; then
            hours=$((ttl / 3600))
            mins=$(( (ttl % 3600) / 60 ))
            echo "TTL: $ttl seconds (${hours}h ${mins}m remaining)"
        elif [ "$ttl" -gt 60 ]; then
            mins=$((ttl / 60))
            secs=$((ttl % 60))
            echo "TTL: $ttl seconds (${mins}m ${secs}s remaining)"
        else
            echo "TTL: $ttl seconds"
        fi
    fi

    # Get size
    size_bytes=$(redis-cli MEMORY USAGE "$key" 2>/dev/null)
    if [ -n "$size_bytes" ]; then
        size_kb=$((size_bytes / 1024))
        if [ "$size_kb" -gt 1024 ]; then
            size_mb=$((size_kb / 1024))
            echo "Size: ${size_mb} MB ($size_bytes bytes)"
        else
            echo "Size: ${size_kb} KB ($size_bytes bytes)"
        fi
    fi

    # Get type
    type=$(redis-cli TYPE "$key" 2>/dev/null)
    echo "Type: $type"

    # Show first 500 chars of data (if it's a string)
    if [ "$type" = "string" ]; then
        echo ""
        echo "Data preview (first 500 bytes):"
        echo "----------------------------------------"
        redis-cli GET "$key" 2>/dev/null | head -c 500 | strings | head -10
        echo ""
        echo "[... truncated ...]"
    fi

    echo "========================================="
done

echo ""
echo "📈 Cache Statistics:"
echo "----------------------------------------"

# Overall Redis memory info
echo "Memory usage:"
redis-cli INFO memory | grep -E "used_memory_human|used_memory_peak_human" | sed 's/:/: /'

echo ""
echo "🛠️  Useful Commands:"
echo "----------------------------------------"
echo "View specific key:     redis-cli GET 'sql-cli:web:...'"
echo "Delete specific key:   redis-cli DEL 'sql-cli:web:...'"
echo "Clear all cache:       redis-cli --scan --pattern 'sql-cli:*' | xargs redis-cli DEL"
echo "Monitor in real-time:  redis-cli MONITOR | grep sql-cli"
echo "Export to file:        redis-cli --scan --pattern 'sql-cli:*' | xargs -I{} sh -c 'echo {}; redis-cli GET {}' > cache_dump.txt"
echo ""