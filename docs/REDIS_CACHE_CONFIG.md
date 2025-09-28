# Redis Cache Configuration & Graceful Degradation

## Configuration Hierarchy

Cache can be controlled at multiple levels (in order of precedence):

1. **Command-line flags** (highest priority)
2. **Environment variables**
3. **Config file**
4. **Auto-detection** (lowest priority)

## 1. Command-Line Control

```bash
# Explicitly enable cache (tries to connect to Redis)
sql-cli query.sql --cache

# Explicitly disable cache (never tries Redis)
sql-cli query.sql --no-cache

# Default: auto-detect (tries Redis, continues if not available)
sql-cli query.sql

# Specify Redis URL
sql-cli query.sql --cache --redis-url redis://localhost:6379

# Force cache refresh (bypass cache for this run, but update it)
sql-cli query.sql --cache-refresh
```

## 2. Environment Variables

```bash
# Enable/disable cache
export SQL_CLI_CACHE=true
export SQL_CLI_CACHE=false

# Redis connection
export SQL_CLI_REDIS_URL=redis://localhost:6379

# Default TTL (seconds)
export SQL_CLI_CACHE_TTL=3600

# Disable cache for one command
SQL_CLI_CACHE=false sql-cli query.sql
```

## 3. Config File

```toml
# ~/.config/sql-cli/config.toml

[cache]
enabled = "auto"  # auto | true | false
backend = "redis"  # redis | file | none

[cache.redis]
url = "redis://localhost:6379"
connect_timeout = 1  # seconds - fail fast if Redis not available
key_prefix = "sql-cli"
compression = true  # Compress before storing

[cache.ttl]
# Default TTLs in seconds
default = 3600  # 1 hour
web_cte = 3600  # 1 hour for Web CTEs
historical = 86400  # 24 hours for historical data
current = 300  # 5 minutes for current data
```

## 4. Implementation: Graceful Auto-Detection

```rust
// src/config/cache_config.rs
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub mode: CacheMode,
    pub redis_url: String,
    pub connect_timeout: Duration,
    pub compression: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheMode {
    Auto,     // Try Redis, continue if not available
    Enabled,  // Try Redis, warn if not available
    Disabled, // Never try Redis
}

impl CacheConfig {
    pub fn from_env_and_args(args: &Args) -> Self {
        // 1. Check command line args
        if args.no_cache {
            return Self {
                mode: CacheMode::Disabled,
                ..Default::default()
            };
        }

        if args.cache {
            return Self {
                mode: CacheMode::Enabled,
                redis_url: args.redis_url.clone()
                    .or_else(|| std::env::var("SQL_CLI_REDIS_URL").ok())
                    .unwrap_or_else(|| "redis://127.0.0.1:6379".to_string()),
                ..Default::default()
            };
        }

        // 2. Check environment variable
        if let Ok(cache_env) = std::env::var("SQL_CLI_CACHE") {
            let mode = match cache_env.to_lowercase().as_str() {
                "true" | "1" | "yes" => CacheMode::Enabled,
                "false" | "0" | "no" => CacheMode::Disabled,
                _ => CacheMode::Auto,
            };

            return Self {
                mode,
                ..Default::default()
            };
        }

        // 3. Load from config file
        if let Ok(config) = Self::from_config_file() {
            return config;
        }

        // 4. Default: auto-detect
        Self {
            mode: CacheMode::Auto,
            redis_url: "redis://127.0.0.1:6379".to_string(),
            connect_timeout: Duration::from_secs(1),  // Fail fast
            compression: true,
        }
    }
}

// src/cache/redis_cache.rs
impl RedisCache {
    pub fn new(config: &CacheConfig) -> Option<Self> {
        match config.mode {
            CacheMode::Disabled => {
                debug!("Cache disabled by configuration");
                return None;
            }
            CacheMode::Auto => {
                // Try to connect, but don't report errors
                match Self::try_connect(config) {
                    Ok(cache) => {
                        eprintln!("Redis cache enabled (auto-detected)");
                        Some(cache)
                    }
                    Err(_) => {
                        debug!("Redis not available, continuing without cache");
                        None
                    }
                }
            }
            CacheMode::Enabled => {
                // Try to connect, warn if fails
                match Self::try_connect(config) {
                    Ok(cache) => {
                        eprintln!("Redis cache enabled");
                        Some(cache)
                    }
                    Err(e) => {
                        eprintln!("Warning: Redis cache requested but not available: {}", e);
                        eprintln!("Continuing without cache...");
                        None
                    }
                }
            }
        }
    }

    fn try_connect(config: &CacheConfig) -> Result<Self> {
        // Try to connect with timeout
        let client = redis::Client::open(config.redis_url.as_str())?;

        // Test connection with timeout
        let mut conn = client.get_connection_with_timeout(config.connect_timeout)?;

        // Verify Redis is working
        redis::cmd("PING").query::<String>(&mut conn)?;

        Ok(Self {
            client: Some(client),
            compression: config.compression,
        })
    }
}

// src/web/http_fetcher.rs
impl WebDataFetcher {
    pub fn fetch(&self, spec: &WebCTESpec, table_name: &str) -> Result<DataTable> {
        // Get cache config from global context
        let cache_config = get_cache_config();

        // Try to create cache (returns None if disabled or unavailable)
        let cache = RedisCache::new(&cache_config);

        // Generate cache key
        let cache_key = Self::generate_cache_key(spec);

        // Try cache if available
        if let Some(ref cache) = cache {
            if let Some(data) = cache.get(&cache_key) {
                eprintln!("Cache HIT: {}", table_name);
                return Ok(data);
            }
            eprintln!("Cache MISS: {}", table_name);
        }

        // Fetch from network (existing code)
        let data = self.fetch_from_network(spec)?;

        // Store in cache if available
        if let Some(ref cache) = cache {
            let ttl = spec.cache_seconds.unwrap_or(cache_config.default_ttl);
            if let Err(e) = cache.set(&cache_key, &data, ttl) {
                debug!("Failed to cache: {}", e);
                // Continue - caching is best-effort
            }
        }

        Ok(data)
    }
}
```

