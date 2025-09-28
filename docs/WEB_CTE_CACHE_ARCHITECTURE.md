# Web CTE Cache Architecture

## Overview
The cache sits between the Web CTE parser and the HTTP fetcher, transparently replacing network calls with cached data when appropriate.

## Architecture Layers

```
┌─────────────────────────────────┐
│         SQL Query               │
│  WITH WEB trades AS (...)       │
└──────────────┬──────────────────┘
               ▼
┌─────────────────────────────────┐
│      Parser (recursive_parser)   │
│   Parses CACHE directive         │
└──────────────┬──────────────────┘
               ▼
┌─────────────────────────────────┐
│    Query Engine (query_engine)   │
│   CTEType::Web execution point   │
└──────────────┬──────────────────┘
               ▼
┌─────────────────────────────────┐
│    Cache Layer (NEW)             │ ← Intercept here
│   Check cache before fetch       │
└──────────┬─────────┬────────────┘
           ▼         ▼
      Cache Hit   Cache Miss
           │         │
           ▼         ▼
    Return cached  WebDataFetcher
        data           │
                       ▼
                  Store in cache
                       │
                       ▼
                  Return data
```

## Implementation Points

### 1. Cache Interception Point
In `src/data/query_engine.rs` at lines 185-189 and 251-255:

```rust
CTEType::Web(web_spec) => {
    // NEW: Check cache first
    let cache_key = WebCteCache::generate_key(&web_spec);

    if let Some(cached_data) = WebCteCache::get(&cache_key, &web_spec) {
        // Return cached DataTable
        return DataView::new(Arc::new(cached_data));
    }

    // Cache miss - fetch from network
    let fetcher = WebDataFetcher::new();
    let data = fetcher.fetch(&web_spec, name)?;

    // Store in cache if caching enabled
    if web_spec.cache_seconds.is_some() {
        WebCteCache::store(&cache_key, &data, &web_spec)?;
    }

    DataView::new(Arc::new(data))
}
```

### 2. Cache Module Structure

```rust
// src/cache/mod.rs
pub mod web_cte_cache;

// src/cache/web_cte_cache.rs
pub struct WebCteCache {
    cache_dir: PathBuf,
}

impl WebCteCache {
    /// Generate deterministic cache key from Web CTE spec
    pub fn generate_key(spec: &WebCTESpec) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&spec.url);

        if let Some(method) = &spec.method {
            hasher.update(format!("{:?}", method));
        }

        if let Some(body) = &spec.body {
            hasher.update(body);
        }

        // Include headers in hash
        for (key, value) in &spec.headers {
            hasher.update(format!("{}:{}", key, value));
        }

        // Add temporal context for smart caching
        let temporal_hint = Self::extract_temporal_hint(spec);
        hasher.update(&temporal_hint);

        format!("{:x}", hasher.finalize())
    }

    /// Check if cached data exists and is valid
    pub fn get(key: &str, spec: &WebCTESpec) -> Option<DataTable> {
        let cache_path = Self::cache_path(key);

        if !cache_path.exists() {
            return None;
        }

        // Check expiry
        if Self::is_expired(&cache_path, spec)? {
            // Clean up expired cache
            let _ = std::fs::remove_file(&cache_path);
            return None;
        }

        // Load from Parquet
        Self::load_from_parquet(&cache_path).ok()
    }

    /// Store data in cache
    pub fn store(key: &str, data: &DataTable, spec: &WebCTESpec) -> Result<()> {
        let cache_path = Self::cache_path(key);

        // Create cache directory if needed
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Save as Parquet for efficiency
        Self::save_as_parquet(data, &cache_path)?;

        // Save metadata
        Self::save_metadata(key, spec)?;

        Ok(())
    }
}
```

### 3. Cache as Virtual System Table

We can expose the cache as a queryable system table:

