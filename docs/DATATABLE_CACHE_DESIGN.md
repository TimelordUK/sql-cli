# DataTable Materialization Cache

## Core Concept
Cache at the single point where DataTables are materialized - whether from Web CTEs, file reads, subqueries, or computed CTEs. This gives us maximum leverage with minimal code changes.

## The Materialization Point

```rust
// Every DataTable creation flows through a materialization point
pub trait DataTableSource {
    fn materialize(&self) -> Result<DataTable>;
    fn cache_key(&self) -> Option<String>;  // None means don't cache
    fn cache_hint(&self) -> CacheHint;
}

// Unified materialization with caching
pub struct CachedMaterializer {
    cache: DataTableCache,
}

impl CachedMaterializer {
    pub fn materialize<S: DataTableSource>(&self, source: &S) -> Result<Arc<DataTable>> {
        // Check if cacheable
        if let Some(key) = source.cache_key() {
            // Try cache first
            if let Some(cached) = self.cache.get(&key) {
                return Ok(cached);
            }

            // Materialize from source
            let table = source.materialize()?;
            let arc_table = Arc::new(table);

            // Store in cache
            self.cache.put(&key, arc_table.clone(), source.cache_hint())?;

            return Ok(arc_table);
        }

        // Non-cacheable - just materialize
        Ok(Arc::new(source.materialize()?))
    }
}
```

## Implementation for Different Sources

### Web CTE Source
```rust
impl DataTableSource for WebCTESpec {
    fn materialize(&self) -> Result<DataTable> {
        WebDataFetcher::new().fetch(self, &self.name)
    }

    fn cache_key(&self) -> Option<String> {
        // Hash URL + method + headers + body
        Some(self.generate_cache_key())
    }

    fn cache_hint(&self) -> CacheHint {
        if let Some(seconds) = self.cache_seconds {
            CacheHint::Duration(Duration::from_secs(seconds))
        } else {
            CacheHint::Smart  // Use smart rules
        }
    }
}
```

### Standard CTE Source
```rust
impl DataTableSource for ComputedCTE {
    fn materialize(&self) -> Result<DataTable> {
        // Execute the CTE query
        self.engine.execute_select(&self.query)
    }

    fn cache_key(&self) -> Option<String> {
        // Only cache if explicitly marked or expensive
        if self.is_expensive() || self.has_cache_hint() {
            Some(self.generate_cache_key())
        } else {
            None
        }
    }

    fn cache_hint(&self) -> CacheHint {
        // CTEs are often reused in same query session
        CacheHint::Duration(Duration::from_secs(300))  // 5 min default
    }
}
```

### File Source
```rust
impl DataTableSource for FileSource {
    fn materialize(&self) -> Result<DataTable> {
        match self.format {
            FileFormat::CSV => load_csv(&self.path),
            FileFormat::JSON => load_json(&self.path),
            FileFormat::Parquet => load_parquet(&self.path),
        }
    }

    fn cache_key(&self) -> Option<String> {
        // Cache based on file path + modification time
        let metadata = std::fs::metadata(&self.path).ok()?;
        let modified = metadata.modified().ok()?;

        Some(format!("file:{}:{:?}", self.path.display(), modified))
    }

    fn cache_hint(&self) -> CacheHint {
        // Files are stable until modified
        CacheHint::UntilInvalidated
    }
}
```

## Cache Storage Layer

```rust
pub struct DataTableCache {
    storage: Box<dyn CacheStorage>,
    stats: CacheStatistics,
}

pub trait CacheStorage: Send + Sync {
    fn get(&self, key: &str) -> Option<Arc<DataTable>>;
    fn put(&self, key: &str, table: Arc<DataTable>, hint: CacheHint) -> Result<()>;
    fn invalidate(&self, key: &str) -> Result<()>;
    fn clear(&self) -> Result<()>;
    fn stats(&self) -> CacheStatistics;
}

// Different storage implementations
pub struct MemoryStorage {
    tables: Arc<DashMap<String, CachedTable>>,
    max_size: usize,
}

pub struct FileStorage {
    cache_dir: PathBuf,
    format: StorageFormat,  // Parquet, Arrow, etc.
}

pub struct HybridStorage {
    memory: MemoryStorage,  // L1 - hot
    file: FileStorage,      // L2 - warm
}
```

