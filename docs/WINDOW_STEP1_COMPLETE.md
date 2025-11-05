# Window Function Optimization - Step 1 Complete

**Date**: 2025-11-04
**Objective**: Add batch evaluation data structures without changing behavior

## What Was Done

### 1. Created BatchWindowEvaluator Module
- Added `src/data/batch_window_evaluator.rs` with:
  - `WindowFunctionSpec` struct to hold window function metadata
  - `BatchWindowEvaluator` struct (stub for now)
  - Module declaration in `src/data/mod.rs`

### 2. WindowFunctionSpec Structure
```rust
pub struct WindowFunctionSpec {
    pub spec: WindowSpec,
    pub function_name: String,
    pub args: Vec<SqlExpression>,
    pub output_column_index: usize,
}
```

This structure captures all metadata needed to:
- Identify the window specification (PARTITION BY, ORDER BY, frame)
- Know which function to call (LAG, LEAD, ROW_NUMBER, etc.)
- Store the function arguments
- Know where to place results in the output table

### 3. BatchWindowEvaluator Structure
```rust
pub struct BatchWindowEvaluator {
    specs: Vec<WindowFunctionSpec>,
    contexts: HashMap<u64, Arc<WindowContext>>,
}
```

This will manage:
- Collection of all window functions in the query
- Pre-created window contexts to avoid repeated lookups
- Batch evaluation logic (to be added in later steps)

## Validation

✓ `cargo build --release` - Succeeds with existing warnings
✓ `cargo test` - All 396 tests pass
✓ Window functions still work - Verified with LAG example
✓ No runtime changes - New code not called yet

## Current Performance Baseline

From Phase 2 work:
- 50k rows with LAG: 1.69s (down from 2.24s after hash optimization)
- Per-row overhead: ~34μs
- Target: 600ms (matching GROUP BY performance)

## Next Steps

According to the batch evaluation plan:

### Step 2: Extract Window Specs (1 hour)
- Add `extract_window_specs()` function in `query_engine.rs`
- Recursively collect all window function specs from SelectItems
- Still run old code path, just collect specs in parallel

### Step 3: Add Feature Flag & Pre-creation (30 min)
- Add `--enable-batch-windows` CLI flag
- Pre-create all window contexts upfront
- Measure impact of eliminating repeated context creation

### Steps 4-9: Implement Batch Evaluation
- Add batch evaluation methods to WindowContext
- Switch to new evaluation path
- Remove old per-row evaluation code

## Risk Assessment

✓ Step 1 - Zero risk (no behavior change)
? Step 2 - Low risk (parallel collection only)
? Step 3 - Low risk (feature flagged)
? Steps 4-9 - Medium risk (core logic change)

## Recommendation

Proceed to Step 2: Implement `extract_window_specs()` function to start collecting window function metadata during query planning.