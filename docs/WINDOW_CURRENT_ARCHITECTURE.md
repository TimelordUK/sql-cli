# Window Functions - Current Architecture (Per-Row Evaluation)

**Date**: 2025-11-03
**Purpose**: Document current window function implementation before batch evaluation refactor
**Status**: This represents the architecture as of the hash-based optimization (v1.66.0+)

## Overview

Window functions in SQL CLI currently use a **per-row evaluation model**, where each window function is evaluated once for every row in the result set. This architecture is straightforward but incurs significant overhead when processing large datasets (50,000+ rows).

**Current Performance**:
- 50k rows with LAG: ~1.69s (without logging)
- Target (GROUP BY equivalent): ~600ms
- Gap: 2.8x slower than target

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Query Execution                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  1. Parser (recursive_parser.rs)                                │
│     - Recognizes window function syntax                         │
│     - Creates WindowSpec AST node                               │
│     - Output: SelectStatement with WindowFunction expressions   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  2. Query Engine (query_engine.rs)                              │
│     - Detects has_window_functions flag                         │
│     - Pre-creates WindowContexts (optimization)                 │
│     - LOOPS over each row (50,000 times!)                       │
│     - For each row, evaluates all SELECT items                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (per row)
┌─────────────────────────────────────────────────────────────────┐
│  3. Arithmetic Evaluator (arithmetic_evaluator.rs)              │
│     - Called once PER ROW for each window function              │
│     - Looks up WindowContext from cache (4μs per lookup)        │
│     - Delegates to window function logic                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (per row)
┌─────────────────────────────────────────────────────────────────┐
│  4. Window Context (window_context.rs)                          │
│     - Created once per unique WindowSpec                        │
│     - Contains partitioned + sorted data                        │
│     - get_offset_value() called 50,000 times                    │
└─────────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. Parser (`src/sql/recursive_parser.rs`)

**Responsibility**: Parse SQL text into AST

**Window Function Recognition**:
```rust
// Recognizes: LAG(column) OVER (PARTITION BY x ORDER BY y)
fn parse_window_function() -> Result<SqlExpression> {
    let name = parse_identifier();  // "LAG", "LEAD", "ROW_NUMBER", etc.
    let args = parse_function_args();
    let spec = parse_over_clause();  // OVER (...)

    SqlExpression::WindowFunction {
        name,
        args,
        window_spec: spec,
    }
}
```

**Output**: `SelectStatement` containing `SqlExpression::WindowFunction` nodes

**Key Files**:
- `src/sql/recursive_parser.rs:parse_over_clause()` - Parses OVER clause
- `src/sql/parser/ast.rs:WindowSpec` - Window specification structure

---

### 2. Query Engine (`src/data/query_engine.rs`)

**Responsibility**: Orchestrate query execution, manage row loop

**Window Function Detection** (`query_engine.rs:1680-1720`):
```rust
// Check if any SELECT item contains window functions
let has_window_functions = select_items.iter().any(|item| {
    if let SelectItem::Expression { expr, .. } = item {
        Self::contains_window_function(expr)
    } else {
        false
    }
});
```

**Pre-creation Optimization** (`query_engine.rs:1867-1888`):
```rust
if has_window_functions {
    // Extract all unique WindowSpecs from SELECT items
    let mut window_specs = Vec::new();
    for item in &expanded_items {
        if let SelectItem::Expression { expr, .. } = item {
            Self::collect_window_specs(expr, &mut window_specs);
        }
    }

    // Pre-create all WindowContexts (avoids creation overhead)
    for spec in &window_specs {
        let _ = evaluator.get_or_create_window_context(spec);
    }
}
```

**Per-Row Evaluation Loop** (`query_engine.rs:1890-1925`):
```rust
for &row_idx in visible_rows {  // ← LOOPS 50,000 TIMES!
    let mut row_values = Vec::new();

    for item in &expanded_items {
        let value = match item {
            SelectItem::Column { column: col_ref, .. } => {
                evaluator.evaluate(&SqlExpression::Column(col_ref.clone()), row_idx)?
            }
            SelectItem::Expression { expr, .. } => {
                // This evaluates window functions per-row!
                evaluator.evaluate(&expr, row_idx)?  // ← BOTTLENECK
            }
            // ...
        };
        row_values.push(value);
    }

    computed_table.add_row(DataRow::new(row_values))?;
}
```

