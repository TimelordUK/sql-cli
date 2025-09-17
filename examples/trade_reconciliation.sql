-- #! ../data/trade_reconciliation.csv

-- Trade reconciliation query showing PROD trades matched with their UAT counterparts
-- This demonstrates nested CTEs that should be automatically hoisted

WITH summary AS (
    SELECT
        DealType,
        SignedQuantity,
        PlatformOrderId,
        CASE
            WHEN CONTAINS(PlatformOrderId, '|') = TRUE THEN SUBSTRING_AFTER(PlatformOrderId, '|', 1)
            ELSE PlatformOrderId
        END AS root,
        DealId,
        Environment
    FROM trade_reconciliation
),
ranked AS (
    SELECT
        DealId,
        DealType,
        PlatformOrderId,
        root,
        Environment,
        ROW_NUMBER() OVER (PARTITION BY root ORDER BY root ASC) AS rank
    FROM summary
),
with_lead AS (
    SELECT
        root,
        DealType,
        DealId,
        Environment,
        rank,
        LEAD(DealId, 1) OVER (ORDER BY root ASC, rank ASC) AS next_deal_id
    FROM ranked
)
SELECT
    DealType AS deal_type,
    root AS order_id,
    DealId AS prod_deal_id,
    next_deal_id AS uat_deal_id
FROM with_lead
WHERE Environment = 'PROD'
ORDER BY deal_type ASC, order_id ASC;
GO