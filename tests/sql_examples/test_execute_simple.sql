-- #! ../../data/sales_data.csv

-- Simple test script without CREATE TEMP TABLE

-- Statement 1: Independent query
SELECT region, COUNT(*) as count1 FROM sales_data GROUP BY region;
GO

-- Statement 2: Another independent query
SELECT product, SUM(amount) as total FROM sales_data GROUP BY product;
GO

-- Statement 3: Third query
SELECT region, AVG(amount) as avg_amount FROM sales_data GROUP BY region ORDER BY avg_amount DESC;
GO
