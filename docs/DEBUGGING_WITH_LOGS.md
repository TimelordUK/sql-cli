# Debugging SQL CLI Plugin with Logs

**Date**: 2025-10-04
**Purpose**: Guide for troubleshooting expand_star (`\sE`) and CTE testing (`\sC`) issues using comprehensive logging

## Overview

Both `\sE` (expand SELECT *) and `\sC` (test CTE) now have extensive logging to help debug issues with complex queries, especially:
- Queries with WEB CTEs
- FIX protocol uploads with selectors
- Complex nested CTEs
- Queries with multiple data sources

## Quick Start

### 1. Enable DEBUG Logging

In your Neovim config:

```lua
require('sql-cli').setup({
  logging = {
    enabled = true,
    level = "DEBUG",  -- Change from INFO to DEBUG for detailed logs
    max_files = 30,   -- Keep more logs while debugging
  },
})
```

### 2. Reproduce the Issue

```vim
" Open your complex query
:e my_complex_query.sql

" Try the failing operation
\sE    " Expand star
\sC    " Test CTE
```

### 3. View the Log

```vim
:SqlCliOpenLog    " Opens log in split
:SqlCliTailLog    " Live tail (refreshes every second)
```

Or from terminal:
```bash
# View latest log
cat ~/.local/share/sql-cli/logs/latest.log

# Tail live
tail -f ~/.local/share/sql-cli/logs/latest.log

# Search for errors
grep ERROR ~/.local/share/sql-cli/logs/nvim-plugin*.log
```

## What Gets Logged

### Expand Star (`\sE`) Logging

**Module**: `expand_star`

#### Entry Point
```
[HH:MM:SS] INFO [expand_star] === expand_star_smart called ===
[HH:MM:SS] DEBUG [expand_star] smart_expansion config: {...}
```

#### Query Extraction
```
[HH:MM:SS] DEBUG [expand_star] get_query_for_expansion: cursor at line 45, total lines: 200
[HH:MM:SS] DEBUG [expand_star] Found WITH at line 1, start_line set to 1
[HH:MM:SS] DEBUG [expand_star] Found GO at line 150, end_line set to 149
[HH:MM:SS] INFO [expand_star] Extracted query from lines 1-149 (149 lines)
[HH:MM:SS] DEBUG [expand_star] Query text: WITH WEB trades AS (URL 'http://...
```

**Key Information**:
- Cursor position
- Query boundaries (WHERE did it think the query starts/ends?)
- First 200 chars of extracted query

#### Query Validation
```
[HH:MM:SS] INFO [expand_star] Got query (length: 2543)
[HH:MM:SS] INFO [expand_star] Preview query (length: 2551): WITH WEB trades AS...
```

#### CLI Execution
```
[HH:MM:SS] DEBUG [expand_star] Command path: /home/user/sql-cli/target/release/sql-cli
[HH:MM:SS] INFO [expand_star] Executing query to get schema...
```

#### Result Processing
```
[HH:MM:SS] INFO [expand_star] Query executed successfully (exit code 0)
[HH:MM:SS] DEBUG [expand_star] Raw result length: 1234
[HH:MM:SS] DEBUG [expand_star] Raw result preview: [{"TradeID":"1234",...
[HH:MM:SS] DEBUG [expand_star] JSON after filtering comments (length: 1200): [{"TradeID":...
[HH:MM:SS] INFO [expand_star] Successfully parsed JSON result
[HH:MM:SS] DEBUG [expand_star] Data type: table
[HH:MM:SS] DEBUG [expand_star] Data has columns field: false
[HH:MM:SS] DEBUG [expand_star] Data array length: 1
[HH:MM:SS] INFO [expand_star] Using format 1 (array of objects)
[HH:MM:SS] INFO [expand_star] Extracted 25 columns: TradeID, Symbol, Price, ...
[HH:MM:SS] INFO [expand_star] === expand_star_smart completed successfully ===
```

#### Error Cases
```
[HH:MM:SS] ERROR [expand_star] get_query_for_expansion returned empty query
[HH:MM:SS] WARN [expand_star] No SELECT * found in query
[HH:MM:SS] ERROR [expand_star] Query execution failed with exit code 1
[HH:MM:SS] ERROR [expand_star] Error message: Could not find data file...
[HH:MM:SS] ERROR [expand_star] JSON decode failed: Expected value but found T_END at character 1
```

