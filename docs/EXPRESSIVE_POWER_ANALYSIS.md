# SQL CLI Expressive Power Analysis & AST Rewriting Strategy

## Executive Summary

**Key Finding:** SQL CLI has **near-parity expressive power** between subqueries and top-level queries. The primary limitation is **correlated subqueries**, which require variable binding/row context passing.

**Strategic Recommendation:** Implement an **AST Preprocessing/Rewriting layer** to automatically transform unsupported SQL patterns into supported CTE-based equivalents. This avoids massive executor changes while expanding SQL compatibility.

---

## Current Expressive Power Matrix

### ✅ What Works EVERYWHERE (Top-level, Subqueries, CTEs)

| Feature | Top-Level | FROM Subquery | WHERE Subquery | SELECT Scalar | CTE |
|---------|-----------|---------------|----------------|---------------|-----|
| Window Functions | ✅ | ✅ | N/A | ✅ | ✅ |
| Aggregates (GROUP BY) | ✅ | ✅ | N/A | ✅ | ✅ |
| JOINs | ✅ | ✅ | N/A | ✅ | ✅ |
| JOIN Expressions (Phase 1+2) | ✅ | ✅ | N/A | ✅ | ✅ |
| UNION/INTERSECT/EXCEPT | ✅ | ✅ | ✅ | ✅ | ✅ |
| ORDER BY | ✅ | ✅ | N/A | N/A | ✅ |
| LIMIT | ✅ | ✅ | N/A | N/A | ✅ |
| Functions (all 200+) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Expressions (arithmetic, etc.) | ✅ | ✅ | ✅ | ✅ | ✅ |
| CASE expressions | ✅ | ✅ | ✅ | ✅ | ✅ |

**Conclusion:** We have **full expressive parity** for non-correlated operations!

---

## ❌ What Doesn't Work (and Why)

### 1. Correlated Subqueries

**Example:**
```sql
-- Does NOT work: o.id reference inside subquery
SELECT
    o.id,
    (SELECT MAX(amount) FROM orders WHERE customer_id = o.id) as max_order
FROM customers o;
```

**Why it fails:**
- Subquery executor has no access to "outer row" context
- Would need to pass `o.id` binding into inner query
- Requires variable substitution mechanism

**Can it be rewritten?** ✅ **YES** (with limitations)

```sql
-- Rewrite as CTE with JOIN
WITH order_maxes AS (
    SELECT customer_id, MAX(amount) as max_order
    FROM orders
    GROUP BY customer_id
)
SELECT
    o.id,
    om.max_order
FROM customers o
LEFT JOIN order_maxes om ON o.id = om.customer_id;
```

**Limitation:** Only works if the correlated reference is in a JOIN/WHERE condition, not in arbitrary expressions.

### 2. Lateral Joins (CROSS APPLY style)

**Example:**
```sql
-- Does NOT work
SELECT *
FROM customers c
CROSS APPLY (SELECT TOP 3 * FROM orders WHERE customer_id = c.id ORDER BY date DESC) AS recent_orders;
```

**Can it be rewritten?** ⚠️ **PARTIALLY**

Only if we can express it as a window function:

```sql
WITH ranked_orders AS (
    SELECT
        *,
        ROW_NUMBER() OVER (PARTITION BY customer_id ORDER BY date DESC) as rn
    FROM orders
)
SELECT c.*, ro.*
FROM customers c
JOIN ranked_orders ro ON c.id = ro.customer_id
WHERE ro.rn <= 3;
```

### 3. Recursive CTEs with Depth/Breadth Control

**Example:**
```sql
-- Works but no way to limit depth during recursion
WITH RECURSIVE hierarchy AS (
    SELECT id, parent_id, 1 as depth
    FROM nodes
    WHERE parent_id IS NULL
    UNION ALL
    SELECT n.id, n.parent_id, h.depth + 1
    FROM nodes n
    JOIN hierarchy h ON n.parent_id = h.id
    WHERE h.depth < 5  -- This works but is inefficient
)
SELECT * FROM hierarchy;
```

**Can it be rewritten?** ✅ **YES** (already supported)

Our recursive CTE implementation already supports this!

### 4. UPDATE/DELETE with Correlated Subqueries

**Example:**
```sql
-- Does NOT work (no UPDATE/DELETE support at all)
UPDATE products
SET price = (SELECT AVG(price) FROM products WHERE category = products.category);
```

