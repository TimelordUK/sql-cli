-- #! ../../data/test_simple_math.csv

-- Test UNION ALL with actual table data

-- Test 1: UNION ALL with WHERE clauses on same table
SELECT id, a FROM test_simple_math WHERE id <= 2
UNION ALL
SELECT id, a FROM test_simple_math WHERE id >= 19;
GO

-- Test 2: UNION ALL with different column selections
SELECT id, a, b FROM test_simple_math WHERE id = 1
UNION ALL
SELECT id, a, b FROM test_simple_math WHERE id = 10
UNION ALL
SELECT id, a, b FROM test_simple_math WHERE id = 20;
GO

-- Test 3: UNION ALL with ORDER BY on result
-- Note: This tests that ORDER BY in individual queries works
SELECT id FROM test_simple_math WHERE id < 3 ORDER BY id DESC
UNION ALL
SELECT id FROM test_simple_math WHERE id > 18 ORDER BY id ASC;
GO
