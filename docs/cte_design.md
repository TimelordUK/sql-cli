# CTE (Common Table Expression) & Subquery Design for SQL-CLI

## Overview
Add support for WITH clauses and FROM subqueries to enable filtering on computed expressions (like window functions) and create temporary named result sets.

## Key Insights
1. A CTE is essentially a named DataView that gets evaluated first, then used as the source for the main query.
2. **Subqueries in FROM and CTEs are fundamentally the same concept** - both create temporary named result sets (derived tables).
3. We can implement both features with unified logic, treating them as "derived tables".

## Unified Derived Tables Approach

### These are equivalent:
```sql
-- Subquery in FROM:
SELECT * FROM (
    SELECT n, IS_PRIME(n) as is_prime FROM numbers
) AS t WHERE is_prime = true;

-- CTE:
WITH t AS (
    SELECT n, IS_PRIME(n) as is_prime FROM numbers
)
SELECT * FROM t WHERE is_prime = true;
```

Both create a temporary named result set "t" that can be referenced in the main query.

## Proposed Implementation

### 1. Parser Changes (recursive_parser.rs)
```rust
// Unified representation for both CTEs and subqueries
enum TableSource {
    File(String),                    // Regular table from CSV/JSON
    DerivedTable {                   // Both CTE and subquery
        name: String,                // "t" in both examples
        query: Box<SqlExpression>,   // The SELECT that defines it
    }
}

// Add to SqlExpression enum
SqlExpression::WithClause {
    ctes: Vec<CTE>,
    query: Box<SqlExpression>,
}

struct CTE {
    name: String,
    query: SqlExpression,  // The SELECT that defines this CTE
}
```

### 2. Unified Query Context
```rust
struct QueryContext {
    // Map of available "tables" - could be CTEs or subqueries
    derived_tables: HashMap<String, Arc<DataView>>,
    physical_tables: HashMap<String, Arc<DataTable>>,
}

impl QueryContext {
    fn resolve_table(&self, name: &str) -> Result<Arc<DataView>> {
        // First check derived tables (CTEs/subqueries)
        if let Some(view) = self.derived_tables.get(name) {
            return Ok(Arc::clone(view));
        }
        
        // Then check physical tables
        if let Some(table) = self.physical_tables.get(name) {
            return Ok(Arc::new(DataView::new(Arc::clone(table))));
        }
        
        Err(anyhow!("Table '{}' not found", name))
    }
}
```

### 3. Unified Execution
```rust
fn execute_query_with_context(
    expr: SqlExpression,
    context: &mut QueryContext,
) -> Result<DataView> {
    match expr {
        // Handle subquery in FROM
        SqlExpression::Select { from: TableSource::DerivedTable { name, query }, .. } => {
            // Execute the subquery first
            let result = execute_query_with_context(*query, context)?;
            // Store it in context
            context.derived_tables.insert(name, Arc::new(result));
            // Continue with main query...
        }
        
        // Handle CTEs
        SqlExpression::WithClause { ctes, query } => {
            // Execute each CTE and add to context
            for cte in ctes {
                let result = execute_query_with_context(cte.query, context)?;
                context.derived_tables.insert(cte.name, Arc::new(result));
            }
            // Execute main query with CTEs available
            execute_query_with_context(*query, context)
        }
        // ... existing cases
    }
}
```

## Use Cases

### 1. Filtering on Computed Expressions (Main Goal)
```sql
-- Problem: Can't use alias in WHERE
SELECT n, IS_PRIME(n) as is_prime 
FROM numbers 
WHERE is_prime = true;  -- ERROR: Column 'is_prime' not found

-- Solution with CTE:
WITH prime_check AS (
    SELECT n, IS_PRIME(n) as is_prime FROM numbers
)
SELECT * FROM prime_check WHERE is_prime = true;

-- Solution with subquery:
SELECT * FROM (
    SELECT n, IS_PRIME(n) as is_prime FROM numbers
) AS prime_check 
WHERE is_prime = true;
```

