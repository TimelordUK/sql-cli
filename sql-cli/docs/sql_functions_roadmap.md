# SQL Functions & GROUP BY Roadmap

## Current Architecture Strengths
Our recursive descent parser is well-positioned for these extensions:
- Clean AST with SqlExpression enum
- Composable expression evaluation
- Existing ArithmeticEvaluator can be extended

## Feature: Math Functions

### Example Query
```sql
SELECT ROUND(quantity * price, 2) as rounded_total,
       ABS(profit - loss) as net_difference  
FROM trades
```

### Parser Extension (Minimal Changes)
```rust
// In parse_primary():
if let Token::Identifier(name) = &self.current_token {
    let name_upper = name.to_uppercase();
    // Check if it's a function
    if matches!(name_upper.as_str(), "ROUND" | "ABS" | "FLOOR" | "CEILING") {
        self.advance(); // consume function name
        self.consume(Token::LeftParen)?;
        
        let mut args = Vec::new();
        loop {
            args.push(self.parse_additive()?); // Parse any expression as argument
            if !matches!(self.current_token, Token::Comma) {
                break;
            }
            self.advance(); // consume comma
        }
        
        self.consume(Token::RightParen)?;
        return Ok(SqlExpression::FunctionCall {
            name: name_upper,
            args,
        });
    }
}
```

### Evaluator Extension
```rust
// In ArithmeticEvaluator::evaluate():
SqlExpression::FunctionCall { name, args } => {
    match name.as_str() {
        "ROUND" => {
            let value = self.evaluate(&args[0], row_idx)?;
            let precision = self.evaluate(&args[1], row_idx)?;
            // Implement rounding logic
        }
        "ABS" => {
            let value = self.evaluate(&args[0], row_idx)?;
            // Return absolute value
        }
        // ... more functions
    }
}
```

**Effort: 1-2 days** ✅ Very feasible

## Feature: GROUP BY with Aggregates

### Example Query
```sql
SELECT product_type,
       SUM(quantity * price) as total_revenue,
       COUNT(*) as num_transactions,
       AVG(price) as avg_price
FROM sales
GROUP BY product_type
```

### Parser (Already Mostly Ready!)
```rust
pub struct SelectStatement {
    pub select_items: Vec<SelectItem>,  // Can include FunctionCall for SUM()
    pub group_by: Option<Vec<String>>,  // Already exists!
    // Just need to recognize aggregate functions
}
```

### Query Engine Changes (More Complex)
```rust
// New method in QueryEngine:
fn apply_group_by(&self, view: DataView, 
                  group_cols: &[String], 
                  select_items: &[SelectItem]) -> Result<DataView> {
    // 1. Create groups (HashMap<Vec<DataValue>, Vec<usize>>)
    let mut groups = HashMap::new();
    for row_idx in view.visible_rows() {
        let key = extract_group_key(row_idx, group_cols);
        groups.entry(key).or_insert(vec![]).push(row_idx);
    }
    
    // 2. For each group, evaluate aggregates
    let mut result_table = DataTable::new("grouped");
    for (group_key, row_indices) in groups {
        let mut row = vec![];
        
        // Add group columns
        row.extend(group_key);
        
        // Evaluate aggregate functions
        for item in select_items {
            match item {
                SelectItem::FunctionCall { name: "SUM", args } => {
                    let sum = compute_sum(args, &row_indices);
                    row.push(sum);
                }
                // ... other aggregates
            }
        }
        result_table.add_row(row);
    }
    
    Ok(DataView::new(Arc::new(result_table)))
}
```

**Effort: 1-2 weeks** for full implementation

## Complexity Assessment

| Feature | Parser Complexity | Engine Complexity | Total Effort |
|---------|------------------|-------------------|--------------|
| Math Functions (ROUND, ABS) | Low | Low | 1-2 days |
| String Functions (UPPER, LOWER) | Low | Low | 1 day |
| Date Functions | Low | Medium | 2-3 days |
| Simple Aggregates (no GROUP BY) | Low | Medium | 2-3 days |
| GROUP BY (single column) | Low | High | 1 week |
| GROUP BY (multiple columns) | Low | High | 1-2 weeks |
| HAVING clause | Medium | Medium | 3-4 days |
| Window Functions | High | Very High | 3-4 weeks |

## Recommended Implementation Order

### Phase 1: Quick Wins (1 week total)
1. **Math Functions** - Immediate value, low effort
   - ROUND, ABS, FLOOR, CEILING, POWER
2. **String Functions** - User-requested, easy
   - UPPER, LOWER, LENGTH (already have), SUBSTRING
3. **Simple Aggregates** - No grouping needed
   - COUNT(*), SUM(col), AVG(col), MIN(col), MAX(col)

### Phase 2: Core GROUP BY (2 weeks)
1. Single column GROUP BY
2. Multiple column GROUP BY  
3. GROUP BY with all aggregate functions

### Phase 3: Advanced Features (optional)
1. HAVING clause
2. GROUP BY with expressions
3. DISTINCT within aggregates

## Trade-offs to Consider

### Option A: Full SQL Compliance
- **Pros**: Maximum compatibility, no surprises for users
- **Cons**: 6-8 weeks of effort, complex codebase

### Option B: Pragmatic Subset (Recommended)
- **Pros**: 80% of use cases in 20% of time, maintainable
- **Cons**: Some SQL won't work, need good error messages

### Option C: Delegate Complex Queries
- **Pros**: Minimal effort, leverage existing SQL engines
- **Cons**: Performance overhead, external dependencies

## Performance Considerations

With our in-memory architecture:
- GROUP BY on 100K rows: ~50-100ms (estimated)
- Aggregate functions: Very fast (simple loops)
- Nested functions: Some overhead but acceptable

The bigger challenge is memory usage for large GROUP BY result sets.

## Conclusion

Our parser architecture can definitely handle these features! The recursive descent design with a clean AST makes adding new SQL features straightforward. The main effort is in the query execution engine, not the parser.

**Recommended approach**: Start with math functions (immediate value, low effort), then tackle basic GROUP BY for the most common analytics use cases.