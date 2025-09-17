# SQL CLI Performance Roadmap & Benchmarking Plan

## Executive Summary

The SQL CLI engine is designed for interactive exploration of datasets up to 100K rows. This document outlines our performance benchmarking strategy and optimization roadmap to ensure predictable, responsive performance within our target use cases.

## Current State Assessment

### What We Have Built

- **11 Generator Functions**: Mathematical sequences, random data, UUIDs
- **Comprehensive Function Library**: 100+ SQL functions across categories
- **Query Optimization**: Expression lifting, CTE hoisting, IN operator optimization
- **Window Functions**: Full SQL window function support
- **Debug Tracing**: Zero-cost debug system for performance analysis

### Known Performance Characteristics

- **Memory Model**: Full in-memory operation (no streaming yet)
- **String Handling**: Heavy string allocation and cloning
- **Expression Evaluation**: Row-by-row evaluation model
- **No Indexes**: Full table scans for all WHERE clauses

## Benchmarking Framework Design

### 1. Data Generation Scenarios

Using our generator functions to create reproducible test data:

```sql
-- Narrow table (high rows, few columns)
CREATE TABLE narrow_bench AS
SELECT
    id,
    value AS amount,
    RANDOM_INT(1, 100, 42) AS category_id
FROM RANDOM_INT(100000, 1, 1000000, 42);

-- Wide table (many columns)
CREATE TABLE wide_bench AS
SELECT
    id,
    RANDOM_INT(1, 1000, seed+1) AS int_col_1,
    RANDOM_FLOAT(0, 1000, seed+2) AS float_col_1,
    GENERATE_UUID() AS uuid_col,
    -- ... up to 50 columns
FROM GENERATE_SERIES(100000);

-- Mixed types table (realistic data)
CREATE TABLE mixed_bench AS
SELECT
    GENERATE_UUID() AS id,
    RANDOM_INT(1, 5, 42) AS category,
    RANDOM_FLOAT(10, 1000, 43) AS price,
    RANDOM_INT(1, 100, 44) AS quantity,
    CASE WHEN RANDOM_INT(0, 1, 45) = 1 THEN 'Active' ELSE 'Inactive' END AS status
FROM GENERATE_SERIES(100000);
```

### 2. Query Benchmark Suite

#### Category A: Basic Operations
```sql
-- A1: Full table scan
SELECT * FROM table;

-- A2: Simple filter
SELECT * FROM table WHERE id > 50000;

-- A3: Multiple conditions
SELECT * FROM table WHERE category = 3 AND price > 500;

-- A4: String operations
SELECT * FROM table WHERE status = 'Active';

-- A5: LIKE patterns
SELECT * FROM table WHERE status LIKE 'Act%';
```

#### Category B: Aggregations
```sql
-- B1: Simple aggregates
SELECT COUNT(*), SUM(price), AVG(quantity) FROM table;

-- B2: GROUP BY single column
SELECT category, COUNT(*) FROM table GROUP BY category;

-- B3: GROUP BY multiple columns
SELECT category, status, SUM(price * quantity)
FROM table GROUP BY category, status;

-- B4: HAVING clause
SELECT category, AVG(price) AS avg_price
FROM table GROUP BY category HAVING AVG(price) > 500;
```

#### Category C: Sorting & Limits
```sql
-- C1: Simple ORDER BY
SELECT * FROM table ORDER BY price DESC;

-- C2: Multi-column sort
SELECT * FROM table ORDER BY category, price DESC;

-- C3: TOP-N queries
SELECT * FROM table ORDER BY price DESC LIMIT 100;

-- C4: OFFSET pagination
SELECT * FROM table ORDER BY id LIMIT 100 OFFSET 1000;
```

#### Category D: Window Functions
```sql
-- D1: ROW_NUMBER
SELECT *, ROW_NUMBER() OVER (ORDER BY price) AS rn FROM table;

-- D2: Running totals
SELECT *, SUM(price) OVER (ORDER BY id) AS running_total FROM table;

-- D3: LAG/LEAD
SELECT *, LAG(price) OVER (ORDER BY id) AS prev_price FROM table;

-- D4: Partitioned windows
SELECT *, RANK() OVER (PARTITION BY category ORDER BY price DESC) FROM table;
```

#### Category E: Complex Queries
```sql
-- E1: CTEs
WITH summary AS (
    SELECT category, AVG(price) as avg_price
    FROM table GROUP BY category
)
SELECT * FROM summary WHERE avg_price > 500;

-- E2: Nested CTEs (test hoisting)
WITH level1 AS (
    SELECT * FROM table WHERE category IN (1,2,3)
),
level2 AS (
    SELECT * FROM level1 WHERE price > 100
)
SELECT COUNT(*) FROM level2;

-- E3: Complex expressions
SELECT
    id,
    price * quantity * 1.1 AS total_with_tax,
    CASE
        WHEN price > 1000 THEN 'Premium'
        WHEN price > 500 THEN 'Standard'
        ELSE 'Budget'
    END AS tier
FROM table;
```

### 3. Metrics to Capture

```rust
pub struct BenchmarkMetrics {
    // Timing
    pub query_parse_time: Duration,
    pub query_plan_time: Duration,
    pub query_execute_time: Duration,
    pub total_time: Duration,

    // Throughput
    pub rows_processed: usize,
    pub rows_per_second: f64,

    // Memory
    pub peak_memory_bytes: usize,
    pub allocations_count: usize,

    // Query specifics
    pub table_scans: usize,
    pub expressions_evaluated: usize,
    pub strings_allocated: usize,
}
```

