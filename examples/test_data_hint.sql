-- Test script demonstrating data file hint system
-- #!data: ../data/sales_data.csv
-- This hint tells the script to use sales_data.csv from the data directory

-- Query 1: Show first 5 sales records
SELECT * FROM sales_data 
ORDER BY sales_amount DESC
LIMIT 5;
GO

-- Query 2: Get sales summary by region
SELECT 
    region,
    COUNT(*) as sales_count,
    SUM(sales_amount) as total_sales,
    AVG(sales_amount) as avg_sale
FROM sales_data
GROUP BY region
ORDER BY total_sales DESC;
GO

-- Query 3: Top 3 salespersons by total sales
WITH sales_summary AS (
    SELECT 
        salesperson,
        SUM(sales_amount) as total_sales,
        COUNT(*) as num_sales
    FROM sales_data
    GROUP BY salesperson
)
SELECT 
    salesperson,
    total_sales,
    num_sales,
    ROUND(total_sales / num_sales, 2) as avg_per_sale
FROM sales_summary
ORDER BY total_sales DESC
LIMIT 3;
GO