# Window Functions Implementation Design

## Overview
This document outlines an incremental approach to implementing window functions in SQL CLI, leveraging our existing DataView architecture to minimize refactoring.

## Priority Order (Easiest to Hardest)

### Phase 1: HAVING Clause (Easiest)
**Why Easy**: We already have GROUP BY working, HAVING is just post-aggregation filtering.

```sql
SELECT trader, COUNT(*) as count, SUM(quantity) as total
FROM trades
GROUP BY trader
HAVING COUNT(*) > 5 AND SUM(quantity) > 1000
```

**Implementation**:
1. Parse HAVING clause after GROUP BY in recursive_parser.rs
2. Evaluate HAVING conditions after aggregation in query_engine.rs
3. Filter grouped results before returning

### Phase 2: Simple Window Functions (LAG/LEAD)
**Why Manageable**: These only need access to adjacent rows, no complex partitioning.

```sql
SELECT 
    price,
    LAG(price, 1) OVER (ORDER BY trade_time) as prev_price,
    LEAD(price, 1) OVER (ORDER BY trade_time) as next_price,
    price - LAG(price, 1) OVER (ORDER BY trade_time) as price_change
FROM trades
```

**Implementation using DataView**:
1. Add window function expressions to SelectItem enum
2. Create a sorted DataView based on ORDER BY
3. For each visible row, compute LAG/LEAD by accessing adjacent rows
4. Return results as computed columns

### Phase 3: ROW_NUMBER with Simple Ordering
**Why Next**: Natural extension of LAG/LEAD, just needs position tracking.

```sql
SELECT 
    trader,
    price,
    ROW_NUMBER() OVER (ORDER BY price DESC) as price_rank
FROM trades
```

**Implementation**:
1. Sort DataView by ORDER BY clause
2. Add row numbers as we iterate through visible_rows
3. Return as computed column

### Phase 4: PARTITION BY with ROW_NUMBER
**Why More Complex**: Requires grouping + ordering within groups.

```sql
SELECT 
    trader,
    book,
    price,
    ROW_NUMBER() OVER (PARTITION BY trader ORDER BY price DESC) as rank_within_trader
FROM trades
```

**Implementation using DataView**:
1. Use existing group_by() to partition data
2. For each partition DataView, apply ordering
3. Compute ROW_NUMBER within each partition
4. Merge results back

### Phase 5: Advanced Window Functions (RANK, DENSE_RANK, PERCENT_RANK)
**Why Complex**: Need to handle ties and percentage calculations.

```sql
SELECT 
    trader,
    price,
    RANK() OVER (ORDER BY price DESC) as rank,
    DENSE_RANK() OVER (ORDER BY price DESC) as dense_rank,
    PERCENT_RANK() OVER (ORDER BY price DESC) as percent_rank
FROM trades
```

### Phase 6: Moving Aggregates (SUM/AVG OVER)
**Why Most Complex**: Need frame specifications (ROWS BETWEEN).

```sql
SELECT 
    trade_time,
    price,
    AVG(price) OVER (ORDER BY trade_time ROWS BETWEEN 5 PRECEDING AND CURRENT ROW) as moving_avg
FROM trades
```

## Implementation Strategy

### 1. Extend AST for Window Functions
```rust
#[derive(Debug, Clone)]
pub enum WindowFunction {
    RowNumber,
    Lag { offset: i32 },
    Lead { offset: i32 },
    Rank,
    DenseRank,
}

#[derive(Debug, Clone)]
pub struct WindowSpec {
    pub partition_by: Vec<String>,
    pub order_by: Vec<OrderByColumn>,
    pub frame: Option<WindowFrame>,
}

#[derive(Debug, Clone)]
pub enum SelectItem {
    Column(String),
    Expression { expr: SqlExpression, alias: String },
    WindowExpression { 
        func: WindowFunction,
        spec: WindowSpec,
        alias: String 
    },
    Star,
}
```

### 2. DataView Extensions
```rust
impl DataView {
    /// Get value from a different row (for LAG/LEAD)
    pub fn get_offset_value(&self, current_idx: usize, offset: i32, column: &str) -> Option<DataValue> {
        let target_idx = (current_idx as i32 + offset) as usize;
        if target_idx < self.visible_rows.len() {
            let row_idx = self.visible_rows[target_idx];
            self.source.get_value(row_idx, column)
        } else {
            None
        }
    }
    
    /// Partition data and apply function to each partition
    pub fn partition_apply<F>(&self, partition_cols: &[String], apply_fn: F) -> Result<DataView>
    where F: Fn(&DataView) -> Result<Vec<(usize, DataValue)>>
    {
        // Use existing group_by
        let partitions = self.group_by(partition_cols)?;
        // Apply function to each partition
        // Merge results
    }
}
```

### 3. Query Engine Integration
```rust
// In query_engine.rs
fn evaluate_window_functions(&self, view: DataView, window_exprs: Vec<WindowExpression>) -> Result<DataView> {
    // For each window expression:
    // 1. Sort/partition as needed
    // 2. Compute window function values
    // 3. Add as new column to result
}
```

## Benefits of This Approach

1. **Incremental**: Each phase builds on the previous
2. **Leverages DataView**: Reuses our existing abstractions
3. **No Major Refactoring**: Works within current architecture
4. **Testable**: Each phase can be fully tested before moving on
5. **User Value**: Even Phase 1 (HAVING) adds immediate value

## Testing Strategy

For each phase:
1. Add parser tests for new syntax
2. Add unit tests for DataView methods
3. Add integration tests with sample data
4. Add Python tests for end-to-end validation

## Example Use Cases

### HAVING (Phase 1)
```sql
-- Find high-volume traders
SELECT trader, COUNT(*) as trades, SUM(quantity) as volume
FROM trades
GROUP BY trader
HAVING COUNT(*) > 100 AND SUM(quantity) > 10000
```

### LAG/LEAD (Phase 2)
```sql
-- Price momentum analysis
SELECT 
    trade_time,
    price,
    LAG(price, 1) OVER (ORDER BY trade_time) as prev_price,
    CASE 
        WHEN price > LAG(price, 1) OVER (ORDER BY trade_time) THEN 'UP'
        WHEN price < LAG(price, 1) OVER (ORDER BY trade_time) THEN 'DOWN'
        ELSE 'FLAT'
    END as momentum
FROM trades
```

### ROW_NUMBER with PARTITION (Phase 4)
```sql
-- Top 3 trades per trader
SELECT * FROM (
    SELECT 
        trader,
        trade_time,
        price,
        quantity,
        ROW_NUMBER() OVER (PARTITION BY trader ORDER BY quantity DESC) as rn
    FROM trades
) t
WHERE rn <= 3
```

## Next Steps

1. Start with HAVING clause implementation
2. Get user feedback on syntax preferences
3. Implement LAG/LEAD as proof of concept for window functions
4. Gradually add more complex features based on user needs