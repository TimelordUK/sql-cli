-- GROUP BY and aggregate functions with STDDEV/VARIANCE
-- sql-cli now supports STDDEV and VARIANCE aggregates!
-- 
-- Run with: sql-cli data/sales_sample.csv -f examples/group_by_aggregates.sql

-- Basic aggregation functions 
SELECT 
    COUNT(*) as total_rows,
    SUM(amount) as total_amount,
    AVG(amount) as average_amount,
    MIN(amount) as min_amount,
    MAX(amount) as max_amount,
    STDDEV(amount) as standard_deviation,
    VARIANCE(amount) as variance
FROM test;
GO

-- Simple GROUP BY with new STDDEV/VARIANCE
SELECT 
    category,
    COUNT(*) as count,
    SUM(amount) as total,
    AVG(amount) as average,
    STDDEV(amount) as std_deviation,
    VARIANCE(amount) as variance
FROM test
GROUP BY category
ORDER BY total DESC;
GO

-- GROUP BY with HAVING clause using aggregates
SELECT 
    category,
    COUNT(*) as count,
    SUM(amount) as total,
    AVG(amount) as average
FROM test
GROUP BY category
HAVING count > 3 AND average > 500
ORDER BY average DESC;
GO

-- Analyze by status
SELECT 
    status,
    COUNT(*) as transactions,
    SUM(amount) as total_revenue,
    AVG(amount) as avg_transaction,
    MAX(amount) - MIN(amount) as price_range,
    STDDEV(amount) as volatility
FROM test
GROUP BY status
HAVING total_revenue > 1000
ORDER BY total_revenue DESC;
GO

-- Regional analysis
SELECT 
    region,
    COUNT(*) as sales_count,
    SUM(amount) as total_sales,
    AVG(amount) as avg_sale,
    VARIANCE(amount) as sales_variance,
    MIN(amount) as min_sale,
    MAX(amount) as max_sale
FROM test
WHERE status = 'completed'
GROUP BY region
ORDER BY total_sales DESC;
GO

-- Category statistics with volatility measure
SELECT 
    category,
    COUNT(*) as sales,
    SUM(quantity) as total_units,
    AVG(amount) as avg_sale,
    STDDEV(amount) as sale_volatility,
    SUM(amount - discount) as net_revenue
FROM test
WHERE status = 'completed'
GROUP BY category
HAVING sales >= 3
ORDER BY net_revenue DESC;
GO

-- Note: The following would be useful but are not yet implemented:
-- COUNT(DISTINCT column) - Count unique values
-- MEDIAN(column) - Median value
-- MODE(column) - Most frequent value  
-- PERCENTILE(column, 0.5) - Percentile calculations