# SQL CLI AST Building and Walking Flow

## Overview
This document explains how queries flow through the SQL CLI system, from raw text to filtered results.

## Example Query
```sql
SELECT * FROM customers WHERE name.StartsWith('John') AND age > 25
```

## Phase 1: Tokenization (Lexical Analysis)
**Location**: `recursive_parser.rs` - `Lexer::next_token()`

```
Input:  "SELECT * FROM customers WHERE name.StartsWith('John') AND age > 25"
           ↓
Output: [Select, Star, From, Identifier("customers"), Where, 
         Identifier("name"), Dot, Identifier("StartsWith"), 
         LeftParen, StringLiteral("John"), RightParen, And, 
         Identifier("age"), GreaterThan, NumberLiteral("25")]
```

## Phase 2: AST Construction (Parsing)

### Main Parser (`recursive_parser.rs`)
```
Parser::parse() → parse_select_statement()
                     ↓
                  Creates SelectStatement {
                    columns: ["*"],
                    from_table: "customers",
                    where_clause: <delegates to WHERE parser>
                  }
```

### WHERE Parser (`where_parser.rs`)
```
WhereParser::parse() → parse_or_expr() → parse_and_expr() → parse_primary_expr()
                            ↓
                    Builds WhereExpr::And(
                      Box::new(WhereExpr::StartsWith("name", "John")),
                      Box::new(WhereExpr::GreaterThan("age", 25))
                    )
```

The parser uses **recursive descent** with operator precedence:
- OR (lowest precedence)
- AND (higher precedence)  
- Comparisons and method calls (highest)

## Phase 3: Query Execution

### Execution Dispatch (`enhanced_tui.rs`)
```
execute_query()
    ↓
    ├─→ API Client (remote data)
    ├─→ CSV Client (local files)
    └─→ Cache Client (in-memory data)
```

### Data Source Processing (`csv_datasource.rs`)
```
query()
  ↓
  ├── Parse query string → AST
  ├── Load/access data
  └── filter_results()
        ↓
        For each row:
          evaluate_where_expr(AST, row)
```

## Phase 4: AST Walking/Evaluation

### WHERE Expression Evaluation (`where_ast.rs`)
```
evaluate_where_expr(WhereExpr::And(left, right), row)
    ↓
    ├── evaluate_where_expr(left, row)  // StartsWith
    │     ↓
    │     row["name"].starts_with("John")
    │
    └── evaluate_where_expr(right, row) // GreaterThan
          ↓
          row["age"] > 25
```

## Key Differences: Building vs Walking

### AST Building (Parsing)
- **When**: Once per query
- **Where**: `recursive_parser.rs`, `where_parser.rs`
- **Purpose**: Convert text → structured tree
- **Process**: Recursive descent parsing with precedence
- **Result**: Immutable AST structure

### AST Walking (Evaluation)
- **When**: Once per row (potentially thousands of times)
- **Where**: `where_ast.rs::evaluate_where_expr()`
- **Purpose**: Apply filters to data
- **Process**: Recursive tree traversal
- **Result**: Boolean (include/exclude row)

## Performance Implications

1. **Parse Once, Evaluate Many**: The AST is built once but evaluated for every row
2. **Short-circuit Evaluation**: AND/OR operators can skip unnecessary evaluations
3. **Method Dispatch**: LINQ methods like `StartsWith` compile to direct Rust string operations

## Extension Points

### Adding New Operators
1. Add token type in `recursive_parser.rs` Token enum
2. Update lexer to recognize the token
3. Add WhereExpr variant in `where_ast.rs`
4. Implement parsing logic in `where_parser.rs`
5. Implement evaluation in `evaluate_where_expr()`

### Adding New LINQ Methods
1. Add to method list in `where_parser.rs` (line ~195)
2. Add WhereExpr variant
3. Implement evaluation logic
4. Update syntax highlighter and autocomplete

## Architecture Benefits

1. **Clean Separation**: Lexing, parsing, and evaluation are independent
2. **Type Safety**: AST is strongly typed, catching errors at compile time
3. **Extensibility**: New operators/functions only need changes in specific locations
4. **Optimization Opportunities**: AST can be optimized before evaluation
5. **Multiple Backends**: Same AST works for CSV, JSON, API data sources