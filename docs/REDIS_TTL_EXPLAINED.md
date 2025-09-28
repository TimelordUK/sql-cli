# Redis TTL (Time To Live) Explained

## How Redis TTL Works

Redis has built-in key expiration - you don't need to store timestamps in keys!

### Setting Keys with TTL

```bash
# Set key with expiry in seconds
SETEX key 3600 "value"  # Expires in 1 hour

# Or set key then add expiry
SET key "value"
EXPIRE key 3600

# Set with millisecond precision
PSETEX key 3600000 "value"  # 3600000ms = 1 hour
```

### Checking TTL

```bash
# Get remaining TTL in seconds
TTL key
# Returns:
#   -2 = key doesn't exist
#   -1 = key exists but has no TTL (won't expire)
#   N  = N seconds until expiry

# Get TTL in milliseconds
PTTL key

# Check if key exists
EXISTS key
```

### Redis Expiry Examples

```bash
# Start Redis
docker run -d --name redis-test -p 6379:6379 redis:alpine

# Connect with redis-cli
redis-cli

# Set a key with 60 second TTL
127.0.0.1:6379> SETEX trades:cache:001 60 "trade_data_here"
OK

# Check TTL
127.0.0.1:6379> TTL trades:cache:001
(integer) 57  # 57 seconds remaining

# Wait a bit...
127.0.0.1:6379> TTL trades:cache:001
(integer) 45  # 45 seconds remaining

# After 60 seconds
127.0.0.1:6379> GET trades:cache:001
(nil)  # Key automatically deleted

# Check TTL of expired key
127.0.0.1:6379> TTL trades:cache:001
(integer) -2  # Key doesn't exist
```

## For Our Cache Implementation

We DON'T need to embed timestamps in keys! Redis handles it:

```rust
// Set with TTL
conn.set_ex(
    "sql-cli:web:a3f4b2c1",  // Simple hash key
    data_bytes,
    86400  // TTL in seconds (24 hours)
)?;

// Check remaining TTL
let ttl: i64 = conn.ttl("sql-cli:web:a3f4b2c1")?;
match ttl {
    -2 => println!("Key doesn't exist"),
    -1 => println!("Key never expires"),
    n => println!("Key expires in {} seconds", n),
}
```

## Monitoring Cache Age

```bash
# See all sql-cli keys with their TTLs
redis-cli --scan --pattern "sql-cli:*" | while read key; do
    ttl=$(redis-cli TTL "$key")
    echo "$key: TTL=$ttl seconds"
done

# Output:
sql-cli:web:a3f4b2c1: TTL=85234 seconds
sql-cli:web:b5d6e7f8: TTL=298 seconds
sql-cli:web:c9a0b1d2: TTL=-1 seconds (never expires)
```

## Smart TTL Strategy for Trades

```rust
fn get_smart_ttl(spec: &WebCTESpec) -> u64 {
    // Check if querying historical data
    if let Some(body) = &spec.body {
        // Parse date from body
        if body.contains("2025-09-01") {  // Yesterday
            return 86400;  // 24 hours - historical data won't change
        }
        if body.contains(today()) {
            return 300;  // 5 minutes - today's data might update
        }
    }

    // Default based on URL
    if spec.url.contains("prod") {
        3600  // 1 hour for production
    } else {
        300   // 5 minutes for staging
    }
}
```

## Cache Inspection Script

```bash
#!/bin/bash
# show_cache_status.sh

echo "SQL-CLI Cache Status"
echo "==================="

# Count total keys
total=$(redis-cli --scan --pattern "sql-cli:*" | wc -l)
echo "Total cached queries: $total"

# Show keys with TTL
echo -e "\nCached Queries:"
redis-cli --scan --pattern "sql-cli:*" | while read key; do
    ttl=$(redis-cli TTL "$key")
    size=$(redis-cli MEMORY USAGE "$key")

    # Convert TTL to human readable
    if [ "$ttl" -eq -1 ]; then
        ttl_str="never expires"
    elif [ "$ttl" -eq -2 ]; then
        ttl_str="expired"
    elif [ "$ttl" -gt 3600 ]; then
        hours=$((ttl / 3600))
        ttl_str="${hours}h remaining"
    elif [ "$ttl" -gt 60 ]; then
        mins=$((ttl / 60))
        ttl_str="${mins}m remaining"
    else
        ttl_str="${ttl}s remaining"
    fi

    # Short key for display
    short_key=${key#sql-cli:web:}
    short_key=${short_key:0:12}...

    echo "  $short_key: $ttl_str (${size:-0} bytes)"
done

# Show memory usage
echo -e "\nMemory Usage:"
redis-cli INFO memory | grep used_memory_human
```

## Benefits of Redis TTL

1. **Automatic cleanup** - No manual deletion needed
2. **Memory efficient** - Redis automatically reclaims expired keys
3. **No timestamp parsing** - TTL is metadata, not in the key
4. **Atomic operations** - Set and expiry in one command
5. **Millisecond precision** - Use PTTL for sub-second expiry

This is why Redis is perfect for caching - the TTL feature is exactly what we need!