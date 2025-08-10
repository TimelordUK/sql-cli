# SQL Mathematical Extensions Design

## Overview
Add support for mathematical expressions in SELECT statements to enable computed columns and data transformations without requiring external processing.

## Priority Levels

### Phase 1: Basic Arithmetic (2-3 days)
**Goal:** Support basic mathematical operations on columns and literals

#### Supported Operations
- Addition: `quantity + 10`
- Subtraction: `price - discount`
- Multiplication: `quantity * price`
- Division: `total / quantity`
- Modulo: `id % 10`
- Parentheses: `(price - cost) * quantity`

#### Examples
```sql
-- Simple column arithmetic
SELECT quantity * price as total FROM orders

-- Mixed column and literal
SELECT price * 1.2 as price_with_tax FROM products

-- Complex expressions
SELECT (price - cost) / cost * 100 as margin_pct FROM products

-- Using aliases
SELECT 
    quantity,
    price,
    quantity * price as subtotal,
    quantity * price * 0.1 as tax
FROM order_items
```

### Phase 2: Mathematical Functions (3-5 days)
**Goal:** Add common mathematical functions

#### Functions to Implement
- `ROUND(value, decimals)` - Round to N decimal places
- `FLOOR(value)` - Round down
- `CEIL(value)` - Round up
- `ABS(value)` - Absolute value
- `POW(base, exponent)` - Power
- `SQRT(value)` - Square root
- `MOD(a, b)` - Modulo operation
- `GREATEST(a, b, ...)` - Maximum value
- `LEAST(a, b, ...)` - Minimum value

#### Examples
```sql
SELECT 
    ROUND(price * quantity, 2) as total,
    FLOOR(price) as price_floor,
    CEIL(quantity / 10) as boxes_needed,
    ABS(balance) as abs_balance
FROM transactions

SELECT 
    POW(growth_rate, years) as compound_growth,
    SQRT(variance) as std_deviation
FROM statistics
```

### Phase 3: Aggregate Functions (1-2 weeks)
**Goal:** Support aggregate calculations

#### Core Aggregates
- `SUM(expression)` - Sum of values
- `AVG(expression)` - Average
- `COUNT(*)` / `COUNT(column)` - Count rows
- `MIN(expression)` - Minimum value
- `MAX(expression)` - Maximum value
- `STDDEV(expression)` - Standard deviation
- `VARIANCE(expression)` - Variance

#### Examples
```sql
-- Simple aggregates
SELECT SUM(quantity * price) as total_revenue FROM orders
SELECT AVG(price) as average_price FROM products

-- With GROUP BY
SELECT 
    category,
    COUNT(*) as item_count,
    AVG(price) as avg_price,
    SUM(quantity) as total_quantity
FROM products
GROUP BY category

-- Mixed expressions and aggregates
SELECT 
    customer_id,
    COUNT(*) as order_count,
    SUM(quantity * price) as total_spent,
    AVG(quantity * price) as avg_order_value
FROM orders
GROUP BY customer_id
```

## Implementation Architecture

### 1. Parser Extensions

```rust
// Extend expression types
enum Expression {
    Column(String),
    Literal(Value),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },
    Function {
        name: String,
        args: Vec<Expression>,
    },
    Aggregate {
        func: AggregateFunction,
        expr: Box<Expression>,
        distinct: bool,
    },
}

enum BinaryOperator {
    Add, Subtract, Multiply, Divide, Modulo,
    GreaterThan, LessThan, Equal, NotEqual,
    And, Or,
}

enum UnaryOperator {
    Minus, Not,
}

enum AggregateFunction {
    Sum, Avg, Count, Min, Max, StdDev, Variance,
}
```

### 2. Expression Evaluator

```rust
struct ExpressionEvaluator {
    // Cache for computed values
    cache: HashMap<String, Value>,
}

impl ExpressionEvaluator {
    fn evaluate(&mut self, expr: &Expression, row: &Row) -> Result<Value> {
        match expr {
            Expression::BinaryOp { left, op, right } => {
                let lval = self.evaluate(left, row)?;
                let rval = self.evaluate(right, row)?;
                self.apply_binary_op(lval, op, rval)
            },
            Expression::Function { name, args } => {
                let arg_values = args.iter()
                    .map(|arg| self.evaluate(arg, row))
                    .collect::<Result<Vec<_>>>()?;
                self.apply_function(name, arg_values)
            },
            // ... other cases
        }
    }
}
```

