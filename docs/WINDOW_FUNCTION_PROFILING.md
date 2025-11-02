# Window Function Performance Profiling Plan

## Current Performance (Benchmarks)

### 10k rows
- LAG + LEAD combined: **321ms**
- LAG with PARTITION BY: **111ms**
- ROW_NUMBER: **161ms**

### 20k rows
- LAG function: **402ms**
- LEAD function: **400ms**
- LAG + LEAD combined: **648ms**
- LAG with PARTITION BY: **221ms**
- ROW_NUMBER: **322ms**

### 50k rows (Performance bottleneck)
- **LAG function: 1.04s** ⚠️
- **LEAD function: 992ms** ⚠️
- **LAG + LEAD combined: 1.67s** ⚠️
- LAG with PARTITION BY: 565ms
- ROW_NUMBER: 839ms

**Comparison** (50k rows):
- GROUP BY with multiple aggregates: **965ms** ✅ (faster than LAG/LEAD)
- Hash JOIN (equality): **6ms** ✅✅ (very efficient!)

## Performance Gap Analysis

Window functions are ~2x slower than GROUP BY operations:
- LAG/LEAD: ~1 second for 50k rows
- GROUP BY: ~500-600ms for 50k rows
- Hash JOIN: Negligible (~6ms)

**Scaling**: Window functions appear to scale linearly (O(n)) but with higher constant factor than GROUP BY.

## Current Profiling Capabilities

### ✅ Already Have Timing For:
1. **CTE execution** - `cte_start.elapsed()`
2. **JOIN operations** - `join_start.elapsed()`
3. **WHERE filtering** - `filter_start.elapsed()`
4. **GROUP BY aggregation** - `group_start.elapsed()`
5. **DISTINCT** - `distinct_start.elapsed()`
6. **ORDER BY sorting** - `sort_start.elapsed()`
7. **Set operations** (UNION/INTERSECT/EXCEPT) - `op_start.elapsed()`

### ❌ Missing Timing For:
1. **Window function evaluation** - NO TIMING CURRENTLY
2. **WindowContext creation** - NO TIMING
3. **Partition computation** - NO TIMING
4. **LAG/LEAD offset access** - NO TIMING
5. **Per-row window function calls** - NO TIMING

## Proposed Profiling Additions

### Phase 1: Coarse-Grained Timing (Quick Win - 1 hour)

Add top-level timing for window function operations in query_engine.rs:

```rust
// Around line 1200+ (after GROUP BY, before SELECT item evaluation)
let window_start = Instant::now();

// ... window function evaluation ...

info!(
    "Window function evaluation took {:.2}ms for {} rows",
    window_start.elapsed().as_secs_f64() * 1000.0,
    filtered_table.len()
);
```

**Location**: `src/data/query_engine.rs` in `execute()` method
**Benefit**: Confirms window functions are the bottleneck
**Effort**: 15 minutes

### Phase 2: WindowContext Profiling (Medium - 2 hours)

Add timing breakdown in `arithmetic_evaluator.rs`:

1. **WindowContext creation timing**:
```rust
// In get_or_create_window_context()
let context_start = Instant::now();
let context = WindowContext::new_with_spec(...)?;
debug!(
    "WindowContext creation took {:.2}ms (partitions: {}, rows: {})",
    context_start.elapsed().as_secs_f64() * 1000.0,
    context.partition_count(),
    table.len()
);
```

2. **Per-function timing**:
```rust
// In evaluate_window_function()
let func_start = Instant::now();
// ... evaluate LAG/LEAD/etc ...
debug!(
    "{} evaluation took {:.2}μs",
    name_upper,
    func_start.elapsed().as_micros()
);
```

**Location**: `src/data/arithmetic_evaluator.rs`
**Benefit**: Identifies if creation or evaluation is slow
**Effort**: 30 minutes

### Phase 3: Detailed Window Context Timing (Fine-Grained - 4 hours)

