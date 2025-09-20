# SQL CLI Performance Benchmarks

## Key Performance Highlights

SQL CLI delivers exceptional performance across various operations, with particularly impressive results for pattern matching and window functions.

## LIKE Pattern Matching (Regex Cached) ⚡

One of SQL CLI's standout features - **blazing fast pattern matching** thanks to regex caching:

| Row Count | LIKE with wildcards | Multiple LIKE conditions | Time Complexity |
|-----------|-------------------|-------------------------|-----------------|
| 10,000    | **~30ms**         | **~33ms**               | O(n)            |
| 50,000    | **~110ms**        | **~108ms**              | O(n)            |
| 100,000   | **~240ms**        | **~244ms**              | O(n)            |

**Key insight**: Linear scaling with near-zero overhead for complex patterns due to compiled regex caching!

## Window Functions (LAG/LEAD)

Minimal overhead for analytical queries:

| Row Count | LAG (1000 rows) | LEAD (1000 rows) | LAG+LEAD Combined | With PARTITION BY |
|-----------|-----------------|------------------|-------------------|-------------------|
| 10,000    | **152ms**       | **155ms**        | **275ms**         | **99ms**          |
| 50,000    | **104ms**       | **113ms**        | **107ms**         | **112ms**         |
| 100,000   | **1.5s**        | **1.5s**         | N/A               | **954ms**         |

**Note**: LAG/LEAD performance scales with result set size, not total table size.

## GROUP BY Performance (v1.48.0+)

Benchmark results on typical hardware (measured on Linux WSL2):

### Simple GROUP BY (100 unique groups)

| Row Count | Time (before) | Time (after Phase 1) | Improvement |
|-----------|--------------|---------------------|-------------|
| 1,000     | ~55ms        | ~55ms               | baseline    |
| 5,000     | ~250ms       | ~250ms              | baseline    |
| 10,000    | ~510ms       | ~510ms              | baseline    |
| 20,000    | ~1,050ms     | ~1,050ms            | baseline    |
| 50,000    | ~2,700ms     | ~2,700ms            | baseline    |
| 100,000   | ~5,100ms     | TBD                 | TBD         |

### Multi-column GROUP BY

| Row Count | Time (before) | Time (after Phase 1) | Improvement |
|-----------|--------------|---------------------|-------------|
| 1,000     | ~115ms       | ~115ms              | baseline    |
| 5,000     | ~570ms       | ~570ms              | baseline    |
| 10,000    | ~1,100ms     | ~1,100ms            | baseline    |
| 20,000    | ~2,200ms     | ~2,200ms            | baseline    |
| 50,000    | ~5,100ms     | ~5,100ms            | baseline    |
| 100,000   | ~10,400ms    | TBD                 | TBD         |

## Running Benchmarks

### Quick GROUP BY Test
```bash
./scripts/simple_group_by_test.sh
```

### Comprehensive Benchmark
```bash
./scripts/benchmark_group_by.sh
```

### Built-in Benchmarks
```bash
# Run all benchmarks
./target/release/sql-cli --benchmark

# Run GROUP BY specific benchmarks
./target/release/sql-cli --benchmark group-by 10000 20000 50000

# Progressive benchmarks (10k increments)
./target/release/sql-cli --benchmark --progressive
```

## Optimization Phases

### Phase 1: Quick Wins (Completed)
- ✅ Replaced HashMap with FxHashMap for faster hashing
- ✅ Added cardinality estimation for hash table pre-sizing
- ✅ Created benchmark infrastructure

**Result**: Foundation laid for future optimizations

### Phase 2: Algorithmic Improvements (Planned)
- ⏳ Streaming aggregation for sorted data
- ⏳ Type-specialized aggregators
- ⏳ Lazy aggregation for unused columns

**Expected**: 2-3x improvement

### Phase 3: Advanced Optimizations (Planned)
- ⏳ Parallel aggregation with Rayon
- ⏳ Adaptive strategy selection
- ⏳ Memory layout optimization

**Expected**: Additional 2-4x improvement

## Other Performance Features

### Regex Caching
The engine maintains a cache of compiled regular expressions across the query pipeline, providing massive performance gains for LIKE operations and pattern matching.

### Query Categories Benchmarked

- **Basic**: Full scan, filtering, projections
- **Aggregation**: GROUP BY, HAVING, COUNT DISTINCT
- **Sorting**: ORDER BY, TOP N, pagination
- **Window Functions**: ROW_NUMBER, running totals, LAG/LEAD
- **Complex**: CTEs, nested queries, complex expressions

## Hardware Notes

Results vary based on:
- CPU speed and core count
- Available RAM
- Storage speed (for initial CSV loading)
- Operating system and background processes

For consistent benchmarking:
1. Close unnecessary applications
2. Run benchmarks multiple times and average results
3. Use release builds (`cargo build --release`)
4. Ensure adequate free memory

## Future Work

1. **Implement remaining Phase 2 & 3 optimizations**
2. **Add more granular benchmarks** for specific scenarios
3. **Profile memory usage** alongside execution time
4. **Create regression test suite** to prevent performance degradation
5. **Add comparative benchmarks** against other SQL tools