-- Test JOIN parsing examples
-- These should parse successfully once JOIN support is complete

-- Simple INNER JOIN
SELECT * FROM users JOIN orders ON users.id = orders.user_id;

-- LEFT JOIN
SELECT * FROM users LEFT JOIN orders ON users.id = orders.user_id;

-- RIGHT JOIN  
SELECT * FROM users RIGHT JOIN orders ON users.id = orders.user_id;

-- FULL OUTER JOIN
SELECT * FROM users FULL OUTER JOIN orders ON users.id = orders.user_id;

-- CROSS JOIN (no ON clause)
SELECT * FROM users CROSS JOIN products;

-- Multiple JOINs
SELECT * 
FROM users u
JOIN orders o ON u.id = o.user_id
JOIN products p ON o.product_id = p.id;

-- JOIN with CTE
WITH active_users AS (
    SELECT * FROM users WHERE active = 1
)
SELECT * 
FROM active_users 
JOIN orders ON active_users.id = orders.user_id;