-- Test COUNT(DISTINCT) functionality
-- #!data: ../data/sales_data.csv

-- Example 1: Count distinct salespersons per region
SELECT 
    region,
    COUNT(*) as total_sales,
    COUNT(DISTINCT salesperson) as unique_salespersons,
    COUNT(DISTINCT product) as unique_products
FROM sales_data
GROUP BY region
ORDER BY region;
GO

-- Example 2: Count distinct months per salesperson
SELECT 
    salesperson,
    COUNT(*) as total_records,
    COUNT(DISTINCT month) as months_active,
    COUNT(DISTINCT product) as products_sold,
    SUM(sales_amount) as total_sales,
    SUM(DISTINCT sales_amount) as sum_unique_amounts
FROM sales_data
GROUP BY salesperson
ORDER BY total_sales DESC;
GO

-- Example 3: Overall statistics with DISTINCT
SELECT 
    COUNT(*) as total_records,
    COUNT(DISTINCT region) as unique_regions,
    COUNT(DISTINCT salesperson) as unique_salespersons,
    COUNT(DISTINCT product) as unique_products,
    COUNT(DISTINCT month) as unique_months,
    AVG(DISTINCT sales_amount) as avg_unique_amount
FROM sales_data;
GO

-- Example 4: Combine DISTINCT with window functions
WITH sales_analysis AS (
    SELECT 
        region,
        salesperson,
        COUNT(DISTINCT product) as products_per_person,
        SUM(sales_amount) as total_sales
    FROM sales_data
    GROUP BY region, salesperson
)
SELECT 
    region,
    salesperson,
    products_per_person,
    total_sales,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY products_per_person DESC) as product_diversity_rank
FROM sales_analysis
ORDER BY region, product_diversity_rank;
GO