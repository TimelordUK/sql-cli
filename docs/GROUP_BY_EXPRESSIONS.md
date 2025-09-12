# GROUP BY Expression Support Analysis

## Current Status
GROUP BY currently only supports column names, not expressions.

### ❌ What Doesn't Work
```sql
-- Direct expression in GROUP BY
SELECT MOD(value, 5) AS remainder, COUNT(*) 
FROM numbers 
GROUP BY MOD(value, 5)  -- Error: Column 'MOD' not found
```

### ✅ Workaround Pattern
Pre-compute the expression in a CTE or subquery:

```sql
-- Using CTE to pre-compute expression
WITH numbers AS (
    SELECT value, MOD(value, 5) AS remainder 
    FROM RANGE(1, 20)
)
SELECT remainder, COUNT(*) AS count 
FROM numbers 
GROUP BY remainder
```

This returns:
```
remainder | count
----------|------
0         | 4
1         | 4
2         | 4
3         | 4
4         | 4
```

## Implementation Requirements

To support expressions in GROUP BY, we need to:

### 1. Update AST Definition
Change in `recursive_parser.rs`:
```rust
// Current
pub group_by: Option<Vec<String>>,

// Needed
pub group_by: Option<Vec<SqlExpression>>,
```

### 2. Update Parser
Change `parse_group_by` to parse expressions instead of just identifiers.

### 3. Update Query Executor
- Modify `apply_group_by` to evaluate expressions for each row
- Create temporary columns for expression results
- Group by these computed values

### 4. Handle Complex Cases
- Alias resolution (GROUP BY 1, GROUP BY alias)
- Expression equivalence (same expression in SELECT and GROUP BY)
- Performance considerations for complex expressions

## Examples of Desired Behavior

```sql
-- Group by expression
SELECT value % 3, COUNT(*) 
FROM data 
GROUP BY value % 3

-- Group by function result
SELECT UPPER(name), COUNT(*) 
FROM users 
GROUP BY UPPER(name)

-- Group by complex expression
SELECT YEAR(date), MONTH(date), SUM(amount)
FROM sales
GROUP BY YEAR(date), MONTH(date)
```

## Recommendation

For now, use the CTE workaround pattern. Full expression support in GROUP BY requires significant changes to the parser and executor. The workaround is:
1. More explicit about what's being computed
2. Allows reuse of the computed value in SELECT
3. Works with current implementation

## Test Coverage Needed

When implementing expression support:
- Basic arithmetic expressions (col + 1, col * 2, col % n)
- Function calls (UPPER(), LENGTH(), MOD())
- Nested expressions
- Mixed expressions and column names
- Performance tests with large datasets