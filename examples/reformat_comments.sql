exit;
go

-- Example 1: Simple Query with Leading Comments
-- This demonstrates basic comment preservation
-- Author: SQL CLI Team
SELECT id, name, email FROM users WHERE active = true;
GO

-- Example 2: Multi-line Query with Documentation
-- Purpose: Find high-value customers
-- Business Rule: Orders over $1000 in last 30 days
SELECT u.id, u.name, COUNT(*) as order_count, SUM(o.amount) as total_spent FROM users u JOIN orders o ON u.id = o.user_id WHERE o.amount > 1000 GROUP BY u.id, u.name;
GO

-- Example 3: Query with Inline Formatting Issues
-- This query needs reformatting for readability
SELECT p.product_name,p.category,p.price,p.stock_quantity,s.supplier_name FROM products p JOIN suppliers s ON p.supplier_id=s.id WHERE p.stock_quantity<10 AND p.active=true;
GO

-- Example 4: Complex Analytics Query
-- Sales performance analysis
-- Calculates monthly metrics by region
SELECT region,COUNT(DISTINCT customer_id) as unique_customers,SUM(revenue) as total_revenue,AVG(revenue) as avg_revenue FROM sales WHERE sale_date>='2025-01-01' GROUP BY region;
GO

-- Example 5: Simple Aggregation
-- Total order count per user
SELECT user_id,COUNT(*) as order_count FROM orders GROUP BY user_id;
GO

-- Example 6: Filter and Sort
-- Active products sorted by price
SELECT product_id,name,price,category FROM products WHERE active=true;
GO

-- Example 7: Block Comment Style
/*
   Multi-line block comment
   This query retrieves user statistics
   Used for dashboard reporting
*/
SELECT user_id, name, email, created_at FROM users WHERE created_at >= '2025-01-01';
GO

-- Example 8: Mixed Comment Styles
-- Line comment before query
/* Block comment with metadata */
SELECT id, status, total FROM transactions WHERE status = 'completed';
GO

-- Example 9: Data Quality Check
-- Find duplicate email addresses
-- Important: This should return zero rows in production
SELECT email, COUNT(*) as duplicate_count FROM users GROUP BY email;
GO

-- Example 10: Simple Join
-- Combine user and profile data
SELECT u.id, u.name, p.bio, p.avatar_url FROM users u JOIN profiles p ON u.id = p.user_id WHERE u.active = true;
GO