### 4. Benchmark Execution Plan

```rust
// Row count scenarios
const ROW_COUNTS: &[usize] = &[
    100,      // Tiny
    1_000,    // Small
    10_000,   // Medium
    50_000,   // Large
    100_000,  // XL
];

// Run each query at each scale
// Track performance degradation
// Identify O(n²) or worse patterns
```

## Performance Optimization Roadmap

### Phase 1: Quick Wins (1-2 days)

#### 1.1 String Interning Enhancement
- **Problem**: Duplicate strings in categorical columns
- **Solution**: Aggressive interning for repeated values
- **Expected Impact**: 30-50% memory reduction for string-heavy tables

#### 1.2 Expression Result Caching
- **Problem**: Re-evaluating same expressions
- **Solution**: Cache expression results within query execution
- **Expected Impact**: 20-30% speedup for complex expressions

#### 1.3 Early Termination
- **Problem**: Processing all rows even with LIMIT
- **Solution**: Stop processing once LIMIT reached
- **Expected Impact**: Massive speedup for LIMIT queries

### Phase 2: Structural Improvements (3-5 days)

#### 2.1 Columnar Expression Evaluation
- **Problem**: Row-by-row evaluation has poor cache locality
- **Solution**: Process expressions in column chunks
- **Expected Impact**: 2-3x speedup for expression-heavy queries

#### 2.2 Simple Hash Indexes
- **Problem**: O(n) scans for equality filters
- **Solution**: Build hash indexes for WHERE column = value
- **Expected Impact**: O(1) lookups for indexed columns

#### 2.3 Statistics Collection
- **Problem**: No query planning optimization
- **Solution**: Track min/max/null counts per column
- **Expected Impact**: Better query plans, skip unnecessary work

### Phase 3: Advanced Optimizations (1-2 weeks)

#### 3.1 Vectorized Operations
- **Problem**: Scalar operations miss SIMD opportunities
- **Solution**: Use packed_simd for numerical operations
- **Expected Impact**: 4-8x speedup for numerical aggregations

#### 3.2 Parallel Execution
- **Problem**: Single-threaded execution
- **Solution**: Use Rayon for parallelizable operations
- **Expected Impact**: Near-linear speedup with core count

#### 3.3 Lazy Materialization
- **Problem**: Materializing full intermediate results
- **Solution**: Stream results between operators
- **Expected Impact**: Reduced memory usage, better cache behavior

## Implementation Priority Matrix

| Optimization | Impact | Effort | Priority |
|-------------|--------|--------|----------|
| String Interning | High | Low | P0 |
| Early LIMIT | High | Low | P0 |
| Expression Cache | Medium | Low | P1 |
| Hash Indexes | High | Medium | P1 |
| Column Statistics | Medium | Medium | P2 |
| Vectorization | High | High | P2 |
| Parallel Execution | High | Medium | P2 |
| Streaming Results | Medium | High | P3 |

## Success Criteria

### Performance Targets (100K rows)

- **Full Scan**: < 100ms
- **Filtered Scan** (WHERE): < 50ms
- **Simple Aggregation**: < 200ms
- **GROUP BY** (10 groups): < 300ms
- **ORDER BY**: < 500ms
- **Window Functions**: < 1s
- **Complex CTE**: < 2s

### Memory Targets

- **Base memory**: < 10MB overhead
- **Row storage**: < 1KB per row average
- **Peak memory**: < 3x table size
- **String deduplication**: > 50% for categorical data

## Testing Strategy

### 1. Baseline Establishment
- Run full benchmark suite on current codebase
- Profile with `perf` and `flamegraph`
- Identify hotspots and bottlenecks

### 2. Incremental Optimization
- Implement one optimization at a time
- Re-run affected benchmarks
- Document performance changes
- Ensure no regressions

### 3. Real-World Validation
- Test with actual CSV files
- Mixed workload scenarios
- Interactive usage patterns
- Memory pressure situations

## Next Steps (Tomorrow's Session)

1. **Create Benchmark Module**
   ```rust
   // src/benchmarks/mod.rs
   pub mod data_generator;
   pub mod query_suite;
   pub mod metrics;
   pub mod runner;
   ```

2. **Implement Data Generators**
   - Use existing generator functions
   - Create realistic data distributions
   - Save as benchmark fixtures

3. **Run Baseline Benchmarks**
   - Execute full suite
   - Generate performance report
   - Create flamegraphs

4. **Start Quick Wins**
   - Implement string interning improvements
   - Add early termination for LIMIT
   - Measure impact

## Appendix: Profiling Commands

```bash
# CPU Profiling
cargo build --release
perf record --call-graph=dwarf ./target/release/sql-cli bench
perf script | inferno-flamegraph > flamegraph.svg

# Memory Profiling
valgrind --tool=massif ./target/release/sql-cli bench
ms_print massif.out.<pid> > memory_profile.txt

# Benchmark Execution
cargo bench --bench sql_benchmarks

# Quick Performance Test
time ./target/release/sql-cli data.csv -q "SELECT COUNT(*) FROM data"
```

## Conclusion

Our goal is not to build a general-purpose OLAP engine, but rather to ensure the SQL CLI provides responsive, predictable performance for interactive data exploration up to 100K rows. By focusing on the most common query patterns and implementing targeted optimizations, we can achieve excellent performance within our design constraints.

The benchmark suite will give us quantitative metrics to guide optimization efforts and ensure we don't regress as we add features. The modular approach allows us to tackle improvements incrementally while maintaining stability.