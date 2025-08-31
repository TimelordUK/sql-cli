# CASE WHEN Implementation Feasibility Analysis

## Overview
Analysis of implementing CASE WHEN expressions in the SQL CLI parser and evaluator.

## Current Architecture

### Parser Structure
The recursive parser (`src/sql/recursive_parser.rs`) currently supports:
- **SqlExpression enum**: Contains all expression types
- **Token enum**: Lexical tokens for parsing
- **parse_primary()**: Handles primary expressions
- **parse_expression()**: Handles complex expressions with operators

### Current Expression Types
```rust
pub enum SqlExpression {
    Column(String),
    StringLiteral(String), 
    NumberLiteral(String),
    FunctionCall { name, args },
    MethodCall { object, method, args },
    BinaryOp { left, op, right },
    InList { expr, values },
    NotInList { expr, values },
    Between { expr, lower, upper },
    Not { expr },
    // ... date/time constructors
}
```

## Required Changes for CASE WHEN

### 1. Parser Changes (MODERATE COMPLEXITY)

#### New Tokens
```rust
// Add to Token enum:
Case,
When, 
Then,
Else,
End,
```

#### New SqlExpression Variant
```rust
CaseExpression {
    when_branches: Vec<WhenBranch>,
    else_branch: Option<Box<SqlExpression>>,
}

struct WhenBranch {
    condition: Box<SqlExpression>,  // The WHEN condition
    result: Box<SqlExpression>,      // The THEN result
}
```

#### Parser Implementation
```rust
fn parse_case_expression(&mut self) -> Result<SqlExpression, String> {
    self.consume(Token::Case)?;
    let mut when_branches = Vec::new();
    
    // Parse WHEN clauses
    while matches!(self.current_token, Token::When) {
        self.advance(); // consume WHEN
        let condition = self.parse_expression()?;
        self.consume(Token::Then)?;
        let result = self.parse_expression()?;
        
        when_branches.push(WhenBranch {
            condition: Box::new(condition),
            result: Box::new(result),
        });
    }
    
    // Parse optional ELSE
    let else_branch = if matches!(self.current_token, Token::Else) {
        self.advance();
        Some(Box::new(self.parse_expression()?))
    } else {
        None
    };
    
    self.consume(Token::End)?;
    
    Ok(SqlExpression::CaseExpression {
        when_branches,
        else_branch,
    })
}
```

### 2. Lexer Changes (EASY)

Add new keywords to the lexer:
```rust
"CASE" | "case" => Token::Case,
"WHEN" | "when" => Token::When,
"THEN" | "then" => Token::Then,
"ELSE" | "else" => Token::Else,
"END" | "end" => Token::End,
```

### 3. Query Plan Output (EASY)

The AST will automatically display CASE expressions in --query-plan output once the parser changes are made.

### 4. Evaluator Changes (MODERATE COMPLEXITY)

#### ArithmeticEvaluator Enhancement
```rust
// In arithmetic_evaluator.rs
SqlExpression::CaseExpression { when_branches, else_branch } => {
    // Evaluate each WHEN condition in order
    for branch in when_branches {
        let condition_result = self.evaluate_as_bool(&branch.condition, row_index)?;
        if condition_result {
            return self.evaluate(&branch.result, row_index);
        }
    }
    
    // If no WHEN matched, evaluate ELSE (or return NULL)
    match else_branch {
        Some(else_expr) => self.evaluate(else_expr, row_index),
        None => Ok(DataValue::Null),
    }
}
```

## Implementation Complexity Assessment

### Parser Phase (2-3 hours)
1. **Lexer updates**: 30 minutes
   - Add 5 new tokens
   - Update keyword recognition

2. **AST definition**: 30 minutes
   - Add CaseExpression variant
   - Define WhenBranch structure

3. **Parser logic**: 1-2 hours
   - Implement parse_case_expression()
   - Integrate into parse_primary()
   - Handle nested CASE expressions

4. **Testing**: 30 minutes
   - Test query plan output
   - Verify AST structure

### Evaluator Phase (2-3 hours)
1. **ArithmeticEvaluator**: 1 hour
   - Implement CASE evaluation logic
   - Handle NULL cases

2. **RecursiveWhereEvaluator**: 1 hour
   - Support CASE in WHERE clauses
   - Boolean evaluation of CASE results

3. **Testing**: 1 hour
   - Test various CASE scenarios
   - Edge cases and NULL handling

## Total Estimated Effort: 4-6 hours

## Implementation Strategy

### Phase 1: Parser Only (Recommended First Step)
1. Implement lexer changes
2. Add AST structures
3. Implement parser logic
4. Test with --query-plan to verify correct parsing

### Phase 2: Evaluator
1. Implement arithmetic evaluation
2. Add WHERE clause support
3. Comprehensive testing

## Example Test Cases

### Basic CASE
```sql
SELECT 
    id,
    CASE 
        WHEN price > 100 THEN 'Expensive'
        WHEN price > 50 THEN 'Moderate'
        ELSE 'Cheap'
    END as price_category
FROM products
```

### CASE with Functions
```sql
SELECT
    CASE 
        WHEN price.Contains('.') THEN 'Decimal'
        WHEN MOD(ROUND(price, 0), 2) = 0 THEN 'Even'
        ELSE 'Odd'
    END as price_type
FROM trade_data
```

### Nested CASE
```sql
SELECT
    CASE 
        WHEN category = 'A' THEN
            CASE 
                WHEN price > 100 THEN 'Premium A'
                ELSE 'Standard A'
            END
        ELSE 'Other'
    END as classification
FROM items
```

## Conclusion

Implementing CASE WHEN is **moderately complex** but very achievable:

- **Parser changes**: Straightforward, following existing patterns
- **Evaluator changes**: Moderate complexity, but clear logic flow
- **No breaking changes**: Additive feature only
- **High value**: Enables powerful conditional logic in queries

The implementation can be done incrementally:
1. First get parsing working (verify with --query-plan)
2. Then add evaluation support
3. Finally add comprehensive tests

This would be a valuable addition to the SQL CLI's capabilities.