-- #! ../data/food_eaten.csv

-- ============================================================================
-- PIVOT Examples - Transform rows into columns for easier analysis
-- ============================================================================
-- PIVOT is a powerful SQL feature that transforms row data into columnar format.
-- Think of it like creating a spreadsheet pivot table - it rotates your data
-- so that distinct values from one column become new columns in the result.
--
-- Syntax: PIVOT (aggregate_function(column) FOR pivot_column IN (values))
-- ============================================================================

-- Example 1: Basic PIVOT - Food eaten by date
-- This transforms food names into columns, showing amounts eaten each day
SELECT *
FROM (SELECT Date, FoodName, AmountEaten FROM food_eaten) AS src
PIVOT (MAX(AmountEaten) FOR FoodName IN ('Sammich', 'Pickle', 'Apple'));
GO

-- Example 2: Same query but selecting specific columns from PIVOT result
-- You can explicitly choose which columns to display from the pivoted data
SELECT Date, Sammich, Pickle, Apple
FROM (SELECT Date, FoodName, AmountEaten FROM food_eaten) AS src
PIVOT (MAX(AmountEaten) FOR FoodName IN ('Sammich', 'Pickle', 'Apple'));
GO

-- Example 3: PIVOT with SUM aggregate
-- If you had multiple rows per day/food, SUM would add them up
-- (In this dataset, MAX and SUM give same results since we only have one entry per day/food)
SELECT Date, Sammich, Pickle, Apple
FROM (SELECT Date, FoodName, AmountEaten FROM food_eaten) AS src
PIVOT (SUM(AmountEaten) FOR FoodName IN ('Sammich', 'Pickle', 'Apple'));
GO

-- Example 4: PIVOT with filtered data
-- You can filter the source data before pivoting
SELECT Date, Sammich, Apple
FROM (SELECT Date, FoodName, AmountEaten FROM food_eaten WHERE FoodName != 'Pickle') AS src
PIVOT (MAX(AmountEaten) FOR FoodName IN ('Sammich', 'Apple'));
GO

-- ============================================================================
-- How PIVOT Works Under the Hood
-- ============================================================================
-- PIVOT is transformed into CASE expressions with GROUP BY:
--
-- Input:  PIVOT (MAX(AmountEaten) FOR FoodName IN ('Sammich', 'Pickle'))
--
-- Becomes:
-- SELECT Date,
--     MAX(CASE WHEN FoodName = 'Sammich' THEN AmountEaten ELSE NULL END) AS Sammich,
--     MAX(CASE WHEN FoodName = 'Pickle' THEN AmountEaten ELSE NULL END) AS Pickle
-- FROM source
-- GROUP BY Date
--
-- Use --show-transformations to see this transformation in action!
-- ============================================================================

-- Example 5: Understanding NULL values in PIVOT
-- When a combination doesn't exist (like Pickle on 2023-01-03), you get NULL
-- This is because the CASE expression returns NULL when the condition doesn't match
SELECT Date, Sammich, Pickle, Apple
FROM (SELECT Date, FoodName, AmountEaten FROM food_eaten) AS src
PIVOT (MAX(AmountEaten) FOR FoodName IN ('Sammich', 'Pickle', 'Apple'));
GO

-- ============================================================================
-- PIVOT Best Practices
-- ============================================================================
-- 1. Always use a subquery or CTE as the source for PIVOT
--    ✓ Good: FROM (SELECT ...) AS src PIVOT (...)
--    ✗ Bad:  FROM table PIVOT (...)  [requires explicit column list]
--
-- 2. The source should contain EXACTLY the columns you need:
--    - The GROUP BY column(s) (Date in our examples)
--    - The pivot column (FoodName)
--    - The aggregate column (AmountEaten)
--
-- 3. Choose appropriate aggregate functions:
--    - MAX/MIN: When you have one value per group
--    - SUM: When you want to add up multiple values
--    - COUNT: When you want to count occurrences
--    - AVG: When you want averages
--
-- 4. NULL values appear when:
--    - The combination doesn't exist in your data
--    - The aggregate function returns NULL (e.g., MAX of empty set)
-- ============================================================================
