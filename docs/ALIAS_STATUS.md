# Table Alias Implementation Status

## What Works ✅

### Column Access in SELECT
- `SELECT booked.Status, voided.Status` - Works correctly
- Columns from the right table in self-joins get qualified names (e.g., `voided.Status`)
- The qualified name is both the column name and can be used in SELECT expressions

### Basic Join Operations
- `FROM summary booked INNER JOIN summary voided` - Table aliases are parsed and stored
- Join conditions work with simple column names

## What Doesn't Work Yet ❌

### WHERE Clause with Qualified Names
- `WHERE voided.Status = 'Void'` - Does NOT work
- The WHERE clause evaluator doesn't properly resolve the qualified column names
- Workaround: Must reference columns without table prefixes in WHERE

### ORDER BY with Qualified Names
- `ORDER BY booked.Instrument` - Does NOT work
- ORDER BY doesn't recognize table-prefixed columns
- Workaround: Use simple column names

## Current Solution for Trade Reconciliation

Since we can't filter with `voided.Status` in WHERE clause, the working approach is:

```sql
-- This WORKS: Using position-based filtering
WITH summary AS (
    SELECT Instrument, Status, SUM(Quantity) as TotalQuantity
    FROM test_trades
    GROUP BY Instrument, Status
)
SELECT
    Instrument,
    Status as BookedStatus,
    TotalQuantity as BookedQty,
    voided.Status as VoidStatus,
    voided.TotalQuantity as VoidQty
FROM summary booked
INNER JOIN summary voided ON booked.Instrument = voided.Instrument
-- Can see all combinations, then filter in application
```

## Technical Details

The alias resolution works at the column metadata level:
- Columns get proper `qualified_name` set (e.g., `voided.Status`)
- The arithmetic evaluator can find columns by qualified name
- But WHERE clause evaluation doesn't use qualified names for comparison

## Next Steps Needed

1. Fix WHERE clause to use qualified column names when evaluating conditions
2. Fix ORDER BY to recognize table-prefixed columns
3. Support multiple join conditions (AND in ON clause)