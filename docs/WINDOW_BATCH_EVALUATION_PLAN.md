# Window Function Batch Evaluation - Implementation Plan

**Date**: 2025-11-03
**Goal**: Implement batch evaluation to eliminate 50,000× per-row overhead
**Target**: Reduce 50k row execution from 1.69s to ~600ms (2.8x speedup)
**Strategy**: Incremental implementation with testing at each step

## Current Architecture

```rust
// Current: Per-row evaluation (SLOW - 50,000 calls)
for row_idx in visible_rows {
    for select_item in items {
        if is_window_function(select_item) {
            // This happens 50,000 times!
            let context = evaluator.get_or_create_window_context(&spec)?;  // 4μs
            let value = context.get_offset_value(row_idx, ...)?;           // 2μs
            row_data.push(value);
        }
    }
}
```

## Target Architecture

```rust
// Target: Batch evaluation (FAST - 1-2 calls per window spec)
// Phase 1: Extract all window specs and their column positions
let window_specs = extract_window_specs_with_positions(&select_items);

// Phase 2: Pre-create contexts once
for (spec, positions) in &window_specs {
    let context = evaluator.get_or_create_window_context(spec)?;  // Once!
    context_map.insert(spec.compute_hash(), (context, positions));
}

// Phase 3: Batch evaluate each window function
for (hash, (context, positions)) in context_map {
    let column_values = context.evaluate_all_rows(visible_rows)?;  // Batch!

    // Phase 4: Write results to output columns
    for (row_idx, value) in column_values.iter().enumerate() {
        for &col_position in positions {
            result_table[row_idx][col_position] = value.clone();
        }
    }
}

// Phase 5: Fill in non-window columns
for row_idx in visible_rows {
    for (col_idx, select_item) in non_window_items {
        result_table[row_idx][col_idx] = evaluate_scalar(select_item, row_idx)?;
    }
}
```

## Implementation Steps

### Step 0: Preparation (1 hour)
**Goal**: Establish safety net before any changes

**Tasks**:
1. Create comprehensive test suite for all window function types
   - LAG, LEAD, ROW_NUMBER, RANK, DENSE_RANK
   - With/without PARTITION BY
   - With/without ORDER BY
   - Different frame specifications
   - Edge cases: NULL values, empty partitions, single-row partitions

2. Add performance benchmarks
   - 10k, 50k, 100k row datasets
   - Capture baseline times for comparison

3. Document current code paths
   - Where window functions are detected
   - Where evaluation happens
   - Data flow diagram

**Files**:
- `tests/sql_examples/test_window_functions_comprehensive.sql` (NEW)
- `tests/integration/benchmark_window_functions.sh` (NEW)
- `docs/WINDOW_CURRENT_ARCHITECTURE.md` (NEW)

**Validation**: All tests pass, benchmarks establish baseline

---

### Step 1: Add Data Structures (No Behavior Change) (30 min)
**Goal**: Add new structures without changing any logic

**Tasks**:
1. Add `WindowFunctionSpec` struct to hold window function metadata
   ```rust
   struct WindowFunctionSpec {
       spec: WindowSpec,
       function_name: String,
       args: Vec<SqlExpression>,
       output_column_index: usize,
   }
   ```

2. Add `BatchWindowEvaluator` stub (empty for now)
   ```rust
   struct BatchWindowEvaluator {
       specs: Vec<WindowFunctionSpec>,
       contexts: HashMap<u64, Arc<WindowContext>>,
   }

   impl BatchWindowEvaluator {
       fn new() -> Self { ... }
       // Methods added in later steps
   }
   ```

**Files**:
- `src/data/batch_window_evaluator.rs` (NEW - mostly empty)
- `src/data/mod.rs` (add module declaration)

**Validation**:
- `cargo build --release` succeeds
- `cargo test` passes
- No runtime changes (new code not called yet)

---

### Step 2: Extract Window Specs (Parallel Path) (1 hour)
**Goal**: Extract all window specs upfront, but don't use them yet

**Tasks**:
1. Add `extract_window_specs()` function in `query_engine.rs`
   ```rust
   fn extract_window_specs(items: &[SelectItem]) -> Vec<WindowFunctionSpec> {
       let mut specs = Vec::new();
       for (idx, item) in items.iter().enumerate() {
           if let SelectItem::Expression { expr, .. } = item {
               Self::collect_window_function_specs(expr, idx, &mut specs);
           }
       }
       specs
   }
   ```

