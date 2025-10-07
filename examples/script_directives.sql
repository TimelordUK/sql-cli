-- #! ../data/sales_data.csv

-- Demonstration of script directives: EXIT and [SKIP]
-- These directives give you control over script execution flow

-- =====================================================
-- Example 1: Normal execution
-- =====================================================

-- This query runs normally
SELECT COUNT(*) as total_sales FROM sales_data;
GO

-- =====================================================
-- Example 2: [SKIP] directive - skip this statement
-- =====================================================

-- [SKIP]
-- This query will be skipped due to the [SKIP] directive above
SELECT * FROM sales_data WHERE region = 'NonExistent';
GO

-- =====================================================
-- Example 3: Alternative syntax [IGNORE]
-- =====================================================

-- [IGNORE]
-- [IGNORE] is an alias for [SKIP] - this query will also be skipped
SELECT * FROM invalid_table;
GO

-- =====================================================
-- Example 4: More queries after skipped ones
-- =====================================================

-- This query runs normally after the skipped ones
SELECT region, COUNT(*) as count
FROM sales_data
GROUP BY region;
GO

-- =====================================================
-- Example 5: EXIT statement - stops script here
-- =====================================================

-- EXIT statement stops the script execution
-- Queries after EXIT will not run
EXIT;
GO

-- =====================================================
-- THIS QUERY WILL NEVER RUN
-- =====================================================

-- This query won't execute because EXIT was called above
SELECT 'This will never be printed' as message;
GO
