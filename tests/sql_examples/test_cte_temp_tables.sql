-- Test: CTEs can now reference temp tables
-- This demonstrates that temp tables are now first-class citizens
-- that can be used in CTEs just like regular tables

-- Example 1: Simple CTE referencing temp table
WITH data AS (
    SELECT value as id, value * 10 as amount
    FROM RANGE(1, 5)
)
SELECT * FROM data INTO #sample_data;
GO

WITH filtered AS (
    SELECT * FROM #sample_data WHERE amount > 20
)
SELECT * FROM filtered ORDER BY id;
GO

-- Example 2: Window functions in CTEs over temp tables
WITH partitioned AS (
    SELECT
        id,
        amount,
        ROW_NUMBER() OVER (ORDER BY amount DESC) as rank
    FROM #sample_data
)
SELECT * FROM partitioned ORDER BY rank;
GO

-- Example 3: Multiple CTEs with temp table reference
WITH doubled AS (
    SELECT id, amount * 2 as doubled FROM #sample_data
),
tripled AS (
    SELECT id, amount * 3 as tripled FROM #sample_data
)
SELECT
    doubled.id,
    doubled.doubled,
    tripled.tripled
FROM doubled
JOIN tripled ON doubled.id = tripled.id
ORDER BY doubled.id;
GO

-- Example 4: Temp table with string filtering
WITH strings AS (
    SELECT
        value as id,
        CASE
            WHEN value % 2 = 0 THEN 'even.type'
            ELSE 'odd.type'
        END as category
    FROM RANGE(1, 6)
)
SELECT * FROM strings INTO #categories;
GO

WITH filtered_cats AS (
    SELECT * FROM #categories WHERE category.Contains('.')
)
SELECT * FROM filtered_cats ORDER BY id;
GO
