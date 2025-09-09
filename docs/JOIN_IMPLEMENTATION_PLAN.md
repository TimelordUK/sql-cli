# JOIN Implementation Plan for SQL CLI

## Executive Summary

This document outlines the plan for implementing JOIN operations within Common Table Expressions (CTEs) in the SQL CLI query engine. The implementation is highly feasible due to existing CTE infrastructure, flexible data model, and extensible parser architecture.

## Current Architecture Analysis

### Existing Strengths

#### 1. CTE Infrastructure
- CTEs are fully parsed and stored in `SelectStatement.ctes`
- CTE execution uses HashMap context for efficient materialization
- CTEs can already reference previously defined CTEs
- Architecture supports nested data sources

#### 2. Data Model
- **DataTable**: Immutable data storage with columns and rows
- **DataView**: Lightweight view with filtering/projection (no data copying)
- **Arc-based sharing**: Prevents unnecessary cloning
- **Lazy materialization**: Only materializes when necessary

#### 3. Expression Framework
- Robust expression evaluator for complex conditions
- Supports column references across tables
- Case-insensitive comparison support
- Type coercion capabilities

## Implementation Strategy

### Phase 1: Basic INNER JOIN (Weeks 1-2)

#### Target Syntax
```sql
WITH orders_with_customers AS (
    SELECT o.*, c.customer_name 
    FROM orders o
    INNER JOIN customers c ON o.customer_id = c.id
)
SELECT * FROM orders_with_customers;
```

#### Technical Implementation

##### 1.1 Parser Extensions (`src/sql/recursive_parser.rs`)

```rust
// New tokens to add
enum Token {
    // ... existing tokens
    Join,
    Inner,
    Left, 
    Right,
    On,
}

// Extend SelectStatement
struct SelectStatement {
    // ... existing fields
    pub join_clauses: Vec<JoinClause>,
}

struct JoinClause {
    pub join_type: JoinType,
    pub right_table: TableSource,
    pub on_condition: SqlExpression,
    pub right_alias: Option<String>,
}

enum JoinType {
    Inner,
    Left,
    Right,
}
```

##### 1.2 Hash Join Algorithm

```rust
// Efficient hash join without data cloning
fn hash_join(
    left: Arc<DataTable>, 
    right: Arc<DataTable>,
    left_col: &str, 
    right_col: &str,
    join_type: JoinType
) -> Result<DataTable> {
    // Step 1: Build hash index on smaller table
    let (build_table, probe_table) = if left.row_count() < right.row_count() {
        (left, right)
    } else {
        (right, left)
    };
    
    // Step 2: Create hash index
    let mut hash_index: HashMap<DataValue, Vec<usize>> = HashMap::new();
    for (idx, row) in build_table.rows.iter().enumerate() {
        let key = row.get_value(build_col_idx);
        hash_index.entry(key).or_default().push(idx);
    }
    
    // Step 3: Probe and build results
    let mut result = DataTable::new("joined");
    for probe_row in probe_table.rows {
        if let Some(matching_indices) = hash_index.get(&probe_key) {
            for &build_idx in matching_indices {
                emit_joined_row(&mut result, probe_row, &build_table.rows[build_idx]);
            }
        } else if join_type == JoinType::Left {
            emit_row_with_nulls(&mut result, probe_row, build_table.column_count());
        }
    }
    
    Ok(result)
}
```

##### 1.3 Query Engine Updates (`src/data/query_engine.rs`)

```rust
impl QueryEngine {
    fn build_view_with_joins(
        &self,
        base_table: Arc<DataTable>,
        statement: &SelectStatement,
        cte_context: &HashMap<String, Arc<DataView>>
    ) -> Result<DataView> {
        let mut current_table = base_table;
        
        for join_clause in &statement.join_clauses {
            let right_table = self.resolve_table_source(
                &join_clause.right_table, 
                cte_context
            )?;
            
            current_table = Arc::new(self.execute_join(
                current_table,
                right_table,
                join_clause
            )?);
        }
        
        Ok(DataView::new(current_table))
    }
}
```

### Phase 2: LEFT/RIGHT OUTER JOINs (Week 3)

#### Target Syntax
```sql
WITH all_customers_orders AS (
    SELECT c.*, o.order_id, o.order_date
    FROM customers c
    LEFT JOIN orders o ON c.id = o.customer_id
)
SELECT * FROM all_customers_orders WHERE order_id IS NULL;
```

#### Implementation Details
- Extend hash join to preserve unmatched rows
- Add NULL values for missing columns
- Handle IS NULL/IS NOT NULL in WHERE clauses
- Minimal changes to Phase 1 infrastructure

### Phase 3: Multi-Column & Complex JOINs (Week 4)

#### Target Syntax
```sql
WITH complex_join AS (
    SELECT *
    FROM sales s
    INNER JOIN regions r 
        ON s.region_id = r.id 
        AND s.sale_date >= r.effective_date
        AND s.sale_date < r.expiry_date
)
SELECT * FROM complex_join;
```

#### Implementation Approach
1. **Composite Keys**: Hash on concatenated values
2. **Mixed Conditions**: Hash join for equality, filter for inequalities
3. **Optimization**: Push down non-equality filters when possible

### Phase 4: Optimization & Testing (Weeks 5-6)

#### Performance Optimizations

1. **Statistics-Based Planning**
   - Track table cardinality
   - Choose optimal join order
   - Select best join algorithm