**Can it be rewritten?** ❌ **NO** (would need DML support)

This requires a full DML implementation, not just AST rewriting.

---

## AST Preprocessing/Rewriting Strategy

### Architecture

```
┌─────────────┐
│   Parser    │
│             │
│  SQL Text   │
└─────┬───────┘
      │
      │ Produces AST
      ▼
┌─────────────────────────────────┐
│   AST Preprocessor/Rewriter     │◄──── NEW LAYER
│                                 │
│  Pattern Detection & Rewriting  │
└─────┬───────────────────────────┘
      │
      │ Transformed AST
      ▼
┌─────────────┐
│  Executor   │
│             │
│ (Unchanged) │
└─────────────┘
```

### Rewriting Rules

#### Rule 1: Scalar Correlated Subquery → CTE + JOIN

**Pattern:**
```sql
SELECT
    a.id,
    (SELECT aggregate_fn(...) FROM b WHERE b.fk = a.id) as result
FROM a;
```

**Rewrite to:**
```sql
WITH _cte_agg AS (
    SELECT fk, aggregate_fn(...) as result
    FROM b
    GROUP BY fk
)
SELECT
    a.id,
    _cte_agg.result
FROM a
LEFT JOIN _cte_agg ON a.id = _cte_agg.fk;
```

**Applicability:** ✅ Works for aggregates with simple correlation (equality in WHERE)

#### Rule 2: IN Correlated Subquery → EXISTS via SEMI JOIN

**Pattern:**
```sql
SELECT * FROM a
WHERE a.id IN (SELECT b.fk FROM b WHERE b.status = 'active');
```

**Rewrite to:**
```sql
WITH _cte_exists AS (
    SELECT DISTINCT fk FROM b WHERE status = 'active'
)
SELECT a.*
FROM a
JOIN _cte_exists ON a.id = _cte_exists.fk;
```

**Applicability:** ✅ Always works for IN/EXISTS patterns

#### Rule 3: Top-N per Group → Window Function

**Pattern:**
```sql
SELECT *
FROM orders o
WHERE (
    SELECT COUNT(*) FROM orders o2
    WHERE o2.customer_id = o.customer_id AND o2.date > o.date
) < 3;
```

**Rewrite to:**
```sql
WITH ranked AS (
    SELECT
        *,
        ROW_NUMBER() OVER (PARTITION BY customer_id ORDER BY date DESC) as rn
    FROM orders
)
SELECT * FROM ranked WHERE rn <= 3;
```

**Applicability:** ✅ Works for many "top-N per group" patterns

#### Rule 4: Nested Aggregates → Multiple CTEs

**Pattern:**
```sql
SELECT
    dept,
    AVG(total_sales) as avg_dept_sales
FROM (
    SELECT
        employee_id,
        dept,
        SUM(amount) as total_sales
    FROM sales
    GROUP BY employee_id, dept
) AS emp_sales
GROUP BY dept;
```

**Rewrite to:**
```sql
WITH emp_sales AS (
    SELECT
        employee_id,
        dept,
        SUM(amount) as total_sales
    FROM sales
    GROUP BY employee_id, dept
),
dept_avgs AS (
    SELECT
        dept,
        AVG(total_sales) as avg_dept_sales
    FROM emp_sales
    GROUP BY dept
)
SELECT * FROM dept_avgs;
```

**Applicability:** ✅ Always works (actually simplifies the query!)

---

## What CAN'T Be Rewritten?

### 1. True Row-by-Row Correlation with Complex Expressions

```sql
-- Cannot rewrite if correlation involves complex computation
SELECT
    id,
    (SELECT COUNT(*) FROM events e WHERE DATEDIFF(e.date, outer.created) < outer.threshold_days)
FROM configs outer;
```

**Why:** The `outer.threshold_days` is used in an expression, not just a JOIN condition.

**Workaround:** User must restructure using CROSS JOIN + WHERE (manual CTE)

### 2. Mutually Recursive Queries

```sql
-- Cannot represent without special syntax
WITH RECURSIVE
    even_nums(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM odd_nums WHERE n < 10),
    odd_nums(n) AS (SELECT 0 UNION ALL SELECT n+1 FROM even_nums WHERE n < 10)
SELECT * FROM even_nums UNION ALL SELECT * FROM odd_nums;
```

