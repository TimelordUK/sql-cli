# Generalized Cache Architecture for SQL CLI

## Vision
A unified caching system that can cache at multiple levels of the query execution pipeline, not just Web CTEs.

## Cache Levels

### 1. Result Cache (Highest Level)
Cache entire query results based on the full query hash.

```sql
-- This entire query result could be cached
SELECT * FROM trades WHERE date = '2024-03-26';
-- Cache key: hash(query + data_source + parameters)
```

### 2. CTE Cache (Current Focus)
Cache individual CTE results - both Web and Standard CTEs.

```sql
WITH
  -- Web CTE (external data)
  web_trades AS (
    URL 'https://api/trades'
    CACHE '1h'
  ),
  -- Standard CTE (could also be cached)
  filtered_trades AS (
    SELECT * FROM web_trades WHERE amount > 1000000
    -- CACHE '30m'  -- Future: cache derived CTEs too
  )
SELECT * FROM filtered_trades;
```

### 3. Subquery Cache
Cache frequently used subqueries.

```sql
SELECT * FROM orders
WHERE customer_id IN (
  -- This subquery result could be cached
  SELECT id FROM customers WHERE country = 'US'
);
```

### 4. Expression Cache
Cache expensive computed expressions.

```sql
SELECT
  -- Expensive calculation cached at expression level
  COMPLEX_CALCULATION(col1, col2, col3) as result
FROM large_table;
```

## Unified Cache Interface

```rust
// src/cache/mod.rs
pub trait CacheableResult: Send + Sync {
    fn cache_key(&self) -> String;
    fn to_bytes(&self) -> Result<Vec<u8>>;
    fn from_bytes(bytes: &[u8]) -> Result<Self> where Self: Sized;
    fn cache_hint(&self) -> CacheHint;
}

pub enum CacheHint {
    NoCache,
    Duration(Duration),
    Persistent,
    Smart(SmartCacheRules),
}

pub struct UnifiedCache {
    storage: Box<dyn CacheStorage>,
    policy: CachePolicy,
}

impl UnifiedCache {
    pub fn get<T: CacheableResult>(&self, key: &str) -> Option<T> {
        // Check all cache levels
        self.storage.get(key)
            .and_then(|bytes| T::from_bytes(&bytes).ok())
    }

    pub fn put<T: CacheableResult>(&self, item: &T) -> Result<()> {
        let key = item.cache_key();
        let bytes = item.to_bytes()?;
        let hint = item.cache_hint();

        self.storage.put(&key, bytes, hint)
    }
}
```

## Cache Storage Backends

### 1. File-Based Cache (Current)
```rust
pub struct FileCache {
    base_dir: PathBuf,
    format: StorageFormat, // Parquet, Arrow, MessagePack
}
```

### 2. In-Memory Cache (Server Mode)
```rust
pub struct MemoryCache {
    data: Arc<DashMap<String, CachedItem>>,
    max_size: usize,
    eviction_policy: EvictionPolicy,
}
```

### 3. Hybrid Cache
```rust
pub struct HybridCache {
    memory: MemoryCache,  // L1 - hot data
    file: FileCache,      // L2 - warm data
    archive: S3Cache,     // L3 - cold data (future)
}
```

## Server Mode Architecture

Your idea about running sql-cli in server mode is brilliant!

```
┌─────────────────────┐     ┌─────────────────────┐
│   nvim Plugin       │────▶│  SQL CLI Server     │
│                     │     │  (Persistent Cache) │
└─────────────────────┘     └─────────────────────┘
                                      │
┌─────────────────────┐              ▼
│   CLI Client        │────▶ ┌─────────────────┐
│                     │      │  Shared Cache    │
└─────────────────────┘      │  (In-Memory)     │
                             └─────────────────┘
```

### Benefits:
1. **Persistent Cache**: Cache survives between CLI invocations
2. **Shared Cache**: Multiple clients share the same cache
3. **Background Refresh**: Server can refresh cache in background
4. **Memory Efficiency**: One cache instance for all clients

