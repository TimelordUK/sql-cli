WITH
    centigrade AS (
        SELECT value AS deg_c
        FROM RANGE(0, 100, 2)
    ),
    temp_chart AS (
        SELECT
            deg_c,
            ROUND(CONVERT(deg_c, 'celsius', 'fahrenheit'), 2) AS deg_f,
            ROUND(CONVERT(deg_c, 'celsius', 'kelvin'), 2) AS deg_k
        FROM centigrade
    )
SELECT
    *,
    CASE deg_c
        WHEN 0 THEN 'freezing point water'
        WHEN 4 THEN 'max density water'
        WHEN 20 THEN 'room temperature'
        WHEN 36 THEN 'human body temp (avg)'
        WHEN 37 THEN 'human body temp'
        WHEN 38 THEN 'mild fever'
        WHEN 56 THEN 'pasteurization temp'
        WHEN 78 THEN 'ethanol boiling point'
        WHEN 100 THEN 'boiling point water'
        ELSE ''
    END AS notable_temp
FROM temp_chart;

