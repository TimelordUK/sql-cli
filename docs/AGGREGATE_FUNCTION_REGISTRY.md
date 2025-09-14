# Aggregate Function Registry Architecture

## Overview

The Aggregate Function Registry provides a clean, extensible API for group-based aggregate computations, moving all aggregate logic out of the evaluator into a registry pattern.

## Core Design Principles

1. **Separation of Concerns**: Aggregate logic is completely decoupled from the evaluator
2. **Extensibility**: Easy to add new aggregate functions without modifying core
3. **State Management**: Each aggregate manages its own state independently
4. **DISTINCT Support**: Built-in support for DISTINCT across all aggregates
5. **Performance**: Potential for parallel processing of independent groups

## Architecture

### AggregateFunction Trait

```rust
pub trait AggregateFunction: Send + Sync {
    /// Function name (e.g., "SUM", "COUNT", "STRING_AGG")
    fn name(&self) -> &str;

    /// Create initial state for this aggregate
    fn create_state(&self) -> Box<dyn AggregateState>;

    /// Does this aggregate support DISTINCT?
    fn supports_distinct(&self) -> bool;

    /// For aggregates with parameters (like STRING_AGG separator)
    fn set_parameters(&self, params: &[DataValue]) -> Result<Box<dyn AggregateFunction>>;
}
```

### AggregateState Trait

```rust
pub trait AggregateState: Send + Sync {
    /// Add a value to the aggregate
    fn accumulate(&mut self, value: &DataValue) -> Result<()>;

    /// Finalize and return the aggregate result
    fn finalize(self: Box<Self>) -> DataValue;

    /// Reset the state for reuse
    fn reset(&mut self);
}
```

## Built-in Aggregates

### Basic Aggregates
- `COUNT(*)` or `COUNT(column)` - Count rows or non-null values
- `SUM(column)` - Sum numeric values
- `AVG(column)` - Calculate average
- `MIN(column)` - Find minimum value
- `MAX(column)` - Find maximum value

### String Aggregates
- `STRING_AGG(column, separator)` - Concatenate strings with separator
  - Supports `DISTINCT`: `STRING_AGG(DISTINCT column, ', ')`

### Statistical Aggregates (Planned)
- `STDDEV(column)` - Standard deviation
- `VARIANCE(column)` - Variance
- `MEDIAN(column)` - Median value
- `PERCENTILE(column, n)` - Nth percentile

## Usage Examples

### Before (Hardcoded in Evaluator)
```rust
// In arithmetic_evaluator.rs
match name {
    "COUNT" => {
        // Hardcoded COUNT logic
    }
    "SUM" => {
        // Hardcoded SUM logic
    }
    // ... more hardcoded aggregates
}
```

### After (Registry Pattern)
```rust
// In arithmetic_evaluator.rs
if let Some(agg_func) = self.aggregate_registry.get(name) {
    let mut state = agg_func.create_state();

    // Process values (with DISTINCT handling if needed)
    for value in values {
        state.accumulate(value)?;
    }

    return Ok(state.finalize());
}
```

## SQL Examples

### STRING_AGG with DISTINCT
```sql
-- Concatenate distinct currencies
SELECT
    product,
    STRING_AGG(DISTINCT currency, ', ') as currencies
FROM international_sales
GROUP BY product;
```

### COUNT with DISTINCT
```sql
-- Count unique customers per product
SELECT
    product,
    COUNT(DISTINCT customer_id) as unique_customers
FROM sales
GROUP BY product;
```

## Integration Points

### Parser Integration
The parser recognizes aggregate functions and:
1. Parses DISTINCT keyword if present
2. Extracts parameters (like STRING_AGG separator)
3. Creates appropriate AST nodes

### Evaluator Integration
The evaluator:
1. Looks up the function in the registry
2. Creates aggregate state
3. Handles DISTINCT by deduplicating values
4. Accumulates values into state
5. Finalizes to get result

### GROUP BY Processing
For GROUP BY queries:
1. Create separate state for each group
2. Accumulate values per group
3. Finalize all states at the end

## Benefits

1. **Clean Architecture**: Evaluator no longer contains aggregate logic
2. **Extensibility**: Add new aggregates without touching core code
3. **Testability**: Each aggregate tested independently
4. **Maintainability**: Aggregate logic centralized in one place
5. **Performance**: Clear abstraction for future optimizations
6. **DISTINCT Support**: Uniform DISTINCT handling across all aggregates

## Adding New Aggregates

To add a new aggregate function:

1. **Implement the AggregateFunction trait**:
```rust
struct MyAggFunction;

impl AggregateFunction for MyAggFunction {
    fn name(&self) -> &str { "MY_AGG" }

    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(MyAggState::new())
    }
}
```

2. **Implement the AggregateState trait**:
```rust
struct MyAggState {
    // Your state fields
}

impl AggregateState for MyAggState {
    fn accumulate(&mut self, value: &DataValue) -> Result<()> {
        // Process each value
    }

    fn finalize(self: Box<Self>) -> DataValue {
        // Return final result
    }
}
```

3. **Register in the registry**:
```rust
registry.register(Box::new(MyAggFunction));
```

## Future Enhancements

1. **Parallel Group Processing**: Process independent groups in parallel
2. **Streaming Aggregates**: Support for streaming/windowed aggregates
3. **Approximate Aggregates**: HyperLogLog for COUNT(DISTINCT) on large datasets
4. **Custom Aggregates**: Allow users to define custom aggregates in Lua
5. **Memory Optimization**: Spill to disk for large GROUP BY operations