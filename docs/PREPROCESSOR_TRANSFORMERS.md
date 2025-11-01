# SQL Preprocessor Transformers

## Overview

SQL CLI uses AST (Abstract Syntax Tree) transformers to **rewrite queries before execution**. This allows us to support advanced SQL features that would be extremely complex or impossible to implement directly in the query executor.

**Key Principle:** Transformers are **enabled by default** (opt-out, not opt-in). This ensures queries "just work" without requiring users to enable flags.

## Philosophy: Opt-Out, Not Opt-In

All transformers are enabled by default because:

1. **Users expect standard SQL to work** - Features like `WHERE` with SELECT aliases should "just work"
2. **Complexity is hidden** - Users don't need to know about internal rewrites
3. **Compatibility first** - Only disable if a transformer breaks an existing query
4. **Future-proof** - New transformers can be added without breaking changes

If a transformer causes issues with specific queries, users can disable it with `--no-*` flags.

## Available Transformers

### 1. Expression Lifter (`ExpressionLifterTransformer`)

**Purpose:** Enables window functions and complex expressions in WHERE, GROUP BY, and HAVING clauses

**Flag:** `--no-expression-lifter` (disable)

**What it does:**
- Detects window functions or complex expressions in clauses where they can't be evaluated directly
- Creates a CTE that computes the expression
- Replaces the expression with a column reference to the CTE

**Example:**
```sql
-- Input
SELECT region, sales_amount,
       ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) AS rank
FROM sales
WHERE ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) <= 3

-- Rewritten to (conceptually)
WITH _lifted_0 AS (
    SELECT region, sales_amount,
           ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) AS _expr_0
    FROM sales
)
SELECT region, sales_amount, _expr_0 AS rank
FROM _lifted_0
WHERE _expr_0 <= 3
```

**Why it matters:**
- Enables "top N per group" queries
- Allows filtering on window function results
- Supports complex CASE expressions in GROUP BY

**Status:** ✅ Implemented, enabled by default

---

### 2. WHERE Alias Expander (`WhereAliasExpander`)

**Purpose:** Allows using SELECT column aliases in WHERE clauses

**Flag:** `--no-where-expansion` (disable)

**What it does:**
- Finds alias references in WHERE clause
- Replaces them with the original expression from SELECT

**Example:**
```sql
-- Input
SELECT sales_amount * 1.1 AS adjusted, region
FROM sales
WHERE adjusted > 15000

-- Rewritten to
SELECT sales_amount * 1.1 AS adjusted, region
FROM sales
WHERE (sales_amount * 1.1) > 15000
```

**Why it matters:**
- Standard SQL feature that most engines don't support natively
- Makes queries more readable (define expression once, use multiple times)
- Reduces duplication

**Status:** ✅ Implemented, enabled by default

---

### 3. GROUP BY Alias Expander (`GroupByAliasExpander`)

**Purpose:** Allows using SELECT column aliases in GROUP BY clauses

**Flag:** `--no-group-by-expansion` (disable)

**What it does:**
- Finds alias references in GROUP BY clause
- Replaces them with the original expression from SELECT

**Example:**
```sql
-- Input
SELECT UPPER(region) AS region_code, SUM(sales_amount) AS total
FROM sales
GROUP BY region_code

-- Rewritten to
SELECT UPPER(region) AS region_code, SUM(sales_amount) AS total
FROM sales
GROUP BY UPPER(region)
```

**Why it matters:**
- SQL standard allows this (SELECT clause is conceptually evaluated first)
- Makes queries more maintainable
- Reduces expression duplication

**Status:** ✅ Implemented, enabled by default

---

### 4. HAVING Alias Transformer (`HavingAliasTransformer`)

**Purpose:** Automatically creates aliases for aggregates in SELECT, rewrites HAVING to use them

**Flag:** `--no-having-expansion` (disable)

**What it does:**
- Detects aggregate functions in HAVING clause
- Creates aliases for them in SELECT (if not already present)
- Rewrites HAVING to reference the aliases

**Example:**
```sql
-- Input
SELECT region, SUM(sales_amount)
FROM sales
GROUP BY region
HAVING SUM(sales_amount) > 50000

-- Rewritten to
SELECT region, SUM(sales_amount) AS _agg_0
FROM sales
GROUP BY region
HAVING _agg_0 > 50000
```

**Why it matters:**
- Avoids recomputing aggregates (compute once in SELECT, filter in HAVING)
- Simplifies executor logic
- Improves performance

**Status:** ✅ Implemented, enabled by default

---

### 5. CTE Hoister (`CTEHoisterTransformer`)

**Purpose:** Moves nested CTEs to the top level for cleaner AST structure

**Flag:** `--no-cte-hoister` (disable)

**What it does:**
- Finds CTEs in subqueries or nested contexts
- Hoists them to the top level
- Renames if necessary to avoid conflicts

