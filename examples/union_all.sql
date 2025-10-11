-- UNION ALL Examples
-- Demonstrates combining multiple SELECT queries into a single result set

-- Example 1: Simple UNION ALL with literal values
-- Combines two queries with string literals
SELECT 'North' as region, 100 as sales
UNION ALL
SELECT 'South' as region, 150 as sales;
GO

-- Example 2: Chained UNION ALL
-- Combines three or more queries together
SELECT 'Q1' as quarter, 1000 as revenue
UNION ALL
SELECT 'Q2' as quarter, 1200 as revenue
UNION ALL
SELECT 'Q3' as quarter, 1500 as revenue
UNION ALL
SELECT 'Q4' as quarter, 1800 as revenue;
GO

-- Example 3: UNION ALL with NULL values
-- Shows how NULL values are preserved in the result
SELECT 'Alice' as name, 'Engineering' as department
UNION ALL
SELECT 'Bob' as name, NULL as department
UNION ALL
SELECT NULL as name, 'Marketing' as department;
GO

-- Example 4: UNION ALL with arithmetic expressions
-- Combines computed values from different queries
SELECT 'Addition' as operation, 10 + 5 as result
UNION ALL
SELECT 'Multiplication' as operation, 10 * 5 as result
UNION ALL
SELECT 'Division' as operation, 10 / 2 as result;
GO

-- Example 5: UNION ALL for data consolidation
-- Useful for combining similar data from different time periods or categories
SELECT 'Product A' as product, 'Store 1' as location, 25 as quantity
UNION ALL
SELECT 'Product A' as product, 'Store 2' as location, 30 as quantity
UNION ALL
SELECT 'Product B' as product, 'Store 1' as location, 15 as quantity
UNION ALL
SELECT 'Product B' as product, 'Store 2' as location, 20 as quantity;
GO

-- Notes:
-- - UNION ALL keeps all rows, including duplicates (faster performance)
-- - All queries must have the same number of columns
-- - Column names come from the first query
-- - Use UNION (without ALL) to automatically remove duplicate rows
-- - See examples/union.sql for UNION with deduplication examples
