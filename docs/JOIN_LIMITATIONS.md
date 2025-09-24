# SQL CLI Join Limitations and Workarounds

## Current Limitations

### 1. Self-Join Column Name Conflicts
When joining a CTE or table with itself, duplicate column names from the right table are automatically suffixed with `_right`. However:
- These `_right` columns cannot be accessed using table aliases (e.g., `voided.Status_right` doesn't work)
- You must use the bare column name: `Status_right`

### 2. Multiple Join Conditions
The current JOIN implementation only supports a single condition:
- ✅ `ON table1.col = table2.col`
- ❌ `ON table1.col1 = table2.col1 AND table1.col2 = table2.col2`

### 3. Table Alias Resolution
While table aliases are parsed and stored:
- ✅ Works: `SELECT booked.Account FROM summary booked`
- ⚠️ Partial: In self-joins, the second table's columns get `_right` suffix but can't use alias
- ❌ Doesn't work: `voided.Status_right` (must use `Status_right` directly)

## Workarounds

### For Trade Reconciliation (Booked vs Void)

Instead of a complex self-join with multiple conditions, use separate CTEs:

```sql
-- DOESN'T WORK: Multiple join conditions
FROM summary booked
INNER JOIN summary voided
ON booked.Instrument = voided.Instrument
AND booked.Account = voided.Account  -- AND not supported

-- WORKAROUND: Create specific CTEs for each status
WITH booked_summary AS (
    SELECT Account, Instrument, SUM(Quantity) as BookedQuantity
    FROM trades WHERE Status = 'Booked'
    GROUP BY Account, Instrument
),
void_summary AS (
    SELECT Account, Instrument, SUM(Quantity) as VoidQuantity
    FROM trades WHERE Status = 'Void'
    GROUP BY Account, Instrument
)
-- Then use LEFT JOIN with COALESCE for missing values
SELECT
    Account,
    Instrument,
    BookedQuantity,
    COALESCE(VoidQuantity, 0) as VoidQuantity
FROM booked_summary
LEFT JOIN void_summary USING (Instrument)  -- Single column join
```

### For Self-Joins with Inequality

When you need `!=` in a self-join:

```sql
-- WORKS BUT LIMITED:
WITH summary AS (...)
SELECT * FROM summary s1
INNER JOIN summary s2 ON s1.Instrument = s2.Instrument
WHERE s1.Status = 'Booked' AND Status_right = 'Void'  -- Use _right suffix

-- Note: Can't use s2.Status, must use Status_right
```

## Implementation Details

The limitations stem from:
1. **Column Resolution Order**: Join conditions are evaluated before columns are renamed with `_right`
2. **Qualified Name Handling**: The `_right` suffixed columns don't get proper qualified names with table aliases
3. **Parser Restrictions**: JOIN ON clause only parses single comparison operations

## Future Improvements Needed

1. Support composite join conditions (AND, OR)
2. Properly apply table aliases to renamed columns
3. Better handling of qualified column names in self-joins
4. Consider using AS aliases for columns in joins to avoid conflicts