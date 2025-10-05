# UNNEST Goal - FIX Allocation Example

## Input Data (fix_allocations.csv)
```csv
msg_type,order_id,symbol,accounts,amounts
AS,ORD001,ZX5Y,ACC_1|ACC_2|ACC_3,"200,200,200"
AS,ORD002,ABCD,ACC_4|ACC_5,"300,700"
8,ORD003,WXYZ,ACC_1,"1000"
```

## Desired Query
```sql
SELECT
  msg_type,
  order_id,
  symbol,
  UNNEST(accounts, '|') AS account,
  UNNEST(amounts, ',') AS amount
FROM fix_allocations;
```

## Expected Output
```
+----------+----------+--------+---------+--------+
| msg_type | order_id | symbol | account | amount |
+----------+----------+--------+---------+--------+
| AS       | ORD001   | ZX5Y   | ACC_1   | 200    |
| AS       | ORD001   | ZX5Y   | ACC_2   | 200    |
| AS       | ORD001   | ZX5Y   | ACC_3   | 200    |
| AS       | ORD002   | ABCD   | ACC_4   | 300    |
| AS       | ORD002   | ABCD   | ACC_5   | 700    |
| 8        | ORD003   | WXYZ   | ACC_1   | 1000   |
+----------+----------+--------+---------+--------+
```

## How It Works

1. **Row ORD001**: `accounts` has 3 items, `amounts` has 3 items
   - Creates 3 output rows (max of both)
   - Each regular column (msg_type, order_id, symbol) is replicated 3 times
   - UNNEST columns get their respective split values

2. **Row ORD002**: `accounts` has 2 items, `amounts` has 2 items
   - Creates 2 output rows
   - Values aligned by index

3. **Row ORD003**: `accounts` has 1 item, `amounts` has 1 item
   - Creates 1 output row (no expansion needed)

## Mismatched Length Example (NULL Padding)

If we had mismatched data:
```csv
AS,ORD004,TEST,ACC_1|ACC_2|ACC_3,"100,200"
```

Query result would be:
```
| AS | ORD004 | TEST | ACC_1 | 100  |
| AS | ORD004 | TEST | ACC_2 | 200  |
| AS | ORD004 | TEST | ACC_3 | NULL |  <- NULL padding
```

## Implementation Requirements

1. **Parser**: Recognize `UNNEST(column_expr, 'delimiter')`
2. **Evaluator**: Return array of split values (not executed row-by-row)
3. **Query Executor**:
   - Detect all UNNEST expressions in SELECT
   - For each input row:
     - Evaluate each UNNEST → get arrays
     - Find max array length
     - Generate N output rows
     - Fill in values (NULL if array exhausted)
