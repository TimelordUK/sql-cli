# Parser Analysis: Extending ORDER BY to Support Expressions

## Current State

### AST Structure
```rust
// Current: ORDER BY only supports column names (strings)
pub struct OrderByColumn {
    pub column: String,        // Just a column name!
    pub direction: SortDirection,
}
```

### Parser Implementation
Location: `src/sql/recursive_parser.rs:1309` (`parse_order_by_list()`)

**Current behavior:**
- Only parses **identifiers** (column names)
- Supports qualified names (`table.column`)
- Supports quoted identifiers
- Handles ASC/DESC keywords
- Returns `Vec<OrderByColumn>`

**Limitation:**
When parsing `ORDER BY SUM(sales_amount)`:
- Sees `SUM` as identifier
- Stops at `(` token
- Never parses the function call

## Required Changes

### 1. Update AST Structure

**Current:**
```rust
pub struct OrderByColumn {
    pub column: String,
    pub direction: SortDirection,
}
```

**Proposed:**
```rust
pub struct OrderByItem {
    pub expr: SqlExpression,   // Can be Column, FunctionCall, BinaryOp, etc.
    pub direction: SortDirection,
}
```

**Why this works:**
- `SqlExpression` already exists and handles all expression types
- Can represent:
  - Simple columns: `SqlExpression::Column(ColumnRef { name: "total", ... })`
  - Aggregates: `SqlExpression::FunctionCall { name: "SUM", args: [...], ... }`
  - Arithmetic: `SqlExpression::BinaryOp { ... }`
  - CASE: `SqlExpression::Case { ... }`

### 2. Update Parser Method

**Current signature:**
```rust
fn parse_order_by_list(&mut self) -> Result<Vec<OrderByColumn>, String>
```

**Proposed signature:**
```rust
fn parse_order_by_list(&mut self) -> Result<Vec<OrderByItem>, String>
```

**Implementation strategy:**
```rust
fn parse_order_by_list(&mut self) -> Result<Vec<OrderByItem>, String> {
    let mut order_items = Vec::new();

    loop {
        // Parse ANY expression (not just column names)
        let expr = self.parse_expression()?;

        // Check for ASC/DESC
        let direction = match &self.current_token {
            Token::Asc => {
                self.advance();
                SortDirection::Asc
            }
            Token::Desc => {
                self.advance();
                SortDirection::Desc
            }
            _ => SortDirection::Asc,
        };

        order_items.push(OrderByItem { expr, direction });

        // Handle comma-separated list
        if matches!(self.current_token, Token::Comma) {
            self.advance();
        } else {
            break;
        }
    }

    Ok(order_items)
}
```

**Why this works:**
- `parse_expression()` already exists and is battle-tested
- Handles ALL expression types:
  - Columns
  - Function calls (including aggregates)
  - Binary operators (+, -, *, /, etc.)
  - CASE expressions
  - Parentheses
  - Literals

### 3. Update SelectStatement AST

**Current:**
```rust
pub struct SelectStatement {
    // ...
    pub order_by: Option<Vec<OrderByColumn>>,
    // ...
}
```

**Proposed:**
```rust
pub struct SelectStatement {
    // ...
    pub order_by: Option<Vec<OrderByItem>>,
    // ...
}
```

### 4. Update Query Executor

The executor currently expects strings. We'll need to:

**Current execution:**
```rust
// Expects order_by to be Vec<OrderByColumn> with column: String
for order_col in order_by {
    // Look up column by name
    let column_name = &order_col.column;
    // ...
}
```

**Updated execution:**
```rust
// Needs to evaluate expressions
for order_item in order_by {
    match &order_item.expr {
        SqlExpression::Column(col_ref) => {
            // Simple column reference - use column name
            // ...
        }
        _ => {
            // Complex expression - evaluate it
            let value = evaluate_expression(&order_item.expr, row, context)?;
            // Use value for sorting
        }
    }
}
```

## Impact Analysis

### Files That Need Changes

1. **AST Definition** (`src/sql/parser/ast.rs`)
   - Add `OrderByItem` struct
   - Update `SelectStatement.order_by` type
   - Keep `OrderByColumn` as deprecated alias for compatibility

2. **Parser** (`src/sql/recursive_parser.rs`)
   - Update `parse_order_by_list()` to use `parse_expression()`
   - ~10 lines changed

3. **Query Executor** (`src/data/query_executor.rs`)
   - Update sorting logic to evaluate expressions
   - ~30-50 lines changed

4. **AST Formatter** (`src/sql/parser/ast_formatter.rs`)
   - Update to format ORDER BY expressions
   - ~10-20 lines

5. **All existing transformers** - NO CHANGES NEEDED!
   - Transformers work on `SqlExpression` already
   - `OrderByAliasTransformer` will work automatically

