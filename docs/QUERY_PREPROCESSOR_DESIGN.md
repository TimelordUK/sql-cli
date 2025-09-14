# Query Preprocessor Design

## Overview

A query preprocessing layer that transforms "natural" SQL queries into the execution-ready form that sql-cli requires. This preprocessor would run before the parser, rewriting queries to handle common patterns automatically while also identifying optimization opportunities.

## Goals

1. **Developer Experience**: Allow users to write queries in a natural SQL style
2. **Automatic CTE Lifting**: Transform expressions into CTEs where needed
3. **Parallel Execution Planning**: Identify parallelizable operations
4. **GPU/CUDA Offloading**: Mark operations suitable for GPU acceleration
5. **Query Optimization**: Apply rewrite rules for better performance

## Core Transformations

### 1. Expression Lifting to CTEs

#### WHERE Clause Expressions
**Input:**
```sql
SELECT * FROM products
WHERE CONTAINS(UPPER(name), 'PHONE')
```

**Output:**
```sql
WITH _expr_cte AS (
    SELECT *,
           CONTAINS(UPPER(name), 'PHONE') AS _expr_1
    FROM products
)
SELECT * FROM _expr_cte
WHERE _expr_1 = true
```

#### Window Functions in Expressions
**Input:**
```sql
SELECT 'Order #' || ROW_NUMBER() OVER (ORDER BY date) || ': ' || product
FROM sales
```

**Output:**
```sql
WITH _window_cte AS (
    SELECT *,
           ROW_NUMBER() OVER (ORDER BY date) AS _row_num
    FROM sales
)
SELECT 'Order #' || _row_num || ': ' || product
FROM _window_cte
```

### 2. Boolean Shorthand Normalization

**Input:**
```sql
WHERE is_active AND NOT is_deleted
```

**Output:**
```sql
WHERE is_active = true AND is_deleted = false
```

### 3. Complex Expression Pipeline

Transform nested/complex queries into a pipeline of CTEs:

**Input:**
```sql
SELECT product,
       AVG(price) AS avg_price,
       COUNT(*) AS total_sales
FROM sales
WHERE YEAR(date) = 2024
  AND CONTAINS(LOWER(product), 'phone')
GROUP BY product
HAVING AVG(price) > 100
ORDER BY total_sales DESC
```

**Output:**
```sql
WITH _filter_cte AS (
    SELECT *,
           YEAR(date) AS _year,
           CONTAINS(LOWER(product), 'phone') AS _has_phone
    FROM sales
),
_filtered AS (
    SELECT * FROM _filter_cte
    WHERE _year = 2024 AND _has_phone = true
),
_aggregated AS (
    SELECT product,
           AVG(price) AS avg_price,
           COUNT(*) AS total_sales
    FROM _filtered
    GROUP BY product
),
_having AS (
    SELECT *,
           avg_price > 100 AS _having_condition
    FROM _aggregated
)
SELECT product, avg_price, total_sales
FROM _having
WHERE _having_condition = true
ORDER BY total_sales DESC
```

## Parallel Execution Opportunities

### 1. Independent CTE Branches

Identify CTEs that can be executed in parallel:

```sql
WITH
  -- These can run in parallel
  web_data AS (
    SELECT * FROM WEB('https://api.example.com/data')
  ),
  local_data AS (
    SELECT * FROM products WHERE category = 'Electronics'
  ),
  -- This depends on both above
  combined AS (
    SELECT * FROM web_data
    UNION ALL
    SELECT * FROM local_data
  )
SELECT * FROM combined
```

**Execution Plan:**
```
┌─────────────┐     ┌──────────────┐
│  web_data   │     │  local_data  │
│  (async)    │     │  (parallel)  │
└─────┬───────┘     └──────┬───────┘
      └─────────┬──────────┘
            ┌───▼────┐
            │combined│
            └────────┘
```

### 2. Partition-Based Parallelism

For operations that can be partitioned:

