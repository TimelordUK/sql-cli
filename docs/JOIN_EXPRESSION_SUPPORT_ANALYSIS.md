# JOIN Expression Support Analysis & Implementation Proposal

## Executive Summary

**Current State**: JOIN conditions only support simple column references
**User Need**: Support functions and expressions in JOIN conditions
**Complexity**: Medium - requires changes to parser, AST, and executor
**Benefit**: High - enables common real-world scenarios like TRIM(), case-insensitive joins, etc.

---

## 1. What Currently Works ✓

### Multiple Column Joins (Fully Supported)
```sql
-- Multiple AND conditions work perfectly
SELECT * FROM orders
JOIN executions
  ON orders.id = executions.order_id
 AND orders.account = executions.account
 AND orders.date = executions.date;
```

### All Comparison Operators
```sql
-- Equality, inequality, and range operators all work
SELECT * FROM buy
JOIN sell
  ON buy.instrument = sell.instrument
 AND buy.price > sell.price
 AND buy.quantity <> sell.quantity;
```

### Smart Algorithm Selection
- **Hash Join**: Used for single equality condition (O(n+m) complexity)
- **Nested Loop**: Used for multiple conditions or inequalities (O(n*m) complexity)
- Automatic selection based on condition analysis

### Advanced Features
- Subquery JOINs (DerivedTable)
- CTE JOINs
- Table aliases
- All JOIN types: INNER, LEFT, RIGHT, CROSS (FULL not yet implemented)

---

## 2. Critical Gap: No Expressions in JOIN Conditions ✗

### What Doesn't Work Today

**User's Exact Use Cases:**
```sql
-- ❌ Function on right side (padding/trimming issue)
ON fix_portfolio.portfolio = TRIM(fund_portfolio.Name)

-- ❌ Function on either side (string manipulation)
ON lhs.order_id = SUBSTRING(rhs.field, 1, 10)
ON lhs.order_id = SPLIT_LEFT(rhs.field, '.')

-- ❌ Case-insensitive comparison
ON LOWER(a.email) = LOWER(b.email)

-- ❌ Arithmetic expressions
ON users.id = orders.user_id + 1
ON ROUND(buy.price, 2) = ROUND(sell.price, 2)

-- ❌ Complex transformations
ON DATE_TRUNC('day', a.timestamp) = DATE_TRUNC('day', b.timestamp)
```

### Why It Doesn't Work

**Parser Limitation** (`src/sql/recursive_parser.rs:1820-1844`):
```rust
fn parse_single_join_condition(&mut self) -> Result<SingleJoinCondition, String> {
    let left_column = self.parse_column_reference()?;  // ❌ Only column names
    let operator = /* ... */;
    let right_column = self.parse_column_reference()?; // ❌ Only column names

    Ok(SingleJoinCondition {
        left_column,   // String, not Expression
        operator,
        right_column,  // String, not Expression
    })
}
```

**AST Limitation** (`src/sql/parser/ast.rs:481-487`):
```rust
pub struct SingleJoinCondition {
    pub left_column: String,    // ❌ Just a string
    pub operator: JoinOperator,
    pub right_column: String,   // ❌ Just a string
}
```

Compare this to WHERE clauses, GROUP BY, and HAVING which all use `SqlExpression`:
```rust
pub struct SelectStatement {
    pub where_clause: Option<WhereClause>,        // Uses SqlExpression
    pub group_by: Option<Vec<SqlExpression>>,     // Uses SqlExpression ✓
    pub having: Option<SqlExpression>,            // Uses SqlExpression ✓
    pub joins: Vec<JoinClause>,                   // Uses String ✗
}
```

---

## 3. Proposed Solution: Expression-Based JOIN Conditions

### Design Overview

Upgrade JOIN conditions to use `SqlExpression` (already defined in `ast.rs:136-200`) instead of `String`.

**Benefits:**
- Reuse existing expression evaluation infrastructure
- Consistent with WHERE, GROUP BY, HAVING
- Supports all functions in function registry
- Enables complex transformations

**Complexity:**
- **Parser changes**: Medium (swap `parse_column_reference()` → `parse_expression()`)
- **AST changes**: Low (change String → SqlExpression)
- **Executor changes**: Medium (evaluate expressions before comparison)

### Architecture Changes

