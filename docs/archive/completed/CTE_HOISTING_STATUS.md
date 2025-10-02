# CTE Hoisting Implementation Status

## What's Implemented

We've implemented automatic CTE hoisting that transforms nested WITH clauses into a flat list at the query's top level. The implementation includes:

1. **Parser Support**: The parser now accepts WITH clauses inside subqueries
2. **CTE Hoister Module**: `src/query_plan/cte_hoister.rs` implements the hoisting logic
3. **Integration**: Hoisting is integrated into the query execution service

## What Works

### Simple Nested CTEs (No Table References)
```sql
SELECT * FROM (
    WITH inner AS (SELECT 1 as value)
    SELECT * FROM inner
) t
```

### Multi-Level Nested CTEs
```sql
SELECT * FROM (
    WITH level3 AS (
        SELECT * FROM (
            WITH level2 AS (
                SELECT * FROM (
                    WITH level1 AS (SELECT 1 as a, 2 as b)
                    SELECT a + b as result FROM level1
                ) t1
            )
            SELECT result * 2 as doubled FROM level2
        ) t2
    )
    SELECT doubled + 10 as final FROM level3
) t3
```

## Current Limitation

### Table References in Nested CTEs
When a nested CTE references the base table, the table name resolution fails after hoisting:

```sql
-- This DOESN'T work yet:
SELECT * FROM (
    WITH summary AS (
        SELECT col1, col2 FROM my_table  -- Table reference fails after hoisting
    )
    SELECT * FROM summary
) t
```

### Workaround
Use flat CTEs instead of nested ones:

```sql
-- This WORKS:
WITH summary AS (
    SELECT col1, col2 FROM my_table
)
SELECT * FROM summary
```

## Trade Reconciliation Example

The trade reconciliation query should be written using flat CTEs:

```sql
-- Working version with flat CTEs:
WITH summary AS (
    SELECT
        DealType,
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
```

## Technical Details

The issue occurs because:
1. When CTEs are hoisted from nested positions, they lose their connection to the base table context
2. The table name in the CTE refers to the loaded data table, but after hoisting, this reference is not properly resolved
3. The query engine needs to maintain table name mappings through the hoisting process

## Next Steps

To fully support nested CTEs with table references, we need to:
1. Track table name mappings during CTE hoisting
2. Ensure the base table context is preserved when CTEs are moved
3. Update the query engine to handle table name resolution for hoisted CTEs