**Example:**
```sql
-- Input (when we support subqueries)
SELECT * FROM (
    WITH regional AS (SELECT region, SUM(sales_amount) AS total FROM sales GROUP BY region)
    SELECT * FROM regional WHERE total > 1000
) AS filtered

-- Rewritten to
WITH regional AS (SELECT region, SUM(sales_amount) AS total FROM sales GROUP BY region)
SELECT * FROM (SELECT * FROM regional WHERE total > 1000) AS filtered
```

**Why it matters:**
- Simplifies AST structure
- Enables better optimization
- Prepares for future subquery support

**Status:** ✅ Implemented, enabled by default (ready for when we add subqueries)

---

### 6. IN Operator Lifter (`InOperatorLifterTransformer`)

**Purpose:** Optimizes large IN lists by converting to temporary tables or subqueries

**Flag:** `--no-in-lifter` (disable)

**What it does:**
- Detects IN clauses with many values (e.g., 100+ items)
- Converts to a temporary table or CTE
- Rewrites IN to use a subquery

**Example:**
```sql
-- Input (with 100+ values)
SELECT * FROM sales WHERE region IN ('East', 'West', ..., 'value_100')

-- Rewritten to (conceptually)
WITH _in_list_0 AS (
    SELECT 'East' AS value UNION ALL SELECT 'West' UNION ALL ...
)
SELECT * FROM sales WHERE region IN (SELECT value FROM _in_list_0)
```

**Why it matters:**
- Performance optimization for large IN lists
- Reduces memory pressure
- Enables index usage (in future)

**Status:** ✅ Implemented, enabled by default

---

## Transformer Pipeline

Transformers are applied in a **specific order** because some depend on others:

```
1. ExpressionLifter      (lifts complex expressions to CTEs)
   ↓
2. WhereAliasExpander    (expands aliases in WHERE)
   ↓
3. GroupByAliasExpander  (expands aliases in GROUP BY)
   ↓
4. HavingAliasTransformer (adds aliases for aggregates, rewrites HAVING)
   ↓
5. CTEHoister            (hoists nested CTEs to top level)
   ↓
6. InOperatorLifter      (optimizes large IN lists)
```

**Order matters!** For example:
- `ExpressionLifter` must run **before** `CTEHoister` (it creates CTEs that might need hoisting)
- `HavingAliasTransformer` must run **after** GROUP BY expansion (needs to see final GROUP BY structure)

## Configuration

### CLI Flags (Both `-q` and `-f` modes)

```bash
# Disable individual transformers
./target/release/sql-cli -q "SELECT ..." --no-expression-lifter
./target/release/sql-cli -q "SELECT ..." --no-where-expansion
./target/release/sql-cli -q "SELECT ..." --no-group-by-expansion
./target/release/sql-cli -q "SELECT ..." --no-having-expansion
./target/release/sql-cli -q "SELECT ..." --no-cte-hoister
./target/release/sql-cli -q "SELECT ..." --no-in-lifter

# Combine multiple disables
./target/release/sql-cli -q "SELECT ..." --no-expression-lifter --no-where-expansion

# Show preprocessing steps (currently limited output)
./target/release/sql-cli -q "SELECT ..." --show-preprocessing
```

### Programmatic Configuration

```rust
use crate::query_plan::TransformerConfig;

// All enabled (default)
let config = TransformerConfig::all_enabled();

// Custom configuration
let config = TransformerConfig {
    enable_expression_lifter: true,
    enable_where_expansion: true,
    enable_group_by_expansion: true,
    enable_having_expansion: true,
    enable_cte_hoister: false,  // Disable this one
    enable_in_lifter: true,
};

// Create pipeline with config
let mut pipeline = create_pipeline_with_config(false, config);
let transformed = pipeline.process(statement)?;
```

## Testing Transformers

### Test File
See `examples/expander_rewriters.sql` for comprehensive examples of all transformers.

### Quick Tests

```bash
# Test WHERE alias expansion
./target/release/sql-cli -q "SELECT sales_amount * 1.1 AS adj FROM sales WHERE adj > 15000" -d data/sales_data.csv

# Test GROUP BY alias expansion
./target/release/sql-cli -q "SELECT UPPER(region) AS r, SUM(sales_amount) AS t FROM sales GROUP BY r" -d data/sales_data.csv

# Test HAVING auto-aliasing
./target/release/sql-cli -q "SELECT region, SUM(sales_amount) FROM sales GROUP BY region HAVING SUM(sales_amount) > 50000" -d data/sales_data.csv

# Test expression lifter (window function in WHERE)
./target/release/sql-cli -q "SELECT region, ROW_NUMBER() OVER (ORDER BY sales_amount) AS rn FROM sales WHERE ROW_NUMBER() OVER (ORDER BY sales_amount) <= 5" -d data/sales_data.csv

# Run all examples
./target/release/sql-cli -f examples/expander_rewriters.sql
```

