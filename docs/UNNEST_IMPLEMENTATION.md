# UNNEST Implementation Guide

## Status: Foundation Complete ✅

### Completed Work
1. ✅ AST: SqlExpression::Unnest variant
2. ✅ Lexer: UNNEST keyword token
3. ✅ Parser: parse_unnest() function (creates FunctionCall currently)
4. ✅ Row Expansion Module: Isolated, reusable system
   - `RowExpander` trait
   - `RowExpanderRegistry`
   - `UnnestExpander` implementation
5. ✅ Tests: Row expander unit tests passing (403 total)
6. ✅ Design Docs: UNNEST_DESIGN.md with examples
7. ✅ Test Data: data/fix_allocations.csv

### Current Parsing Behavior
```sql
SELECT UNNEST(accounts, '|') AS account FROM fix_allocations
```

Parsed as:
```rust
FunctionCall {
    name: "UNNEST",
    args: [Column("accounts"), StringLiteral("|")]
}
```

This is **fine** - UNNEST can be treated as a special function that triggers row expansion.

## Remaining Implementation

### Step 1: Handle UNNEST in Arithmetic Evaluator

**File:** `src/data/arithmetic_evaluator.rs`

**Location:** In `evaluate_function()` method (around line 596)

**Add before function registry lookup:**
```rust
// Special handling for row expansion functions
if name_upper == "UNNEST" {
    // For UNNEST, we need to:
    // 1. Evaluate the first argument (the column value)
    // 2. Pass to row expander
    // 3. Store expansion info for query executor
    //
    // Problem: evaluate_function returns DataValue, not Vec<DataValue>
    // Solution: Store expansion metadata and return a special marker

    // For now, return an error directing to use row expansion
    return Err(anyhow::anyhow!(
        "UNNEST requires row expansion - must be handled at query executor level"
    ));
}
```

**Better approach:** Detect UNNEST at query executor level, **before** row-by-row evaluation.

### Step 2: Detect UNNEST in SELECT Items

**File:** `src/csv_datasource.rs` or wherever SELECT projection happens

**Pseudo-code:**
```rust
// Before processing rows, scan select_items for UNNEST
let has_unnest = select_items.iter().any(|item| {
    if let SelectItem::Expression { expr, .. } = item {
        contains_unnest(expr)  // Recursive check
    } else {
        false
    }
});

if has_unnest {
    // Use row expansion mode
    return execute_with_row_expansion(stmt, table);
}
```

### Step 3: Row Expansion Execution

**New function:** `execute_with_row_expansion()`

**Algorithm:**
```rust
fn execute_with_row_expansion(
    stmt: &SelectStatement,
    table: &DataTable,
) -> Result<DataTable> {
    let expander_registry = RowExpanderRegistry::new();
    let mut result_table = DataTable::new("result");

    // 1. Identify which SELECT items are UNNEST calls
    let expansion_info = analyze_unnest_items(&stmt.select_items);

    // 2. For each input row:
    for row_idx in 0..table.row_count() {
        // 3. Evaluate all UNNEST expressions to get arrays
        let mut expansion_arrays = Vec::new();
        for unnest_item in &expansion_info {
            let (column_value, delimiter) = evaluate_unnest_args(
                unnest_item, table, row_idx
            )?;

            let expander = expander_registry.get("UNNEST").unwrap();
            let expansion = expander.expand(
                &column_value,
                &[DataValue::String(delimiter)]
            )?;
            expansion_arrays.push(expansion);
        }

        // 4. Find max expansion count (for NULL padding)
        let max_count = expansion_arrays.iter()
            .map(|exp| exp.row_count())
            .max()
            .unwrap_or(1);

        // 5. Generate N output rows
        for output_row_idx in 0..max_count {
            let mut output_values = Vec::new();

            // For each SELECT item:
            for (col_idx, select_item) in stmt.select_items.iter().enumerate() {
                let value = if is_unnest_item(select_item) {
                    // Get value from expansion array (or NULL if exhausted)
                    let array = &expansion_arrays[get_unnest_index(col_idx)];
                    array.values.get(output_row_idx)
                        .cloned()
                        .unwrap_or(DataValue::Null)
                } else {
                    // Regular column - replicate from input row
                    evaluate_regular_expression(select_item, table, row_idx)?
                };

                output_values.push(value);
            }

            result_table.add_row(DataRow::new(output_values))?;
        }
    }

    Ok(result_table)
}
```

