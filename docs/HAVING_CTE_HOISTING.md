# HAVING Clause Auto-Hoisting into CTEs

## Problem Statement

Currently, HAVING clauses with aggregate function calls don't work:

```sql
-- BROKEN: Aggregate functions in HAVING not recognized
SELECT value % 3 as grp, COUNT(*) as cnt
FROM RANGE(1, 10)
GROUP BY value % 3
HAVING COUNT(*) > 2;
-- Error: Column 'value' not found

-- WORKAROUND: Use aliases instead
SELECT value % 3 as grp, COUNT(*) as cnt
FROM RANGE(1, 10)
GROUP BY value % 3
HAVING cnt > 2;
-- Works!
```

The issue: When evaluating HAVING, we're working with a temp table containing only the aggregated results (grp, cnt). The original columns and aggregate function definitions are no longer available.

## Root Cause

The HAVING clause is evaluated against the post-GROUP BY result set, which only contains:
1. GROUP BY expressions (or their aliases)
2. Aggregate result columns (their aliases)

When the HAVING clause contains `COUNT(*)` or `SUM(value)`, the evaluator tries to:
- Look up COUNT as a function → succeeds
- Evaluate COUNT(*) against the temp table → fails because `value` column doesn't exist

Standard SQL expects HAVING to re-evaluate aggregates or reference their aliases, but our current architecture doesn't support re-evaluation.

## Proposed Solution: Auto-Hoist HAVING into CTEs

Transform queries with HAVING clauses into CTEs that separate aggregation from filtering.

### Transformation Examples

#### Example 1: Simple COUNT
**Input:**
```sql
SELECT value % 3 as grp, COUNT(*) as cnt
FROM RANGE(1, 10)
GROUP BY value % 3
HAVING COUNT(*) > 2;
```

**Auto-transformed to:**
```sql
WITH __having_cte_1 AS (
    SELECT value % 3 as grp, COUNT(*) as cnt
    FROM RANGE(1, 10)
    GROUP BY value % 3
)
SELECT grp, cnt
FROM __having_cte_1
WHERE cnt > 2;
```

#### Example 2: Multiple Aggregates
**Input:**
```sql
SELECT dept, COUNT(*) as emp_count, AVG(salary) as avg_sal
FROM employees
GROUP BY dept
HAVING COUNT(*) > 5 AND AVG(salary) > 50000;
```

**Auto-transformed to:**
```sql
WITH __having_cte_1 AS (
    SELECT dept, COUNT(*) as emp_count, AVG(salary) as avg_sal
    FROM employees
    GROUP BY dept
)
SELECT dept, emp_count, avg_sal
FROM __having_cte_1
WHERE emp_count > 5 AND avg_sal > 50000;
```

#### Example 3: Complex Expressions
**Input:**
```sql
SELECT category, SUM(amount) as total
FROM sales
GROUP BY category
HAVING SUM(amount) / COUNT(*) > 100;
```

**Auto-transformed to:**
```sql
WITH __having_cte_1 AS (
    SELECT category, SUM(amount) as total, COUNT(*) as __count_1
    FROM sales
    GROUP BY category
)
SELECT category, total
FROM __having_cte_1
WHERE total / __count_1 > 100;
```

Note: Need to add `COUNT(*)` to SELECT list with auto-generated alias, then hide it from final output.

## Implementation Strategy

### Phase 1: HAVING Detection and Analysis

**File:** `src/sql/rewriter/having_hoister.rs` (new)

```rust
pub struct HavingHoister;

impl HavingHoister {
    /// Detects if a SELECT statement has a HAVING clause
    pub fn needs_hoisting(stmt: &SelectStatement) -> bool {
        stmt.having.is_some()
    }

    /// Analyzes HAVING clause to identify:
    /// - Aggregate function calls
    /// - Column references to aliases
    /// - Expressions that mix both
    fn analyze_having(having: &SqlExpression) -> HavingAnalysis {
        // Walk the expression tree
        // Identify aggregate calls: COUNT(*), SUM(x), AVG(y), etc.
        // Identify column references
        // Return struct with both lists
    }
}

struct HavingAnalysis {
    aggregate_calls: Vec<AggregateFunctionCall>,
    column_refs: Vec<String>,
    has_complex_expr: bool,
}
```

### Phase 2: Aggregate-to-Alias Mapping

**Challenge:** Map aggregate function calls in HAVING to their SELECT list aliases.

**Approach:**
1. Scan SELECT list for aggregate expressions
2. For each aggregate in HAVING, find matching aggregate in SELECT
3. If match found, replace with alias reference
4. If no match, add aggregate to SELECT with auto-generated alias

**Example:**
```sql
SELECT dept, COUNT(*) as cnt
HAVING COUNT(*) > 5

-- COUNT(*) in HAVING matches COUNT(*) in SELECT with alias "cnt"
-- Replace: HAVING COUNT(*) > 5 → WHERE cnt > 5
```