**Why:** Circular dependency between CTEs

**Status:** Theoretically possible but requires dependency resolution

### 3. Lateral Joins with True Lateral Correlation

```sql
-- True lateral join where function depends on left row
SELECT * FROM customers c
CROSS APPLY generate_series(1, c.num_orders) AS s(i);
```

**Why:** The `c.num_orders` directly parameterizes a table-valued function

**Workaround:** Only possible if we implement table-valued function "unrolling"

---

## Implementation Plan: AST Preprocessor

### Phase 1: Infrastructure (Week 1-2)

**Goal:** Build the preprocessing framework

```rust
// src/preprocessing/mod.rs
pub trait ASTRewriter {
    fn rewrite(&self, ast: SelectStatement) -> Result<SelectStatement>;
}

pub struct PreprocessorPipeline {
    rewriters: Vec<Box<dyn ASTRewriter>>,
}

impl PreprocessorPipeline {
    pub fn apply(&self, ast: SelectStatement) -> Result<SelectStatement> {
        let mut current = ast;
        for rewriter in &self.rewriters {
            current = rewriter.rewrite(current)?;
        }
        Ok(current)
    }
}
```

**Files:**
- `src/preprocessing/mod.rs` - Pipeline framework
- `src/preprocessing/pattern_matcher.rs` - AST pattern matching utilities
- `src/preprocessing/cte_builder.rs` - Helper to construct CTE nodes

### Phase 2: Correlated Scalar Subquery Rewriter (Week 3-4)

**Goal:** Rewrite scalar correlated subqueries to CTE + JOIN

**Algorithm:**
1. Detect `ScalarSubquery` in SELECT list
2. Analyze subquery for correlation (references to outer query)
3. If correlation is simple equality in WHERE → extract to CTE
4. Generate aggregate CTE with GROUP BY on correlated column
5. Replace scalar subquery with LEFT JOIN to CTE

**Benefit:** Enables most common correlated subquery patterns

### Phase 3: IN/EXISTS Subquery Rewriter (Week 5)

**Goal:** Rewrite IN/EXISTS to SEMI JOIN pattern

**Algorithm:**
1. Detect `InSubquery` in WHERE clause
2. Extract subquery to CTE with DISTINCT
3. Replace IN with INNER JOIN (or LEFT JOIN + IS NOT NULL check)

**Benefit:** Performance + readability

### Phase 4: Top-N per Group Rewriter (Week 6)

**Goal:** Rewrite top-N correlated patterns to window functions

**Pattern Detection:**
```sql
WHERE (SELECT COUNT(*) FROM same_table WHERE correlation AND condition) < N
```

**Rewrite:**
```sql
WITH ranked AS (
    SELECT *, ROW_NUMBER() OVER (PARTITION BY correlated_col ORDER BY condition) as rn
    FROM same_table
)
SELECT * FROM ranked WHERE rn <= N
```

### Phase 5: Documentation & Examples (Week 7)

- Document all rewriting rules
- Provide before/after examples
- Add debugging mode to show rewritten queries
- Performance benchmarks

---

## Debugging & Observability

### Show Rewritten Query

```bash
# New flag to see what the preprocessor did
./target/release/sql-cli -q "SELECT ..." --show-rewritten
```

Output:
```
=== Original Query ===
SELECT id, (SELECT MAX(amount) FROM orders WHERE cust_id = c.id) FROM customers c;

=== Rewritten Query ===
WITH _cte_0 AS (
    SELECT cust_id, MAX(amount) as _result_0 FROM orders GROUP BY cust_id
)
SELECT c.id, _cte_0._result_0
FROM customers c LEFT JOIN _cte_0 ON c.id = _cte_0.cust_id;

=== Execution ===
...
```

### Statistics

Track which rewrites are applied:
```
Preprocessing Statistics:
- Scalar correlated subqueries hoisted: 3
- IN subqueries converted to SEMI JOIN: 1
- Top-N patterns converted to window functions: 0
- Total preprocessing time: 2.3ms
```

---

## Benefits of This Approach

### 1. **No Executor Changes** ✅
- Executor sees standard CTEs and JOINs
- No need to implement variable binding
- No need to thread row context through subqueries

