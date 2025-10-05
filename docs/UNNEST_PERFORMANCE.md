# UNNEST Performance Analysis

## Big-O Complexity

### Time Complexity

Given:
- `N` = number of input rows
- `M` = average expansion factor (items per delimited string)
- `C` = number of columns in SELECT
- `U` = number of UNNEST columns

**Overall: O(N × M × C)**

### Detailed Breakdown

#### 1. Row Iteration
```rust
for &row_idx in visible_rows {  // O(N)
```
- **Cost**: O(N) - Must process every input row

#### 2. String Splitting (per UNNEST column)
```rust
text.split(delimiter)           // O(L) where L = string length
    .filter(|s| !s.is_empty())  // O(M) - iterate over M parts
    .map(|s| DataValue::String(s.to_string()))  // O(M × L_avg)
    .collect()
```
- **Cost per UNNEST**: O(L + M × L_avg)
  - Where L = total string length, L_avg = average part length
  - Typically L ≈ M × L_avg, so **O(M × L_avg)**
  - With U UNNEST columns: **O(U × M × L_avg)**

#### 3. Output Row Generation
```rust
for output_idx in 0..expansion_count {  // O(M) - M expanded rows
    for (col_idx, item) in expanded_items.iter().enumerate() {  // O(C) columns
        // Value extraction: O(1) for UNNEST (array lookup)
        // Value extraction: O(1) for regular columns (direct access)
        // Value extraction: O(E) for expressions (E = expression complexity)
    }
}
```
- **Cost**: O(M × C)
- For each expanded row, we evaluate C columns

#### 4. Result Table Construction
```rust
result_table.add_row(DataRow::new(row_values))  // O(C) - clone C values
```
- **Cost**: O(M × C) - M rows × C columns

### Combined Analysis

**Per Input Row:**
- String splitting: O(U × M × L_avg)
- Row generation: O(M × C)
- **Per-row total**: O(U × M × L_avg + M × C)

**Total for N Rows:**
```
O(N × (U × M × L_avg + M × C))
= O(N × M × (U × L_avg + C))
```

**Simplified**: **O(N × M × C)** when U and L_avg are constants

### Space Complexity

**Total Output Size: O(N × M × C)**

#### Memory Allocation
1. **Expansion arrays**: O(U × M) per input row (temporary)
2. **Result table**: O(N × M × C) total
3. **String storage**: O(N × M × C × L_avg) - actual string data

**Peak memory**: O(N × M × C) for the result table

### Performance Characteristics

#### Best Case: O(N × C)
- M = 1 (no actual expansion, single item per field)
- Example: `accounts = "ACC_1"` (no delimiter found)

#### Typical Case: O(N × M × C)
- M ≈ 3-10 (moderate expansion)
- Example: FIX allocations with 3-5 accounts per order

#### Worst Case: O(N × M × C)
- M is large (hundreds of items per string)
- C is large (many columns)
- Example: CSV with 100 tags and 20 columns

## Practical Performance

### Example Scenarios

#### Scenario 1: FIX Allocations (Your Use Case)
```sql
SELECT order_id, UNNEST(accounts, '|') AS account, UNNEST(amounts, ',') AS amount
FROM fix_allocations
```

**Parameters:**
- N = 10,000 orders
- M = 5 accounts per order average
- C = 3 columns (order_id, account, amount)
- U = 2 UNNEST columns

**Operations:**
- String splits: 10,000 × 2 × 5 = 100,000 splits
- Output rows: 10,000 × 5 = 50,000 rows
- Memory cells: 50,000 × 3 = 150,000 values

**Expected Performance**: ~10-50ms on modern hardware

#### Scenario 2: Large Dataset
```sql
SELECT UNNEST(tags, '|') AS tag FROM data
```

**Parameters:**
- N = 1,000,000 rows
- M = 10 tags per row
- C = 1 column
- U = 1 UNNEST column

**Operations:**
- String splits: 1,000,000 × 10 = 10,000,000 splits
- Output rows: 10,000,000 rows
- Memory cells: 10,000,000 values

**Expected Performance**: ~1-5 seconds
**Memory**: ~100-500MB (depending on tag size)

#### Scenario 3: Extreme Case
```sql
SELECT UNNEST(large_array, ',') AS item FROM huge_table
```

**Parameters:**
- N = 10,000,000 rows
- M = 100 items per row
- C = 1 column

**Operations:**
- Output rows: 1,000,000,000 (1 billion)
- Memory: ~10-50GB