### Backward Compatibility

**Breaking change:** Yes, but minimal impact

**Migration strategy:**
```rust
// Helper function to create OrderByItem from column name (compatibility)
impl OrderByItem {
    pub fn from_column_name(name: String, direction: SortDirection) -> Self {
        Self {
            expr: SqlExpression::Column(ColumnRef {
                name,
                quote_style: QuoteStyle::None,
                table_prefix: None,
            }),
            direction,
        }
    }
}
```

## Benefits Beyond ORDER BY

This pattern can be reused for other clauses:

### 1. GROUP BY Expressions (Already Partially Done)
```sql
-- Currently supported (aliased)
GROUP BY YEAR(order_date)

-- Could support directly with same pattern
```

### 2. PARTITION BY in Window Functions
```sql
-- Already uses expressions, but could be more consistent
PARTITION BY CASE WHEN amount > 1000 THEN 'High' ELSE 'Low' END
```

### 3. Future: DISTINCT ON (PostgreSQL)
```sql
-- Would need expression support
DISTINCT ON (customer_id, order_date)
```

## Testing Strategy

### Unit Tests (Parser)
```rust
#[test]
fn test_order_by_simple_column() {
    // ORDER BY region
}

#[test]
fn test_order_by_aggregate() {
    // ORDER BY SUM(sales_amount)
}

#[test]
fn test_order_by_expression() {
    // ORDER BY sales_amount * 1.1
}

#[test]
fn test_order_by_case() {
    // ORDER BY CASE WHEN region = 'North' THEN 1 ELSE 2 END
}

#[test]
fn test_order_by_multiple() {
    // ORDER BY region, SUM(amount) DESC
}
```

### Integration Tests (SQL)
```sql
-- Test aggregate in ORDER BY
SELECT region, SUM(sales_amount) AS total
FROM sales
GROUP BY region
ORDER BY SUM(sales_amount) DESC;

-- Test expression in ORDER BY
SELECT region, sales_amount
FROM sales
ORDER BY sales_amount * 1.1 DESC;

-- Test CASE in ORDER BY
SELECT region, product
FROM sales
ORDER BY CASE WHEN region = 'North' THEN 1 ELSE 2 END;
```

## Implementation Phases

### Phase 1: AST Changes (30 min)
- Add `OrderByItem` struct to `ast.rs`
- Update `SelectStatement.order_by` type
- Add compatibility helper

### Phase 2: Parser Changes (1 hour)
- Update `parse_order_by_list()` to call `parse_expression()`
- Update all call sites
- Add unit tests

### Phase 3: Executor Changes (2 hours)
- Update sorting logic to handle expressions
- Add expression evaluation in sort key generation
- Handle edge cases (NULL, type mismatches)

### Phase 4: Formatter Changes (30 min)
- Update AST formatter for ORDER BY
- Ensure pretty-printing works

### Phase 5: Testing (1 hour)
- Add comprehensive tests
- Test with `--show-transformations`
- Verify transformer works

**Total estimated time: ~5 hours**

## Risk Assessment

### Low Risk
- Parser already has `parse_expression()` - battle-tested
- Executor already evaluates expressions elsewhere
- Transformers already work with expressions

### Medium Risk
- Executor sorting logic needs careful handling of:
  - Type mismatches (can't compare string to number)
  - NULL values in expression results
  - Performance (evaluating expressions for every row)

### Mitigation
- Comprehensive testing
- Error messages for type mismatches
- Consider caching evaluated expressions

## Performance Considerations

### Concern: Expression Evaluation Overhead
Evaluating `SUM(sales_amount)` for every row during sorting.

### Solution 1: Recognize Aggregate Aliases (Transformer)
```sql
-- Input
ORDER BY SUM(sales_amount)

-- Transformer rewrites to
ORDER BY __agg_1  -- Reference the alias from SELECT
```

This is exactly what `OrderByAliasTransformer` does!

### Solution 2: Expression Caching
Cache evaluation results during sort:
```rust
// Evaluate once per row, cache result
let sort_key = row_cache.get_or_insert(row_id, || {
    evaluate_expression(&expr, row, context)
});
```

## Conclusion

**Recommendation: Implement this change**

**Benefits:**
1. ✅ Supports standard SQL (`ORDER BY SUM(...)`)
2. ✅ Minimal code changes (~100 lines total)
3. ✅ Reuses existing infrastructure (`parse_expression()`)
4. ✅ Enables future features (GROUP BY expressions, etc.)
5. ✅ OrderByAliasTransformer will optimize automatically

**Effort:** ~5 hours total

**Risk:** Low - leverages existing, tested code

**Next Step:** Implement Phase 1 (AST changes)
