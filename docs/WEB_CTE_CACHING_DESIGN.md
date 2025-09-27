# Web CTE Caching System Design

## Problem Statement
When working with production data, repeatedly fetching the same dataset (e.g., 7,000 trades from yesterday) is:
- Wasteful of network bandwidth
- Puts unnecessary load on production servers
- Slows down query iteration and development
- Particularly painful when developing complex queries that require many iterations

## Proposed Solution
Implement a transparent caching layer for Web CTE queries that intelligently caches results based on query content and temporal characteristics.

## Cache Key Design

### Hash Components
```
cache_key = SHA256(
    URL +
    METHOD +
    HEADERS +
    BODY +
    JSON_PATH +
    DATE_CONTEXT  // For date-aware caching
)
```

### Cache Metadata
```json
{
  "query_hash": "sha256_hash",
  "url": "https://prod.api/trades",
  "timestamp": "2024-03-27T10:30:00Z",
  "expiry": "2024-03-27T11:30:00Z",
  "row_count": 7234,
  "size_bytes": 4567890,
  "hit_count": 15,
  "last_accessed": "2024-03-27T10:45:00Z"
}
```

## Implementation Strategy

### Option 1: Rust-Based Caching (Recommended)
Add cache support directly to sql-cli's Web CTE processor.

**Syntax Extension:**
```sql
WITH WEB trades_data AS (
    URL 'https://prod.api/trades'
    METHOD POST
    CACHE '1h'  -- Cache for 1 hour
    -- OR
    CACHE 'persistent'  -- Cache until explicitly cleared
    -- OR
    CACHE 'auto'  -- Use smart caching rules
    HEADERS (...)
    BODY '...'
)
```

**Advantages:**
- Works across all interfaces (CLI, TUI, nvim)
- Efficient storage for large datasets
- Can use fast serialization (bincode, MessagePack)
- Cache sharing between different tools

**Implementation Details:**
- Cache location: `~/.cache/sql-cli/web_cte/`
- Storage format: Parquet or Arrow for efficiency
- Automatic compression for large datasets
- Background cache cleanup process

### Option 2: Lua-Based Caching
Implement in nvim plugin before sending to sql-cli.

**Advantages:**
- Easier to prototype and iterate
- Better UI integration (show cache status)
- Can preview cached data without execution

**Disadvantages:**
- Only works in nvim
- Less efficient for large datasets
- No cache sharing

### Option 3: Hybrid Approach (Best of Both)
- Rust handles storage, retrieval, and expiry
- Lua provides UI and management features
- Shared cache between all interfaces

## Smart Caching Rules

### Automatic Cache Expiry

1. **Date-Based Queries**
   ```sql
   WHERE TradeDate = '2024-03-26'  -- Yesterday
   -- Cache: persistent (historical data won't change)

   WHERE TradeDate = CURRENT_DATE
   -- Cache: 5 minutes (today's data changes)
   ```

2. **URL-Based Rules**
   ```
   prod.api/* -> Default cache: 1 hour
   staging.api/* -> Default cache: 5 minutes
   localhost/* -> Default cache: disabled
   ```

3. **Time-Aware Caching**
   - If query includes "yesterday" or past dates: Cache persistently
   - If query includes "today": Cache briefly (5-15 minutes)
   - If query includes "real-time": Never cache

### Cache Directives

```sql
-- Explicit cache control
-- @cache: none           -- Bypass cache completely
-- @cache: 5m            -- Cache for 5 minutes
-- @cache: 1h            -- Cache for 1 hour
-- @cache: 1d            -- Cache for 1 day
-- @cache: persistent    -- Cache until manually cleared
-- @cache: auto          -- Use smart rules (default)

WITH WEB trades AS (
    URL 'https://prod.api/trades'
    -- Query automatically cached based on smart rules
)
```

## Cache Management UI (nvim plugin)

### Visual Indicators
```
[CACHED] Query using cached results (2 hours old, 7234 rows)
[LIVE]   Query fetching fresh data
[EXPIRED] Cache expired, fetching fresh data...
```

### Keybindings
- `\sc` - Show cache status for current query
  - Shows: age, size, hit count
  - Option to refresh

- `\sC` - Cache management menu
  - List all cached queries
  - Clear specific cache entries
  - Clear all cache
  - Show cache statistics

- `\sr` - Force refresh (bypass cache)

### Status Line Integration
```
[SQL] trades.sql [CACHED:2h] [7234 rows]
```

## Cache Storage Structure

```
~/.cache/sql-cli/web_cte/
├── index.json                    # Cache index with metadata
├── 2024-03-27/
│   ├── a3f4b2c1.parquet         # Cached data
│   ├── a3f4b2c1.meta.json       # Query metadata
│   └── d7e8f9a2.parquet
└── cleanup.log                   # Cleanup history
```

## CLI Integration

### New Flags
```bash
# Use cache (default behavior)
sql-cli query.sql --cache

# Bypass cache for this execution
sql-cli query.sql --no-cache

# Clear cache before execution
sql-cli query.sql --clear-cache

# Show cache statistics
sql-cli --cache-stats

# Clear all cache
sql-cli --clear-all-cache

# Set cache directory
sql-cli --cache-dir /custom/path
```

## Configuration

### nvim plugin config
```lua
web_cte_cache = {
  enabled = true,
  default_ttl = "1h",
  max_cache_size = "1GB",
  cache_dir = "~/.cache/sql-cli/web_cte",

  -- Smart rules
  smart_rules = {
    production_ttl = "1h",
    staging_ttl = "5m",
    localhost_ttl = "0",  -- Don't cache

    -- Date-based rules
    historical_data = "persistent",  -- Data from past dates
    current_date = "15m",            -- Today's data
  },

  -- Visual settings
  show_cache_status = true,
  show_in_statusline = true,
}
```

## Implementation Priority

### Phase 1: Basic Caching (MVP)
1. Implement cache key generation in Rust
2. Add simple file-based caching
3. Add `CACHE` directive to Web CTE
4. Basic cache/no-cache functionality

### Phase 2: Smart Caching
1. Implement date-aware caching rules
2. Add URL pattern matching
3. Automatic expiry based on query analysis

### Phase 3: UI Integration
1. nvim plugin cache status display
2. Cache management keybindings
3. Visual indicators and statusline

### Phase 4: Advanced Features
1. Cache compression for large datasets
2. Background cache cleanup
3. Cache sharing between users (team cache server)
4. Query result diffing (show what changed)

## Performance Targets
- Cache hit: < 10ms to load 10K rows
- Cache miss: Current performance (unchanged)
- Storage: ~10% of original size with compression
- Memory: Streaming to avoid loading full dataset

## Security Considerations
- Cache files should respect file permissions
- Option to encrypt cache for sensitive data
- Automatic cache clearing on logout/lock
- No caching for queries with credentials in body

## Next Session Action Plan

1. **Prototype cache key generation**
   - Hash the Web CTE components
   - Test with various query types

2. **Implement basic file caching in Rust**
   - Add CACHE directive parser
   - Save/load from Parquet files

3. **Add nvim UI for cache status**
   - Show when using cached data
   - Add force-refresh keybinding

4. **Test with production-like queries**
   - Measure performance improvement
   - Verify cache hit/miss logic

This caching system would provide massive productivity gains when working with production data, reducing both server load and development iteration time.