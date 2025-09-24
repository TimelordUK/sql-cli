-- Test multiple JOIN conditions
-- Create test data with CTEs

WITH
    orders AS (
        SELECT * FROM (
            SELECT 1 as order_id, 101 as customer_id, 'Electronics' as category, 500.00 as amount
            UNION ALL
            SELECT 2 as order_id, 102 as customer_id, 'Electronics' as category, 750.00 as amount
            UNION ALL
            SELECT 3 as order_id, 101 as customer_id, 'Clothing' as category, 200.00 as amount
            UNION ALL
            SELECT 4 as order_id, 103 as customer_id, 'Electronics' as category, 1200.00 as amount
        )
    ),
    customers AS (
        SELECT * FROM (
            SELECT 101 as customer_id, 'Electronics' as preferred_category, 'Alice' as name
            UNION ALL
            SELECT 102 as customer_id, 'Electronics' as preferred_category, 'Bob' as name
            UNION ALL
            SELECT 103 as customer_id, 'Books' as preferred_category, 'Charlie' as name
        )
    )
SELECT
    o.order_id,
    c.name,
    o.category,
    c.preferred_category,
    o.amount
FROM orders o
INNER JOIN customers c
    ON o.customer_id = c.customer_id
    AND o.category = c.preferred_category
ORDER BY o.order_id;