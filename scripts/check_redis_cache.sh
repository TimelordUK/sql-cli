#!/bin/bash
# Script to check Redis cache status for sql-cli

echo "=== SQL-CLI Redis Cache Status ==="
echo ""

# Check if Redis is running
if redis-cli ping > /dev/null 2>&1; then
    echo "✅ Redis is running"
else
    echo "❌ Redis is not running"
    echo "Start with: sudo service redis-server start"
    exit 1
fi

echo ""
echo "=== Cache Entries ==="

# Count total entries
TOTAL=$(redis-cli --scan --pattern "sql-cli:*" 2>/dev/null | wc -l)
echo "Total cached queries: $TOTAL"

if [ "$TOTAL" -eq 0 ]; then
    echo "No cached entries found"
    exit 0
fi

echo ""
echo "=== Cache Details ==="
echo "Key                                     | TTL    | Size      | URL Preview"
echo "----------------------------------------|--------|-----------|------------------"

# Show each cache entry with details
redis-cli --scan --pattern "sql-cli:*" 2>/dev/null | while read key; do
    # Get TTL
    ttl=$(redis-cli TTL "$key" 2>/dev/null)

    # Format TTL
    if [ "$ttl" -eq -1 ]; then
        ttl_str="never"
    elif [ "$ttl" -eq -2 ]; then
        ttl_str="gone"
    elif [ "$ttl" -gt 3600 ]; then
        hours=$((ttl / 3600))
        ttl_str="${hours}h"
    elif [ "$ttl" -gt 60 ]; then
        mins=$((ttl / 60))
        ttl_str="${mins}m"
    else
        ttl_str="${ttl}s"
    fi

    # Get size
    size_bytes=$(redis-cli MEMORY USAGE "$key" 2>/dev/null)
    if [ -n "$size_bytes" ]; then
        size_kb=$((size_bytes / 1024))
        if [ "$size_kb" -gt 1024 ]; then
            size_mb=$((size_kb / 1024))
            size_str="${size_mb}MB"
        else
            size_str="${size_kb}KB"
        fi
    else
        size_str="?"
    fi

    # Extract key hash (last part)
    short_key=${key#sql-cli:web:}
    short_key=${short_key:0:16}...

    # Try to peek at the data to guess the URL (first 100 bytes)
    data_peek=$(redis-cli GET "$key" 2>/dev/null | head -c 100 | strings | head -1)

    printf "%-40s| %-6s | %-9s | %-20s\n" "$short_key" "$ttl_str" "$size_str" "${data_peek:0:20}"
done

echo ""
echo "=== Cache Memory Usage ==="
redis-cli INFO memory | grep used_memory_human | cut -d: -f2

echo ""
echo "=== Quick Commands ==="
echo "View all keys:        redis-cli KEYS 'sql-cli:*'"
echo "Delete specific key:  redis-cli DEL 'sql-cli:web:...'"
echo "Clear all cache:      redis-cli --scan --pattern 'sql-cli:*' | xargs redis-cli DEL"
echo "Monitor activity:     redis-cli MONITOR | grep sql-cli"
echo ""
echo "Check if cache is enabled: echo \$SQL_CLI_CACHE"