-- #! ../data/trade_field_differences.csv


SELECT PlatformOrderId, "Prod-Value", "Test-Value", Difference
FROM trade_field_differences;
go


SELECT * FROM trade_field_differences 
WHERE 
  IS_DATE("Prod-Value") = true;
GO

SELECT
    PlatformOrderId,
    COUNT('*') AS difference_count
FROM trade_field_differences
GROUP BY PlatformOrderId
ORDER BY PlatformOrderId ASC;
GO

SELECT
    PlatformOrderId,
    FieldName,
    "Prod-Value",
    "Test-Value",
    Difference,
    DATEDIFF('day', "Prod-Value", "Test-Value") AS delta_days
FROM trade_field_differences
WHERE IS_DATE("Prod-Value") = TRUE;
GO

WITH
    x AS (
        SELECT
            PlatformOrderId,
            FieldName,
            "Prod-Value",
            "Test-Value",
            Difference,
            DATEDIFF('day', "Prod-Value", "Test-Value") AS delta_days
        FROM trade_field_differences
        WHERE IS_DATE("Prod-Value") = TRUE
    )
SELECT
    PlatformOrderId,
    Difference - delta_days AS as_expected
FROM x;
GO

SELECT
    PlatformOrderId,
    FieldName,
    "Prod-Value",
    IS_DATE("Prod-Value") AS is_date,
    IS_BOOL("Prod-Value") AS is_bool,
    IS_NUMERIC("Prod-Value") AS is_numeric,
    IS_FLOAT("Prod-Value") AS is_float,
    IS_INTEGER("Prod-Value") AS is_int
FROM trade_field_differences;
GO


