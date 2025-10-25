# Correlated Subquery Rewrite Examples

## Purpose

This document provides **testable, concrete examples** of SQL patterns that:
1. **Can be rewritten** from correlated subqueries to CTEs
2. **Cannot be rewritten** (fundamental limitations)
3. **Already work** in SQL CLI today

Use these as test cases for the AST preprocessor implementation.

---

## Category 1: ✅ Rewritable Patterns

### Pattern 1.1: Scalar Aggregate Correlated Subquery

**Original (Correlated):**
```sql
SELECT
    c.customer_id,
    c.customer_name,
    (SELECT COUNT(*) FROM orders o WHERE o.customer_id = c.customer_id) as order_count,
    (SELECT SUM(amount) FROM orders o WHERE o.customer_id = c.customer_id) as total_spent
FROM customers c
WHERE c.region = 'EMEA';
```

**Rewritten (CTE + JOIN):**
```sql
WITH order_stats AS (
    SELECT
        customer_id,
        COUNT(*) as order_count,
        SUM(amount) as total_spent
    FROM orders
    GROUP BY customer_id
)
SELECT
    c.customer_id,
    c.customer_name,
    COALESCE(os.order_count, 0) as order_count,
    COALESCE(os.total_spent, 0) as total_spent
FROM customers c
LEFT JOIN order_stats os ON c.customer_id = os.customer_id
WHERE c.region = 'EMEA';
```

**Rewrite Rule:**
- **Pattern:** `(SELECT aggregate_fn FROM table WHERE table.fk = outer.pk)`
- **Action:** Extract to CTE with GROUP BY, replace with LEFT JOIN
- **Complexity:** Easy

---

### Pattern 1.2: EXISTS Correlated Subquery

**Original (Correlated):**
```sql
SELECT *
FROM customers c
WHERE EXISTS (
    SELECT 1 FROM orders o
    WHERE o.customer_id = c.customer_id
    AND o.status = 'PENDING'
);
```

**Rewritten (CTE + SEMI JOIN):**
```sql
WITH pending_customers AS (
    SELECT DISTINCT customer_id
    FROM orders
    WHERE status = 'PENDING'
)
SELECT c.*
FROM customers c
JOIN pending_customers pc ON c.customer_id = pc.customer_id;
```

**Rewrite Rule:**
- **Pattern:** `EXISTS (SELECT ... WHERE correlated_condition AND filter)`
- **Action:** Extract to CTE with DISTINCT, use INNER JOIN
- **Complexity:** Easy

---

### Pattern 1.3: NOT EXISTS

**Original (Correlated):**
```sql
SELECT *
FROM customers c
WHERE NOT EXISTS (
    SELECT 1 FROM orders o WHERE o.customer_id = c.customer_id
);
```

**Rewritten (CTE + ANTI JOIN):**
```sql
WITH customers_with_orders AS (
    SELECT DISTINCT customer_id FROM orders
)
SELECT c.*
FROM customers c
LEFT JOIN customers_with_orders cwo ON c.customer_id = cwo.customer_id
WHERE cwo.customer_id IS NULL;
```

**Rewrite Rule:**
- **Pattern:** `NOT EXISTS (SELECT ... WHERE correlated_condition)`
- **Action:** LEFT JOIN + IS NULL check
- **Complexity:** Easy

---

### Pattern 1.4: IN Correlated Subquery

**Original (Correlated):**
```sql
SELECT *
FROM products p
WHERE p.category_id IN (
    SELECT category_id FROM featured_categories
    WHERE region = p.region
);
```

**Rewritten (CTE + JOIN with WHERE):**
```sql
SELECT p.*
FROM products p
JOIN featured_categories fc
    ON p.category_id = fc.category_id
    AND p.region = fc.region;
```

**Rewrite Rule:**
- **Pattern:** `column IN (SELECT column FROM table WHERE correlated_condition)`
- **Action:** Convert to INNER JOIN with both conditions
- **Complexity:** Easy

