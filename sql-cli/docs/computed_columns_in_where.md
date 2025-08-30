# WHERE Clause with Computed Columns - Design Notes

## Current Limitation
Currently, computed columns cannot be used in WHERE clauses. For example:
```sql
SELECT id, quantity * price as notional 
FROM test_arithmetic 
WHERE notional > 1000  -- This will fail
```

## Why It Doesn't Work
The current query execution order is:
1. **Filter (WHERE)** - Applied directly to source table
2. **Project (SELECT)** - Compute expressions and select columns  
3. **Sort (ORDER BY)** - Sort the result set

Since computed columns don't exist until step 2, they can't be referenced in step 1.

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