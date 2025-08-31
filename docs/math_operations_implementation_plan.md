# Math Operations Implementation Plan

## API Design Decision: SQL Functions vs C#/LINQ vs Excel

### Recommendation: **SQL-style Functions** with **Excel function names**

**Rationale:**
1. **Familiar to most users**: Everyone knows `ROUND()`, `ABS()`, `MAX()`
2. **Standard SQL compatibility**: Works like PostgreSQL, MySQL, etc.
3. **Easier to extend**: Clear function namespace
4. **Better error reporting**: Function names are unambiguous

### API Comparison

| Style | Example | Pros | Cons |
|-------|---------|------|------|
| **SQL (Recommended)** | `ROUND(fees, 2)` | Familiar, Standard, Extensible | Slightly more verbose |
| C#/LINQ | `Math.Round(fees, 2)` | Familiar to .NET devs | Namespace conflicts, Complex parsing |
| Excel | `=ROUND(fees, 2)` | Universal familiarity | Not SQL-like, Confusing in context |

## Phase 1: Basic Arithmetic Operations

### Implementation Order

1. **Parser Extensions** (Day 1)
   - Add arithmetic operators to tokenizer
   - Extend SqlExpression AST
   - Add operator precedence (PEMDAS)

2. **Expression Evaluator** (Day 2)  
   - Implement arithmetic evaluation
   - Add type coercion (int + float = float)
   - Handle division by zero

3. **Integration** (Day 3)
   - Integrate with SELECT statement parsing
   - Add column aliasing (`as notional`)
   - Update query executor

### Target Syntax
```sql
-- Basic operations
SELECT quantity * price as notional FROM trades
SELECT (price - cost) / cost * 100 as margin_pct FROM products
SELECT commission + fees as total_cost FROM trades

-- Mixed types (auto-coerce)
SELECT quantity * 1.05 as adjusted_quantity FROM orders
SELECT price / 100.0 as price_in_dollars FROM products
```

### AST Structure
```rust
pub enum SqlExpression {
    // Existing variants...
    Column(String),
    StringLiteral(String),
    NumberLiteral(String),
    
    // NEW: Arithmetic expressions
    BinaryOp {
        left: Box<SqlExpression>,
        op: ArithmeticOp,
        right: Box<SqlExpression>,
    },
}

pub enum ArithmeticOp {
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // % (Phase 2)
}
```

## Phase 2: Math Functions (After arithmetic works)

### Function Syntax
```sql
-- Core functions (Excel-style names, SQL syntax)
SELECT ROUND(fees, 2) as rounded_fees FROM trades
SELECT ABS(balance) as abs_balance FROM accounts
SELECT FLOOR(price) as price_floor FROM products
SELECT CEILING(quantity / 10) as boxes_needed FROM inventory
SELECT POWER(growth_rate, years) as compound_growth FROM projections
```

### Implementation Approach
```rust
pub enum SqlExpression {
    // ... existing variants
    
    FunctionCall {
        name: String,              // "ROUND", "ABS", etc.
        args: Vec<SqlExpression>,  // Function arguments
    },
}

// In recursive_where_evaluator.rs (extend existing pattern)
match function_name.to_uppercase().as_str() {
    "ROUND" => {
        if args.len() != 2 { return Err("ROUND requires 2 arguments"); }
        let value = self.evaluate_expression(&args[0], row_index)?;
        let decimals = self.evaluate_expression(&args[1], row_index)?;
        // Implement rounding logic
    }
    "ABS" => {
        if args.len() != 1 { return Err("ABS requires 1 argument"); }
        let value = self.evaluate_expression(&args[0], row_index)?;
        // Return absolute value
    }
    // ... more functions
}
```

## Enhanced Error Reporting Strategy

### Current Problem
Parser errors are cryptic:
```
Error: Expected identifier at position 15
```

### Improved Error Messages
```
Error: Invalid arithmetic expression
  → SELECT quantity * price FROM trades
                     ^ Expected number or column name, found 'price'
  
Suggestion: Did you mean 'unit_price'? Available columns: quantity, unit_price, total
```

### Implementation Plan

1. **Error Context Enhancement**
```rust
pub struct ParseError {
    pub message: String,
    pub position: usize,
    pub length: usize,
    pub suggestions: Vec<String>,
    pub context: String,
}

impl ParseError {
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }
    
    pub fn format_with_context(&self, query: &str) -> String {
        // Format with ^^^^ pointing to error location
        // Show nearby valid options
    }
}
```

