-- ============================================================================
-- Mathematical Sequences and Patterns using RANGE
-- ============================================================================
-- This example demonstrates various mathematical sequences and patterns
-- that can be generated using RANGE, prime functions, and window functions
-- ============================================================================
-- Run: ./target/release/sql-cli -f examples/range_mathematical_sequences.sql -o table
-- ============================================================================

-- 1. Prime Numbers Analysi s
WITH is_primes AS (
    SELECT 
        value,
        IS_PRIME(value) AS is_prime,
        PRIME_PI(value) AS cumulative_primes
    FROM RANGE(1, 50)
),
primes as (
  select value, is_prime, cumulative_primes from is_primes where is_prime = true
)
SELECT 
    value,
    CASE WHEN is_prime = true THEN 'PRIME' ELSE '' END AS is_prime,
    cumulative_primes,
    value - LAG(value, 1, 0) OVER (ORDER BY value) AS gap_from_prev
FROM primes
WHERE is_prime = true AND value <= 30
ORDER BY value;
GO

-- 2. Triangular Numbers Sequence
WITH sequence AS (
    SELECT value FROM RANGE(1, 15)
),
triangular AS (
    SELECT 
        value as n,
        SUM_N(value) AS triangular_number
    FROM sequence
)
SELECT 
    n,
    triangular_number,
    n * (n + 1) / 2 AS formula_check,
    triangular_number - LAG(triangular_number, 1, 0) OVER (ORDER BY n) AS difference
FROM triangular
ORDER BY n;
GO

-- 3. Perfect Squares and Their Properties
WITH numbers AS (
    SELECT value FROM RANGE(1, 10)
),
squares AS (
    SELECT 
        value AS n,
        value * value AS perfect_square,
        SQRT(value * value) AS square_root
    FROM numbers
)
SELECT 
    n,
    perfect_square,
    square_root,
    SUM(perfect_square) OVER (ORDER BY n) AS sum_of_squares,
    perfect_square - LAG(perfect_square, 1, 0) OVER (ORDER BY n) AS difference,
    (2 * n - 1) AS expected_diff
FROM squares
ORDER BY n;
GO

-- 4. Collatz Sequence Simulation (3n+1 problem)
-- Shows first few steps for numbers 1-10
WITH starting_numbers AS (
    SELECT value AS n FROM RANGE(1, 10)
),
collatz_step AS (
    SELECT 
        n AS start_value,
        n AS current,
        CASE 
            WHEN n % 2 = 0 THEN n / 2
            ELSE (n * 3) + 1
        END AS next_value,
        CASE 
            WHEN n % 2 = 0 THEN 'Even: n/2'
            ELSE 'Odd: 3n+1'
        END AS operation
    FROM starting_numbers
)
SELECT 
    start_value,
    current,
    next_value,
    operation,
    ROW_NUMBER() OVER (PARTITION BY start_value ORDER BY start_value) AS step_number
FROM collatz_step
ORDER BY start_value;
GO

-- 5. Divisor Count and Perfect Numbers Check
WITH numbers AS (
    SELECT value AS n FROM RANGE(1, 30)
),
divisor_analysis AS (
    SELECT 
        n,
        -- Count divisors (simplified - checking common divisors)
        -- Note: AND/OR not supported in CASE, using mathematical workarounds
        CASE 
            WHEN n % 6 = 0 THEN 'Divisible by 6'
            WHEN IS_PRIME(n) = true THEN 'Prime (2 divisors)'
            WHEN n % 2 = 0 THEN 'Even Composite'
            WHEN n % 3 = 0 THEN 'Divisible by 3'
            WHEN n % 5 = 0 THEN 'Divisible by 5'
            ELSE 'Other'
        END AS divisor_class,
        -- Sum of factors approximation
        CASE 
            WHEN n = 6 THEN 'Perfect (6 = 1+2+3)'
            WHEN n = 28 THEN 'Perfect (28 = 1+2+4+7+14)'
            ELSE 'Not Perfect'
        END AS perfect_status
    FROM numbers
)
SELECT 
    n,
    divisor_class,
    perfect_status,
    COUNT(*) OVER (PARTITION BY divisor_class) AS count_in_class
FROM divisor_analysis
WHERE n <= 30
ORDER BY n
LIMIT 20;
GO

-- 6. Pythagorean Triple Check for Known Values
WITH known_triples AS (
    SELECT 
        value AS n,
        CASE 
            WHEN value = 3 THEN '3-4-5'
            WHEN value = 5 THEN '5-12-13'
            WHEN value = 8 THEN '8-15-17'
            WHEN value = 7 THEN '7-24-25'
            WHEN value = 9 THEN '9-40-41'
            ELSE 'Not a known triple base'
        END AS triple_info
    FROM RANGE(1, 10)
)
SELECT 
    n,
    triple_info,
    n * n AS n_squared
FROM known_triples
WHERE triple_info != 'Not a known triple base'
ORDER BY n;
GO

-- 7. Harmonic Series Approximation
WITH sequence AS (
    SELECT value AS n FROM RANGE(1, 10)
),
harmonic AS (
    SELECT 
        n,
        1.0 / n AS term,
        SUM(1.0 / value) OVER (ORDER BY value) AS harmonic_sum,
        LOG(n) AS ln_n
    FROM sequence
)
SELECT 
    n,
    ROUND(term, 4) AS harmonic_term,
    ROUND(harmonic_sum, 4) AS sum_to_n,
    ROUND(ln_n, 4) AS natural_log,
    ROUND(harmonic_sum - ln_n, 4) AS difference
FROM harmonic
ORDER BY n;
GO