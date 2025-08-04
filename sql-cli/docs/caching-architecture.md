# Query Caching Architecture

## Overview

To minimize expensive API calls to the trading platform, we'll implement a caching layer that allows:
1. One-time fetch of large datasets (10k+ records)
2. Local storage of query results
3. Fast local analysis using the TUI
4. Offline mode for working with cached data

## Architecture Components

### 1. Server-Side Proxy Endpoint

Add a new endpoint to your C# server that acts as a proxy to the platform's REST API:

```csharp
[HttpPost("api/trade/proxy")]
public async Task<IActionResult> ProxyQuery(
    [FromBody] ProxyRequest request,
    [FromHeader(Name = "X-Platform-Token")] string bearerToken)
{
    // Forward request to platform API with bearer token
    // Return results + cache metadata
}
```

### 2. CLI Cache Storage

Store cached queries in a local SQLite database or JSON files:

```
~/.sql-cli/
├── cache/
│   ├── cache.db          # SQLite for metadata
│   └── data/             # JSON files for actual data
│       ├── query_001.json
│       └── query_002.json
└── config.json
```

### 3. Cache Metadata Schema

```sql
CREATE TABLE cached_queries (
    id INTEGER PRIMARY KEY,
    query_hash TEXT UNIQUE,
    query_text TEXT,
    timestamp DATETIME,
    row_count INTEGER,
    file_path TEXT,
    description TEXT,
    expires_at DATETIME
);
```

### 4. CLI Commands

#### Cache Management Commands
```
\cache list              # List all cached queries
\cache load <id>         # Load cached query by ID
\cache save <desc>       # Save current results to cache
\cache delete <id>       # Delete cached query
\cache clear             # Clear all cache
\cache info              # Show cache statistics
```

#### Fetch Modes
```
\mode live               # Query server directly (default)
\mode cached             # Work with cached data only
\mode hybrid             # Check cache first, then server
```

#### Large Dataset Commands
```
\fetch 10000            # Fetch up to 10k records
\fetch all              # Fetch all records (warning!)
\fetch incremental 1000 # Fetch in chunks of 1000
```

## Implementation Plan

### Phase 1: Server Proxy
1. Add proxy endpoint to C# server
2. Handle bearer token authentication
3. Forward requests to platform API
4. Add response caching headers

### Phase 2: Local Cache Storage
1. Create cache directory structure
2. Implement SQLite metadata storage
3. Add JSON file storage for data
4. Create cache management module

### Phase 3: CLI Integration
1. Add cache commands to enhanced TUI
2. Implement offline mode
3. Add cache status to status bar
4. Create cache browser UI

### Phase 4: Advanced Features
1. Incremental fetching for huge datasets
2. Cache expiration and refresh
3. Compression for large JSON files
4. Query result diffing

## Usage Examples

### Initial Large Fetch
```sql
-- Fetch 10k records and cache them
\mode hybrid
\fetch 10000
SELECT * FROM trade_deal WHERE tradeDate > DateTime(2024, 01, 01)
\cache save "All 2024 trades"
```

### Working Offline
```sql
-- List available cached datasets
\cache list

-- Load cached dataset
\cache load 1

-- Now all queries run against cached data
SELECT * FROM trade_deal WHERE counterparty.Contains("Bank")
```

### Incremental Updates
```sql
-- Fetch only new records since last cache
\fetch incremental 1000
SELECT * FROM trade_deal WHERE createdDate > @last_cache_date
```

## Benefits

1. **Cost Reduction**: Minimize expensive API calls
2. **Performance**: Local queries are instant
3. **Offline Work**: Analyze data without connection
4. **Large Datasets**: Handle 10k+ records efficiently
5. **Flexibility**: Mix live and cached queries

## Technical Considerations

1. **Memory Management**: Stream large JSON files
2. **Compression**: Use gzip for cached data
3. **Security**: Encrypt sensitive cached data
4. **Cleanup**: Auto-expire old cache entries
5. **Sync**: Option to refresh cached data