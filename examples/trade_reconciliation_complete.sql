-- #! ../data/trade_reconciliation.csv

-- Complete trade reconciliation example using the preprocessor pattern
-- This shows how nested column dependencies would be automatically lifted

-- Step 1: Extract root order ID and rank within each order
-- (This is what the preprocessor would generate from a natural query)
WITH __lifted_root AS (
    SELECT
        *,
        CASE
            WHEN CONTAINS(PlatformOrderId, '|') = TRUE THEN SUBSTRING_AFTER(PlatformOrderId, '|', 1)
            ELSE PlatformOrderId
        END AS root
    FROM trade_reconciliation
),
-- Step 2: Add ranking within each root order
__with_rank AS (
    SELECT
        *,
        ROW_NUMBER() OVER (PARTITION BY root ORDER BY PlatformOrderId) AS rank
    FROM __lifted_root
),
-- Step 3: Add lead to get next deal in sequence
__with_lead AS (
    SELECT
        *,
        LEAD(DealId, 1) OVER (PARTITION BY root ORDER BY rank) AS next_deal_id,
        LEAD(Environment, 1) OVER (PARTITION BY root ORDER BY rank) AS next_environment
    FROM __with_rank
),
-- Step 4: Final selection - pair PROD trades with their UAT counterparts
trade_pairs AS (
    SELECT
        DealType,
        root AS order_id,
        DealId AS prod_deal_id,
        next_deal_id AS uat_deal_id,
        SignedQuantity
    FROM __with_lead
    WHERE Environment = 'PROD'
      AND next_environment = 'UAT'
)
SELECT
    DealType AS deal_type,
    order_id,
    prod_deal_id,
    uat_deal_id,
    SignedQuantity AS quantity,
    'Matched' AS status
FROM trade_pairs
ORDER BY deal_type, order_id;
GO