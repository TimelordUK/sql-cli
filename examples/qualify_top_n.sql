-- #! ../data/sales_data.csv
-- QUALIFY clause demonstration: Filtering on window function results
-- QUALIFY is Snowflake-style syntax that simplifies filtering on window functions
-- without needing nested subqueries

-- Example 1: Top 3 sales per region
SELECT
    region,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) AS rn
FROM sales
QUALIFY rn <= 3;
GO

-- Example 2: Bottom 2 sales per region (ascending order)
SELECT
    region,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount ASC) AS rn
FROM sales
QUALIFY rn <= 2;
GO

-- Example 3: QUALIFY with complex conditions (top sales above threshold)
SELECT
    region,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) AS rn
FROM sales
QUALIFY rn <= 3 AND sales_amount > 15000;
GO

-- Example 4: Only the top sale per region
SELECT
    region,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) AS rn
FROM sales
QUALIFY rn = 1;
GO