**Key Insight**: The row loop calls `evaluator.evaluate()` for EVERY expression in EVERY row. For window functions, this means 50,000 function calls even though the underlying data is already partitioned and sorted.

**Key Files**:
- `src/data/query_engine.rs:apply_select_items()` - Main SELECT evaluation
- `src/data/query_engine.rs:collect_window_specs()` - Extract window specs
- `src/data/query_engine.rs:contains_window_function()` - Detect window functions

---

### 3. Arithmetic Evaluator (`src/data/arithmetic_evaluator.rs`)

**Responsibility**: Evaluate expressions against data, manage WindowContext cache

**WindowContext Cache** (`arithmetic_evaluator.rs:28`):
```rust
pub struct ArithmeticEvaluator<'a> {
    // ...
    window_contexts: HashMap<u64, Arc<WindowContext>>,  // Hash-based cache
}
```

**Cache Lookup** (`arithmetic_evaluator.rs:865-910`):
```rust
pub fn get_or_create_window_context(&mut self, spec: &WindowSpec) -> Result<Arc<WindowContext>> {
    let overall_start = Instant::now();

    // Hash-based key (much faster than format!("{:?}", spec))
    let key = spec.compute_hash();

    if let Some(context) = self.window_contexts.get(&key) {
        // Cache hit - happens 49,999 times out of 50,000
        info!("WindowContext cache hit for spec (lookup: {:.2}μs)",
              overall_start.elapsed().as_micros());
        return Ok(Arc::clone(context));
    }

    // Cache miss - create new context
    info!("WindowContext cache miss - creating new context");
    let context = WindowContext::new_with_spec(view, spec.clone())?;
    let context_arc = Arc::new(context);
    self.window_contexts.insert(key, Arc::clone(&context_arc));
    Ok(context_arc)
}
```

**Per-Row Evaluation** (`arithmetic_evaluator.rs:923-1322`):
```rust
fn evaluate_window_function(
    &mut self,
    name: &str,
    args: &[SqlExpression],
    spec: &WindowSpec,
    row_index: usize,  // ← Called with different row_index 50,000 times
) -> Result<DataValue> {
    let func_start = Instant::now();

    // Get context from cache (4μs per call)
    let context = self.get_or_create_window_context(spec)?;

    // Evaluate the specific window function
    match name.to_uppercase().as_str() {
        "LAG" => {
            let column = /* extract column from args */;
            let offset = /* extract offset from args */;
            context.get_offset_value(row_index, -offset, &column.name)
                .unwrap_or(DataValue::Null)
        }
        "LEAD" => { /* similar */ }
        "ROW_NUMBER" => { /* similar */ }
        "RANK" => { /* similar */ }
        "DENSE_RANK" => { /* similar */ }
        _ => return Err(anyhow!("Unknown window function: {}", name)),
    }
}
```

**Bottleneck Analysis**:
- **Cache lookup**: 4μs per call × 50,000 = 200ms
- **Function call overhead**: ~10-20μs per call × 50,000 = 500-1,000ms
- **Total overhead**: ~1,200-1,500ms just from calling the function 50,000 times!

**Key Files**:
- `src/data/arithmetic_evaluator.rs:get_or_create_window_context()` - Cache management
- `src/data/arithmetic_evaluator.rs:evaluate_window_function()` - Window function dispatch
- `src/data/arithmetic_evaluator.rs:evaluate()` - Main expression evaluator

---

### 4. Window Context (`src/sql/window_context.rs`)

**Responsibility**: Store partitioned and sorted data, provide offset access

