# CTE (Common Table Expression) Support

## Current Status

✅ **UPDATE**: CTEs are now fully accessible within subqueries as of v1.44.0. The scoping issue has been resolved by processing CTEs before subqueries and passing the CTE context to the subquery executor.

## Working Examples

### Basic CTE Usage ✅
```sql
WITH discoveries AS (
    SELECT Year, COUNT(*) AS number_discoveries
    FROM periodic_table
    WHERE Year > 0
    GROUP BY Year
)
SELECT MAX(number_discoveries) AS most_discoveries
FROM discoveries
-- Returns: 5
```

### Multiple CTEs ✅
```sql
WITH 
    discoveries AS (
        SELECT Year, COUNT(*) AS number_discoveries
        FROM periodic_table
        WHERE Year > 0
        GROUP BY Year
    ),
    elements AS (
        SELECT * FROM periodic_table
    )
SELECT * FROM elements WHERE Year = 1898
-- Works correctly
```

### CTE with WHERE clause ✅
```sql
WITH discoveries AS (
    SELECT Year, COUNT(*) AS number_discoveries
    FROM periodic_table
    WHERE Year > 0
    GROUP BY Year
)
SELECT Year FROM discoveries 
WHERE number_discoveries = 5
-- Returns: 1898
```

## Previously Fixed Issues

### CTEs in Subqueries ✅ (Fixed in v1.44.0)
CTEs are now fully accessible within subqueries. The issue was resolved by restructuring the query execution pipeline to process CTEs before subqueries and passing the CTE context through.

```sql
WITH discoveries AS (
    SELECT Year, COUNT(*) AS number_discoveries
    FROM periodic_table
    WHERE Year > 0
    GROUP BY Year
)
SELECT (
    SELECT MAX(number_discoveries) 
    FROM discoveries  -- ✅ Now works correctly!
) as max_disc
-- Returns: 5
```

### More Complex CTE Subquery Examples ✅

Nested subqueries with CTEs also work:
```sql
WITH discoveries AS (
    SELECT Year, COUNT(*) AS number_discoveries
    FROM periodic_table
    WHERE Year > 0
    GROUP BY Year
)
SELECT * FROM periodic_table 
WHERE Year = (
    SELECT Year FROM discoveries 
    WHERE number_discoveries = (
        SELECT MAX(number_discoveries) FROM discoveries
    )
)
-- Returns: All elements discovered in 1898 (the year with most discoveries)
```

## Technical Implementation

### How the Fix Works
The fix involved restructuring the query execution pipeline:

1. **New execution order**:
   1. Parse SQL
   2. Process CTEs first (build context)
   3. Execute subqueries (with CTE context available)
   4. Execute main query

2. **Key changes made**:
   - `SubqueryExecutor` now accepts CTE context via `with_cte_context()` constructor
   - `QueryEngine::execute_statement()` processes CTEs before instantiating SubqueryExecutor
   - Added `execute_statement_with_cte_context()` for nested CTE resolution
   - CTE context is passed through the entire query execution tree

3. **Files modified**:
   - `src/data/query_engine.rs`: Restructured execution flow
   - `src/data/subquery_executor.rs`: Added CTE context support

## Related Issues

- Correlated subqueries are not yet supported
- Recursive CTEs are not implemented
- CTE column aliases might not work in all cases