### Implementation:
```rust
// Server mode
sql-cli --server --port 7890 --cache-size 1GB

// Client connects to server
sql-cli --connect localhost:7890 query.sql

// Or direct query
sql-cli --connect localhost:7890 -q "SELECT * FROM trades"
```

## Cache Placement in Query Pipeline

```rust
// src/data/query_engine.rs
impl QueryEngine {
    pub fn execute_with_cache(&self, query: &str) -> Result<DataTable> {
        // Level 1: Check result cache
        let query_key = self.hash_query(query);
        if let Some(result) = self.cache.get_result(&query_key) {
            return Ok(result);
        }

        // Parse query
        let statement = self.parse(query)?;

        // Level 2: Check CTE cache
        let ctes = self.process_ctes_with_cache(&statement.ctes)?;

        // Level 3: Check subquery cache
        let subqueries = self.process_subqueries_with_cache(&statement)?;

        // Execute with cached components
        let result = self.execute_with_components(statement, ctes, subqueries)?;

        // Store in result cache
        self.cache.put_result(&query_key, &result)?;

        Ok(result)
    }
}
```

## Cache Metadata System Table

```sql
-- Query all cache levels
SELECT * FROM __cache__ WHERE level = 'CTE';

-- Results show different cache levels
level  | key      | source           | created_at | hit_count | size_mb
-------|----------|------------------|------------|-----------|--------
RESULT | a3f4b2c1 | full_query       | 2024-03-27 | 45        | 12.3
CTE    | b5c6d7e8 | web_trades       | 2024-03-27 | 123       | 8.7
CTE    | c9d0e1f2 | filtered_trades  | 2024-03-27 | 89        | 3.2
SUBQ   | d3e4f5g6 | customer_filter  | 2024-03-27 | 234       | 0.5

-- Clear specific cache
DELETE FROM __cache__ WHERE key = 'a3f4b2c1';

-- Clear all CTE cache
DELETE FROM __cache__ WHERE level = 'CTE';
```

## Progressive Implementation Plan

### Phase 1: Web CTE Cache (Current Focus)
- Implement basic file-based cache for Web CTEs
- Add CACHE directive support
- Simple key generation from URL + body

### Phase 2: Generalize to All CTEs
- Cache standard CTEs that are expensive
- Add cache hints to CTE syntax
- Implement smart expiry rules

### Phase 3: Server Mode
- Add --server flag to run as daemon
- Implement client-server protocol
- Shared in-memory cache

### Phase 4: Multi-Level Caching
- Result cache for full queries
- Subquery cache
- Expression cache for complex calculations

### Phase 5: Advanced Features
- Background cache refresh
- Cache warming from query logs
- Distributed cache (Redis backend)

## Configuration

```toml
# ~/.config/sql-cli/cache.toml
[cache]
enabled = true
default_ttl = "1h"

[cache.levels]
result = true
cte = true
subquery = false  # Not yet implemented
expression = false  # Not yet implemented

[cache.storage]
type = "hybrid"  # file | memory | hybrid
memory_size = "256MB"
file_dir = "~/.cache/sql-cli"
compression = true

[cache.server]
enabled = false
port = 7890
max_connections = 10
background_refresh = true

[cache.rules]
# Smart caching rules
historical_data = "persistent"
today_data = "15m"
production_url = "1h"
staging_url = "5m"
```

## Key Design Decisions

1. **Pluggable Storage**: Different backends for different use cases
2. **Multi-Level**: Cache at different granularities
3. **Smart Defaults**: Automatic caching based on patterns
4. **Server Mode**: Optional persistent cache daemon
5. **Queryable**: Cache exposed as system table

## Why This Architecture?

1. **Future-Proof**: Can add cache at any level without refactoring
2. **Flexible**: File cache for single-user, server for teams
3. **Performant**: Multi-level cache hierarchy (L1/L2/L3)
4. **Observable**: System tables for cache introspection
5. **Extensible**: Easy to add new cache backends

The Web CTE cache becomes just one implementation of the `CacheableResult` trait, making it easy to add caching elsewhere without redesigning the system.