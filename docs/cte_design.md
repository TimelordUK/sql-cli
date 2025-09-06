# CTE (Common Table Expression) Design for SQL-CLI

## Overview
Add support for WITH clauses to enable filtering on window function results and create temporary named result sets.

## Key Insight
A CTE is essentially a named DataView that gets evaluated first, then used as the source for the main query.

## Proposed Implementation

### 1. Parser Changes (recursive_parser.rs)
```rust
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

### 2. Query Engine Flow
```rust
// In query_engine.rs
fn execute_query(expr: SqlExpression) -> Result<DataView> {
    match expr {
        SqlExpression::WithClause { ctes, query } => {
            // Step 1: Evaluate each CTE into a DataView
            let mut cte_views: HashMap<String, Arc<DataView>> = HashMap::new();
            
            for cte in ctes {
                let view = execute_query(cte.query)?;
                cte_views.insert(cte.name, Arc::new(view));
            }
            
            // Step 2: Execute main query with CTE context
            // Modify table resolution to check cte_views first
            execute_with_ctes(query, cte_views)
        }
        // ... existing cases
    }
}
```

### 3. Example Usage
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

-- Multiple CTEs
WITH 
    top_sales AS (
        SELECT * FROM test WHERE sales_amount > 20000
    ),
    ranked AS (
        SELECT *, ROW_NUMBER() OVER (ORDER BY sales_amount DESC) as rank
        FROM top_sales
    )
SELECT * FROM ranked WHERE rank <= 3;
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

## Benefits:
1. **Enables window function filtering**: `WHERE rank = 1` becomes possible
2. **Cleaner complex queries**: Break down logic into named steps
3. **Reusable subqueries**: Reference same CTE multiple times
4. **No performance penalty**: CTEs evaluated once, cached as DataView

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

## Conclusion:
Adding CTE support is very feasible given our architecture. The key insight is that CTEs are just named DataViews that get evaluated before the main query. Since we don't need JOIN support, this becomes a relatively straightforward extension of our existing query engine.

Estimated total implementation time: **6-8 hours**