-- UNION Examples
-- Demonstrates combining multiple SELECT queries with automatic deduplication

-- Example 1: UNION removes duplicate rows
-- Compare with UNION ALL which keeps all duplicates
SELECT 'apple' as fruit
UNION
SELECT 'apple' as fruit
UNION
SELECT 'banana' as fruit;
GO

-- Example 2: UNION with multiple columns
-- Deduplication is row-level - entire row must match to be removed
SELECT 'apple' as fruit, 'red' as color
UNION
SELECT 'apple' as fruit, 'red' as color
UNION
SELECT 'apple' as fruit, 'green' as color
UNION
SELECT 'banana' as fruit, 'yellow' as color;
GO

-- Example 3: Combining unique data from different sources
-- UNION ensures no duplicate customers appear in the result
SELECT 'Alice' as customer, 'New York' as city
UNION
SELECT 'Bob' as customer, 'Boston' as city
UNION
SELECT 'Alice' as customer, 'New York' as city;
GO

-- Example 4: UNION with computed values
-- Useful when you want unique calculation results
SELECT 'Sum' as operation, 5 + 5 as result
UNION
SELECT 'Product' as operation, 5 * 2 as result
UNION
SELECT 'Sum' as operation, 5 + 5 as result;
GO

-- Example 5: Mixed UNION and UNION ALL
-- If ANY operation is UNION (not ALL), entire result is deduplicated
SELECT 'apple' as fruit
UNION
SELECT 'apple' as fruit
UNION ALL
SELECT 'banana' as fruit
UNION ALL
SELECT 'banana' as fruit;
GO

-- Notes:
-- - UNION automatically removes duplicate rows
-- - UNION ALL keeps all rows (faster, no deduplication overhead)
-- - Deduplication is based on entire row, not individual columns
-- - All queries must have the same number of columns
-- - Column names come from the first query
-- - Use UNION when you need unique results, UNION ALL when duplicates are okay
