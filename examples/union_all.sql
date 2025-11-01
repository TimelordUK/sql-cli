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

-- Example 7: UNION ALL in IN subquery (actual subquery usage)
-- Demonstrates using UNION ALL inside WHERE IN clause
-- Pattern: WHERE col IN (SELECT val1 UNION ALL SELECT val2 UNION ALL SELECT val3)
-- This creates a virtual value list for filtering
SELECT 1 as id, 'One' as name, 'odd' as type
UNION ALL
SELECT 2 as id, 'Two' as name, 'even' as type
UNION ALL
SELECT 3 as id, 'Three' as name, 'odd' as type
UNION ALL
SELECT 4 as id, 'Four' as name, 'even' as type
UNION ALL
SELECT 5 as id, 'Five' as name, 'odd' as type;
GO

-- Example 8: Filtering with UNION ALL subquery
-- Uses UNION ALL in WHERE IN to filter a result set
-- This demonstrates the actual subquery pattern from the question
WITH numbers AS (
    SELECT 1 as id, 'One' as name, 'odd' as type
    UNION ALL
    SELECT 2 as id, 'Two' as name, 'even' as type
    UNION ALL
    SELECT 3 as id, 'Three' as name, 'odd' as type
    UNION ALL
    SELECT 4 as id, 'Four' as name, 'even' as type
    UNION ALL
    SELECT 5 as id, 'Five' as name, 'odd' as type
)
SELECT * FROM numbers
WHERE id IN (
    SELECT 1 as num
    UNION ALL
    SELECT 2 as num
    UNION ALL
    SELECT 5 as num
);
GO

-- Notes:
-- - UNION ALL keeps all rows, including duplicates (faster performance)
-- - All queries must have the same number of columns
-- - Column names come from the first query
-- - Use UNION (without ALL) to automatically remove duplicate rows
-- - UNION ALL works in subqueries (WHERE IN, FROM clause, etc.)
-- - See examples/union.sql for UNION with deduplication examples
