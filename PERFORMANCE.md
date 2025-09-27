# SQL CLI Performance Benchmarks

## Overview

SQL CLI is an in-memory query engine optimized for datasets up to 100,000 rows, delivering exceptional performance by eliminating I/O overhead. This document presents comprehensive benchmarks demonstrating sub-second query execution for most operations.

## Key Performance Achievements

- **Simple queries**: 8-20ms at 100K rows
- **JOINs**: Under 40ms for all join types
- **Pattern matching**: 26-52ms for LIKE operations
- **Complex aggregations**: Under 2.5 seconds worst case
- **Window functions**: ~1.2 seconds at 100K rows

## Benchmark Results

### 100,000 Row Performance

```
Basic Operations:
    Simple SELECT *                                   8.17ms
    WHERE numeric comparison                         17.31ms
    WHERE string equality                            196.0ms
    Complex WHERE (3 conditions)                     227.8ms
    ORDER BY single column                           18.43ms
    ORDER BY multiple columns                        29.85ms
    COUNT DISTINCT                                   371.7ms

LIKE Pattern Matching:
    LIKE with suffix wildcard                        35.80ms
    LIKE with prefix wildcard                        32.84ms
    LIKE with both wildcards                         39.06ms
    Multiple LIKE conditions                         52.32ms
    LIKE with result set                             26.41ms

GROUP BY Operations:
    Simple GROUP BY                                  433.3ms
    GROUP BY with SUM                                785.9ms
    GROUP BY with multiple aggregates                  1.83s
    Multi-column GROUP BY                              2.49s

Window Functions:
    LAG function                                       1.23s
    LEAD function                                      1.23s
    LAG + LEAD combined                                2.34s
    LAG with PARTITION BY                            662.6ms
    ROW_NUMBER                                       858.1ms

JOIN Operations:
    Hash JOIN (equality) - small                     17.99ms
    Hash JOIN (equality) - medium                    20.42ms
    Nested loop JOIN (inequality) - small            18.45ms
    Self-join for cumulative sum                     38.79ms
    LEFT JOIN with inequality                        19.18ms
    JOIN with CTE categories                         26.47ms
```

### 50,000 Row Performance

```
Basic Operations:
    Simple SELECT *                                   4.33ms
    WHERE numeric comparison                          9.11ms
    WHERE string equality                            96.64ms
    Complex WHERE (3 conditions)                     118.9ms
    ORDER BY single column                            7.97ms
    ORDER BY multiple columns                        12.80ms
    COUNT DISTINCT                                   181.8ms

LIKE Pattern Matching:
    LIKE with suffix wildcard                        16.48ms
    LIKE with prefix wildcard                        17.27ms
    LIKE with both wildcards                         20.17ms
    Multiple LIKE conditions                         27.33ms
    LIKE with result set                             12.99ms

GROUP BY Operations:
    Simple GROUP BY                                  241.3ms
    GROUP BY with SUM                                408.5ms
    GROUP BY with multiple aggregates                961.8ms
    Multi-column GROUP BY                              1.29s

Window Functions:
    LAG function                                     604.3ms
    LEAD function                                    608.8ms
    LAG + LEAD combined                                1.17s
    LAG with PARTITION BY                            326.0ms
    ROW_NUMBER                                       416.0ms

JOIN Operations:
    Hash JOIN (equality) - small                      9.60ms
    Hash JOIN (equality) - medium                    12.20ms
    Nested loop JOIN (inequality) - small             9.42ms
    Self-join for cumulative sum                     28.19ms
    LEFT JOIN with inequality                        11.24ms
    JOIN with CTE categories                         18.07ms
```

## Performance Categories

### ⚡ Lightning Fast (<20ms at 100K rows)
- Simple SELECT queries
- Numeric WHERE clauses
- Hash JOINs
- Single column ORDER BY

### 🚀 Very Fast (<100ms at 100K rows)
- LIKE pattern matching (all variants)
- Multi-column ORDER BY
- JOIN with CTEs
- Complex JOINs

### 💪 Fast (<1s at 100K rows)
- String equality WHERE
- Complex WHERE conditions
- COUNT DISTINCT
- Simple GROUP BY
- GROUP BY with single aggregate
- Partitioned window functions
- ROW_NUMBER

### 📊 Acceptable (1-3s at 100K rows)
- Individual window functions (LAG/LEAD)
- GROUP BY with multiple aggregates
- Multi-column GROUP BY
- Combined window functions

## Comparison with Traditional Databases

SQL CLI's in-memory architecture provides significant advantages over traditional databases for datasets up to 100K rows:

### Advantages
1. **No I/O overhead** - All data in memory
2. **No network latency** - Local execution
3. **No query planning overhead** - Optimized for known patterns
4. **No disk seeks** - Pure memory access
5. **No cache warming** - Always hot

### When SQL CLI Excels
- **Interactive data exploration** - Sub-second response for most queries
- **CSV/JSON file analysis** - Direct file querying without import
- **Development and testing** - Fast iteration on data queries
- **Small to medium datasets** - Optimal for up to 100K rows
- **Complex analytical queries** - Window functions and aggregations without database setup

### Performance Evolution
- Previous: GROUP BY at 50K rows took **12 seconds**
- Current: GROUP BY at 100K rows takes **2.49 seconds**
- Improvement: **~10x faster** at double the data size

## Running Benchmarks

To run the benchmark suite:

```bash
# Run full benchmark suite
python3 scripts/benchmark_all.py

# Run specific sizes
python3 scripts/benchmark_all.py --sizes 50000 100000

# Run and save results
python3 scripts/benchmark_all.py --output results.json
```

## Hardware Used

These benchmarks were performed on:
- CPU: [System specific - update as needed]
- RAM: Sufficient to hold dataset in memory
- OS: Linux (WSL2)
- Build: Release mode with optimizations

## Optimization Techniques

The exceptional performance is achieved through:

1. **Efficient data structures** - Columnar storage for cache locality
2. **Vectorized operations** - Process multiple rows simultaneously
3. **Hash-based algorithms** - O(1) lookups for JOINs and GROUP BY
4. **Lazy evaluation** - Compute only what's needed
5. **Zero-copy parsing** - Minimize data movement
6. **Optimized recursion** - Clean parser architecture after refactoring

## Conclusion

SQL CLI delivers production-ready performance for datasets up to 100K rows, with most operations completing in under 1 second. The in-memory architecture eliminates traditional database overhead, making it ideal for interactive data exploration, development workflows, and analytical queries on small to medium datasets.

For datasets beyond 100K rows or requiring persistence, traditional databases remain appropriate. However, within its target domain, SQL CLI's performance rivals or exceeds many commercial solutions, particularly when considering cold-start scenarios where traditional databases must load data from disk and warm caches.