## Integration Points

### In Query Engine
```rust
// src/data/query_engine.rs
impl QueryEngine {
    fn execute_cte(&self, cte: &CTE) -> Result<DataView> {
        match &cte.cte_type {
            CTEType::Web(web_spec) => {
                // Use materializer instead of direct fetch
                let table = self.materializer.materialize(web_spec)?;
                Ok(DataView::new(table))
            }
            CTEType::Standard(query) => {
                // Wrap standard CTE for potential caching
                let source = ComputedCTE::new(query, self);
                let table = self.materializer.materialize(&source)?;
                Ok(DataView::new(table))
            }
        }
    }
}
```

## Cache as System Table

```sql
-- View all cached DataTables
SELECT * FROM __cache__;

-- Output:
source_type | cache_key | created_at          | expires_at          | rows  | size_mb | hits
------------|-----------|--------------------|--------------------|-------|---------|------
WEB_CTE     | a3f4b2c1  | 2024-03-27 10:30:00 | 2024-03-27 11:30:00 | 7234  | 12.5    | 45
STANDARD_CTE| b5c6d7e8  | 2024-03-27 10:35:00 | 2024-03-27 10:40:00 | 1500  | 3.2     | 12
FILE        | c9d0e1f2  | 2024-03-27 09:00:00 | NULL               | 50000 | 23.7    | 89

-- Inspect specific cached table
SELECT * FROM __cache_view__('a3f4b2c1') LIMIT 10;

-- Force refresh
DELETE FROM __cache__ WHERE cache_key = 'a3f4b2c1';
```

## Smart Cache Invalidation

```rust
impl DataTableCache {
    pub fn smart_invalidate(&self) {
        // Invalidate based on patterns

        // 1. Time-based: expired entries
        self.invalidate_expired();

        // 2. Size-based: when cache too large
        if self.total_size() > self.max_size {
            self.evict_lru();
        }

        // 3. Dependency-based: if source changed
        self.check_source_modifications();

        // 4. Pattern-based: e.g., "today's data" at midnight
        self.invalidate_by_pattern();
    }
}
```

## Benefits of DataTable-Level Caching

1. **Single Implementation Point**: All caching logic in one place
2. **Type Safety**: Always dealing with DataTables
3. **Efficient Storage**: Can use columnar formats (Parquet/Arrow)
4. **Memory Sharing**: Arc<DataTable> for zero-copy in-memory cache
5. **Transparent**: Sources don't need to know about caching
6. **Flexible**: Each source type can have its own cache strategy

## Minimal Code Changes Required

```rust
// Before (in query_engine.rs):
let data = fetcher.fetch(&web_spec, name)?;
DataView::new(Arc::new(data))

// After:
let data = self.materializer.materialize(&web_spec)?;
DataView::new(data)  // Already Arc'd
```

## Server Mode for Persistent Cache

```rust
// Start cache server
sql-cli --cache-server --port 7890

// Client automatically connects to cache server if running
// Cache persists across CLI invocations
sql-cli query.sql  // Uses cache server if available
```

## Configuration

```toml
[cache]
enabled = true
strategy = "hybrid"  # memory | file | hybrid

[cache.memory]
max_size = "512MB"
eviction = "lru"

[cache.file]
directory = "~/.cache/sql-cli/tables"
format = "parquet"
compression = "snappy"

[cache.server]
auto_start = false
port = 7890
```

This design treats the cache as a materialized DataTable store, sitting at the perfect interception point where all data sources converge into the common DataTable format.