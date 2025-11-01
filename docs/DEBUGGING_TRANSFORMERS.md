# Debugging SQL Transformers

## Overview

SQL CLI provides powerful debugging tools to understand how queries are transformed before execution. This is critical when adding new transformers or debugging unexpected query behavior.

## Quick Start

```bash
# Show SQL transformations (recommended for understanding transformers)
./target/release/sql-cli -q "SELECT UPPER(region) AS r FROM sales GROUP BY r" \
    -d data/sales_data.csv \
    --show-transformations

# Works in script mode too
./target/release/sql-cli -f examples/expander_rewriters.sql --show-transformations
```

## Available Debugging Flags

### `--show-transformations` (NEW - Recommended)

**Purpose:** Shows the SQL before and after each transformation in a readable format.

**Output:**  Beautiful boxed display showing exactly what each transformer did.

**Example:**
```bash
./target/release/sql-cli -q "SELECT sales_amount * 1.1 AS adj FROM sales WHERE adj > 20000" \
    -d data/sales_data.csv \
    --show-transformations
```

**Output:**
```
╔════════════════════════════════════════════════════════════════╗
║ Transformer: WhereAliasExpander                                  ║
╠════════════════════════════════════════════════════════════════╣
║ BEFORE:                                                        ║
╠════════════════════════════════════════════════════════════════╣
  SELECT sales_amount * 1.1 AS adjusted
  FROM sales
  WHERE adjusted > 20000
╠════════════════════════════════════════════════════════════════╣
║ AFTER:                                                         ║
╠════════════════════════════════════════════════════════════════╣
  SELECT sales_amount * 1.1 AS adjusted
  FROM sales
  WHERE sales_amount * 1.1 > 20000
╚════════════════════════════════════════════════════════════════╝
```

**When to use:**
- ✅ Understanding what transformers are doing
- ✅ Debugging unexpected query behavior
- ✅ Learning how the preprocessor works
- ✅ Verifying transformer implementation

**When NOT to use:**
- ❌ Production queries (performance overhead)
- ❌ When output needs to be clean (debug output goes to stderr)

---

### `--show-preprocessing` (Existing)

**Purpose:** Shows internal verbose logging about the preprocessing pipeline.

**Output:** Log messages about which transformers are running and statistics.

**Example:**
```bash
./target/release/sql-cli -q "SELECT * FROM sales" \
    -d data/sales_data.csv \
    --show-preprocessing
```

**When to use:**
- Debugging pipeline configuration
- Seeing transformer statistics
- Understanding pipeline flow

---

### `--query-plan` (Existing)

**Purpose:** Shows the parsed AST structure before any transformations.

**Output:** Rust Debug format of the AST.

**Example:**
```bash
./target/release/sql-cli -q "SELECT * FROM sales WHERE region = 'North'" \
    -d data/sales_data.csv \
    --query-plan
```

**When to use:**
- Debugging parser issues
- Understanding AST structure
- Verifying query parsing

---

## Common Debugging Workflows

### Workflow 1: Understanding a Query Transformation

**Problem:** "Why is my query behaving differently than expected?"

**Solution:**
```bash
# Step 1: See what transformers are doing
./target/release/sql-cli -q "YOUR_QUERY" -d data.csv --show-transformations

# Step 2: If you need AST details
./target/release/sql-cli -q "YOUR_QUERY" -d data.csv --query-plan

# Step 3: Isolate which transformer is causing the issue
./target/release/sql-cli -q "YOUR_QUERY" -d data.csv --show-transformations \
    --no-expression-lifter \
    --no-where-expansion \
    # ... disable one at a time
```

### Workflow 2: Developing a New Transformer

**Problem:** "I'm adding a new transformer. How do I verify it works?"

**Solution:**
```bash
# Step 1: Write test query that should trigger your transformer
./target/release/sql-cli -q "YOUR_TEST_QUERY" -d data.csv --show-transformations

# Step 2: Verify the BEFORE/AFTER output shows your transformation

# Step 3: Test with complex queries
./target/release/sql-cli -f examples/expander_rewriters.sql --show-transformations

# Step 4: Add to examples file for regression testing
```

