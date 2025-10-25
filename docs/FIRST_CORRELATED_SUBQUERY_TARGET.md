# First Correlated Subquery Target

## The Simplest Possible Case

This document defines the **absolute simplest** correlated subquery pattern we'll implement first. This serves as our proof-of-concept for the preprocessor enhancement.

---

## Target Pattern

**Input Query:**
```sql
SELECT
    customer_id,
    customer_name,
    (SELECT COUNT(*) FROM orders WHERE customer_id = customers.customer_id) as order_count
FROM customers;
```

**Characteristics:**
- ✅ Scalar subquery (returns single value)
- ✅ Simple aggregate function (COUNT)
- ✅ Simple correlation (single equality: `WHERE orders.customer_id = customers.customer_id`)
- ✅ No additional filters in subquery
- ✅ No outer query WHERE clause

**This is the Hello World of correlated subqueries!**

---

## Rewritten Query

**Output (what preprocessor generates):**
```sql
WITH __corr_orders_1 AS (
    SELECT
        customer_id,
        COUNT(*) as __agg_result
    FROM orders
    GROUP BY customer_id
)
SELECT
    customers.customer_id,
    customers.customer_name,
    COALESCE(__corr_orders_1.__agg_result, 0) as order_count
FROM customers
LEFT JOIN __corr_orders_1 ON customers.customer_id = __corr_orders_1.customer_id;
```

**Key transformations:**
1. Subquery → CTE with GROUP BY on correlated column
2. Aggregate function → Selected in CTE
3. Scalar subquery → LEFT JOIN to CTE
4. NULL handling → COALESCE for customers with no orders

---

## Step-by-Step Transformation

### Step 1: Identify Pattern

```rust
// In CorrelatedSubqueryRewriter
fn analyze_select_item(item: &SelectItem) -> Option<CorrelatedPattern> {
    if let SelectItem::Expression { expr, alias } = item {
        if let SqlExpression::ScalarSubquery { subquery } = expr {
            // Found a scalar subquery!
            return Some(analyze_correlation(subquery));
        }
    }
    None
}
```

### Step 2: Analyze Correlation

```rust
struct CorrelationAnalysis {
    is_correlated: bool,
    outer_table: String,        // "customers"
    outer_column: String,       // "customer_id"
    inner_table: String,        // "orders"
    inner_column: String,       // "customer_id"
    aggregate_fn: AggregateFn,  // COUNT(*)
}

fn analyze_correlation(subquery: &SelectStatement) -> CorrelationAnalysis {
    // 1. Check SELECT clause for aggregate
    let agg_fn = find_aggregate_function(&subquery.select_items)?;

    // 2. Check WHERE clause for correlation
    if let Some(where_clause) = &subquery.where_clause {
        for condition in &where_clause.conditions {
            if let Some(correlation) = extract_correlation(&condition.expr) {
                return CorrelationAnalysis {
                    is_correlated: true,
                    aggregate_fn: agg_fn,
                    ...correlation
                };
            }
        }
    }

    // Not correlated or too complex
    None
}
```

### Step 3: Extract Correlation Condition

```rust
fn extract_correlation(expr: &SqlExpression) -> Option<Correlation> {
    if let SqlExpression::BinaryOp { op, left, right } = expr {
        if op == &BinaryOperator::Equal {
            // Check if one side references outer table
            match (left.as_ref(), right.as_ref()) {
                (SqlExpression::Column(col1), SqlExpression::Column(col2)) => {
                    // Example: orders.customer_id = customers.customer_id
                    if col1.table.is_some() && col2.table.is_some() {
                        return Some(Correlation {
                            inner_col: col1.clone(),
                            outer_col: col2.clone(),
                        });
                    }
                }
                _ => {}
            }
        }
    }
    None
}
```

### Step 4: Generate CTE

