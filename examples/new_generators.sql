-- #!
-- New Generator Functions Examples
-- Demonstrates the extended set of table generators
SELECT *
FROM COLLATZ(7);
GO

  
-- Find the length of Collatz sequences for given starting position
WITH sequences AS (
    SELECT 27, COUNT(*) FROM COLLATZ(27)
)
SELECT * FROM sequences;
GO

-- ============================================================================
-- PASCAL'S TRIANGLE
-- ============================================================================

-- Generate first 6 rows of Pascal's triangle
SELECT * FROM PASCAL_TRIANGLE(6);
GO

-- Get just row 5 of Pascal's triangle (binomial coefficients)
SELECT position, value
FROM PASCAL_TRIANGLE(6)
WHERE row = 5;
GO

-- Sum each row (should be powers of 2: 1, 2, 4, 8, 16, 32...)
SELECT row, SUM(value) AS row_sum
FROM PASCAL_TRIANGLE(10)
GROUP BY row
ORDER BY row;
GO

-- ============================================================================
-- TRIANGULAR AND SQUARE NUMBERS
-- ============================================================================

-- Generate triangular numbers (1, 3, 6, 10, 15...)
SELECT n as t_n, value as t_value FROM TRIANGULAR(10);
GO

-- Generate perfect squares
SELECT * FROM SQUARES(10);
GO

-- Compare triangular numbers with squares
WITH
    tri AS (
        SELECT n AS t_n, value AS t_val
        FROM TRIANGULAR(20)
    ),
    sq AS (
        SELECT n AS s_n, square
        FROM SQUARES(20)
    )
SELECT
    s_n as n,
    t_val AS triangular,
    square,
    square - t_val AS difference
FROM tri
INNER JOIN sq ON tri.t_n = sq.s_n;
GO

-- ============================================================================
-- FACTORIALS
-- ============================================================================

-- Generate factorials up to 10!
SELECT * FROM FACTORIALS(10);
GO

-- Show factorial growth rate
SELECT
    n,
    factorial,
    ROUND(factorial * 1.0 / LAG(factorial) OVER (ORDER BY n), 2) AS growth_factor
FROM FACTORIALS(10)
WHERE n > 0;
GO

-- ============================================================================
-- RANDOM NUMBER GENERATION
-- ============================================================================

-- Generate 10 random integers between 1 and 100
SELECT * FROM RANDOM_INT(10, 1, 100, 42);  -- seed=42 for reproducibility
GO

-- Generate random floats between 0 and 1
SELECT * FROM RANDOM_FLOAT(5, 0, 1, 123);  -- seed=123
GO

-- Simulate dice rolls (d6)
WITH
    rolls AS (
        SELECT id AS roll_number, value AS dice_result
        FROM RANDOM_INT(20, 1, 6, 999)
    )
SELECT
    roll_number + 1 AS roll_number,
    dice_result
FROM rolls;
GO

WITH
    random AS (
        SELECT id AS num, value AS result
        FROM RANDOM_INT(1000, 1, 10, 42)
    )
SELECT *
FROM random;
GO

-- Generate histogram of random distribution
WITH
    random AS (
        SELECT value
        FROM RANDOM_INT(1000, 1, 10, 42)
    ),
    counts AS (
        SELECT
            value,
            COUNT('*') AS frequency
        FROM random
        GROUP BY value
    )
SELECT
    *,
    REPEAT('*', frequency / 10) AS histogram
FROM counts;
GO


-- ============================================================================
-- UUID GENERATION
-- ===========================================================================

-- Generate 5 UUIDs
SELECT * FROM GENERATE_UUID(5);
GO

-- Create mock user data with UUIDs
WITH users AS (
    SELECT
        uuid,
        'User_' || id AS username
    FROM GENERATE_UUID(10)
)
SELECT * FROM users;
GO

-- ============================================================================
-- COMBINING GENERATORS
-- ============================================================================

-- Find perfect square triangular numbers
WITH tri AS (SELECT value FROM TRIANGULAR(100)),
     sq AS (SELECT square FROM SQUARES(100))
SELECT
    tri.value AS perfect_square_triangular
FROM tri
WHERE tri.value IN (SELECT square FROM sq);
GO

-- Generate a multiplication table for primes
-- WITH small_primes AS (
   -- SELECT prime FROM GENERATE_PRIMES(20)
-- )
-- SELECT
    -- p1.prime AS prime1,
    -- p2.prime AS prime2,
    -- p1.prime * p2.prime AS product
-- FROM small_primes p1, small_primes p2
-- WHERE p1.prime <= 10 AND p2.prime <= 10
-- ORDER BY p1.prime, p2.prime;
-- GO

-- Pascal's triangle and combinations
-- The values in Pascal's triangle are C(n,k) - combinations
SELECT
    row AS n,
    position AS k,
    value AS combinations
FROM PASCAL_TRIANGLE(8)
WHERE row = 7;  -- Shows C(7,0) through C(7,7)
GO

-- ============================================================================
-- STATISTICAL EXAMPLES
-- ============================================================================

-- Monte Carlo estimation of Pi
-- Generate random points and check if inside unit circle
-- WITH points AS (
    -- SELECT
        -- x.value AS x,
        -- y.value AS y,
        -- x.value * x.value + y.value * y.value AS distance_squared
    -- FROM
        -- RANDOM_FLOAT(10000, -1, 1, 1) x,
        -- RANDOM_FLOAT(10000, -1, 1, 2) y
    -- WHERE x.id = y.id
-- )
-- SELECT
    -- 4.0 * COUNT(CASE WHEN distance_squared <= 1 THEN 1 END) / COUNT(*) AS pi_estimate
-- FROM points;
GO

-- ============================================================================
-- ADVANCED MATHEMATICAL RELATIONSHIPS
-- ============================================================================

-- Verify factorial formula: n! = n * (n-1)!
WITH facts AS (
    SELECT
        n,
        factorial,
        LAG(factorial) OVER (ORDER BY n) AS prev_factorial
    FROM FACTORIALS(10)
)
SELECT
    n,
    factorial,
    prev_factorial,
    n * prev_factorial AS calculated,
    factorial = n * prev_factorial AS formula_verified
FROM facts
WHERE n > 0;
GO

-- End of new generator examples
