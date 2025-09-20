-- HAVING Clause Workaround Examples
-- SQL CLI currently has limitations with HAVING clauses containing aggregates
-- This file demonstrates the CTE workaround pattern

-- #! ../data/sales_data.csv

-- ============================================================================
-- PROBLEM: HAVING with COUNT(*) doesn't work as expected
-- ============================================================================
-- This query should return products with more than 5 sales, but returns nothing:
-- SELECT product, COUNT(*) as sales_count
-- FROM sales_data
-- GROUP BY product
-- HAVING COUNT(*) > 5;

-- ============================================================================
-- SOLUTION: Use CTE with WHERE clause
-- ============================================================================
-- Wrap the GROUP BY in a CTE, then filter with WHERE
WITH product_counts AS (
    SELECT
        product,
        COUNT(*) as sales_count
    FROM sales_data
    GROUP BY product
)
SELECT *
FROM product_counts
WHERE sales_count > 5
ORDER BY sales_count DESC;
GO

-- ============================================================================
-- Example: Statistical Analysis with Filtering
-- ============================================================================
-- Instead of HAVING COUNT(*) > 5 in the GROUP BY, use CTE + WHERE
WITH stats AS (
    SELECT
        product,
        COUNT(*) as n,
        ROUND(AVG(sales_amount), 2) as mean,
        ROUND(STDDEV(sales_amount), 2) as stddev_pop,
        ROUND(STDDEV_SAMP(sales_amount), 2) as stddev_samp,
        ROUND(VARIANCE(sales_amount), 2) as var_pop,
        ROUND(VAR_SAMP(sales_amount), 2) as var_samp
    FROM sales_data
    GROUP BY product
)
SELECT
    product,
    n as sample_size,
    mean,
    stddev_pop,
    stddev_samp,
    ROUND(stddev_samp - stddev_pop, 2) as stddev_difference,
    var_pop,
    var_samp,
    ROUND(var_samp - var_pop, 2) as variance_difference
FROM stats
WHERE n > 5  -- Filter here instead of HAVING
ORDER BY sample_size DESC;
GO

-- ============================================================================
-- Example: Coefficient of Variation with Minimum Sample Size
-- ============================================================================
-- Calculate CV only for products with enough data points
WITH product_stats AS (
    SELECT
        product,
        COUNT(*) as n,
        ROUND(AVG(sales_amount), 2) as mean,
        ROUND(STDDEV(sales_amount), 2) as std_dev,
        ROUND((STDDEV(sales_amount) / AVG(sales_amount)) * 100, 2) as coefficient_of_variation
    FROM sales_data
    GROUP BY product
)
SELECT *
FROM product_stats
WHERE n > 3  -- Ensure minimum sample size
ORDER BY coefficient_of_variation DESC;
GO

-- ============================================================================
-- Example: Finding High-Volume Regions
-- ============================================================================
-- Regions with significant transaction volume
WITH region_summary AS (
    SELECT
        region,
        COUNT(*) as transaction_count,
        SUM(sales_amount) as total_sales,
        AVG(sales_amount) as avg_sale
    FROM sales_data
    GROUP BY region
)
SELECT
    region,
    transaction_count,
    ROUND(total_sales, 2) as total_sales,
    ROUND(avg_sale, 2) as avg_sale
FROM region_summary
WHERE transaction_count >= 10  -- High-volume regions only
ORDER BY total_sales DESC;
GO

-- ============================================================================
-- Example: Products Meeting Multiple Aggregate Criteria
-- ============================================================================
-- Find products with both high volume AND high average sales
WITH product_metrics AS (
    SELECT
        product,
        COUNT(*) as sales_count,
        AVG(sales_amount) as avg_amount,
        MIN(sales_amount) as min_amount,
        MAX(sales_amount) as max_amount
    FROM sales_data
    GROUP BY product
)
SELECT
    product,
    sales_count,
    ROUND(avg_amount, 2) as avg_amount,
    ROUND(min_amount, 2) as min_amount,
    ROUND(max_amount, 2) as max_amount,
    ROUND(max_amount - min_amount, 2) as price_range
FROM product_metrics
WHERE sales_count > 5 AND avg_amount > 50  -- Multiple criteria
ORDER BY avg_amount DESC;
GO

-- ============================================================================
-- Pattern Summary:
-- ============================================================================
-- 1. INSTEAD OF:
--    SELECT ... GROUP BY ... HAVING aggregate_function() > value
--
-- 2. USE:
--    WITH cte AS (
--        SELECT ..., aggregate_function() as alias
--        GROUP BY ...
--    )
--    SELECT * FROM cte WHERE alias > value
--
-- This workaround actually provides more flexibility:
-- - You can reference the aggregate by its alias (cleaner)
-- - You can apply multiple WHERE conditions easily
-- - You can join the CTE with other tables/CTEs
-- - The query is often more readable
-- ============================================================================