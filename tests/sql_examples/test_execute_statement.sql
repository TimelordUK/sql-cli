-- #! ../../data/sales_data.csv

-- Test script for --execute-statement feature
-- This script creates temp tables and shows dependency relationships

-- Statement 1: Create a temp table from base data
CREATE TEMP TABLE raw_sales AS
SELECT * FROM sales_data WHERE region != '';
GO

-- Statement 2: Independent query (should be skipped when targeting statement 3)
SELECT COUNT(*) as total_products FROM sales_data;
GO

-- Statement 3: Query using temp table from statement 1
SELECT
    region,
    COUNT(*) as order_count,
    SUM(amount) as total_sales
FROM raw_sales
GROUP BY region
ORDER BY total_sales DESC;
GO

-- Statement 4: Another temp table
CREATE TEMP TABLE high_value AS
SELECT * FROM raw_sales WHERE amount > 500;
GO

-- Statement 5: Query using temp table from statement 4 (which depends on 1)
SELECT
    region,
    AVG(amount) as avg_amount
FROM high_value
GROUP BY region;
GO
