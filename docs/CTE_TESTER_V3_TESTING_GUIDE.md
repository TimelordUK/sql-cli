# CTE Tester V3 - Testing Guide for Work

**Date**: 2025-10-04
**Version**: V3 (CLI Analysis-based)
**Status**: Ready for testing

## What Changed

### Before (V2): Fragile Regex Parsing
- 200+ lines of complex text parsing
- Manual CTE detection with regex patterns
- Often failed with WEB CTEs or complex chains
- Errors like "cant find cte at cursor" or "unable to find cte 1"

### After (V3): CLI Analysis
- **20 lines** of simple logic
- Uses `--analyze-query` to detect CTEs
- Uses `--extract-cte` to get CTE query
- Should work 100% reliably with WEB CTEs and chains

## How to Test at Work

### 1. Pull Latest Code
```bash
cd /path/to/sql-cli
git pull
```

### 2. Enable Logging
In your Neovim config (before testing):
```lua
require('sql-cli').setup({
  logging = {
    enabled = true,
    level = "DEBUG",  -- Get detailed logs
    max_files = 30,
  },
})
```

### 3. Test with Your Work Query

Open a SQL file with your typical work query structure:
```sql
WITH WEB raw_trades AS (
    URL 'http://your-api/trades'
    METHOD GET
    HEADERS ('Authorization' = 'Bearer your-token')
    FORMAT JSON
),
filtered AS (
    SELECT
        Source,
        PlatformOrderId,
        Price
    FROM raw_trades
    WHERE Source = 'Bloomberg'
      AND Price > 100
),
aggregated AS (
    SELECT
        Source,
        COUNT(*) as cnt,
        AVG(Price) as avg_price
    FROM filtered
    GROUP BY Source
)
SELECT * FROM aggregated;
```

### 4. Test CTE Extraction

**Put cursor on different lines and press `\sC`**:

- **Line 1-6** (in WEB CTE) → Should extract WEB CTE
- **Line 8-15** (in filtered CTE) → Should extract filtered (references raw_trades)
- **Line 17-24** (in aggregated CTE) → Should extract aggregated (references filtered)
- **Line 26** (main query) → Should extract last CTE (aggregated)

### 5. What to Look For

#### ✅ Success Indicators
- Modal appears showing extracted CTE query
- CTE name shown in modal title: `CTE Test: filtered_trades (Standard)`
- For WEB CTE: Shows `(WEB)` in title
- Query looks correct with proper FROM references
- Press Enter → Executes successfully
- Results appear in results window

#### ❌ Failure Indicators
- Error: "Failed to analyze query"
- Error: "Failed to extract CTE"
- Error: "Could not determine CTE at cursor"
- Wrong CTE extracted (e.g., cursor in filtered but got aggregated)
- Execution fails with "table not found"

### 6. Check the Logs

After testing, check logs:
```vim
:SqlCliOpenLog
```

**Look for these log entries**:
```
[INFO] [cte_test_v3] === test_cte_at_cursor called (V3 with CLI analysis) ===
[INFO] [cte_test_v3] Found query boundaries: lines 1-27 (27 lines)
[DEBUG] [cli_analyzer] analyze_query called with query length: 542
[DEBUG] [cli_analyzer] Executing: sql-cli -q ... --analyze-query
[INFO] [cli_analyzer] Analysis successful: 3 CTEs, has_star=true, 1 tables
[INFO] [cte_test_v3] Analysis complete: 3 CTEs found
[INFO] [cte_test_v3] CTEs: raw_trades, filtered, aggregated
[DEBUG] [cte_test_v3] Finding CTE at cursor (relative line 12)
[INFO] [cte_test_v3] Found CTE #1 "raw_trades" at line 1
[INFO] [cte_test_v3] Found CTE #2 "filtered" at line 8
[INFO] [cte_test_v3] Found CTE #3 "aggregated" at line 17
[INFO] [cte_test_v3] Target CTE: filtered (Standard)
[DEBUG] [cli_analyzer] extract_cte called for: filtered
[DEBUG] [cli_analyzer] Executing: sql-cli -q ... --extract-cte filtered
[INFO] [cli_analyzer] CTE extracted successfully (234 chars)
[INFO] [cte_test_v3] Executing CTE test query
```

