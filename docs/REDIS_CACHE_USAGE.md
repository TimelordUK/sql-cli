# Redis Cache Usage Guide

## Default Behavior: Cache is OFF

By default, sql-cli runs **without any caching** - exactly like it always has. No Redis connection is attempted, no performance penalty, no changes to existing workflows.

## Enabling Cache (Opt-In)

To use Redis caching, you must explicitly enable it:

```bash
# Enable cache for a single command
SQL_CLI_CACHE=true sql-cli query.sql

# Or export for your session
export SQL_CLI_CACHE=true
sql-cli query.sql
```

## Configuration Options

### 1. Basic Enable/Disable

```bash
# Enable cache (connects to Redis)
SQL_CLI_CACHE=true sql-cli query.sql

# Default behavior (no cache, no Redis)
sql-cli query.sql
```

### 2. Custom Redis Server

```bash
# Use different Redis host
SQL_CLI_CACHE=true SQL_CLI_REDIS_URL=redis://redis-server:6379 sql-cli query.sql
```

### 3. Shell Configuration

Add to your `.bashrc` or `.zshrc` if you always want caching:

```bash
# Always use cache when Redis is available
export SQL_CLI_CACHE=true

# Optional: custom Redis location
export SQL_CLI_REDIS_URL=redis://localhost:6379
```

## Usage Examples

### Default (No Cache)
```bash
# Works exactly as before - no Redis, no caching
./sql-cli -q "WITH WEB trades AS (URL 'https://api/trades' CACHE 300) SELECT * FROM trades"
# Always fetches from network, ignores CACHE directive
```

### With Cache Enabled
```bash
# First run: fetches and caches
SQL_CLI_CACHE=true ./sql-cli -q "WITH WEB trades AS (URL 'https://api/trades' CACHE 300) SELECT * FROM trades"
# Output: "Cached trades for 300 seconds"

# Second run: uses cache
SQL_CLI_CACHE=true ./sql-cli -q "WITH WEB trades AS (URL 'https://api/trades' CACHE 300) SELECT * FROM trades"
# Output: "Cache HIT for trades"
```

## Production Setup

### For Development/Testing
```bash
# Don't enable cache - test with real data
sql-cli query.sql
```

### For Production Queries
```bash
# In your production script
#!/bin/bash
export SQL_CLI_CACHE=true
export SQL_CLI_REDIS_URL=redis://prod-redis:6379

sql-cli production_queries.sql
```

### Personal Workstation
```bash
# Add to ~/.bashrc or ~/.zshrc
export SQL_CLI_CACHE=true  # Enable caching when available

# Install and start Redis
sudo apt install redis-server
sudo service redis-server start
```

## What Happens When...

### Redis is not running but cache is enabled?
- sql-cli tries to connect (1 second timeout)
- Connection fails silently
- Continues without caching
- No error messages or failures

### Cache is enabled but no CACHE directive in query?
- Query is not cached
- Only queries with explicit `CACHE n` directive are cached

### Multiple users with same Redis?
- Cache is shared
- Same query = same cache key
- Everyone benefits from cached data

## Performance Impact

### Without Cache (Default)
- No Redis connection attempt
- No performance overhead
- Exactly like sql-cli v1.54 and earlier

### With Cache Enabled
- 1 second max connection timeout to Redis
- ~1ms overhead to check cache
- 10-100x speedup for cached queries

## Migration Guide

### For Existing Users
No changes needed! sql-cli continues to work exactly as before.

### To Start Using Cache
1. Install Redis: `sudo apt install redis-server`
2. Start Redis: `sudo service redis-server start`
3. Enable cache: `export SQL_CLI_CACHE=true`
4. Add `CACHE` directives to Web CTEs that should be cached

## Summary

- **Default**: No cache, no Redis, no changes
- **Opt-in**: Set `SQL_CLI_CACHE=true` to enable
- **Graceful**: Works with or without Redis
- **Compatible**: Existing scripts continue to work unchanged

The cache is a pure performance enhancement that requires explicit opt-in, ensuring backward compatibility while providing massive speedups for those who need it.