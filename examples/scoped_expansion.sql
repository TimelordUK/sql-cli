-- #! ../data/test_users.csv
-- 
-- Scoped Table Star Expansion - SELECT table.* syntax
-- 
-- This example demonstrates how to use table-scoped star expansion
-- to select all columns from specific tables in JOINs.
--
-- Syntax: SELECT table_name.* 
-- This expands to all columns from that specific table only.

-- Example 1: Select all user columns + specific item columns
-- Result: id, name, email (from users) + item_name, weight (from items)
WITH
    WEB users AS (URL 'file://data/test_users.csv' FORMAT CSV),
    WEB items AS (URL 'file://data/test_items.csv' FORMAT CSV)
SELECT 
    users.*,                    -- Expands to: id, name, email
    items.name as item_name,    -- Just the item name
    items.weight                -- Just the item weight
FROM users
JOIN items ON users.id = items.user_id
ORDER BY users.id, items.id
LIMIT 5;
GO

-- Example 2: Multiple table stars
-- Result: All user columns, then all item columns
-- Duplicate column names get suffixes (e.g., id, name become id_right, name_right)
WITH
    WEB users AS (URL 'file://data/test_users.csv' FORMAT CSV),
    WEB items AS (URL 'file://data/test_items.csv' FORMAT CSV)
SELECT 
    users.*,    -- Expands to: id, name, email
    items.*     -- Expands to: id_right, user_id, name_right, weight
FROM users
JOIN items ON users.id = items.user_id
WHERE users.id = 1;
GO

-- Example 3: Mix explicit columns with scoped star
-- Result: Specific calculations first, then all user details
WITH
    WEB users AS (URL 'file://data/test_users.csv' FORMAT CSV),
    WEB items AS (URL 'file://data/test_items.csv' FORMAT CSV)
SELECT
    items.name as product,
    items.weight,
    ANSI_COLOR('green', items.weight || ' kg') as weight_display,
    users.*                     -- All user info: id, name, email
FROM users
JOIN items ON users.id = items.user_id
WHERE items.weight > 1.0
ORDER BY weight DESC;
GO

-- Example 4: Backward compatibility - unscoped star still works
-- Result: All columns from the table
WITH
    WEB users AS (URL 'file://data/test_users.csv' FORMAT CSV)
SELECT *
FROM users
WHERE id <= 2;
GO

-- Example 5: Practical use case - User orders report
-- Show complete user info with specific order details
WITH
    WEB users AS (URL 'file://data/test_users.csv' FORMAT CSV),
    WEB items AS (URL 'file://data/test_items.csv' FORMAT CSV)
SELECT 
    users.*,                                        -- All user details
    items.name as product_name,
    items.weight,
    ANSI_COLOR('cyan', 'Order #' || items.id) as order_id
FROM users
JOIN items ON users.id = items.user_id
ORDER BY users.name, items.name;
GO
