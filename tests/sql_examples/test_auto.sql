SELECT
    CASE
        WHEN CONTAINS(PlatformOrderId, '|') = TRUE THEN SUBSTRING_AFTER(PlatformOrderId, '|', 1)
        ELSE PlatformOrderId
    END AS root,
    ROW_NUMBER() OVER (PARTITION BY root ORDER BY PlatformOrderId) AS rank
FROM trade_reconciliation