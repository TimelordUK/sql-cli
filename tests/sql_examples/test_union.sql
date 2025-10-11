-- Test UNION (with deduplication) functionality

-- Test 1: Simple UNION removes duplicates
SELECT 'apple' as fruit
UNION
SELECT 'apple' as fruit
UNION
SELECT 'banana' as fruit;
GO

-- Test 2: UNION with multiple columns - row-level deduplication
SELECT 'apple' as fruit, 'red' as color
UNION
SELECT 'apple' as fruit, 'red' as color
UNION
SELECT 'apple' as fruit, 'green' as color
UNION
SELECT 'banana' as fruit, 'yellow' as color;
GO

-- Test 3: UNION preserves unique rows
SELECT 1 as num, 'first' as label
UNION
SELECT 2 as num, 'second' as label
UNION
SELECT 3 as num, 'third' as label;
GO

-- Test 4: UNION with NULL values
SELECT 'data' as col1, NULL as col2
UNION
SELECT 'data' as col1, NULL as col2
UNION
SELECT NULL as col1, 'value' as col2;
GO

-- Test 5: Mix of UNION and UNION ALL in same query
-- Note: If ANY operation is UNION, deduplication is applied to final result
SELECT 'apple' as fruit
UNION
SELECT 'apple' as fruit
UNION ALL
SELECT 'banana' as fruit;
GO
