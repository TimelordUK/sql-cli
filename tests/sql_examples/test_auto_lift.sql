-- Test if automatic expression lifting works
-- This query uses 'root' in PARTITION BY which should trigger automatic CTE generation

SELECT
    DealType,
    PlatformOrderId,
    CASE
        WHEN CONTAINS(PlatformOrderId, '|') = TRUE THEN SUBSTRING_AFTER(PlatformOrderId, '|', 1)
        ELSE PlatformOrderId
    END AS root,
    DealId,
    Environment,
    ROW_NUMBER() OVER (PARTITION BY root ORDER BY PlatformOrderId) AS rank
FROM trade_reconciliation
ORDER BY root, rank;
GO