# JOIN Expression Support - Phase 2 Plan

## Executive Summary

**Goal:** Mirror Phase 1 functionality by supporting complex expressions on the **LEFT** side of JOIN conditions while the **RIGHT** side remains a simple column reference.

**Phase 1 Status:** ✅ Complete - Expressions on RIGHT side
**Phase 2 Target:** Expressions on LEFT side (RIGHT side simple)

---

## Current State (Phase 1)

### What Works Now ✅
```sql
-- RIGHT side can be an expression
SELECT * FROM portfolios
JOIN fund_names ON portfolios.portfolio = TRIM(fund_names.Name);
--                                        ^^^^^^^^^^^^^^^^^^^^^^
--                                        Expression on RIGHT
```

### What Doesn't Work Yet ❌
```sql
-- LEFT side must be a simple column
SELECT * FROM users
JOIN contacts ON TRIM(users.email) = contacts.email;
--               ^^^^^^^^^^^^^^^^^
--               Expression on LEFT - NOT SUPPORTED YET
```

---

## Phase 2 Target Use Cases

### Use Case 1: Case-Insensitive Matching (Left Side)
```sql
-- Normalize left side for matching
SELECT * FROM users
JOIN contacts ON LOWER(users.email) = contacts.email_lower;
```

### Use Case 2: Data Cleaning on Left
```sql
-- Remove padding from left side
SELECT * FROM legacy_data
JOIN master_table ON TRIM(legacy_data.code) = master_table.code;
```

### Use Case 3: Substring Extraction (Left Side)
```sql
-- Extract prefix from left side
SELECT * FROM transactions
JOIN codes ON SUBSTRING(transactions.ref_code, 1, 4) = codes.code;
```

### Use Case 4: Nested Functions (Left Side)
```sql
-- Complex transformation on left
SELECT * FROM external_data
JOIN internal ON UPPER(TRIM(external_data.name)) = internal.name_normalized;
```

---

## Implementation Strategy

### 1. AST Changes

**Current Structure:**
```rust
pub struct SingleJoinCondition {
    pub left_column: String,           // Simple column only
    pub operator: JoinOperator,
    pub right_expr: SqlExpression,     // Expression supported ✅
}
```

**Phase 2 Structure:**
```rust
pub struct SingleJoinCondition {
    pub left_expr: SqlExpression,      // Expression supported ✅ NEW!
    pub operator: JoinOperator,
    pub right_expr: SqlExpression,     // Already supports expressions ✅
}
```

**Migration Strategy:**
- Change `left_column: String` → `left_expr: SqlExpression`
- This is the EXACT mirror of what we did in Phase 1 for the right side
- All existing tests should continue to work since simple column references are a type of expression

### 2. Parser Changes

**Current Code (src/sql/recursive_parser.rs:1820-1844):**
```rust
fn parse_single_join_condition(&mut self) -> Result<SingleJoinCondition, String> {
    // Parse left column (can include table prefix)
    let left_column = self.parse_column_reference()?;  // Simple column only

    // Parse operator
    let operator = match &self.current_token {
        Token::Equal => JoinOperator::Equal,
        // ... other operators
    };
    self.advance();

    // Parse right side as expression (supports functions, columns, etc.)
    let right_expr = self.parse_expression()?;  // Expression ✅

    Ok(SingleJoinCondition {
        left_column,
        operator,
        right_expr,
    })
}
```

**Phase 2 Code:**
```rust
fn parse_single_join_condition(&mut self) -> Result<SingleJoinCondition, String> {
    // Parse left side as expression (supports functions, columns, etc.)
    let left_expr = self.parse_expression()?;  // Expression ✅ NEW!

    // Parse operator
    let operator = match &self.current_token {
        Token::Equal => JoinOperator::Equal,
        // ... other operators
    };
    self.advance();

    // Parse right side as expression (supports functions, columns, etc.)
    let right_expr = self.parse_expression()?;  // Expression ✅

    Ok(SingleJoinCondition {
        left_expr,    // NEW!
        operator,
        right_expr,
    })
}
```

### 3. Executor Changes (src/data/hash_join.rs)

**Algorithm Selection Logic:**

Phase 1 logic:
- If RIGHT side is complex expression → use nested loop
- If RIGHT side is simple column → can use hash join

Phase 2 logic (mirror):
- If **LEFT** side is complex expression → use nested loop
- If LEFT side is simple column AND right side is simple column → can use hash join
- If either side is complex → use nested loop

**Implementation:**

