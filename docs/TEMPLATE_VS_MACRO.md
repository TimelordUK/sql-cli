# Templates vs Macros in SQL-CLI

Understanding the difference between templates and macros is key to using the SQL-CLI template system effectively.

## Quick Reference

| Type | Purpose | Definition | Usage | Expansion |
|------|---------|------------|-------|-----------|
| **Macro** | Small reusable snippets | `-- MACRO: name` | `@{MACRO:name}` or `@name` | Inline replacement |
| **Template** | Complete query structures | `-- Template: name` | `@{TEMPLATE:name}` or `@name` | Full query insertion |

## Macros

Macros are **small, reusable text snippets** that get expanded inline.

### Definition
```sql
-- MACRO: COLUMNS_BASIC
-- DealId, PlatformOrderId, DealType, SignedQuantity
-- END MACRO

-- MACRO: WHERE_ACTIVE
-- Status = 'Active' AND Amount > 0
-- END MACRO
```

### Usage
```sql
-- Use in queries
SELECT @{MACRO:COLUMNS_BASIC}
FROM trades
WHERE @{MACRO:WHERE_ACTIVE};

-- Shorthand (if no conflict with templates)
SELECT @COLUMNS_BASIC FROM trades;
```

### Best For
- Column lists
- WHERE conditions
- Reusable expressions
- Configuration values

## Templates

Templates are **complete query structures** with placeholders for dynamic values.

### Definition
```sql
-- Template: FETCH_TRADES
-- Description: Fetch trades from API
WITH WEB trades AS (
    URL '@{VAR:BASE_URL}/query'
    METHOD POST
    BODY '{
        "select": "@{MACRO:COLUMNS_BASIC}",
        "where": "@{INPUT:WHERE clause}"
    }'
    FORMAT JSON
)
SELECT * FROM trades;
```

### Usage

#### Method 1: Template Selection (Recommended)
```vim
:SqlTemplateSelect    " or press \sts
```
This shows a menu to:
1. Choose template file
2. Choose template from file
3. Insert at cursor position

#### Method 2: Inline Expansion (New!)
```sql
-- Type this in your SQL file:
@FETCH_TRADES

-- Or with explicit syntax:
@{TEMPLATE:FETCH_TRADES}

-- Then press \ste to expand
```

### Best For
- Complete query structures
- CTE definitions
- Complex multi-step queries
- Query templates with parameters

## Key Differences

### 1. **Size & Scope**
- **Macros**: Small snippets (1-5 lines typically)
- **Templates**: Complete queries (5-50+ lines)

### 2. **Purpose**
- **Macros**: Replace repetitive text fragments
- **Templates**: Insert entire query structures

### 3. **Nesting**
- **Macros**: Can be used inside templates and other macros
- **Templates**: Standalone, but can contain macros

### 4. **Interaction**
- **Macros**: Direct expansion at cursor
- **Templates**: Usually selected from menu or expanded as whole query

## Combined Usage

Templates often use macros for flexibility:

```sql
-- Template: RECONCILIATION_REPORT
-- Description: Daily reconciliation with configurable columns
WITH WEB data AS (
    URL '@{VAR:BASE_URL}/reconciliation'
    BODY '{
        "select": "@{MACRO:COLUMNS_RECONCILIATION}",  -- Macro for columns
        "where": "@{MACRO:WHERE_TODAY}",              -- Macro for date
        "source": "@{JPICKER:SOURCES:Source}"         -- Dynamic picker
    }'
)
SELECT * FROM data
WHERE @{MACRO:WHERE_BREAK_CONDITIONS};                -- Another macro
```

## Expansion Behavior

### When you type `@NAME` and press `\ste`:

1. **First checks templates** - If NAME is a template, expands full template
2. **Then checks macros** - If NAME is a macro, expands inline
3. **Error if neither** - Shows message to define it

### Examples:

```sql
-- If COLUMNS_BASIC is a macro:
SELECT @COLUMNS_BASIC FROM trades;
-- Expands to:
SELECT DealId, PlatformOrderId, DealType, SignedQuantity FROM trades;

-- If FETCH_TRADES is a template:
@FETCH_TRADES
-- Expands to entire query:
WITH WEB trades AS (
    URL 'http://localhost/query'
    METHOD POST
    ...
)
SELECT * FROM trades;
```

## File Organization

### Recommended Structure

```sql
-- templates/trades.sql

-- ============================================
-- Macros (building blocks)
-- ============================================

-- MACRO: COLS_ID
-- DealId, PlatformOrderId
-- END MACRO

-- MACRO: WHERE_TODAY
-- TradeDate = DateTime(2025, 9, 25)
-- END MACRO

-- ============================================
-- Templates (complete queries using macros)
-- ============================================

-- Template: DAILY_TRADES
-- Description: Today's trades with standard columns
SELECT @{MACRO:COLS_ID}, Price, Amount
FROM trades
WHERE @{MACRO:WHERE_TODAY};

-- Template: TRADE_SEARCH
-- Description: Interactive trade search
SELECT @{PICKER:COLUMN_SETS:Columns}
FROM trades
WHERE @{INPUT:WHERE conditions:1=1};
```

## Quick Decision Guide

**Use a MACRO when:**
- You're repeating the same text in multiple places
- It's a small snippet (column list, WHERE clause)
- You want inline replacement
- It will be used inside templates

**Use a TEMPLATE when:**
- You need a complete query structure
- It has multiple steps (WITH clauses, subqueries)
- You want to insert a full query
- It's a standard report or analysis pattern

## Conversion Between Types

### Macro → Template
```sql
-- Original macro:
-- MACRO: SIMPLE_SELECT
-- SELECT * FROM trades WHERE Status = 'Active'
-- END MACRO

-- Convert to template:
-- Template: ACTIVE_TRADES
-- Description: Query for active trades
SELECT * FROM trades WHERE Status = 'Active';
```

### Template → Macro
```sql
-- Original template:
-- Template: PRICE_CHECK
SELECT Symbol, Price FROM trades;

-- Convert to macro (wrap in MACRO tags):
-- MACRO: PRICE_CHECK_QUERY
-- SELECT Symbol, Price FROM trades
-- END MACRO
```

## Tips

1. **Start with templates** for complete queries
2. **Extract common parts to macros** as patterns emerge
3. **Use descriptive names** that indicate type:
   - `COLUMNS_*` for column macros
   - `WHERE_*` for WHERE clause macros
   - `FETCH_*` or `QUERY_*` for templates
4. **Document purpose** with descriptions for templates
5. **Test expansion** with `\ste` before using in production