### CTE Testing (`\sC`) Logging

**Module**: `cte_test`

#### Entry Point
```
[HH:MM:SS] INFO [cte_test] === test_cte_at_cursor called ===
[HH:MM:SS] DEBUG [cte_test] Buffer: 3, cursor line: 45, total lines: 200
```

#### Query Boundaries
```
[HH:MM:SS] INFO [cte_test] Found query boundaries: lines 1-149 (149 lines)
[HH:MM:SS] DEBUG [cte_test] Query preview: WITH WEB trades AS (URL...
```

#### CTE Detection
Currently using vim.notify (visible to user) - these will be logged too:
```
[CTE PARSE] Searching for CTE #1 'trades'...
[CTE PARSE] ✓ Found CTE #1 'trades' at line 1 (matched: 'WITH WEB trades AS (')
[CTE DEBUG] Cursor at relative line 45 (absolute line 45)
[CTE DEBUG] Found 3 CTEs with start lines
[CTE DEBUG] CTE 1 (trades): starts at line 1
[CTE DEBUG] CTE 2 (filtered): starts at line 25
[CTE DEBUG] CTE 3 (summary): starts at line 40
[CTE DEBUG] → Cursor is at/after this CTE start (updating target to 3)
[CTE DEBUG] ✓ Final target_index: 3 (CTE: summary)
```

**Key Information**:
- Which CTE was detected at cursor
- How many CTEs found total
- Line numbers where each CTE starts
- Logic for determining target CTE

## Common Issues & What to Look For

### Issue: "Could not determine query context"

**Log Location**: Search for `expand_star`

**What to check**:
1. Query boundaries detection:
   ```
   grep "Found.*at line" ~/.local/share/sql-cli/logs/latest.log
   ```
   - Did it find the WITH keyword?
   - Did it find GO statements incorrectly?
   - Are the start/end lines correct?

2. Extracted query:
   ```
   grep "Extracted query from lines" ~/.local/share/sql-cli/logs/latest.log
   ```
   - Is the entire query captured?
   - Check the "Query preview" - does it show the right SQL?

**Common Causes**:
- GO statement in unexpected place
- WITH keyword not detected (spacing issues)
- Cursor outside query boundaries

### Issue: "No SELECT * found in query"

**Log Location**: `expand_star`

**What to check**:
```
grep "Query preview:" ~/.local/share/sql-cli/logs/latest.log
```

Look for:
- Is there actually a `SELECT *` in the extracted query?
- Case sensitivity issues (should match both SELECT and select)
- Extra whitespace breaking the pattern

### Issue: "Query execution failed"

**Log Location**: `expand_star`

**What to check**:
1. Exit code:
   ```
   grep "exit code" ~/.local/share/sql-cli/logs/latest.log
   ```

2. Error message:
   ```
   grep "Error message:" ~/.local/share/sql-cli/logs/latest.log
   ```

3. Preview query that was executed:
   ```
   grep "Preview query" ~/.local/share/sql-cli/logs/latest.log
   ```

**Common Causes**:
- WEB CTE needs authentication (missing token)
- Data file not set
- Syntax error in query
- LIMIT clause issues

### Issue: "No columns found in query result"

**Log Location**: `expand_star`

**What to check**:
1. Raw JSON result:
   ```
   grep "Raw result preview" ~/.local/share/sql-cli/logs/latest.log
   ```

2. JSON structure:
   ```
   grep "Data has columns field" ~/.local/share/sql-cli/logs/latest.log
   grep "Data array length" ~/.local/share/sql-cli/logs/latest.log
   ```

3. Format detection:
   ```
   grep "Using format" ~/.local/share/sql-cli/logs/latest.log
   ```

**Common Causes**:
- JSON format changed
- Empty result set
- JSON parsing failed

### Issue: "Could not find CTE at cursor" or "Unable to find CTE 1"

**Log Location**: `cte_test`