```rust
pub fn execute_join(
    &self,
    left_table: Arc<DataTable>,
    join_clause: &JoinClause,
    right_table: Arc<DataTable>,
) -> Result<DataTable> {
    // Check if both sides are simple columns
    let left_is_simple = Self::extract_simple_column_name(&single_condition.left_expr).is_some();
    let right_is_simple = Self::extract_simple_column_name(&single_condition.right_expr).is_some();

    let use_hash_join = all_equal && left_is_simple && right_is_simple;

    if use_hash_join {
        // Both sides simple - use optimized hash join
        self.hash_join_inner(...)
    } else {
        // At least one side has expression - use nested loop with evaluation
        self.nested_loop_join_inner_multi(...)
    }
}
```

**Nested Loop Changes:**

Current (Phase 1):
- LEFT side: Direct column lookup
- RIGHT side: Expression evaluation OR column lookup

Phase 2:
- LEFT side: Expression evaluation OR column lookup
- RIGHT side: Expression evaluation OR column lookup

```rust
fn nested_loop_join_inner_multi(
    &self,
    left_table: Arc<DataTable>,
    right_table: Arc<DataTable>,
    join_clause: &JoinClause,
) -> Result<DataTable> {
    let left_evaluator = ArithmeticEvaluator::new(Arc::clone(&left_table), self.case_insensitive);
    let right_evaluator = ArithmeticEvaluator::new(Arc::clone(&right_table), self.case_insensitive);

    for (left_idx, left_row) in left_table.rows.iter().enumerate() {
        for (right_idx, right_row) in right_table.rows.iter().enumerate() {
            let mut all_conditions_match = true;

            for condition in &join_clause.condition.conditions {
                // Evaluate LEFT expression (NEW! - was just column lookup before)
                let left_value = left_evaluator.evaluate(&condition.left_expr, left_idx)?;

                // Evaluate RIGHT expression (already working from Phase 1)
                let right_value = right_evaluator.evaluate(&condition.right_expr, right_idx)?;

                // Compare values
                if !self.compare_values(&left_value, &right_value, &condition.operator)? {
                    all_conditions_match = false;
                    break;
                }
            }

            if all_conditions_match {
                // Emit joined row
                result_rows.push(self.create_joined_row(left_row, right_row));
            }
        }
    }

    Ok(result_table)
}
```

### 4. AST Formatter Changes

**Current Code:**
```rust
// Left side is just a string
write!(f, "{}", condition.left_column)?;

// Right side is formatted as expression
self.format_expression(&condition.right_expr, f, 0)?;
```

**Phase 2 Code:**
```rust
// Left side formatted as expression
self.format_expression(&condition.left_expr, f, 0)?;

// Right side formatted as expression
self.format_expression(&condition.right_expr, f, 0)?;
```

---

## Migration Path

### Step 1: Update AST (Minimal Breaking Change)
- Change `left_column: String` to `left_expr: SqlExpression`
- Simple columns become `SqlExpression::ColumnReference`

### Step 2: Update Parser
- Replace `parse_column_reference()` with `parse_expression()` for left side
- This automatically supports both simple columns AND expressions

### Step 3: Update Executor
- Add left-side expression evaluation to nested loop join
- Update algorithm selection to check both sides for complexity

### Step 4: Update Formatter
- Format left side using expression formatter instead of string

### Step 5: Update Tests
- Existing tests continue to work (backward compatible)
- Add new tests for left-side expressions

---

## Backward Compatibility

✅ **100% Backward Compatible**

All existing queries continue to work because:
1. Simple column references are a type of `SqlExpression::ColumnReference`
2. Parser automatically handles simple columns via `parse_expression()`
3. Executor detects simple columns and uses optimized hash join when possible

**Examples:**
```sql
-- Simple column on left (existing behavior)
ON users.id = orders.user_id
-- Parsed as: SqlExpression::ColumnReference("users.id")

-- Expression on left (new Phase 2 behavior)
ON TRIM(users.name) = contacts.name
-- Parsed as: SqlExpression::FunctionCall("TRIM", [ColumnReference("users.name")])
```

---

## Testing Strategy

### Unit Tests
1. **Parser Tests**
   - Parse LEFT-side TRIM()
   - Parse LEFT-side UPPER()/LOWER()
   - Parse LEFT-side nested functions
   - Parse LEFT-side with RIGHT-side simple column

2. **Executor Tests**
   - Left TRIM() + right simple column
   - Left UPPER() + right simple column
   - Left SUBSTRING() + right simple column
   - Verify hash join NOT used when left has expression

