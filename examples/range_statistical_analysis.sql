-- ============================================================================
-- Statistical Analysis with RANGE and Window Functions
-- ============================================================================
-- This example demonstrates statistical analysis using RANGE to generate data
-- combined with window functions for running totals and group analysis
-- ============================================================================
-- Run: ./target/release/sql-cli -f examples/range_statistical_analysis.sql -o table
-- ============================================================================

-- 1. Basic Statistics: Mean, Variance, and Standard Deviation simulation
WITH numbers AS (
    SELECT value FROM RANGE(1, 100)
),
stats AS (
    SELECT 
        COUNT(*) AS n,
        SUM(value) AS total,
        AVG(value) AS mean,
        MIN(value) AS min_val,
        MAX(value) AS max_val
    FROM numbers
)
SELECT 
    n AS sample_size,
    mean AS average,
    min_val AS minimum,
    max_val AS maximum,
    total AS sum_total
FROM stats;
GO

-- 2. Frequency Distribution: Analyze value distributions in buckets
WITH data_points AS (
    SELECT value FROM RANGE(1, 100)
),
bucketed AS (
    SELECT 
        value,
        FLOOR(value / 10) AS bucket
    FROM data_points
)
SELECT 
    bucket,
    bucket * 10 AS bucket_start,
    (bucket * 10) + 9 AS bucket_end,
    COUNT(*) AS frequency,
    SUM(value) AS bucket_sum
FROM bucketed
GROUP BY bucket
ORDER BY bucket;
GO

-- 3. Prime Number Analysis with PRIME_PI Function
WITH prime_data AS (
    SELECT 
        value,
        IS_PRIME(value) AS is_prime,
        PRIME_PI(value) AS primes_up_to_value
    FROM RANGE(1, 50)
),
prime_values AS (
    SELECT 
        value,
        CASE WHEN is_prime = true THEN 'PRIME' ELSE '' END AS prime_flag,
        primes_up_to_value,
        CASE WHEN is_prime = true THEN value ELSE 0 END AS prime_value
    FROM prime_data
)
SELECT 
    value,
    prime_flag,
    primes_up_to_value,
    SUM(prime_value) OVER (ORDER BY value) AS running_prime_sum
FROM prime_values
WHERE value <= 20
ORDER BY value;
GO

-- 4. Modulo Group Analysis with Window Functions
WITH modulo_groups AS (
    SELECT 
        value,
        value % 5 AS mod_group
    FROM RANGE(1, 25)
)
SELECT 
    value,
    mod_group,
    ROW_NUMBER() OVER (PARTITION BY mod_group ORDER BY value) AS position_in_group,
    SUM(value) OVER (PARTITION BY mod_group) AS group_total,
    COUNT(value) OVER (PARTITION BY mod_group) AS group_size
FROM modulo_groups
ORDER BY value
LIMIT 15;
GO

-- 5. Cumulative Sum Sequence
WITH sequence AS (
    SELECT 
        value AS n,
        value AS term
    FROM RANGE(1, 10)
)
SELECT 
    n,
    term,
    SUM(term) OVER (ORDER BY n) AS cumulative_sum,
    term + LAG(term, 1, 0) OVER (ORDER BY n) AS sum_with_previous
FROM sequence
ORDER BY n;
GO

-- 6. Percentile Groups Analysis
WITH data AS (
    SELECT value FROM RANGE(1, 100)
),
quartiles AS (
    SELECT 
        value,
        CASE 
            WHEN value <= 25 THEN 'Q1'
            WHEN value <= 50 THEN 'Q2'
            WHEN value <= 75 THEN 'Q3'
            ELSE 'Q4'
        END AS quartile
    FROM data
)
SELECT 
    quartile,
    COUNT(*) AS count,
    MIN(value) AS min_value,
    MAX(value) AS max_value,
    AVG(value) AS avg_value,
    SUM(value) AS total
FROM quartiles
GROUP BY quartile
ORDER BY quartile;
GO