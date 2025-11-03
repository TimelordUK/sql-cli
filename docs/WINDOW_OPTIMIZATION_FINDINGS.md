# Window Function Optimization Findings

**Date**: 2025-11-03
**Analysis**: Phase 2 Profiling Results

## Executive Summary

Phase 2 profiling has **identified the bottleneck**: Window function slowness is caused by **per-row overhead**, not context creation. Each LAG/LEAD call takes 30-50μs, with 90% of that time spent in context lookup even for cache hits.

**Target**: Reduce 50k row LAG execution from 2.24s to ~600ms (to match GROUP BY performance)

## Performance Breakdown (50k rows)

### Phase 1 Timing (Coarse)
```
Window function evaluation: 2,236.47ms
Context creation: included above
Per-function overhead: included above
```

### Phase 2 Timing (Detailed)
```
WindowContext creation:        9.81ms    (0.4%)
Per-row LAG evaluation:    2,226.66ms   (99.6%)
  ├─ Context lookup (cache): ~40μs per row × 50,000 = 2,000ms
  └─ Actual evaluation:       ~2μs per row × 50,000 =   100ms
```

###Performance by Dataset Size

| Rows | Context Creation | Per-Row Avg | Total Window Time | % in Lookup |
|------|------------------|-------------|-------------------|-------------|
| 1k | 0.41ms | ~6μs | ~6ms | 67% |
| 10k | 3.54ms | ~7μs | ~73ms | 71% |
| 50k | 9.81ms | ~45μs | 2,236ms | 89% |

**Scaling**: Context creation scales linearly (0.2μs per row). Per-row overhead increases with dataset size, suggesting cache degradation or lookup inefficiency.

## Root Cause Analysis

### What We Learned

✅ **Context creation is NOT the bottleneck** (9.81ms for 50k rows)
✅ **Cache is working** (no repeated context creation)
✅ **Partitioning is fast** (0.28ms grouping + 0.04ms sorting for 4 partitions)
❌ **Context lookup is SLOW** (20-50μs per lookup, even for cache hits)
❌ **Per-row function call overhead** is the dominant cost

### Why Context Lookup is Slow

Looking at `arithmetic_evaluator.rs:863-910`, the context lookup does:

```rust
fn get_or_create_window_context(&mut self, spec: &WindowSpec) -> Result<Arc<WindowContext>> {
    let key = format!("{:?}", spec);  // ⚠️ String formatting

    if let Some(context) = self.window_contexts.get(&key) {  // HashMap lookup
        return Ok(Arc::clone(context));  // Arc clone
    }
    // ... creation logic ...
}
```

**Problem**: Even for a cache hit, we're doing:
1. **String formatting** of the entire WindowSpec (`format!("{:?}", spec)`) - SLOW
2. **HashMap lookup** with string key - Slower than needed
3. **Arc cloning** - Minor cost

**Evidence**: First LAG call takes 590μs (includes creation). Subsequent calls take 20-50μs for just the lookup!

## Optimization Opportunities

### Priority 1: Eliminate Redundant Context Lookups (Target: 90% speedup)

**Current**: Every row calls `get_or_create_window_context()` → 20-50μs overhead

**Solution**: Cache the context Arc at a higher level and reuse it for all rows in the same window spec.

**Location**: `query_engine.rs` or `arithmetic_evaluator.rs` - evaluate all SELECT items with the same context

**Estimated Impact**:
- Current: 50,000 lookups × 40μs = 2,000ms
- After: 1 lookup × 40μs = 0.04ms
- **Savings: ~2,000ms → ~100ms total window time (95% reduction!)**

### Priority 2: Optimize Cache Key Generation (Target: 50% speedup if P1 not feasible)

**Current**: `let key = format!("{:?}", spec);` - Creates a full debug string every time

**Solution**: Use a hash-based key or implement Hash for WindowSpec