2. **Memory Management**
   ```rust
   struct JoinedDataView {
       left: Arc<DataTable>,
       right: Arc<DataTable>,
       join_indices: Vec<(usize, usize)>, // (left_idx, right_idx)
   }
   ```

3. **Index Caching**
   - Cache hash indices for repeated joins
   - Reuse indices across CTEs

4. **Lazy Materialization**
   - Keep joins as views until necessary
   - Only materialize for aggregations or output

## Memory & Performance Analysis

### Space Complexity
| Join Type | Memory Usage | Notes |
|-----------|-------------|-------|
| Hash Join | O(min(n,m)) | Hash smaller table |
| Nested Loop | O(1) | No extra memory |
| Sort-Merge | O(n + m) | If not pre-sorted |

### Time Complexity
| Join Type | Average Case | Worst Case | Best For |
|-----------|-------------|------------|----------|
| Hash Join | O(n + m) | O(n × m) | Equality joins |
| Nested Loop | O(n × m) | O(n × m) | Small tables |
| Sort-Merge | O(n log n + m log m) | Same | Pre-sorted data |

## Implementation Challenges & Solutions

### Challenge 1: Column Name Conflicts
**Problem**: Both tables have column "id"  
**Solution**: 
- Require table aliases
- Qualify columns: `t1.id`, `t2.id`
- Error on ambiguous references

### Challenge 2: Self-Joins
**Problem**: Joining table to itself  
**Solution**:
```sql
WITH hierarchy AS (
    SELECT e1.*, e2.name as manager_name
    FROM employees e1
    LEFT JOIN employees e2 ON e1.manager_id = e2.id
)
```

### Challenge 3: Result Size Explosion
**Problem**: Cartesian products from many-to-many joins  
**Solution**:
- Implement result size estimation
- Add configurable limits
- Warning for large result sets

### Challenge 4: Complex Join Conditions
**Problem**: Joins with functions, expressions  
**Solution**:
- Start with simple equality
- Add comparison operators
- Eventually support expressions

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_inner_join_single_column() {
    let engine = QueryEngine::new();
    let result = engine.execute(
        table,
        "WITH joined AS (
            SELECT * FROM orders o 
            INNER JOIN customers c ON o.customer_id = c.id
        ) SELECT * FROM joined"
    );
    assert_eq!(result.unwrap().row_count(), expected_count);
}
```

### Integration Tests
1. **Correctness Tests**
   - Compare against expected results
   - Test all join types
   - Verify NULL handling

2. **Performance Tests**
   - Large table joins (100K+ rows)
   - Multiple joins in single CTE
   - Nested CTEs with joins

3. **Edge Cases**
   - Empty tables
   - No matching rows
   - All rows match
   - Duplicate join keys

### Example Test Cases

```sql
-- Test 1: Basic INNER JOIN
WITH test AS (
    SELECT * FROM orders o
    INNER JOIN customers c ON o.customer_id = c.id
)
SELECT COUNT(*) FROM test;

-- Test 2: LEFT JOIN with NULLs
WITH test AS (
    SELECT c.id, o.order_id
    FROM customers c
    LEFT JOIN orders o ON c.id = o.customer_id
)
SELECT * FROM test WHERE order_id IS NULL;

-- Test 3: Multiple CTEs with joins
WITH 
customer_orders AS (
    SELECT c.*, o.order_id
    FROM customers c
    INNER JOIN orders o ON c.id = o.customer_id
),
order_items AS (
    SELECT co.*, i.product_id, i.quantity
    FROM customer_orders co
    INNER JOIN items i ON co.order_id = i.order_id
)
SELECT * FROM order_items;
```

## Success Metrics

1. **Functional Success**
   - ✅ All join types implemented
   - ✅ Correct results for test suite
   - ✅ No memory leaks

2. **Performance Targets**
   - Join 10K × 10K rows in < 100ms
   - Join 100K × 100K rows in < 1 second
   - Memory usage < 2× input data size

3. **Code Quality**
   - 90%+ test coverage
   - No clippy warnings
   - Documented public APIs

## Implementation Timeline

| Week | Phase | Deliverables |
|------|-------|-------------|
| 1-2 | Parser & Basic INNER JOIN | Parse JOIN syntax, hash join algorithm |
| 3 | OUTER JOINs | LEFT/RIGHT JOIN support |
| 4 | Complex JOINs | Multi-column, inequality conditions |
| 5 | Optimization | Performance tuning, index caching |
| 6 | Testing & Documentation | Full test suite, performance benchmarks |

## Future Enhancements

1. **Advanced Join Types**
   - FULL OUTER JOIN
   - CROSS JOIN
   - NATURAL JOIN

2. **Join Optimizations**
   - Cost-based optimizer
   - Join reordering
   - Predicate pushdown

3. **Distributed Joins**
   - Partition-wise joins
   - Broadcast joins
   - Shuffle joins

## Conclusion

The JOIN implementation is highly feasible with our current architecture:

- ✅ **CTE infrastructure** provides the execution framework
- ✅ **DataView model** enables efficient joins without data copying
- ✅ **Expression evaluator** can handle join conditions
- ✅ **Extensible parser** supports new syntax easily
- ✅ **Phased approach** delivers value incrementally

The proposed implementation prioritizes memory efficiency through hash joins and lazy materialization while maintaining query performance suitable for interactive use.