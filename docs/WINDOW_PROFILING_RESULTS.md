# Window Function Profiling Results

**Date**: 2025-11-02
**Phase**: Phase 1 - Coarse-Grained Timing

## Benchmark Results Summary

### 10k Rows Performance

| Test | Window Functions | Evaluation Time | % of Total |
|------|-----------------|-----------------|------------|
| LAG (no partition) | 1 | 167.56ms | 100.9% |
| LEAD (no partition) | 1 | 170.92ms | 102.2% |
| LAG + LEAD combined | 2 | 257.63ms | 99.8% |
| LAG with PARTITION BY | 1 | 166.12ms | 97.3% |
| ROW_NUMBER | 1 | 164.06ms | 101.9% |
| 3 functions + PARTITION | 3 | 361.84ms | 102.3% |

### 20k Rows Performance

| Test | Window Functions | Evaluation Time | % of Total |
|------|-----------------|-----------------|------------|
| LAG (no partition) | 1 | 336.24ms | 101.6% |
| LEAD (no partition) | 1 | 331.71ms | 99.7% |
| LAG + LEAD combined | 2 | 507.47ms | 99.7% |
| LAG with PARTITION BY | 1 | 330.99ms | 98.8% |
| ROW_NUMBER | 1 | 319.67ms | 99.9% |
| 3 functions + PARTITION | 3 | 684.72ms | 99.9% |

### 50k Rows Performance

| Test | Window Functions | Evaluation Time | % of Total |
|------|-----------------|-----------------|------------|
| LAG (no partition) | 1 | 880.04ms | 102.6% |
| LEAD (no partition) | 1 | 872.67ms | 104.0% |
| LAG + LEAD combined | 2 | 1372.48ms | ~100% |
| LAG with PARTITION BY | 1 | 867.67ms | 98.4% |
| ROW_NUMBER | 1 | 866.12ms | 103.1% |
| 3 functions + PARTITION | 3 | 1770.85ms | ~100% |

## Key Findings

### 1. Linear Scaling Confirmed
Window functions scale approximately O(n):
- 10k rows: ~170ms for single function
- 20k rows: ~335ms for single function (1.97x)
- 50k rows: ~875ms for single function (2.61x from 20k, 5.15x from 10k)

**Ratio**: Doubling rows approximately doubles execution time.

### 2. Window Functions Dominate Query Time
Window function evaluation accounts for **98-104% of total query time**:
- Nearly ALL query time is spent in window function evaluation
- Other operations (parsing, data loading, LIMIT) are negligible
- Clear bottleneck identified

### 3. Multiple Window Functions Add Up Linearly
- 1 function @ 50k: ~870ms
- 2 functions @ 50k: ~1370ms (1.58x)
- 3 functions @ 50k: ~1770ms (2.04x)

Each additional window function adds roughly the same overhead.

### 4. PARTITION BY Has Minimal Impact
Comparing LAG with and without PARTITION BY:
- 10k rows: 167ms (no partition) vs 166ms (partition) - **same**
- 20k rows: 336ms (no partition) vs 331ms (partition) - **same**
- 50k rows: 880ms (no partition) vs 868ms (partition) - **same**

**Conclusion**: Partitioning overhead is minimal. The bottleneck is in the evaluation itself, not partition creation.

### 5. LAG vs LEAD vs ROW_NUMBER - Similar Performance
At 50k rows:
- LAG: 880ms
- LEAD: 873ms
- ROW_NUMBER: 866ms

All three functions have nearly identical performance, suggesting the bottleneck is in the **common infrastructure** (WindowContext, iteration, etc.) rather than function-specific logic.

## Performance Comparison to Roadmap Data

From `docs/SQL_FEATURES_AND_ROADMAP.md`:

**50k rows (previous benchmarks)**:
- LAG function: 1.04s ✅ (we got 880ms - **15% faster**)
- LEAD function: 992ms ✅ (we got 873ms - **12% faster**)
- LAG + LEAD combined: 1.67s ✅ (we got 1.37s - **18% faster**)
- GROUP BY: ~600ms (comparison target)

**Window functions are still ~40% slower than GROUP BY at 50k rows.**

## Bottleneck Analysis

### What We Know:
1. ✅ Window function evaluation is the bottleneck (98%+ of query time)
2. ✅ Partitioning is NOT the bottleneck (same time with/without partition)
3. ✅ Scaling is O(n) - linear with row count
4. ✅ Each function adds roughly equal overhead
5. ✅ LAG/LEAD/ROW_NUMBER have similar performance (common infrastructure issue)

### What We DON'T Know Yet:
- ❓ Is the bottleneck in WindowContext creation or evaluation?
- ❓ How much time is spent in per-row iteration?
- ❓ Is there unnecessary data cloning?
- ❓ What's the breakdown of time in window_context.rs operations?

## Next Steps (Phase 2)

Based on these findings, Phase 2 should focus on:

1. **Add timing in `arithmetic_evaluator.rs`**:
   - Time WindowContext creation (get_or_create_window_context)
   - Time per-function evaluation (evaluate_window_function)
   - Identify if creation or evaluation is the bottleneck

2. **Add timing in `window_context.rs`**:
   - Time partition creation
   - Time row sorting within partitions
   - Time offset access (LAG/LEAD)
   - Time value extraction

3. **Profile a specific scenario**:
   - Run LAG @ 50k rows with RUST_LOG=debug
   - Analyze timing breakdown
   - Identify the single biggest time sink

**Target**: Reduce 50k LAG/LEAD from ~870ms to ~600ms (30% improvement)

## How to Reproduce

Run the benchmark script:
```bash
./scripts/benchmark_window_profiling.sh
```

Or test individual queries with execution plan:
```bash
./target/release/sql-cli data.csv \
  -q "SELECT *, LAG(value) OVER (ORDER BY id) FROM data" \
  --execution-plan
```

## Conclusion

Phase 1 profiling successfully:
- ✅ Confirmed window functions as the primary bottleneck
- ✅ Established baseline timing (10k, 20k, 50k rows)
- ✅ Ruled out partitioning as a bottleneck
- ✅ Identified need for deeper profiling in WindowContext

**Phase 2 is essential** to drill down into the WindowContext internals and find optimization opportunities.
