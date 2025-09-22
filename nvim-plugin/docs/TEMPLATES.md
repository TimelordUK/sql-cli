# SQL-CLI Template & Macro System Documentation

## Overview

The SQL-CLI Neovim plugin provides a powerful template and macro system that allows you to:
- Define reusable SQL query templates with variable substitution
- Create macros in SQL comment blocks for easy expansion
- Configure environment-specific settings
- Reduce copy-pasting of similar queries
- Auto-escape quotes in JSON bodies

## Key Concepts

### 1. Variables
Variables use the `${VARIABLE}` or `${VARIABLE:default}` syntax:
- `${SOURCE}` - Will prompt for value
- `${LIMIT:100}` - Will use 100 as default if not provided
- `${JWT_TOKEN}` - Automatically uses environment variable if available

### 2. Macros
Macros are defined in SQL comment blocks and expanded with `@MACRO_NAME`:
```sql
-- MACRO: MY_MACRO
-- SELECT * FROM table
-- WHERE condition = true
-- END MACRO
```

### 3. Special Variables
Certain variable names trigger smart pickers:
- `SOURCE` - Shows source selector from SOURCES macro
- `DATE`, `TRADE_DATE` - Shows date picker
- Variables matching env vars are auto-substituted

## Keybindings

| Keybinding | Description |
|------------|-------------|
| `\sT` | Select from pre-defined templates |
| `\sTq` | Quick substitute all variables in current query |
| `\sTe` | Expand macro at cursor position |
| `\sTm` | List and insert macros from current file |
| `\sTs` | Save current query as a template |
| `\sTd` | Insert quick date picker |
| `\sTS` | Insert trade source selector |

## Defining Macros

### Basic Macro Definition
```sql
-- MACRO: SIMPLE_SELECT
-- SELECT * FROM trades
-- WHERE TradeDate = TODAY()
-- END MACRO
```

### Macro with Variables
```sql
-- MACRO: FILTERED_TRADES
-- SELECT * FROM trades
-- WHERE Source = "${SOURCE}"
--   AND TradeDate = ${TRADE_DATE}
--   AND Quantity > ${MIN_QTY:0}
-- END MACRO
```

### Configuration Macro
```sql
-- MACRO: CONFIG
-- BASE_URL = http://localhost:5001
-- API_TOKEN = ${JWT_TOKEN}
-- DEFAULT_COLUMNS = TradeId, Source, Symbol, Quantity, Price
-- END MACRO
```

### Source List Macro
```sql
-- MACRO: SOURCES
-- Bloomberg_FIX_FX
-- Bloomberg_FIX_Equity
-- Reuters_FX
-- Internal_System
-- Manual_Entry
-- END MACRO
```

## Using Macros

### Method 1: Direct Expansion
1. Type `@MACRO_NAME` in your SQL file
2. Place cursor on the macro name
3. Press `\sTe` to expand

### Method 2: Quick Substitution
1. Write query with `${VARIABLES}`
2. Press `\sTq` to substitute all variables at once

### Method 3: Template Selection
1. Press `\sT` to show template menu
2. Select template
3. Fill in prompted variables

## Advanced Features

### Recursive Macro Expansion
Macros can reference other macros:
```sql
-- MACRO: BASE_CTE
-- WITH data AS (
--     SELECT * FROM source
-- )
-- END MACRO

-- MACRO: FULL_QUERY
-- @BASE_CTE
-- SELECT * FROM data
-- WHERE condition = true
-- END MACRO
```

### Auto-Escaping for JSON
Quotes in JSON bodies are automatically escaped:
```sql
-- Input: Source = "Bloomberg_FIX"
-- Result in JSON: \"Bloomberg_FIX\"
```

### Environment Variable Integration
```sql
-- If JWT_TOKEN is set in environment:
-- ${JWT_TOKEN} automatically uses the env value
-- ${API_KEY:default} checks env first, then uses default
```

## Complete Example