**Expected Performance**: Minutes, likely out-of-memory

## Optimization Opportunities

### Current Implementation
✅ **Good:**
- Single pass over input data
- No unnecessary copies during expansion
- Direct array access for expanded values (O(1))
- String splitting uses Rust's efficient `split()` iterator

❌ **Could be better:**
- Evaluates non-UNNEST expressions M times (could cache)
- Creates temporary expansion arrays (could stream)
- Replicates column values for each output row (could use pointers)

### Potential Optimizations

#### 1. Expression Caching
**Current**: Evaluates regular expressions M times per row
```rust
for output_idx in 0..expansion_count {  // M iterations
    evaluator.evaluate(expr, row_idx)?  // Called M times!
}
```

**Optimized**: Evaluate once, reuse M times
```rust
let cached_value = evaluator.evaluate(expr, row_idx)?;
for output_idx in 0..expansion_count {
    cached_value.clone()  // Just clone the result
}
```
**Impact**: Reduces O(M × E) to O(E + M) for expressions

#### 2. Streaming (for large M)
**Current**: Materializes entire result in memory

**Optimized**: Could use iterator pattern
```rust
impl Iterator for UnnestIterator {
    type Item = DataRow;
    // Yield rows one at a time
}
```
**Impact**: Constant memory instead of O(N × M × C)
**Tradeoff**: Can't sort, can't count rows beforehand

#### 3. Copy-on-Write for Regular Columns
**Current**: Clones DataValue for each replicated column

**Optimized**: Use Rc<DataValue> or similar
**Impact**: Reduces memory and clone cost
**Tradeoff**: More complex code

## Comparison to Alternatives

### vs. Manual Post-Processing (Python/Shell)
**UNNEST**: O(N × M × C) - Single pass in Rust
**Post-processing**: O(N) read + O(N × M) parse + O(N × M × C) process

**Winner**: UNNEST (faster, no I/O overhead)

### vs. Database UNNEST (PostgreSQL, DuckDB)
**This implementation**: O(N × M × C), in-memory
**PostgreSQL UNNEST**: Similar complexity, but:
- Can spill to disk for huge datasets
- Better query optimization (push down filters)
- Parallel execution possible

**Winner**: PostgreSQL for huge datasets, this for medium data

### vs. JOIN with Numbers Table
```sql
SELECT order_id, SPLIT_PART(accounts, '|', n.num) AS account
FROM fix_allocations
CROSS JOIN generate_series(1, 100) AS n(num)
WHERE SPLIT_PART(accounts, '|', n.num) != ''
```
**Complexity**: O(N × 100 × C) - Always generates 100 rows, filters later

**Winner**: UNNEST (adaptive to actual data)

## Recommendations

### ✅ Good Use Cases
- **Moderate datasets**: N < 1M, M < 100
- **FIX message parsing**: Allocations, trades, order routing
- **Tag processing**: User tags, categories, labels
- **CSV-in-CSV**: Embedded lists in CSV files
- **Real-time analysis**: Fast interactive queries

### ⚠️ Use with Caution
- **Large M**: Hundreds of items per string
- **Large N**: Millions of rows
- **Many UNNEST columns**: U > 5
- **Combined with JOINs**: Multiplicative effect

### 🚫 Avoid
- **Huge datasets**: N × M > 100M rows
- **Nested UNNEST**: Multiple levels of expansion
- **Memory-constrained**: < 1GB available
- **Use DuckDB/PostgreSQL instead for these**

## Monitoring Performance

### Add Timing
```rust
let start = Instant::now();
let result = self.apply_select_with_row_expansion(view, select_items)?;
debug!("UNNEST expansion took: {:?}", start.elapsed());
```

### Key Metrics
- Input rows: `visible_rows.len()`
- Output rows: `result_table.row_count()`
- Expansion factor: `output / input`
- Time per input row: `total_time / input_rows`

### Example Log Output
```
UNNEST expansion: 10,000 → 50,000 rows (5.0x) in 23.4ms (2.34µs/row)
```

## Conclusion

**UNNEST is efficient for typical use cases:**
- Linear in input rows (O(N))
- Linear in expansion factor (O(M))
- No quadratic behavior
- Good cache locality
- Minimal allocations

**Sweet spot: N × M < 10M rows**

For your FIX allocations at work, UNNEST should be **very fast** - expect sub-100ms query times for datasets with thousands of orders.
