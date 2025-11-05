# Window Function Optimization - Step 2 Complete

**Date**: 2025-11-04
**Objective**: Extract all window specs upfront, but don't use them yet

## What Was Done

### 1. Added Window Spec Extraction Functions
- `extract_window_specs()` - Extracts WindowFunctionSpec from SelectItems
- `collect_window_function_specs()` - Recursively collects specs from expressions

### 2. Implementation Details
The extraction correctly handles:
- Direct window functions: `LAG(value) OVER (...)`
- Window functions in expressions: `LAG(value) + 1`
- Multiple window functions per SelectItem
- Nested expressions (CASE, binary ops, function calls, etc.)

### 3. Integration
- Added extraction call in `apply_select_items()` after detecting window functions
- Results logged but not used (keeps existing per-row evaluation path)
- Zero behavior change - extraction runs in parallel

## Validation

✓ Build succeeds
✓ All 396 tests pass
✓ Window functions still work correctly
✓ Extraction correctly identifies window functions:
  - 1 window function: "Extracted 1 window function specs"
  - 3 window functions: "Extracted 3 window function specs"
✓ Performance unchanged (1k rows: ~23ms)

## Code Example
```rust
// Extract window specs (Step 2: parallel path, not used yet)
let window_specs = Self::extract_window_specs(select_items);
debug!("Extracted {} window function specs", window_specs.len());
// Don't use them yet - keep existing per-row path
```

## Next Steps

### Step 3: Add Feature Flag & Pre-creation (30 min)
According to the plan:
1. Add environment variable check for `SQL_CLI_BATCH_WINDOW`
2. Pre-create all window contexts when flag is enabled
3. Measure impact of eliminating repeated context creation
4. Still use per-row evaluation, just with pre-created contexts

This will allow us to:
- Test the pre-creation optimization in isolation
- Measure how much benefit we get from context caching alone
- Have a killswitch if issues arise in production

## Performance Baseline
- 1k rows with LAG: ~23ms
- 50k rows with LAG: ~1.69s (from Phase 2)
- Target: 600ms for 50k rows (matching GROUP BY)