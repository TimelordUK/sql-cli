# File-Level Variables

## Overview
Define reusable variables at the top of your SQL files for easy value substitution throughout your scripts. Perfect for working with different ORDER_IDs, dates, or any values you frequently change.

## Quick Start

### 1. Define Variables at Top of File

```sql
-- @SET ORDER_ID = ORDER_001
-- @SET TRADE_DATE = 2025-01-15
-- @SET SOURCE = Bloomberg
-- @SET MIN_AMOUNT = 1000
```

### 2. Use Variables Anywhere in Your Queries

```sql
SELECT * FROM orders
WHERE order_id = '${ORDER_ID}'
  AND trade_date = '${TRADE_DATE}'
  AND source = '${SOURCE}'
  AND amount >= ${MIN_AMOUNT}
;
GO
```

### 3. Change Values and Re-run

Simply change the value in the `@SET` line:
```sql
-- @SET ORDER_ID = ORDER_002
```

Then run with `\sq` in Neovim - all instances of `${ORDER_ID}` are automatically substituted!

## Syntax

### Defining Variables
```sql
-- @SET VARIABLE_NAME = value
```

**Rules:**
- Variable names must be UPPERCASE with underscores (e.g., `ORDER_ID`, `TRADE_DATE`)
- Values can be single or multi-word
- Quotes around values are optional and will be stripped
- Leading/trailing whitespace is automatically trimmed

**Examples:**
```sql
-- @SET ORDER_ID = ORDER_001
-- @SET ORDER_ID = "ORDER_001"        -- quotes stripped
-- @SET DESCRIPTION = Multi Word Value -- works!
-- @SET API_URL = http://api.example.com/v1
```

### Using Variables
```sql
${VARIABLE_NAME}
```

**In Queries:**
```sql
-- String values (add quotes in SQL)
WHERE order_id = '${ORDER_ID}'

-- Numeric values (no quotes needed)
WHERE amount > ${MIN_AMOUNT}

-- In function calls
WHERE DATE(timestamp) = '${TRADE_DATE}'

-- In JSON bodies (auto-escaped)
SELECT json_object('order', '${ORDER_ID}', 'source', '${SOURCE}')
```

## Complete Example

```sql
-- #! ../data/trades.csv
--
-- Trade Analysis Script
-- Change the variables below to analyze different trades
--

-- @SET ORDER_ID = ORD-2025-001
-- @SET TRADE_DATE = 2025-01-15
-- @SET COUNTERPARTY = Goldman Sachs
-- @SET MIN_NOTIONAL = 1000000

-- Find the specific trade
SELECT
    order_id,
    trade_date,
    counterparty,
    notional,
    status
FROM trades
WHERE order_id = '${ORDER_ID}'
;
GO

-- Find related trades on same date
SELECT
    order_id,
    counterparty,
    notional,
    status
FROM trades
WHERE trade_date = '${TRADE_DATE}'
  AND counterparty = '${COUNTERPARTY}'
  AND notional >= ${MIN_NOTIONAL}
ORDER BY notional DESC
;
GO

-- Summary statistics
SELECT
    '${ORDER_ID}' as target_order,
    '${TRADE_DATE}' as target_date,
    COUNT(*) as total_trades,
    SUM(notional) as total_notional,
    AVG(notional) as avg_notional
FROM trades
WHERE trade_date = '${TRADE_DATE}'
;
GO
```

## Variable Precedence

File-level variables are resolved in this order:

1. **File-level variables** (defined with `@SET`) - highest priority
2. **Environment variables** (like `${JWT_TOKEN}`)
3. **Keep placeholder** if variable not found

This means your `@SET` definitions override environment variables, giving you fine-grained control.

## Use Cases

### 1. Order/Trade Investigation
```sql
-- @SET ORDER_ID = ORD-12345

SELECT * FROM orders WHERE order_id = '${ORDER_ID}';
GO

SELECT * FROM executions WHERE order_id = '${ORDER_ID}';
GO

SELECT * FROM audit_log WHERE order_id = '${ORDER_ID}';
GO
```

