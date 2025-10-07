-- #! ../data/sales_data.csv

-- ============================================================================
-- WEB CTE Array Expansion Example
-- ============================================================================
-- Demonstrates injecting temp table data as JSON arrays into POST bodies
-- This is the KEY feature for multi-system data integration workflows
-- Template syntax: #table.column expands to JSON array

SELECT region, COUNT(*) as cnt
INTO #regions
FROM sales_data
GROUP BY region;
GO

SELECT * FROM #regions ORDER BY region;
GO

-- Array expansion: ${#regions.region} should become ["East","North","South","West"]
WITH WEB result AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    BODY '{
        "trade_ids": ${#regions.region}
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT * FROM result LIMIT 5;
GO
