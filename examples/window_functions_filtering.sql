-- Window Functions: Filtering and Ordering Examples
-- 
-- Since window functions are evaluated after WHERE clauses,
-- you cannot directly filter on window function results in WHERE.
-- However, you can ORDER BY window function results.
--
-- Run with: sql-cli data/sales_data.csv -f examples/window_functions_filtering.sql -o table

-- 1. ORDER BY rank - Shows all rows ordered by their rank within each region
SELECT 
    region,
    salesperson,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank
FROM test
WHERE month = '2024-03'
ORDER BY rank, region;

GO

-- 2. ORDER BY rank then sales_amount - Top performers first
SELECT 
    region,
    salesperson,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank
FROM test
ORDER BY rank, sales_amount DESC
LIMIT 10;

GO

-- 3. Complex ordering - rank within region, then by absolute sales
SELECT 
    region,
    salesperson,
    month,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region, month ORDER BY sales_amount DESC) as monthly_rank
FROM test
ORDER BY monthly_rank, sales_amount DESC;

GO

-- 4. Using LAG with ordering to show trends
SELECT 
    salesperson,
    month,
    sales_amount,
    LAG(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month) as prev_month,
    sales_amount - LAG(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month) as change
FROM test
WHERE region = 'North'
ORDER BY salesperson, month;

GO

-- Note: To filter for only rank=1 rows (top performer from each region),
-- you would typically need a CTE or subquery in standard SQL:
-- WITH ranked AS (
--     SELECT *, ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank
--     FROM test
-- )
-- SELECT * FROM ranked WHERE rank = 1;
--
-- In this CLI, you can:
-- 1. Export results and filter in your application
-- 2. Use the ORDER BY to put desired rows first
-- 3. Use LIMIT to get top N results after ordering