### Step 4: Helper Functions

```rust
// Check if expression contains UNNEST
fn contains_unnest(expr: &SqlExpression) -> bool {
    match expr {
        SqlExpression::FunctionCall { name, .. } => {
            name.to_uppercase() == "UNNEST"
        }
        SqlExpression::BinaryOp { left, right, .. } => {
            contains_unnest(left) || contains_unnest(right)
        }
        // ... handle other expression types
        _ => false
    }
}

// Analyze UNNEST items and build metadata
struct UnnestInfo {
    column_index: usize,
    column_expr: SqlExpression,
    delimiter_expr: SqlExpression,
    alias: String,
}

fn analyze_unnest_items(
    select_items: &[SelectItem]
) -> Vec<UnnestInfo> {
    // Extract UNNEST calls and their parameters
    // ...
}
```

## Key Design Decisions

### Why Not Modify DataValue?
- Don't add `DataValue::Array` type - too invasive
- Keep DataValue simple (String, Integer, Float, etc.)
- Row expansion is a query-time operation, not a value type

### Why Detect at Query Executor Level?
- arithmetic_evaluator works row-by-row (DataValue → DataValue)
- Row expansion needs to see ALL rows, ALL UNNEST calls
- Need to coordinate multiple UNNEST columns for NULL padding

### NULL Padding Strategy
```sql
-- Input: accounts="A|B|C", amounts="100,200"
-- Output:
-- Row 1: A, 100
-- Row 2: B, 200
-- Row 3: C, NULL  <- NULL padding for exhausted amounts
```

**Algorithm:**
1. Evaluate all UNNEST expressions for the row
2. Find max(array_lengths)
3. Generate max rows
4. Pad shorter arrays with NULL

## Testing Strategy

### Unit Tests (row_expanders module) ✅
Already done - basic splitting works

### Integration Tests (needed)

1. **Single UNNEST:**
```sql
SELECT order_id, UNNEST(accounts, '|') AS account
FROM fix_allocations
WHERE order_id = 'ORD001'
-- Expected: 3 rows (ORD001 replicated 3 times)
```

2. **Multiple UNNEST - Matching lengths:**
```sql
SELECT
  UNNEST(accounts, '|') AS account,
  UNNEST(amounts, ',') AS amount
FROM fix_allocations
WHERE order_id = 'ORD001'
-- Expected: 3 rows with matched pairs
```

3. **Multiple UNNEST - Mismatched lengths:**
```sql
SELECT
  UNNEST(accounts, '|') AS account,  -- 3 items
  UNNEST(amounts, ',') AS amount     -- 2 items
-- Expected: 3 rows, last amount is NULL
```

4. **UNNEST with WHERE:**
```sql
SELECT UNNEST(accounts, '|') AS account
FROM fix_allocations
WHERE msg_type = 'AS'
-- Expected: All AS rows expanded
```

5. **UNNEST with ORDER BY:**
```sql
SELECT order_id, UNNEST(accounts, '|') AS account
FROM fix_allocations
ORDER BY account
-- Expected: Rows sorted by expanded account values
```

## File Locations

**Implementation files to modify:**
- `src/csv_datasource.rs` - Main SELECT execution
- `src/data/query_engine.rs` - If that's the entry point
- OR find wherever `execute_select()` is implemented

**Search for:**
- `execute_select`
- `process_select_statement`
- Where SELECT items are projected into result rows

## Future Extensions

Once UNNEST works, the row expansion system enables:

### EXPLODE (JSON Arrays)
```sql
SELECT EXPLODE(json_column, '$.items[*]') AS item
FROM json_data
```

### GENERATE_SERIES (Range Expansion)
```sql
SELECT
  order_id,
  GENERATE_SERIES(1, quantity) AS item_number
FROM orders
```

### Custom Expansions
- Expand bit flags into separate rows
- Explode hierarchical data
- Time series generation

## Notes

- The `RowExpander` trait is designed to be **stateless**
- Each expansion is independent (no shared state between rows)
- Registry pattern makes it easy to add new expanders
- Clean separation: expanders know nothing about SQL/tables