```sql
SELECT dept, COUNT(*) as cnt
HAVING SUM(salary) > 100000

-- SUM(salary) not in SELECT list
-- Add to SELECT: COUNT(*) as cnt, SUM(salary) as __sum_1
-- Replace: HAVING SUM(salary) > 100000 → WHERE __sum_1 > 100000
-- Hide __sum_1 from final output
```

### Phase 3: CTE Generation

**File:** `src/sql/rewriter/having_hoister.rs`

```rust
impl HavingHoister {
    pub fn hoist_to_cte(stmt: SelectStatement) -> SelectStatement {
        // 1. Analyze HAVING clause
        let analysis = Self::analyze_having(stmt.having.unwrap());

        // 2. Build aggregate mapping
        let (alias_map, augmented_select) = Self::map_aggregates_to_aliases(
            &stmt.select_list,
            &analysis
        );

        // 3. Rewrite HAVING expression using aliases
        let where_expr = Self::rewrite_having_to_where(
            stmt.having.unwrap(),
            &alias_map
        );

        // 4. Create CTE with augmented SELECT
        let cte_name = Self::generate_cte_name(); // "__having_cte_1"
        let cte = CommonTableExpression {
            name: cte_name.clone(),
            statement: Box::new(SelectStatement {
                select_list: augmented_select,
                from: stmt.from,
                where_clause: stmt.where_clause,
                group_by: stmt.group_by,
                having: None, // HAVING moved to WHERE
                order_by: stmt.order_by,
                limit: stmt.limit,
                offset: stmt.offset,
            }),
        };

        // 5. Build outer SELECT from CTE
        SelectStatement {
            with: Some(vec![cte]),
            select_list: Self::filter_hidden_columns(stmt.select_list),
            from: Some(TableReference::Table(cte_name)),
            where_clause: Some(where_expr),
            group_by: None,
            having: None,
            order_by: stmt.order_by, // Preserve ORDER BY
            limit: stmt.limit,       // Preserve LIMIT
            offset: stmt.offset,     // Preserve OFFSET
        }
    }
}
```

### Phase 4: Column Hiding (Optional Enhancement)

For auto-generated aggregate aliases (e.g., `__count_1`), we may want to hide them from final output:

```rust
struct SelectItem {
    expression: SqlExpression,
    alias: Option<String>,
    hidden: bool,  // New field
}
```

Alternatively, reconstruct SELECT list without hidden columns.

## Integration Points

### 1. Query Preprocessor Entry Point

**File:** `src/sql/rewriter/mod.rs`

```rust
pub fn preprocess_query(stmt: SelectStatement) -> Result<SelectStatement> {
    let mut stmt = stmt;

    // Existing rewriters...
    stmt = window_hoister::hoist_window_functions(stmt)?;

    // NEW: HAVING hoister
    if HavingHoister::needs_hoisting(&stmt) {
        stmt = HavingHoister::hoist_to_cte(stmt)?;
    }

    Ok(stmt)
}
```

### 2. Aggregate Function Detection

**File:** `src/sql/rewriter/aggregate_detector.rs` (new or existing)

```rust
pub fn is_aggregate_function(func_name: &str) -> bool {
    matches!(
        func_name.to_uppercase().as_str(),
        "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "STDDEV" | "VARIANCE"
    )
}

pub fn extract_aggregates(expr: &SqlExpression) -> Vec<AggregateFunctionCall> {
    // Walk expression tree
    // Collect all aggregate function calls
    // Return list with function name, arguments, and position
}
```

## Edge Cases and Considerations

### 1. HAVING with Only Alias References
```sql
SELECT dept, COUNT(*) as cnt
GROUP BY dept
HAVING cnt > 5;
```

**Decision:** Still hoist? Or detect this case and skip hoisting?

**Recommendation:** Skip hoisting if HAVING only references aliases (optimization).

### 2. HAVING with Mixed References
```sql
SELECT dept, COUNT(*) as cnt
GROUP BY dept
HAVING cnt > 5 AND SUM(salary) > 100000;
```

**Must hoist:** Add `SUM(salary)` to SELECT, rewrite both conditions.

### 3. HAVING with GROUP BY Column References
```sql
SELECT dept, COUNT(*) as cnt
GROUP BY dept
HAVING dept LIKE 'Sales%';
```

**Can be optimized to WHERE:** Move condition to WHERE clause instead of HAVING.

**Advanced:** Detect GROUP BY column refs in HAVING and push down to WHERE (optimization).

### 4. Nested Aggregates (Not Standard SQL)
```sql
HAVING COUNT(DISTINCT category) > 3
```

**Handle like any other aggregate:** Map or add to SELECT.

### 5. ORDER BY/LIMIT/OFFSET Preservation
Ensure these clauses are preserved correctly:
- If in original query, they apply to final result
- Should be on outer SELECT, not inner CTE

## Testing Strategy

### Unit Tests

**File:** `tests/having_hoisting_tests.rs`

