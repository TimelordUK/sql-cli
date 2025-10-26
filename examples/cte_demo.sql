-- #! data/test_simple_math.csv
-- ============================================================================
-- Common Table Expressions (CTEs) Demo
-- Shows how to use WITH clauses to filter on computed expressions
-- ============================================================================
-- Run: ./target/release/sql-cli data/test_simple_math.csv -f examples/cte_demo.sql -o csv
-- ============================================================================

-- Basic CTE: Create a temporary named result set
WITH simple AS (
    SELECT a, b 
    FROM test 
    WHERE a <= 10
)
SELECT * FROM simple;

-- CTE with computed expressions: Filter on calculated values
WITH calculations AS (
    SELECT 
        a,
        b,
        a * b as product,
        a + b as sum
    FROM test
)
SELECT * FROM calculations WHERE product > 500;

-- Multiple CTEs: Define multiple temporary tables
WITH 
    evens AS (
        SELECT a, b 
        FROM test 
        WHERE MOD(a, 2) = 0
    ),
    odds AS (
        SELECT a, b 
        FROM test 
        WHERE MOD(a, 2) = 1
    )
SELECT * FROM evens WHERE a <= 10;

-- CTE with CASE expressions for categorization
WITH categorized AS (
    SELECT 
        a,
        b,
        CASE 
            WHEN a <= 5 THEN 'small'
            WHEN a <= 15 THEN 'medium'
            ELSE 'large'
        END as size_category
    FROM test
)
SELECT * FROM categorized WHERE size_category = 'medium';

-- Complex expression filtering (the main use case!)
-- This solves the problem of not being able to use aliases in WHERE
WITH prime_factors AS (
    SELECT 
        a,
        MOD(a, 2) = 0 as divisible_by_2,
        MOD(a, 3) = 0 as divisible_by_3,
        MOD(a, 5) = 0 as divisible_by_5
    FROM test
)
SELECT * FROM prime_factors 
WHERE divisible_by_2 = true 
  AND divisible_by_3 = true;