# Redis Cache for Web CTEs

## Overview

SQL-CLI now supports Redis caching for Web CTEs, providing dramatic performance improvements when working with production APIs that return large datasets. This is particularly useful when iterating on complex queries against Bloomberg, Barclays, or other financial data sources.

## Performance Impact

- **Without cache**: 600-640ms per query (fetching from API each time)
- **With cache**: 50-60ms per query (10-12x faster)
- Perfect for iterative query development against production data

## Setup

### 1. Install Redis (WSL/Ubuntu)

```bash
sudo apt update && sudo apt install -y redis-server
sudo service redis-server start
redis-cli ping  # Should return PONG
```

### 2. Enable Cache

The cache is **opt-in by default** to ensure sql-cli works for users without Redis.

#### Option A: Environment Variable (Recommended for testing)
```bash
export SQL_CLI_CACHE=true
./target/release/sql-cli your_query.sql
```

#### Option B: Neovim Plugin Configuration
```lua
require('sql-cli').setup({
  cache = {
    enabled = true,              -- Enable Redis cache
    redis_url = nil,            -- Optional: custom Redis URL (default: localhost:6379)
    show_notifications = true,   -- Show cache hit/miss notifications
  },
})
```

## Usage

### Basic Web CTE with Cache

```sql
WITH WEB trades AS (
    URL 'https://api.bloomberg.com/trades/yesterday'
    FORMAT JSON
    CACHE 3600  -- Cache for 1 hour
)
SELECT * FROM trades WHERE price > 100;
```

### Cache Duration Guidelines

- **Historical data** (won't change): `CACHE 86400` (24 hours)
- **Yesterday's trades**: `CACHE 43200` (12 hours)
- **Current day data**: `CACHE 300` (5 minutes)
- **Real-time data**: `CACHE 60` (1 minute) or omit CACHE

### Visual Feedback

When using the nvim plugin with cache enabled, you'll see:

**First execution (Cache MISS):**
```
⚠️ Cache MISS for trades (key: sql-cli:web:9a5b0f087d...)
💾 Cached trades for 3600 seconds
[Query results...]
```

**Subsequent executions (Cache HIT):**
```
✅ Cache HIT for trades (key: sql-cli:web:9a5b0f087d...)
[Query results instantly!]
```

## Cache Management

### Check Cache Status

```bash
# Custom helper script
./scripts/check_redis_cache.sh

# Or manually
redis-cli KEYS "sql-cli:*"
redis-cli TTL "sql-cli:web:..."
```

### Clear Cache

```bash
# Clear all sql-cli cache entries
redis-cli --scan --pattern "sql-cli:*" | xargs redis-cli DEL

# Clear specific entry
redis-cli DEL "sql-cli:web:9a5b0f087d..."
```

### Monitor Cache Activity

```bash
# Watch cache operations in real-time
redis-cli MONITOR | grep sql-cli
```

## How It Works

1. **Cache Key Generation**: SHA256 hash of URL + method + headers + body
2. **Storage Format**: MessagePack serialization of DataTable
3. **TTL Management**: Redis handles expiry automatically
4. **Graceful Degradation**: If Redis is down, queries still work (just slower)

## Production Example

```sql
-- Fetch 20k trades from Barclays (expensive operation)
WITH WEB trades AS (
    URL 'https://api.barclays.com/trades/2024-01-15'
    METHOD 'POST'
    HEADERS 'Authorization: Bearer YOUR_TOKEN'
    BODY '{"product": "FX", "size": 20000}'
    FORMAT JSON
    CACHE 43200  -- Cache for 12 hours (historical data)
)
SELECT
    trade_id,
    product,
    SUM(notional) as total_notional,
    AVG(price) as avg_price
FROM trades
GROUP BY trade_id, product
HAVING total_notional > 1000000;
```

First run: ~2 seconds (fetching 20k trades)
Subsequent runs: ~100ms (from cache)

## Troubleshooting

### Cache not working?

1. Check Redis is running:
```bash
redis-cli ping  # Should return PONG
```

2. Verify cache is enabled:
```bash
echo $SQL_CLI_CACHE  # Should be "true"
```

3. Check for cached entries:
```bash
redis-cli KEYS "sql-cli:*"
```

### WSL-Specific Issues

If Redis won't start in WSL:
```bash
# Fix Redis permissions
sudo chown redis:redis /var/lib/redis
sudo chmod 750 /var/lib/redis

# Start Redis
sudo service redis-server restart
```

## Architecture Notes

- Cache is implemented at the WebDataFetcher level
- Each unique combination of URL/method/headers/body gets its own cache entry
- Cache is transparent to the rest of the application
- No changes needed to existing queries - just add CACHE directive

## Security Considerations

- Cache keys are hashed - URLs/credentials are not stored in plain text
- Redis should only be accessible locally (default configuration)
- Cached data inherits Redis security model
- Consider Redis AUTH for production environments

## Future Enhancements

- [ ] Parquet format for better compression
- [ ] Cache statistics in TUI status bar
- [ ] Smart cache warming for common queries
- [ ] Distributed cache for team environments