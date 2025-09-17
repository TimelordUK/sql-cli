-- Simple nested CTE test
SELECT * FROM (
    WITH inner_cte AS (
        SELECT 1 as value
    )
    SELECT * FROM inner_cte
) t;
GO