-- Test file for copy query feature
-- #!data: data/trade_reconciliation.csv

-- Query 1: Find all valid settlement dates
SELECT PlatformOrderId, FieldName, "Prod-Value" 
FROM trade_reconciliation 
WHERE FieldName = 'settlementDate' 
  AND IS_DATE("Prod-Value") = true;
GO

-- Query 2: Calculate settlement delays
SELECT 
    PlatformOrderId, 
    "Prod-Value" as prod_date,
    "Test-Value" as test_date,
    DATEDIFF('day', "Prod-Value", "Test-Value") as delay_days
FROM trade_reconciliation 
WHERE FieldName = 'settlementDate' 
  AND IS_DATE("Prod-Value") = true 
  AND IS_DATE("Test-Value") = true
ORDER BY delay_days DESC;
GO

-- Query 3: Find data quality issues
SELECT PlatformOrderId, FieldName, "Prod-Value"
FROM trade_reconciliation
WHERE (FieldName = 'quantity' AND IS_NUMERIC("Prod-Value") = false)
   OR (FieldName = 'settlementDate' AND IS_DATE("Prod-Value") = false);