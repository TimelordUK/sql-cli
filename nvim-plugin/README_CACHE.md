# Redis Cache Support in Neovim Plugin

## Enabling Cache in Neovim

The nvim plugin now shows cache status with visual notifications!

### Setup

In your nvim config:

```lua
require('sql-cli').setup({
  -- Your existing config...

  -- Enable Redis cache
  cache = {
    enabled = true,  -- Enable cache support
    redis_url = nil, -- Optional: custom Redis URL (default: localhost:6379)
    show_notifications = true, -- Show cache hit/miss notifications
  },
})
```

### What You'll See

When cache is enabled and you execute a Web CTE query:

#### Cache MISS (first execution):
- **Notification**: 🔍 Cache MISS for trades (key: sql-cli:web:9a5b0f087d...)
- **Output Buffer**: ⚠️ Cache MISS for trades (key: sql-cli:web:9a5b0f087d...)
- **Then**: 💾 Cached trades for 300 seconds

#### Cache HIT (subsequent executions):
- **Notification**: 🚀 Cache HIT for trades (key: sql-cli:web:9a5b0f087d...)
- **Output Buffer**: ✅ Cache HIT for trades (key: sql-cli:web:9a5b0f087d...)
- Query runs instantly!

### Visual Indicators

The plugin uses emojis to make cache status clear:
- 🚀 **Cache HIT** - Data retrieved from Redis (fast!)
- 🔍 **Cache MISS** - Fetching from network
- 💾 **Cached** - Data was stored in cache
- ⚠️ **Warning** - Cache miss in output buffer

### Example Workflow

1. Execute query with `\sx`:
```sql
WITH WEB trades AS (
    URL 'https://api/trades'
    CACHE 3600
) SELECT * FROM trades
```

2. First run shows:
```
-- SQL CLI Output --
⚠️ Cache MISS for trades (key: sql-cli:web:9a5b0f087d...)
💾 Cached trades for 3600 seconds
[Results...]
```

3. Second run shows:
```
-- SQL CLI Output --
✅ Cache HIT for trades (key: sql-cli:web:9a5b0f087d...)
[Results instantly!]
```

### Checking Cache Status

From nvim, run in terminal mode:
```vim
:terminal redis-cli KEYS "sql-cli:*"
```

Or use the helper script:
```vim
:!./scripts/check_redis_cache.sh
```

### Clearing Cache

To force refresh, clear from terminal:
```vim
:!redis-cli --scan --pattern "sql-cli:*" | xargs redis-cli DEL
```

### Alternative: Environment Variable

If you prefer not to modify nvim config, set environment before starting nvim:
```bash
export SQL_CLI_CACHE=true
nvim your_query.sql
```

## Benefits

- **Visual Feedback**: Instant notification of cache hits/misses
- **Performance Monitoring**: See exactly when cache is helping
- **Easy Debugging**: Cache key shown for troubleshooting
- **Seamless Integration**: Works with all existing keybindings

The cache makes iterating on complex production queries incredibly fast!