**Creation** (`window_context.rs:137-257`):
```rust
pub fn new_with_spec(view: Arc<DataView>, spec: WindowSpec) -> Result<Self> {
    let row_count = view.row_count();

    if spec.partition_by.is_empty() {
        // Single partition - all rows in one group
        let mut all_rows: Vec<usize> = (0..row_count).collect();

        // Sort by ORDER BY if specified
        if !spec.order_by.is_empty() {
            Self::sort_rows(&mut all_rows, &view, &spec.order_by)?;
        }

        Ok(Self {
            view,
            partitions: vec![all_rows],
            spec,
        })
    } else {
        // Multiple partitions - group by PARTITION BY columns
        let mut partition_map: HashMap<Vec<String>, Vec<usize>> = HashMap::new();

        for row_idx in 0..row_count {
            let key = Self::compute_partition_key(row_idx, &view, &spec.partition_by)?;
            partition_map.entry(key).or_insert_with(Vec::new).push(row_idx);
        }

        // Sort each partition by ORDER BY
        let mut partitions: Vec<Vec<usize>> = partition_map.into_values().collect();
        for partition in &mut partitions {
            if !spec.order_by.is_empty() {
                Self::sort_rows(partition, &view, &spec.order_by)?;
            }
        }

        Ok(Self { view, partitions, spec })
    }
}
```

**Offset Access** (`window_context.rs:408-456`):
```rust
pub fn get_offset_value(
    &self,
    row_index: usize,
    offset: i32,  // -1 for LAG, +1 for LEAD
    column_name: &str,
) -> Result<DataValue> {
    // Find which partition this row belongs to
    for partition in &self.partitions {
        if let Some(position) = partition.iter().position(|&r| r == row_index) {
            // Calculate target position
            let target_pos = position as i32 + offset;

            if target_pos < 0 || target_pos >= partition.len() as i32 {
                return Ok(DataValue::Null);  // Out of bounds
            }

            let target_row = partition[target_pos as usize];
            return self.view.get_cell_value(target_row, column_name);
        }
    }

    Err(anyhow!("Row {} not found in any partition", row_index))
}
```

**Key Insight**: WindowContext is created ONCE per unique WindowSpec, but `get_offset_value()` is called 50,000 times (once per row).

**Key Files**:
- `src/sql/window_context.rs:new_with_spec()` - Context creation
- `src/sql/window_context.rs:get_offset_value()` - LAG/LEAD access
- `src/sql/window_context.rs:get_row_number()` - ROW_NUMBER access
- `src/sql/window_context.rs:get_rank()` - RANK/DENSE_RANK access

---

## Data Flow

### Single Query Execution (50,000 rows)

```
1. User query: SELECT LAG(amount) OVER (ORDER BY id) FROM sales_test

2. Parser creates AST:
   SelectStatement {
       items: [
           Expression {
               expr: WindowFunction {
                   name: "LAG",
                   args: [Column("amount")],
                   window_spec: WindowSpec {
                       partition_by: [],
                       order_by: [OrderByItem { expr: Column("id"), direction: Asc }],
                       frame: None,
                   }
               },
               alias: "lag_amt"
           }
       ],
       ...
   }

3. Query Engine:
   a. Detects has_window_functions = true
   b. Extracts WindowSpecs: [WindowSpec { order_by: [id ASC] }]
   c. Pre-creates WindowContext (9.8ms)
      - Groups all 50,000 rows into 1 partition
      - Sorts by id
   d. LOOP over 50,000 rows:
      - Call evaluator.evaluate(WindowFunction, row_0)
      - Call evaluator.evaluate(WindowFunction, row_1)
      - ...
      - Call evaluator.evaluate(WindowFunction, row_49999)

4. Arithmetic Evaluator (called 50,000 times):
   a. Hash WindowSpec (1μs)
   b. Lookup in cache (2μs) → cache hit (49,999 times)
   c. Clone Arc<WindowContext> (1μs)
   d. Call evaluate_window_function()

5. Window Context (called 50,000 times):
   a. context.get_offset_value(row_idx, -1, "amount")
   b. Find partition (fast - single partition)
   c. Find position in partition (fast - sorted)
   d. Calculate offset position
   e. Return value at offset position

6. Result: 50,000 rows with LAG values computed
   Total time: ~1.69s (1,500ms overhead + 190ms actual work)
```

## Performance Characteristics

### Time Breakdown (50,000 rows)

| Operation | Time | Percentage | Called |
|-----------|------|------------|--------|
| **WindowContext creation** | 9.8ms | 0.6% | 1× |
| **Per-row cache lookup** | 200ms | 11.8% | 50,000× |
| **Function call overhead** | 800ms | 47.3% | 50,000× |
| **Actual LAG logic** | 100ms | 5.9% | 50,000× |
| **Other (row building, etc.)** | 580ms | 34.3% | - |
| **Total** | 1,690ms | 100% | - |

