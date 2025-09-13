# Recursive CTE Implementation Plan

## Overview
This document outlines the plan for implementing recursive Common Table Expressions (CTEs) in SQL CLI, starting with simple mathematical series generation and expanding to more complex use cases.

## Phase 1: Basic Recursive CTEs (Initial Target)

### Goal
Implement basic recursive CTE support with simple numeric sequences and clear termination conditions.

### Target Examples

#### 1. Simple Number Sequence
```sql
WITH RECURSIVE numbers(n) AS (
    SELECT 1 AS n                    -- Anchor: starting value
    UNION ALL
    SELECT n + 1 FROM numbers        -- Recursive: increment
    WHERE n < 100                    -- Termination: stop at 100
)
SELECT * FROM numbers;
```

#### 2. Fibonacci Sequence
```sql
WITH RECURSIVE fib(n, curr, next) AS (
    SELECT 1, 0, 1                   -- Anchor: F(0)=0, F(1)=1
    UNION ALL
    SELECT n + 1, next, curr + next  -- Recursive: next Fibonacci
    FROM fib
    WHERE n < 20                     -- Termination: first 20 numbers
)
SELECT n, curr AS fibonacci FROM fib;
```

#### 3. Powers of 2
```sql
WITH RECURSIVE powers(n, value) AS (
    SELECT 0, 1                       -- Anchor: 2^0 = 1
    UNION ALL
    SELECT n + 1, value * 2          -- Recursive: double
    FROM powers
    WHERE n < 10                     -- Termination: 2^10
)
SELECT n AS exponent, value AS result FROM powers;
```

#### 4. Factorial Calculation
```sql
WITH RECURSIVE factorial(n, fact) AS (
    SELECT 1, 1                       -- Anchor: 1! = 1
    UNION ALL
    SELECT n + 1, fact * (n + 1)     -- Recursive: n! = n * (n-1)!
    FROM factorial
    WHERE n < 10                     -- Termination: up to 10!
)
SELECT n, fact FROM factorial;
```

## Implementation Requirements

### 1. Parser Modifications

```rust
// Add to ast.rs
pub struct CTE {
    pub name: String,
    pub column_list: Option<Vec<String>>,
    pub query: SelectStatement,
    pub is_recursive: bool,  // NEW FIELD
}

// Parse RECURSIVE keyword after WITH
// WITH RECURSIVE name AS (...)
```

### 2. Execution Engine Changes

```rust
// Core recursive execution logic
fn execute_recursive_cte(
    &self,
    cte: &CTE,
    cte_context: &mut HashMap<String, Arc<DataView>>
) -> Result<DataTable> {
    // 1. Split query into anchor and recursive parts (separated by UNION ALL)
    let (anchor_query, recursive_query) = split_union_all(cte.query)?;

    // 2. Execute anchor query to get initial result set
    let mut result_table = execute_query(anchor_query)?;
    let mut working_table = result_table.clone();

    // 3. Iteratively execute recursive part
    let mut iteration = 0;
    const MAX_ITERATIONS: usize = 1000;  // Safety limit

    loop {
        iteration += 1;

        // Safety check
        if iteration > MAX_ITERATIONS {
            return Err("Recursive CTE exceeded maximum iterations");
        }

        // Add working table to context for self-reference
        cte_context.insert(cte.name.clone(), Arc::new(working_table));

        // Execute recursive part with current working set
        let new_rows = execute_query_with_context(recursive_query, cte_context)?;

        // Check termination
        if new_rows.is_empty() {
            break;
        }

        // Append new rows to result (UNION ALL behavior)
        result_table.append(new_rows.clone())?;

        // Update working table for next iteration
        working_table = new_rows;
    }

    Ok(result_table)
}
```

### 3. Key Components to Build

#### A. UNION ALL Parser
- Split CTE query into anchor and recursive parts
- Validate structure (must be UNION ALL for recursive)

#### B. Self-Reference Resolution
- Allow CTE to reference itself in recursive part
- Maintain working table in context during iteration

#### C. Termination Logic
- WHERE clause in recursive part determines termination
- Empty result set ends recursion

#### D. Safety Mechanisms
- Maximum iteration limit (configurable)
- Maximum row count limit
- Memory usage monitoring

## Testing Strategy

### Phase 1 Tests (Mathematical Series)
1. Simple incrementing sequence (1 to N)
2. Fibonacci sequence
3. Powers of 2
4. Factorial calculation
5. Triangular numbers
6. Error cases:
   - Missing termination condition
   - Exceeding max iterations
   - Invalid recursive reference

### Success Criteria
- All mathematical series examples work correctly
- Clear error messages for invalid recursive CTEs
- Performance acceptable for sequences up to 1000 rows
- Execution plan shows iteration count and timing

## Phase 2: Future Enhancements (After Phase 1)

### Date/Time Series Generation
```sql
WITH RECURSIVE dates(date) AS (
    SELECT DATE('2024-01-01')
    UNION ALL
    SELECT DATE_ADD(date, INTERVAL 1 DAY)
    FROM dates
    WHERE date < '2024-12-31'
)
SELECT * FROM dates;
```

### Hierarchical Data (Future)
```sql
-- Organizational chart traversal
WITH RECURSIVE org_chart AS (
    SELECT id, name, manager_id, 0 as level
    FROM employees
    WHERE manager_id IS NULL
    UNION ALL
    SELECT e.id, e.name, e.manager_id, oc.level + 1
    FROM employees e
    JOIN org_chart oc ON e.manager_id = oc.id
)
SELECT * FROM org_chart;
```

### Graph Algorithms (Future)
- Shortest path
- Transitive closure
- Connected components

## Implementation Timeline

### Day 1 Goals
1. ✅ Add RECURSIVE keyword parsing
2. ✅ Implement UNION ALL splitting
3. ✅ Basic recursive execution loop
4. ✅ Test with simple number sequence

### Day 2 Goals
1. ✅ Self-reference resolution in context
2. ✅ Termination condition evaluation
3. ✅ Safety limits (iteration count)
4. ✅ Test all mathematical examples

### Success Metrics
- Simple recursive CTEs working end-to-end
- Clear foundation for more complex features
- Performance baseline established
- Clean, maintainable code structure

## Notes

### Why Start with Mathematical Series?
1. **Simple semantics** - Clear anchor and recursive relationship
2. **Easy testing** - Results are predictable and verifiable
3. **No external dependencies** - Pure computation, no date/string functions needed
4. **Performance friendly** - Small result sets, fast iteration
5. **Clear termination** - Simple numeric comparisons

### Challenges to Address
1. **Parser complexity** - Need to handle UNION ALL specially in recursive context
2. **Context management** - CTE must see its own previous iteration
3. **Memory efficiency** - Avoid keeping all iterations in memory if not needed
4. **Error handling** - Clear messages for common mistakes

### Design Decisions
1. Start with UNION ALL only (not UNION - deduplication adds complexity)
2. Require explicit termination condition in WHERE clause
3. Use iteration limit as safety net (default 1000, configurable)
4. Focus on correctness over optimization initially

## References
- PostgreSQL Recursive CTE documentation
- SQL Server CTE recursion examples
- SQLite WITH RECURSIVE implementation