-- #! ../data/sales_data.csv
-- QUALIFY clause: Snowflake-style filtering on window function results
--
-- QUALIFY allows filtering on window function results without nested subqueries.
-- The ExpressionLifter automatically detects QUALIFY references to window functions
-- and lifts them to CTEs, then QualifyToWhereTransformer converts QUALIFY to WHERE.

-- Example 1: Top 3 sales per region
SELECT
    region,
    sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) AS rn
FROM sales
QUALIFY rn <= 3
ORDER BY region, rn;
GO
