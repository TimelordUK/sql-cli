-- Inequality JOIN Examples
-- Demonstrates JOIN conditions with comparison operators (<, <=, >, >=, !=)
-- These use nested loop join instead of hash join for non-equality conditions

-- ============================================================================
-- CUMULATIVE SUM using self-join with inequality
-- ============================================================================
-- Classic problem: compute running total up to each position
WITH numbers AS (
    SELECT value as n FROM RANGE(1, 5)
),
numbers2 AS (
    SELECT value as m FROM RANGE(1, 5)
)
SELECT
    n,
    SUM(m) as cumulative_sum
FROM numbers
INNER JOIN numbers2 ON n >= m
GROUP BY n
ORDER BY n;
GO

-- ============================================================================
-- TRIANGULAR NUMBER PAIRS
-- ============================================================================
-- Find all pairs where the first number is less than or equal to the second
WITH nums AS (
    SELECT value as num FROM RANGE(1, 4)
),
nums2 AS (
    SELECT value as num2 FROM RANGE(1, 4)
)
SELECT
    num,
    num2
FROM nums
INNER JOIN nums2 ON num <= num2
ORDER BY num, num2;
GO

-- ============================================================================
-- RANGE OVERLAP DETECTION (simplified)
-- ============================================================================
-- Find which ranges have overlapping starts
-- Note: Compound conditions (AND/OR) in JOIN not yet supported
WITH ranges1 AS (
    SELECT
        value as id,
        value * 10 as start_val,
        value * 10 + 15 as end_val
    FROM RANGE(1, 3)
),
ranges2 AS (
    SELECT
        value as id2,
        value * 10 + 5 as start_val2,
        value * 10 + 20 as end_val2
    FROM RANGE(1, 3)
)
SELECT
    id,
    start_val,
    end_val,
    id2,
    start_val2,
    end_val2
FROM ranges1
INNER JOIN ranges2 ON start_val <= start_val2;
GO

-- ============================================================================
-- COUNT ELEMENTS BELOW THRESHOLD
-- ============================================================================
-- For each threshold value, count how many numbers are below it
WITH thresholds AS (
    SELECT value * 2 as threshold FROM RANGE(1, 5)
),
data_points AS (
    SELECT value as val FROM RANGE(1, 10)
)
SELECT
    threshold,
    COUNT(*) as values_below
FROM thresholds
INNER JOIN data_points ON threshold > val
GROUP BY threshold
ORDER BY threshold;
GO

-- ============================================================================
-- FACTORIAL-LIKE COUNTING using inequality join
-- ============================================================================
-- Count how many factors each number has (all numbers up to n)
WITH nums AS (
    SELECT value as n FROM RANGE(1, 5)
),
factors AS (
    SELECT value as factor FROM RANGE(1, 5)
)
SELECT
    n,
    COUNT(*) as count_factors,
    MIN(factor) as smallest_factor,
    MAX(factor) as largest_factor
FROM nums
INNER JOIN factors ON n >= factor
GROUP BY n
ORDER BY n;
GO

-- ============================================================================
-- LEFT JOIN with inequality - Find values with their larger pairs
-- ============================================================================
WITH small AS (
    SELECT value as s FROM RANGE(1, 3)
),
large AS (
    SELECT value * 2 as l FROM RANGE(1, 3)
)
SELECT
    s,
    l
FROM small
LEFT JOIN large ON s < l
ORDER BY s;
GO

-- ============================================================================
-- PIPELINE PATTERN - Using JOIN result in subsequent CTEs
-- ============================================================================
-- Compute cumulative sum, then apply window functions to the result
WITH numbers AS (
    SELECT value as n FROM RANGE(1, 5)
),
numbers2 AS (
    SELECT value as m FROM RANGE(1, 5)
),
cumulative AS (
    -- First CTE: compute cumulative sum using inequality join
    SELECT
        n as position,
        SUM(m) as running_total
    FROM numbers
    INNER JOIN numbers2 ON n >= m
    GROUP BY n
),
with_analytics AS (
    -- Second CTE: add analytical calculations on top
    SELECT
        position,
        running_total,
        LAG(running_total) OVER (ORDER BY position) as prev_total,
        running_total - LAG(running_total, 1, 0) OVER (ORDER BY position) as increment
    FROM cumulative
)
SELECT * FROM with_analytics
ORDER BY position;
GO

-- ============================================================================
-- Notes on Current Support:
-- ============================================================================
-- 1. All comparison operators work: =, !=, <, <=, >, >=
-- 2. Inequality joins use nested loop algorithm (O(n*m) complexity)
-- 3. Equality joins still use efficient hash join (O(n+m) complexity)
-- 4. Currently requires: left operand from left table, right from right table
--    Example: "numbers.n >= numbers2.m" works
--    But: "numbers2.m <= numbers.n" would need to be rewritten
-- 5. Works with CTEs and RANGE() function for generating test data
-- 6. Can combine with GROUP BY for analytical queries like cumulative sums