#### Change 1: AST Structure
```rust
// BEFORE (src/sql/parser/ast.rs:481-487)
pub struct SingleJoinCondition {
    pub left_column: String,
    pub operator: JoinOperator,
    pub right_column: String,
}

// AFTER
pub struct SingleJoinCondition {
    pub left_expr: SqlExpression,    // Can be Column, FunctionCall, BinaryOp, etc.
    pub operator: JoinOperator,
    pub right_expr: SqlExpression,   // Can be Column, FunctionCall, BinaryOp, etc.
}
```

#### Change 2: Parser
```rust
// BEFORE (src/sql/recursive_parser.rs:1820-1844)
fn parse_single_join_condition(&mut self) -> Result<SingleJoinCondition, String> {
    let left_column = self.parse_column_reference()?;
    let operator = /* parse operator */;
    let right_column = self.parse_column_reference()?;

    Ok(SingleJoinCondition { left_column, operator, right_column })
}

// AFTER
fn parse_single_join_condition(&mut self) -> Result<SingleJoinCondition, String> {
    let left_expr = self.parse_expression()?;  // Supports functions, ops, etc.
    let operator = /* parse operator */;
    let right_expr = self.parse_expression()?; // Supports functions, ops, etc.

    Ok(SingleJoinCondition { left_expr, operator, right_expr })
}
```

#### Change 3: Executor

**Hash Join** (`src/data/hash_join.rs:221-529`):
Currently extracts column indices:
```rust
// BEFORE: Assumes simple column names
let left_col_idx = find_column_index(&left_table, &condition.left_column)?;
let right_col_idx = find_column_index(&right_table, &condition.right_column)?;
```

**AFTER**: Evaluate expressions for each row
```rust
// Build hash table
for row in left_table.rows {
    let key_value = evaluate_expression(&condition.left_expr, row, left_table)?;
    hash_map.entry(key_value).or_insert(vec![]).push(row);
}

// Probe hash table
for row in right_table.rows {
    let key_value = evaluate_expression(&condition.right_expr, row, right_table)?;
    if let Some(matching_rows) = hash_map.get(&key_value) {
        // Emit joined rows
    }
}
```

**Nested Loop Join** (`src/data/hash_join.rs:637-1092`):
Currently compares column values:
```rust
// BEFORE
let left_value = &left_row[left_idx];
let right_value = &right_row[right_idx];
let matches = compare_values(left_value, right_value, operator);
```

**AFTER**: Evaluate both expressions
```rust
let left_value = evaluate_expression(&condition.left_expr, left_row, left_table)?;
let right_value = evaluate_expression(&condition.right_expr, right_row, right_table)?;
let matches = compare_values(&left_value, &right_value, operator);
```

---

## 4. Implementation Complexity Analysis

### Low Complexity: AST Changes
- **Files**: `src/sql/parser/ast.rs`
- **Changes**: Rename fields, change types
- **Risk**: Low (breaking change, but straightforward)

### Medium Complexity: Parser Changes
- **Files**: `src/sql/recursive_parser.rs`
- **Changes**:
  - Replace `parse_column_reference()` with `parse_expression()`
  - Expression parser already exists and is mature
  - Need to handle qualified column names (table.column) in expressions
- **Risk**: Medium (parser context tracking for table references)

### Medium Complexity: Executor Changes
- **Files**:
  - `src/data/hash_join.rs` (hash join and nested loop algorithms)
  - `src/data/arithmetic_evaluator.rs` (expression evaluation)
- **Changes**:
  - Add expression evaluation in join execution
  - Handle table context for column resolution (left vs right table)
  - Performance consideration: expression evaluation per row pair in nested loop
- **Risk**: Medium (performance impact, context handling)

### High Complexity Areas to Watch
1. **Column Context Resolution**:
   - In JOIN condition `ON users.id = orders.user_id`, need to resolve which table each column comes from
   - Current expression evaluator may not have table context awareness
   - May need to pass table metadata to evaluator

2. **Performance**:
   - Hash join currently uses column index lookup (fast)
   - Expression evaluation per row is slower
   - May need to detect simple column references and optimize them

3. **Error Messages**:
   - Need clear errors when expression references wrong table
   - "Column 'users.name' not found in right table 'orders'"

---

## 5. Workarounds Available Today

While waiting for expression support, users can use CTEs to pre-compute transformations:

