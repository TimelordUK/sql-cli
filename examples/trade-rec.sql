-- #! ../data/trade_field_differences.csv


SELECT PlatformOrderId, "Prod-Value", "Test-Value", Difference
FROM trade_field_differences;
go


SELECT * FROM trade_field_differences 
WHERE 
  IS_DATE("Prod-Value") = true;
go

SELECT
    PlatformOrderId,
    count(*) as difference_count
FROM trade_field_differences
GROUP BY PlatformOrderId
ORDER BY PlatformOrderId;
GO

SELECT
    PlatformOrderId,
    FieldName,
    "Prod-Value",
    "Test-Value",
    Difference,
    DateDiff('day',
    "Prod-Value",
    "Test-Value") as delta_days
FROM trade_field_differences
WHERE IS_DATE("Prod-Value") = true;
GO

with x as (
SELECT 
    PlatformOrderId, 
    FieldName, 
    "Prod-Value", 
    "Test-Value", 
    Difference,
    DateDiff('day', "Prod-Value", "Test-Value") as delta_days
FROM trade_field_differences
WHERE IS_DATE("Prod-Value") = true
) 
select
    PlatformOrderId,
    (Difference-delta_days) as as_expected
FROM x;
GO

SELECT
    PlatformOrderId,
    FieldName,
    "Prod-Value",
    IS_DATE("Prod-Value") as is_date,
    IS_BOOL("Prod-Value") as is_bool,
    IS_NUMERIC("Prod-Value") as is_numeric,
    IS_FLOAT("Prod-Value") as is_float,
    IS_INTEGER("Prod-Value") as is_int
FROM trade_field_differences;
GO


