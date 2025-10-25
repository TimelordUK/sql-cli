# AST Transformation Ordering

This document explains the order in which AST transformers are applied in the preprocessing pipeline and why that order matters.

## Current Standard Pipeline

The standard pipeline (created by `create_standard_pipeline()` in `src/query_plan/mod.rs`) applies transformers in this order:

1. **ExpressionLifter** - Lifts window functions and column alias dependencies to CTEs
2. **CTEHoister** - Hoists nested WITH clauses to top level
3. **InOperatorLifter** - Optimizes large IN expressions by lifting to CTEs

## Why Order Matters

### 1. ExpressionLifter Must Run First

**Why:** Window functions and complex expressions need to be lifted into CTEs before any other transformations. This is because:

- Window functions can't be used in WHERE clauses or GROUP BY
- Column alias dependencies need to be resolved before other optimizations
- Creates a clean, normalized AST for subsequent transformers

**Example:**
```sql
-- Input
SELECT
    amount,
    ROW_NUMBER() OVER (ORDER BY amount) as rn
FROM sales
WHERE amount > 100

-- After ExpressionLifter (creates intermediate CTE)
WITH __lifted_1 AS (
    SELECT
        amount,
        ROW_NUMBER() OVER (ORDER BY amount) as rn
    FROM sales
)
SELECT * FROM __lifted_1 WHERE amount > 100
```

### 2. CTEHoister Runs Second

**Why:** After ExpressionLifter creates new CTEs, we may have nested CTEs that need to be flattened:

- ExpressionLifter might create CTEs inside queries that already have CTEs
- Nested CTEs are harder to optimize and execute
- Hoisting brings all CTEs to the top level for better visibility

**Example:**
```sql
-- After ExpressionLifter might produce nested CTEs
WITH outer_cte AS (
    SELECT * FROM (
        WITH inner_cte AS (SELECT 1 as x)
        SELECT * FROM inner_cte
    )
)
SELECT * FROM outer_cte

-- After CTEHoister (flattened)
WITH inner_cte AS (SELECT 1 as x),
     outer_cte AS (SELECT * FROM inner_cte)
SELECT * FROM outer_cte
```

### 3. InOperatorLifter Runs Last

**Why:** IN expression optimization should happen after structural transformations:

- Works on the clean, normalized AST
- Doesn't interfere with window function lifting
- Can safely lift IN expressions to CTEs knowing the structure is stable

**Example:**
```sql
-- Input (after previous transformations)
SELECT * FROM sales
WHERE product_id IN (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)

-- After InOperatorLifter (if threshold met)
WITH __in_values_1 AS (
    SELECT 1 as value UNION ALL
    SELECT 2 UNION ALL
    -- ... (11 values)
)
SELECT * FROM sales
WHERE product_id IN (SELECT value FROM __in_values_1)
```

## Transformation Dependencies

```
ExpressionLifter
    ↓ (may create nested CTEs)
CTEHoister
    ↓ (flattens structure)
InOperatorLifter
    ↓ (optimizes on clean AST)
Final AST
```

## Adding New Transformers

When adding new transformers, consider these guidelines:

### Early Stage (before ExpressionLifter)
- Very rare! Only for transformations that need to see the original query structure
- Must be able to handle window functions in expressions
- Example: Syntax sugar expansion that doesn't affect semantics

### Middle Stage (between ExpressionLifter and CTEHoister)
- Transformations that work with expressions but don't create CTEs
- Must handle window functions that are now in CTEs
- Example: Expression simplification, constant folding

### Late Stage (after CTEHoister, before InOperatorLifter)
- Transformations that benefit from flat CTE structure
- May create new CTEs (which are already at top level)
- Example: Correlated subquery rewriting (Phase 3)

### Final Stage (after InOperatorLifter)
- Optimizations that work on fully normalized queries
- Should not create new CTEs
- Example: Predicate pushdown, join reordering

## Phase 0 Configuration

Currently in Phase 0, the pipeline is configured to:

- ✅ Apply all three existing transformers in correct order
- ✅ Collect statistics on transformation duration
- ✅ Support verbose logging via `--show-preprocessing` flag
- ✅ Handle errors gracefully (fall back to original AST if transformation fails)

## Future Phases

### Phase 1 Additions (Quick Wins)
- **HAVING Auto-aliaser** - Insert between ExpressionLifter and CTEHoister
  - Adds implicit aliases to aggregate functions in HAVING clauses
  - Must run after expression lifting but before CTE hoisting

- **SELECT * Expander** - Run very early (before ExpressionLifter)
  - Expands `SELECT *` to explicit column names
  - Helps other transformers by making column references explicit

### Phase 2 Additions (Parallel CTEs)
- **CTE Dependency Analyzer** - Run after CTEHoister
  - Analyzes CTE dependencies to identify parallelizable CTEs
  - Doesn't modify AST, just annotates for executor

### Phase 3 Additions (Correlated Subqueries)
- **Correlated Subquery Rewriter** - Run after CTEHoister
  - Converts correlated subqueries to CTEs + JOINs
  - Must run after all CTEs are hoisted to top level
  - May create additional CTEs (which are already normalized)

## Testing Transformation Order

The integration test `tests/integration/test_preprocessing_pipeline.sh` verifies:

1. All transformers are invoked in correct order
2. Statistics show each transformer's execution time
3. Transformations are applied correctly
4. Pipeline handles errors gracefully

Run with:
```bash
./tests/integration/test_preprocessing_pipeline.sh
```

## Debugging

To see transformation order in action:

```bash
# Show preprocessing statistics
./target/release/sql-cli data/test.csv \
    -q "YOUR_QUERY" \
    --check-in-lifting \
    --show-preprocessing

# Output will show:
# === Preprocessing Pipeline ===
# 3 transformer(s) applied in 0.04ms
#   ExpressionLifter - 0.02ms
#   CTEHoister - 0.01ms
#   InOperatorLifter - 0.01ms
```

## References

- Pipeline implementation: `src/query_plan/pipeline.rs`
- Transformer adapters: `src/query_plan/transformer_adapters.rs`
- Standard pipeline factory: `src/query_plan/mod.rs` (lines 52-74)
- Individual transformers:
  - ExpressionLifter: `src/query_plan/expression_lifter.rs`
  - CTEHoister: `src/query_plan/cte_hoister.rs`
  - InOperatorLifter: `src/query_plan/in_operator_lifter.rs`