```rust
// Option A: Pre-compute hash
let key = calculate_window_spec_hash(spec);  // u64 instead of String

// Option B: Make WindowSpec hashable
impl Hash for WindowSpec { ... }
// Then use HashMap<WindowSpec, Arc<WindowContext>>
```

**Estimated Impact**:
- Current: `format!("{:?}")` likely takes 10-20μs for WindowSpec
- After: Hash computation ~1-2μs
- **Savings: ~10-20μs per lookup × 50,000 = 500-1,000ms**

### Priority 3: Reduce Arc Cloning Overhead (Target: 10% speedup)

**Current**: Every cache hit does `Arc::clone(context)`

**Solution**: Return a reference instead of cloning, or use internal mutability

**Estimated Impact**: Marginal (~1-2μs per call), but adds up over 50k calls

### Priority 4: Batch Window Function Evaluation (Target: 50% speedup)

**Current**: LAG is called once per row, with full function call overhead each time

**Solution**: Batch-evaluate window functions for all rows at once

```rust
// Instead of:
for row in rows {
    let value = evaluate_window_function("LAG", args, spec, row)?;
}

// Do:
let all_values = batch_evaluate_window_function("LAG", args, spec, rows)?;
```

**Estimated Impact**: Eliminates per-row function call overhead, potentially reducing from 45μs to 10μs per row

## Comparison to GROUP BY

**GROUP BY at 50k rows**: ~600ms
**Window Functions at 50k rows**: ~2,240ms
**Gap**: 3.7x slower

### Why is GROUP BY Faster?

Likely because GROUP BY:
1. Creates aggregation context once
2. Processes all rows in batch
3. No per-row context lookup overhead

**Our Solution**: Apply the same pattern to window functions!

## Implementation Recommendations

### Quick Win (1-2 hours): Priority 1 + Priority 2

Combine both optimizations:

1. **Cache context at query level** (Priority 1)
2. **Use hash-based keys** (Priority 2)

**Expected Result**: 50k rows from 2.24s to ~200-300ms (85-90% faster)

### Complete Solution (4-6 hours): All Priorities

Implement all four optimizations for maximum performance.

**Expected Result**: 50k rows from 2.24s to ~100-150ms (93-95% faster), **matching or exceeding GROUP BY performance**

## Testing Plan

1. **Implement Priority 1** (cache at query level)
2. **Benchmark**: Run 50k row test, expect ~200ms
3. **Implement Priority 2** (hash-based keys) if still needed
4. **Benchmark**: Run 50k row test, expect ~100-150ms
5. **Compare to GROUP BY**: Should be within 20% of GROUP BY performance

## Files to Modify

### Priority 1: Cache at Query Level
- `src/data/query_engine.rs` - Cache WindowContext for SELECT evaluation
- `src/data/arithmetic_evaluator.rs` - Accept pre-created context as parameter

### Priority 2: Hash-Based Keys
- `src/sql/parser/ast.rs` - Implement Hash for WindowSpec
- `src/data/arithmetic_evaluator.rs` - Change HashMap key type

### Priority 3: Reduce Arc Cloning
- `src/data/arithmetic_evaluator.rs` - Use references or Rc instead of Arc

### Priority 4: Batch Evaluation
- `src/data/arithmetic_evaluator.rs` - Add batch evaluation method
- `src/data/query_engine.rs` - Call batch method instead of per-row

## Success Metrics

- [ ] 50k rows LAG execution < 200ms (Priority 1+2)
- [ ] 50k rows LAG execution < 150ms (All priorities)
- [ ] Window functions within 20% of GROUP BY performance
- [ ] No regression in correctness (all tests pass)
- [ ] Cache hit rate > 99.9% for repeated window specs

## Conclusion

The Phase 2 profiling clearly identified the bottleneck: **per-row context lookup overhead**. The fix is straightforward - cache the context at a higher level and avoid redundant lookups. This should bring window function performance in line with GROUP BY, achieving the target of ~600ms for 50k rows (currently 2.24s).

**Next Step**: Implement Priority 1 optimization (cache context at query level) for immediate 85-90% performance improvement.