Add timing in `window_context.rs` for key operations:

1. **Partition creation**:
```rust
// In new_with_spec()
let partition_start = Instant::now();
// ... partition creation logic ...
debug!(
    "Partition creation: {:.2}ms ({} partitions, {} rows)",
    partition_start.elapsed().as_secs_f64() * 1000.0,
    partitions.len(),
    total_rows
);
```

2. **Partition sorting**:
```rust
// In sort_rows()
let sort_start = Instant::now();
// ... sorting logic ...
debug!(
    "Partition sorting: {:.2}ms ({} rows)",
    sort_start.elapsed().as_secs_f64() * 1000.0,
    rows.len()
);
```

3. **LAG/LEAD offset access**:
```rust
// In get_offset_value()
let offset_start = Instant::now();
// ... offset logic ...
trace!(
    "Offset access: {:.2}μs (row: {}, offset: {})",
    offset_start.elapsed().as_micros(),
    row_index,
    offset
);
```

**Location**: `src/sql/window_context.rs`
**Benefit**: Pinpoints exact bottleneck (partition creation, sorting, or access)
**Effort**: 1-2 hours

### Phase 4: Execution Plan Integration (Polish - 2 hours)

Add window function steps to execution plan output:

```rust
// In query_engine.rs
if let Some(ref mut plan) = execution_plan {
    plan.add_step(
        StepType::WindowFunction,
        "Evaluate window functions",
        format!("{} window functions, {} partitions",
                window_func_count, partition_count),
        window_duration
    );
}
```

**Update ExecutionPlan enum**: Add `WindowFunction` variant to `StepType`
**Benefit**: Shows window timing in `--execution-plan` output
**Effort**: 30 minutes

## Recommended Execution Order

### Week 1: Monday-Tuesday (Investigation)
1. ✅ **Phase 1** (15 min) - Add coarse-grained timing - **COMPLETED**
   - Added `contains_window_function()` helper in `query_engine.rs`
   - Modified `apply_select_items()` to detect and time window functions
   - Added execution plan step with `StepType::WindowFunction`
   - Logs total window function evaluation time and count
2. ⏸️ **Phase 2** (30 min) - Add WindowContext profiling - **TODO**
3. ⏸️ **Phase 3** (2 hours) - Add detailed timing - **TODO**
4. 🔬 **Run benchmarks** with `RUST_LOG=debug` to collect timing data
5. 📊 **Analyze** where time is spent

### Week 1: Wednesday-Thursday (Optimization)
Based on profiling data, likely hotspots to investigate:

#### Hotspot 1: Partition Creation
- Are we creating too many partitions?
- Can we cache partition boundaries?
- Memory allocation patterns?

#### Hotspot 2: Offset Access (LAG/LEAD)
- Are we cloning data unnecessarily?
- Can we use indices instead of cloning?
- Is the offset lookup O(1) or O(n)?

#### Hotspot 3: Per-Row Overhead
- Are we re-computing WindowContext for every row?
- Can we batch window function calls?
- Is there redundant work per row?

### Week 1: Friday (Validation)
4. ✅ **Phase 4** (30 min) - Add to execution plan
5. 📈 **Re-benchmark** to measure improvements
6. ✅ **Document** findings and optimizations

## Expected Outcomes

### Immediate (after profiling)
- Clear understanding of where time is spent
- Data-driven optimization targets
- Better --execution-plan output

### After Optimization (Target)
- **LAG/LEAD at 50k rows**: Reduce from 1.04s to ~600ms (40% improvement)
- **WindowContext creation**: Should be sub-100ms
- **Per-row overhead**: Should be sub-microseconds

### Success Metrics
- [ ] LAG/LEAD comparable to GROUP BY performance (~500-600ms at 50k rows)
- [ ] Clear timing breakdown in `--execution-plan`
- [ ] No more than 20% overhead from window function infrastructure
- [ ] Linear scaling confirmed (O(n) with acceptable constant factor)

