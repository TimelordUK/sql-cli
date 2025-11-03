# Window Function Phase 2 Profiling - Implementation Summary

**Date**: 2025-11-03
**Status**: Phase 2 Instrumentation Complete

## Summary

Phase 2 detailed timing instrumentation has been successfully added to track window function performance at a granular level. This builds on Phase 1's coarse-grained timing to provide deep insights into where time is spent during window function execution.

## What Was Added

### 1. **arithmetic_evaluator.rs** Timing

Located in `src/data/arithmetic_evaluator.rs`:

#### WindowContext Creation Timing (`get_or_create_window_context()` - lines 863-910)
```rust
- Cache hit/miss detection with timing
- DataView creation time
- WindowContext::new_with_spec call time
- Total creation time for cache misses
```

**Log Output**:
```
WindowContext cache hit for spec (lookup: XXμs)
  OR
WindowContext cache miss - creating new context
DataView creation took XXμs
WindowContext::new_with_spec took XX.XXms (rows: N)
Total WindowContext creation (cache miss) took XX.XXms
```

#### Per-Function Evaluation Timing (`evaluate_window_function()` - lines 912-1322)
```rust
- Registry-based function timing (transform + compute)
- Built-in function timing breakdown:
  - Total function time
  - Context lookup time
  - Evaluation time
- LAG/LEAD offset access timing
```

**Log Output**:
```
// For registry functions:
FUNCTION_NAME (registry) evaluation: total=XXμs, compute=XXμs

// For built-in functions:
FUNCTION_NAME (built-in) evaluation: total=XXμs, context=XXμs, eval=XXμs

// LAG/LEAD specific:
LAG offset access took XXμs (offset=1)
LEAD offset access took XXμs (offset=1)
```

### 2. **window_context.rs** Timing

Located in `src/sql/window_context.rs`:

#### WindowContext Creation (`new_with_spec()` - lines 135-258)
```rust
- Single partition path timing
- Multi-partition path timing
- Partition grouping time
- Partition sorting time
```

**Log Output**:
```
// Single partition:
Creating single partition (no PARTITION BY) for N rows
Single partition created in XX.XXms (1 partition, N rows)
WindowContext::new_with_spec (single partition) took XX.XXms total

// Multi-partition:
Creating partitions with PARTITION BY for N rows
Partition grouping took XX.XXms (M partitions created)
Partition sorting took XX.XXms (M partitions, ORDER BY: true/false)
Total partition creation took XX.XXms
WindowContext::new_with_spec (multi-partition) took XX.XXms total
```

#### Sorting Details (`sort_rows()` - lines 280-352)
```rust
- Sort preparation time (column index lookup)
- Actual sort operation time
```

**Log Output** (debug level):
```
Sort preparation took XXμs (N sort columns)
Actual sort operation took XXμs (N rows)

// For single partition:
Single partition sort took XX.XXms (N rows)
```

#### Offset Access (`get_offset_value()` - lines 354-396)
```rust
- Partition lookup time
- Offset navigation time
- Value access time
- Only logs if > 10μs to avoid spam
```

**Log Output** (debug level, only for slow calls):
```
get_offset_value slow: total=XXμs, partition_lookup=XXμs, offset_nav=XXμs, value_access=XXμs
```

## Timing Granularity

| Operation | Level | Unit | Purpose |
|-----------|-------|------|---------|
| WindowContext creation | info | ms | High-level context setup time |
| Partition grouping | info | ms | Cost of creating partitions |
| Partition sorting | info | ms | Cost of sorting within partitions |
| Function evaluation | info | μs | Per-function call overhead |
| Offset access (LAG/LEAD) | debug | μs | Per-row access cost (only if slow) |
| Sort preparation | debug | μs | Column lookup overhead |

## How to Use

### View Phase 1 (Coarse) Timing
```bash
./target/release/sql-cli data.csv \
  -q "SELECT *, LAG(value) OVER (ORDER BY id) FROM data" \
  --execution-plan
```

Output includes:
```
└─ WINDOW Evaluate 1 window function(s) (rows: N) [X.XXms]
    • Input: N rows
    • 1 window functions evaluated
    • Evaluation time: XX.XXms
```

### View Phase 2 (Detailed) Timing

