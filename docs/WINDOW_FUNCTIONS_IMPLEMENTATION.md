# Window Functions Implementation Architecture

## Overview
Window functions require a helper class (`WindowContext`) that manages partitioned and ordered views of data, similar to GROUP BY but maintaining row-level access for functions like LAG/LEAD.

## Core Concept

```rust
// Window functions operate on "frames" within partitions
// Each partition is an ordered subset of rows
// Unlike GROUP BY, we don't collapse rows - we augment them

WindowContext {
    partitions: BTreeMap<PartitionKey, OrderedPartition>,
    frame_spec: FrameSpecification,
}

OrderedPartition {
    rows: Vec<usize>,          // Row indices in original DataView
    sort_order: Vec<SortKey>,  // How this partition is ordered
}
```

## Architecture Design

### 1. WindowContext Helper Class

```rust
pub struct WindowContext {
    source: Arc<DataView>,
    partitions: BTreeMap<Vec<DataValue>, OrderedPartition>,
    window_spec: WindowSpecification,
}

impl WindowContext {
    /// Create window context from DataView with PARTITION BY and ORDER BY
    pub fn new(
        view: Arc<DataView>,
        partition_by: Vec<String>,
        order_by: Vec<OrderByColumn>,
    ) -> Result<Self> {
        // 1. Partition the data (like GROUP BY)
        // 2. Sort each partition independently
        // 3. Store ordered row indices
    }
    
    /// Get value at offset from current row (for LAG/LEAD)
    pub fn get_offset_value(
        &self,
        current_row: usize,
        offset: i32,
        column: &str,
    ) -> Option<DataValue> {
        // Find which partition this row belongs to
        // Navigate within that partition's ordered rows
    }
    
    /// Get row number within partition
    pub fn get_row_number(&self, row_index: usize) -> usize {
        // Return position within partition
    }
    
    /// Get first/last value in partition
    pub fn get_first_value(&self, row_index: usize, column: &str) -> DataValue
    pub fn get_last_value(&self, row_index: usize, column: &str) -> DataValue
}
```

### 2. OrderedPartition Structure

```rust
pub struct OrderedPartition {
    /// Original row indices from DataView, in sorted order
    rows: Vec<usize>,
    
    /// Quick lookup: row_index -> position in partition
    row_positions: HashMap<usize, usize>,
    
    /// The sort keys used for ordering
    sort_spec: Vec<SortColumn>,
}

impl OrderedPartition {
    /// Navigate to offset from current position
    pub fn get_row_at_offset(&self, current_pos: usize, offset: i32) -> Option<usize> {
        let target_pos = (current_pos as i32) + offset;
        if target_pos >= 0 && target_pos < self.rows.len() as i32 {
            Some(self.rows[target_pos as usize])
        } else {
            None
        }
    }
    
    /// Get position of row in this partition
    pub fn get_position(&self, row_index: usize) -> Option<usize> {
        self.row_positions.get(&row_index).copied()
    }
}
```

### 3. Integration with Query Engine

```rust
// In query_engine.rs or window_functions.rs

pub fn evaluate_window_function(
    view: &DataView,
    func: &WindowFunction,
    spec: &WindowSpec,
) -> Result<Vec<DataValue>> {
    // Create WindowContext with partition and order specs
    let context = WindowContext::new(
        Arc::new(view.clone()),
        spec.partition_by.clone(),
        spec.order_by.clone(),
    )?;
    
    // Evaluate function for each row
    let mut results = Vec::new();
    for row_idx in view.get_visible_rows() {
        let value = match func {
            WindowFunction::Lag { column, offset, default } => {
                context.get_offset_value(row_idx, -offset, column)
                    .unwrap_or_else(|| default.clone())
            }
            WindowFunction::Lead { column, offset, default } => {
                context.get_offset_value(row_idx, offset, column)
                    .unwrap_or_else(|| default.clone())
            }
            WindowFunction::RowNumber => {
                DataValue::Integer(context.get_row_number(row_idx) as i64)
            }
            WindowFunction::FirstValue { column } => {
                context.get_first_value(row_idx, column)
            }
            WindowFunction::LastValue { column } => {
                context.get_last_value(row_idx, column)
            }
            // ... other functions
        };
        results.push(value);
    }
    
    Ok(results)
}
```

## Implementation Phases

### Phase 1: LAG/LEAD (Simplest)
```sql
SELECT 
    price,
    LAG(price, 1) OVER (ORDER BY trade_time) as prev_price,
    price - LAG(price, 1) OVER (ORDER BY trade_time) as price_change
FROM trades
```

**Implementation:**
1. Create WindowContext with no partitions (single partition)
2. Order by trade_time
3. For each row, navigate offset within ordered partition

### Phase 2: ROW_NUMBER
```sql
SELECT 
    trader,
    price,
    ROW_NUMBER() OVER (ORDER BY price DESC) as price_rank
FROM trades
```

**Implementation:**
1. Single partition, ordered by price DESC
2. Return position in ordered list

### Phase 3: PARTITION BY
```sql
SELECT 
    trader,
    price,
    ROW_NUMBER() OVER (PARTITION BY trader ORDER BY price DESC) as rank_within_trader
FROM trades
```

**Implementation:**
1. Multiple partitions (one per trader)
2. Each partition independently ordered
3. Row number restarts at 1 for each partition

### Phase 4: FIRST_VALUE/LAST_VALUE
```sql
SELECT 
    trader,
    price,
    FIRST_VALUE(price) OVER (PARTITION BY trader ORDER BY trade_time) as opening_price,
    LAST_VALUE(price) OVER (PARTITION BY trader ORDER BY trade_time) as current_price
FROM trades
```

## Key Design Decisions

### 1. Partition Storage
- Use `BTreeMap` for sorted partition keys
- Each partition maintains its own ordered row indices
- Quick lookup from row -> partition -> position

### 2. Memory Efficiency
- Don't duplicate data, only store indices
- Share source DataView across all partitions
- Lazy evaluation where possible

### 3. API Design
- WindowContext encapsulates all partition logic
- Clean separation from DataView (composition, not modification)
- Reusable for different window functions

### 4. Performance Considerations
- Pre-compute partitions and ordering once
- O(1) lookup for row position within partition
- O(1) navigation for LAG/LEAD within partition

## Example Usage Flow

```rust
// 1. Parser creates WindowSpec from SQL
let window_spec = WindowSpec {
    partition_by: vec!["trader".to_string()],
    order_by: vec![OrderByColumn { 
        column: "price".to_string(), 
        ascending: false 
    }],
};

// 2. Create WindowContext
let window_ctx = WindowContext::new(data_view, window_spec)?;

// 3. Evaluate window function for each row
for row in data_view.get_visible_rows() {
    let lag_value = window_ctx.get_offset_value(row, -1, "price");
    let row_num = window_ctx.get_row_number(row);
    // Add to result columns
}
```

## Benefits of This Architecture

1. **Separation of Concerns**: Window logic separate from DataView
2. **Reusability**: Same context works for multiple window functions
3. **Efficiency**: Partitions computed once, used many times
4. **Extensibility**: Easy to add new window functions
5. **Similarity to GROUP BY**: Leverages existing partitioning concepts

## Next Steps

1. Implement WindowContext struct
2. Add LAG/LEAD as proof of concept
3. Extend parser to recognize OVER clause
4. Add tests with sample data
5. Optimize for large datasets