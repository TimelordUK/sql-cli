# SQL Feature Gaps & Transformer Roadmap

## Current State (Post-Unification)

As of the execution mode unification completion, we have:

✅ **Solid Foundation:**
- Unified execution path (`StatementExecutor` used by all modes: `-q`, `-f`, `--execute-statement`)
- 7 active preprocessor transformers (all enabled by default)
- WHERE/GROUP BY alias expansion
- HAVING auto-aliasing
- ORDER BY aggregate rewriting
- Expression lifting for window functions
- CTE hoisting (ready for subqueries)
- IN operator optimization

✅ **What Works Well:**
- Basic SELECT, WHERE, GROUP BY, HAVING, ORDER BY
- Window functions (ROW_NUMBER, RANK, DENSE_RANK, LAG, LEAD, etc.)
- CTEs (WITH clauses)
- Aggregates (SUM, COUNT, AVG, MIN, MAX)
- String functions (UPPER, LOWER, TRIM, SUBSTR, etc.)
- Math functions
- Date functions
- Unit conversions (CONVERT)
- JSON data sources
- CSV data sources
- Template expansion
- Temp tables

## Feature Gaps (Ordered by Priority)

### HIGH PRIORITY - Standard SQL Features

#### 1. DISTINCT in Aggregates
**Status:** ❌ Not implemented
**Difficulty:** Medium
**Example:**
```sql
SELECT region, COUNT(DISTINCT salesperson) AS unique_salespeople
FROM sales
GROUP BY region
```

**Implementation Approach:**
- **Transformer:** `DistinctAggregateLifter`
- Detect `COUNT(DISTINCT x)` or `SUM(DISTINCT x)`
- Lift to CTE that computes DISTINCT values first
- Then aggregate the CTE result

**Rewrite:**
```sql
-- Input
SELECT region, COUNT(DISTINCT salesperson) AS cnt
FROM sales
GROUP BY region

-- Becomes
WITH _distinct_0 AS (
    SELECT region, salesperson
    FROM sales
    GROUP BY region, salesperson
)
SELECT region, COUNT(*) AS cnt
FROM _distinct_0
GROUP BY region
```

**Estimated Effort:** 2-3 days

---

#### 2. Aggregate Expressions in ORDER BY
**Status:** ✅ **COMPLETED!**
**Difficulty:** Easy
**Example:**
```sql
SELECT region, SUM(sales_amount) AS total
FROM sales
GROUP BY region
ORDER BY SUM(sales_amount) DESC  -- Works perfectly!
```

**Implementation:**
- ✅ **Transformer:** `OrderByAliasTransformer` (implemented)
- ✅ Extended parser to support expressions in ORDER BY
- ✅ Updated AST with `OrderByItem` struct
- ✅ Handles COUNT(*), all aggregates, auto-generates aliases
- ✅ Works in all execution modes (fixed dependency-aware path)
- ✅ See `examples/order_by_expressions.sql` with 8 examples
- ✅ Formal test coverage

**Completed:** 2025-11-01

---

#### 3. Basic Correlated Subqueries (EXISTS/NOT EXISTS)
**Status:** ❌ Not implemented
**Difficulty:** Hard
**Example:**
```sql
SELECT *
FROM sales s1
WHERE EXISTS (
    SELECT 1
    FROM sales s2
    WHERE s2.region = s1.region
    AND s2.sales_amount > s1.sales_amount
)
```

**Implementation Approach:**
- **Transformer:** `CorrelatedSubqueryRewriter`
- Detect correlated subqueries
- Rewrite as self-join with appropriate conditions
- Handle EXISTS → semi-join, NOT EXISTS → anti-join

**Rewrite:**
```sql
-- Input
SELECT * FROM sales s1
WHERE EXISTS (SELECT 1 FROM sales s2 WHERE s2.region = s1.region AND s2.sales_amount > s1.sales_amount)

-- Becomes
SELECT DISTINCT s1.*
FROM sales s1
INNER JOIN sales s2 ON s2.region = s1.region AND s2.sales_amount > s1.sales_amount
```

**Estimated Effort:** 1-2 weeks (requires JOIN support first)

---

### MEDIUM PRIORITY - Quality of Life Features

#### 4. QUALIFY Clause (Snowflake-style)
**Status:** ❌ Not implemented (but ExpressionLifter partially handles this)
**Difficulty:** Easy
**Example:**
```sql
SELECT region, sales_amount,
       ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) AS rn
FROM sales
QUALIFY rn <= 3  -- Cleaner than WHERE with window function
```

