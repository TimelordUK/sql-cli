# HAVING Clause - Current Support & Limitations

## Executive Summary

**Current Status:** HAVING clause is **partially implemented** and works well for common use cases.

**What Works:** ✅ HAVING with column aliases
**What Doesn't:** ❌ HAVING with aggregate functions like `COUNT(*)`, `SUM(col)`

**Recommendation:** Two-track approach:
1. **Short-term (1-2 hours):** Document current limitations and workarounds
2. **Long-term (CTE Hoister):** Full aggregate expression support via AST rewriting

---

## Current Implementation

### Architecture
**Location:** `src/data/group_by_expressions.rs` lines 288-320

**How it works:**
1. After aggregation completes for each group
2. Creates a **temporary single-row table** containing:
   - GROUP BY column values
   - Computed aggregate results (with their aliases)
3. Evaluates HAVING expression against this temp table
4. Filters out groups that don't match

**Key limitation:** Only GROUP BY columns and aggregate **aliases** are available to HAVING - original columns are not accessible.

---

## What Works Today ✅

### 1. HAVING with Aggregate Aliases
```sql
SELECT trader, SUM(notional) as total
FROM trades
GROUP BY trader
HAVING total > 50000;  -- ✅ Works!
```

### 2. HAVING with COUNT Alias
```sql
SELECT trader, COUNT(*) as trade_count
FROM trades
GROUP BY trader
HAVING trade_count > 1;  -- ✅ Works!
```

### 3. HAVING with Multiple Conditions
```sql
SELECT trader, SUM(qty) as total_qty, AVG(price) as avg_price
FROM trades
GROUP BY trader
HAVING total_qty > 100 AND avg_price > 200;  -- ✅ Works!
```

### 4. HAVING with Expressions on Aliases
```sql
SELECT trader, SUM(qty) as total_qty, COUNT(*) as count
FROM trades
GROUP BY trader
HAVING total_qty / count > 120;  -- ✅ Works!
```

### 5. HAVING with Comparison Operators
```sql
SELECT trader, MAX(price) as max_price, MIN(price) as min_price
FROM trades
GROUP BY trader
HAVING max_price - min_price > 100;  -- ✅ Works!
```

---

## What Doesn't Work ❌

### 1. HAVING with Aggregate Functions
```sql
SELECT trader, COUNT(*) as trade_count
FROM trades
GROUP BY trader
HAVING COUNT(*) > 1;  -- ❌ Fails: can't re-evaluate COUNT(*)
```

**Error:** Varies depending on aggregate:
- `COUNT(*)`: Returns 0 rows (incorrect result)
- `SUM(column)`: Error: "Column 'column' not found"

**Why:** The temp table only contains the **result** of aggregates, not the underlying data to re-compute them.

### 2. HAVING with Original Column References
```sql
SELECT trader, SUM(notional) as total
FROM trades
GROUP BY trader
HAVING notional > 10000;  -- ❌ Error: Column 'notional' not found
```

**Why:** Original columns are not included in the HAVING evaluation temp table.

### 3. HAVING with Complex Aggregate Expressions
```sql
SELECT trader, SUM(qty * price) as value
FROM trades
GROUP BY trader
HAVING SUM(qty * price) > 50000;  -- ❌ Error: Column 'qty' not found
```

**Why:** Can't access `qty` or `price` to re-compute the expression.

---

## Standard SQL Comparison

### What SQL Standard Allows
```sql
-- All of these are valid in PostgreSQL, MySQL, SQL Server:
HAVING COUNT(*) > 1                    -- ❌ We don't support
HAVING SUM(amount) > 1000              -- ❌ We don't support
HAVING AVG(price) > MAX(price) * 0.8   -- ❌ We don't support
HAVING total_amount > 1000             -- ✅ We support (alias)
```

### Implications
- **Most SQL tutorials** show `HAVING COUNT(*) > 1` as the canonical example
- Our users must learn to use aliases instead: `HAVING trade_count > 1`
- This is a significant deviation from SQL standards

---

## Workarounds (Current Best Practice)

### Pattern: Always Use Aliases
```sql
-- ❌ Don't do this (doesn't work):
SELECT trader, COUNT(*)
FROM trades
GROUP BY trader
HAVING COUNT(*) > 2;

-- ✅ Do this instead (works):
SELECT trader, COUNT(*) as trade_count
FROM trades
GROUP BY trader
HAVING trade_count > 2;
```

### Pattern: Pre-compute Complex Expressions in SELECT
```sql
-- ❌ Don't do this:
SELECT trader, SUM(qty * price) as value
FROM trades
GROUP BY trader
HAVING SUM(qty * price) > 50000;

-- ✅ Do this instead:
SELECT trader, SUM(qty * price) as value
FROM trades
GROUP BY trader
HAVING value > 50000;
```

---

## Solutions Analysis

### Option 1: Quick Documentation Fix (1 hour)
**Approach:** Document the limitation and workarounds

**Pros:**
- No code changes
- Zero risk
- Users can work around it today