### Workflow 3: Debugging Transformer Interactions

**Problem:** "Transformers are interfering with each other"

**Solution:**
```bash
# Run with all transformers, note the order
./target/release/sql-cli -q "COMPLEX_QUERY" -d data.csv --show-transformations

# Disable transformers one by one to isolate
./target/release/sql-cli -q "COMPLEX_QUERY" -d data.csv --show-transformations --no-expression-lifter
./target/release/sql-cli -q "COMPLEX_QUERY" -d data.csv --show-transformations --no-where-expansion

# Check transformer order in pipeline (docs/PREPROCESSOR_TRANSFORMERS.md)
```

---

## Transformer Execution Order

Transformers run in this order (see pipeline.rs):

1. **ExpressionLifter** - Lifts complex expressions to CTEs
2. **WhereAliasExpander** - Expands aliases in WHERE
3. **GroupByAliasExpander** - Expands aliases in GROUP BY
4. **HavingAliasTransformer** - Auto-aliases aggregates in HAVING
5. **CTEHoister** - Hoists nested CTEs
6. **InOperatorLifter** - Optimizes IN lists

**Order matters!** ExpressionLifter must run before CTEHoister because it creates CTEs.

---

## Examples

### Example 1: WHERE Alias Expansion

```bash
./target/release/sql-cli -q "SELECT sales_amount * 1.1 AS adj FROM sales WHERE adj > 15000" \
    -d data/sales_data.csv \
    --show-transformations
```

**Shows:**
- Original query with `WHERE adj > 15000`
- Transformed to `WHERE sales_amount * 1.1 > 15000`

### Example 2: GROUP BY Alias Expansion

```bash
./target/release/sql-cli -q "SELECT UPPER(region) AS r, SUM(sales_amount) FROM sales GROUP BY r" \
    -d data/sales_data.csv \
    --show-transformations
```

**Shows:**
- Original query with `GROUP BY r`
- Transformed to `GROUP BY UPPER(region)`

### Example 3: HAVING Auto-Aliasing

```bash
./target/release/sql-cli -q "SELECT region, SUM(sales_amount) FROM sales GROUP BY region HAVING SUM(sales_amount) > 50000" \
    -d data/sales_data.csv \
    --show-transformations
```

**Shows:**
- Original query with `HAVING SUM(sales_amount) > 50000`
- Transformed to add alias to SELECT and use in HAVING

### Example 4: Expression Lifter (Window Functions)

```bash
./target/release/sql-cli -q "SELECT region, ROW_NUMBER() OVER (ORDER BY sales_amount) AS rn FROM sales WHERE ROW_NUMBER() OVER (ORDER BY sales_amount) <= 5" \
    -d data/sales_data.csv \
    --show-transformations
```

**Shows:**
- Original query with window function in WHERE
- Transformed to CTE with lifted expression

### Example 5: All Transformers

```bash
./target/release/sql-cli -f examples/expander_rewriters.sql --show-transformations
```

**Shows:**
- All transformers in action
- Multiple queries showcasing each feature

---

## Disabling Transformers for Debugging

Sometimes you need to isolate which transformer is causing an issue:

```bash
# Disable expression lifter
--no-expression-lifter

# Disable WHERE expansion
--no-where-expansion

# Disable GROUP BY expansion
--no-group-by-expansion

# Disable HAVING auto-aliasing
--no-having-expansion

# Disable CTE hoisting
--no-cte-hoister

# Disable IN operator lifting
--no-in-lifter

# Disable all (useful for baseline testing)
--no-expression-lifter \
--no-where-expansion \
--no-group-by-expansion \
--no-having-expansion \
--no-cte-hoister \
--no-in-lifter
```

**Example:**
```bash
# Test query without WHERE expansion
./target/release/sql-cli -q "SELECT sales_amount * 1.1 AS adj FROM sales WHERE adj > 15000" \
    -d data/sales_data.csv \
    --no-where-expansion
# Should fail with "Column 'adj' not found"
```

---

## Performance Considerations