### 7. Common Scenarios to Test

#### Scenario 1: WEB CTE with Headers
```sql
WITH WEB trades AS (
    URL 'http://api/trades'
    HEADERS ('Authorization' = 'Bearer token')
)
SELECT * FROM trades LIMIT 5;
```
- Cursor in WEB CTE → Should extract full WEB CTE spec
- Should execute successfully (if API is reachable)

#### Scenario 2: Chained CTEs (3+ levels)
```sql
WITH WEB raw AS (...),
step1 AS (SELECT * FROM raw WHERE ...),
step2 AS (SELECT * FROM step1 WHERE ...),
step3 AS (SELECT * FROM step2 WHERE ...)
SELECT * FROM step3;
```
- Test cursor in each CTE
- Each should extract correctly with proper FROM reference
- step1 should reference raw_trades, step2 should reference step1, etc.

#### Scenario 3: FIX Log with Selector
```sql
WITH WEB fix_log AS (
    URL 'http://localhost:8080/upload'
    METHOD POST
    HEADERS ('Content-Type' = 'multipart/form-data')
    BODY '...'
    SELECTOR JSON '{...}'
)
SELECT * FROM fix_log;
```
- Should extract full WEB CTE with SELECTOR
- Should execute (if server running)

## Reporting Issues

If something doesn't work, capture:

### 1. The SQL Query
Save your work query to a file (sanitize sensitive data):
```bash
# Remove tokens/credentials
sed 's/Bearer .*/Bearer REDACTED/g' my_query.sql > my_query_sanitized.sql
```

### 2. The Logs
```vim
:SqlCliLogPath  " Get log file path
```

Then extract relevant section:
```bash
# Get last 200 lines of cte_test_v3 logs
grep "cte_test_v3" /path/to/log.log | tail -200 > cte_test_issue.log

# Get CLI analyzer logs too
grep "cli_analyzer" /path/to/log.log | tail -200 >> cte_test_issue.log
```

### 3. What Happened
- Which CTE were you trying to test?
- What line was cursor on?
- What error appeared?
- Did modal appear? If yes, was query correct?

### 4. Expected vs Actual
- **Expected**: "Should extract `filtered` CTE"
- **Actual**: "Got error: Failed to extract CTE: CTE 'filtered' not found"

## Quick Validation Tests

### Test 1: Simple CTE
```sql
WITH test AS (SELECT 1 as a, 2 as b)
SELECT * FROM test;
```
- Cursor on line 1 → `\sC` → Should show: `SELECT 1 as a, 2 as b`

### Test 2: Two CTEs
```sql
WITH
a AS (SELECT 1 as x),
b AS (SELECT * FROM a WHERE x > 0)
SELECT * FROM b;
```
- Cursor on line 2 (`a` CTE) → Should extract: `SELECT 1 as x`
- Cursor on line 3 (`b` CTE) → Should extract: `SELECT * FROM a WHERE x > 0`

## Known Limitations (Current)

1. **Line number estimation**: CTE detection uses text search, not exact parser positions
   - **Impact**: Usually works, but if two CTEs have similar names, might pick wrong one
   - **Workaround**: Ensure unique CTE names

2. **Comments in CTEs**: Heavy comments might confuse detection
   - **Impact**: Might not find CTE start line correctly
   - **Workaround**: Remove comments before testing

## Success Criteria

**V3 should be ready for work if:**
- ✅ Works with your WEB CTE queries
- ✅ Works with 3+ chained CTEs
- ✅ Correctly identifies which CTE cursor is in
- ✅ Extracts CTEs with proper FROM references
- ✅ No "cant find cte at cursor" errors
- ✅ Logs show clear progression (analyze → extract → execute)

If all these pass, V3 is production-ready! 🚀

## Next Steps After Testing

If V3 works well:
1. Report success + share logs showing it working
2. I'll do the same for `\sE` (expand star)
3. Then remove old V2 code
4. Plugin will be rock-solid for production!

If issues found:
1. Share query + logs + error
2. I'll debug and fix
3. Iterate until solid

The architecture is sound - any issues will just be minor edge cases to fix! 💪