```rust
#[test]
fn test_simple_count_having() {
    let sql = "SELECT value % 3 as grp, COUNT(*) as cnt \
               FROM RANGE(1, 10) GROUP BY value % 3 HAVING COUNT(*) > 2";

    let result = execute_query(sql);
    assert_eq!(result.rows.len(), 2); // Only groups with count > 2
}

#[test]
fn test_aggregate_added_to_select() {
    let sql = "SELECT dept, COUNT(*) as cnt \
               GROUP BY dept HAVING SUM(salary) > 100000";

    // Verify SUM(salary) was added to SELECT
    // Verify it's hidden from final output
}

#[test]
fn test_mixed_alias_and_aggregate() {
    let sql = "SELECT category, SUM(amount) as total \
               GROUP BY category HAVING total > 1000 AND COUNT(*) > 5";

    // Verify both conditions work
}
```

### Integration Tests

**File:** `tests/python_tests/test_having_hoisting.py`

```python
def test_having_count_star():
    result = run_query("""
        SELECT value % 3 as grp, COUNT(*) as cnt
        FROM RANGE(1, 10)
        GROUP BY value % 3
        HAVING COUNT(*) > 2
    """)
    assert len(result) == 2

def test_having_with_avg():
    result = run_query("""
        SELECT dept, AVG(salary) as avg_sal
        FROM employees
        GROUP BY dept
        HAVING AVG(salary) > 50000
    """)
    # Verify results
```

### End-to-End Examples

**File:** `examples/having_clause_examples.sql`

```sql
-- #! ../data/sales_data.csv

-- Example 1: Simple COUNT
SELECT region, COUNT(*) as order_count
FROM sales_data
GROUP BY region
HAVING COUNT(*) > 100;
GO

-- Example 2: AVG filter
SELECT product, AVG(price) as avg_price
FROM sales_data
GROUP BY product
HAVING AVG(price) > 50;
GO

-- Example 3: Complex expression
SELECT category, SUM(quantity) as total_qty
FROM sales_data
GROUP BY category
HAVING SUM(quantity * price) > 10000;
GO
```

## Future Enhancements

### 1. Optimization: HAVING → WHERE Pushdown
If HAVING only references GROUP BY columns, push condition to WHERE clause:

```sql
-- Before
SELECT dept, COUNT(*) FROM emp GROUP BY dept HAVING dept = 'Sales';

-- Optimize to
SELECT dept, COUNT(*) FROM emp WHERE dept = 'Sales' GROUP BY dept;
```

### 2. Better Error Messages
If user writes invalid HAVING (references non-existent column), provide helpful error:

```
Error: HAVING clause references column 'xyz' which is not in GROUP BY or SELECT list.
Did you mean one of: [dept, cnt, avg_salary]?
```

### 3. QUALIFY Clause Support
Standard SQL has QUALIFY for window function filtering, similar to HAVING for aggregates:

```sql
SELECT emp, salary, RANK() OVER (ORDER BY salary DESC) as rnk
QUALIFY rnk <= 10;
```

Similar hoisting strategy could apply.

## Migration Path

### Phase 1: Implement Basic Hoisting
- Detect HAVING with aggregates
- Hoist to CTE with simple mapping
- Get basic cases working

### Phase 2: Handle Complex Cases
- Mixed alias/aggregate HAVING
- Aggregate expressions (SUM/COUNT)
- Auto-generated aliases

### Phase 3: Optimizations
- Skip hoisting for alias-only HAVING
- Push down GROUP BY column refs to WHERE
- Better column hiding

### Phase 4: Polish
- Error messages
- Documentation
- Performance testing

## Dependencies and Prerequisites

### Existing Code to Leverage
1. **CTE infrastructure** - Already have CTE support
2. **Window function hoisting** - Similar pattern, can reuse approach
3. **Expression rewriting** - Need to walk/transform expression trees
4. **Alias resolution** - Already handle aliases in various contexts

### Potential Blockers
1. **Column scoping across joins** - You mentioned this was recently fixed
2. **Expression tree walking** - Need robust visitor pattern for HAVING expressions
3. **Aggregate detection** - Need to reliably identify aggregate function calls

### Testing Requirements
1. **Query executor** - Must support CTEs (already does)
2. **Test data** - Need datasets with GROUP BY scenarios (have sales_data.csv)
3. **Python test infrastructure** - Already in place

## Open Questions

1. **CTE naming:** Use `__having_cte_N` with counter? Or hash-based names?
2. **Column hiding:** Implement hidden column support, or reconstruct SELECT list?
3. **Error handling:** If HAVING can't be hoisted (malformed), fall through or error?
4. **Optimization toggles:** Allow users to disable hoisting (for debugging)?
5. **Query plan output:** Show original + transformed query in `--query-plan`?

## References

- **Window function hoisting:** `src/sql/rewriter/window_hoister.rs` (if exists)
- **CTE support:** `src/sql/parser/ast.rs` - `CommonTableExpression`
- **Expression evaluation:** `src/data/arithmetic_evaluator.rs`
- **SELECT statement structure:** `src/sql/parser/ast.rs` - `SelectStatement`

---

**Status:** Design document - Not yet implemented
**Priority:** Medium - Workaround exists (use aliases), but full SQL compatibility desired
**Effort:** High - Requires careful expression analysis and rewriting
**Dependencies:** None - Can be implemented now that column scoping is fixed
