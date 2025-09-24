# GROUP BY Performance Optimization Results

## Problem Identified
The GROUP BY operation was creating a new `ArithmeticEvaluator` instance for EVERY expression evaluation in EVERY row. For 30,000 rows with 1 GROUP BY expression, this meant:
- 30,000 ArithmeticEvaluator creations
- Each creation instantiated 4 function registries with hundreds of functions
- Massive unnecessary overhead

## Easy Win Solution
**Simple fix**: Create the ArithmeticEvaluator once before the loop and reuse it.

```rust
// BEFORE - Creating evaluator inside nested loops
for row_idx in visible_rows {
    for expr in group_by_exprs {
        let mut evaluator = ArithmeticEvaluator::new(view.source()); // 30,000+ times!
        let value = evaluator.evaluate(expr, row_idx);
    }
}

// AFTER - Create once, reuse many times
let mut evaluator = ArithmeticEvaluator::new(view.source()); // Once!
for row_idx in visible_rows {
    for expr in group_by_exprs {
        let value = evaluator.evaluate(expr, row_idx);
    }
}
```

## Performance Improvements

| Row Count | Phase | Before (ms) | After (ms) | Speedup |
|-----------|-------|-------------|------------|---------|
| 30,000 | Group Building | 2,191 | 177 | **12.4x** |
| 30,000 | Total GROUP BY | 2,421 | 402 | **6.0x** |
| 50,000 | Group Building | 3,596 | 269 | **13.4x** |
| 50,000 | Total GROUP BY | 3,808 | 633 | **6.0x** |

## Additional Optimizations Made
1. **Vector reuse**: Pre-allocate and clear/reuse the key_values vector instead of creating new ones
2. **Reduced allocations**: Overall memory allocation pressure significantly reduced

## Key Lessons
- **Profile before optimizing**: The execution plan tracing revealed the dominant phase
- **Look for repeated object creation**: Creating complex objects in tight loops is a common performance killer
- **Easy wins exist**: A 2-line change resulted in 6x performance improvement
- **Registries should be shared**: Function/aggregate registries are expensive to create and should be cached

## Future Optimization Opportunities
1. **Cache ArithmeticEvaluator at QueryEngine level**: Could be shared across multiple operations
2. **Optimize simple column references**: Direct column access doesn't need full expression evaluation
3. **Parallel GROUP BY**: Process different row ranges in parallel threads
4. **SIMD optimizations**: Use vectorized operations for simple aggregations