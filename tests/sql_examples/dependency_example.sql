SELECT * FROM sales_data WHERE amount > 100 ORDER BY amount DESC;
GO

SELECT region, SUM(amount) as total FROM sales_data GROUP BY region;
GO

SELECT region, product, amount FROM sales_data ORDER BY region, amount DESC;
GO
