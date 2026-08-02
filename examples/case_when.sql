-- #! ../data/AAPL_data.csv
SELECT *
FROM AAPL_data;
GO

SELECT
    value,
    CASE value
        WHEN 1 THEN 'one'
        WHEN 2 THEN 'two'
        WHEN 3 THEN 'three'
        WHEN 4 THEN 'four'
        WHEN 5 THEN 'five'
        ELSE RPAD(value,5,'#')
    END as label
FROM range(1,7);
GO