---

### Pattern 1.5: Top-N per Group

**Original (Correlated):**
```sql
SELECT *
FROM orders o
WHERE (
    SELECT COUNT(*)
    FROM orders o2
    WHERE o2.customer_id = o.customer_id
    AND o2.order_date > o.order_date
) < 3;
```

**Rewritten (Window Function):**
```sql
WITH ranked_orders AS (
    SELECT
        *,
        ROW_NUMBER() OVER (
            PARTITION BY customer_id
            ORDER BY order_date DESC
        ) as rn
    FROM orders
)
SELECT *
FROM ranked_orders
WHERE rn <= 3;
```

**Rewrite Rule:**
- **Pattern:** `WHERE (SELECT COUNT(*) FROM same_table WHERE correlated AND ordering) < N`
- **Action:** Convert to window function with PARTITION BY
- **Complexity:** Medium

---

### Pattern 1.6: Running Total (Correlated)

**Original (Correlated):**
```sql
SELECT
    order_id,
    order_date,
    amount,
    (SELECT SUM(amount) FROM orders o2
     WHERE o2.customer_id = o.customer_id
     AND o2.order_date <= o.order_date) as running_total
FROM orders o;
```

**Rewritten (Window Function):**
```sql
SELECT
    order_id,
    order_date,
    amount,
    SUM(amount) OVER (
        PARTITION BY customer_id
        ORDER BY order_date
        ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
    ) as running_total
FROM orders;
```

**Rewrite Rule:**
- **Pattern:** `(SELECT aggregate FROM same_table WHERE correlated AND <= ordering)`
- **Action:** Convert to window function with frame
- **Complexity:** Medium

---

### Pattern 1.7: Multiple Correlated Subqueries

**Original (Correlated):**
```sql
SELECT
    p.product_id,
    p.product_name,
    (SELECT AVG(rating) FROM reviews r WHERE r.product_id = p.product_id) as avg_rating,
    (SELECT COUNT(*) FROM reviews r WHERE r.product_id = p.product_id) as review_count,
    (SELECT MAX(review_date) FROM reviews r WHERE r.product_id = p.product_id) as latest_review
FROM products p;
```

**Rewritten (Single CTE):**
```sql
WITH review_stats AS (
    SELECT
        product_id,
        AVG(rating) as avg_rating,
        COUNT(*) as review_count,
        MAX(review_date) as latest_review
    FROM reviews
    GROUP BY product_id
)
SELECT
    p.product_id,
    p.product_name,
    COALESCE(rs.avg_rating, 0) as avg_rating,
    COALESCE(rs.review_count, 0) as review_count,
    rs.latest_review
FROM products p
LEFT JOIN review_stats rs ON p.product_id = rs.product_id;
```

**Rewrite Rule:**
- **Pattern:** Multiple scalar subqueries with same correlation
- **Action:** Combine into single CTE with multiple aggregates
- **Complexity:** Easy (optimization opportunity!)

---

## Category 2: ⚠️ Partially Rewritable Patterns

### Pattern 2.1: Correlated with Additional Filter on Outer

**Original (Correlated):**
```sql
SELECT *
FROM orders o
WHERE o.status = 'ACTIVE'
AND EXISTS (
    SELECT 1 FROM line_items li
    WHERE li.order_id = o.order_id
    AND li.quantity > 10
);
```

**Rewritten:**
```sql
WITH high_quantity_orders AS (
    SELECT DISTINCT order_id
    FROM line_items
    WHERE quantity > 10
)
SELECT o.*
FROM orders o
JOIN high_quantity_orders hqo ON o.order_id = hqo.order_id
WHERE o.status = 'ACTIVE';
```

**Complexity:** Easy (WHERE clause pushdown works)

---

### Pattern 2.2: Nested Correlated Subqueries

