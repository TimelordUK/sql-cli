# SQL CLI Baseline Performance Metrics

**Date**: September 18, 2025
**Version**: Pre-optimization baseline
**Test Environment**: Release build (`cargo build --release`)

## Executive Summary

Initial performance benchmarking reveals good performance for basic operations but significant optimization opportunities for complex queries, particularly those involving LIKE patterns and aggregations. The system handles datasets up to 50K rows with acceptable performance for most operations.

## Key Findings

### Performance Characteristics

1. **Excellent Performance** (< 20ms):
   - Simple SELECT queries: 0.15-16ms
   - Indexed lookups (WHERE id > N): 2-18ms
   - Projections (column selection): 0.15-3.6ms
   - BETWEEN range queries: 0.29-6ms

2. **Good Performance** (20-100ms):
   - Multi-condition WHERE clauses: 2.8-55ms
   - String equality filters: 2.6-52ms
   - IN operator queries: 5.6-111ms (scales linearly)

3. **Poor Performance** (> 100ms):
   - LIKE pattern matching: 284ms-5724ms (O(n²) behavior)
   - GROUP BY aggregations: 594ms-3163ms (scales poorly)

### Scalability Analysis

| Operation | 1K rows | 10K rows | 20K rows | Scale Factor |
|-----------|---------|----------|----------|--------------|
| Full Scan | 0.32ms | 4.62ms | 5.43ms | Linear/Sub-linear |
| Simple Filter | 0.32ms | 2.87ms | 6.49ms | Linear |
| LIKE Pattern | 285ms | 2854ms | 5724ms | **O(n²)** ⚠️ |
| IN Operator | 5.65ms | 56.89ms | 111ms | Linear |
| GROUP BY | ~60ms | ~595ms | ~1234ms | **Super-linear** ⚠️ |

## Bottleneck Analysis

### Critical Issues

1. **LIKE Pattern Matching**
   - Current: O(n²) performance
   - Cause: Row-by-row evaluation with string allocation
   - Impact: 5.7 seconds for 20K rows
   - Fix: Compile regex once, use string interning

2. **GROUP BY Operations**
   - Current: 3+ seconds for 50K rows
   - Cause: Inefficient hash map operations, excessive cloning
   - Impact: Unusable for interactive queries > 30K rows
   - Fix: Pre-sort optimization, better hash functions

3. **Throughput Calculation**
   - Current: Shows 0 rows/sec
   - Cause: Metrics collection bug
   - Fix: Correct timing measurement

### Memory Usage Patterns

Based on the data generator outputs:
- Narrow tables (3 cols): ~0.5MB per 10K rows
- Mixed tables (6 cols): ~1.2MB per 10K rows
- Wide tables (20 cols): ~4MB per 10K rows
- Very wide tables (50 cols): ~10MB per 10K rows

## Performance Targets vs Actual

| Query Type | Target (100K) | Current (50K) | Gap |
|------------|---------------|---------------|-----|
| Full Scan | < 100ms | ~10ms | ✅ Exceeds |
| Filtered Scan | < 50ms | ~18ms | ✅ Exceeds |
| Simple Aggregation | < 200ms | ~3200ms | ❌ 16x slower |
| GROUP BY | < 300ms | ~3200ms | ❌ 10x slower |
| ORDER BY | < 500ms | ~20ms | ✅ Exceeds |
| Window Functions | < 1s | N/A | ⚠️ Not implemented |

## Optimization Priorities

### Immediate (Quick Wins)

1. **Fix LIKE Performance**
   - Compile patterns once
   - Use Boyer-Moore for string search
   - Expected improvement: 10-50x

2. **Fix Throughput Metrics**
   - Correct timing in MetricsCollector
   - Add proper row counting

3. **String Interning**
   - Intern repeated strings (categories, statuses)
   - Expected memory reduction: 30-50%

### Short Term (1-2 days)

1. **GROUP BY Optimization**
   - Pre-sort when beneficial
   - Better hash map sizing
   - Reduce allocations
   - Expected improvement: 5-10x

2. **Expression Caching**
   - Cache computed expressions
   - Avoid re-evaluation
   - Expected improvement: 2-3x

### Medium Term (3-5 days)

1. **Columnar Evaluation**
   - Process columns instead of rows
   - Better cache locality
   - Expected improvement: 2-4x

2. **Simple Indexing**
   - Hash indexes for equality
   - B-tree for ranges
   - Expected improvement: 10-100x for indexed queries

## Benchmark Query Details

### Progressive Benchmark Queries

- **prog_query_0**: Full table scan with LIMIT
- **prog_query_1**: Simple WHERE filter
- **prog_query_2**: GROUP BY with aggregations (main bottleneck)
- **prog_query_3**: ORDER BY with LIMIT
- **prog_query_4**: Window functions (currently failing - RANK not implemented)

## Recommendations

1. **Immediate Focus**: Fix LIKE pattern matching - it's unusable at current speeds
2. **Next Priority**: Optimize GROUP BY operations for interactive use
3. **Infrastructure**: Fix metrics collection to show actual throughput
4. **Future**: Implement missing window functions (RANK, DENSE_RANK)

## Test Configuration

```bash
# Progressive benchmark (10K increments to 50K)
./target/release/sql-cli --benchmark --progressive \
  --increment 10000 --max-rows 50000 \
  --csv benchmark_results.csv --report benchmark_report.md

# Category-specific benchmarks
./target/release/sql-cli --benchmark --category basic \
  --sizes 1000,5000,10000,20000 --csv basic_benchmark.csv
```

## Next Steps

1. Implement string interning for categorical data
2. Fix LIKE operator performance
3. Optimize GROUP BY with pre-sorting
4. Add missing window functions
5. Re-run benchmarks after each optimization
6. Target: All queries < 500ms at 100K rows

## Appendix: Successful Operations

The following operations already meet or exceed performance targets:

- Simple projections (SELECT specific columns)
- Basic WHERE filters
- ORDER BY operations
- LIMIT clauses
- Calculated columns
- BETWEEN ranges
- Basic arithmetic expressions

These provide a solid foundation, with optimization efforts needed primarily in pattern matching and aggregation operations.