### Scalability

| Rows | Current Time | Per-Row Cost | Overhead |
|------|-------------|--------------|----------|
| 10k | 316ms | 31.6μs | ~200ms |
| 50k | 1,690ms | 33.8μs | ~1,500ms |
| 100k | ~3,400ms | 34.0μs | ~3,000ms |

**Key Observation**: Per-row cost is roughly constant (~33μs), meaning the architecture scales linearly with O(n) complexity. However, 90% of this time is overhead from calling functions 50,000 times.

## Limitations

### 1. Per-Row Function Call Overhead
- **Problem**: Even "free" operations (cache hits) cost 4μs when done 50,000 times = 200ms
- **Impact**: 11.8% of total time just looking up cached contexts
- **Root Cause**: HashMap lookups aren't free at this scale

### 2. Redundant Context Switching
- **Problem**: Call stack overhead from evaluator → window_function → context × 50,000
- **Impact**: ~800ms (47% of total time)
- **Root Cause**: Function calls aren't free when repeated thousands of times

### 3. Memory Allocations
- **Problem**: Each function call creates temporary values, clones Arcs, etc.
- **Impact**: Garbage collection pressure, cache misses
- **Root Cause**: Per-row evaluation creates many small, short-lived allocations

### 4. Cannot Leverage Vectorization
- **Problem**: Processing one row at a time prevents SIMD optimizations
- **Impact**: Could be 4-8x faster with vectorization
- **Root Cause**: Architecture doesn't expose batch operations

### 5. Inefficient for Multiple Window Functions
- **Problem**: Multiple window functions with same WindowSpec still look up cache N×M times
- **Impact**: Wasted lookups (cache hit is fast but not free)
- **Root Cause**: No way to share context across multiple functions in same query

## Why This Architecture Was Chosen

### Advantages (Original Design)
1. **Simple and intuitive** - matches SQL semantics (row-by-row)
2. **Easy to implement** - straightforward translation from SQL
3. **Works correctly** - handles all edge cases properly
4. **Cache optimization** - added later to avoid redundant context creation

### Why It Worked Initially
- Small datasets (<1,000 rows) - overhead negligible
- Few window functions - not a performance bottleneck
- Correctness over performance - get it working first

### Why It Doesn't Scale
- **Linear overhead with row count** - 50k rows = 50k function calls
- **Cache hit still costs time** - 4μs × 50,000 = 200ms
- **Function call stack overhead** - 800ms wasted on calling functions

## Path Forward: Batch Evaluation

The fundamental issue is that we're treating window functions like scalar functions (evaluated per-row) when they're actually aggregate-like operations (should be evaluated per-partition or per-query).

**Goal**: Move from:
```rust
for row in rows {
    result[row] = LAG(row)  // 50,000 calls
}
```

To:
```rust
results = LAG_BATCH(all_rows)  // 1 call
```

This is what the batch evaluation plan (`WINDOW_BATCH_EVALUATION_PLAN.md`) aims to achieve.

## References

**Related Documents**:
- `WINDOW_BATCH_EVALUATION_PLAN.md` - Plan for implementing batch evaluation
- `WINDOW_HASH_OPTIMIZATION_RESULTS.md` - Results from hash-based cache keys
- `WINDOW_LOGGING_OVERHEAD_ANALYSIS.md` - Analysis of profiling overhead

**Key Code Locations**:
- Parser: `src/sql/recursive_parser.rs:parse_over_clause()`
- Query Engine: `src/data/query_engine.rs:apply_select_items()`
- Evaluator: `src/data/arithmetic_evaluator.rs:evaluate_window_function()`
- Context: `src/sql/window_context.rs:get_offset_value()`
- AST: `src/sql/parser/ast.rs:WindowSpec`

**Profiling**:
- Set `RUST_LOG=info` to see timing logs
- Use `--execution-plan` flag for high-level timing (no overhead)
- Benchmark script: `tests/integration/benchmark_window_functions.sh`