2. Call extraction but log and discard results (no behavior change)
   ```rust
   if has_window_functions {
       let window_specs = Self::extract_window_specs(&expanded_items);
       debug!("Extracted {} window function specs", window_specs.len());
       // Don't use them yet - keep existing per-row path
   }
   ```

**Files**:
- `src/data/query_engine.rs` (add extraction, don't use yet)

**Validation**:
- Build and test succeed
- Log output shows correct number of window specs extracted
- Performance unchanged (still using per-row path)

---

### Step 3: Feature Flag for Batch Evaluation (30 min)
**Goal**: Add runtime toggle between old and new paths

**Tasks**:
1. Add environment variable check
   ```rust
   let use_batch_evaluation = std::env::var("SQL_CLI_BATCH_WINDOW")
       .map(|v| v == "1" || v.to_lowercase() == "true")
       .unwrap_or(false);

   if use_batch_evaluation {
       debug!("Using BATCH window function evaluation");
       // New path (to be implemented)
   } else {
       debug!("Using PER-ROW window function evaluation");
       // Existing path (keep working)
   }
   ```

**Files**:
- `src/data/query_engine.rs` (add flag check)

**Validation**:
- Default behavior unchanged (flag off = per-row path)
- Can enable batch path with `SQL_CLI_BATCH_WINDOW=1` (but does nothing yet)

---

### Step 4: Implement LAG/LEAD Batch Evaluator (2 hours)
**Goal**: Get ONE window function type working in batch mode

**Tasks**:
1. Implement `WindowContext::evaluate_lag_batch()`
   ```rust
   impl WindowContext {
       pub fn evaluate_lag_batch(
           &self,
           visible_rows: &[usize],
           column_name: &str,
           offset: i64,
       ) -> Result<Vec<DataValue>> {
           let mut results = Vec::with_capacity(visible_rows.len());
           for &row_idx in visible_rows {
               results.push(self.get_offset_value(row_idx, -offset, column_name)?);
           }
           Ok(results)
       }
   }
   ```

2. Implement batch evaluation path for LAG/LEAD only
   ```rust
   if use_batch_evaluation {
       // Extract LAG/LEAD specs
       let lag_lead_specs: Vec<_> = window_specs.iter()
           .filter(|s| matches!(s.function_name.as_str(), "LAG" | "LEAD"))
           .collect();

       // Batch evaluate each LAG/LEAD
       for spec in lag_lead_specs {
           let context = evaluator.get_or_create_window_context(&spec.spec)?;
           let values = context.evaluate_lag_batch(visible_rows, ...)?;

           // Write to output table
           for (output_row_idx, value) in values.iter().enumerate() {
               result_rows[output_row_idx][spec.output_column_index] = value.clone();
           }
       }

       // Fall back to per-row for non-LAG/LEAD functions
       // ... existing logic for other columns
   }
   ```

**Files**:
- `src/sql/window_context.rs` (add batch methods)
- `src/data/query_engine.rs` (implement batch path for LAG/LEAD)

**Validation**:
- Default mode: all tests pass (using per-row path)
- Batch mode (`SQL_CLI_BATCH_WINDOW=1`): LAG/LEAD tests pass
- Compare outputs: `diff <(normal run) <(batch run)` should be identical
- Measure performance: LAG/LEAD queries should be faster in batch mode

---

### Step 5: Add ROW_NUMBER Batch Evaluator (1 hour)
**Goal**: Add second window function type to batch path

**Tasks**:
1. Implement `WindowContext::evaluate_row_number_batch()`
   ```rust
   pub fn evaluate_row_number_batch(
       &self,
       visible_rows: &[usize],
   ) -> Result<Vec<DataValue>> {
       let mut results = Vec::with_capacity(visible_rows.len());
       for &row_idx in visible_rows {
           let partition = self.get_partition_for_row(row_idx)?;
           let position_in_partition = partition.iter()
               .position(|&r| r == row_idx)
               .ok_or_else(|| ...)?;
           results.push(DataValue::Integer((position_in_partition + 1) as i64));
       }
       Ok(results)
   }
   ```

2. Add ROW_NUMBER to batch evaluation path

**Files**:
- `src/sql/window_context.rs` (add row_number batch method)
- `src/data/query_engine.rs` (extend batch path)

**Validation**:
- Batch mode: ROW_NUMBER tests pass
- Output identical to per-row mode
- Performance measurement

---

### Step 6: Add RANK/DENSE_RANK Batch Evaluators (1 hour)
**Goal**: Add remaining common window functions

**Tasks**:
1. Implement batch methods for RANK and DENSE_RANK
2. Add to batch evaluation path

**Files**:
- `src/sql/window_context.rs` (add rank batch methods)
- `src/data/query_engine.rs` (extend batch path)

**Validation**:
- Batch mode: RANK/DENSE_RANK tests pass
- Output identical to per-row mode
- Performance measurement

---

### Step 7: Performance Testing and Tuning (1 hour)
**Goal**: Verify we're achieving target performance

**Tasks**:
1. Run comprehensive benchmarks with batch mode enabled
2. Compare to baseline (per-row mode)
3. Profile any remaining bottlenecks
4. Optimize if needed

**Expected Results**:
- 50k rows: 1.69s → ~600ms (2.8x faster)
- 10k rows: 316ms → ~120ms (2.6x faster)
- 100k rows: ~3.4s → ~1.2s (2.8x faster)

**Validation**:
- Performance meets or exceeds targets
- All tests pass in both modes
- Outputs are byte-for-byte identical

---

### Step 8: Enable Batch by Default (30 min)
**Goal**: Switch default to batch mode, keep flag for rollback

**Tasks**:
1. Change default to batch mode:
   ```rust
   let use_batch_evaluation = std::env::var("SQL_CLI_BATCH_WINDOW")
       .map(|v| v != "0" && v.to_lowercase() != "false")
       .unwrap_or(true);  // Changed: default to TRUE
   ```

2. Update documentation about environment variable

**Files**:
- `src/data/query_engine.rs` (change default)
- `README.md` (document SQL_CLI_BATCH_WINDOW=0 for old behavior)

**Validation**:
- All tests pass with new default
- Can still rollback with `SQL_CLI_BATCH_WINDOW=0`
- User-facing performance improvement visible

---

### Step 9: Cleanup - Remove Per-Row Path (30 min)
**Goal**: Remove old code once batch is stable

**Tasks**:
1. Remove feature flag check
2. Remove old per-row evaluation path
3. Remove `SQL_CLI_BATCH_WINDOW` environment variable
4. Clean up dead code

**Files**:
- `src/data/query_engine.rs` (remove old path)
- `src/data/arithmetic_evaluator.rs` (may simplify)

**Validation**:
- All tests still pass
- Code is cleaner and easier to maintain
- Performance unchanged from Step 8

---

## Rollback Strategy

At each step, we can rollback safely:

- **Steps 0-2**: No behavior changes, just revert commits
- **Step 3**: Feature flag defaults to OFF, old path still works
- **Steps 4-6**: Feature flag OFF = old path, ON = new path
- **Step 7**: If performance doesn't meet targets, keep flag OFF
- **Step 8**: Set `SQL_CLI_BATCH_WINDOW=0` to use old path
- **Step 9**: Revert to Step 8 if issues found

## Testing Strategy

### After Each Step:
1. Run full test suite: `./run_all_tests.sh`
2. Run window function benchmarks
3. Compare outputs: batch vs per-row must be identical
4. Check for memory leaks or performance regressions

### Before Merging:
1. All tests pass in both modes (until Step 9)
2. Performance targets met (2.8x speedup for 50k rows)
3. No memory regressions
4. Documentation updated

## Risk Mitigation

### Low Risk Steps (0-3):
- Pure addition, no behavior change
- Can't break anything

### Medium Risk Steps (4-6):
- Behind feature flag
- Extensive testing
- Easy rollback

### Higher Risk Steps (7-9):
- Changing defaults
- Removing old code
- Requires confidence from Steps 4-6

## Time Estimate

| Step | Time | Cumulative |
|------|------|------------|
| 0. Preparation | 1h | 1h |
| 1. Data structures | 0.5h | 1.5h |
| 2. Extract specs | 1h | 2.5h |
| 3. Feature flag | 0.5h | 3h |
| 4. LAG/LEAD batch | 2h | 5h |
| 5. ROW_NUMBER batch | 1h | 6h |
| 6. RANK batch | 1h | 7h |
| 7. Performance tuning | 1h | 8h |
| 8. Enable by default | 0.5h | 8.5h |
| 9. Cleanup | 0.5h | 9h |

**Total**: ~9 hours over multiple sessions

## Success Criteria

- ✅ All existing tests pass
- ✅ 50k rows: 1.69s → ~600ms (2.8x faster)
- ✅ Output identical between batch and per-row modes
- ✅ No memory regressions
- ✅ Code maintainability improved (batch is cleaner conceptually)
- ✅ Safe rollback possible at every step

## Next Actions

1. Review this plan with user
2. Begin with Step 0 (Preparation)
3. Proceed incrementally, testing thoroughly at each step
4. Adjust plan based on learnings
