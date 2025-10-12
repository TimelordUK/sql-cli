-- Test styled table output
SELECT 
    'Buy' as Side,
    450.25 as ExecutionPrice,
    85 as LatencyMs,
    'Active' as Status
UNION ALL
SELECT 
    'Sell' as Side,
    380.50 as ExecutionPrice,
    320 as LatencyMs,
    'Inactive' as Status
UNION ALL
SELECT 
    'Buy' as Side,
    320.00 as ExecutionPrice,
    150 as LatencyMs,
    'Active' as Status
UNION ALL
SELECT 
    'Sell' as Side,
    410.75 as ExecutionPrice,
    480 as LatencyMs,
    'Pending' as Status;
