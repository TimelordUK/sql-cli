# SQL CLI Parameter System

The parameter system allows you to create reusable queries with runtime parameters, perfect for testing different scenarios without duplicating queries.

## Quick Start

1. Add parameters to your query using `{{PARAM_NAME}}` or `${PARAM_NAME}` syntax
2. Execute with `\sx` - you'll be prompted for values
3. Previous values are remembered for quick selection

## Basic Example

```sql
-- Query with parameters
SELECT * FROM trades
WHERE order_id = '{{ORDER_ID}}'
  AND source = '{{SOURCE}}'
  AND trade_date = '{{DATE}}';
```

When you execute this with `\sx`, you'll be prompted for:
- ORDER_ID: Choose from history or enter new
- SOURCE: Pick from common sources or history
- DATE: Select from recent dates or enter new

## Parameter Sets

Define common parameter combinations at the top of your SQL file:

```sql
-- @PARAMS: morning_bloomberg = { ORDER_ID: "ORDER_001", SOURCE: "Bloomberg", DATE: "2025,09,27" }
-- @PARAMS: afternoon_reuters = { ORDER_ID: "ORDER_002", SOURCE: "Reuters", DATE: "2025,09,27" }
-- @PARAMS: test_order = { ORDER_ID: "TEST_999", SOURCE: "TradeWeb", DATE: "2025,09,26" }
```

These sets appear in the parameter picker for quick selection.

## Real-World Examples

### Example 1: Testing Different Orders

```sql
WITH web_trades AS (
    SELECT * FROM WEB('https://trade-server/api/trades',
    '{
        "where": "PlatformOrderId.StartsWith(\"{{ORDER_ID}}\")"
    }')
)
SELECT
    PlatformOrderId,
    TradeDate,
    Symbol,
    Quantity,
    Price,
    Status
FROM web_trades
ORDER BY TradeDate DESC;
```

**Workflow:**
1. Run with `\sx`
2. First time: Enter "ORDER_001"
3. Next time: Pick "ORDER_001" from history or enter "ORDER_002"
4. System remembers last 10 order IDs

### Example 2: Daily Source Analysis

```sql
-- @PARAMS: bloomberg_today = { SOURCE: "Bloomberg", DATE: "2025,09,27" }
-- @PARAMS: reuters_today = { SOURCE: "Reuters", DATE: "2025,09,27" }

WITH source_trades AS (
    SELECT * FROM WEB('https://trade-server/api/trades',
    '{
        "where": "Source = \"{{SOURCE}}\" AND TradeDate = DateTime({{DATE}})"
    }')
)
SELECT
    Source,
    COUNT(*) as trade_count,
    SUM(Quantity * Price) as total_value,
    AVG(Price) as avg_price
FROM source_trades
GROUP BY Source;
```

**Workflow:**
1. Run with `\sx`
2. Select parameter set "bloomberg_today" for Bloomberg analysis
3. Run again, select "reuters_today" for Reuters
4. Or manually pick SOURCE from: Bloomberg, Reuters, TradeWeb, MarketAxess, ICE, CME, Eurex, LSE, NYSE

### Example 3: Complex Multi-Parameter Query

```sql
WITH filtered_trades AS (
    SELECT * FROM WEB('https://trade-server/api/trades',
    '{
        "where": "PlatformOrderId.StartsWith(\"{{ORDER_PREFIX}}\")
                  AND Source = \"{{SOURCE}}\"
                  AND TradeDate >= DateTime({{START_DATE}})
                  AND TradeDate <= DateTime({{END_DATE}})"
    }')
)
SELECT
    DATE(TradeDate) as trade_date,
    Source,
    COUNT(*) as trades,
    SUM(CASE WHEN Status = 'FILLED' THEN Quantity ELSE 0 END) as filled_qty
FROM filtered_trades
GROUP BY DATE(TradeDate), Source
ORDER BY trade_date DESC;
```

## Keyboard Shortcuts

| Shortcut | Action | Description |
|----------|--------|-------------|
| `\sx` | Execute with parameters | Run query, prompting for parameter values |
| `<leader>spn` | Next parameter value | Cycle to next value in history (cursor on parameter) |
| `<leader>spp` | Previous parameter value | Cycle to previous value in history (cursor on parameter) |
| `<leader>sps` | Save parameter set | Save current parameter values as a named set |

## Parameter Cycling

Put your cursor on a parameter in the query and use cycling shortcuts:

```sql
WHERE order_id = '{{ORDER_ID}}'
                     ↑ cursor here
```

Press `<leader>spn` to cycle through:
- ORDER_001 [1/5]
- ORDER_002 [2/5]
- ORDER_003 [3/5]
- etc.

## The Picker Interface

When you execute a query with parameters, you'll see:

```
Select value for {{ORDER_ID}}:
[Set: morning_bloomberg] ORDER_001       ← From parameter set
[Set: test_order] TEST_999              ← From parameter set
[Recent 1] ORDER_001                    ← From history
[Recent 2] ORDER_002                    ← From history
[Recent 3] TEST_999                     ← From history
[Similar: ORDER_PREFIX] ORD             ← From similar parameters
[Enter custom value...]                 ← Type new value
```

## Tips and Tricks

### 1. Quick Testing Multiple Scenarios

