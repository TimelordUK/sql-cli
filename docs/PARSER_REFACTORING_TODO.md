# Parser Refactoring & Debug Infrastructure TODO

Created: 2025-01-21
Priority: High - Critical for maintainability

## 1. Remove Hardcoded Generator Functions from Parser

### Current Problem
The parser has a hardcoded list of generator functions in `src/sql/recursive_parser.rs` (lines 543-558):
```rust
let is_generator = (name.to_uppercase() == "RANGE"
                    || name.to_uppercase() == "SPLIT"
                    || name.to_uppercase() == "TOKENIZE"
                    || name.to_uppercase() == "CHARS"
                    || name.to_uppercase() == "LINES"
                    || name.to_uppercase() == "SERIES"
                    || name.to_uppercase() == "DATES"
                    || name.to_uppercase().starts_with("GENERATE_")
                    || name.to_uppercase().starts_with("RANDOM_")
                    || name.to_uppercase() == "FIBONACCI"
                    || name.to_uppercase() == "PRIME_FACTORS"
                    || name.to_uppercase() == "COLLATZ"
                    || name.to_uppercase() == "PASCAL_TRIANGLE"
                    || name.to_uppercase() == "TRIANGULAR"
                    || name.to_uppercase() == "SQUARES"
                    || name.to_uppercase() == "FACTORIALS")
```

This violates the principle of the function registry being the single source of truth.

### Proposed Solution
1. **Add trait method to SqlFunction**:
   ```rust
   pub trait SqlFunction {
       fn signature(&self) -> FunctionSignature;
       fn evaluate(&self, args: &[DataValue]) -> Result<DataValue>;
       fn is_table_valued(&self) -> bool { false } // New method
   }
   ```

2. **Mark generator functions in registry**:
   - Override `is_table_valued()` to return `true` for all generator functions
   - Could also add a `FunctionType` enum: `Scalar`, `TableValued`, `Aggregate`, `Window`

3. **Parser queries the registry**:
   ```rust
   // Parser gets a reference to the function registry
   let is_generator = self.function_registry
       .get_function(&name.to_uppercase())
       .map_or(false, |f| f.is_table_valued());
   ```

4. **Benefits**:
   - Single source of truth
   - New generator functions automatically work
   - No parser updates needed when adding functions
   - Cleaner, more maintainable code

### Other Hardcoded Functions to Remove
- Search for other hardcoded function names in parser
- Move all function-specific logic to the registry

## 2. CLI Debug Trace Infrastructure

### Current Problem
- Adding/removing debug print statements for every debugging session
- No structured way to enable/disable debug output
- Hard to trace execution flow without modifying code

### Proposed Solution

1. **Add `--debug-trace` CLI flag** with levels:
   ```bash
   sql-cli --debug-trace parser   # Parser decisions only
   sql-cli --debug-trace eval     # Expression evaluation
   sql-cli --debug-trace exec     # Query execution
   sql-cli --debug-trace all      # Everything
   sql-cli --debug-trace parser,eval  # Multiple components
   ```

2. **Structured Debug Macros**:
   ```rust
   // Create debug macros that check debug level
   macro_rules! debug_parser {
       ($($arg:tt)*) => {
           if DEBUG_CONFIG.parser_enabled() {
               eprintln!("[PARSER] {}", format!($($arg)*));
           }
       }
   }

   macro_rules! debug_eval {
       ($($arg:tt)*) => {
           if DEBUG_CONFIG.eval_enabled() {
               eprintln!("[EVAL] {}", format!($($arg)*));
           }
       }
   }
   ```

3. **Key Decision Points to Trace**:

   **Parser**:
   - CTE detection and parsing
   - Generator function recognition
   - Parenthesis depth changes
   - Token lookahead decisions
   - Expression parsing choices
   - Function call parsing

   **Evaluator**:
   - Expression evaluation steps
   - Type coercion decisions
   - Function calls and arguments
   - NULL handling
   - Operator precedence

   **Query Executor**:
   - Pipeline stages (WHERE, GROUP BY, ORDER BY, etc.)
   - Row filtering decisions
   - Aggregate calculations
   - Join operations

4. **Implementation Plan**:
   - Add `DebugConfig` struct to hold debug settings
   - Pass through context or use global/thread-local
   - Replace existing debug prints with structured macros
   - Add debug output at all key decision points

## 3. Engine Architecture Goals

### Vision
"Get the engine to the point where all we're doing is plugging in new functions, generators, and aggregators"

### Current State
- Function registry ✓ (mostly working)
- But parser still has hardcoded knowledge
- Some special cases in evaluator
- Aggregate functions partially pluggable

