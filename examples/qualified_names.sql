-- #! ../data/sales_data.csv

-- Qualified Names and Table Aliases
-- Demonstrates table alias support across SQL clauses (WHERE, SELECT, ORDER BY, GROUP BY)

-- Example 1: Basic CTE with alias in WHERE and SELECT
-- Shows qualified column references (t.column_name)

WITH data AS (
    SELECT value as id, value * 10 as amount, value % 3 as category
    FROM RANGE(1, 10)
)
SELECT t.id, t.amount, t.category
FROM data t
WHERE t.amount > 30;
GO

-- Example 2: CTE with alias in ORDER BY
-- Demonstrates sorting by qualified column names
WITH products AS (
    SELECT
        value as product_id,
        value * 100 as price,
        CASE WHEN value % 2 = 0 THEN 'even' ELSE 'odd' END as type
    FROM RANGE(1, 10)
)
SELECT p.product_id, p.price, p.type
FROM products p
WHERE p.price >= 300
ORDER BY p.price DESC;
GO

-- Example 3: CTE with alias in GROUP BY
-- Shows aggregation with qualified column references
WITH sales AS (
    SELECT
        value % 4 as region,
        value % 3 as product_category,
        value * 100 as revenue
    FROM RANGE(1, 24)
)
SELECT
    s.region,
    s.product_category,
    COUNT(*) as count,
    SUM(s.revenue) as total_revenue,
    AVG(s.revenue) as avg_revenue
FROM sales s
GROUP BY s.region, s.product_category
ORDER BY s.region, s.product_category;
GO

-- Example 4: Nested CTEs with aliases
-- Demonstrates multiple CTEs with qualified names at each level
WITH base AS (
    SELECT value as id, value % 5 as bucket
    FROM RANGE(1, 25)
),
enriched AS (
    SELECT b.id, b.bucket, b.id * 10 as score
    FROM base b
    WHERE b.bucket IN (0, 1, 2)
),
aggregated AS (
    SELECT
        e.bucket,
        COUNT(*) as count,
        SUM(e.score) as total_score,
        AVG(e.score) as avg_score
    FROM enriched e
    GROUP BY e.bucket
)
SELECT a.bucket, a.count, a.total_score, a.avg_score
FROM aggregated a
ORDER BY a.avg_score DESC;
GO

-- Example 5: Complex filtering with string methods
-- Shows qualified names with method calls
WITH records AS (
    SELECT
        value as id,
        CASE
            WHEN value % 3 = 0 THEN 'type.category.A'
            WHEN value % 3 = 1 THEN 'type.category.B'
            ELSE 'type.category.C'
        END as classification
    FROM RANGE(1, 20)
)
SELECT r.id, r.classification
FROM records r
WHERE r.classification.Contains('category.B');
GO

-- Example 6: Multiple aggregations with HAVING
-- Demonstrates qualified names in HAVING clause
-- Note: With RANGE(1, 50) and % 5, each customer has 10 transactions
WITH transactions AS (
    SELECT
        value % 5 as customer_id,
        value * 50 as transaction_amount
    FROM RANGE(1, 100)
)
SELECT
    t.customer_id,
    COUNT(*) as transaction_count,
    SUM(t.transaction_amount) as total_amount
FROM transactions t
GROUP BY t.customer_id
HAVING transaction_count >= 10
ORDER BY total_amount DESC;
GO

-- Example 7: Combining all clauses with aliases
-- Full-featured query using qualified names throughout
WITH raw_data AS (
    SELECT
        value as id,
        value % 10 as category,
        value * 25 as value,
        CASE
            WHEN value % 2 = 0 THEN 'even'
            ELSE 'odd'
        END as parity
    FROM RANGE(1, 100)
)
SELECT
    r.category,
    r.parity,
    COUNT(*) as record_count,
    SUM(r.value) as category_total,
    AVG(r.value) as category_average,
    MIN(r.value) as min_value,
    MAX(r.value) as max_value
FROM raw_data r
WHERE r.value >= 125 AND r.category < 8
GROUP BY r.category, r.parity
ORDER BY r.category ASC, category_total DESC;
GO