```sql
-- Mark for parallel execution
SELECT /*+ PARALLEL(4) */
       product,
       SUM(amount) AS total
FROM large_sales_table
GROUP BY product
```

Transform to:
```sql
WITH
  _partition_1 AS (SELECT * FROM large_sales_table WHERE _row_hash % 4 = 0),
  _partition_2 AS (SELECT * FROM large_sales_table WHERE _row_hash % 4 = 1),
  _partition_3 AS (SELECT * FROM large_sales_table WHERE _row_hash % 4 = 2),
  _partition_4 AS (SELECT * FROM large_sales_table WHERE _row_hash % 4 = 3),
  -- Execute in parallel, then merge
  _merged AS (
    SELECT product, SUM(amount) AS total FROM _partition_1 GROUP BY product
    UNION ALL
    SELECT product, SUM(amount) AS total FROM _partition_2 GROUP BY product
    UNION ALL
    SELECT product, SUM(amount) AS total FROM _partition_3 GROUP BY product
    UNION ALL
    SELECT product, SUM(amount) AS total FROM _partition_4 GROUP BY product
  )
SELECT product, SUM(total) AS total
FROM _merged
GROUP BY product
```

## GPU/CUDA Acceleration Candidates

### 1. Vector Operations

Identify operations suitable for GPU:

```sql
-- Mathematical operations on large datasets
SELECT
    SQRT(x*x + y*y + z*z) AS distance,
    SIN(angle) * radius AS x_pos,
    COS(angle) * radius AS y_pos
FROM coordinates
```

Mark for GPU execution:
```sql
WITH _gpu_compute AS (
    SELECT /*+ GPU */
        _id,
        SQRT(x*x + y*y + z*z) AS distance,
        SIN(angle) * radius AS x_pos,
        COS(angle) * radius AS y_pos
    FROM coordinates
)
SELECT * FROM _gpu_compute
```

### 2. Aggregations

Large aggregations that benefit from GPU:
- SUM, AVG, MIN, MAX over large datasets
- GROUP BY with high cardinality
- Window functions with large frames

## Implementation Architecture

```
┌─────────────────────────────────────────────┐
│            User SQL Query                   │
└────────────────┬────────────────────────────┘
                 ▼
┌─────────────────────────────────────────────┐
│         Query Preprocessor                  │
├─────────────────────────────────────────────┤
│ 1. Parse into preliminary AST               │
│ 2. Apply transformation rules               │
│ 3. Identify parallelization opportunities   │
│ 4. Mark GPU-eligible operations            │
│ 5. Generate execution plan                  │
└────────────────┬────────────────────────────┘
                 ▼
┌─────────────────────────────────────────────┐
│      Transformed Query + Metadata           │
│   - Rewritten SQL                          │
│   - Execution hints                        │
│   - Dependency graph                       │
└────────────────┬────────────────────────────┘
                 ▼
┌─────────────────────────────────────────────┐
│         Existing SQL Parser                 │
└────────────────┬────────────────────────────┘
                 ▼
┌─────────────────────────────────────────────┐
│      Parallel Execution Engine              │
├─────────────────────────────────────────────┤
│ - Thread pool for parallel CTEs            │
│ - GPU kernel dispatch                      │
│ - Result merging                           │
└─────────────────────────────────────────────┘
```

## Transformation Rules Engine

### Rule Definition Format

```rust
struct RewriteRule {
    name: String,
    pattern: Pattern,
    condition: Box<dyn Fn(&ASTNode) -> bool>,
    transform: Box<dyn Fn(ASTNode) -> ASTNode>,
    priority: i32,
}

enum Pattern {
    WhereExpression { contains_function: bool },
    WindowInExpression,
    BooleanColumn,
    AggregateInHaving,
    SubqueryInFrom,
    // ... more patterns
}
```

### Example Rules

