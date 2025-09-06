-- GROUP BY and aggregate functions
-- sql-cli supports sophisticated grouping and aggregation with HAVING clause
-- 
-- NOTE: These examples require actual data files with the referenced columns.
-- To run these queries, use: sql-cli your_data.csv -q "QUERY"
-- 
-- Example data files needed:
-- - sales_data: Should have columns: category, amount, status
-- - orders: Should have columns: order_date, amount, region, customer_id, status

-- Basic aggregation functions (requires sales_data.csv)
SELECT 
    COUNT(*) as total_rows,
    COUNT(DISTINCT category) as unique_categories,
    SUM(amount) as total_amount,
    AVG(amount) as average_amount,
    MIN(amount) as min_amount,
    MAX(amount) as max_amount,
    STDDEV(amount) as standard_deviation,
    VARIANCE(amount) as variance
FROM sales_data;

-- Simple GROUP BY
SELECT 
    category,
    COUNT(*) as count,
    SUM(amount) as total,
    AVG(amount) as average
FROM sales_data
GROUP BY category
ORDER BY total DESC;

-- Multiple column GROUP BY
SELECT 
    year,
    month,
    category,
    COUNT(*) as transactions,
    SUM(amount) as total_sales,
    AVG(amount) as avg_sale
FROM sales_data
GROUP BY year, month, category
ORDER BY year DESC, month DESC, total_sales DESC;

-- GROUP BY with HAVING clause
SELECT 
    category,
    COUNT(*) as count,
    SUM(amount) as total,
    AVG(amount) as average
FROM sales_data
GROUP BY category
HAVING COUNT(*) > 10 AND AVG(amount) > 100
ORDER BY average DESC;

-- Complex aggregation with expressions
SELECT 
    category,
    COUNT(*) as total_transactions,
    SUM(amount) as total_revenue,
    AVG(amount) as avg_transaction,
    MAX(amount) - MIN(amount) as price_range,
    SUM(amount) / COUNT(*) as calculated_avg,
    COUNT(CASE WHEN amount > 1000 THEN 1 END) as high_value_count
FROM sales_data
GROUP BY category
HAVING SUM(amount) > 10000;

-- Nested aggregations with CASE
SELECT 
    region,
    COUNT(*) as total_sales,
    SUM(CASE WHEN status = 'completed' THEN amount ELSE 0 END) as completed_revenue,
    SUM(CASE WHEN status = 'pending' THEN amount ELSE 0 END) as pending_revenue,
    SUM(CASE WHEN status = 'cancelled' THEN amount ELSE 0 END) as cancelled_amount,
    AVG(CASE WHEN status = 'completed' THEN amount END) as avg_completed_sale
FROM orders
GROUP BY region
HAVING COUNT(*) > 5;

-- Time-based grouping
SELECT 
    EXTRACT(YEAR FROM order_date) as year,
    EXTRACT(MONTH FROM order_date) as month,
    COUNT(*) as orders,
    SUM(amount) as revenue,
    AVG(amount) as avg_order_value,
    MIN(order_date) as first_order,
    MAX(order_date) as last_order
FROM orders
WHERE order_date >= DATEADD('year', -2, TODAY())
GROUP BY EXTRACT(YEAR FROM order_date), EXTRACT(MONTH FROM order_date)
HAVING SUM(amount) > 5000
ORDER BY year DESC, month DESC;

-- Practical example: Customer analysis
SELECT 
    customer_id,
    COUNT(*) as order_count,
    SUM(amount) as lifetime_value,
    AVG(amount) as avg_order_value,
    MIN(order_date) as first_purchase,
    MAX(order_date) as last_purchase,
    DATEDIFF('day', MIN(order_date), MAX(order_date)) as customer_lifetime_days
FROM orders
GROUP BY customer_id
HAVING COUNT(*) >= 3 AND SUM(amount) > 1000
ORDER BY lifetime_value DESC
LIMIT 100;