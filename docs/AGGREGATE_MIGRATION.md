# Aggregate Function Migration Guide

## Overview
We are migrating from a hardcoded aggregate system to a plugin-based registry system that makes aggregate functions as extensible as regular SQL functions.

### Two Systems
1. **Old System** (`src/sql/aggregates/`): Hardcoded aggregate functions
2. **New System** (`src/sql/aggregate_functions/`): Registry-based, uses traits for extensibility

## Migration Pattern (Using SUM as Example)

### Step 1: Ensure Function Exists in New Registry
The new aggregate function should implement the `AggregateFunction` trait and `AggregateState` trait.

```rust
// In src/sql/aggregate_functions/mod.rs

struct SumFunction;

impl AggregateFunction for SumFunction {
    fn name(&self) -> &str { "SUM" }
    fn description(&self) -> &str { "Calculate the sum of values" }
    fn create_state(&self) -> Box<dyn AggregateState> {
        Box::new(SumState { /* ... */ })
    }
}

// Register in the registry
self.register(Box::new(SumFunction));
```

### Step 2: Update ArithmeticEvaluator
The evaluator needs to check both registries during migration:

1. Add new registry to the struct:
```rust
pub struct ArithmeticEvaluator<'a> {
    // ...
    aggregate_registry: Arc<AggregateRegistry>, // Old
    new_aggregate_registry: Arc<AggregateFunctionRegistry>, // New
}
```

2. Update the evaluation logic to check new registry first for migrated functions:
```rust
// In evaluate_function method
if name_upper == "SUM" && self.new_aggregate_registry.get(&name_upper).is_some() {
    // Use new registry
    let agg_func = self.new_aggregate_registry.get(&name_upper).unwrap();
    let mut state = agg_func.create_state();

    // Accumulate values
    for &row_idx in &rows_to_process {
        let value = self.evaluate(&args[0], row_idx)?;
        state.accumulate(&value)?;
    }

    return Ok(state.finalize());
}
```

3. Handle DISTINCT case similarly in `evaluate_aggregate_with_distinct`

### Step 3: Update Aggregate Detection
The `contains_aggregate` and `is_aggregate` functions need to check both registries:

```rust
// In src/sql/aggregates/mod.rs
pub fn contains_aggregate(expr: &SqlExpression) -> bool {
    // Check old registry
    let registry = AggregateRegistry::new();
    if registry.is_aggregate(name) {
        return true;
    }

    // Check new registry for migrated functions
    let new_registry = AggregateFunctionRegistry::new();
    if new_registry.contains(name) {
        return true;
    }
}
```

### Step 4: Remove from Old Registry
Once the function is working through the new registry:
```rust
// In AggregateRegistry::new()
let functions: Vec<Box<dyn AggregateFunction>> = vec![
    // Box::new(SumFunction), // MIGRATED to new registry
    Box::new(AvgFunction),
    // ...
];
```

### Step 5: Test Thoroughly
Test the migrated function with:
- Simple queries: `SELECT SUM(column) FROM table`
- DISTINCT: `SELECT SUM(DISTINCT column) FROM table`
- GROUP BY: `SELECT category, SUM(value) FROM table GROUP BY category`
- HAVING: `SELECT category, SUM(value) FROM table GROUP BY category HAVING SUM(value) > 100`
- Window functions (if applicable)

## Next Functions to Migrate

Good candidates for migration (in order of simplicity):
1. **COUNT** - Similar to SUM, straightforward accumulation
2. **AVG** - Builds on SUM pattern
3. **MIN/MAX** - Simple state tracking
4. **MEDIAN/MODE** - Already collect all values
5. **STRING_AGG** - Has parameters, good test for parameter handling

## Benefits of New System

1. **Extensibility**: New aggregates can be added without modifying core code
2. **Consistency**: Same pattern as regular SQL functions
3. **Testability**: Each aggregate is self-contained
4. **Type Safety**: Trait system ensures correct implementation
5. **Performance**: No change in performance, same accumulation pattern

## Future Enhancements

Once migration is complete:
- Add support for custom aggregate functions from plugins
- Implement parallel aggregation for large datasets
- Add aggregate function composition (e.g., AVG of SUMs)
- Support for approximate aggregates (HyperLogLog, etc.)