### Pattern: CTE Transformation
```sql
-- Instead of: ON a.portfolio = TRIM(b.Name)
-- Use CTE to pre-trim:

WITH trimmed_funds AS (
    SELECT
        fund_id,
        TRIM(Name) AS trimmed_name,
        other_columns
    FROM fund_portfolio
)
SELECT *
FROM fix_portfolio
JOIN trimmed_funds
  ON fix_portfolio.portfolio = trimmed_funds.trimmed_name;
```

### Pattern: Case-Insensitive Join
```sql
-- Instead of: ON LOWER(a.email) = LOWER(b.email)
-- Use CTE:

WITH normalized_users AS (
    SELECT user_id, LOWER(email) AS email_lower
    FROM users
),
normalized_contacts AS (
    SELECT contact_id, LOWER(email) AS email_lower
    FROM contacts
)
SELECT *
FROM normalized_users u
JOIN normalized_contacts c ON u.email_lower = c.email_lower;
```

**Pros**: Works today, readable, can be reused
**Cons**: Verbose, requires materialization, loses performance of hash join on expressions

---

## 6. Recommended Implementation Plan

### Phase 1: Simple Function Support (MVP)
**Goal**: Support functions on ONE side of join condition

Example: `ON portfolio = TRIM(fund_portfolio.Name)`

**Changes**:
1. AST: Change `right_column` to `right_expr: SqlExpression`
2. Parser: Parse right side as expression
3. Executor: Evaluate right expression for each right table row

**Benefit**: Solves most common use cases with minimal changes
**Complexity**: Low-Medium

### Phase 2: Full Expression Support
**Goal**: Support expressions on BOTH sides

Example: `ON LOWER(a.email) = LOWER(b.email)`

**Changes**:
1. AST: Change both fields to SqlExpression
2. Parser: Parse both sides as expressions
3. Executor: Evaluate both expressions
4. Add column context resolution

**Benefit**: Complete parity with WHERE clause
**Complexity**: Medium

### Phase 3: Optimization
**Goal**: Maintain performance for simple column-to-column joins

**Changes**:
1. Detect when expression is simple column reference
2. Use fast path (current index-based approach)
3. Only evaluate expressions when necessary

**Benefit**: No performance regression for existing queries
**Complexity**: Medium

---

## 7. Testing Strategy

### Parser Tests
```rust
#[test]
fn test_join_with_function() {
    let sql = "SELECT * FROM a JOIN b ON a.id = TRIM(b.name)";
    let ast = parse(sql).unwrap();
    // Verify right_expr is FunctionCall
}

#[test]
fn test_join_with_both_expressions() {
    let sql = "SELECT * FROM a JOIN b ON LOWER(a.email) = LOWER(b.email)";
    let ast = parse(sql).unwrap();
    // Verify both are FunctionCall
}
```

### Integration Tests
```sql
-- Test with actual data (tests/sql_examples/test_join_expressions.sql)

-- #! ../data/users_padded.csv
-- Test: TRIM() in join condition
WITH contacts AS (
    SELECT 1 as id, '  John  ' as name
)
SELECT * FROM users
JOIN contacts ON users.name = TRIM(contacts.name);
GO

-- Test: Case-insensitive email matching
WITH emails AS (
    SELECT 'User1' as source, 'JOHN@EXAMPLE.COM' as email
)
SELECT * FROM users
JOIN emails ON LOWER(users.email) = LOWER(emails.email);
GO

-- Test: Arithmetic in join
SELECT * FROM orders o1
JOIN orders o2 ON o1.id = o2.parent_id + 1;
GO
```

### Performance Tests
```bash
# Benchmark: Simple column join (should use hash join)
time ./target/release/sql-cli -q "SELECT * FROM large1 JOIN large2 ON large1.id = large2.id"

# Benchmark: Expression join (will use evaluated approach)
time ./target/release/sql-cli -q "SELECT * FROM large1 JOIN large2 ON large1.id = TRIM(large2.id)"

# Target: < 10% performance degradation for simple joins after changes
```

---

## 8. User Impact & Examples

### Real-World Use Cases Enabled

