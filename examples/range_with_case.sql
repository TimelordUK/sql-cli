-- Example: Using RANGE() with generated CASE statements
--
-- This demonstrates how to use the --generate-case-range command
-- to create CASE statements for synthetic data from RANGE() function.
--
-- To generate the CASE statement in Neovim:
--   1. Press <leader>sr (Generate CASE for range inline)
--   2. Enter: value
--   3. Enter: 100 (or 1-100)
--   4. Enter: 5
--   5. Select labels or use numeric ranges

-- Basic example with RANGE() and banding
WITH numbers AS (
    SELECT value FROM RANGE(1, 101)
)
SELECT
    value,
    -- Generated using: --generate-case-range --column value --min 1 --max 100 --bands 5
    CASE
        WHEN value <= 20 THEN '1-20'
        WHEN value > 20 AND value <= 40 THEN '20-40'
        WHEN value > 40 AND value <= 60 THEN '40-60'
        WHEN value > 60 AND value <= 80 THEN '60-80'
        WHEN value > 80 THEN '80+'
    END AS value_band
FROM numbers
ORDER BY value;
GO

-- Example with custom labels for score ranges
WITH scores AS (
    SELECT value as score FROM RANGE(0, 101)
)
SELECT
    score,
    -- Generated using: --generate-case-range --column score --min 0 --max 100 --bands 5 --labels "Very Low,Low,Medium,High,Very High"
    CASE
        WHEN score <= 20 THEN 'Very Low'
        WHEN score > 20 AND score <= 40 THEN 'Low'
        WHEN score > 40 AND score <= 60 THEN 'Medium'
        WHEN score > 60 AND score <= 80 THEN 'High'
        WHEN score > 80 THEN 'Very High'
    END AS performance_level
FROM scores
WHERE score % 10 = 0  -- Sample every 10th value
ORDER BY score;
GO

-- Aggregated summary using generated bands
WITH numbers AS (
    SELECT value FROM RANGE(1, 101)
),
banded AS (
    SELECT
        value,
        -- Generated using: --generate-case-range --column value --min 1 --max 100 --bands 4 --labels "Q1,Q2,Q3,Q4"
        CASE
            WHEN value <= 25 THEN 'Q1'
            WHEN value > 25 AND value <= 50 THEN 'Q2'
            WHEN value > 50 AND value <= 75 THEN 'Q3'
            WHEN value > 75 THEN 'Q4'
        END AS quartile
    FROM numbers
)
SELECT
    quartile,
    COUNT(*) as count,
    MIN(value) as min_value,
    MAX(value) as max_value,
    ROUND(AVG(value), 2) as avg_value
FROM banded
GROUP BY quartile
ORDER BY quartile;
GO