# Minimal Redis Integration for SQL CLI

## The Plan
- sql-cli remains transient (start → check cache → run → exit)
- Redis runs in Docker as persistent cache
- Cache key = SHA256 hash of (URL + Method + Headers + Body)
- Changes limited to `WebDataFetcher::fetch()` only

## 1. Start Redis (at work)
```bash
# Run Redis in Docker
docker run -d \
  --name sql-cli-cache \
  -p 6379:6379 \
  -v redis-data:/data \
  redis:alpine \
  redis-server --appendonly yes

# Verify it's running
docker ps
redis-cli ping  # Should return PONG
```

## 2. Minimal Changes to sql-cli

### Add Redis dependency
```toml
# Cargo.toml
[dependencies]
redis = { version = "0.23", optional = true }
sha2 = "0.10"

[features]
default = ["redis-cache"]
redis-cache = ["redis"]
```

### Add cache module
```rust
// src/cache/mod.rs
#[cfg(feature = "redis-cache")]
pub mod redis_cache;

// src/cache/redis_cache.rs
use redis::{Client, Commands};
use sha2::{Sha256, Digest};
use crate::data::datatable::DataTable;

pub struct RedisCache {
    enabled: bool,
    client: Option<Client>,
}

impl RedisCache {
    pub fn new() -> Self {
        // Try to connect, but don't fail if Redis isn't available
        let client = Client::open("redis://127.0.0.1:6379").ok();

        Self {
            enabled: client.is_some() && std::env::var("SQL_CLI_NO_CACHE").is_err(),
            client,
        }
    }

    pub fn generate_key(url: &str, method: &str, headers: &[(String, String)], body: Option<&str>) -> String {
        let mut hasher = Sha256::new();

        // Hash all the components that make this request unique
        hasher.update(url.as_bytes());
        hasher.update(method.as_bytes());

        for (k, v) in headers {
            hasher.update(k.as_bytes());
            hasher.update(v.as_bytes());
        }

        if let Some(body) = body {
            hasher.update(body.as_bytes());
        }

        format!("sql-cli:web:{:x}", hasher.finalize())
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        if !self.enabled {
            return None;
        }

        let mut conn = self.client.as_ref()?.get_connection().ok()?;
        conn.get(key).ok()
    }

    pub fn set(&self, key: &str, data: &[u8], ttl_seconds: u64) -> Result<(), redis::RedisError> {
        if !self.enabled {
            return Ok(());
        }

        let mut conn = self.client.as_ref()
            .ok_or_else(|| redis::RedisError::from((redis::ErrorKind::IoError, "No client")))?
            .get_connection()?;

        conn.set_ex(key, data, ttl_seconds as usize)?;
        Ok(())
    }
}
```

### Modify WebDataFetcher (the ONLY change to existing code)
```rust
// src/web/http_fetcher.rs

impl WebDataFetcher {
    pub fn fetch(&self, spec: &WebCTESpec, table_name: &str) -> Result<DataTable> {
        // NEW: Generate cache key from spec
        let cache_key = RedisCache::generate_key(
            &spec.url,
            &format!("{:?}", spec.method.as_ref().unwrap_or(&HttpMethod::GET)),
            &spec.headers,
            spec.body.as_deref(),
        );

        // NEW: Try Redis cache first
        #[cfg(feature = "redis-cache")]
        {
            let cache = RedisCache::new();
            if let Some(cached_bytes) = cache.get(&cache_key) {
                // Deserialize from Parquet bytes
                if let Ok(table) = DataTable::from_parquet_bytes(&cached_bytes) {
                    eprintln!("Cache HIT for {} (key: {}...)", table_name, &cache_key[0..16]);
                    return Ok(table);
                }
            }
            eprintln!("Cache MISS for {} (key: {}...)", table_name, &cache_key[0..16]);
        }

        // Existing fetch logic
        let data = if spec.url.starts_with("file://") {
            self.fetch_file(spec, table_name)?
        } else {
            self.fetch_http(spec, table_name)?
        };

        // NEW: Store in cache
        #[cfg(feature = "redis-cache")]
        {
            let ttl = spec.cache_seconds.unwrap_or_else(|| {
                // Smart default: 1 hour for prod, 5 min for staging
                if spec.url.contains("prod") {
                    3600
                } else {
                    300
                }
            });

            if let Ok(parquet_bytes) = data.to_parquet_bytes() {
                let cache = RedisCache::new();
                let _ = cache.set(&cache_key, &parquet_bytes, ttl);
                eprintln!("Cached {} for {} seconds", table_name, ttl);
            }
        }

        Ok(data)
    }
}
```