## 5. Behavior Matrix

| Redis Status | Config Mode | Behavior |
|-------------|------------|----------|
| Running | Auto | ✅ Cache enabled silently |
| Running | Enabled | ✅ Cache enabled with message |
| Running | Disabled | ❌ No cache (by choice) |
| Not Running | Auto | ⚠️ Continue without cache (silent) |
| Not Running | Enabled | ⚠️ Warning, continue without cache |
| Not Running | Disabled | ❌ No cache (by choice) |

## 6. User Experience

### Default Experience (Auto Mode)
```bash
# Redis running - cache just works
$ sql-cli query.sql
[Results...]

# Redis not running - still works
$ sql-cli query.sql
[Results...]
```

### Explicit Cache Mode
```bash
# User wants cache
$ sql-cli query.sql --cache
Redis cache enabled
Cache MISS: trades
[Results after fetch...]

# Run again
$ sql-cli query.sql --cache
Redis cache enabled
Cache HIT: trades
[Instant results...]

# Redis not running
$ sql-cli query.sql --cache
Warning: Redis cache requested but not available: Connection refused
Continuing without cache...
[Results after fetch...]
```

### Debug Mode
```bash
# See what's happening
$ SQL_CLI_LOG=debug sql-cli query.sql
[DEBUG] Checking Redis at redis://127.0.0.1:6379
[DEBUG] Redis connection successful
[DEBUG] Cache key: sql-cli:web:a3f4b2c1...
[DEBUG] Cache MISS
[DEBUG] Fetching from https://prod.api/trades
[DEBUG] Storing in cache with TTL 3600
```

## 7. Testing Cache Behavior

```bash
# Test without Redis (should work)
docker stop sql-cli-cache
sql-cli query.sql  # Should work normally

# Test with Redis
docker start sql-cli-cache
sql-cli query.sql  # Should use cache

# Force no cache
sql-cli query.sql --no-cache

# Force cache refresh
sql-cli query.sql --cache-refresh
```

## Summary

- **Default**: Auto-detect Redis, use if available, continue if not
- **No changes needed**: Existing sql-cli usage works exactly the same
- **Opt-in improvements**: Add `--cache` for explicit caching
- **Always works**: sql-cli never fails due to Redis issues
- **Progressive enhancement**: Cache when available, normal operation when not

This way sql-cli remains the stable, simple tool it is today, with Redis as an optional turbo boost!