**Implementation Approach:**
- **Transformer:** `QualifyToWhereTransformer`
- Convert QUALIFY to WHERE after lifting window functions
- ExpressionLifter already does the hard work!

**Rewrite:**
```sql
-- Input
SELECT region, sales_amount, ROW_NUMBER() OVER (...) AS rn
FROM sales
QUALIFY rn <= 3

-- Becomes (via ExpressionLifter + QualifyToWhere)
WITH _lifted AS (...)
SELECT * FROM _lifted WHERE rn <= 3
```

**Estimated Effort:** 1 day (parser + simple transformer)

---

#### 5. PIVOT/UNPIVOT
**Status:** ❌ Not implemented
**Difficulty:** Medium
**Example:**
```sql
SELECT *
FROM sales
PIVOT (SUM(sales_amount) FOR month IN ('2024-01', '2024-02', '2024-03'))
```

**Implementation Approach:**
- **Transformer:** `PivotExpander`
- Generate CASE expressions for each pivot value
- Create aggregates with CASE WHEN conditions

**Rewrite:**
```sql
-- Input
PIVOT (SUM(sales_amount) FOR month IN ('2024-01', '2024-02', '2024-03'))

-- Becomes
SELECT
    region,
    SUM(CASE WHEN month = '2024-01' THEN sales_amount ELSE 0 END) AS "2024-01",
    SUM(CASE WHEN month = '2024-02' THEN sales_amount ELSE 0 END) AS "2024-02",
    SUM(CASE WHEN month = '2024-03' THEN sales_amount ELSE 0 END) AS "2024-03"
FROM sales
GROUP BY region
```

**Estimated Effort:** 3-5 days

---

#### 6. Array/List Aggregate Functions
**Status:** ❌ Not implemented
**Difficulty:** Medium
**Example:**
```sql
SELECT region, ARRAY_AGG(salesperson) AS salespeople
FROM sales
GROUP BY region
```

**Implementation Approach:**
- Implement ARRAY_AGG, STRING_AGG, LIST, etc.
- Return as JSON array or delimited string
- May require new DataValue type

**Estimated Effort:** 3-4 days

---

### LOW PRIORITY - Advanced Features

#### 7. Recursive CTEs
**Status:** ❌ Not implemented
**Difficulty:** Very Hard
**Example:**
```sql
WITH RECURSIVE hierarchy AS (
    SELECT id, parent_id, 1 AS level FROM employees WHERE parent_id IS NULL
    UNION ALL
    SELECT e.id, e.parent_id, h.level + 1
    FROM employees e
    JOIN hierarchy h ON e.parent_id = h.id
)
SELECT * FROM hierarchy
```

**Implementation Approach:**
- **Executor-level change** (not a transformer)
- Requires iterative evaluation
- Track recursion depth
- Handle termination conditions

**Estimated Effort:** 2-3 weeks

---

#### 8. LATERAL Joins
**Status:** ❌ Not implemented
**Difficulty:** Hard
**Example:**
```sql
SELECT s1.region, top_sales.sales_amount
FROM sales s1,
LATERAL (
    SELECT sales_amount
    FROM sales s2
    WHERE s2.region = s1.region
    ORDER BY sales_amount DESC
    LIMIT 3
) AS top_sales
```

**Implementation Approach:**
- **Transformer:** Rewrite as correlated subquery or window function
- May require multiple passes

**Estimated Effort:** 1-2 weeks

---

#### 9. Multi-Value INSERT (for temp tables)
**Status:** ❌ Not implemented
**Difficulty:** Medium
**Example:**
```sql
INSERT INTO #temp_regions (region, target)
VALUES ('North', 100000), ('South', 120000), ('East', 90000), ('West', 130000)
```

**Implementation Approach:**
- Extend script parser to handle INSERT
- Support VALUES with multiple rows
- Wire into temp table system

**Estimated Effort:** 3-4 days

---

### NICE TO HAVE - Syntactic Sugar

#### 10. Column Name Inference (SELECT * EXCLUDE/REPLACE)
**Status:** ❌ Not implemented
**Difficulty:** Easy
**Example:**
```sql
SELECT * EXCLUDE (id, internal_id) FROM sales
SELECT * REPLACE (UPPER(region) AS region) FROM sales
```

**Implementation Approach:**
- **Transformer:** `StarExcludeReplaceExpander`
- Expand `*` to all columns
- Remove excluded columns
- Replace specified columns

