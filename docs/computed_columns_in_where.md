# WHERE Clause with Computed Columns - Design Notes

## ✅ UPDATE: Arithmetic Expressions Now Supported!
As of v1.30.0, arithmetic expressions ARE supported in WHERE clauses:
```sql
-- This now works!
SELECT id, quantity * price as notional 
FROM test_arithmetic 
WHERE quantity * price > 1000  -- ✅ Works!

-- Complex expressions also work
WHERE (quantity * price) - discount > 500
WHERE price / quantity > 10
```

## Current Limitation: Aliases Not Yet Supported
Computed column **aliases** still cannot be used in WHERE clauses:
```sql
SELECT id, quantity * price as notional 
FROM test_arithmetic 
WHERE notional > 1000  -- ❌ Still doesn't work (alias reference)
WHERE quantity * price > 1000  -- ✅ Works! (expression directly)
```

## How It Works
The query execution order remains optimal:
1. **Filter (WHERE)** - Evaluates expressions directly on source table rows
2. **Project (SELECT)** - Compute expressions for remaining rows only
3. **Sort (ORDER BY)** - Sort the result set (can use aliases)

This is efficient because we avoid computing SELECT expressions for filtered-out rows.

## Potential Solutions

### Option 1: Two-Pass Evaluation (Recommended)
1. First pass: Identify computed columns referenced in WHERE
2. Compute ONLY those columns for all rows
3. Apply WHERE filter using computed values
4. Project final SELECT columns (may reuse computed values)

**Pros:**
- Efficient - only computes what's needed for filtering
- Maintains lazy evaluation where possible

**Cons:**
- Requires parsing WHERE to find column references
- More complex execution logic

### Option 2: Always Compute First
1. Compute ALL expressions in SELECT for all rows
2. Apply WHERE filter 
3. Return filtered computed results

**Pros:**
- Simpler implementation
- Computed values available everywhere

**Cons:**
- Inefficient for large datasets (computes before filtering)
- Wasteful if WHERE eliminates many rows

### Option 3: Subquery Rewriting
Internally rewrite the query as:
```sql
SELECT * FROM (
  SELECT id, quantity * price as notional 
  FROM test_arithmetic
) WHERE notional > 1000
```

**Pros:**
- Clean separation of concerns
- Follows SQL standard subquery semantics

**Cons:**
- Requires subquery support
- Still needs to compute before filtering

## Workaround for Users
Users can achieve the same result using the expression directly in WHERE:
```sql
SELECT id, quantity * price as notional 
FROM test_arithmetic 
WHERE quantity * price > 1000  -- Use expression instead of alias
```

## Implementation Priority
This is a lower priority enhancement because:
1. There's a working workaround (repeat expression in WHERE)
2. ORDER BY with computed columns works (more common use case)
3. Would require significant refactoring of query execution pipeline

## Future Implementation Notes
If implementing Option 1:
1. Add `WhereColumnAnalyzer` to extract referenced columns
2. Add `required_computations` field to track what needs early computation
3. Modify execution order to compute required expressions before filtering
4. Cache computed values to avoid recomputation in SELECT