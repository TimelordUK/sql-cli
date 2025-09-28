#!/bin/bash
# Show all sql-cli cache entries with their TTL

echo "=== SQL-CLI Cache TTL Report ==="
echo ""

# Get all cache keys
keys=$(redis-cli --scan --pattern "sql-cli:*" 2>/dev/null)

if [ -z "$keys" ]; then
    echo "No cache entries found"
    exit 0
fi

echo "Key Hash                                | TTL (seconds) | Expires In"
echo "----------------------------------------|---------------|------------------"

for key in $keys; do
    # Get TTL
    ttl=$(redis-cli TTL "$key" 2>/dev/null)

    # Extract short hash
    short_key=${key#sql-cli:web:}
    short_key=${short_key:0:16}...

    # Format TTL display
    if [ "$ttl" -eq -1 ]; then
        ttl_display="Never"
        human_ttl="Permanent"
    elif [ "$ttl" -eq -2 ]; then
        ttl_display="Expired"
        human_ttl="Already expired"
    else
        ttl_display="$ttl"
        # Convert to human readable
        if [ "$ttl" -gt 3600 ]; then
            hours=$((ttl / 3600))
            mins=$(( (ttl % 3600) / 60 ))
            human_ttl="${hours}h ${mins}m"
        elif [ "$ttl" -gt 60 ]; then
            mins=$((ttl / 60))
            secs=$((ttl % 60))
            human_ttl="${mins}m ${secs}s"
        else
            human_ttl="${ttl}s"
        fi
    fi

    printf "%-40s| %-13s | %s\n" "$short_key" "$ttl_display" "$human_ttl"
done

echo ""
echo "To get full details for a specific key:"
echo "  redis-cli GET 'sql-cli:web:HASH' | head -c 100"
echo ""
echo "To see when a key was created (if Redis tracks it):"
echo "  redis-cli OBJECT IDLETIME 'sql-cli:web:HASH'"