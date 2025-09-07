-- RANGE() Function and Prime Number Analysis Examples
-- Demonstrates the power of virtual table generation with mathematical functions
-- Run with: sql-cli -f examples/range_and_primes.sql -o table

-- ============================================================================
-- PART 1: Basic RANGE() Function Usage
-- ============================================================================

-- Generate numbers 1 to 10
SELECT value FROM RANGE(1, 10);

GO

-- Generate numbers with step (every 10th number)
SELECT value FROM RANGE(0, 100, 10);

GO

-- Generate block numbers for our analysis
SELECT value AS block_number FROM RANGE(1, 10);

GO

-- ============================================================================
-- PART 2: Prime Number Analysis - Building Blocks
-- ============================================================================

-- Step 1: Calculate boundaries for 100-number blocks
SELECT 
    value AS block,
    value * 100 AS upper_boundary,
    (value - 1) * 100 AS lower_boundary
FROM RANGE(1, 10);

GO

-- Step 2: Apply PRIME_PI to get cumulative prime counts at each boundary
SELECT 
    value AS block,
    value * 100 AS boundary,
    PRIME_PI(value * 100) AS primes_up_to_boundary,
    PRIME_PI((value - 1) * 100) AS primes_up_to_prev_boundary
FROM RANGE(1, 10);

GO

-- Step 3: Calculate primes in each block using the difference
SELECT 
    value AS block,
    PRIME_PI(value * 100) AS cumulative,
    PRIME_PI((value - 1) * 100) AS prev_cumulative,
    PRIME_PI(value * 100) - PRIME_PI((value - 1) * 100) AS primes_in_block
FROM RANGE(1, 10);

GO

-- ============================================================================
-- PART 3: Complete Prime Distribution Analysis
-- ============================================================================

-- Final analysis: Prime distribution by 100-number blocks from 1-1000
SELECT 
    value AS block_num,
    PRIME_PI(value * 100) - PRIME_PI((value - 1) * 100) AS primes_in_block,
    ROUND((PRIME_PI(value * 100) - PRIME_PI((value - 1) * 100)) * 100.0 / 100, 1) AS density_pct,
    PRIME_PI(value * 100) AS total_primes_up_to_block
FROM RANGE(1, 10);

GO

-- ============================================================================
-- PART 4: Specific Prime Examples
-- ============================================================================

-- List first 10 primes using IS_PRIME
SELECT value AS prime FROM RANGE(2, 30) WHERE IS_PRIME(value) = true;

GO

-- Count primes in first block (1-100)
SELECT COUNT(*) AS prime_count FROM RANGE(2, 100) WHERE IS_PRIME(value) = true;

GO

-- List all primes in the last block (901-1000)
SELECT value AS prime FROM RANGE(901, 1000) WHERE IS_PRIME(value) = true;

GO

-- ============================================================================
-- PART 5: Other RANGE() Applications
-- ============================================================================

-- Generate squares
SELECT value, value * value AS squared FROM RANGE(1, 10);

GO

-- Generate triangular numbers
SELECT value AS n, value * (value + 1) / 2 AS triangular FROM RANGE(1, 10);

GO

-- Generate temperature conversion table
SELECT 
    value AS celsius,
    CONVERT(value, 'celsius', 'fahrenheit') AS fahrenheit,
    CONVERT(value, 'celsius', 'kelvin') AS kelvin
FROM RANGE(0, 100, 10);

GO

-- Generate Fibonacci-like sequence (each number with its double)
SELECT value AS n, value * 2 AS double, value * 3 AS triple FROM RANGE(1, 10);

GO

-- ============================================================================
-- PART 6: Using CTEs with RANGE
-- ============================================================================

-- Simple CTE with RANGE
WITH numbers AS (
    SELECT value AS n FROM RANGE(1, 5)
)
SELECT n, n * n AS squared FROM numbers;

GO

-- Find multiples of 7 up to 100
WITH numbers AS (
    SELECT value FROM RANGE(1, 100)
)
SELECT value FROM numbers WHERE value % 7 = 0;

GO

-- ============================================================================
-- Summary:
-- RANGE() enables powerful numerical analysis without needing CSV files
-- Combined with PRIME_PI(), IS_PRIME(), and other functions, complex 
-- mathematical analysis becomes simple SQL queries
-- The elegant formula PRIME_PI(n) - PRIME_PI(n-100) gives prime count
-- in each block without checking every number individually
-- ============================================================================