2. **Smart Suggestions**
- Column name fuzzy matching
- Function name suggestions
- Common syntax error fixes

3. **Expression Validation**
- Type checking before evaluation
- Operator precedence warnings
- NULL value handling hints

## Precedence & Parsing Rules

### Operator Precedence (Mathematical Order)
1. **Parentheses**: `(expression)`
2. **Unary**: `-value` (negative numbers)  
3. **Multiplication/Division**: `*`, `/`
4. **Addition/Subtraction**: `+`, `-`

### Grammar Extension
```rust
// Current: column.method() comparisons
// NEW: Support arithmetic in SELECT and WHERE

fn parse_expression(&mut self) -> Result<SqlExpression> {
    self.parse_additive()  // + and -
}

fn parse_additive(&mut self) -> Result<SqlExpression> {
    let mut left = self.parse_multiplicative()?;
    
    while matches!(self.current_token(), Some(Token::Plus) | Some(Token::Minus)) {
        let op = match self.current_token() {
            Some(Token::Plus) => ArithmeticOp::Add,
            Some(Token::Minus) => ArithmeticOp::Subtract,
            _ => unreachable!(),
        };
        self.advance();
        let right = self.parse_multiplicative()?;
        left = SqlExpression::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_multiplicative(&mut self) -> Result<SqlExpression> {
    // Similar pattern for * and /
}
```

## Integration Points

### 1. Tokenizer Updates (recursive_parser.rs)
```rust
enum Token {
    // ... existing tokens
    Plus,        // +
    Minus,       // -
    Multiply,    // *
    Divide,      // /
    LeftParen,   // (
    RightParen,  // )
}
```

### 2. Query Executor Updates
- Handle computed columns in result set
- Preserve column metadata for aliases
- Support mixed expression and column results

### 3. Type System Enhancements
```rust
impl DataValue {
    pub fn add(&self, other: &DataValue) -> Result<DataValue> {
        match (self, other) {
            (DataValue::Integer(a), DataValue::Integer(b)) => 
                Ok(DataValue::Integer(a + b)),
            (DataValue::Integer(a), DataValue::Float(b)) => 
                Ok(DataValue::Float(*a as f64 + b)),
            (DataValue::Float(a), DataValue::Float(b)) => 
                Ok(DataValue::Float(a + b)),
            // ... handle all combinations + NULL cases
            _ => Err(anyhow!("Cannot add {:?} and {:?}", self, other))
        }
    }
    
    // Similar for subtract, multiply, divide
}
```

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_basic_arithmetic() {
    let query = "SELECT 2 + 3 * 4 as result";
    let result = execute_query(query).unwrap();
    assert_eq!(result[0]["result"], DataValue::Integer(14)); // Not 20!
}

#[test]  
fn test_column_arithmetic() {
    let query = "SELECT quantity * price as total FROM test_table";
    // Test with sample data
}

#[test]
fn test_type_coercion() {
    let query = "SELECT 10 / 3.0 as result";
    let result = execute_query(query).unwrap();
    assert_eq!(result[0]["result"], DataValue::Float(3.333...));
}
```

### Error Testing
```rust
#[test]
fn test_division_by_zero() {
    let query = "SELECT 10 / 0 as result";
    let err = execute_query(query).unwrap_err();
    assert!(err.to_string().contains("Division by zero"));
}

#[test]
fn test_missing_operator() {
    let query = "SELECT quantity price as result FROM test";  // Missing *
    let err = execute_query(query).unwrap_err();
    assert!(err.to_string().contains("Expected operator"));
    // Should suggest: "Did you mean 'quantity * price'?"
}
```

## Implementation Steps

### Step 1: Start Simple (Today)
1. Add `+`, `-`, `*`, `/` tokens to parser
2. Create basic `BinaryOp` AST node  
3. Parse simple expressions: `a * b`
4. Implement evaluation for integers only

### Step 2: Extend (Next session)
1. Add parentheses support: `(a + b) * c`
2. Add type coercion: `int + float`
3. Handle edge cases: NULL values, division by zero
4. Add comprehensive tests

### Step 3: Functions (Future)
1. Add function call parsing: `ROUND(value, 2)`
2. Implement core math functions
3. Add better error messages with suggestions

This approach gives us a working foundation quickly, while setting up for advanced features later!