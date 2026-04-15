-- Tuple IN — match multiple columns at once against a subquery
-- Useful for "find rows where (col1, col2) appears in some other computation"

-- The classic "first occurrence" pattern:
-- Find each product's first sale by matching on (product_id, min(year))
WITH Sales AS (
    SELECT 1 as sale_id, 100 as product_id, 2008 as year, 10 as quantity, 5000 as price
    UNION ALL SELECT 2, 100, 2009, 12, 5000
    UNION ALL SELECT 7, 200, 2011, 15, 9000
    UNION ALL SELECT 8, 200, 2012, 20, 9500
)
SELECT product_id, year AS first_year, quantity, price
FROM Sales
WHERE (product_id, year) IN (
    SELECT product_id, MIN(year) FROM Sales GROUP BY product_id
);
GO

-- Find rows matching a set of composite keys
WITH orders AS (
    SELECT 1 as customer_id, '2024-01-01' as order_date, 100 as amount
    UNION ALL SELECT 1, '2024-02-01', 200
    UNION ALL SELECT 2, '2024-01-15', 150
    UNION ALL SELECT 2, '2024-03-01', 300
    UNION ALL SELECT 3, '2024-02-20', 250
),
targets AS (
    SELECT 1 as cid, '2024-01-01' as od
    UNION ALL SELECT 2, '2024-03-01'
)
SELECT * FROM orders
WHERE (customer_id, order_date) IN (SELECT cid, od FROM targets);
GO

-- Tuple NOT IN — find rows that DON'T match any composite in the set
WITH items AS (
    SELECT 'A' as category, 1 as id UNION ALL SELECT 'A', 2
    UNION ALL SELECT 'B', 1 UNION ALL SELECT 'B', 2 UNION ALL SELECT 'B', 3
),
excluded AS (
    SELECT 'A' as c, 1 as i UNION ALL SELECT 'B', 2
)
SELECT category, id FROM items
WHERE (category, id) NOT IN (SELECT c, i FROM excluded)
ORDER BY category, id;
GO

-- Three-column tuple IN
WITH events AS (
    SELECT 1 as uid, 'login' as action, '2024-01-01' as d
    UNION ALL SELECT 1, 'purchase', '2024-01-02'
    UNION ALL SELECT 2, 'login', '2024-01-01'
    UNION ALL SELECT 2, 'login', '2024-01-05'
),
firsts AS (
    SELECT uid, action, MIN(d) as d FROM events GROUP BY uid, action
)
SELECT uid, action, d FROM events
WHERE (uid, action, d) IN (SELECT uid, action, d FROM firsts)
ORDER BY uid, d;
GO
