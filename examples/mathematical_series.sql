-- Classical Mathematical Series Functions
-- Demonstrates various series summations and sequences

-- 1. Compare different series for n=10
SELECT 
    10 as n,
    SUM_N(10) as triangular,      -- 1+2+3+...+10 = 55
    SUM_N_SQR(10) as squares,      -- 1²+2²+3²+...+10² = 385
    SUM_N_CUBE(10) as cubes,       -- 1³+2³+3³+...+10³ = 3025
    HARMONIC(10) as harmonic,      -- 1 + 1/2 + 1/3 + ... + 1/10 ≈ 2.929
    FACTORIAL(10) as factorial     -- 10! = 3628800
FROM DUAL;
GO

-- 2. Beautiful identity: sum of cubes equals square of sum!
WITH series AS (
    SELECT value as n FROM RANGE(1, 10)
)
SELECT 
    n,
    SUM_N(n) as sum_n,
    SUM_N_CUBE(n) as sum_cubes,
    POWER(SUM_N(n), 2) as sum_squared,
    CASE 
        WHEN SUM_N_CUBE(n) = POWER(SUM_N(n), 2) THEN 'YES ✓' 
        ELSE 'NO' 
    END as identity_holds
FROM series
ORDER BY n;
GO

-- 3. Fibonacci sequence
WITH fib_sequence AS (
    SELECT value as n FROM RANGE(0, 15)
)
SELECT 
    n,
    FIBONACCI(n) as fib_n,
    CASE 
        WHEN n > 0 THEN FIBONACCI(n-1) 
        ELSE 0 
    END as fib_prev,
    CASE 
        WHEN n > 1 THEN ROUND(CAST(FIBONACCI(n) AS FLOAT) / FIBONACCI(n-1), 3)
        ELSE 0 
    END as ratio
FROM fib_sequence
ORDER BY n;
GO

-- 4. Harmonic series convergence (slowly!)
WITH harmonic_series AS (
    SELECT value as n FROM RANGE(1, 20)
)
SELECT 
    n,
    ROUND(HARMONIC(n), 6) as H_n,
    ROUND(HARMONIC(n) - LN(n), 6) as euler_gamma_approx,
    ROUND(1.0/n, 6) as term_added
FROM harmonic_series
WHERE n IN (1, 2, 3, 5, 10, 15, 20)
ORDER BY n;
GO

-- 5. Geometric series examples
SELECT 
    GEOMETRIC(1, 2, 10) as powers_of_2,      -- 1 + 2 + 4 + ... + 512 = 1023
    GEOMETRIC(1, 0.5, 20) as halving_series,  -- Converges to ~2
    GEOMETRIC(1, 3, 6) as powers_of_3,        -- 1 + 3 + 9 + 27 + 81 + 243 = 364
    GEOMETRIC(5, 2, 4) as custom_series       -- 5 + 10 + 20 + 40 = 75
FROM DUAL;
GO

-- 6. Comparing growth rates
WITH growth_comparison AS (
    SELECT value as n FROM RANGE(1, 12)
)
SELECT 
    n,
    SUM_N(n) as linear,           -- O(n²)
    SUM_N_SQR(n) as quadratic,     -- O(n³)
    SUM_N_CUBE(n) as cubic,        -- O(n⁴)
    FIBONACCI(n) as exponential,   -- O(φⁿ)
    FACTORIAL(n) as factorial      -- O(n!)
FROM growth_comparison
WHERE n <= 10
ORDER BY n;
GO

-- 7. Mathematical constants from series
SELECT 
    SQRT(6 * HARMONIC(10000)) as pi_approx,      -- π ≈ 3.1414... (gets better with larger n)
    POWER(1 + 1.0/10000, 10000) as e_approx,     -- e ≈ 2.7181...
    CAST(FIBONACCI(20) AS FLOAT) / FIBONACCI(19) as golden_ratio  -- φ ≈ 1.618...
FROM DUAL;
GO