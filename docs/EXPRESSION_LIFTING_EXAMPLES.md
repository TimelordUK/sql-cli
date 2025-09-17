# Expression Lifting Examples - Working Implementation

## Overview

The expression lifting preprocessor automatically generates CTEs when column aliases are referenced within the same SELECT statement. This allows natural SQL writing while maintaining compatibility with SQL execution requirements.

## Working Example: Trade Reconciliation

### The Problem

You want to write:
```sql
SELECT
    CASE
        WHEN CONTAINS(PlatformOrderId, '|') THEN SUBSTRING_AFTER(PlatformOrderId, '|', 1)
        ELSE PlatformOrderId
    END AS root,
    ROW_NUMBER() OVER (PARTITION BY root) AS rank  -- Can't use 'root' here!
FROM trade_reconciliation
```

This fails because `root` is defined in the same SELECT where it's being used in PARTITION BY.

### The Solution: Automatic CTE Generation

The preprocessor transforms this into:
```sql
WITH __lifted_1 AS (
    SELECT
        *,
        CASE
            WHEN CONTAINS(PlatformOrderId, '|') THEN SUBSTRING_AFTER(PlatformOrderId, '|', 1)
            ELSE PlatformOrderId
        END AS root
    FROM trade_reconciliation
)
SELECT
    root,
    ROW_NUMBER() OVER (PARTITION BY root) AS rank
FROM __lifted_1
```

## Implementation Status

### ✅ What Works Now

1. **Manual CTE construction** - You can write the lifted CTEs manually
2. **Expression Lifter foundation** - The `ExpressionLifter` class exists with:
   - Column alias dependency detection
   - Automatic CTE generation logic
   - Integration points for the preprocessor

3. **Working examples**:
   - `examples/trade_reconciliation_auto_lifted.sql` - Shows the pattern
   - `examples/trade_reconciliation_complete.sql` - Full reconciliation query

### 🔧 What's In Progress

The preprocessor integration to automatically apply these transformations during query execution.

## Real-World Use Case: Trade Reconciliation

### Input Data Structure
```csv
DealType,PlatformOrderId,DealId,Environment
Swap,MKG-THYT-IK-0-0,MKG-THYT-IK-0-0,PROD
Swap,TT202509162008|MKG-THYT-IK-0-0,TT202509162008,UAT
```

### Complete Query Pattern
```sql
-- Step 1: Extract root order (lifted because used in PARTITION BY)
WITH __lifted_root AS (
    SELECT *,
           CASE WHEN CONTAINS(PlatformOrderId, '|')
                THEN SUBSTRING_AFTER(PlatformOrderId, '|', 1)
                ELSE PlatformOrderId
           END AS root
    FROM trade_reconciliation
),
-- Step 2: Rank within each root
__with_rank AS (
    SELECT *,
           ROW_NUMBER() OVER (PARTITION BY root ORDER BY PlatformOrderId) AS rank
    FROM __lifted_root
),
-- Step 3: Get next deal for pairing
__with_lead AS (
    SELECT *,
           LEAD(DealId, 1) OVER (PARTITION BY root ORDER BY rank) AS next_deal_id
    FROM __with_rank
)
-- Step 4: Final selection
SELECT DealType, root AS order_id, DealId AS prod_deal_id, next_deal_id AS uat_deal_id
FROM __with_lead
WHERE Environment = 'PROD'
```

### Output
```csv
deal_type,order_id,prod_deal_id,uat_deal_id
Swap,MKG-THYT-IK-0-0,MKG-THYT-IK-0-0,TT202509162008
```

## How It Works

### 1. Dependency Analysis
The `ExpressionLifter::analyze_column_alias_dependencies()` method:
- Scans SELECT items for alias definitions
- Checks window functions for alias references in PARTITION BY/ORDER BY
- Returns list of dependencies to lift

### 2. CTE Generation
The `lift_column_aliases()` method:
- Creates a CTE with SELECT * plus the computed columns
- Moves the original FROM/WHERE clauses to the CTE
- Updates main query to reference the CTE

### 3. Query Rewriting
- Replace complex expressions with simple column references
- Update FROM clause to reference generated CTEs
- Maintain query semantics while enabling execution

## Benefits

1. **Natural SQL Writing** - Write queries as you think about them
2. **Automatic Optimization** - Complex expressions computed once in CTEs
3. **Cleaner Queries** - Separation of computation and logic
4. **Reusability** - CTEs can be referenced multiple times

## Testing

Run the examples:
```bash
# Test automatic lifting pattern
./target/release/sql-cli -f examples/trade_reconciliation_auto_lifted.sql -o csv

# Test complete reconciliation
./target/release/sql-cli -f examples/trade_reconciliation_complete.sql -o csv
```

## Next Steps

1. **Integrate with query execution** - Apply lifting automatically
2. **Extend patterns** - Handle GROUP BY, HAVING, ORDER BY aliases
3. **Optimization** - Detect when lifting isn't necessary
4. **Performance** - Analyze impact of CTE materialization