**What to check**:
1. How many CTEs were found:
   ```
   grep "Found.*CTEs with start lines" ~/.local/share/sql-cli/logs/latest.log
   ```

2. CTE detection:
   ```
   grep "CTE PARSE" ~/.local/share/sql-cli/logs/latest.log
   ```

3. Target selection:
   ```
   grep "Final target_index" ~/.local/share/sql-cli/logs/latest.log
   ```

**Common Causes**:
- CTE name pattern not matching (spaces, WEB keyword)
- Cursor position calculation wrong
- CTE start line detection failed

## Log Patterns for Complex Queries

### WEB CTE with Authentication

**Successful**:
```
[expand_star] Extracted query from lines 1-50
[expand_star] Query text: WITH WEB trades AS (URL 'http://api.example.com/trades'
[expand_star] Preview query (length: 1234): WITH WEB trades AS...LIMIT 1
[expand_star] Command path: /path/to/sql-cli
[expand_star] Executing query to get schema...
[expand_star] Query executed successfully (exit code 0)
[expand_star] Extracted 15 columns: TradeID, Symbol, Side, Price, ...
```

**Failed (Auth Issue)**:
```
[expand_star] Query execution failed with exit code 1
[expand_star] Error message: HTTP 401 Unauthorized
```

### FIX Protocol Selector

**Successful**:
```
[expand_star] Query text: WITH WEB fix_log AS (URL 'http://localhost:8080/upload'
[expand_star] Using format 1 (array of objects)
[expand_star] Extracted 8 columns: msg_type, symbol, isin, cusip, ...
```

### Nested CTEs

**Successful**:
```
[cte_test] Found query boundaries: lines 1-100 (100 lines)
[CTE PARSE] Searching for CTE #1 'raw_data'...
[CTE PARSE] ✓ Found CTE #1 'raw_data' at line 1
[CTE PARSE] Searching for CTE #2 'filtered'...
[CTE PARSE] ✓ Found CTE #2 'filtered' at line 20
[CTE PARSE] Searching for CTE #3 'aggregated'...
[CTE PARSE] ✓ Found CTE #3 'aggregated' at line 40
[CTE DEBUG] Cursor at relative line 45
[CTE DEBUG] ✓ Final target_index: 3 (CTE: aggregated)
```

## Extracting Logs for Bug Reports

When reporting an issue, include:

```bash
# Get last 100 lines of expand_star logs
grep "expand_star" ~/.local/share/sql-cli/logs/latest.log | tail -100 > expand_star_issue.log

# Get last 100 lines of cte_test logs
grep "cte_test" ~/.local/share/sql-cli/logs/latest.log | tail -100 > cte_test_issue.log

# Get all ERROR lines
grep "ERROR" ~/.local/share/sql-cli/logs/latest.log > errors.log
```

Or share the entire log file (sanitize sensitive data first):
```bash
# Copy log path
:SqlCliLogPath

# Sanitize and share
cat /path/to/log | sed 's/Bearer .*/Bearer REDACTED/' > sanitized_log.txt
```

## Performance Impact

Logging has minimal impact:
- **INFO level**: ~1-2ms overhead per operation
- **DEBUG level**: ~5-10ms overhead per operation
- Buffered writes (100 messages before flush)
- Auto-flush every 5 seconds

For production use, keep at INFO level. Use DEBUG only when debugging.

## Tips for Effective Debugging

1. **Set log level to DEBUG** before reproducing issue
2. **Clear old logs** or note the timestamp when you start
3. **Reproduce issue once** - logs can be verbose
4. **Search for ERROR first**, then WARN, then trace backwards
5. **Look for patterns** - same error at same point?
6. **Check the full query** - logs show first 200-500 chars
7. **Compare working vs failing** - run a simple query first to see "good" logs

## Related Documentation

- [NVIM_PLUGIN_LOGGING.md](NVIM_PLUGIN_LOGGING.md) - Logging system overview
- [FUZZY_FILTER_EXACT_MODE.md](FUZZY_FILTER_EXACT_MODE.md) - Uses logging too
- [NVIM_SMART_COLUMN_COMPLETION.md](NVIM_SMART_COLUMN_COMPLETION.md) - Expand star documentation
