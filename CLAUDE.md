# SQL CLI Project - AI Context Guide

Vim-like terminal SQL editor with in-memory query engine for ultra-fast navigation and data exploration. Built in Rust using ratatui.

## 🚀 QUICK START - Essential Commands

```bash
# When in doubt about available functions:
./target/release/sql-cli --list-functions           # List all SQL functions
./target/release/sql-cli --function-help CONVERT    # Get help for specific function

# Test SQL queries quickly:
./target/release/sql-cli -q "SELECT CONVERT(100, 'km', 'miles')" -o csv
./target/release/sql-cli data/test.csv -q "SELECT * FROM test WHERE id > 5" -o csv

# Test examples:
./scripts/test_all_examples.sh                      # Run all SQL examples
```

## 🏗️ CORE ARCHITECTURE - Key Files

### Entry Points & CLI
- `src/main.rs` - CLI entry point, argument parsing, mode selection (TUI vs non-interactive)
- `src/non_interactive.rs` - Handles `-q` queries, script execution, output formatting

### SQL Parsing & Execution Pipeline
1. **Parser**: `src/sql/recursive_parser.rs` - Recursive descent parser, builds AST
   - Parses SELECT, WHERE, GROUP BY, ORDER BY, functions, expressions
   - Returns `SelectStatement` AST structure

2. **Evaluator**: `src/data/arithmetic_evaluator.rs` - Evaluates expressions against data
   - Handles arithmetic, comparisons, function calls
   - **IMPORTANT**: Do NOT add function implementations here - use function registry

3. **Query Executor**: `src/data/query_executor.rs` - Orchestrates query execution
   - Applies WHERE filters, GROUP BY, ORDER BY, LIMIT

### Function System (CRITICAL)
- **Registry**: `src/sql/functions/mod.rs` - Central function registry
  - ALL new functions must be registered here
  - Categories: Mathematical, String, Date, Conversion, etc.
  
- **Adding New Functions**:
  1. Create implementation in `src/sql/functions/<category>.rs`
  2. Implement `SqlFunction` trait
  3. Register in `mod.rs` under appropriate category
  4. Function automatically available in CLI and help

- **Example Function Modules**:
  - `src/sql/functions/math.rs` - Mathematical functions
  - `src/sql/functions/string_methods.rs` - String manipulation
  - `src/sql/functions/convert.rs` - Unit conversions (CONVERT function)
  - `src/sql/functions/astronomy.rs` - Astronomical constants
  - `src/sql/functions/chemistry.rs` - Chemical elements

### Data Structures
- `src/data/datatable.rs` - Core data table structure (columns, rows, types)
- `src/data/data_view.rs` - View layer with sorting, filtering, column operations
- `src/data/csv_datasource.rs` - CSV loading and parsing
- `src/data/json_datasource.rs` - JSON data handling

### TUI Components (for interactive mode)
- `src/ui/enhanced_tui.rs` - Main TUI interface
- `src/app_state_container.rs` - Central state management
- `src/action.rs` - Action system for state updates
- `src/handlers/` - Event handlers for keyboard input

### Unit Conversion System
- `src/data/unit_converter.rs` - Core conversion logic
- `src/sql/functions/convert.rs` - CONVERT() SQL function
- Supports: temperature, distance, weight, volume, area, speed, pressure, time, energy

## 📝 DEVELOPMENT WORKFLOW

### Adding a New SQL Function

1. **Choose the right category** or create new one in `src/sql/functions/mod.rs`:
   ```rust
   pub enum FunctionCategory {
       Mathematical,
       String,
       Date,
       Conversion,  // etc.
   }
   ```

2. **Create function implementation**:
   ```rust
   // In src/sql/functions/your_category.rs
   pub struct YourFunction;
   
   impl SqlFunction for YourFunction {
       fn signature(&self) -> FunctionSignature {
           FunctionSignature {
               name: "YOUR_FUNC",
               category: FunctionCategory::YourCategory,
               arg_count: ArgCount::Fixed(2),
               description: "What it does",
               returns: "Return type",
               examples: vec!["SELECT YOUR_FUNC(arg1, arg2)"],
           }
       }
       
       fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
           // Implementation
       }
   }
   ```

3. **Register in function registry** (`src/sql/functions/mod.rs`):
   ```rust
   fn register_your_category(&mut self) {
       self.register(Box::new(YourFunction));
   }
   ```

4. **Test it**:
   ```bash
   ./target/release/sql-cli -q "SELECT YOUR_FUNC(1, 2)" -o csv
   ./target/release/sql-cli --function-help YOUR_FUNC
   ```

### Parser Modifications
- **File**: `src/sql/recursive_parser.rs`
- **Key methods**:
  - `parse_select_list()` - SELECT clause items
  - `parse_where_clause()` - WHERE conditions
  - `parse_expression()` - Expressions and operators
  - `parse_function_call()` - Function parsing
- **ALWAYS add Python tests** after parser changes

### Testing Strategy

