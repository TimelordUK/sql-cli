# GROUP BY Architecture Design

## Overview
Implement GROUP BY using a recursive descent approach with DataView partitions, where each grouping level creates a map of child views.

## Core Concepts

### 1. Single Column Grouping per Level
- Each DataView can handle ONE GROUP BY column at a time
- Creates a HashMap<GroupValue, DataView> for that level
- Multiple columns handled through recursive descent

### 2. Master View Pattern
```rust
struct GroupedView {
    source: DataView,
    group_column: String,
    partitions: HashMap<DataValue, DataView>,
    aggregates: Vec<AggregateExpr>,
}
```

### 3. Recursive Descent for Multiple Columns
For `GROUP BY country, city, department`:
```
MasterView
├── country="USA" → View
│   ├── city="NYC" → View
│   │   ├── department="Sales" → View (leaf - apply aggregates)
│   │   └── department="IT" → View (leaf - apply aggregates)
│   └── city="LA" → View
│       └── department="Sales" → View (leaf - apply aggregates)
└── country="UK" → View
    └── city="London" → View
        └── department="IT" → View (leaf - apply aggregates)
```

## Implementation Steps

### Phase 1: Additional Aggregate Functions
- [ ] GREATEST(col1, col2, ...) - Maximum value across columns
- [ ] LEAST(col1, col2, ...) - Minimum value across columns  
- [ ] GROUP_CONCAT(col) - Concatenate values
- [ ] STDDEV(col) - Standard deviation
- [ ] VARIANCE(col) - Variance

### Phase 2: Single Column GROUP BY
- [ ] Create PartitionManager that takes DataView + group column
- [ ] Build HashMap<DataValue, Vec<usize>> for row indices
- [ ] Create child DataView for each unique value
- [ ] Apply aggregates to each partition

### Phase 3: Multi-Column GROUP BY
- [ ] Recursive descent through group columns
- [ ] Each level creates its own partition map
- [ ] Leaf nodes apply aggregate calculations
- [ ] Bubble results back up through the tree

### Phase 4: HAVING Clause
- [ ] Filter aggregated results
- [ ] Apply after GROUP BY completes
- [ ] Reuse WHERE evaluator with aggregate results

## Example Query Flow

```sql
SELECT country, city, COUNT(*) as cnt, AVG(salary) as avg_sal
FROM employees
WHERE active = true
GROUP BY country, city
HAVING COUNT(*) > 10
ORDER BY avg_sal DESC
```

1. Apply WHERE → filtered DataView
2. Partition by country → Map<country, DataView>
3. For each country, partition by city → Map<city, DataView>
4. Apply aggregates to leaf views (COUNT, AVG)
5. Collect results into new DataTable
6. Apply HAVING on aggregate results
7. Apply ORDER BY on final results

## Key Design Decisions

### Why Single Column per View?
- Simpler implementation
- Clear ownership of partitions
- Natural recursive structure
- Easier to debug and optimize

### Why DataView-based?
- Reuses existing infrastructure
- Virtual views are lightweight
- Already have filtering/sorting
- Natural for window functions later

### Memory Considerations
- Views share underlying DataTable
- Only store row indices, not duplicate data
- Lazy evaluation where possible
- Can spill large groups to disk later

## Testing Strategy

### Start Simple
1. Single column GROUP BY with COUNT(*)
2. Add SUM, AVG, MIN, MAX
3. Multi-column GROUP BY
4. Complex aggregates (GREATEST, LEAST)
5. HAVING clause
6. Performance testing with large datasets

### Test Data
- Use trades data (GROUP BY symbol, exchange)
- Geographic data (GROUP BY country, city)
- Time series (GROUP BY date functions)

## Future Extensions

### Window Functions (Later)
- Can use similar partitioning approach
- OVER (PARTITION BY x ORDER BY y)
- Running totals, rank, row_number

### Optimization Opportunities
- Hash-based aggregation for large groups
- Sort-based aggregation for ordered data
- Parallel aggregation for independent groups
- Incremental aggregation for streaming

## Implementation Notes

### Tomorrow's Priority:
1. Start with GREATEST/LEAST functions (simpler, builds on existing)
2. Implement single-column GROUP BY with COUNT(*) 
3. Test with trades data grouping by symbol
4. Add remaining aggregate functions
5. Move to multi-column if time permits

### Code Location:
- `src/data/partition_manager.rs` - New partitioning logic
- `src/data/grouped_view.rs` - GROUP BY view implementation  
- `src/sql/aggregates/` - Additional aggregate functions
- `src/data/query_engine.rs` - Wire up GROUP BY execution