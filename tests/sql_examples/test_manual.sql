WITH __lifted_1 AS (
    SELECT
        *,
        CASE
            WHEN CONTAINS(PlatformOrderId, '|') = TRUE THEN SUBSTRING_AFTER(PlatformOrderId, '|', 1)
            ELSE PlatformOrderId
        END AS root
    FROM trade_reconciliation
)
SELECT
    root,
    ROW_NUMBER() OVER (PARTITION BY root ORDER BY PlatformOrderId) AS rank
FROM __lifted_1