```rust
fn create_aggregate_cte(
    subquery: &SelectStatement,
    correlation: &CorrelationAnalysis,
    cte_name: &str
) -> CTE {
    CTE {
        name: cte_name.to_string(),
        cte_type: CTEType::Standard(SelectStatement {
            select_items: vec![
                // GROUP BY column
                SelectItem::Expression {
                    expr: SqlExpression::Column(ColumnRef {
                        table: Some(correlation.inner_table.clone()),
                        column: correlation.inner_column.clone(),
                    }),
                    alias: correlation.inner_column.clone(),
                },
                // Aggregate result
                SelectItem::Expression {
                    expr: SqlExpression::FunctionCall {
                        name: "COUNT".to_string(),
                        args: vec![SqlExpression::Star],
                    },
                    alias: "__agg_result".to_string(),
                },
            ],
            from: Some(correlation.inner_table.clone()),
            group_by: Some(vec![correlation.inner_column.clone()]),
            ..Default::default()
        }),
    }
}
```

### Step 5: Rewrite Main Query

```rust
fn rewrite_main_query(
    stmt: &mut SelectStatement,
    cte_name: &str,
    correlation: &CorrelationAnalysis,
    original_alias: &str
) {
    // Replace scalar subquery with CTE reference
    for item in &mut stmt.select_items {
        if let SelectItem::Expression { expr, alias } = item {
            if matches!(expr, SqlExpression::ScalarSubquery { .. }) {
                *expr = SqlExpression::FunctionCall {
                    name: "COALESCE".to_string(),
                    args: vec![
                        SqlExpression::Column(ColumnRef {
                            table: Some(cte_name.to_string()),
                            column: "__agg_result".to_string(),
                        }),
                        SqlExpression::NumberLiteral("0".to_string()),
                    ],
                };
                *alias = original_alias.to_string();
            }
        }
    }

    // Add LEFT JOIN to CTE
    stmt.joins.push(JoinClause {
        join_type: JoinType::Left,
        table: TableSource::Table(cte_name.to_string()),
        alias: Some(cte_name.to_string()),
        condition: JoinCondition {
            conditions: vec![SingleJoinCondition {
                left_expr: SqlExpression::Column(ColumnRef {
                    table: Some(correlation.outer_table.clone()),
                    column: correlation.outer_column.clone(),
                }),
                operator: JoinOperator::Equal,
                right_expr: SqlExpression::Column(ColumnRef {
                    table: Some(cte_name.to_string()),
                    column: correlation.inner_column.clone(),
                }),
            }],
        },
    });
}
```

---

## Test Cases

### Test 1: Basic COUNT(*)

**Input:**
```sql
SELECT c.id, (SELECT COUNT(*) FROM orders WHERE customer_id = c.id)
FROM customers c;
```

**Expected Output:**
```sql
WITH __corr_1 AS (
    SELECT customer_id, COUNT(*) as __agg_result FROM orders GROUP BY customer_id
)
SELECT c.id, COALESCE(__corr_1.__agg_result, 0)
FROM customers c
LEFT JOIN __corr_1 ON c.id = __corr_1.customer_id;
```

### Test 2: SUM Aggregate

**Input:**
```sql
SELECT c.id, (SELECT SUM(amount) FROM orders WHERE customer_id = c.id)
FROM customers c;
```

**Expected Output:**
```sql
WITH __corr_1 AS (
    SELECT customer_id, SUM(amount) as __agg_result FROM orders GROUP BY customer_id
)
SELECT c.id, COALESCE(__corr_1.__agg_result, 0)
FROM customers c
LEFT JOIN __corr_1 ON c.id = __corr_1.customer_id;
```

### Test 3: AVG Aggregate

**Input:**
```sql
SELECT c.id, (SELECT AVG(amount) FROM orders WHERE customer_id = c.id)
FROM customers c;
```

**Expected Output:**
```sql
WITH __corr_1 AS (
    SELECT customer_id, AVG(amount) as __agg_result FROM orders GROUP BY customer_id
)
SELECT c.id, COALESCE(__corr_1.__agg_result, 0)
FROM customers c
LEFT JOIN __corr_1 ON c.id = __corr_1.customer_id;
```

### Test 4: Multiple Subqueries (Same Correlation)

