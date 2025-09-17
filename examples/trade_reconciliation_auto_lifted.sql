-- #! ../data/trade_reconciliation.csv

-- Example showing what the preprocessor would generate automatically
-- from a natural query that uses column aliases in PARTITION BY

-- Original query (what user wants to write):
-- SELECT
--     DealType,
--     PlatformOrderId,
--     CASE
--         WHEN CONTAINS(PlatformOrderId, '|') = TRUE THEN SUBSTRING_AFTER(PlatformOrderId, '|', 1)
--         ELSE PlatformOrderId
--     END AS root,
--     DealId,
--     Environment,
--     ROW_NUMBER() OVER (PARTITION BY root ORDER BY PlatformOrderId) AS rank
-- FROM trade_reconciliation

-- The preprocessor would automatically transform it to:
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
    DealType,
    PlatformOrderId,
    root,
    DealId,
    Environment,
    ROW_NUMBER() OVER (PARTITION BY root ORDER BY PlatformOrderId) AS rank
FROM __lifted_1
ORDER BY root, rank;
GO