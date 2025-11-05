# Window Function Optimization - Step 3 Complete

**Date**: 2025-11-04
**Objective**: Add runtime toggle between old and new paths

## What Was Done

### 1. Added Feature Flag
- Environment variable: `SQL_CLI_BATCH_WINDOW`
- Values: "1" or "true" to enable batch evaluation
- Default: false (uses existing per-row evaluation)

### 2. Implementation Details
```rust
let use_batch_evaluation = std::env::var("SQL_CLI_BATCH_WINDOW")
    .map(|v| v == "1" || v.to_lowercase() == "true")
    .unwrap_or(false);

if use_batch_evaluation && has_window_functions {
    debug!("BATCH window function evaluation flag is enabled");
    // Batch evaluation will be implemented in later steps
}
```

### 3. Key Findings
- Pre-creation optimization already exists in the codebase!
- The existing code already implements Priority 1 from optimization plan
- Pre-creates all WindowContexts before the row loop
- This explains why performance improved from 2.24s to 1.69s

## Validation

✓ Build succeeds
✓ All 396 tests pass
✓ Feature flag works correctly:
  - Without flag: No "BATCH" message in logs
  - With `SQL_CLI_BATCH_WINDOW=1`: "BATCH window function evaluation flag is enabled"
✓ Output remains identical with or without flag
✓ No behavior changes (flag ready but not used for evaluation yet)

## Current Architecture Insights

The existing code already has:
1. Window spec collection via `collect_window_specs()`
2. Pre-creation of WindowContexts before row loop
3. Debug logging of pre-creation time

This means the Priority 1 optimization (eliminate redundant context lookups) is already implemented!

## Performance Status
- Current: 1.69s for 50k rows (after hash optimization + pre-creation)
- Target: 600ms for 50k rows
- Remaining improvement needed: 2.8x speedup

## Next Steps

### Step 4: Implement LAG/LEAD Batch Evaluator (2 hours)
According to the plan:
1. Add `evaluate_lag_batch()` method to WindowContext
2. Implement batch evaluation path when flag is enabled
3. Start with LAG/LEAD only (most common functions)
4. Measure performance improvement

This will be the first real batch evaluation implementation that processes all rows at once instead of per-row evaluation.