### 2. Date Range Analysis
```sql
-- @SET START_DATE = 2025-01-01
-- @SET END_DATE = 2025-01-31

SELECT * FROM trades
WHERE trade_date BETWEEN '${START_DATE}' AND '${END_DATE}'
;
GO
```

### 3. Multi-Source Comparison
```sql
-- @SET SOURCE = Bloomberg
-- @SET BENCHMARK_SOURCE = Reuters

SELECT * FROM prices WHERE source = '${SOURCE}';
GO

SELECT * FROM prices WHERE source = '${BENCHMARK_SOURCE}';
GO
```

### 4. Testing Different Thresholds
```sql
-- @SET THRESHOLD = 1000000
-- @SET RISK_LIMIT = 5000000

SELECT * FROM positions
WHERE notional > ${THRESHOLD}
  AND risk_exposure < ${RISK_LIMIT}
;
GO
```

## Keybindings

All standard execution commands work with file-level variables:

| Command | Description |
|---------|-------------|
| `\sq` | Execute entire script with variable substitution |
| `\sx` | Execute statement at cursor with substitution |
| `\ss` | Execute visual selection with substitution |

## Comparison with Other Features

### vs. `{{PARAM}}` (Parameter System)
- **`@SET` + `${VAR}`**: Define once at top, use throughout file
- **`{{PARAM}}`**: Interactive prompt every time, good for one-off queries

### vs. Environment Variables
- **`@SET` variables**: File-specific, override env vars, easy to change
- **Environment variables**: System-wide, persistent across sessions

### vs. Templates
- **`@SET` variables**: Quick inline substitution, no file management
- **Templates**: Reusable across files, stored separately

## Best Practices

1. **Group variables at the top** for easy visibility:
   ```sql
   -- Configuration
   -- @SET ORDER_ID = ORD-001
   -- @SET TRADE_DATE = 2025-01-15
   -- @SET SOURCE = Bloomberg
   ```

2. **Use descriptive names**:
   ```sql
   -- Good
   -- @SET PRIMARY_ORDER_ID = ORD-001
   -- @SET COUNTERPARTY_NAME = Goldman Sachs

   -- Avoid
   -- @SET ID = 001
   -- @SET NAME = GS
   ```

3. **Comment your variables**:
   ```sql
   -- Configuration
   -- @SET ORDER_ID = ORD-001      -- Primary order to investigate
   -- @SET THRESHOLD = 1000000     -- Minimum notional for filtering
   ```

4. **Use consistent naming**:
   - Dates: `TRADE_DATE`, `START_DATE`, `END_DATE`
   - IDs: `ORDER_ID`, `TRADE_ID`, `CUSTOMER_ID`
   - Amounts: `MIN_AMOUNT`, `MAX_AMOUNT`, `THRESHOLD`

## Troubleshooting

### Variable not being substituted
- Check variable name is UPPERCASE with underscores only
- Ensure `@SET` line starts with `--` (SQL comment)
- Verify spacing: `-- @SET VAR = value` (spaces around `=`)

### Getting literal `${VAR}` in output
- Variable not defined - add `-- @SET VAR = value` at top
- Typo in variable name - check spelling matches exactly

### Quotes in values
- Single/double quotes in `@SET` are stripped automatically
- Add quotes in SQL where needed: `'${VAR}'` for strings

## Examples

See `examples/file_variables_demo.sql` for a complete working example.

## Integration with Other Features

File-level variables work seamlessly with:
- GO statement separators (multi-statement scripts)
- Data file hints (`-- #! data.csv`)
- Parameter system (`{{PARAM}}`)
- Environment variables (`${JWT_TOKEN}`)
- All export formats
- Query history

Simply change the `@SET` value and re-run your script!
