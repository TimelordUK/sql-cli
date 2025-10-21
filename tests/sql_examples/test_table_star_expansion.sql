-- Test table-scoped star expansion feature
-- Tests: SELECT table.* syntax in JOINs

-- Test 1: Basic table.* with explicit columns
-- Should return: users.id, users.name, users.email, item_name, weight
WITH
    WEB users AS (URL 'file://data/test_users.csv' FORMAT CSV),
    WEB items AS (URL 'file://data/test_items.csv' FORMAT CSV)
SELECT users.*, items.name as item_name, items.weight
FROM users
JOIN items ON users.id = items.user_id
ORDER BY users.id, items.id
LIMIT 2;
GO

-- Test 2: Multiple table stars
-- Should return all columns from users, then all columns from items
WITH
    WEB users AS (URL 'file://data/test_users.csv' FORMAT CSV),
    WEB items AS (URL 'file://data/test_items.csv' FORMAT CSV)
SELECT users.*, items.*
FROM users
JOIN items ON users.id = items.user_id
WHERE users.id = 1;
GO

-- Test 3: Table star after specific columns
-- Should return: item_name, weight, then all user columns
WITH
    WEB users AS (URL 'file://data/test_users.csv' FORMAT CSV),
    WEB items AS (URL 'file://data/test_items.csv' FORMAT CSV)
SELECT items.name as item_name, items.weight, users.*
FROM users
JOIN items ON users.id = items.user_id
WHERE users.id = 2;
GO

-- Test 4: Unscoped star (backward compatibility)
-- Should return all columns
WITH
    WEB users AS (URL 'file://data/test_users.csv' FORMAT CSV)
SELECT *
FROM users
WHERE id = 1;
GO

-- Test 5: Only scoped stars (no explicit columns)
-- Should return items.* then users.*
WITH
    WEB users AS (URL 'file://data/test_users.csv' FORMAT CSV),
    WEB items AS (URL 'file://data/test_items.csv' FORMAT CSV)
SELECT items.*, users.*
FROM users
JOIN items ON users.id = items.user_id
WHERE users.id = 3
ORDER BY items.id;
GO
