# Current GROUP BY Limitations and Improvement Plan

## Current Implementation Limitations

### 1. Parser Level (recursive_parser.rs:495)
```rust
let group_by = if matches!(self.current_token, Token::GroupBy) {
    self.advance();
    Some(self.parse_identifier_list()?)  // <-- Only parses column names!
} else {
    None
};
```

### 2. AST Level (ast.rs:164)
```rust
pub struct SelectStatement {
    ...
    pub group_by: Option<Vec<String>>,  // <-- Only stores column names!
    ...
}
```

### 3. Execution Level (query_engine.rs)
- `apply_group_by()` expects column names
- Looks up columns by name in the table
- Cannot evaluate expressions for grouping

### 4. DataView Level (data_view.rs)
- `group_by()` method only accepts column names
- Looks up column indices by name
- Groups by actual column values only

## What This Prevents

1. **Time bucketing**: Cannot do `GROUP BY TIME_BUCKET(60, UNIX_TIMESTAMP(trade_time))`
2. **CASE expressions**: Cannot do `GROUP BY CASE WHEN volume < 1000 THEN 'small' ELSE 'large' END`
3. **Function calls**: Cannot do `GROUP BY YEAR(date)` or `GROUP BY UPPER(name)`
4. **Arithmetic**: Cannot do `GROUP BY price / 10` (for price ranges)
5. **Complex expressions**: Cannot do `GROUP BY CONCAT(symbol, '_', exchange)`

## Improvement Plan

### Phase 1: Update AST
- Change `group_by` from `Vec<String>` to `Vec<SqlExpression>`
- This allows storing any expression, not just column names

### Phase 2: Update Parser
- Change from `parse_identifier_list()` to `parse_expression_list()`
- Support parsing full expressions after GROUP BY

### Phase 3: Update Query Engine
- Evaluate GROUP BY expressions for each row
- Create group keys from expression results
- Support both simple columns and complex expressioans

### Phase 4: Update DataView
- Accept computed values for grouping
- Group by expression results rather than column lookups

## Examples That Will Work After Fix

```sql
-- Time-based grouping (1-minute bars)
SELECT 
    TIME_BUCKET(60, UNIX_TIMESTAMP(trade_time)) as minute,
    MIN(price) as low,
    MAX(price) as high,
    SUM(volume) as volume
FROM trades
GROUP BY TIME_BUCKET(60, UNIX_TIMESTAMP(trade_time))

-- CASE-based grouping (trade size buckets)
SELECT 
    CASE 
        WHEN volume < 1000 THEN 'small'
        WHEN volume < 5000 THEN 'medium'
        ELSE 'large'
    END as size_bucket,
    COUNT(*) as count,
    AVG(price) as avg_price
FROM trades
GROUP BY CASE 
    WHEN volume < 1000 THEN 'small'
    WHEN volume < 5000 THEN 'medium'
    ELSE 'large'
END

-- Date grouping
SELECT 
    YEAR(trade_date) as year,
    MONTH(trade_date) as month,
    SUM(amount) as total
FROM transactions
GROUP BY YEAR(trade_date), MONTH(trade_date)

-- Price range grouping
SELECT 
    FLOOR(price / 10) * 10 as price_range,
    COUNT(*) as count
FROM products
GROUP BY FLOOR(price / 10) * 10
```