### Target State

1. **Fully Pluggable Functions**:
   - All functions defined in registry
   - Parser has zero hardcoded function names
   - Evaluator just looks up and calls functions
   - Type checking driven by function signatures

2. **Pluggable Generators**:
   - Generator trait/interface
   - Register like regular functions
   - Parser identifies via registry metadata
   - Support for custom generators

3. **Pluggable Aggregates**:
   - Aggregate function trait
   - Register in function registry
   - Support for custom aggregates
   - Window function support

4. **Clean Extension Points**:
   ```rust
   // Adding a new function should be just:
   registry.register(Box::new(MyNewFunction));

   // Adding a generator:
   registry.register_generator(Box::new(MyGenerator));

   // Adding an aggregate:
   registry.register_aggregate(Box::new(MyAggregate));
   ```

## 4. Parser Token Handling Cleanup

### Current Problem
The parser is doing string comparisons instead of working with tokens:
```rust
// Bad - parser doing lexer's job:
if s.to_uppercase() == "ORDER"  // line 781
if id.to_uppercase() == "WEB"   // checking for WEB keyword
if name.to_uppercase() == "RANGE"  // checking generator names
```

### Root Cause
The lexer is returning generic `Token::Identifier` for keywords that should be specific tokens.

### Proposed Solution

1. **Extend Token Enum** for all SQL keywords:
   ```rust
   pub enum Token {
       // Keywords that should be tokens
       Order,
       By,
       Group,
       Having,
       With,
       As,
       From,
       Where,
       Select,
       Limit,
       Offset,
       Inner,
       Left,
       Right,
       Full,
       Outer,
       Join,
       On,
       Case,
       When,
       Then,
       Else,
       End,
       Web,  // For WEB CTEs
       // ... etc
   }
   ```

2. **Lexer handles case-insensitive keyword recognition**:
   ```rust
   // In lexer
   fn lex_identifier(&mut self) -> Token {
       let ident = self.read_identifier();
       match ident.to_uppercase().as_str() {
           "SELECT" => Token::Select,
           "FROM" => Token::From,
           "WHERE" => Token::Where,
           "ORDER" => Token::Order,
           "BY" => Token::By,
           "WEB" => Token::Web,
           // ... etc
           _ => Token::Identifier(ident)
       }
   }
   ```

3. **Parser uses token comparison**:
   ```rust
   // Clean parser code:
   if self.current_token == Token::Order {
       self.consume(Token::Order)?;
       self.consume(Token::By)?;
       // parse order by clause
   }
   ```

### Benefits
- Clear separation of concerns
- Parser is simpler and cleaner
- No string operations in parser
- Better performance (token comparison vs string comparison)
- Type safety - compiler catches typos
- Easier to add new keywords

### Migration Strategy
1. Identify all `to_uppercase()` calls in parser
2. Add corresponding tokens to Token enum
3. Update lexer to recognize keywords
4. Replace string comparisons with token comparisons
5. Remove all string manipulation from parser

## 5. Immediate Next Steps (Tomorrow)

1. **Phase 1: Debug Infrastructure** (Quick win)
   - Add `--debug-trace` CLI flag
   - Create debug macros
   - Add trace points in parser for CTE/generator detection
   - Test with complex queries

2. **Phase 2: Function Registry Metadata**
   - Add `is_table_valued()` to SqlFunction trait
   - Update all generator functions
   - Create `FunctionType` enum if needed

3. **Phase 3: Parser Refactoring**
   - Pass function registry reference to parser
   - Replace hardcoded generator list with registry lookup
   - Test thoroughly with all examples

4. **Phase 4: Document Extension Points**
   - Create developer guide for adding functions
   - Template/example for each function type
   - Automated tests for new functions

## 5. Testing Strategy

- Ensure all existing tests pass after each refactoring phase
- Add tests for debug trace output
- Create test harness for pluggable functions
- Benchmark to ensure no performance regression

## 6. Success Metrics

- Zero hardcoded function names in parser ✅
- New functions work without parser changes ✅
- Debug trace available without code changes ✅
- Clear documentation for extending the engine ✅
- All tests passing ✅

---

## Notes from Today's Debugging Session

The parenthesis depth tracking issue we fixed today highlighted the need for better debug infrastructure. We had to:
1. Add manual debug prints
2. Rebuild multiple times
3. Remove debug prints after fixing

With proper `--debug-trace`, we could have:
- Enabled parser trace from CLI
- Seen the depth tracking issue immediately
- No code changes needed for debugging

This would have saved significant time and made the debugging process much cleaner.