3. **Integration Tests**
   - Two-file JOIN with left-side TRIM
   - Case-insensitive matching with left-side LOWER
   - Complex nested functions on left

### Example Test Queries

```sql
-- Test 1: LEFT TRIM, RIGHT simple
WITH
    WEB users AS (URL 'file://data/users_padded.csv' FORMAT CSV),
    WEB contacts AS (URL 'file://data/contacts.csv' FORMAT CSV)
SELECT users.*, contacts.phone
FROM users
JOIN contacts ON TRIM(users.email) = contacts.email;
GO

-- Test 2: LEFT UPPER, RIGHT simple
WITH
    WEB legacy AS (URL 'file://data/legacy_data.csv' FORMAT CSV),
    WEB master AS (URL 'file://data/master.csv' FORMAT CSV)
SELECT *
FROM legacy
JOIN master ON UPPER(legacy.code) = master.code_upper;
GO

-- Test 3: LEFT nested functions, RIGHT simple
SELECT *
FROM external
JOIN internal ON UPPER(TRIM(external.name)) = internal.name_clean;
GO
```

---

## Performance Characteristics

### Algorithm Selection

**Fast Path (Hash Join O(n+m)):**
- LEFT: Simple column reference
- RIGHT: Simple column reference
- Operator: Equality
→ Use optimized hash join

**Slow Path (Nested Loop O(n×m)):**
- LEFT: Complex expression OR
- RIGHT: Complex expression OR
- Operator: Inequality
→ Use nested loop with expression evaluation

### Expression Evaluation Cost

Phase 2 adds left-side evaluation:
- Phase 1: Evaluate right expression n×m times
- Phase 2: Evaluate left expression n×m times + right expression n×m times

**Optimization:** Could cache left-side evaluation results (future enhancement)

---

## Files to Modify

### Core Implementation
1. `src/sql/parser/ast.rs` - Change `left_column` to `left_expr`
2. `src/sql/recursive_parser.rs` - Parse left side as expression
3. `src/data/hash_join.rs` - Evaluate left expressions in nested loop
4. `src/sql/parser/ast_formatter.rs` - Format left side as expression

### Tests
5. `tests/test_join_parser.rs` - Update parser tests
6. `examples/join_left_expression.sql` - New examples (NEW FILE)
7. `data/users_padded.csv` - Test data (NEW FILE)

### Documentation
8. `docs/JOIN_EXPRESSION_PHASE2_COMPLETE.md` - Completion doc (NEW FILE)

---

## Implementation Checklist

- [ ] Update AST: `left_column: String` → `left_expr: SqlExpression`
- [ ] Update parser: `parse_column_reference()` → `parse_expression()`
- [ ] Update executor: Add left-side expression evaluation
- [ ] Update algorithm selection: Check both sides for complexity
- [ ] Update AST formatter: Format left side as expression
- [ ] Update all test files that reference `left_column`
- [ ] Create new test examples for left-side expressions
- [ ] Run cargo test
- [ ] Run cargo fmt
- [ ] Run integration tests
- [ ] Document completion

---

## Estimated Effort

**Implementation Time:** 2-3 hours

**Breakdown:**
- AST changes: 15 minutes
- Parser changes: 15 minutes
- Executor changes: 60 minutes
- Formatter changes: 10 minutes
- Test updates: 30 minutes
- New examples: 20 minutes
- Testing & verification: 30 minutes

---

## Success Metrics

✅ **Functional Success:**
- [ ] LEFT-side TRIM() works
- [ ] LEFT-side UPPER()/LOWER() works
- [ ] LEFT-side nested functions work
- [ ] Backward compatibility maintained
- [ ] All existing tests pass

✅ **Performance:**
- [ ] Hash join still used for simple column pairs
- [ ] Nested loop used when left OR right has expression
- [ ] No performance regression for simple queries

✅ **Code Quality:**
- [ ] cargo fmt passes
- [ ] cargo clippy passes
- [ ] All tests pass (library + integration)

---

## Phase 3 Preview (Future)

After Phase 2 completes, we could support expressions on **BOTH** sides simultaneously:

```sql
-- Phase 3 target: Both sides can be expressions
SELECT * FROM users
JOIN contacts ON LOWER(users.email) = LOWER(contacts.email);
```

This would require no additional AST or parser changes! Just executor optimization to avoid redundant evaluations.

---

**Implementation Date:** 2025-01-25 (planned)
**Status:** Ready to Implement
**Dependencies:** None (Phase 1 complete)
