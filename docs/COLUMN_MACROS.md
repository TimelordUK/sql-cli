# Column Macros in SQL-CLI Templates

## Overview

Column macros allow you to define reusable column lists that can be expanded within your SQL templates. This eliminates repetition and makes it easy to maintain consistent column selections across queries.

## Basic Usage

### Define a Column Macro

```sql
-- Macro: COLUMNS_BASIC
-- DealId, PlatformOrderId, DealType, SignedQuantity, ExecutionTime
-- END MACRO
```

### Use in a Template

```sql
-- Template: FETCH_TRADES
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query'
    BODY '{
        "select": "@{MACRO:COLUMNS_BASIC}",
        "where": "Status = 'Active'"
    }'
)
SELECT * FROM trades;
```

When expanded, `@{MACRO:COLUMNS_BASIC}` becomes:
```sql
"select": "DealId, PlatformOrderId, DealType, SignedQuantity, ExecutionTime"
```

## Advanced Patterns

### 1. Nested Column Macros (Composition)

Define building blocks:
```sql
-- Macro: COLS_ID
-- DealId, PlatformOrderId
-- END MACRO

-- Macro: COLS_PRICING
-- Price, Amount, Currency
-- END MACRO

-- Macro: COLUMNS_FULL
-- @{MACRO:COLS_ID}, @{MACRO:COLS_PRICING}, Status, ExecutionTime
-- END MACRO
```

When `COLUMNS_FULL` expands:
1. First pass: `@{MACRO:COLS_ID}, @{MACRO:COLS_PRICING}, Status, ExecutionTime`
2. Second pass: `DealId, PlatformOrderId, Price, Amount, Currency, Status, ExecutionTime`

### 2. Dynamic Column Selection

Create a picker list of column sets:
```sql
-- Macro: COLUMN_SETS
-- @{MACRO:COLUMNS_BASIC}
-- @{MACRO:COLUMNS_PRICING}
-- @{MACRO:COLUMNS_FULL}
-- *
-- END MACRO

-- Template: DYNAMIC_QUERY
BODY '{
    "select": "@{PICKER:COLUMN_SETS:Choose columns}"
}'
```

User sees a menu:
- Basic columns (ID, Type, Quantity...)
- Pricing columns (Price, Amount, Currency...)
- Full columns (All fields)
- * (All columns)

### 3. Mixed Static and Dynamic Columns

```sql
-- Template: MIXED_COLUMNS
BODY '{
    "select": "@{MACRO:COLS_ID}, @{INPUT:Additional columns:}, LastUpdated"
}'
```

This gives you:
- Fixed columns from macro (DealId, PlatformOrderId)
- User-specified columns
- Another fixed column (LastUpdated)

### 4. Conditional Column Sets

Different columns based on query type:
```sql
-- Macro: REPORT_COLUMNS
-- @{MACRO:COLUMNS_RECONCILIATION}::For Reconciliation
-- @{MACRO:COLUMNS_RISK}::For Risk Report
-- @{MACRO:COLUMNS_ACCOUNTING}::For Accounting
-- END MACRO

-- Template: REPORT_QUERY
BODY '{
    "select": "@{PICKER:REPORT_COLUMNS:Report Type}"
}'
```

## Best Practices

### 1. Organize by Purpose

```sql
-- Core identity columns
-- Macro: COLS_ID

-- Trading-specific columns
-- Macro: COLS_TRADE

-- Financial columns
-- Macro: COLS_FINANCIAL

-- Combine for different uses
-- Macro: COLUMNS_RECONCILIATION
-- @{MACRO:COLS_ID}, @{MACRO:COLS_TRADE}, Status, MatchStatus
```

### 2. Use Descriptive Names

```sql
-- Good
-- Macro: COLUMNS_RECONCILIATION
-- Macro: COLS_RISK_METRICS
-- Macro: COLS_SETTLEMENT_DATES

-- Avoid
-- Macro: COLS1
-- Macro: TEMP_COLS
```

### 3. Document Column Sets

```sql
-- Macro: COLUMNS_RISK
-- Risk metrics columns for Greeks calculation
-- Includes: Delta, Gamma, Vega, Theta, Rho, ImpliedVol
-- Delta, Gamma, Vega, Theta, Rho, ImpliedVol
-- END MACRO
```

### 4. Create Hierarchical Sets

