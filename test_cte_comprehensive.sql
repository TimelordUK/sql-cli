-- Comprehensive CTE test file
-- To test: place cursor on different lines and press \sC

WITH step1 AS (
    SELECT value as id, value * 10 as amount
    FROM RANGE(1, 5)
),
step2 AS (
    SELECT id, amount, amount / 2 as half
    FROM step1
    WHERE id > 2
),
step3 AS (
    SELECT id, half, half + 100 as adjusted
    FROM step2
    WHERE half < 25
)
SELECT * FROM step3
WHERE adjusted > 110;

-- Test cases:
-- 1. Cursor on line 5 (inside step1) -> Should show only step1
-- 2. Cursor on line 9 (inside step2) -> Should show step1 and step2
-- 3. Cursor on line 13 (inside step3) -> Should show step1, step2, and step3
