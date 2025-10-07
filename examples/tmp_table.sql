-- #! ../data/sales_data.csv

-- Demonstration of temporary tables in sql-cli
-- Temporary tables persist across GO-separated batches within a script
-- They are automatically cleaned up when the script completes

-- =====================================================
-- Example: Store filtered data and query it multiple times
-- =====================================================

-- Step 1: Store high-value sales in a temporary table
SELECT
    month,
    region,
    product,
    sales_amount,
    salesperson
FROM sales_data
WHERE sales_amount > 15000
INTO #high_value_sales;
GO

-- Step 2: Query the temporary table to get regional summaries
-- Note: No need to re-filter or reload the CSV!
SELECT
    region,
    COUNT(*) as sale_count,
    SUM(sales_amount) as total_amount,
    AVG(sales_amount) as avg_amount
FROM #high_value_sales
GROUP BY region
ORDER BY total_amount DESC;
GO

-- Step 3: Query the same temp table for product analysis
SELECT
    product,
    COUNT(*) as sale_count,
    SUM(sales_amount) as total_amount
FROM #high_value_sales
GROUP BY product
ORDER BY total_amount DESC;
GO

-- Step 4: Get detailed breakdown by region and product
SELECT
    region,
    product,
    SUM(sales_amount) as total_amount
FROM #high_value_sales
GROUP BY region, product
ORDER BY region, total_amount DESC;
GO