**Original (Correlated):**
```sql
SELECT
    c.customer_id,
    (SELECT COUNT(*)
     FROM orders o
     WHERE o.customer_id = c.customer_id
     AND o.total > (
         SELECT AVG(total) FROM orders o2
         WHERE o2.customer_id = c.customer_id
     )
    ) as above_avg_orders
FROM customers c;
```

**Rewritten (Multi-level CTEs):**
```sql
WITH customer_avg_totals AS (
    SELECT
        customer_id,
        AVG(total) as avg_total
    FROM orders
    GROUP BY customer_id
),
above_avg_counts AS (
    SELECT
        o.customer_id,
        COUNT(*) as above_avg_orders
    FROM orders o
    JOIN customer_avg_totals cat ON o.customer_id = cat.customer_id
    WHERE o.total > cat.avg_total
    GROUP BY o.customer_id
)
SELECT
    c.customer_id,
    COALESCE(aac.above_avg_orders, 0) as above_avg_orders
FROM customers c
LEFT JOIN above_avg_counts aac ON c.customer_id = aac.customer_id;
```

**Complexity:** Hard (requires recursive rewriting)

---

## Category 3: ❌ Cannot Be Rewritten

### Pattern 3.1: Outer Column in Arbitrary Expression

**Original (Correlated):**
```sql
SELECT
    c.customer_id,
    c.threshold,
    (SELECT COUNT(*)
     FROM orders o
     WHERE o.customer_id = c.customer_id
     AND o.amount > c.threshold * 1.5  -- Outer column in expression!
    ) as high_value_orders
FROM customers c;
```

**Why it can't be rewritten:**
- The `c.threshold * 1.5` computation depends on each outer row
- No way to pre-compute this in a CTE without cartesian product
- Would need row-by-row evaluation (true correlation)

**Workaround:**
```sql
-- User must manually use CROSS JOIN
WITH high_value_orders AS (
    SELECT
        c.customer_id,
        c.threshold,
        COUNT(*) as high_value_orders
    FROM customers c
    JOIN orders o ON o.customer_id = c.customer_id
    WHERE o.amount > c.threshold * 1.5
    GROUP BY c.customer_id, c.threshold
)
SELECT
    c.customer_id,
    c.threshold,
    COALESCE(hvo.high_value_orders, 0) as high_value_orders
FROM customers c
LEFT JOIN high_value_orders hvo ON c.customer_id = hvo.customer_id;
```

---

### Pattern 3.2: Lateral Correlation with Table Function

**Original (Correlated):**
```sql
SELECT *
FROM customers c
CROSS APPLY generate_series(1, c.num_orders) AS s(n);
```

**Why it can't be rewritten:**
- `c.num_orders` parameterizes a table-valued function
- Each row produces a different number of output rows
- Requires true lateral join semantics

**No workaround** without lateral join support.

---

### Pattern 3.3: Mutually Recursive Correlation

**Original (Correlated):**
```sql
SELECT *
FROM employees e
WHERE e.salary > (
    SELECT AVG(salary)
    FROM employees e2
    WHERE e2.department = e.department
    AND e2.salary > (
        SELECT AVG(salary)
        FROM employees e3
        WHERE e3.location = e.location  -- References original e!
    )
);
```

**Why it can't be rewritten:**
- Inner-most subquery references outermost table
- Creates circular dependency
- Would need multiple passes or fixpoint iteration

**No practical workaround** (query is likely malformed anyway).

---

## Category 4: ✅ Already Works (No Rewrite Needed)

### Pattern 4.1: Non-Correlated Scalar Subquery

**Works Today:**
```sql
SELECT
    product_id,
    price,
    (SELECT AVG(price) FROM products) as avg_price,
    price - (SELECT AVG(price) FROM products) as diff_from_avg
FROM products;
```

**No rewrite needed** - scalar subquery executes once, value used for all rows.

---

### Pattern 4.2: Non-Correlated IN Subquery

**Works Today:**
```sql
SELECT *
FROM products
WHERE category_id IN (SELECT id FROM featured_categories WHERE region = 'US');
```