### 2. Window Function Filtering
```sql
-- Get top performer from each region
WITH ranked AS (
    SELECT 
        region,
        salesperson,
        sales_amount,
        ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank
    FROM test
)
SELECT * FROM ranked WHERE rank = 1;
```

### 3. Complex Multi-Level Queries
```sql
-- Multiple levels of derived tables
WITH 
    primes AS (
        SELECT n FROM numbers WHERE IS_PRIME(n) = true
    ),
    prime_pairs AS (
        SELECT p1.n as first, p2.n as second 
        FROM primes p1, primes p2 
        WHERE p2.n = p1.n + 2  -- Twin primes
    )
SELECT * FROM (
    SELECT first, second, first * second as product 
    FROM prime_pairs
) AS products 
WHERE product < 1000;
```

## Implementation Complexity: MEDIUM

### What We Already Have:
1. ✅ DataView as data abstraction
2. ✅ Window function evaluation creating new columns
3. ✅ WHERE clause evaluation on computed columns
4. ✅ Table aliasing in FROM clause
5. ✅ Recursive descent parser structure

### What We Need to Add:
1. **Parser**: Recognize WITH keyword and parse CTE declarations
2. **AST**: New SqlExpression::WithClause variant
3. **Query Engine**: CTE evaluation before main query
4. **Table Resolution**: Check CTE names before looking for files

### Key Simplifications (No Joins):
- Each CTE produces a single DataView
- No need for complex join logic
- CTEs can reference earlier CTEs (or not, for simplicity)
- Main query treats CTEs like regular tables

## Estimated Implementation Steps:

### Phase 1: Parser (2-3 hours)
```rust
// In recursive_parser.rs
fn parse_statement(&mut self) -> Result<SqlExpression> {
    if self.current_token == Token::With {
        self.parse_with_clause()
    } else {
        self.parse_select_statement()
    }
}

fn parse_with_clause(&mut self) -> Result<SqlExpression> {
    self.expect(Token::With)?;
    let mut ctes = Vec::new();
    
    loop {
        let name = self.parse_identifier()?;
        self.expect(Token::As)?;
        self.expect(Token::LeftParen)?;
        let query = self.parse_select_statement()?;
        self.expect(Token::RightParen)?;
        
        ctes.push(CTE { name, query });
        
        if self.current_token != Token::Comma {
            break;
        }
        self.advance();
    }
    
    let main_query = self.parse_select_statement()?;
    Ok(SqlExpression::WithClause {
        ctes,
        query: Box::new(main_query),
    })
}
```

### Phase 2: Query Engine (2-3 hours)
- Modify execute_query to handle WithClause
- Pass CTE context through evaluation
- Update table resolution logic

### Phase 3: Testing (1-2 hours)
- Test window function filtering
- Test multiple CTEs
- Test nested references

## Key Advantages of Unified Approach

1. **Single implementation** - Both CTEs and subqueries share execution logic
2. **Parser simplification** - Can transform subqueries to CTEs internally
3. **Enables alias filtering** - Solve the "can't use alias in WHERE" problem
4. **Clean architecture** - One concept: "derived tables"
5. **No performance penalty** - Results cached as DataView, evaluated once

## Example Implementation Test:
```python
# In test_cte.py
def test_window_function_filtering():
    query = """
    WITH ranked AS (
        SELECT 
            region,
            salesperson,
            sales_amount,
            ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank
        FROM test
    )
    SELECT * FROM ranked WHERE rank = 1
    """
    
    results = run_query('sales_data.csv', query)
    
    # Should have exactly 4 rows (one per region)
    assert len(results) == 4
    
    # All should have rank = 1
    for row in results:
        assert row['rank'] == '1'
```

## Conclusion

The unified approach treating CTEs and FROM subqueries as "derived tables" is elegant and powerful:
- **Subqueries in FROM** are just inline CTEs
- **CTEs** are just named subqueries defined at the top
- Both create temporary DataViews that can be referenced by name

This solves the immediate problem of not being able to filter on aliased expressions (like `IS_PRIME(n) as is_prime`) while providing a foundation for more complex query composition.

Estimated total implementation time: **8-10 hours** for full CTE + subquery support