### Overhead of Debugging Flags

| Flag | Overhead | When to Use |
|------|----------|-------------|
| `--show-transformations` | Medium (~5-10ms) | Development/debugging only |
| `--show-preprocessing` | Low (~1-2ms) | Development/debugging only |
| `--query-plan` | Low (~1ms) | Anytime (minimal impact) |
| No flags | None | Production |

**Recommendation:** Only use debugging flags during development. Disable for production queries.

---

## Comparing Before and After

Sometimes you want to see the final SQL that's executed:

```bash
# Step 1: See transformations
./target/release/sql-cli -q "SELECT UPPER(region) AS r FROM sales GROUP BY r" \
    -d data/sales_data.csv \
    --show-transformations

# Step 2: Copy the "AFTER" SQL from the last transformer

# Step 3: Run directly to verify
./target/release/sql-cli -q "SELECT UPPER(region) AS r FROM sales GROUP BY UPPER(region)" \
    -d data/sales_data.csv

# Should produce identical results
```

---

## Troubleshooting Common Issues

### Issue 1: Transformation Not Showing

**Problem:** Expect a transformer to run, but don't see output.

**Cause:** Transformer didn't detect a pattern to transform.

**Solution:**
```bash
# Verify transformer is enabled (default: all enabled)
./target/release/sql-cli -q "YOUR_QUERY" -d data.csv --show-preprocessing

# Check if query matches transformer's pattern
# See docs/PREPROCESSOR_TRANSFORMERS.md for patterns
```

### Issue 2: Too Much Output

**Problem:** `--show-transformations` shows many transformers, hard to read.

**Solution:**
```bash
# Disable transformers you don't care about
./target/release/sql-cli -q "YOUR_QUERY" -d data.csv --show-transformations \
    --no-cte-hoister \
    --no-in-lifter

# Or pipe through less for scrolling
./target/release/sql-cli -f examples/expander_rewriters.sql --show-transformations 2>&1 | less
```

### Issue 3: Understanding AST Format

**Problem:** `--query-plan` output is hard to read (Rust Debug format).

**Solution:**
Use `--show-transformations` instead - it shows formatted SQL which is much more readable.

---

## Integration with Testing

### Manual Testing

```bash
# Run examples with transformations to see all features
./target/release/sql-cli -f examples/expander_rewriters.sql --show-transformations

# Test specific feature
./target/release/sql-cli -q "YOUR_TEST_QUERY" -d data.csv --show-transformations
```

### Automated Testing

The transformation output goes to `stderr`, so it won't interfere with automated tests that check `stdout`:

```bash
# Test still works (stdout is clean)
./target/release/sql-cli -q "SELECT * FROM sales" -d data/sales_data.csv -o csv > output.csv

# Debug output goes to stderr
./target/release/sql-cli -q "SELECT * FROM sales" -d data/sales_data.csv -o csv --show-transformations 2> debug.log
```

---

## Best Practices

1. **Always use `--show-transformations` when debugging** - It's the most useful flag
2. **Check transformation order** - Transformers run in a specific order
3. **Disable unnecessary transformers** - Reduces noise in output
4. **Copy-paste transformed SQL** - Verify it works directly
5. **Add examples** - If you find a good test case, add to `examples/expander_rewriters.sql`

---

## Summary

| Need | Use This Flag | Example |
|------|--------------|---------|
| See what transformers did | `--show-transformations` | See SQL before/after each step |
| Debug parser | `--query-plan` | See raw AST |
| Pipeline stats | `--show-preprocessing` | See timing/stats |
| Disable specific transformer | `--no-where-expansion` etc. | Test without a transformer |
| Production query | (no flags) | No overhead |

**The `--show-transformations` flag is your best friend when working with transformers!**

---

## Related Documentation

- **Transformer Guide:** `docs/PREPROCESSOR_TRANSFORMERS.md`
- **Feature Roadmap:** `docs/SQL_FEATURE_GAPS_AND_ROADMAP.md`
- **Examples:** `examples/expander_rewriters.sql`
- **Main Guide:** `CLAUDE.md`