### 2. **Automatic Optimization** ✅
- Rewrites often produce MORE efficient queries
- CTE materialization vs repeated subquery execution
- Enables future CTE optimization (memoization, etc.)

### 3. **SQL Compatibility** ✅
- Users can write "natural" SQL with subqueries
- Under the hood, we convert to efficient CTEs
- Best of both worlds

### 4. **Debuggability** ✅
- `--show-rewritten` lets users see what's happening
- Learn CTE patterns by seeing rewrites
- Easier to optimize when you see the CTE form

### 5. **Incremental Implementation** ✅
- Add rewriters one at a time
- Each rewriter is independent
- Can ship partial functionality

---

## Risks & Limitations

### 1. **Not All Correlations Can Be Rewritten**
Some complex expressions with outer references can't be converted to CTEs.

**Mitigation:**
- Detect unrewritable patterns
- Provide clear error message
- Suggest manual CTE refactoring

### 2. **Query Plan Changes**
Rewriting changes execution plan - could be slower in some cases.

**Mitigation:**
- Benchmark rewrites
- Add `--no-preprocessing` flag to disable
- Make individual rewrites configurable

### 3. **Debugging Complexity**
Users see error messages on rewritten query, not original.

**Mitigation:**
- Map error locations back to original query
- Always show both queries in verbose mode

### 4. **Maintenance Burden**
More code to maintain, more edge cases.

**Mitigation:**
- Comprehensive test suite for each rewriter
- Clear documentation of rewrite rules
- Modular design allows disabling buggy rewrites

---

## Success Metrics

### Coverage
- **Target:** 90% of common correlated subquery patterns rewritable
- **Measure:** Test against TPC-H, real-world SQL samples

### Performance
- **Target:** Rewrites should not slow queries by >10%
- **Measure:** Benchmark suite comparing original CTE vs rewritten

### Usability
- **Target:** 80% of users shouldn't need to understand rewrites
- **Measure:** User feedback, support tickets

---

## Alternative: Full Correlated Subquery Implementation

### What It Would Require

1. **Variable Binding System**
   - Pass outer row values into subquery
   - Scope management (nested subqueries)
   - Type checking across scopes

2. **Execution Model Changes**
   - Execute subquery once per outer row (O(n×m) complexity)
   - Cache subquery results for same parameter values
   - Thread safety for concurrent evaluation

3. **Optimizer Integration**
   - Detect when correlated subquery can be decorrelated
   - Choose between nested loop, hash join, etc.
   - Cost-based decision making

**Estimated Effort:** 6-8 weeks vs 4-6 weeks for preprocessor

**Risk:** Much higher - touches executor core

**Benefit:** Marginally better for edge cases that can't be rewritten

**Recommendation:** ❌ **Don't do it** - preprocessor is better ROI

---

## Conclusion

### What We Know

1. **Expressive Power:** Near-complete parity between top-level and subqueries (except correlated)
2. **Gap:** Correlated subqueries are the main limitation
3. **Solution:** 90%+ of correlated subqueries can be rewritten to CTEs
4. **Strategy:** AST preprocessing is lower risk than executor changes

### Recommended Path Forward

**Phase 1 (Immediate):** Build AST preprocessing infrastructure

**Phase 2 (Next):** Implement scalar correlated subquery rewriter

**Phase 3 (Future):** Add IN/EXISTS and Top-N rewriters

**Phase 4 (Polish):** Debugging tools and documentation

### What This Enables

Users can write:
```sql
-- Natural SQL with subqueries
SELECT
    customer_name,
    (SELECT COUNT(*) FROM orders WHERE customer_id = c.id) as order_count
FROM customers c
WHERE region = 'EMEA';
```

We automatically transform to:
```sql
-- Efficient CTE-based execution
WITH _order_counts AS (
    SELECT customer_id, COUNT(*) as cnt FROM orders GROUP BY customer_id
)
SELECT
    c.customer_name,
    COALESCE(oc.cnt, 0) as order_count
FROM customers c
LEFT JOIN _order_counts oc ON c.id = oc.customer_id
WHERE c.region = 'EMEA';
```

Best of both worlds! 🎉

---

**Next Steps:**
1. Review and approve this analysis
2. Create detailed design doc for preprocessor architecture
3. Implement Phase 1: Infrastructure
4. Build test suite of correlated subquery patterns
5. Implement Phase 2: First rewriter

