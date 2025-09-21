# Neovim SQL Refactoring Tools Design

## Overview
CLI-powered refactoring commands for the Neovim plugin that analyze SQL patterns and suggest/apply transformations.

## Core Principle
Use the SQL CLI engine to analyze and transform SQL, not string manipulation in Lua.

## Proposed CLI Commands

### 1. Auto-Hoisting (`--auto-hoist`)
```bash
sql-cli --auto-hoist "SELECT * FROM table WHERE price * quantity > 1000"
# Returns JSON with suggested CTE transformation
```

**Use Cases:**
- Complex expressions in WHERE/SELECT
- Repeated subqueries
- Window functions that need preprocessing

**Output:**
```json
{
  "original": "SELECT * FROM table WHERE price * quantity > 1000",
  "transformed": {
    "ctes": [
      {
        "name": "computed",
        "query": "SELECT *, price * quantity as total FROM table"
      }
    ],
    "main": "SELECT * FROM computed WHERE total > 1000"
  }
}
```

### 2. Banding Generator (`--generate-bands`)
```bash
sql-cli --generate-bands --column "age" --bands "0-10,11-20,21-30,31-40,40+"
# Or with automatic band detection:
sql-cli --generate-bands --column "age" --auto --buckets 5
```

**Output:**
```sql
CASE
    WHEN age <= 10 THEN '0-10'
    WHEN age <= 20 THEN '11-20'
    WHEN age <= 30 THEN '21-30'
    WHEN age <= 40 THEN '31-40'
    ELSE '40+'
END AS age_band
```

### 3. Conditional Aggregation Builder (`--build-conditional-agg`)
```bash
sql-cli --build-conditional-agg --column "BuySell" --values "Buy,Sell" --aggregate "SUM(Quantity)"
```

**Output:**
```sql
SUM(CASE WHEN BuySell = 'Buy' THEN Quantity ELSE 0 END) as Buy_qty,
SUM(CASE WHEN BuySell = 'Sell' THEN Quantity ELSE 0 END) as Sell_qty
```

### 4. CTE Chain Analyzer (`--analyze-cte-chain`)
```bash
sql-cli --analyze-cte-chain query.sql
```

**Output:**
```json
{
  "chain": ["summary", "ranked", "with_lead"],
  "dependencies": {
    "ranked": ["summary"],
    "with_lead": ["ranked"]
  },
  "suggestions": [
    "Consider combining 'ranked' and 'with_lead' CTEs",
    "Expression 'SUBSTRING_AFTER' could be pre-computed"
  ]
}
```

### 5. Extract to CTE (`--extract-to-cte`)
```bash
# Pass selected expression and full query
sql-cli --extract-to-cte \
  --expression "SUBSTRING_AFTER(PlatformOrderId, '|', 1)" \
  --query "SELECT ... WHERE ..."
```

**Output:**
```json
{
  "cte_name": "extracted_1",
  "cte_query": "SELECT *, SUBSTRING_AFTER(PlatformOrderId, '|', 1) as extracted_value FROM table",
  "modified_query": "SELECT ... FROM extracted_1 WHERE ..."
}
```

## Neovim Integration

### Visual Mode Commands
```lua
-- Select expression, extract to CTE
vim.keymap.set('v', '<leader>se', function()
    local selected = get_visual_selection()
    local full_query = get_current_query()

    local result = vim.fn.system({
        'sql-cli',
        '--extract-to-cte',
        '--expression', selected,
        '--query', full_query
    })

    -- Apply transformation
    apply_cte_transformation(result)
end)
```

### Interactive Banding
```lua
-- Generate banding CASE statement
vim.keymap.set('n', '<leader>sb', function()
    local column = vim.fn.input('Column name: ')
    local bands = vim.fn.input('Bands (e.g., 0-10,11-20,21+): ')

    local result = vim.fn.system({
        'sql-cli',
        '--generate-bands',
        '--column', column,
        '--bands', bands
    })

    -- Insert at cursor
    vim.api.nvim_put({result}, 'l', true, true)
end)
```

### Smart Hoisting
```lua
-- Analyze current query and suggest hoisting
vim.keymap.set('n', '<leader>sh', function()
    local query = get_current_query()

    local result = vim.fn.system({
        'sql-cli',
        '--auto-hoist',
        query
    })

    -- Show preview with options to apply
    show_refactoring_preview(result)
end)
```

## Common Patterns Library

### Pattern: Pivot-like Conditional Aggregation
```sql
-- Input: Column with discrete values
-- Output: Multiple aggregation columns
SUM(CASE WHEN status = 'ACTIVE' THEN 1 ELSE 0 END) as active_count,
SUM(CASE WHEN status = 'INACTIVE' THEN 1 ELSE 0 END) as inactive_count
```

### Pattern: Root ID Extraction
```sql
-- Input: Delimited field
-- Output: CTE with extracted root
WITH roots AS (
    SELECT *,
        CASE
            WHEN CONTAINS(field, '|') THEN SUBSTRING_AFTER(field, '|', 1)
            ELSE field
        END AS root_id
    FROM table
)
```

### Pattern: Percentile Banding
```sql
-- Input: Continuous value
-- Output: Percentile bands
WITH percentiles AS (
    SELECT *,
        PERCENT_RANK() OVER (ORDER BY value) as pct_rank,
        CASE
            WHEN PERCENT_RANK() OVER (ORDER BY value) <= 0.25 THEN 'Q1'
            WHEN PERCENT_RANK() OVER (ORDER BY value) <= 0.50 THEN 'Q2'
            WHEN PERCENT_RANK() OVER (ORDER BY value) <= 0.75 THEN 'Q3'
            ELSE 'Q4'
        END AS quartile
    FROM table
)
```

## Implementation Priority

1. **Phase 1: Basic Refactoring**
   - Extract to CTE (visual selection)
   - Generate banding CASE statements
   - Build conditional aggregations

2. **Phase 2: Smart Analysis**
   - Auto-hoist complex expressions
   - CTE chain optimization
   - Pattern detection and suggestions

3. **Phase 3: Advanced Patterns**
   - Pivot transformations
   - Window function optimization
   - Multi-step refactoring chains

## Benefits

1. **No String Parsing in Lua** - All SQL analysis done by the engine
2. **Consistent Transformations** - Engine ensures valid SQL output
3. **Pattern Learning** - Can analyze user's SQL to suggest patterns
4. **Incremental Adoption** - Start with simple commands, add complexity
5. **Testing** - Each transformation can be tested independently

## Example Workflow

1. User writes complex WHERE clause
2. Selects the complex expression
3. Presses `<leader>se` (extract)
4. CLI analyzes and suggests CTE
5. Preview shows transformation
6. User accepts, CTE is added above
7. Original expression replaced with reference

This approach leverages the SQL engine's parser and keeps the Lua code simple and maintainable.