**1. String Normalization (User's Exact Problem)**
```sql
-- Fix padded spaces in database exports
SELECT *
FROM fix_portfolio
JOIN fund_portfolio
  ON fix_portfolio.portfolio = TRIM(fund_portfolio.Name);
```

**2. Case-Insensitive Matching**
```sql
-- Match emails regardless of case
SELECT *
FROM users
JOIN login_attempts
  ON LOWER(users.email) = LOWER(login_attempts.email);
```

**3. Date Truncation**
```sql
-- Join on date ignoring time component
SELECT *
FROM transactions t1
JOIN transactions t2
  ON DATE_TRUNC('day', t1.timestamp) = DATE_TRUNC('day', t2.timestamp)
 AND t1.account <> t2.account;
```

**4. Numeric Precision**
```sql
-- Join on rounded prices
SELECT *
FROM buy_orders
JOIN sell_orders
  ON buy_orders.instrument = sell_orders.instrument
 AND ROUND(buy_orders.price, 2) = ROUND(sell_orders.price, 2);
```

**5. String Extraction**
```sql
-- Join on extracted identifiers
SELECT *
FROM events e1
JOIN events e2
  ON SUBSTRING(e1.event_id, 1, 10) = SUBSTRING(e2.event_id, 1, 10)
 AND e1.event_id <> e2.event_id;
```

---

## 9. Alternative Approaches Considered

### Option 1: Add Specific Functions (REJECTED)
**Idea**: Add `TRIMJOIN()` or similar specialized functions

**Pros**: Simple, focused
**Cons**: Doesn't scale, inconsistent with SQL standards

### Option 2: Pre-Join Transformation Hint (REJECTED)
**Idea**: Special syntax like `JOIN ... ON TRANSFORM(TRIM) column = column`

**Pros**: Explicit performance control
**Cons**: Non-standard, doesn't support arbitrary expressions

### Option 3: Expression Support (RECOMMENDED)
**Idea**: Full `SqlExpression` support in JOIN conditions

**Pros**:
- Standard SQL behavior
- Consistent with WHERE/HAVING/GROUP BY
- Supports all existing functions
- Future-proof

**Cons**:
- More complex implementation
- Potential performance impact (mitigable with optimization)

---

## 10. Open Questions

1. **Should we support OR in join conditions?**
   - Current: Only AND is supported
   - Proposal: Keep AND-only for Phase 1/2, consider OR in Phase 3
   - Rationale: OR in joins is rare, complicates hash join algorithm

2. **Should we optimize simple column references?**
   - Current: N/A (no expressions yet)
   - Proposal: Yes, detect `SqlExpression::Column` and use fast path
   - Rationale: Avoid performance regression for existing queries

3. **How to handle aggregate functions in JOIN conditions?**
   - Example: `ON a.id = MAX(b.id)` (semantically unclear)
   - Proposal: Disallow aggregates in JOIN conditions (like WHERE clause)
   - Rationale: Aggregates belong in HAVING, not JOIN

4. **Column context for table qualification?**
   - Challenge: `ON table1.id = table2.id` - which table does each column come from?
   - Current evaluator may not have table context
   - Proposal: Extend evaluator to accept table context parameter

---

## 11. Conclusion & Recommendation

**Recommendation**: Implement in phases starting with Phase 1 (MVP)

**Justification**:
- Solves real user pain points (trimming, case-insensitive joins)
- Aligns with SQL standards
- Reuses existing expression infrastructure
- Medium complexity with manageable risk

**Next Steps**:
1. Create feature branch: `feature/join-expressions`
2. Implement Phase 1: Right-side expression support
3. Add comprehensive tests
4. Validate performance impact
5. If successful, proceed to Phase 2

**Estimated Effort**:
- Phase 1 (MVP): 2-3 days
- Phase 2 (Full): 3-4 days
- Phase 3 (Optimization): 2-3 days
- Testing: 1-2 days per phase

**Total**: 8-12 days for complete implementation

---

## Appendix: Code Location Reference

| Component | File Path | Lines |
|-----------|-----------|-------|
| Parser: JOIN parsing | `src/sql/recursive_parser.rs` | 1648-1869 |
| AST: JoinCondition | `src/sql/parser/ast.rs` | 481-502 |
| AST: SqlExpression | `src/sql/parser/ast.rs` | 136-200 |
| Executor: Hash Join | `src/data/hash_join.rs` | 221-529 |
| Executor: Nested Loop | `src/data/hash_join.rs` | 637-1092 |
| Evaluator: Expression eval | `src/data/arithmetic_evaluator.rs` | All |
| Query Engine: JOIN orchestration | `src/data/query_engine.rs` | ~1000-1070 |

---

**Document Version**: 1.0
**Date**: 2025-01-22
**Author**: Analysis based on codebase exploration
**Status**: Proposal - Awaiting decision
