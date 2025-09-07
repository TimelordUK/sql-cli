# Subquery Implementation Plan

## Current State

### What Works
- ✅ **CTEs (WITH clauses)** - Fully functional, can chain multiple CTEs
- ✅ **CTE Architecture** - Perfect foundation for subqueries
- ✅ **FROM subquery parsing** - AST correctly parses `FROM (SELECT...) AS alias`
- ✅ **Materialization** - Can convert DataView to DataTable

### What Doesn't Work Yet
- ❌ **FROM subquery execution** - Parsed but fails during execution 
- ❌ **WHERE IN subqueries** - Not parsed yet
- ❌ **Scalar subqueries in SELECT** - Not parsed yet
- ❌ **EXISTS subqueries** - Not parsed yet
- ❌ **Correlated subqueries** - Would need variable binding

## The Key Insight

**Subqueries are just unnamed CTEs!** The same evaluation architecture works:

```sql
-- These are equivalent:
WITH sub AS (SELECT a, b FROM test WHERE a > 5)
SELECT * FROM sub WHERE b > 60;

SELECT * FROM (SELECT a, b FROM test WHERE a > 5) AS sub WHERE b > 60;
```

## Implementation Path

### 1. Fix FROM Subqueries (Easy)
The issue is in column resolution after materializing the subquery. The fix:

```rust
// In build_view_with_context, when handling from_subquery:
let source_table = if let Some(ref subquery) = statement.from_subquery {
    // Execute subquery - this already works!
    let subquery_result = self.build_view_with_context(
        table.clone(), 
        *subquery.clone(), 
        cte_context
    )?;
    
    // Materialize it - this works!
    let materialized = self.materialize_view(subquery_result)?;
    
    // THE FIX: Ensure column metadata is preserved correctly
    // Currently failing here - columns not found after materialization
    Arc::new(materialized)
}
```

### 2. WHERE IN Subqueries (Medium)
Add to the parser:

```rust
// In parse_in_expression:
if self.current_token == Token::LeftParen {
    if self.peek() == Token::Select {
        // Parse subquery
        let subquery = self.parse_select_statement()?;
        return Ok(SqlExpression::InSubquery { 
            expr: Box::new(left),
            subquery: Box::new(subquery)
        });
    }
}
```

Then in evaluation:
```rust
SqlExpression::InSubquery { expr, subquery } => {
    // Execute subquery to get list of values
    let subquery_result = self.execute_subquery(subquery)?;
    // Check if expr value is in the result set
}
```

### 3. Scalar Subqueries in SELECT (Medium)
Parse `(SELECT ...)` in expression context:

```rust
// In parse_primary:
Token::LeftParen => {
    if self.peek() == Token::Select {
        let subquery = self.parse_select_statement()?;
        self.consume(Token::RightParen)?;
        return Ok(SqlExpression::ScalarSubquery(Box::new(subquery)));
    }
}
```

Evaluate as a single value:
```rust
SqlExpression::ScalarSubquery(subquery) => {
    let result = self.execute_subquery(subquery)?;
    // Ensure exactly one row, one column
    // Return that value
}
```

### 4. EXISTS Subqueries (Easy)
Similar to IN but simpler - just check if subquery returns any rows.

### 5. Correlated Subqueries (Hard)
Would need to pass row context into subquery execution:

```sql
SELECT name, (SELECT MAX(amount) FROM orders o WHERE o.customer_id = c.id)
FROM customers c;
```

This requires variable binding and is significantly more complex.

## Why CTEs Are Actually Better

While implementing full subquery support is possible, CTEs are superior for most use cases:

1. **More Readable** - Named intermediate results
2. **Reusable** - Reference the same CTE multiple times
3. **Debuggable** - Can SELECT from CTEs directly to test
4. **Already Working** - Full support today!

## Recommendation

1. **Use CTEs** for complex queries - they work perfectly now
2. **Fix FROM subqueries** - Should be a small fix to column resolution
3. **Add WHERE IN** - Most commonly needed subquery type
4. **Document CTE patterns** - Show how to convert subqueries to CTEs

## Example Conversions

### Instead of FROM subquery:
```sql
-- Subquery (not working yet):
SELECT * FROM (SELECT a, b FROM test WHERE a > 5) AS sub WHERE b > 60;

-- CTE (works now!):
WITH sub AS (SELECT a, b FROM test WHERE a > 5)
SELECT * FROM sub WHERE b > 60;
```

### Instead of WHERE IN subquery:
```sql
-- Subquery (not supported):
SELECT * FROM orders WHERE customer_id IN (SELECT id FROM customers WHERE region = 'North');

-- CTE (works now!):
WITH north_customers AS (SELECT id FROM customers WHERE region = 'North')
SELECT * FROM orders o, north_customers nc WHERE o.customer_id = nc.id;
```

### Instead of scalar subquery:
```sql
-- Subquery (not supported):
SELECT name, (SELECT COUNT(*) FROM orders WHERE customer_id = c.id) as order_count
FROM customers c;

-- CTE with aggregation (works now!):
WITH order_counts AS (
    SELECT customer_id, COUNT(*) as order_count 
    FROM orders 
    GROUP BY customer_id
)
SELECT c.name, COALESCE(oc.order_count, 0) as order_count
FROM customers c, order_counts oc
WHERE c.id = oc.customer_id;
```

## Conclusion

The CTE implementation provides 90% of subquery functionality with better readability. The architecture is ready for full subquery support - it's mostly parser work since the evaluation engine (via CTEs) already handles the hard parts.