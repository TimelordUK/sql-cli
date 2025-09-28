# Redis as the Cache Layer for SQL CLI

## Why Redis is Perfect Here

1. **Already solved problems**: TTL, eviction, persistence, clustering
2. **Binary safe**: Can store Parquet/Arrow directly
3. **Fast**: In-memory with optional persistence
4. **Shared**: Multiple sql-cli instances share same cache
5. **Simple**: Just GET/SET operations
6. **Observable**: Redis CLI, RedisInsight for debugging

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ nvim/CLI    │────▶│   sql-cli   │────▶│    Redis    │
└─────────────┘     │  (checks     │     │  (cache)    │
                    │   Redis)     │     └─────────────┘
                    └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ Prod APIs   │
                    │ (on miss)   │
                    └─────────────┘
```

## Implementation - Dead Simple

```rust
// Cargo.toml
[dependencies]
redis = "0.23"
bytes = "1.5"

// src/cache/redis_cache.rs
use redis::{Client, Commands, Connection};
use std::time::Duration;

pub struct RedisCache {
    conn: Connection,
}

impl RedisCache {
    pub fn new() -> Result<Self> {
        let client = Client::open("redis://127.0.0.1:6379")?;
        let conn = client.get_connection()?;
        Ok(Self { conn })
    }

    pub fn get_datatable(&mut self, key: &str) -> Option<DataTable> {
        // Get binary data from Redis
        let data: Option<Vec<u8>> = self.conn.get(key).ok()?;

        // Deserialize from Parquet bytes
        data.and_then(|bytes| {
            DataTable::from_parquet_bytes(&bytes).ok()
        })
    }

    pub fn set_datatable(&mut self, key: &str, table: &DataTable, ttl: Duration) -> Result<()> {
        // Serialize to Parquet
        let bytes = table.to_parquet_bytes()?;

        // Store with TTL
        self.conn.set_ex(key, bytes, ttl.as_secs() as usize)?;
        Ok(())
    }
}
```

## In Web Data Fetcher

```rust
impl WebDataFetcher {
    pub fn fetch(&self, spec: &WebCTESpec, table_name: &str) -> Result<DataTable> {
        // Generate cache key
        let cache_key = format!("sql-cli:web:{}", spec.cache_key());

        // Try Redis first
        if let Ok(mut redis) = RedisCache::new() {
            if let Some(cached) = redis.get_datatable(&cache_key) {
                info!("Cache hit for {}", table_name);
                return Ok(cached);
            }
        }

        // Fetch from network
        info!("Cache miss for {}, fetching...", table_name);
        let data = self.fetch_from_network(spec)?;

        // Store in Redis
        if let Some(ttl_seconds) = spec.cache_seconds {
            if let Ok(mut redis) = RedisCache::new() {
                let _ = redis.set_datatable(
                    &cache_key,
                    &data,
                    Duration::from_secs(ttl_seconds)
                );
            }
        }

        Ok(data)
    }
}
```

## Cache Key Strategy

```rust
// For Web CTEs
sql-cli:web:{url_hash}:{body_hash}
sql-cli:web:trades:barclays:2025-09-01

// Examples:
sql-cli:web:a3f4b2c1  // Hash of URL+body+headers
sql-cli:web:trades:bloomberg:2025-09-01
sql-cli:web:trades:barclays:2025-09-01
```

## Redis Commands for Management

```bash
# See all cached queries
redis-cli --scan --pattern "sql-cli:*"

# Check specific cache
redis-cli GET "sql-cli:web:trades:barclays:2025-09-01"

# Check TTL
redis-cli TTL "sql-cli:web:trades:barclays:2025-09-01"

# Manual cache clear
redis-cli DEL "sql-cli:web:trades:barclays:2025-09-01"

# Clear all sql-cli cache
redis-cli --scan --pattern "sql-cli:*" | xargs redis-cli DEL

# Monitor cache hits/misses
redis-cli MONITOR | grep sql-cli
```

## Configuration

```toml
# ~/.config/sql-cli/config.toml
[cache]
enabled = true
backend = "redis"  # redis | file | none

[cache.redis]
url = "redis://localhost:6379"
key_prefix = "sql-cli"
max_value_size = "100MB"
compression = true  # Compress before storing

[cache.ttl]
# Default TTLs by pattern
default = 3600  # 1 hour
historical_data = 86400  # 24 hours
current_data = 300  # 5 minutes
```

## Smart Features with Redis

### 1. Cache Warming
```rust
// Pre-fetch common queries in background
async fn warm_cache() {
    let common_queries = vec![
        ("Barclays", "2025-09-01"),
        ("Bloomberg", "2025-09-01"),
    ];

    for (source, date) in common_queries {
        // Fetch and cache in background
        tokio::spawn(async move {
            fetch_and_cache(source, date).await;
        });
    }
}
```

### 2. Cache Statistics
```lua
-- Redis Lua script for stats
local keys = redis.call('keys', 'sql-cli:*')
local stats = {}
for i=1,#keys do
    local ttl = redis.call('ttl', keys[i])
    local size = redis.call('memory', 'usage', keys[i])
    table.insert(stats, {keys[i], ttl, size})
end
return stats
```

### 3. Pub/Sub for Cache Invalidation
```rust
// Subscribe to cache invalidation events
let mut pubsub = redis.conn.as_pubsub();
pubsub.subscribe("sql-cli:invalidate")?;

// Invalidate across all clients
redis.conn.publish("sql-cli:invalidate", "trades:barclays:*")?;
```

## Why This is Simpler Than Building Our Own

1. **No file management**: Redis handles storage
2. **No TTL logic**: Redis expires automatically
3. **No serialization format debates**: Just store Parquet bytes
4. **No locking**: Redis handles concurrent access
5. **No cleanup**: Redis LRU eviction when full
6. **Debugging**: RedisInsight shows cache contents visually

## Your Workflow with Redis

```bash
# First query - fetches from prod (30s)
sql-cli -q "WITH WEB trades AS (
    URL 'https://prod/trades'
    BODY '{\"source\": \"Barclays\", \"date\": \"2025-09-01\"}'
    CACHE 86400  -- 24 hours for yesterday's data
) SELECT * FROM trades"

# Second query - instant (from Redis)
# Same query runs immediately

# Check what's cached
redis-cli --scan --pattern "sql-cli:*"

# Monitor cache hits
redis-cli MONITOR | grep sql-cli
```

## Deployment Options

### Local Development
```bash
# Just run Redis locally
docker run -p 6379:6379 redis:alpine
```

### Production
```bash
# Use existing Redis instance
# Or Redis Cloud, AWS ElastiCache, etc.
```

## Minimal Code Change

```rust
// Just add to WebDataFetcher::fetch()
if let Some(cached) = RedisCache::get(&cache_key) {
    return Ok(cached);
}

// After fetch
RedisCache::set(&cache_key, &data, ttl)?;
```

This is SO much simpler than building our own caching layer!