**Input:**
```sql
SELECT c.id,
       (SELECT COUNT(*) FROM orders WHERE customer_id = c.id),
       (SELECT SUM(amount) FROM orders WHERE customer_id = c.id)
FROM customers c;
```

**Expected Output (Optimization!):**
```sql
WITH __corr_1 AS (
    SELECT customer_id,
           COUNT(*) as __agg_result_1,
           SUM(amount) as __agg_result_2
    FROM orders
    GROUP BY customer_id
)
SELECT c.id,
       COALESCE(__corr_1.__agg_result_1, 0),
       COALESCE(__corr_1.__agg_result_2, 0)
FROM customers c
LEFT JOIN __corr_1 ON c.id = __corr_1.customer_id;
```

**Note:** This optimization combines multiple subqueries with the same correlation into a single CTE!

---

## What This Pattern Does NOT Handle

### ❌ Additional Filters in Subquery

**Input:**
```sql
SELECT c.id, (SELECT COUNT(*) FROM orders WHERE customer_id = c.id AND status = 'PENDING')
FROM customers c;
```

**Status:** Not supported in first version (Phase 3 enhancement)

### ❌ Non-Aggregate Subqueries

**Input:**
```sql
SELECT c.id, (SELECT order_id FROM orders WHERE customer_id = c.id LIMIT 1)
FROM customers c;
```

**Status:** Different pattern (FIRST_VALUE window function - Phase 4)

### ❌ Nested Correlation

**Input:**
```sql
SELECT c.id,
       (SELECT COUNT(*) FROM orders o
        WHERE o.customer_id = c.id
        AND o.amount > (SELECT AVG(amount) FROM orders WHERE customer_id = c.id))
FROM customers c;
```

**Status:** Too complex for first version (Phase 4)

### ❌ Multiple Correlation Columns

**Input:**
```sql
SELECT t.id, (SELECT COUNT(*) FROM events e
              WHERE e.user_id = t.user_id AND e.region = t.region)
FROM transactions t;
```

**Status:** Phase 2 enhancement (multi-column GROUP BY)

---

## Implementation Checklist

- [ ] Create `src/query_plan/correlated_subquery_rewriter.rs`
- [ ] Implement pattern detection (`is_simple_correlated_scalar`)
- [ ] Implement correlation extraction (`extract_simple_correlation`)
- [ ] Implement CTE generation (`create_aggregate_cte`)
- [ ] Implement main query rewriting (`rewrite_with_join`)
- [ ] Add to preprocessor pipeline
- [ ] Write unit tests for each step
- [ ] Write integration tests for test cases above
- [ ] Add equivalence tests (original manual CTE vs auto-rewritten)
- [ ] Document in user guide

---

## Success Criteria

✅ **Functional:**
- All 4 test cases produce correct results
- Results match manually-written CTE equivalent
- No crashes or panics

✅ **Performance:**
- Rewritten query is faster than hypothetical row-by-row execution
- Overhead of rewriting is <10ms

✅ **Usability:**
- User can write natural SQL with correlated subqueries
- Error message is clear if pattern not supported
- `--show-preprocessing` shows the transformation

---

## Timeline

**Week 1:**
- Pattern detection and correlation extraction
- Unit tests

**Week 2:**
- CTE generation and query rewriting
- Integration tests

**Week 3:**
- Pipeline integration and testing
- Bug fixes and edge cases

**Week 4:**
- Documentation and polish
- Performance benchmarks

---

## Next Steps After This Works

Once we have this simplest pattern working:

1. **Add filters:** Support `AND status = 'ACTIVE'` in subquery WHERE
2. **Multi-column correlation:** Support `WHERE a = x AND b = y`
3. **EXISTS pattern:** Convert `EXISTS (SELECT 1 ...)` to SEMI JOIN
4. **NOT EXISTS pattern:** Convert to ANTI JOIN (LEFT JOIN + IS NULL)
5. **IN pattern:** Convert `IN (SELECT ...)` to SEMI JOIN

Each enhancement builds on the same foundation!

