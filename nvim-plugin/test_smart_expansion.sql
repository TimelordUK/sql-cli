-- #! ../data/sales_data.csv

-- Test 1: Simple SELECT * (old behavior would need data file hint)
SELECT * FROM data LIMIT 5;
GO

-- Test 2: CTE with SELECT * (this is NEW - didn't work before!)
WITH high_value AS (
  SELECT * FROM data WHERE amount > 100
)
SELECT * FROM high_value LIMIT 10;
GO

-- Test 3: Subquery with SELECT *
SELECT *
FROM (
  SELECT * FROM data WHERE region = 'East'
) AS eastern_sales
LIMIT 5;
GO

-- Test 4: After expansion, column hint comment should appear at top
-- Put cursor on any SELECT * line above and press \sE
-- Expected: Columns are expanded AND a hint comment is added at top

-- Test 5: Test \srD for distinct values
-- Put cursor on "region" and press \srD
-- Expected: Floating window showing distinct region values with counts
