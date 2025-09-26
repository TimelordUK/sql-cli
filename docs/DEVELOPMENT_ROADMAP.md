# SQL CLI Development Roadmap

## 1. Bitwise Operations & Binary Visualization

### Goal
Add comprehensive bitwise operations and binary string visualization functions to enable low-level data analysis and bit manipulation.

### Functions to Implement

#### Bitwise Operators
- `BITAND(a, b)` - Bitwise AND operation
- `BITOR(a, b)` - Bitwise OR operation
- `BITXOR(a, b)` - Bitwise XOR (exclusive OR) operation
- `BITNOT(a)` - Bitwise NOT operation
- `BITSHIFT_LEFT(n, bits)` - Left shift operation
- `BITSHIFT_RIGHT(n, bits)` - Right shift operation

#### Binary Visualization
- `TO_BINARY(n)` - Convert number to binary string representation
- `TO_BINARY(n, width)` - With zero-padding to specified width
- `FROM_BINARY(str)` - Parse binary string to number
- `BINARY_FORMAT(n, separator)` - Format with separator (e.g., "1111_0000")

#### Bit Analysis Functions
- `IS_POWER_OF_TWO(n)` - Check if n is exact power of 2 using `n & (n-1) == 0`
- `COUNT_BITS(n)` - Count number of set bits (popcount)
- `HIGHEST_BIT(n)` - Position of highest set bit
- `LOWEST_BIT(n)` - Position of lowest set bit
- `REVERSE_BITS(n)` - Reverse bit order

### Example Usage
```sql
-- Check if number is power of two
SELECT value,
       TO_BINARY(value, 8) as binary,
       IS_POWER_OF_TWO(value) as is_pow2
FROM RANGE(0, 20);

-- Visualize bit operations
SELECT
    TO_BINARY(12, 8) as a,           -- "00001100"
    TO_BINARY(10, 8) as b,           -- "00001010"
    TO_BINARY(BITAND(12, 10), 8) as result  -- "00001000"
FROM DUAL;

-- Bit manipulation
SELECT
    value,
    BITSHIFT_LEFT(value, 2) as shifted,
    COUNT_BITS(value) as set_bits
FROM data;
```

### Implementation Location
- Create new module: `src/sql/functions/bitwise.rs`
- Register in function registry under new category: `FunctionCategory::Bitwise`

---

## 2. Web CTE Module Extraction

### Goal
Move Web CTE parsing and handling from `recursive_parser.rs` into a dedicated module for better organization and maintainability.

### Current State
- Web CTE parsing is embedded in `recursive_parser.rs`
- Mixed with general CTE parsing logic
- Makes the parser file unnecessarily large

### Target Structure
```
src/sql/
├── recursive_parser.rs     (general parsing)
├── web_cte/
│   ├── mod.rs              (public interface)
│   ├── parser.rs           (Web CTE specific parsing)
│   ├── spec.rs             (WebCTESpec structure)
│   └── validator.rs        (URL validation, method checking)
```

### Refactoring Steps
1. Create `src/sql/web_cte/` module structure
2. Extract `parse_web_cte_spec()` and related methods
3. Move WebCTESpec validation logic
4. Update imports in recursive_parser
5. Add comprehensive tests for Web CTE parsing

---

## 3. CTE Hoisting for Complex Expressions

### Goal
Automatically rewrite queries to hoist complex WHERE clause expressions into CTEs for better performance and compatibility.

### Problem Statement
Complex expressions in WHERE clauses can't always be evaluated efficiently. By hoisting them into CTEs, we can:
- Pre-compute expensive expressions once
- Enable window function usage in filtering
- Improve query optimization

### Example Transformation

**Before (user writes):**
```sql
SELECT *
FROM trades
WHERE ROW_NUMBER() OVER (PARTITION BY symbol ORDER BY time) <= 10
  AND DATEDIFF(NOW(), trade_date) < 30
```

**After (preprocessor rewrites to):**
```sql
WITH computed_trades AS (
    SELECT *,
           ROW_NUMBER() OVER (PARTITION BY symbol ORDER BY time) as rn,
           DATEDIFF(NOW(), trade_date) as days_ago
    FROM trades
)
SELECT *
FROM computed_trades
WHERE rn <= 10
  AND days_ago < 30
```

### Implementation Approach

#### Phase 1: AST Analysis
- Identify expressions that need hoisting:
  - Window functions in WHERE/HAVING
  - Expensive function calls used multiple times
  - Complex nested expressions

#### Phase 2: Dependency Analysis
- Track column dependencies
- Handle column name conflicts
- Preserve expression semantics

#### Phase 3: AST Rewriting
- Create new CTE nodes
- Move expressions to SELECT list
- Replace WHERE expressions with column references
- Update column references throughout

#### Phase 4: Optimization
- Merge compatible CTEs
- Eliminate redundant computations
- Preserve user-specified CTEs

### Challenges to Solve
1. **Column Scoping** - Ensure generated column names don't conflict
2. **Expression Dependencies** - Handle expressions that depend on other expressions
3. **Aggregate Context** - Properly handle aggregates vs window functions
4. **Performance** - Don't hoist simple expressions that are better inline

### Implementation Location
- New module: `src/query_rewriter/expression_hoister.rs`
- Integrate with existing `src/query_rewriter/mod.rs`
- Add configuration flags for hoisting behavior

---

## Priority & Effort Estimates

### Quick Wins (1-2 hours each)
1. **Web CTE Module Extraction** - Pure refactoring, low risk
2. **Basic Bitwise Operators** - Simple function implementations

### Medium Effort (3-5 hours)
1. **Binary Visualization Functions** - Need formatting logic
2. **Bit Analysis Functions** - More complex algorithms

### Large Effort (Days)
1. **CTE Expression Hoisting** - Complex AST transformation
2. **Full Query Rewriter** - Needs careful design and testing

## Next Steps

For tonight, we could start with either:
1. **Bitwise operators** - Self-contained, fun to implement, immediately useful
2. **Web CTE extraction** - Clean refactoring, improves codebase organization

Both are achievable in an evening session.