# Window Function ROW_NUMBER() Bug with CTEs and Non-Contiguous Partitions

## Bug Summary
When using `ROW_NUMBER()` window function with CTEs that reorder data, the function fails to properly reset numbering when the same partition key appears in non-contiguous blocks in the result set.

## Issue Description
The `ROW_NUMBER()` function appears to evaluate based on the original physical row order rather than the logical order established by CTEs. This causes incorrect row numbering when:
1. A CTE reorders data using `ORDER BY`
2. The reordering causes rows with the same partition key to become non-contiguous
3. The window function's `PARTITION BY` doesn't include all columns used in the reordering

## Reproducible Test Case

### Setup Test Data
Create a CSV file `/tmp/window_bug_test.csv`:
```csv
order_id,status,instrument
123,Void,FUT1
123,Void,FUT2
123,Booked,FUT1
123,Booked,FUT2
124,Void,FUT1
124,Booked,FUT1
```

### Demonstrating the Bug

```sql
WITH reordered AS (
    -- Reorder to group by status first (all Void, then all Booked)
    SELECT * FROM window_bug_test
    ORDER BY status DESC, order_id, instrument
),
with_row_numbers AS (
    SELECT *,
           ROW_NUMBER() OVER (PARTITION BY order_id ORDER BY order_id) as rn_by_order
    FROM reordered
)
SELECT * FROM with_row_numbers
ORDER BY status DESC, order_id, instrument;
```

### Expected Result
```
order_id | status | instrument | rn_by_order
---------|--------|------------|------------
123      | Void   | FUT1       | 1  ← Should reset for order_id 123
123      | Void   | FUT2       | 2
124      | Void   | FUT1       | 1
123      | Booked | FUT1       | 1  ← Should reset again for order_id 123
123      | Booked | FUT2       | 2
124      | Booked | FUT1       | 1  ← Should reset for order_id 124
```

### Actual (Buggy) Result
```
order_id | status | instrument | rn_by_order
---------|--------|------------|------------
123      | Void   | FUT1       | 1
123      | Void   | FUT2       | 2
124      | Void   | FUT1       | 1
123      | Booked | FUT1       | 3  ← BUG: Continues from 3 instead of resetting to 1
123      | Booked | FUT2       | 4  ← BUG: Continues to 4
124      | Booked | FUT1       | 2  ← BUG: Continues from 2 instead of resetting to 1
```

## Real-World Scenario: Trade Processing

This bug was discovered when processing futures trades where:
- Trades are first booked to a holding account (status='Void')
- Then allocated to various funds (status='Booked')
- Each OrderId can have multiple instruments
- Need to number trades within each OrderId

### Example Query That Exhibits the Problem
```sql
-- Real trade data scenario
WITH base_trades AS (
    SELECT OrderId, Status, BloombergInstrument, Side, Quantity
    FROM trades
    WHERE OrderId LIKE '%.%'  -- Multi-leg trades
),
ordered_by_status AS (
    -- Group all Void trades together, then all Booked trades
    SELECT * FROM base_trades
    ORDER BY Status, OrderId, BloombergInstrument
),
with_order_numbers AS (
    SELECT *,
           -- This SHOULD reset for each OrderId regardless of Status grouping
           -- But it doesn't due to the bug
           ROW_NUMBER() OVER (PARTITION BY OrderId ORDER BY OrderId) as trade_sequence
    FROM ordered_by_status
)
SELECT * FROM with_order_numbers
ORDER BY Status, OrderId, BloombergInstrument;
```

## Root Cause Analysis

The bug appears to be in `/home/me/dev/sql-cli/src/sql/window_context.rs`:

1. The `WindowContext` is created from the underlying DataTable/DataView
2. When CTEs reorder data, the logical ordering is not preserved in the window context
3. The `get_row_number()` method returns positions based on the original physical order
4. When the same partition key appears in non-contiguous positions after reordering, the numbering continues instead of resetting

### Code Location
- Window context creation: `/home/me/dev/sql-cli/src/data/arithmetic_evaluator.rs:744` (`get_or_create_window_context`)
- Row number retrieval: `/home/me/dev/sql-cli/src/sql/window_context.rs:296` (`get_row_number`)
- Evaluation: `/home/me/dev/sql-cli/src/data/arithmetic_evaluator.rs:881` (ROW_NUMBER case)

## Current Workaround

Include all columns that affect the grouping in the `PARTITION BY` clause:

```sql
-- Instead of:
ROW_NUMBER() OVER (PARTITION BY OrderId ORDER BY instrument)

-- Use:
ROW_NUMBER() OVER (PARTITION BY OrderId, Status ORDER BY instrument)
```

This ensures each (OrderId, Status) combination gets its own partition and numbering resets correctly.

## Test Commands

```bash
# Create test data
cat > /tmp/test_trades.csv << 'EOF'
OrderId,Status,Instrument
123,Void,FUT1
123,Void,FUT2
123,Booked,FUT1
123,Booked,FUT2
EOF

# Test the bug
./target/release/sql-cli /tmp/test_trades.csv -q "
WITH reordered AS (
    SELECT * FROM test_trades
    ORDER BY Status DESC, OrderId
)
SELECT *, ROW_NUMBER() OVER (PARTITION BY OrderId ORDER BY OrderId) as rn
FROM reordered
ORDER BY Status DESC, OrderId
" -o csv

# Expected: rn should be 1,2 for first OrderId=123 group and 1,2 for second
# Actual: rn is 1,2 for first and 3,4 for second (incorrect)
```

## Impact

This bug affects any query where:
1. Window functions are used with CTEs
2. The CTE reorders data
3. The reordering causes partition keys to appear in non-contiguous blocks
4. Common in financial data processing, time-series analysis, and reporting

## Proposed Fix

The window function evaluator needs to:
1. Respect the logical row order from CTEs, not the physical order
2. Reset partition counters when encountering a partition key again after a gap
3. Or, maintain separate window contexts per CTE that preserve the CTE's ordering

## Related Issues

- Similar issues may affect other window functions (RANK, DENSE_RANK, etc.)
- May also impact frame-based window functions (ROWS BETWEEN)
- Could affect performance optimizations that assume contiguous partitions

## Date Discovered
2024-11-26

## Status
**OPEN** - Bug confirmed, workaround available, fix pending