-- Test UNION ALL functionality

-- Test 1: Simple UNION ALL with string literals
SELECT 'hello' as word
UNION ALL
SELECT 'world' as word;
GO

-- Test 2: UNION ALL with numbers
SELECT 1 as num, 'first' as label
UNION ALL
SELECT 2 as num, 'second' as label;
GO

-- Test 3: Chained UNION ALL (3 queries)
SELECT 'apple' as fruit, 1 as qty
UNION ALL
SELECT 'banana' as fruit, 2 as qty
UNION ALL
SELECT 'cherry' as fruit, 3 as qty;
GO

-- Test 4: UNION ALL with NULL values
SELECT 'data' as col1, NULL as col2
UNION ALL
SELECT NULL as col1, 'value' as col2;
GO

-- Test 5: UNION ALL with arithmetic expressions
SELECT 1+1 as result, 'sum' as op
UNION ALL
SELECT 2*3 as result, 'multiply' as op;
GO