Instead of maintaining multiple similar queries:
```sql
-- ❌ Old way: Multiple similar CTEs
WITH web_trades_order1 AS (SELECT * FROM WEB(...WHERE "ORDER_001"...))
WITH web_trades_order2 AS (SELECT * FROM WEB(...WHERE "ORDER_002"...))
WITH web_trades_order3 AS (SELECT * FROM WEB(...WHERE "ORDER_003"...))
```

Use one parameterized query:
```sql
-- ✅ New way: One parameterized query
WITH web_trades AS (
    SELECT * FROM WEB(...WHERE "{{ORDER_ID}}"...)
)
```

### 2. Daily Workflow Pattern

Create parameter sets for your daily routine:

```sql
-- Morning checks
-- @PARAMS: morning_bloomberg = { SOURCE: "Bloomberg", DATE: "2025,09,27", STATUS: "PENDING" }
-- @PARAMS: morning_reuters = { SOURCE: "Reuters", DATE: "2025,09,27", STATUS: "PENDING" }

-- Afternoon reconciliation
-- @PARAMS: afternoon_filled = { SOURCE: "Bloomberg", DATE: "2025,09,27", STATUS: "FILLED" }
-- @PARAMS: afternoon_cancelled = { SOURCE: "Bloomberg", DATE: "2025,09,27", STATUS: "CANCELLED" }
```

### 3. Parameter History Management

- History is stored in `~/.local/share/nvim/sql-cli/params.json`
- Keeps last 10 values per parameter
- Shared across all SQL files
- Persists between nvim sessions

### 4. Source Parameter Special Handling

The system recognizes `SOURCE` parameters and provides quick access to common trading sources:
- Bloomberg
- Reuters
- TradeWeb
- MarketAxess
- ICE
- CME
- Eurex
- LSE
- NYSE

### 5. Combining with Macros

You can use parameters within macro templates:

```sql
-- In templates/trades_with_filter.sql
WITH {{TABLE_NAME}} AS (
    SELECT * FROM WEB('{{URL}}',
    '{
        "where": "{{WHERE_CLAUSE}}"
    }')
)

-- After macro expansion, add parameters:
WITH trades AS (
    SELECT * FROM WEB('https://trade-server/api/trades',
    '{
        "where": "Source = \"{{SOURCE}}\" AND Status = \"{{STATUS}}\""
    }')
)
```

## Common Patterns

### Pattern 1: Order Investigation
```sql
-- Quick order lookup with history
SELECT * FROM trades WHERE order_id = '{{ORDER_ID}}';
```

### Pattern 2: Source Comparison
```sql
-- Compare multiple sources by running multiple times
SELECT source, COUNT(*), SUM(value)
FROM trades
WHERE source = '{{SOURCE}}'
GROUP BY source;
```

### Pattern 3: Date Range Analysis
```sql
-- Flexible date range queries
SELECT * FROM trades
WHERE trade_date BETWEEN '{{START_DATE}}' AND '{{END_DATE}}';
```

### Pattern 4: Status Filtering
```sql
-- Quick status checks
SELECT * FROM trades
WHERE status = '{{STATUS}}'  -- FILLED, PARTIAL, CANCELLED, PENDING
  AND source = '{{SOURCE}}';
```

## Advanced Usage

### Nested Parameters in JSON

Perfect for complex WEB CTE queries:

```sql
WITH data AS (
    SELECT * FROM WEB('https://api.example.com/trades',
    '{
        "filters": {
            "order": "{{ORDER_ID}}",
            "source": "{{SOURCE}}",
            "date_from": "{{START_DATE}}",
            "date_to": "{{END_DATE}}"
        },
        "options": {
            "include_cancelled": {{INCLUDE_CANCELLED}},
            "max_results": {{MAX_RESULTS}}
        }
    }')
)
```

### Environment-Specific Parameters

```sql
-- @PARAMS: dev = { API_URL: "https://dev-api.example.com", API_KEY: "dev-key-123" }
-- @PARAMS: prod = { API_URL: "https://api.example.com", API_KEY: "prod-key-456" }

WITH data AS (
    SELECT * FROM WEB('{{API_URL}}/trades',
    '{
        "auth": "{{API_KEY}}",
        "query": "{{QUERY}}"
    }')
)
```

## Troubleshooting

### Parameters Not Being Detected
- Ensure you use the correct syntax: `{{PARAM}}` or `${PARAM}`
- Check for typos in parameter names
- Parameters are case-sensitive

### History Not Showing
- History is saved after successful parameter resolution
- Check `~/.local/share/nvim/sql-cli/params.json` exists
- Maximum 10 history items per parameter

### Parameter Sets Not Appearing
- Ensure the format is exactly: `-- @PARAMS: name = { KEY: "value" }`
- Must be a SQL comment starting with `--`
- Must have space after `--`
- JSON-like syntax with quotes around values

## Summary

The parameter system transforms your workflow from:
- ❌ Maintaining multiple similar queries
- ❌ Manually editing WHERE clauses
- ❌ Losing track of what you tested
- ❌ Copy-pasting and modifying queries

To:
- ✅ One query, multiple scenarios
- ✅ Quick parameter selection with history
- ✅ Named parameter sets for common cases
- ✅ Full audit trail of tested values

This is particularly powerful for:
- Testing different orders throughout the day
- Checking multiple sources for reconciliation
- Running the same analysis with different date ranges
- Investigating issues across different status values

The system maintains your investigation context, making it easy to re-run previous checks or try new values without losing your place.