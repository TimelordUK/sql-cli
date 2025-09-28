# Hybrid Cache Proxy Architecture

## Core Principle
Keep sql-cli simple and stateless, add intelligence via optional proxy service.

## Architecture Options

### Option 1: Transparent Cache Proxy (Recommended)
```
┌─────────────┐     ┌──────────────────┐     ┌─────────────┐
│ nvim Plugin │────▶│  Cache Proxy     │────▶│   sql-cli   │
└─────────────┘     │  (.NET/Rust)     │     │  (simple)   │
                    └──────────────────┘     └─────────────┘
                             │
                    ┌────────▼─────────┐
                    │   Cache Store    │
                    │  (Parquet files) │
                    └──────────────────┘
```

**How it works:**
1. nvim sends query to proxy (if running) or directly to sql-cli
2. Proxy checks cache for Web CTE results
3. If cached: rewrites query to use local files
4. If not: passes through, caches response
5. sql-cli stays completely stateless

### Option 2: Smart Rewriter Proxy
```
Original Query:
WITH WEB trades AS (
    URL 'https://prod/trades'
    BODY '{"source": "Barclays", "date": "2025-09-01"}'
)
SELECT * FROM trades WHERE amount > 1000000

↓ Proxy Rewrites To ↓

WITH trades AS (
    SELECT * FROM '/cache/2025-09-01/barclays_trades.parquet'
)
SELECT * FROM trades WHERE amount > 1000000
```

## Implementation: .NET Cache Proxy

```csharp
// Simple .NET Core service
public class SqlCliCacheProxy
{
    private readonly ICache cache;
    private readonly HttpClient httpClient;

    public async Task<QueryResult> ExecuteQuery(string query)
    {
        // 1. Parse for Web CTEs
        var webCtes = ParseWebCTEs(query);

        // 2. Check cache for each
        var rewrites = new Dictionary<string, string>();
        foreach (var cte in webCtes)
        {
            var cacheKey = GenerateCacheKey(cte);
            var cachedFile = await cache.GetFilePathAsync(cacheKey);

            if (cachedFile != null)
            {
                // Rewrite to local file
                rewrites[cte.Name] = $"SELECT * FROM '{cachedFile}'";
            }
            else
            {
                // Fetch and cache
                var data = await FetchWebData(cte);
                cachedFile = await cache.StoreAsync(cacheKey, data);
                rewrites[cte.Name] = $"SELECT * FROM '{cachedFile}'";
            }
        }

        // 3. Rewrite query
        var rewrittenQuery = ApplyRewrites(query, rewrites);

        // 4. Execute with sql-cli
        return await ExecuteSqlCli(rewrittenQuery);
    }
}
```

## Simpler Option: Shell Script Proxy

```bash
#!/bin/bash
# sql-cli-cached

CACHE_DIR=~/.cache/sql-cli/tables
QUERY="$1"

# Hash the query
CACHE_KEY=$(echo "$QUERY" | sha256sum | cut -d' ' -f1)
CACHE_FILE="$CACHE_DIR/$CACHE_KEY.parquet"

# Check cache
if [[ -f "$CACHE_FILE" ]]; then
    # Rewrite query to use cached file
    REWRITTEN=$(echo "$QUERY" | sed "s|URL.*)|SELECT * FROM '$CACHE_FILE'|")
    sql-cli -q "$REWRITTEN"
else
    # Execute and cache
    sql-cli -q "$QUERY" -o parquet > "$CACHE_FILE"
    cat "$CACHE_FILE" | sql-cli -f parquet
fi
```

## Rust-based Lightweight Proxy

```rust
// Minimal proxy that just handles caching
// Could be part of sql-cli with --proxy flag

use axum::{Router, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct QueryRequest {
    query: String,
    cache_hint: Option<String>,
}

async fn execute_query(Json(req): Json<QueryRequest>) -> Json<QueryResult> {
    // 1. Check if query has Web CTEs
    if let Some(web_ctes) = extract_web_ctes(&req.query) {
        // 2. Check cache
        for cte in web_ctes {
            if let Some(cached_path) = cache.get(&cte.cache_key()) {
                // Rewrite to use file
                rewrite_cte_to_file(&mut query, &cte.name, &cached_path);
            } else {
                // Mark for caching after execution
                pending_cache.insert(cte.cache_key(), cte);
            }
        }
    }

    // 3. Execute (with sql-cli or embedded engine)
    let result = sql_cli_execute(&query)?;

    // 4. Cache any pending Web CTE results
    for (key, cte) in pending_cache {
        cache.store(key, &result.get_cte_data(&cte.name));
    }

    Json(result)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/query", post(execute_query))
        .route("/cache/status", get(cache_status))
        .route("/cache/clear", post(clear_cache));

    axum::Server::bind(&"127.0.0.1:7890")
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## Progressive Implementation Path

### Phase 1: File-based cache in sql-cli (Simple)
```bash
# Add cache flag to sql-cli
sql-cli query.sql --cache-dir ~/.cache/sql-cli

# sql-cli checks cache before fetching
# Minimal changes to existing code
```

### Phase 2: Optional Proxy Mode
```bash
# Start proxy (could be part of sql-cli binary)
sql-cli --proxy-mode --port 7890 &

# nvim plugin detects proxy and uses it
# Falls back to direct sql-cli if not running
```

### Phase 3: Intelligent Proxy
- Smarter cache invalidation
- Background refresh
- Query rewriting optimization
- Multi-user cache sharing

## Why This Architecture?

1. **sql-cli stays simple**: No daemon, no state, no complexity
2. **Optional complexity**: Proxy only when needed
3. **Technology flexibility**: Proxy can be .NET, Rust, Python, whatever
4. **Graceful degradation**: Works without proxy
5. **Clear separation**: sql-cli = execution, proxy = caching

## Quick Win Implementation

Start with the simplest thing that works:

```rust
// In sql-cli: Add cache check for Web CTEs only
impl WebDataFetcher {
    pub fn fetch_with_cache(&self, spec: &WebCTESpec) -> Result<DataTable> {
        // Check for cached parquet file
        if let Some(cache_path) = self.check_cache(spec) {
            return load_parquet(cache_path);
        }

        // Fetch from network
        let data = self.fetch(spec)?;

        // Save to cache
        if let Some(cache_path) = self.get_cache_path(spec) {
            save_as_parquet(&data, cache_path)?;
        }

        Ok(data)
    }
}
```

This gives you:
- Immediate caching benefit
- No architectural changes
- Can evolve to proxy later

## Your Workflow Optimized

```sql
-- First run: fetches and caches (30s)
WITH WEB barclays_trades AS (
    URL 'https://prod/trades'
    BODY '{"source": "Barclays", "date": "2025-09-01"}'
    CACHE '24h'  -- Yesterday's trades never change
)
SELECT * FROM barclays_trades WHERE amount > 1000000;

-- Subsequent runs: instant (cached)
-- Cache key includes date, so different days cached separately
```

## Decision Framework

Choose based on:

1. **Simple file cache in sql-cli** if:
   - Single user
   - Don't mind cache per invocation
   - Want minimal changes

2. **Lightweight Rust proxy** if:
   - Want persistent cache
   - Multiple nvim sessions
   - Prefer staying in Rust ecosystem

3. **.NET proxy service** if:
   - Need complex business logic
   - Integration with other .NET services
   - Team already uses .NET

4. **Full server mode** if:
   - Multiple users
   - Need query result sharing
   - Want maximum performance

My recommendation: Start with simple file cache in sql-cli, evolve to lightweight proxy when needed.