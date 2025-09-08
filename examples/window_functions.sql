-- Window Functions Examples for SQL-CLI
-- 
-- This file demonstrates various window functions including:
-- LAG, LEAD, ROW_NUMBER, FIRST_VALUE, LAST_VALUE
-- with PARTITION BY and ORDER BY clauses
--
-- Run with: sql-cli data/sales_data.csv -f examples/window_functions.sql -o table

-- 1. Basic LAG - Show previous month's sales for each row
SELECT 
    salesperson,
    month,
    sales_amount,
    LAG(sales_amount, 1) OVER (ORDER BY salesperson, month) as prev_sales,
    sales_amount - LAG(sales_amount, 1) OVER (ORDER BY salesperson, month) as change
FROM test
LIMIT 10;
GO

-- 2. LEAD - Show next month's sales
SELECT 
    salesperson,
    month,
    sales_amount,
    LEAD(sales_amount, 1) OVER (ORDER BY salesperson, month) as next_sales
FROM test
WHERE salesperson = 'Alice';
GO

-- 3. ROW_NUMBER - Rank salespeople by total sales
SELECT 
    salesperson,
    sales_amount,
    ROW_NUMBER() OVER (ORDER BY sales_amount DESC) as overall_rank
FROM test;
GO

-- 4. ROW_NUMBER with PARTITION BY - Rank within each region
SELECT 
    region,
    salesperson,
    month,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as region_rank
FROM test;
GO

-- 5. Get top performer from each region (rank = 1)
-- Note: This demonstrates getting the first row from each partition
-- In standard SQL, you'd use a CTE or subquery, but here we show the ranking
SELECT 
    region,
    salesperson,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank
FROM test
WHERE month = '2024-03';  -- Latest month only
GO

-- 6. LAG with PARTITION BY - Compare to previous month within same region
SELECT 
    region,
    salesperson,
    month,
    sales_amount,
    LAG(sales_amount, 1) OVER (PARTITION BY region, salesperson ORDER BY month) as prev_month,
    CASE 
        WHEN LAG(sales_amount, 1) OVER (PARTITION BY region, salesperson ORDER BY month) IS NULL 
        THEN 'N/A'
        WHEN sales_amount > LAG(sales_amount, 1) OVER (PARTITION BY region, salesperson ORDER BY month)
        THEN 'UP'
        ELSE 'DOWN'
    END as trend
FROM test;
GO

-- 7. Running difference using LAG
SELECT 
    salesperson,
    month,
    sales_amount,
    sales_amount - LAG(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month) as month_over_month_change
FROM test
WHERE region = 'North';
GO

-- 8. FIRST_VALUE - Show first sale in each region
SELECT 
    region,
    salesperson,
    month,
    sales_amount,
    FIRST_VALUE(sales_amount) OVER (PARTITION BY region ORDER BY month) as first_month_sales
FROM test;
GO

-- 9. LAST_VALUE - Show last sale in each region
-- Note: LAST_VALUE by default only looks at rows up to current row
SELECT 
    region,
    salesperson,
    month,
    sales_amount,
    LAST_VALUE(sales_amount) OVER (PARTITION BY region ORDER BY month) as last_sale_in_frame
FROM test;
GO

-- 10. Multiple window functions in one query
SELECT 
    region,
    salesperson,
    month,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank_in_region,
    LAG(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month) as prev_month_sales,
    LEAD(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month) as next_month_sales,
    FIRST_VALUE(sales_amount) OVER (PARTITION BY region ORDER BY sales_amount DESC) as region_max
FROM test
LIMIT 20;
GO

-- 11. Identify top 2 performers per region
SELECT 
    region,
    salesperson,
    sales_amount,
    product,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank
FROM test
WHERE month = '2024-03';  -- Latest month
GO

-- 12. Compare each sale to region average (using window context)
-- This shows how window functions maintain row-level detail unlike GROUP BY
SELECT 
    region,
    salesperson,
    month,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank_in_region,
    FIRST_VALUE(sales_amount) OVER (PARTITION BY region ORDER BY sales_amount DESC) as region_highest,
    sales_amount * 100.0 / FIRST_VALUE(sales_amount) OVER (PARTITION BY region ORDER BY sales_amount DESC) as pct_of_highest
FROM test;
GO

-- 13. Detect sales trends using LAG
-- SELECT 
--     salesperson,
--     month,
--     sales_amount,
--     LAG(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month) as m1,
--     LAG(sales_amount, 2) OVER (PARTITION BY salesperson ORDER BY month) as m2,
--     CASE 
--         WHEN LAG(sales_amount, 2) OVER (PARTITION BY salesperson ORDER BY month) IS NULL THEN 'NEW'
--         WHEN sales_amount > LAG(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month)
--          AND LAG(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month) > LAG(sales_amount, 2) OVER (PARTITION BY salesperson ORDER BY month)
--         THEN 'RISING'
--         WHEN sales_amount < LAG(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month)
--          AND LAG(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month) < LAG(sales_amount, 2) OVER (PARTITION BY salesperson ORDER BY month)
--         THEN 'FALLING'
--         ELSE 'MIXED'
--     END as trend
-- FROM test;
-- GO

-- 14. Gap analysis - find gaps in sequences
SELECT 
    salesperson,
    month,
    sales_amount,
    LAG(month, 1) OVER (PARTITION BY salesperson ORDER BY month) as prev_month,
    LEAD(month, 1) OVER (PARTITION BY salesperson ORDER BY month) as next_month
FROM test
WHERE region = 'North';
GO

-- 15. Cumulative ranking across regions
SELECT 
    region,
    salesperson,
    sales_amount,
    ROW_NUMBER() OVER (ORDER BY sales_amount DESC) as global_rank,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as regional_rank
FROM test
WHERE month = '2024-03';
GO