### Add Parquet serialization to DataTable
```rust
// src/data/datatable.rs

impl DataTable {
    pub fn to_parquet_bytes(&self) -> Result<Vec<u8>> {
        // Use arrow2 to serialize to Parquet
        let mut buffer = Vec::new();
        // ... parquet serialization logic ...
        Ok(buffer)
    }

    pub fn from_parquet_bytes(bytes: &[u8]) -> Result<Self> {
        // Deserialize from Parquet
        // ... parquet deserialization logic ...
    }
}
```

## 3. How It Works in Practice

```sql
-- First execution (Monday morning)
WITH WEB trades AS (
    URL 'https://prod.api/trades'
    METHOD POST
    HEADERS ('Authorization': 'Bearer token')
    BODY '{"source": "Barclays", "date": "2025-09-01"}'
    CACHE 86400  -- 24 hours
)
SELECT * FROM trades WHERE amount > 1000000;

-- Console output:
-- Cache MISS for trades (key: a3f4b2c1d5e7...)
-- Fetching from https://prod.api/trades...
-- Cached trades for 86400 seconds
-- [Results displayed]

-- Second execution (any time that day)
-- Same query runs instantly

-- Console output:
-- Cache HIT for trades (key: a3f4b2c1d5e7...)
-- [Results displayed immediately]
```

## 4. Cache Management

```bash
# Check what's cached
redis-cli
> KEYS sql-cli:*
1) "sql-cli:web:a3f4b2c1d5e7..."
2) "sql-cli:web:b5c6d8e9f0a1..."

# Check TTL
> TTL "sql-cli:web:a3f4b2c1d5e7..."
(integer) 85234

# Manual clear if needed
> DEL "sql-cli:web:a3f4b2c1d5e7..."

# Clear all sql-cli cache
> EVAL "return redis.call('del', unpack(redis.call('keys', 'sql-cli:*')))" 0

# Monitor cache activity
> MONITOR
```

## 5. Environment Control

```bash
# Normal operation (cache enabled)
sql-cli query.sql

# Bypass cache for testing
SQL_CLI_NO_CACHE=1 sql-cli query.sql

# Different Redis host
REDIS_URL=redis://other-host:6379 sql-cli query.sql
```

## Key Design Decisions

1. **Graceful Degradation**: If Redis isn't running, sql-cli still works (just no cache)
2. **Minimal Changes**: Only touches `WebDataFetcher::fetch()`
3. **Smart Keys**: Hash includes all request components that affect the response
4. **Optional Feature**: Can compile without Redis support if needed
5. **Observable**: stderr shows cache hits/misses for debugging

## What This Gives You

- **Zero config**: Just run Redis and it works
- **Shared cache**: All sql-cli invocations share the cache
- **Persistent**: Cache survives sql-cli restarts
- **Fast**: Parquet bytes in Redis = millisecond retrieval
- **Simple**: ~100 lines of code total

## Next Steps

1. Add Redis to Cargo.toml
2. Create `src/cache/redis_cache.rs`
3. Add cache check to `WebDataFetcher::fetch()`
4. Add Parquet serialization helpers
5. Test with your Barclays/Bloomberg queries

This keeps sql-cli simple and stable while giving you powerful caching!