**For TUI mode** (interactive):
- Press F5 to see debug output
- Detailed timing will be visible in debug panel

**For Non-Interactive mode** (requires tracing configuration):
```bash
# Note: Currently needs tracing subscriber configured for non-interactive mode
RUST_LOG=info ./target/release/sql-cli data.csv \
  -q "SELECT *, LAG(value) OVER (ORDER BY id) FROM data"
```

## Key Insights from Instrumentation

### What We Can Now Measure

1. **Context Creation Overhead**
   - Is WindowContext creation the bottleneck?
   - Are we creating contexts repeatedly (cache misses)?
   - How long does partitioning take vs. sorting?

2. **Per-Function Overhead**
   - How much time is spent in the function call itself vs. context lookup?
   - Are some functions slower than others?
   - Is offset access (LAG/LEAD) expensive?

3. **Partition Impact**
   - Does PARTITION BY add significant overhead?
   - Is sorting the bottleneck or partition creation?
   - How many partitions are being created?

4. **Scaling Characteristics**
   - How does timing scale with row count?
   - How does timing scale with partition count?
   - Where are the O(n log n) vs O(n) operations?

## Expected Bottlenecks to Investigate

Based on Phase 1 results showing window functions at ~870ms for 50k rows (vs GROUP BY at ~600ms), Phase 2 timing should help identify:

### Hypothesis 1: Context Creation Overhead
- **Check**: WindowContext::new_with_spec time
- **Look for**: High partition creation or sorting time
- **Fix if**: Context creation dominates (> 50% of total)

### Hypothesis 2: Per-Row Overhead
- **Check**: Per-function evaluation time multiplied by row count
- **Look for**: High per-function times (> 1μs per call)
- **Fix if**: Function overhead dominates

### Hypothesis 3: Repeated Context Creation
- **Check**: Cache hit/miss ratio
- **Look for**: Many "cache miss" messages
- **Fix if**: Same context being created multiple times

### Hypothesis 4: Partition Lookup Cost
- **Check**: get_offset_value partition_lookup time
- **Look for**: Slow HashMap lookups
- **Fix if**: HashMap access is slow (> 1μs)

## Next Steps

### Immediate (Testing)
1. ✅ Run benchmarks with Phase 1 execution plan output
2. ⏸️ Configure tracing for non-interactive mode (optional)
3. ⏸️ Run benchmarks with RUST_LOG=info for detailed Phase 2 output
4. ⏸️ Analyze timing data to identify specific bottlenecks

### Phase 3 (Optimization)
Based on Phase 2 findings, likely optimizations:
- If context creation is slow: Cache contexts more aggressively
- If per-row overhead is high: Batch operations or reduce per-call work
- If partition lookup is slow: Use different data structure
- If sorting is slow: Optimize sort comparisons or use pre-sorted data

## Comparison to Other Performance Work

### Similar to GROUP BY Optimization
- Phase 1 showed similar pattern: GROUP BY took ~600ms, window functions ~870ms
- GROUP BY likely has similar context creation + iteration overhead
- Potential to learn from GROUP BY optimizations

### Similar to LIKE Optimization
- LIKE had repeated regex compilation issue
- Window functions might have repeated context allocation
- Check for similar patterns in Phase 2 timing

## Files Modified

- `src/data/arithmetic_evaluator.rs` - Added context creation and function evaluation timing
- `src/sql/window_context.rs` - Added partition creation, sorting, and offset access timing
- `docs/WINDOW_PHASE2_PROFILING_SUMMARY.md` - This document

## Success Criteria

- ✅ All timing instrumentation added
- ✅ Code compiles and runs
- ✅ Phase 1 execution plan shows window function timing
- ⏸️ Phase 2 detailed logs visible (requires tracing config)
- ⏸️ Bottlenecks identified from timing data
- ⏸️ Optimization candidates prioritized

## Conclusion

Phase 2 profiling infrastructure is complete and ready for analysis. The instrumentation provides comprehensive timing at multiple levels:
- **Coarse** (execution plan): Total window function time
- **Medium** (info logs): Context creation, partitioning, sorting
- **Fine** (debug logs): Per-function, per-row operations

Next step is to run benchmarks and analyze the timing data to identify optimization opportunities that can bring window function performance closer to GROUP BY levels (target: reduce 50k row time from ~870ms to ~600ms).
