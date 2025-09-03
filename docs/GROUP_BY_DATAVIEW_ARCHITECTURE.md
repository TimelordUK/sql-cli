# GROUP BY Architecture Using DataView Partitions

## Core Concept
Leverage DataView's efficient reference-based architecture to create partition views for GROUP BY operations. Since DataView doesn't clone data, we can create multiple views representing different partitions at minimal memory cost.

## Architecture Overview

```
DataTable (Immutable)
    ↓
Primary DataView (after WHERE filter)
    ↓
GroupByPartitioner
    ↓
HashMap<GroupKey, DataView> (partitions)
    ↓
AggregateEvaluator (operates on each partition)
    ↓
Result DataTable (aggregated results)
```

## Key Design Principles

1. **Immutable DataTable**: Never modify the original data
2. **View-based Partitioning**: Each group is a DataView with specific row indices
3. **Recursive Application**: Can apply GROUP BY to already grouped results
4. **Memory Efficient**: Views only store row indices, not data copies
5. **Composable**: Aggregate functions work on DataView, not DataTable

## Implementation Plan

### Phase 1: Aggregate Functions Infrastructure

```rust
// src/sql/aggregates/mod.rs
pub trait AggregateFunction: Send + Sync {
    fn name(&self) -> &str;
    fn init(&self) -> AggregateState;
    fn accumulate(&self, state: &mut AggregateState, value: &DataValue);
    fn finalize(&self, state: AggregateState) -> DataValue;
}

pub enum AggregateState {
    Count(i64),
    Sum(SumState),
    Avg(AvgState),
    MinMax(MinMaxState),
}

pub struct SumState {
    int_sum: Option<i64>,
    float_sum: Option<f64>,
}

pub struct AvgState {
    sum: SumState,
    count: i64,
}
```

### Phase 2: GroupBy Partitioner

```rust
// src/data/group_by_partitioner.rs
use std::collections::HashMap;
use std::sync::Arc;

pub struct GroupByPartitioner {
    source_view: DataView,
    group_columns: Vec<String>,
}

impl GroupByPartitioner {
    pub fn new(source_view: DataView, group_columns: Vec<String>) -> Self {
        Self { source_view, group_columns }
    }
    
    /// Create partitioned views based on group columns
    pub fn partition(&self) -> Result<HashMap<GroupKey, DataView>> {
        let mut partitions: HashMap<GroupKey, Vec<usize>> = HashMap::new();
        
        // Iterate through visible rows in the source view
        for row_idx in self.source_view.get_visible_rows() {
            let group_key = self.extract_group_key(row_idx)?;
            partitions.entry(group_key)
                .or_insert_with(Vec::new)
                .push(row_idx);
        }
        
        // Convert row indices to DataViews
        let mut result = HashMap::new();
        let table = self.source_view.get_table();
        
        for (key, indices) in partitions {
            // Create a new DataView with specific row indices
            let mut partition_view = DataView::new(Arc::clone(&table));
            partition_view.set_visible_rows(indices);
            result.insert(key, partition_view);
        }
        
        Ok(result)
    }
    
    fn extract_group_key(&self, row_idx: usize) -> Result<GroupKey> {
        let mut values = Vec::new();
        for col_name in &self.group_columns {
            let value = self.source_view.get_value(row_idx, col_name)?;
            values.push(value.clone());
        }
        Ok(GroupKey(values))
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct GroupKey(Vec<DataValue>);
```

### Phase 3: Aggregate Evaluator

```rust
// src/data/aggregate_evaluator.rs
pub struct AggregateEvaluator {
    aggregates: Vec<(String, Box<dyn AggregateFunction>, SqlExpression)>,
}

impl AggregateEvaluator {
    /// Evaluate aggregates on a single partition
    pub fn evaluate_partition(&self, partition: &DataView) -> Result<Vec<DataValue>> {
        let mut results = Vec::new();
        
        for (alias, func, expr) in &self.aggregates {
            let mut state = func.init();
            
            // For each row in the partition
            for row_idx in partition.get_visible_rows() {
                let value = self.evaluate_expression(expr, partition, row_idx)?;
                func.accumulate(&mut state, &value);
            }
            
            let result = func.finalize(state);
            results.push(result);
        }
        
        Ok(results)
    }
}
```