## Debug Commands for Testing

```bash
# Basic window function with timing
RUST_LOG=debug ./target/release/sql-cli data.csv \
  -q "SELECT *, LAG(value) OVER (ORDER BY id) as prev FROM data" \
  --execution-plan

# Partitioned window function
RUST_LOG=debug ./target/release/sql-cli data.csv \
  -q "SELECT *, LAG(value) OVER (PARTITION BY category ORDER BY id) as prev FROM data" \
  --execution-plan

# Multiple window functions (worst case)
RUST_LOG=debug ./target/release/sql-cli data.csv \
  -q "SELECT *,
      LAG(value) OVER (ORDER BY id) as prev,
      LEAD(value) OVER (ORDER BY id) as next,
      ROW_NUMBER() OVER (ORDER BY id) as rn
      FROM data" \
  --execution-plan

# Large dataset benchmark
./target/release/sql-cli large_data.csv \
  -q "SELECT *, LAG(sales) OVER (PARTITION BY region ORDER BY date) as prev_sales FROM data" \
  --execution-plan
```

## Files to Modify

1. **src/data/query_engine.rs** - Add coarse window function timing (~line 1200+)
2. **src/data/arithmetic_evaluator.rs** - Add WindowContext & function timing
3. **src/sql/window_context.rs** - Add detailed partition/sort/access timing
4. **src/execution_plan.rs** - Add WindowFunction step type
5. **docs/WINDOW_FUNCTION_OPTIMIZATION.md** - Document findings (new file)

## Success Criteria

After completing profiling and optimization:
- ✅ Window functions show explicit timing in logs
- ✅ `--execution-plan` includes window function steps
- ⏸️ LAG/LEAD performance within 20% of GROUP BY
- ⏸️ Clear documentation of bottlenecks and solutions
- ⏸️ Benchmark improvements validated

## Implementation Notes

### Phase 1: Coarse-Grained Timing (COMPLETED - 2025-11-02)

**Changes Made**:
1. **Added `contains_window_function()` helper** in `src/data/query_engine.rs:405-465`
   - Recursively checks if an expression tree contains window functions
   - Similar to existing `contains_unnest()` helper
   - Handles all expression variants: BinaryOp, Not, FunctionCall, CaseExpression, etc.

2. **Modified `apply_select_items()` signature** in `src/data/query_engine.rs:1589`
   - Added `plan: &mut ExecutionPlanBuilder` parameter
   - Updated caller at line 1330

3. **Added window function detection and timing** in `src/data/query_engine.rs:1606-1630`
   - Detects window functions in select items
   - Counts window functions for reporting
   - Starts timing if window functions are present

4. **Added timing log and execution plan step** in `src/data/query_engine.rs:1811-1835`
   - Logs: "Window function evaluation took X.XXms for N rows (M window functions)"
   - Creates execution plan step with:
     - StepType::WindowFunction (already existed in execution_plan.rs:33)
     - Input/output row counts
     - Window function count
     - Evaluation time

**Test Results**:
- 10 rows, 1 window function: ~0.28ms
- 10 rows, 3 window functions: ~0.49ms
- 24 rows, 2 window functions (partitioned): ~0.67ms

**Example Output**:
```
└─ WINDOW Evaluate 3 window function(s) (rows: 10) [0.001ms]
    • Input: 10 rows
    • 3 window functions evaluated
    • Evaluation time: 0.490ms
```

**Benefits Achieved**:
- ✅ Window functions now visible in `--execution-plan` output
- ✅ Baseline timing established for future optimization
- ✅ Can confirm window functions are the bottleneck when present

**Next Steps for Phase 2**:
- Add timing in `src/data/arithmetic_evaluator.rs` for WindowContext creation
- Add per-function timing (LAG vs LEAD vs ROW_NUMBER)
- Identify if creation or evaluation is the bottleneck