### 3. Type System

```rust
enum DataType {
    Integer,
    Float,
    Decimal(precision, scale),
    String,
    Boolean,
    Date,
    DateTime,
}

// Type coercion rules
impl DataType {
    fn coerce(&self, other: &DataType) -> DataType {
        // Integer + Float = Float
        // Float + Decimal = Decimal
        // etc.
    }
}
```

### 4. Integration Points

1. **Parser Integration**
   - Extend `SelectItem` to include expressions
   - Update grammar to recognize operators
   - Add precedence rules (PEMDAS)

2. **Execution Integration**
   - Add computed columns to result set
   - Handle NULL values appropriately
   - Maintain column metadata

3. **Error Handling**
   - Division by zero
   - Type mismatches
   - Overflow/underflow
   - Invalid function arguments

## Testing Strategy

### Unit Tests
- Parser tests for each operator
- Evaluator tests for type coercion
- Function tests with edge cases

### Integration Tests
```sql
-- Test basic arithmetic
SELECT 2 + 2 as four
SELECT 10 / 3 as division
SELECT 10.0 / 3 as decimal_division

-- Test with NULL
SELECT NULL + 5 as null_math
SELECT COALESCE(NULL + 5, 0) as null_handled

-- Test type mixing
SELECT '10' + 5 as string_number
SELECT 10.5 * 2 as float_calc

-- Test complex expressions
SELECT (price * quantity) - (cost * quantity) as profit
```

## Performance Considerations

1. **Expression Caching**
   - Cache computed values for repeated expressions
   - Reuse parsed expression trees

2. **Vectorization**
   - Process columns in batches
   - Use SIMD instructions where possible

3. **Lazy Evaluation**
   - Only compute what's needed
   - Short-circuit boolean expressions

## Future Extensions

### Advanced Math
- Trigonometric functions (SIN, COS, TAN)
- Logarithmic functions (LOG, LN, LOG10)
- Statistical functions (MEDIAN, MODE, PERCENTILE)

### Date/Time Math
```sql
SELECT date_column + INTERVAL '1 day'
SELECT DATEDIFF(end_date, start_date) as duration
```

### String Operations
```sql
SELECT first_name || ' ' || last_name as full_name
SELECT SUBSTRING(description, 1, 100) as summary
```

### Window Functions
```sql
SELECT 
    price,
    AVG(price) OVER (PARTITION BY category) as category_avg,
    price - AVG(price) OVER (PARTITION BY category) as diff_from_avg
FROM products
```

## Dependencies

### Required Before Implementation
1. ✅ Parser infrastructure (already exists)
2. ✅ Expression evaluation for WHERE (already exists)
3. ✅ DataTable column management (already exists)

### Nice to Have
1. ⏳ Completed v10 refactor (cleaner integration)
2. ⏳ Modular parser design (easier to extend)
3. ⏳ Separate data processing pipeline

## Implementation Order

1. **Start Simple** (Week 1)
   - Basic arithmetic operators
   - Integer and float types only
   - No aggregates

2. **Add Functions** (Week 2)
   - ROUND, FLOOR, CEIL
   - Type coercion
   - NULL handling

3. **Add Aggregates** (Week 3-4)
   - SUM, AVG, COUNT
   - GROUP BY support
   - HAVING clause

## Success Metrics

- ✅ Can perform basic calculations without external tools
- ✅ Performance within 20% of native calculations
- ✅ Handles NULL and type mismatches gracefully
- ✅ Clear error messages for invalid expressions
- ✅ Compatible with existing SQL features

## Notes

- This feature would make SQL CLI a complete data analysis tool
- Prioritize correctness over performance initially
- Consider using existing expression evaluation libraries (evalexpr, etc.)
- Keep compatibility with standard SQL where possible