```rust
// Rule: Lift WHERE expressions to CTE
RewriteRule {
    name: "lift_where_expressions",
    pattern: Pattern::WhereExpression { contains_function: true },
    condition: |node| {
        // Check if WHERE contains function calls
        node.where_clause.has_function_calls()
    },
    transform: |node| {
        // Create CTE with computed expressions
        let cte = create_expression_cte(node.where_clause);
        node.with_cte(cte).simplify_where()
    },
    priority: 100,
}

// Rule: Parallelize independent CTEs
RewriteRule {
    name: "parallelize_ctes",
    pattern: Pattern::MultipleCTEs,
    condition: |node| {
        // Check if CTEs are independent
        !has_dependencies(node.ctes)
    },
    transform: |node| {
        node.mark_parallel_execution()
    },
    priority: 50,
}
```

## Configuration

```toml
[preprocessor]
enabled = true

[preprocessor.transforms]
lift_expressions = true
normalize_booleans = true
parallelize_ctes = true
optimize_joins = true

[preprocessor.parallel]
max_threads = 8
min_rows_for_parallel = 10000

[preprocessor.gpu]
enabled = false  # Experimental
min_rows_for_gpu = 100000
preferred_operations = ["sum", "avg", "sqrt", "sin", "cos"]
```

## Benefits

1. **User-Friendly**: Write natural SQL without worrying about sql-cli limitations
2. **Performance**: Automatic parallelization and optimization
3. **Maintainability**: Centralized place for query optimizations
4. **Extensibility**: Easy to add new transformation rules
5. **Future-Proof**: Foundation for GPU acceleration and distributed execution

## Implementation Phases

### Phase 1: Core Expression Lifting
- WHERE expression lifting
- Window function extraction
- Boolean normalization

### Phase 2: CTE Pipeline Generation
- Complex query decomposition
- HAVING clause handling
- Subquery flattening

### Phase 3: Parallel Execution
- Independent CTE detection
- Parallel execution planning
- Web CTE async fetching

### Phase 4: Advanced Optimizations
- Join reordering
- Predicate pushdown
- Common subexpression elimination

### Phase 5: GPU Acceleration (Future)
- GPU kernel generation
- Memory management
- Result materialization

## Example: Complete Transformation

**Original Query:**
```sql
SELECT
    category,
    product,
    'Rank: ' || ROW_NUMBER() OVER (PARTITION BY category ORDER BY sales DESC) AS rank,
    sales,
    sales / SUM(sales) OVER (PARTITION BY category) * 100 AS pct_of_category
FROM products
WHERE YEAR(date) = 2024
  AND sales > AVG(sales) OVER ()
ORDER BY category, sales DESC
```

**After Preprocessing:**
```sql
-- Step 1: Compute window functions
WITH _windows AS (
    SELECT *,
           ROW_NUMBER() OVER (PARTITION BY category ORDER BY sales DESC) AS _rank,
           SUM(sales) OVER (PARTITION BY category) AS _category_total,
           AVG(sales) OVER () AS _overall_avg,
           YEAR(date) AS _year
    FROM products
),
-- Step 2: Apply filters
_filtered AS (
    SELECT *
    FROM _windows
    WHERE _year = 2024 AND sales > _overall_avg
),
-- Step 3: Compute derived columns
_final AS (
    SELECT
        category,
        product,
        'Rank: ' || _rank AS rank,
        sales,
        sales / _category_total * 100 AS pct_of_category
    FROM _filtered
)
SELECT * FROM _final
ORDER BY category, sales DESC
```

## Testing Strategy

1. **Transformation Tests**: Verify each rule produces correct output
2. **Equivalence Tests**: Ensure transformed queries produce same results
3. **Performance Tests**: Measure improvement from parallelization
4. **Regression Tests**: Large suite of before/after query pairs

## Future Enhancements

1. **Query Plan Caching**: Cache preprocessed queries for repeated execution
2. **Cost-Based Optimization**: Choose transformations based on statistics
3. **Distributed Execution**: Extend to multi-node execution
4. **ML-Based Optimization**: Learn optimal transformations from query history
5. **Incremental Processing**: For streaming/real-time queries