# Cache and LINQ Integration

## Overview

This feature allows you to cache API query results locally and then apply LINQ-style methods on the cached data. This solves the problem where the C# server doesn't understand LINQ method syntax like `.Length()`, `.StartsWith()`, etc.

## How It Works

1. **Execute Query on API**: First run a standard SQL query against the API
   ```sql
   SELECT * FROM trade_deal WHERE tradeDate > DateTime(2024, 1, 1)
   ```

2. **Cache the Results**: Save the results locally
   ```
   :cache save
   ```
   This creates a local JSON cache of the query results.

3. **Load Cached Data**: Load previously cached data
   ```
   :cache load 1
   ```
   This switches to "cache mode" where queries run against local data.

4. **Apply LINQ Methods**: Now you can use LINQ methods that work locally
   ```sql
   SELECT * FROM cached_data WHERE counterparty.Length() > 10
   SELECT * FROM cached_data WHERE instrumentName.StartsWith('A')
   SELECT * FROM cached_data WHERE book.EndsWith('Trading')
   ```

## Commands

### Cache Management
- `:cache save [description]` - Save current query results to cache
- `:cache load <id>` - Load cached query by ID (enables cache mode)
- `:cache list` - Show all cached queries
- `:cache clear` - Exit cache mode and return to API mode
- `F7` - Quick access to cache list

### Status Indicators
- `[CACHE MODE]` appears in status bar when using cached data
- Cache ID shown when cached data is loaded

## Example Workflow

```bash
# 1. Start the TUI
$ sql-cli

# 2. Query the API for a large dataset
sql> SELECT * FROM trade_deal WHERE tradeDate > DateTime(2024, 1, 1)
# Returns: 8,543 rows (1250ms)

# 3. Save to cache
sql> :cache save January 2024 trades
# Output: Query cached with ID: 1 (8543 rows)

# 4. Load the cached data
sql> :cache load 1
# Output: Loaded cache ID 1 with 8543 rows. Cache mode enabled.

# 5. Now apply LINQ methods locally (instant!)
sql> SELECT * FROM cached_data WHERE counterparty.Contains("Bank") AND instrumentName.Length() > 5
# Returns: 2,341 rows (0ms) [CACHED]

# 6. Try complex LINQ queries
sql> SELECT * FROM cached_data WHERE book.StartsWith("Equity") AND counterparty.Length() < 20
# Returns: 567 rows (0ms) [CACHED]
```

## Benefits

1. **Performance**: Local queries are instant, no network latency
2. **LINQ Support**: Full LINQ method support without server changes
3. **Cost Savings**: One API call, unlimited local analysis
4. **Offline Work**: Continue analysis without server connection
5. **Complex Queries**: Use methods the server doesn't support

## Technical Details

- Cache stored in `~/.sql-cli/cache/`
- Data saved as JSON with metadata
- Each cache entry tracks:
  - Original query
  - Row count
  - Timestamp
  - Optional description
- Cache persists between sessions

## Supported LINQ Methods

When in cache mode, the following LINQ methods work:
- `.Contains(string)` - Substring search
- `.StartsWith(string)` - Prefix match
- `.EndsWith(string)` - Suffix match
- `.Length()` - String length comparison
- More methods can be added as needed

## Notes

- The table name in cache mode is always `cached_data`
- Schema/columns are automatically detected from cached JSON
- Tab completion works with cached column names
- All filtering happens locally using the Rust LINQ parser