-- Example 1: Basic RANGE usage
SELECT * FROM RANGE(1, 10);
GO
-- Example 2: RANGE with step
SELECT * FROM RANGE(0, 100, 10);
GO
-- Example 3: Calculate squares
SELECT value, value * value AS squared FROM RANGE(1, 10);
GO
--  Example 4: Fibonacci-like calculation
SELECT value AS n, value * (value + 1) / 2 AS triangular FROM RANGE(1, 20);
GO
-- Example 5: Generate temperature conversion table
SELECT 
    value AS celsius,
    CONVERT(value, 'celsius', 'fahrenheit') AS fahrenheit
FROM RANGE(0, 100, 10);
GO
-- Example 6: Using RANGE with WHERE clause
SELECT value FROM RANGE(1, 100) WHERE value % 7 = 0;
GO
-- Example 7: Prime number check (simple version without recursion)
WITH numbers AS (
    SELECT value AS n FROM RANGE(2, 50)
)
SELECT n 
FROM numbers 
WHERE n = 2 OR n = 3 OR n = 5 OR n = 7 OR n = 11 OR n = 13 OR n = 17 OR n = 19 OR n = 23 OR n = 29 OR n = 31 OR n = 37 OR n = 41 OR n = 43 OR n = 47;
GO