**Cons:**
- Deviation from SQL standard
- Confusing for new users
- Doesn't solve the problem

**Effort:** 1 hour (update docs, add examples)

---

### Option 2: Re-evaluate Aggregates in HAVING (2-4 hours)
**Approach:** When HAVING contains an aggregate function, re-run it against the group

**Implementation:**
```rust
// In group_by_expressions.rs, line 308:
if let Some(having_expr) = having {
    // Detect if HAVING contains aggregate functions
    if contains_aggregate_function(having_expr) {
        // Re-evaluate aggregates against the original group data
        let having_result = evaluate_with_group_data(
            having_expr,
            &group_rows,  // Original rows in this group
            group_by_exprs
        )?;
    } else {
        // Current path: use aliases from temp table
        let having_result = evaluator.evaluate(having_expr, 0)?;
    }
}
```

**Pros:**
- Solves the immediate problem
- Relatively straightforward
- No parser changes needed

**Cons:**
- Duplicates aggregation logic
- Performance impact (re-computing aggregates)
- Still doesn't handle `SUM(qty * price)` case (original columns not available)
- Medium complexity

**Effort:** 2-4 hours

**Limitations:** Still can't handle expressions like `SUM(qty * price)` because original columns aren't available in the aggregate phase.

---

### Option 3: CTE Hoister Approach (Long-term, 8+ hours)
**Approach:** AST rewriting to convert HAVING with aggregates into a CTE + WHERE

**Transformation:**
```sql
-- Input SQL:
SELECT trader, COUNT(*) as trade_count
FROM trades
GROUP BY trader
HAVING COUNT(*) > 1;

-- Rewritten to:
WITH grouped AS (
  SELECT trader, COUNT(*) as trade_count
  FROM trades
  GROUP BY trader
)
SELECT trader, trade_count
FROM grouped
WHERE trade_count > 1;
```

**Pros:**
- Clean, elegant solution
- Handles ALL cases (including complex expressions)
- Aligns with SQL standard behavior
- Reuses existing WHERE clause logic
- No performance penalty (same work, different order)

**Cons:**
- Requires CTE hoister infrastructure
- More complex implementation
- Longer timeline

**Effort:** 8+ hours (requires mature CTE hoister)

**This is the "correct" long-term solution**

---

## Recommendation

### Immediate Action (Today - 1 hour)
1. ✅ Document current behavior clearly
2. ✅ Add examples showing alias-based HAVING
3. ✅ Add note: "For aggregate functions in HAVING, use aliases"
4. ✅ Create example file: `examples/having_clause_patterns.sql`

### Short-term (Next Session - Optional, 2-4 hours)
- Implement Option 2 (re-evaluate aggregates) if critical for your work
- Only if alias workaround is causing significant pain

### Long-term (Future - 8+ hours)
- Resume CTE Hoister project
- Implement AST rewriting for HAVING → CTE + WHERE
- This unblocks multiple features, not just HAVING

---

## Impact Assessment

### Current State
**User Experience:** Confusing for new users, but workable for power users who know the pattern

**Your Workflow:** Likely **not a blocker** since:
- You're already a power user who understands SQL deeply
- Alias-based HAVING works for most real-world cases
- You can work around it easily

### Priority Ranking
1. **High:** CROSS JOIN (✅ Done!)
2. **High:** Better error messages (✅ Done!)
3. **Medium:** HAVING improvements (can wait)
4. **High (long-term):** CTE Hoister (enables HAVING + other features)

---

## Examples of What Users Can Do Today

### Risk Analysis (Works)
```sql
SELECT
  trader,
  COUNT(*) as trade_count,
  SUM(notional) as total_exposure,
  AVG(price) as avg_price
FROM trades
GROUP BY trader
HAVING total_exposure > 1000000
   AND trade_count > 10
   AND avg_price > 100;
```

### Counterparty Limits (Works)
```sql
SELECT
  counterparty,
  SUM(exposure) as total_exposure,
  MAX(exposure) as max_single_trade
FROM exposures
GROUP BY counterparty
HAVING total_exposure > risk_limit
   AND max_single_trade > single_trade_limit;
```

### Volume Analysis (Works)
```sql
SELECT
  symbol,
  SUM(volume) as total_volume,
  COUNT(DISTINCT trader) as unique_traders
FROM trades
GROUP BY symbol
HAVING total_volume > 100000
   AND unique_traders > 5;
```

---

## Conclusion

**HAVING clause works well for the majority of real-world use cases** when users follow the alias pattern.

**For your trade workflows:** Alias-based HAVING should handle most scenarios. The limitation is more of a "SQL standards purity" issue than a practical blocker.

**Suggested Path Forward:**
1. Document current behavior (1 hour) ✅
2. Continue with other priorities (CTE Hoister, etc.)
3. Revisit HAVING when CTE Hoister is mature (long-term)

This avoids the trap of "heavy lifting just to say we can" while still acknowledging it as a known limitation.
