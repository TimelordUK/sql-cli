# Window Function Registry Architecture

## Overview

The Window Function Registry provides a clean, extensible API for window computations with syntactic sugar to make complex patterns accessible to users.

## Core Design Principles

1. **Separation of Concerns**: Window logic is decoupled from the core engine
2. **Syntactic Sugar**: Complex patterns wrapped in simple, intuitive functions
3. **Extensibility**: Easy to add new window functions without modifying core
4. **Performance**: Potential for GPU offloading and parallel computation
5. **Composability**: Functions can build on existing window infrastructure

## API Design

### WindowFunction Trait

```rust
pub trait WindowFunction: Send + Sync {
    /// Function name (e.g., "MOVING_AVG")
    fn name(&self) -> &str;

    /// Compute the function value for a specific row
    fn compute(
        &self,
        context: &WindowContext,
        row_index: usize,
        args: &[SqlExpression],
        evaluator: &mut dyn ExpressionEvaluator,
    ) -> Result<DataValue>;

    /// Transform/expand the window specification
    fn transform_window_spec(
        &self,
        base_spec: &WindowSpec,
        args: &[SqlExpression],
    ) -> Result<WindowSpec>;
}
```

### Key Components

1. **WindowContext**: Contains partitioned and ordered data with frame support
2. **ExpressionEvaluator**: Evaluates SQL expressions without coupling to ArithmeticEvaluator
3. **WindowFunctionRegistry**: Manages registration and lookup of window functions

## Syntactic Sugar Functions

### Currently Implemented

| Function | Signature | Expands To |
|----------|-----------|------------|
| `MOVING_AVG` | `MOVING_AVG(column, n)` | `AVG(column) OVER (ORDER BY ... ROWS n-1 PRECEDING)` |
| `ROLLING_STDDEV` | `ROLLING_STDDEV(column, n)` | `STDDEV(column) OVER (ORDER BY ... ROWS n-1 PRECEDING)` |
| `CUMULATIVE_SUM` | `CUMULATIVE_SUM(column)` | `SUM(column) OVER (ORDER BY ... ROWS UNBOUNDED PRECEDING)` |
| `CUMULATIVE_AVG` | `CUMULATIVE_AVG(column)` | `AVG(column) OVER (ORDER BY ... ROWS UNBOUNDED PRECEDING)` |
| `Z_SCORE` | `Z_SCORE(column, n)` | `(value - AVG) / STDDEV` over n-row window |

### Planned Functions

| Function | Purpose | Priority |
|----------|---------|----------|
| `BOLLINGER_UPPER` | Upper Bollinger Band | High |
| `BOLLINGER_LOWER` | Lower Bollinger Band | High |
| `EXPONENTIAL_AVG` | Exponential moving average | Medium |
| `MEDIAN_IN_WINDOW` | Rolling median | Medium |
| `PERCENTILE_IN_WINDOW` | Rolling percentile | Low |

## Usage Examples

### Before (Verbose)
```sql
SELECT
    date,
    close,
    AVG(close) OVER (ORDER BY date ROWS 19 PRECEDING) as ma_20,
    STDDEV(close) OVER (ORDER BY date ROWS 19 PRECEDING) as vol_20,
    SUM(volume) OVER (ORDER BY date ROWS UNBOUNDED PRECEDING) as cumul_vol
FROM stock_data;
```

### After (With Syntactic Sugar)
```sql
SELECT
    date,
    close,
    MOVING_AVG(close, 20) as ma_20,
    ROLLING_STDDEV(close, 20) as vol_20,
    CUMULATIVE_SUM(volume) as cumul_vol
FROM stock_data;
```

## Integration Points

### Parser Integration
The parser recognizes syntactic sugar functions and:
1. Validates arguments using `validate_args()`
2. Transforms the window specification using `transform_window_spec()`
3. Creates a WindowFunction AST node

### Evaluator Integration
The arithmetic evaluator:
1. Looks up the function in the registry
2. Creates/gets the WindowContext
3. Calls `compute()` for each row
4. Returns the computed value

## Performance Optimizations

### Future Enhancements

1. **GPU Offloading**: For large windows, offload computation to GPU
   - Parallel computation across partitions
   - SIMD operations for aggregates

2. **Multi-threading**: Process independent partitions in parallel
   - Thread pool for partition processing
   - Lock-free data structures for results

3. **Lazy Evaluation**: Compute only what's needed
   - Skip computation for filtered rows
   - Cache intermediate results

4. **Memory Optimization**:
   - Columnar storage for better cache locality
   - Compressed frame representations

## Extension Guide

To add a new window function:

1. **Implement the WindowFunction trait**:
```rust
struct MyFunction;

impl WindowFunction for MyFunction {
    fn name(&self) -> &str { "MY_FUNCTION" }

    fn compute(&self, context: &WindowContext, ...) -> Result<DataValue> {
        // Your computation logic
    }

    fn transform_window_spec(&self, ...) -> Result<WindowSpec> {
        // Modify window frame as needed
    }
}
```

2. **Register in the registry**:
```rust
registry.register(Box::new(MyFunction));
```

3. **Add tests**:
```rust
#[test]
fn test_my_function() {
    // Test computation and window transformation
}
```

## Benefits

1. **User-Friendly**: Complex patterns become one-liners
2. **Maintainable**: Window logic centralized in one place
3. **Extensible**: New functions added without core changes
4. **Optimizable**: Clear abstraction for performance improvements
5. **Testable**: Each function tested independently

## Next Steps

1. Integrate registry with parser and evaluator
2. Add more financial functions (Bollinger Bands, EMA, RSI)
3. Implement parallel partition processing
4. Add GPU acceleration for large datasets
5. Create Lua bindings for custom functions