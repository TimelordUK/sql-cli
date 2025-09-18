# GROUP BY and Aggregation Optimization Plan

**Date**: September 18, 2025
**Current Performance**: ~3 seconds for 50K rows
**Target Performance**: < 500ms for 50K rows (6x improvement minimum)

## Problem Analysis

Current GROUP BY operations show super-linear scaling:
- 10K rows: ~595ms
- 20K rows: ~1200ms
- 50K rows: ~3200ms

This suggests O(n²) or O(n log n) behavior with high constants, likely due to:
- Inefficient hash map operations
- Excessive memory allocations
- Poor cache locality
- Redundant data copying

## Optimization Strategies (Minimal Architecture Changes)

### 1. Hash-Based Aggregation Improvements ⚡ [QUICK WIN]

**Current Issue**: Default HashMap with repeated rehashing and slow hash function

**Solution**:
```rust
// Replace:
let mut groups: HashMap<GroupKey, Vec<RowIndex>> = HashMap::new();

// With:
use fxhash::FxHashMap; // or ahash::AHashMap
let estimated_groups = estimate_cardinality(&group_columns);
let mut groups: FxHashMap<GroupKey, Vec<RowIndex>> =
    FxHashMap::with_capacity_and_hasher(estimated_groups, Default::default());
```

**Expected Impact**: 2-3x faster hashing, no rehashing overhead

### 2. Streaming Aggregation with Pre-Sort

**When Applicable**: Data naturally ordered or sort cost amortized over multiple queries

**Implementation**:
```rust
// One-time sort cost O(n log n)
sort_by_columns(&mut data, &group_columns);

// Then stream through with O(n) aggregation
let mut current_group = None;
let mut current_aggregates = AggregateState::new();

for row in sorted_data {
    if row.group != current_group {
        if let Some(group) = current_group {
            emit_result(group, current_aggregates);
        }
        current_group = Some(row.group);
        current_aggregates.reset(); // Reuse object
    }
    current_aggregates.update(row);
}
```

**Expected Impact**: 5-10x faster for pre-sorted data, constant memory usage

### 3. Aggregate State Object Pooling ⚡ [QUICK WIN]

**Current Issue**: Creating/destroying aggregate state objects per group

**Solution**:
```rust
struct AggregatePool {
    available: Vec<Box<AggregateState>>,
}

impl AggregatePool {
    fn acquire(&mut self) -> Box<AggregateState> {
        self.available.pop().unwrap_or_else(|| Box::new(AggregateState::new()))
    }

    fn release(&mut self, mut state: Box<AggregateState>) {
        state.reset();
        self.available.push(state);
    }
}
```

**Expected Impact**: 30-50% reduction in allocation overhead

### 4. Type-Specialized Aggregators

**Current Issue**: Generic DataValue enum boxing/unboxing overhead

**Solution**:
```rust
enum TypedAggregator {
    IntSum { sum: i64, count: usize },
    FloatSum { sum: f64, count: usize },
    StringConcat { buffer: String, count: usize },
    // ... specialized for each type
}

// Dispatch once at start based on column type
let aggregator = match column.data_type() {
    DataType::Integer => TypedAggregator::IntSum { sum: 0, count: 0 },
    DataType::Float => TypedAggregator::FloatSum { sum: 0.0, count: 0 },
    // ...
};
```

**Expected Impact**: 2-3x faster for numeric aggregations

### 5. Lazy/Selective Aggregation

**Optimization**: Only compute aggregates that appear in SELECT clause

```rust
struct LazyAggregates {
    needed: HashSet<AggregateFunction>,
    computed: HashMap<AggregateFunction, DataValue>,
}

// Only compute what's needed
if lazy_aggs.needed.contains(&AggFunc::Sum) {
    lazy_aggs.computed.insert(AggFunc::Sum, compute_sum());
}
```

**Expected Impact**: Proportional to unused aggregates (often 2x in practice)

### 6. Parallel Aggregation (Using Rayon) ⚡ [MEDIUM EFFORT]