```sql
-- Query the cache status
SELECT * FROM __cache__;

-- Results:
cache_key | url | created_at | expires_at | row_count | size_bytes | hit_count
----------|-----|------------|------------|-----------|------------|----------
a3f4b2c1  | ... | 2024-03-27 | 2024-03-28 | 7234      | 4567890   | 15
```

Implementation:
```rust
// In VirtualTableGenerator
if table_name.to_uppercase() == "__CACHE__" {
    return Ok(WebCteCache::as_system_table());
}
```

### 4. File URL Rewriting (Alternative Approach)

Instead of intercepting in query_engine, we could rewrite the Web CTE spec in the parser:

```rust
// In web_cte_parser.rs after parsing
if let Some(cached_file) = WebCteCache::get_cached_file_path(&spec) {
    // Rewrite to file:// URL
    spec.url = format!("file://{}", cached_file.display());
    spec.cache_seconds = None; // Don't re-cache
}
```

This way, the rest of the engine doesn't know data came from cache - it just sees a file:// URL.

## Cache Storage Format

### Directory Structure
```
~/.cache/sql-cli/web_cte/
├── 2024-03-27/
│   ├── a3f4b2c1/
│   │   ├── data.parquet       # Actual data
│   │   ├── metadata.json      # Query metadata
│   │   └── access.log         # Access history
│   └── b5c6d7e8/
│       └── ...
├── index.db                   # SQLite index for fast lookups
└── config.json                # Cache configuration
```

### Metadata Format
```json
{
  "cache_key": "a3f4b2c1...",
  "url": "https://prod.api/trades",
  "method": "POST",
  "headers": {...},
  "body": "...",
  "created_at": "2024-03-27T10:30:00Z",
  "expires_at": "2024-03-27T11:30:00Z",
  "cache_duration": 3600,
  "row_count": 7234,
  "size_bytes": 4567890,
  "hit_count": 15,
  "last_accessed": "2024-03-27T10:45:00Z",
  "temporal_hint": "historical"
}
```

## Smart Caching Rules Implementation

```rust
impl WebCteCache {
    fn extract_temporal_hint(spec: &WebCTESpec) -> String {
        // Check body for date patterns
        if let Some(body) = &spec.body {
            if Self::contains_yesterday_date(body) {
                return "historical".to_string();
            }
            if Self::contains_today_date(body) {
                return "current".to_string();
            }
        }

        // Check URL patterns
        if spec.url.contains("prod.") {
            return "production".to_string();
        }
        if spec.url.contains("staging.") {
            return "staging".to_string();
        }

        "default".to_string()
    }

    fn get_cache_duration(spec: &WebCTESpec) -> Option<u64> {
        // Explicit cache directive takes precedence
        if let Some(seconds) = spec.cache_seconds {
            return Some(seconds);
        }

        // Smart rules based on temporal hint
        match Self::extract_temporal_hint(spec).as_str() {
            "historical" => Some(u64::MAX), // Persistent
            "current" => Some(900),         // 15 minutes
            "production" => Some(3600),     // 1 hour
            "staging" => Some(300),         // 5 minutes
            _ => None,                      // No caching
        }
    }
}
```

## Advantages of This Architecture

1. **Transparent to Query Engine**: The cache layer sits cleanly between parsing and fetching
2. **File URL Option**: Can rewrite to file:// URLs for zero-copy optimization
3. **System Table Access**: Cache is queryable for debugging and management
4. **Smart Defaults**: Automatic caching based on query patterns
5. **Efficient Storage**: Parquet format for fast reads and compression

## Implementation Priority

1. **Phase 1**: Basic cache with explicit CACHE directive
2. **Phase 2**: Smart caching rules based on dates
3. **Phase 3**: System table interface
4. **Phase 4**: File URL rewriting optimization

## Next Steps

1. Create `src/cache/` module
2. Implement `WebCteCache` struct
3. Modify `query_engine.rs` to check cache
4. Add cache management CLI commands
5. Update nvim plugin to show cache status