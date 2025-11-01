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

-- Example 6: UNION ALL standalone (no FROM clause)
-- Creates a virtual result set without querying any table
SELECT 'Value 1' as id, 'First' as label
UNION ALL
SELECT 'Value 2' as id, 'Second' as label
UNION ALL
SELECT 'Value 3' as id, 'Third' as label
UNION ALL
SELECT 'Value 4' as id, 'Fourth' as label
UNION ALL
SELECT 'Value 5' as id, 'Fifth' as label;
GO

-- Example 7: UNION ALL in subquery for virtual value list
-- Demonstrates using UNION ALL inside WHERE IN clause
-- Useful pattern: WHERE id IN (SELECT 1 UNION ALL SELECT 2 UNION ALL SELECT 3)
SELECT 1 as num, 'One' as word
UNION ALL
SELECT 2 as num, 'Two' as word
UNION ALL
SELECT 3 as num, 'Three' as word
UNION ALL
SELECT 5 as num, 'Five' as word
UNION ALL
SELECT 8 as num, 'Eight' as word
UNION ALL
SELECT 13 as num, 'Thirteen' as word;
GO

-- Notes:
-- - UNION ALL keeps all rows, including duplicates (faster performance)
-- - All queries must have the same number of columns
-- - Column names come from the first query
-- - Use UNION (without ALL) to automatically remove duplicate rows
-- - UNION ALL works in subqueries (WHERE IN, FROM clause, etc.)
-- - See examples/union.sql for UNION with deduplication examples