**Implementation**:
```rust
use rayon::prelude::*;

fn parallel_group_by(data: &DataTable, group_cols: &[String]) -> Groups {
    let num_partitions = rayon::current_num_threads();

    // Parallel partition and aggregate
    let partitions: Vec<_> = (0..num_partitions)
        .into_par_iter()
        .map(|p| {
            let partition_data = partition_by_hash(data, p, num_partitions);
            aggregate_partition(partition_data, group_cols)
        })
        .collect();

    // Serial merge (usually small)
    merge_partition_results(partitions)
}
```

**Expected Impact**: 3-4x on multi-core systems

### 7. Smart Cardinality Detection

**Optimization**: Choose strategy based on expected group count

```rust
fn estimate_cardinality(table: &DataTable, group_cols: &[String]) -> usize {
    // Sample first 1000 rows
    let sample_size = min(1000, table.row_count());
    let mut seen = HashSet::new();

    for i in 0..sample_size {
        let key = extract_group_key(table, i, group_cols);
        seen.insert(key);
    }

    // Estimate total cardinality
    let estimated = (seen.len() * table.row_count()) / sample_size;

    // Choose strategy
    if estimated < 100 {
        Strategy::HashAggregate
    } else if estimated > table.row_count() / 10 {
        Strategy::SortAggregate
    } else {
        Strategy::AdaptiveAggregate
    }
}
```

### 8. Memory Layout Optimization

**Pack group keys efficiently**:
```rust
// Instead of:
struct GroupKey {
    col1: String,
    col2: String,
    col3: i64,
}

// Use:
struct PackedGroupKey {
    // Intern strings and use indices
    str_indices: Vec<u32>,
    // Pack numeric values
    numeric_values: Vec<i64>,
}
```

## Implementation Priority

### Phase 1: Quick Wins (1 day)
1. ✅ Replace HashMap with FxHashMap
2. ✅ Pre-size hash maps based on cardinality estimates
3. ✅ Object pooling for aggregate states

**Expected Combined Impact**: 3-5x improvement

### Phase 2: Algorithmic Improvements (2-3 days)
1. ⏳ Implement streaming aggregation for sorted data
2. ⏳ Type-specialized aggregators
3. ⏳ Lazy aggregation for unused columns

**Expected Combined Impact**: Additional 2-3x improvement

### Phase 3: Advanced Optimizations (3-5 days)
1. ⏳ Parallel aggregation with Rayon
2. ⏳ Adaptive strategy selection
3. ⏳ Memory layout optimization

**Expected Combined Impact**: Additional 2-4x improvement

## Success Metrics

| Query Type | Current (50K) | Phase 1 Target | Phase 2 Target | Phase 3 Target |
|------------|---------------|----------------|----------------|----------------|
| Simple GROUP BY | 3200ms | 800ms | 400ms | 200ms |
| Multi-column GROUP BY | 5600ms | 1400ms | 700ms | 350ms |
| GROUP BY + HAVING | 3500ms | 875ms | 437ms | 220ms |
| Multiple aggregates | 2800ms | 700ms | 350ms | 175ms |

## Testing Strategy

1. Create benchmark suite specifically for GROUP BY operations
2. Test with varying cardinalities (1, 10, 100, 1000, 10000 groups)
3. Test with different data types (int, float, string grouping)
4. Measure memory usage alongside performance
5. Verify correctness with existing test suite

## Code Locations to Modify

- `src/data/query_executor.rs` - Main GROUP BY execution
- `src/data/group_by_expressions.rs` - Grouping logic
- `src/data/arithmetic_evaluator.rs` - Aggregate evaluation
- Consider creating new `src/data/aggregate_optimizer.rs`

## Dependencies to Add

```toml
[dependencies]
fxhash = "0.2"  # Faster hashing
ahash = "0.8"   # Alternative fast hasher
rayon = "1.7"   # For parallel execution (already have this)
```

## Notes

- Start with Phase 1 for immediate impact
- Measure after each optimization to verify improvements
- Keep old code path available via feature flag initially
- Consider caching aggregate results for repeated queries
- Profile with `perf` or `flamegraph` to identify remaining bottlenecks

## References

- [ClickHouse Aggregation](https://clickhouse.com/docs/en/development/architecture/#aggregation)
- [DuckDB Push-Based Execution](https://duckdb.org/2021/08/27/external-sorting.html)
- [Apache Arrow Compute Kernels](https://arrow.apache.org/docs/cpp/compute.html)