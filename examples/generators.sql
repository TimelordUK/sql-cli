-- #!
-- Generator Functions Examples
-- Demonstrates the use of table-generating functions in SQL queries

-- ============================================================================
-- PRIME NUMBER GENERATION
-- ============================================================================

-- Generate prime numbers up to 30
SELECT * FROM GENERATE_PRIMES(30);
GO

-- Generate prime numbers up to 100, showing only the largest 10
SELECT * FROM GENERATE_PRIMES(100)
ORDER BY prime DESC
LIMIT 10;
GO

-- Generate primes using an expression as the argument
SELECT * FROM GENERATE_PRIMES(10 * 5)
ORDER BY prime DESC
LIMIT 5;
GO

-- ============================================================================
-- PRIME FACTORIZATION
-- ============================================================================

-- Get prime factorization of 1260
SELECT * FROM PRIME_FACTORS(1260);
GO

-- Prime factorization with calculated values
SELECT
    factor,
    power,
    POWER(factor, power) AS value
FROM PRIME_FACTORS(2520);
GO

-- Verify the factorization by multiplying
-- (2^3 * 3^2 * 5 * 7 = 8 * 9 * 5 * 7 = 2520)
SELECT
    2520 AS original,
    (SELECT 8 * 9 * 5 * 7) AS reconstructed
FROM PRIME_FACTORS(2520)
LIMIT 1;
GO

-- ============================================================================
-- FIBONACCI SEQUENCE
-- ============================================================================

-- Generate first 15 Fibonacci numbers
SELECT * FROM FIBONACCI(15);
GO

-- Fibonacci numbers with ratio to previous (approaching golden ratio)
-- Note: We use a CTE to avoid repeating the LAG window function
WITH fib_with_lag AS (
    SELECT
        n,
        value,
        LAG(value) OVER (ORDER BY n) AS prev_value
    FROM FIBONACCI(15)
)
SELECT
    n,
    value,
    prev_value,
    CASE
        WHEN prev_value > 0
        THEN ROUND(value * 1.0 / prev_value, 4)  -- Multiply by 1.0 for float division
        ELSE NULL
    END AS ratio
FROM fib_with_lag
WHERE n > 0;
GO

-- ============================================================================
-- COMBINING GENERATORS WITH CTEs
-- ============================================================================

-- Find prime numbers and their squares
WITH primes AS (
    SELECT * FROM GENERATE_PRIMES(20)
)
SELECT
    prime,
    prime * prime AS squared
FROM primes
WHERE prime < 10;
GO

-- Get Fibonacci values that are less than 100
SELECT value
FROM FIBONACCI(15)
WHERE value > 1 AND value < 100;
GO

-- Check which numbers from 2 to 89 are prime
-- (This identifies Fibonacci primes: 2, 3, 5, 13, 89)
SELECT prime
FROM GENERATE_PRIMES(100)
WHERE prime IN (2, 3, 5, 8, 13, 21, 34, 55, 89);
GO

-- ============================================================================
-- ADVANCED EXAMPLES WITH WINDOW FUNCTIONS
-- ============================================================================

-- Analyze gaps between consecutive primes
WITH primes AS (
    SELECT
        prime,
        ROW_NUMBER() OVER (ORDER BY prime) AS position
    FROM GENERATE_PRIMES(50)
)
SELECT
    prime,
    LEAD(prime) OVER (ORDER BY prime) AS next_prime,
    LEAD(prime) OVER (ORDER BY prime) - prime AS gap
FROM primes
WHERE prime < 40;
GO

-- Find twin primes (primes that differ by 2)
WITH primes AS (
    SELECT
        prime,
        LEAD(prime) OVER (ORDER BY prime) AS next_prime
    FROM GENERATE_PRIMES(100)
)
SELECT
    prime AS twin1,
    next_prime AS twin2
FROM primes
WHERE next_prime - prime = 2;
GO

-- ============================================================================
-- PRACTICAL APPLICATIONS
-- ============================================================================

-- Generate a multiplication table for primes
WITH small_primes AS (
    SELECT prime FROM GENERATE_PRIMES(10)
)
SELECT
    prime,
    prime * 2 AS "x2",
    prime * 3 AS "x3",
    prime * 4 AS "x4",
    prime * 5 AS "x5"
FROM small_primes;
GO

-- Count prime factors for a number
SELECT
    1260 AS number,
    COUNT(*) AS distinct_prime_factors,
    SUM(power) AS total_prime_factors
FROM PRIME_FACTORS(1260);
GO

-- Find the largest prime factor of a number
SELECT
    MAX(factor) AS largest_prime_factor
FROM PRIME_FACTORS(1260);
GO

-- ============================================================================
-- PERFORMANCE CHARACTERISTICS
-- ============================================================================

-- These generators are highly optimized:
-- - GENERATE_PRIMES uses Sieve of Eratosthenes: O(n log log n)
-- - PRIME_FACTORS uses trial division: O(sqrt(n))
-- - FIBONACCI uses iterative generation: O(n)

-- Example: Generate 1000 primes (typically < 5ms)
SELECT COUNT(*) AS prime_count FROM GENERATE_PRIMES(1000);
GO

-- Example: Factorize a large number
SELECT * FROM PRIME_FACTORS(1234567);
GO

-- ============================================================================
-- EXTENSIBILITY
-- ============================================================================

-- The generator framework is extensible. Future generators could include:
-- - GENERATE_RANDOM(count, min, max) - Random number generation
-- - PERMUTATIONS(text) - Generate all permutations of a string
-- - COMBINATIONS(n, k) - Generate combinations
-- - GRAPH_TRAVERSE(start, edges) - Graph traversal algorithms
-- - COLLATZ(n) - Collatz sequence
-- - PASCAL_TRIANGLE(rows) - Pascal's triangle
-- - And many more mathematical sequences and algorithms!

-- End of generator examples