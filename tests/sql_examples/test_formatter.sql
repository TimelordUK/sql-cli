-- Test SQL Formatter
-- Place cursor on any of these queries and press <leader>s= to format

-- Unformatted query 1 - all on one line
SELECT PlatformOrderId, FieldName, "Prod-Value", "Test-Value" FROM trade_reconciliation WHERE FieldName = 'settlementDate' AND IS_DATE("Prod-Value") = true AND IS_DATE("Test-Value") = true ORDER BY PlatformOrderId DESC LIMIT 10;

-- Unformatted query 2 - messy formatting
SELECT id,name,
price,quantity FROM products WHERE price>100
AND status='active' OR category='electronics' ORDER BY 
price DESC,name ASC LIMIT 50;

-- Unformatted query 3 - with CASE statement
SELECT order_id, CASE WHEN amount > 1000 THEN 'High' WHEN amount > 500 THEN 'Medium' ELSE 'Low' END as priority, customer_name FROM orders WHERE status = 'pending' AND created_date > '2024-01-01' ORDER BY amount DESC;

-- Unformatted query 4 - with JOIN
SELECT o.order_id, o.amount, c.customer_name, c.email FROM orders o LEFT JOIN customers c ON o.customer_id = c.id WHERE o.status = 'active' AND c.country = 'UK' ORDER BY o.created_date DESC LIMIT 100;

-- After formatting with <leader>s=, these should be nicely formatted!
-- The formatter will:
-- - Put each SELECT column on its own line
-- - Put major clauses (FROM, WHERE, ORDER BY, etc.) on new lines
-- - Properly indent AND/OR conditions
-- - Format CASE statements with proper indentation
-- - Handle JOINs with proper formatting