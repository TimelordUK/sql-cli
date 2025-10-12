SELECT region, COUNT(*) as count1 FROM sales_data GROUP BY region;
GO

SELECT product, SUM(amount) as total FROM sales_data GROUP BY product;
GO

SELECT region, AVG(amount) as avg_amount FROM sales_data GROUP BY region ORDER BY avg_amount DESC;
GO
