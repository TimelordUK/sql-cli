# Cache Control Options for Web CTEs

## Methods to Bypass or Refresh Cache

### 1. Disable Cache for Single Query (Environment Variable)
```bash
# This query will bypass cache and fetch fresh data
SQL_CLI_NO_CACHE=1 sql-cli -q "WITH WEB trades AS (...) SELECT * FROM trades"
```

### 2. Don't Cache at All (Omit CACHE Directive)
```sql
-- Without CACHE directive, data is never cached
WITH WEB trades AS (
    URL 'https://prod.api/trades'
    METHOD POST
    BODY '{"date": "2025-09-01"}'
    -- No CACHE directive = no caching
)
SELECT * FROM trades;
```

### 3. Short TTL for Frequently Changing Data
```sql
-- Cache for only 1 second (effectively no cache for human use)
WITH WEB trades AS (
    URL 'https://prod.api/trades'
    CACHE 1  -- 1 second TTL
)
SELECT * FROM trades;
```

### 4. Clear Specific Cache Entry (Redis CLI)
```bash
# Find the cache key
redis-cli KEYS "sql-cli:*" | grep <part_of_url>

# Delete specific cache entry
redis-cli DEL "sql-cli:web:a3f4b2c1..."

# Next query will fetch fresh data
```

### 5. Clear All sql-cli Cache
```bash
# Nuclear option - clear all cached queries
redis-cli --scan --pattern "sql-cli:*" | xargs redis-cli DEL
```

### 6. Force Refresh Pattern (Workaround)
```sql
-- Add a dummy parameter that changes to force new cache key
WITH WEB trades AS (
    URL 'https://prod.api/trades'
    METHOD POST
    BODY '{"date": "2025-09-01", "refresh": "1"}'  -- Change refresh value
    CACHE 3600
)
SELECT * FROM trades;

-- Next time, change refresh value to force new fetch
-- BODY '{"date": "2025-09-01", "refresh": "2"}'
```

## Future Enhancement: CACHE REFRESH Directive

We could add explicit refresh support:
```sql
-- Proposed syntax (not yet implemented)
WITH WEB trades AS (
    URL 'https://prod.api/trades'
    CACHE REFRESH  -- Force refresh but still cache result
    -- or
    CACHE 3600 REFRESH  -- Refresh now, then cache for 1 hour
)
```

## Current Best Practices

### For Development/Testing
```bash
# Bypass cache when testing
SQL_CLI_NO_CACHE=1 sql-cli query.sql
```

### For Production Queries
```sql
-- Historical data: Cache for 24 hours
WITH WEB trades AS (
    URL 'https://prod.api/trades'
    BODY '{"date": "2025-09-01"}'  -- Yesterday
    CACHE 86400  -- 24 hours
)

-- Current data: Cache briefly
WITH WEB trades AS (
    URL 'https://prod.api/trades'
    BODY '{"date": "2025-09-02"}'  -- Today
    CACHE 300  -- 5 minutes
)
```

### Manual Refresh When Needed
```bash
# Step 1: Find cache key
redis-cli KEYS "sql-cli:*"

# Step 2: Check what will be deleted
redis-cli GET "sql-cli:web:..." | head -c 100

# Step 3: Delete specific entry
redis-cli DEL "sql-cli:web:..."

# Step 4: Run query to fetch fresh data
sql-cli query.sql
```

## Monitoring Cache

```bash
# Watch cache activity in real-time
redis-cli MONITOR | grep sql-cli

# Check TTL of cached entries
for key in $(redis-cli KEYS "sql-cli:*"); do
    ttl=$(redis-cli TTL "$key")
    echo "$key: $ttl seconds remaining"
done
```

The current implementation provides good control through environment variables and Redis commands. The cache is transparent and can be bypassed when needed!