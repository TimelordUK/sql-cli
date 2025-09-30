-- Test CTE chain termination
-- Cursor should be placed in the middle CTE (cte2)
-- Expected: Query should stop after cte2 and select from it
-- Actual: It continues to the end

WITH cte1 AS (
    SELECT value as n
    FROM RANGE(1, 10)
),
cte2 AS (
    SELECT n, n * 2 as doubled
    FROM cte1
    WHERE n > 3
),
cte3 AS (
    SELECT doubled, doubled * 10 as final
    FROM cte2
    WHERE doubled < 15
)
SELECT * FROM cte3
WHERE final > 50;