1. **Quick command-line testing**:
   ```bash
   # Test new feature
   ./target/release/sql-cli -q "YOUR_QUERY" -o csv
   
   # Debug parser
   ./target/release/sql-cli -q "YOUR_QUERY" --query-plan
   ```

2. **Run test suites**:
   ```bash
   cargo test                    # Rust tests
   ./run_python_tests.sh        # Python integration tests
   ./scripts/test_all_examples.sh  # Example SQL files
   ```

3. **Always run before committing**:
   ```bash
   cargo fmt                    # Required formatting
   cargo clippy                 # Linting
   ./run_all_tests.sh          # All tests
   ```

## 🗂️ Project Structure

```
sql-cli/
├── src/
│   ├── main.rs                 # Entry point
│   ├── non_interactive.rs      # CLI query mode
│   ├── sql/
│   │   ├── recursive_parser.rs # SQL parser (builds AST)
│   │   ├── functions/          # Function implementations
│   │   │   ├── mod.rs         # Function registry
│   │   │   ├── math.rs        # Math functions
│   │   │   ├── string_methods.rs # String functions
│   │   │   └── convert.rs     # Unit conversions
│   │   └── script_parser.rs   # Script with GO separators
│   ├── data/
│   │   ├── datatable.rs       # Core data structure
│   │   ├── arithmetic_evaluator.rs # Expression evaluation
│   │   ├── query_executor.rs  # Query orchestration
│   │   └── unit_converter.rs  # Unit conversion logic
│   └── ui/                     # TUI components
├── examples/                   # SQL example files
├── data/                       # Test data files
├── tests/
│   └── python_tests/          # Python integration tests
└── scripts/
    └── test_all_examples.sh   # Example test runner
```

## 🎯 Key Principles

1. **Function Registry**: ALL functions go through the registry - no special cases in parser/evaluator
2. **Test Everything**: Add Python tests for SQL features, Rust tests for internals
3. **Use Non-Interactive Mode**: Test queries with `-q` flag before TUI testing
4. **Format Always**: Run `cargo fmt` before every commit
5. **Check Functions**: Use `--list-functions` when unsure about available functions

## 🔧 Common Tasks

### Finding Functions
```bash
# List all functions
./target/release/sql-cli --list-functions

# Search for specific function
./target/release/sql-cli --list-functions | grep -i convert

# Get function help
./target/release/sql-cli --function-help CONVERT
```

### Testing Queries
```bash
# Simple query
./target/release/sql-cli -q "SELECT 1+1" -o csv

# Query with data file
./target/release/sql-cli data/test.csv -q "SELECT * FROM test" -o csv

# Query with functions
./target/release/sql-cli -q "SELECT CONVERT(100, 'celsius', 'fahrenheit')" -o csv

# Show query plan (AST)
./target/release/sql-cli -q "SELECT * FROM test WHERE id > 5" --query-plan
```

### Running Tests
```bash
# All tests
./run_all_tests.sh

# Just Rust tests
cargo test

# Just Python tests
./run_python_tests.sh

# Test examples
./scripts/test_all_examples.sh

# Specific test
cargo test test_convert
python tests/python_tests/test_unit_conversions.py
```

## 📚 Test Data Files

- `data/test_simple_strings.csv` - String operations testing
- `data/test_simple_math.csv` - Math operations testing  
- `data/sales_data.csv` - Window functions, aggregates
- `data/solar_system.csv` - Astronomical calculations
- `data/trades.json` - JSON data source testing

## 🚨 Important Notes

- **ALWAYS run `cargo fmt` before committing** - Required for all commits
- **NULL handling**: Empty CSV fields are NULL, use IS NULL/IS NOT NULL
- **CONVERT function**: Use for all unit conversions, don't create individual functions
- **GO separator**: Supported in script files for batch execution
- **F5 in TUI**: Shows debug view with internal state

## 🔗 Quick Links

- Function Registry: `src/sql/functions/mod.rs`
- Parser: `src/sql/recursive_parser.rs`
- Expression Evaluator: `src/data/arithmetic_evaluator.rs`
- Query Executor: `src/data/query_executor.rs`
- Unit Converter: `src/data/unit_converter.rs`

## Agents (IMPORTANT: Always delegate to these specialized agents)

### rust-build-fixer
**ALWAYS delegate to this agent when:**
- Any `cargo build` or `cargo build --release` fails
- User reports compilation errors (e.g., "I'm getting an error...")
- After writing new Rust code (proactively check compilation)
- Formatting issues reported (cargo fmt failures)
- Type errors, borrow checker issues, or any Rust compilation problems

### rust-test-failure-investigator  
**ALWAYS delegate to this agent when:**
- `cargo test` reports ANY failures
- User mentions test failures (e.g., "tests are failing", "test broken")
- After implementing features that might affect tests
- CI pipeline test failures are reported
- Integration test failures occur

### debug-analyzer
**ALWAYS delegate to this agent when:**
- User provides F5 debug output
- Large debug dumps need analysis
- State inconsistency issues in TUI
- Performance bottlenecks need investigation

**CRITICAL**: Do NOT try to fix compilation errors or test failures yourself. ALWAYS delegate to the appropriate agent immediately.