```sql
-- MACRO: CONFIG
-- BASE_URL = http://localhost:5001
-- API_TOKEN = ${JWT_TOKEN}
-- END MACRO

-- MACRO: SOURCES
-- Bloomberg_FIX_FX
-- Bloomberg_FIX_Equity
-- Reuters_FX
-- END MACRO

-- MACRO: TRADE_BASE
-- WITH WEB trades AS (
--     URL '${BASE_URL}/trades'
--     METHOD POST
--     BODY '{
--         "select": "${COLUMNS:*}",
--         "where": "${WHERE_CLAUSE}"
--     }'
--     FORMAT JSON
--     JSON_PATH 'Result'
--     HEADERS (
--         'Authorization': 'Bearer ${API_TOKEN}',
--         'Content-Type': 'application/json'
--     )
-- )
-- END MACRO

-- MACRO: TODAY_TRADES
-- @TRADE_BASE
-- SELECT * FROM trades
-- WHERE Date(TradeDate) = Date('now')
-- ORDER BY ExecutionTime DESC
-- END MACRO

-- Usage:
-- 1. Type @TODAY_TRADES
-- 2. Press \sTe
-- 3. Fill in prompted variables
-- 4. Query is ready to execute
```

## Best Practices

### 1. Organize Macros at Top of File
Keep all macro definitions at the beginning of your SQL files for easy reference.

### 2. Use Descriptive Names
- `TRADE_BASE` not `TB`
- `SOURCE_FILTER` not `SF`

### 3. Provide Defaults for Optional Parameters
```sql
"limit": ${LIMIT:100}
```

### 4. Document Complex Macros
```sql
-- MACRO: RECONCILIATION
-- Compares our trades with counterparty trades
-- Variables: OUR_SOURCE, THEIR_SOURCE, YEAR, MONTH, DAY
-- Returns: Mismatched trades with status indicators
-- ...macro content...
-- END MACRO
```

### 5. Use CONFIG Macro for Environment
Centralize environment-specific settings:
```sql
-- MACRO: CONFIG
-- BASE_URL = ${API_URL:http://localhost:5001}
-- TIMEOUT = ${TIMEOUT:30}
-- END MACRO
```

## Troubleshooting

### Macro Not Found
- Ensure macro name has no spaces
- Check spelling (case-sensitive)
- Verify `END MACRO` line exists

### Variables Not Substituted
- Check variable syntax: `${VAR}` not `$VAR`
- Ensure closing `}` is present
- For defaults: `${VAR:default}` with colon

### Expansion Overwrites Buffer
- This was a bug that's now fixed
- Update to latest plugin version

### Nested Macros Not Expanding
- Recursive expansion is supported
- Check macro references use `@` prefix
- Verify referenced macros are defined

## Tips & Tricks

### 1. Quick Date Entry
Instead of typing dates manually:
```sql
-- Press \sTd for date picker
-- Options: Today, Yesterday, Last Business Day, Custom
```

### 2. Source Selection
When variable is named SOURCE:
- Automatically shows picker with your SOURCES macro list
- No need to type source names manually

### 3. Testing Macros
Use `\sTm` to see all macros in current file and test expansion.

### 4. Sharing Templates
Create a `templates.sql` file with common macros and source it:
```vim
:e templates.sql
" Copy macros
:e myquery.sql
" Paste macros at top
```

### 5. JSON Body Formatting
Multi-line JSON bodies are supported:
```sql
BODY '{
    "select": "${COLUMNS}",
    "where": "${WHERE}",
    "orderBy": "${ORDER:TradeDate DESC}"
}'
```

## Integration with sql-cli

The template system is designed specifically for sql-cli's features:
- **WEB CTEs**: Easy REST API integration
- **JSON_PATH**: Navigate nested JSON responses
- **Variable substitution**: Before query execution
- **Quote escaping**: Automatic for JSON bodies

## File Organization

Recommended structure:
```
sql-cli/
├── queries/
│   ├── config.sql          # Shared CONFIG and SOURCES macros
│   ├── trade_queries.sql   # Trade-specific templates
│   ├── reconciliation.sql  # Reconciliation templates
│   └── daily_reports.sql   # Daily reporting templates
```

Source shared config:
```sql
-- At top of trade_queries.sql:
-- #include config.sql (conceptually - copy/paste for now)
```