**Estimated Effort:** 2-3 days

---

#### 11. ILIKE (Case-Insensitive LIKE)
**Status:** ❌ Not implemented
**Difficulty:** Trivial
**Example:**
```sql
SELECT * FROM sales WHERE region ILIKE '%north%'
```

**Implementation Approach:**
- **Transformer:** Rewrite as `UPPER(column) LIKE UPPER(pattern)`
- Or add ILIKE operator to executor

**Estimated Effort:** 1 day

---

## Implementation Priority Matrix

| Feature | User Value | Difficulty | SQL Standard | Priority | Status |
|---------|-----------|-----------|--------------|----------|--------|
| ~~ORDER BY aggregates~~ | High | Easy | Yes | ~~**1**~~ | ✅ Done |
| DISTINCT in aggregates | High | Medium | Yes | **1** | ⏳ Next |
| QUALIFY clause | Medium | Easy | No (Snowflake) | **2** | 📋 Ready |
| ILIKE | Low | Trivial | No (Postgres) | **3** | 📋 Ready |
| PIVOT/UNPIVOT | Medium | Medium | Yes (SQL:2016) | **4** | 📋 Ready |
| ARRAY_AGG/STRING_AGG | Medium | Medium | Yes | **5** | 📋 Ready |
| SELECT * EXCLUDE | Low | Easy | No (DuckDB) | **6** | 📋 Ready |
| Correlated subqueries | High | Hard | Yes | **7** | ⚠️ Complex |
| LATERAL joins | Low | Hard | Yes | **8** | ⚠️ Complex |
| Recursive CTEs | Low | Very Hard | Yes | **9** | ⚠️ Complex |

## Next Steps

### Immediate (1-2 weeks)
1. ✅ Complete execution mode unification (Phases 0-3) - **DONE!**
2. ✅ Document all transformers - **DONE!**
3. ✅ Create `examples/expander_rewriters.sql` - **DONE!**
4. ✅ Implement ORDER BY aggregate expansion - **DONE!** (2025-11-01)
5. **NEXT:** Choose from quick wins below

### Short-term (1-2 months)
1. Add QUALIFY clause support
2. Implement PIVOT/UNPIVOT
3. Add ARRAY_AGG and STRING_AGG
4. Improve window function support (PARTITION BY expressions)

### Medium-term (3-6 months)
1. Basic JOIN support (INNER, LEFT, RIGHT)
2. Correlated subqueries (EXISTS, NOT EXISTS, IN with subquery)
3. Subquery support in FROM clause
4. More window functions (NTILE, CUME_DIST, PERCENT_RANK)

### Long-term (6+ months)
1. LATERAL joins
2. Recursive CTEs
3. Full JOIN optimization
4. Query planner/optimizer

## Guidelines for Adding Transformers

When deciding whether to add a new transformer:

### ✅ Good Candidates for Transformers:
- **Syntactic sugar** (can be rewritten to simpler SQL)
- **Standard SQL features** that are complex to execute directly
- **High user value** (frequently requested)
- **Clear rewrite strategy** (deterministic transformation)
- **No executor changes needed** (pure AST manipulation)

### ❌ Bad Candidates for Transformers:
- **Fundamental execution changes** (e.g., JOIN algorithms)
- **Performance optimizations** (belongs in query planner)
- **Data type changes** (belongs in type system)
- **I/O operations** (reading files, network calls)

### Decision Framework:
1. **Can it be rewritten to simpler SQL?** → Transformer
2. **Requires new execution logic?** → Executor enhancement
3. **Needs performance optimization?** → Query planner (future)
4. **Syntactic convenience?** → Transformer
5. **Data manipulation?** → Executor or function

## Conclusion

The transformer-based approach gives us a **powerful lever** to add SQL features:

- ✅ **Proven:** 6 transformers already working in production
- ✅ **Unified:** Same transformers in all execution modes
- ✅ **Extensible:** Easy to add new transformers
- ✅ **Maintainable:** Each transformer is independent
- ✅ **Testable:** Can test transformers in isolation

**Philosophy:** Use transformers to fill SQL feature gaps wherever possible, resort to executor changes only when necessary.

**Next priorities:**
1. DISTINCT in aggregates (high value, medium effort)
2. ORDER BY aggregates (high value, low effort)
3. QUALIFY clause (nice syntax sugar, low effort)

This approach allows us to **incrementally improve** SQL support without major architectural changes, while keeping the executor simple and focused.