**No rewrite needed** - subquery executes once, result set used for lookup.

---

### Pattern 4.3: FROM Subquery

**Works Today:**
```sql
SELECT *
FROM (
    SELECT
        customer_id,
        SUM(amount) as total,
        COUNT(*) as order_count
    FROM orders
    GROUP BY customer_id
) AS customer_totals
WHERE total > 1000;
```

**No rewrite needed** - already working perfectly!

---

## Testing Matrix for Preprocessor

When implementing the preprocessor, test against this matrix:

| Pattern | Current Status | After Preprocessor | Test File |
|---------|----------------|-------------------|-----------|
| Scalar aggregate correlated | ❌ Fails | ✅ Works | `test_scalar_corr.sql` |
| EXISTS correlated | ❌ Fails | ✅ Works | `test_exists_corr.sql` |
| NOT EXISTS correlated | ❌ Fails | ✅ Works | `test_not_exists_corr.sql` |
| IN correlated | ❌ Fails | ✅ Works | `test_in_corr.sql` |
| Top-N per group | ❌ Fails | ✅ Works | `test_topn_corr.sql` |
| Running total correlated | ❌ Fails | ✅ Works | `test_running_total.sql` |
| Multiple scalar correlated | ❌ Fails | ✅ Works | `test_multi_scalar.sql` |
| Nested correlated | ❌ Fails | ✅ Works | `test_nested_corr.sql` |
| Outer col in expression | ❌ Fails | ❌ Fails (clear error) | `test_expr_corr.sql` |
| Lateral correlation | ❌ Fails | ❌ Fails (clear error) | `test_lateral.sql` |
| Non-correlated scalar | ✅ Works | ✅ Works | `test_noncorr_scalar.sql` |
| Non-correlated IN | ✅ Works | ✅ Works | `test_noncorr_in.sql` |
| FROM subquery | ✅ Works | ✅ Works | `test_from_subquery.sql` |

---

## Preprocessor Error Messages

For patterns that **cannot** be rewritten:

```
Error: Correlated subquery cannot be automatically rewritten

The subquery uses outer table column 'threshold' in an expression (threshold * 1.5).
This pattern requires row-by-row evaluation which is not currently supported.

Suggested workaround:
Use CROSS JOIN to manually correlate the tables:

    WITH correlated AS (
        SELECT c.customer_id, c.threshold, o.amount
        FROM customers c
        JOIN orders o ON o.customer_id = c.customer_id
        WHERE o.amount > c.threshold * 1.5
    )
    SELECT customer_id, COUNT(*) as high_value_orders
    FROM correlated
    GROUP BY customer_id;

For more information, see: docs/CORRELATED_SUBQUERY_REWRITE_EXAMPLES.md
```

---

## Performance Comparison

### Benchmark: Scalar Correlated Subquery vs CTE

**Dataset:** 10,000 customers, 100,000 orders

**Original Query:**
```sql
SELECT
    c.customer_id,
    (SELECT COUNT(*) FROM orders o WHERE o.customer_id = c.customer_id) as cnt
FROM customers c;
```

**Execution:** O(n×m) = 1,000,000,000 comparisons if fully nested

**Rewritten Query:**
```sql
WITH order_counts AS (
    SELECT customer_id, COUNT(*) as cnt FROM orders GROUP BY customer_id
)
SELECT c.customer_id, COALESCE(oc.cnt, 0) as cnt
FROM customers c LEFT JOIN order_counts oc ON c.customer_id = oc.customer_id;
```

**Execution:** O(m) for GROUP BY + O(n) for JOIN = 110,000 operations

**Speedup:** ~9,000x faster! 🚀

---

## Conclusion

**Rewritable:** ~90% of real-world correlated subquery patterns
**Non-Rewritable:** ~10% (exotic cases, complex expressions)
**Already Working:** 100% of non-correlated patterns

**Next Step:** Implement preprocessor with these patterns as test cases.

