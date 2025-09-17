-- Test step 1: Just the innermost CTE
SELECT * FROM (
    WITH summary AS (
        SELECT
            DealType,
            PlatformOrderId,
            DealId,
            Environment
        FROM trade_reconciliation
    )
    SELECT * FROM summary
) t;
GO