### Phase 4: Query Engine Integration

```rust
// In QueryEngine::execute_select()
pub fn execute_select_with_group_by(
    &self,
    view: DataView,
    select_stmt: &SelectStatement,
) -> Result<DataView> {
    // 1. Apply WHERE filter (already done)
    let filtered_view = self.apply_where(view, &select_stmt.where_clause)?;
    
    // 2. Check if we have GROUP BY
    if let Some(ref group_by_cols) = select_stmt.group_by {
        // 3. Create partitions
        let partitioner = GroupByPartitioner::new(filtered_view, group_by_cols.clone());
        let partitions = partitioner.partition()?;
        
        // 4. Create result table
        let mut result_table = DataTable::new("grouped_result");
        
        // Add columns for group keys and aggregates
        for col in group_by_cols {
            result_table.add_column(col.clone());
        }
        for item in &select_stmt.select_items {
            if let SelectItem::Expression { expr, alias } = item {
                if is_aggregate_function(expr) {
                    result_table.add_column(alias.clone());
                }
            }
        }
        
        // 5. Process each partition
        let evaluator = AggregateEvaluator::new(&select_stmt.select_items);
        
        for (group_key, partition) in partitions {
            let mut row = Vec::new();
            
            // Add group key values
            row.extend(group_key.0);
            
            // Evaluate aggregates for this partition
            let aggregate_results = evaluator.evaluate_partition(&partition)?;
            row.extend(aggregate_results);
            
            result_table.add_row(row);
        }
        
        // 6. Return view of aggregated results
        Ok(DataView::new(Arc::new(result_table)))
    } else {
        // No GROUP BY - handle simple aggregates or regular SELECT
        Ok(filtered_view)
    }
}
```

## Example Usage

```sql
-- Query with GROUP BY
SELECT 
    product_type,
    COUNT(*) as total_count,
    SUM(quantity) as total_quantity,
    AVG(price) as avg_price,
    MAX(price) as max_price
FROM sales
WHERE date > '2024-01-01'
GROUP BY product_type

-- This would create:
-- 1. Primary view: filtered by date
-- 2. Partitions: one DataView per product_type
-- 3. Aggregates: evaluated on each partition
-- 4. Result: new DataTable with one row per group
```

## Memory Efficiency Analysis

For 100K rows with 10 groups:
- Traditional approach: 10 copies of data = 10x memory
- Our approach: 10 DataViews = 10 * sizeof(Vec<usize>) ≈ 800KB overhead

## Recursive GROUP BY Support

Since the result is itself a DataView, we can apply GROUP BY recursively:

```sql
-- First level grouping
WITH grouped AS (
    SELECT region, product, SUM(sales) as total
    FROM data
    GROUP BY region, product
)
-- Second level grouping  
SELECT region, SUM(total) as region_total
FROM grouped
GROUP BY region
```

## Performance Targets

- 100K rows, 10 groups: < 100ms
- 100K rows, 100 groups: < 200ms
- 100K rows, 1000 groups: < 500ms

## Implementation Priority

1. **Week 1**: Basic aggregates (COUNT, SUM, AVG)
2. **Week 2**: GROUP BY single column
3. **Week 3**: GROUP BY multiple columns
4. **Week 4**: Complex aggregates (STDDEV, percentiles)

## Testing Strategy

1. Unit tests for each aggregate function
2. Integration tests for GROUP BY scenarios
3. Performance benchmarks with large datasets
4. Memory usage validation

## Future Extensions

1. **HAVING clause**: Filter after aggregation
2. **ROLLUP/CUBE**: Multi-level aggregations
3. **Window functions**: Running totals, ranks
4. **Parallel aggregation**: Process partitions concurrently