## Unified Execution Modes

**IMPORTANT:** As of the execution mode unification (Phases 0-3), both `-q` (single query) and `-f` (script) modes use the **same transformer pipeline**.

This means:
- ✅ All transformers work in both modes
- ✅ Same flags control both modes
- ✅ Consistent behavior everywhere
- ✅ Neovim plugin can use same transformers

**No more mode-specific features!**

## Adding New Transformers

To add a new transformer:

1. **Create transformer file** in `src/query_plan/your_transformer.rs`
2. **Implement `AstTransformer` trait**
3. **Add flag** to `NonInteractiveConfig` in `src/non_interactive.rs`
4. **Add to `TransformerConfig`** in `src/query_plan/mod.rs`
5. **Wire into pipeline** in `create_pipeline_with_config()`
6. **Add tests** in `tests/`
7. **Update this document**
8. **Add example** to `examples/expander_rewriters.sql`

**Remember:** New transformers should be **enabled by default** unless they have compatibility concerns.

## SQL Feature Gaps & Future Transformers

### Currently Missing (Could Add with Transformers)

1. **DISTINCT in aggregates** - `COUNT(DISTINCT column)`
   - Could lift to CTE with DISTINCT, then count

2. **Correlated subqueries** - `WHERE x IN (SELECT y FROM other WHERE other.z = main.z)`
   - Could rewrite as JOIN or EXISTS

3. **LATERAL joins** - `FROM table1, LATERAL (SELECT ... FROM table2 WHERE table2.id = table1.id)`
   - Could rewrite as correlated subquery or window function

4. **Recursive CTEs** - `WITH RECURSIVE ...`
   - Would need iterator/recursion support in executor

5. **PIVOT/UNPIVOT** - Transpose rows to columns
   - Could generate CASE expressions

6. **QUALIFY clause** - Filter on window functions (Snowflake syntax)
   - Essentially `WHERE` but for window functions (we have ExpressionLifter for this!)

7. **Computed columns in CTEs** - More aggressive lifting
   - Extract complex expressions to earlier CTEs

8. **Aggregate expressions in ORDER BY** - `ORDER BY SUM(amount) DESC`
   - Similar to HAVING transformer

### Priority Ranking

**High Priority** (standard SQL, high user value):
1. DISTINCT in aggregates
2. Correlated subqueries (basic)
3. Aggregate expressions in ORDER BY

**Medium Priority** (useful, but workarounds exist):
4. PIVOT/UNPIVOT (can be done with CASE)
5. QUALIFY clause (can use ExpressionLifter)

**Low Priority** (complex, limited use cases):
6. LATERAL joins
7. Recursive CTEs
8. Advanced correlated subqueries

## Performance Considerations

### Overhead
- Each transformer pass requires AST traversal
- Typical cost: ~1-2ms for complex queries
- Acceptable for interactive use

### Benefits
- Preprocessed queries often **faster** than naive execution
- Example: HAVING transformer avoids recomputing aggregates
- Example: IN lifter can enable index usage (future)

### When to Disable
- **Never disable for compatibility** - fix the transformer instead
- **Only disable for performance** - if profiling shows specific transformer is slow
- **Temporary debugging** - isolate which transformer causes an issue

## Debugging

### See Transformed SQL

Currently `--show-preprocessing` has limited output. To see the actual transformed AST:

```bash
# Use --query-plan to see the AST
./target/release/sql-cli -q "SELECT ..." --query-plan

# Enable debug logging
RUST_LOG=debug ./target/release/sql-cli -q "SELECT ..." 2>&1 | grep -i transform
```

### Isolate Transformer Issues

```bash
# Disable all except one
./target/release/sql-cli -q "SELECT ..." \
    --no-where-expansion \
    --no-group-by-expansion \
    --no-having-expansion \
    --no-cte-hoister \
    --no-in-lifter
    # Only ExpressionLifter is running

# Test with no transformers
./target/release/sql-cli -q "SELECT ..." \
    --no-expression-lifter \
    --no-where-expansion \
    --no-group-by-expansion \
    --no-having-expansion \
    --no-cte-hoister \
    --no-in-lifter
```

## Summary

Preprocessor transformers are the **secret sauce** that makes SQL CLI powerful:

- ✅ **Enabled by default** - queries "just work"
- ✅ **Fill feature gaps** - support SQL that executor can't handle
- ✅ **Unified across modes** - `-q`, `-f`, and TUI all use same pipeline
- ✅ **Extensible** - easy to add new transformers
- ✅ **Opt-out** - disable if needed, but shouldn't be necessary

**The goal:** Support as much standard SQL as possible by rewriting it into forms our executor can handle, without users needing to think about it.