Small → Medium → Large:
```sql
-- Minimal columns for overview
-- Macro: COLUMNS_MINIMAL
-- DealId, DealType, Amount, Status

-- Standard columns for most queries
-- Macro: COLUMNS_STANDARD
-- @{MACRO:COLUMNS_MINIMAL}, ExecutionTime, Source, Symbol

-- Everything needed for detailed analysis
-- Macro: COLUMNS_DETAILED
-- @{MACRO:COLUMNS_STANDARD}, @{MACRO:COLS_PRICING}, @{MACRO:COLS_PARTIES}
```

## Examples

### Example 1: Trade Reconciliation Template

```sql
-- Macro: COLS_RECON_KEY
-- DealId, PlatformOrderId, Source
-- END MACRO

-- Macro: COLS_RECON_VALUES
-- SignedQuantity, Price, Amount, ExecutionTime
-- END MACRO

-- Macro: COLS_RECON_STATUS
-- Status, MatchStatus, BreakReason, LastUpdated
-- END MACRO

-- Macro: COLUMNS_RECONCILIATION
-- @{MACRO:COLS_RECON_KEY}, @{MACRO:COLS_RECON_VALUES}, @{MACRO:COLS_RECON_STATUS}
-- END MACRO

-- Template: RECONCILE_TRADES
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/reconciliation'
    BODY '{
        "select": "@{MACRO:COLUMNS_RECONCILIATION}",
        "where": "MatchStatus IN ('UNMATCHED', 'BREAK')"
    }'
)
SELECT * FROM trades;
```

### Example 2: Flexible Report Builder

```sql
-- Let user build custom column list
-- Template: CUSTOM_REPORT
WITH WEB data AS (
    URL '@{VAR:BASE_URL}/query'
    BODY '{
        "select": "@{PICKER:BASE_COLS:Start with}, @{INPUT:Add columns (optional):}",
        "where": "@{INPUT:WHERE clause:1=1}"
    }'
)
SELECT * FROM data;

-- Macro: BASE_COLS
-- @{MACRO:COLS_ID}
-- @{MACRO:COLS_ID}, @{MACRO:COLS_TRADE_BASIC}
-- @{MACRO:COLUMNS_MINIMAL}
-- @{MACRO:COLUMNS_STANDARD}
-- END MACRO
```

### Example 3: Context-Aware Columns

```sql
-- Different columns for different data sources
-- Macro: BLOOMBERG_COLUMNS
-- BBG_ID, BBG_Ticker, BBG_Price, BBG_Volume, BBG_Time
-- END MACRO

-- Macro: REUTERS_COLUMNS
-- RIC, Reuters_Price, Reuters_Size, Reuters_Time
-- END MACRO

-- Template: SOURCE_SPECIFIC_QUERY
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query'
    BODY '{
        "select": "@{PICKER:SOURCE_COLUMNS:Source Type}",
        "where": "Date = @{DATE:Query Date}"
    }'
)
SELECT * FROM trades;

-- Macro: SOURCE_COLUMNS
-- @{MACRO:BLOOMBERG_COLUMNS}::Bloomberg Fields
-- @{MACRO:REUTERS_COLUMNS}::Reuters Fields
-- @{MACRO:COLUMNS_STANDARD}::Standard Fields
-- END MACRO
```

## Testing Column Macros

To test your column macro expansions:

1. Create a test template:
```sql
-- Template: TEST_COLUMNS
-- Description: Test column macro expansion
SELECT @{MACRO:COLUMNS_BASIC} FROM test_table;
```

2. Use `:SqlTemplateExpand` on the macro reference

3. Check the expansion is correct

## Troubleshooting

### Macro Not Expanding
- Check macro is defined with `-- END MACRO`
- Verify macro name matches exactly (case-sensitive)
- Ensure no typos in `@{MACRO:name}` syntax

### Nested Macros Not Working
- Check recursion depth (default max: 10)
- Look for circular references (A→B→A)
- Test each level independently

### Columns Have Extra Quotes
- In JSON contexts, columns are usually strings
- The template system preserves your formatting
- Don't add quotes if the macro already has them

### Performance Considerations
- Deeply nested macros (>5 levels) may slow expansion
- Very long column lists (>50 columns) work but may be hard to manage
- Consider breaking into smaller, focused column sets