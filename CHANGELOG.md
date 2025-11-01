# Changelog

All notable changes to SQL CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.64.0] - 2025-11-01

### ✨ Major Features

#### **Execution Mode Unification & Query Transformation Pipeline**
Complete unification of script mode (`-f`) and query mode (`-q`) execution paths, bringing sophisticated query transformation capabilities to both modes.

**Unified Execution Architecture**:
- **Single execution path** - Both `-f` scripts and `-q` queries now use the same underlying execution engine
- **Consistent transformer support** - All query transformations (WHERE, GROUP BY, HAVING, ORDER BY alias expansion) work in both modes
- **Dependency-aware execution** - `--execute-statement N` now applies full preprocessing pipeline
- **Shared infrastructure** - Eliminates code duplication and ensures feature parity

**Query Transformation Pipeline** (now available in both modes):
- **WHERE clause alias expansion** - Use SELECT aliases in WHERE: `SELECT value * 2 AS doubled FROM data WHERE doubled > 100`
- **GROUP BY alias expansion** - Reference SELECT aliases in GROUP BY: `SELECT region, SUM(sales) AS total FROM data GROUP BY region HAVING total > 1000`
- **HAVING auto-aliasing** - Automatic aliases for aggregate expressions in HAVING clause
- **ORDER BY expression support** - Complex expressions in ORDER BY automatically moved to SELECT with hidden columns

**New Debug Capabilities**:
- **`--show-transformations` flag** - See the complete transformation pipeline for any query
- **Nvim `\st` keymap** - Visualize transformations for query at cursor in Neovim plugin
- **`\sz` keymap** - Alternative transformations debug view
- **Detailed pipeline output** - Shows original SQL, intermediate steps, and final transformed query

**ORDER BY Expression Support**:
- **Complex expressions in ORDER BY** - Use any SQL expression in ORDER BY clause
- **Automatic SELECT injection** - Expressions automatically added to SELECT with hidden columns
- **Aggregate support** - ORDER BY can use aggregates: `ORDER BY SUM(value) DESC`
- **Works with transformers** - Integrates seamlessly with WHERE/GROUP BY alias expansion
- **Example**: `SELECT region FROM sales GROUP BY region ORDER BY SUM(amount) DESC`

**Technical Architecture**:
- **Three-phase execution** (Phases 0-2 complete):
  - Phase 0: Unified execution module foundation
  - Phase 1: Refactored `-q` mode to use unified path
  - Phase 2: Refactored `-f` mode to use unified path
  - Phase 3: Enabled full preprocessing pipeline in both modes
- **Transformer orchestration** - Coordinated pipeline of AST transformers
- **Preserved semantics** - All transformations maintain original query intent

#### **QUALIFY Clause Support**
Industry-standard window function filtering using QUALIFY clause (Snowflake, BigQuery, Teradata syntax).

**New Capability**:
```sql
-- Top 3 products per category by sales
SELECT category, product, sales,
       ROW_NUMBER() OVER (PARTITION BY category ORDER BY sales DESC) as rank
FROM products
QUALIFY rank <= 3;
```

**Benefits**:
- **Cleaner syntax** - No need for CTE wrapper around window functions
- **Better readability** - Filter intent clear and concise
- **Standard SQL** - Matches Snowflake/BigQuery syntax
- **Performance** - Efficient filtering after window function evaluation

**Full Window Function Support**:
- Works with ROW_NUMBER(), RANK(), DENSE_RANK()
- Supports LAG(), LEAD(), FIRST_VALUE(), LAST_VALUE()
- Compatible with all window functions (SUM, AVG, COUNT, etc.)
- Handles complex PARTITION BY and ORDER BY clauses

### 🔧 Improvements

**Examples Testing Framework**:
- **Python-based test runner** - Replaced bash scripts with robust Python framework
- **Formal testing** - JSON expectations for critical examples
- **Smoke testing** - 117+ examples validated for basic execution
- **Data file hint support** - Examples automatically find their data files
- **Clear output** - JSON validation failures clearly reported

**Neovim Plugin**:
- **Transformation debug keymaps** - `\st` and `\sz` for pipeline visualization
- **Better keymap organization** - Moved transformations to `\sz` to free up `\st`

**Documentation**:
- **UNION ALL examples** - Added comprehensive subquery examples
- **ORDER BY examples** - New example file for expression patterns
- **Roadmap updates** - Documented ORDER BY completion status

### 🐛 Bug Fixes

**Temp Table Persistence**:
- **Fixed `--execute-statement` mode** - Temp tables now properly persist across statement execution
- **Materialization** - Temp tables correctly materialized and registered
- **Dependency chain** - Multi-statement scripts with temp table dependencies work correctly

**Transformer Pipeline**:
- **Dependency-aware execution** - Transformers now enabled in `--execute-statement` mode
- **Qualified name resolution** - Fixed regression in table.column resolution after transformer changes
- **Correlated subquery detection** - Phase 1 analyzer for future optimization

**Testing**:
- **History file tests** - Ignored tests requiring persistent history file in CI
- **Test output** - Clear JSON validation messages in examples framework
- **CI pipeline** - Examples test suite integrated into continuous integration

### 📚 Documentation & Examples

**New Examples**:
- `examples/order_by_expressions.sql` - ORDER BY expression patterns
- `examples/union_all_subquery.sql` - UNION ALL with subqueries
- Time series generation examples - How to create temporal test data

**Updated Documentation**:
- `CLAUDE.md` - Examples test framework commands and usage
- Roadmap - ORDER BY expression support completion notes
- Test framework - Formal vs smoke test distinctions

### 🎯 Use Cases Enabled

**Complex Analytical Queries**:
```sql
-- Top regions by total sales (ORDER BY with aggregate)
SELECT region, SUM(amount) AS total
FROM sales
GROUP BY region
ORDER BY SUM(amount) DESC
LIMIT 5;

-- Filtered window functions with QUALIFY
SELECT salesperson, month, sales,
       ROW_NUMBER() OVER (PARTITION BY salesperson ORDER BY sales DESC) as rank
FROM monthly_sales
QUALIFY rank <= 3;

-- Multi-stage transformation pipeline (visible with --show-transformations)
SELECT region, value * 2 AS doubled
FROM data
WHERE doubled > 100
GROUP BY region
HAVING SUM(doubled) > 1000
ORDER BY SUM(doubled) DESC;
```

**Unified Workflow**:
- Write query in Neovim with `\st` to see transformations
- Test with `-q` mode: `sql-cli -q "SELECT ..." --show-transformations`
- Save to script file and run with `-f` mode (same execution path!)
- Use `--execute-statement N` with full transformer support

### 🔧 Technical Details

**Files Modified**:
- `src/main.rs` - Unified execution path integration
- `src/execution/mod.rs` - New unified execution module
- `src/query_plan/` - Transformer orchestration
- `tests/integration/test_examples.py` - Python examples framework
- `nvim-plugin/lua/sql-cli/` - Transformation debug keymaps

**Test Results**:
- ✅ 457 library tests passing
- ✅ 397 integration tests passing
- ✅ 119 example tests (2 formal, 117 smoke)

---

## [1.63.0] - 2025-10-25

### ✨ Major Features

#### **JOIN Expression Support (Phase 2) - LEFT Side Expressions**
Complete the JOIN expression support by enabling functions and expressions on the **LEFT side** of JOIN conditions! This mirrors Phase 1 functionality and enables expressions on **BOTH sides** simultaneously.

**Phase 2 Capabilities**:
- **LEFT-side TRIM()** - `JOIN table ON TRIM(table1.name) = table2.name`
- **LEFT-side UPPER()/LOWER()** - `JOIN table ON UPPER(table1.code) = table2.CODE`
- **LEFT-side SUBSTRING()** - `JOIN table ON SUBSTRING(table1.id, 0, 3) = table2.code`
- **LEFT-side nested functions** - `JOIN table ON UPPER(TRIM(table1.name)) = table2.name`
- **LEFT-side arithmetic** - `JOIN table ON table1.value / 10 = table2.bin_id`
- **String concatenation** - `JOIN table ON table1.prefix || '-' || table1.suffix = table2.code`

**BOTH Sides with Expressions** (Phase 1 + Phase 2 Combined!):
```sql
-- Normalize both sides for matching
SELECT *
FROM customers
JOIN accounts ON LOWER(TRIM(customers.email)) = LOWER(TRIM(accounts.email));
```

**Real-World Examples**:
```sql
-- Case-insensitive matching with left-side normalization
SELECT *
FROM products
JOIN inventory ON UPPER(products.code) = inventory.CODE;

-- Extract prefix from left side for matching
SELECT *
FROM orders
JOIN regions ON SUBSTRING(orders.order_id, 0, 3) = regions.code;

-- Multi-condition with left-side expression
SELECT *
FROM sales
JOIN pricing
    ON TRIM(sales.product) = pricing.product
    AND sales.region = pricing.region;
```

**Performance**:
- Smart algorithm selection: hash join when both sides are simple columns
- Nested loop with expression evaluation when either side has expressions
- Expression evaluation: ~2-3ms for typical datasets with expressions on both sides

**Files**:
- Example: `examples/join_left_expression_demo.sql` (8 working examples)
- Docs: `docs/JOIN_EXPRESSION_PHASE2_COMPLETE.md`
- Plan: `docs/JOIN_EXPRESSION_PHASE2_PLAN.md`

### 🔧 Technical Changes

**AST Structure**:
- Changed `SingleJoinCondition.left_column: String` → `left_expr: SqlExpression`
- Both sides now support full expression trees

**Parser**:
- Updated to parse left side as expression using `parse_additive()`
- Critical fix: use `parse_additive()` instead of `parse_expression()` to avoid consuming comparison operators

**Executor**:
- Updated algorithm selection to check both sides for complexity
- Both `nested_loop_join_inner_multi` and `nested_loop_join_left_multi` now evaluate left expressions
- Hash join only used when both sides are simple columns

**Test Results**:
- ✅ 457 library tests passing
- ✅ 397 integration tests passing
- ✅ All 8 demo examples working

### 🐛 Bug Fixes
- Fixed parser precedence issue where comparison operators were consumed too early

---

## [1.62.0] - 2025-10-22

### ✨ Major Features

#### **JOIN Expression Support (Phase 1)**
Enable functions and expressions on the **right side** of JOIN conditions - a game-changer for real-world data integration!

**Use Case**: Handle padded database exports, case-insensitive matching, and data normalization directly in JOIN conditions.

**New Capabilities**:
- **TRIM()** - Remove padding: `JOIN fund_data ON portfolio = TRIM(fund_data.Name)`
- **UPPER()/LOWER()** - Case-insensitive: `JOIN users ON id = UPPER(email)`
- **SUBSTRING()** - Partial matching: `JOIN codes ON id = SUBSTRING(code, 1, 10)`
- **CONCAT()** - Build keys: `JOIN data ON id = CONCAT(prefix, suffix)`
- **Nested functions** - Complex transforms: `JOIN data ON id = UPPER(TRIM(name))`
- **All SQL functions** - Works with any function in the registry

**Example** (your exact use case):
```sql
-- Load two CSV files and join with TRIM to handle padding
WITH
    WEB portfolios AS (URL 'file://data/portfolios.csv' FORMAT CSV),
    WEB fund_names AS (URL 'file://data/fund_names_padded.csv' FORMAT CSV)
SELECT
    portfolios.*,
    fund_names.fund_id,
    fund_names.manager
FROM portfolios
JOIN fund_names ON portfolios.portfolio = TRIM(fund_names.Name);
```

**Performance**:
- Smart algorithm selection: hash join for simple columns, nested loop for expressions
- Backward compatible: existing queries maintain performance
- Expression evaluation overhead: ~1-2ms for typical datasets

**Phase 1 Limitation**: Left side still requires simple column names. Use CTEs to pre-transform left side if needed.

**Files**:
- Example: `examples/join_two_files_with_trim.sql`
- Docs: `docs/JOIN_EXPRESSION_PHASE1_COMPLETE.md`
- Analysis: `docs/JOIN_EXPRESSION_SUPPORT_ANALYSIS.md`

#### **Scoped Star Expansion for JOINs**
SELECT specific table columns with `table.*` syntax in JOIN queries.

**New Capability**:
```sql
SELECT
    users.*,                    -- Expands to all users columns
    orders.order_id,            -- Just specific order columns
    orders.total
FROM users
JOIN orders ON users.id = orders.user_id;
```

**Benefits**:
- Avoid column name collisions in JOINs
- Clear, readable queries
- Select all from one table, specific from another
- Works with multiple JOINs

**Example**:
```sql
SELECT
    portfolios.*,               -- id, portfolio, value
    fund_names.fund_id,
    fund_names.manager
FROM portfolios
JOIN fund_names ON portfolios.portfolio = TRIM(fund_names.Name);
```

### 🔧 Technical Details

**Modified Components**:
- AST: `SingleJoinCondition.right_column` → `right_expr: SqlExpression`
- Parser: Parses right side as full SQL expression
- Executor: Expression evaluation in nested loop join algorithms
- Tests: 854 tests passing (457 lib + 397 integration)

**Backward Compatibility**: ✅ 100% - All existing JOIN queries continue to work unchanged

## [1.61.0] - 2025-10-18

### 🎯 Neovim Plugin UX Enhancements

This release focuses on improving the user experience of the Neovim plugin's table navigation and fuzzy filtering features, making it production-ready for analyzing large multi-table result sets.

### ✨ Improvements

#### **Smart Table Navigation**
- **Nearest table focus** - `\sTt` now jumps to the NEAREST table instead of always jumping to first table
  - Critical for workflows with 8+ result tables where cursor may be near table 5
  - Calculates distance above/below each table and selects minimum
- **Smooth navigation** - Eliminated cursor jumping during cell navigation in large tables (5k-10k rows, 40+ columns)
  - Replaced complex viewport management with simple `zz` centering
  - Predictable, smooth behavior when navigating cells with h/j/k/l
- **Fixed next/prev table jumps** - `\sTn`/`\sTp` no longer cause temporary cursor jumps to top of buffer
  - Same viewport fix applied to multi-table navigation

#### **Context-Aware Fuzzy Filtering**
- **Filters focused table** - Fuzzy filter (`/`) now filters the currently focused table, not always first table
  - Integrates with table navigation mode (`\sTt`)
  - Shows accurate row counts for the table you're viewing
  - Perfect for multi-table workflows

#### **Interactive Fuzzy Filter**
- **Lock mode** - Press Enter to lock filtered results while keeping them visible
  - Closes filter input window
  - Returns to normal mode (not insert mode)
  - Enables free navigation of filtered results
  - Shows: "Filter locked - 12/1000 rows visible (/ to reopen, ESC to clear)"
- **Navigate while filtering** - New keybindings for exploring results without closing filter
  - `Ctrl+j/k` - Scroll results up/down while typing filter pattern
  - `Ctrl+d/u` - Page down/up in results
  - See live updates as you refine your filter
- **Persistent ESC handler** - Press ESC in locked filter mode to restore full table
  - Buffer-local keymap that persists after filter window closes
  - Clean workflow: filter → lock → navigate → ESC to restore

### 🔧 Bug Fixes
- Fixed buffer entering insert mode after locking fuzzy filter
- Fixed ESC not clearing filter after lock mode enabled
- Fixed fuzzy filter always operating on first table in multi-table buffers

### 📝 Use Case
Perfect for FIX message log analysis with ~10k rows and 40 columns across multiple result tables:
- Navigate between 8+ tables with `\sTn`/`\sTp` (smooth, no jumps)
- Focus specific table with `\sTt` (finds nearest table)
- Filter 10k rows to 50 matches with `/` (filters current table)
- Lock filter with Enter and navigate freely
- ESC to restore full table, `/` to refine filter

## [1.60.0] - 2025-10-12

### 🚀 Dependency-Aware Script Execution & Multi-Stage Analysis

This release introduces powerful dependency-aware script execution with comprehensive support for multi-stage SQL pipelines, mimicking real-world hedge fund trading workflows.

### ✨ New Features

#### **Dependency-Aware `--execute-statement` Feature**
- **Smart statement execution** - `--execute-statement N` analyzes dependencies and executes only required statements
- **Temp table tracking** - Automatically detects `SELECT ... INTO #table` and registers temp tables for subsequent queries
- **Minimal execution** - Only runs statements needed to produce target result (skips unrelated statements)
- **DUAL table fallback** - WEB CTEs work without CSV files, using DUAL table when no data file provided
- **Case-insensitive GO** - Script parser now handles `GO`, `go`, and `Go` separators

#### **Neovim Plugin `\sx` Enhancement**
- **Case-insensitive GO support** - `\sx` (execute at cursor) now works with both uppercase and lowercase GO
- **Correct statement counting** - Fixed bug where statement numbers were off by one
- **No data file errors fixed** - WEB CTEs no longer require CSV data files

#### **Comprehensive Hedge Fund Analysis Example**
- **8-stage analysis pipeline** (`examples/hedge_fund_execution_analysis.sql`)
  1. Parse FIX Messages - Fetch execution reports via HTTP
  2. Compute Timing Metrics - Rolling VWAP, cumulative volume, LAG analysis
  3. Fetch Trade Database - Query internal trade records
  4. Enrich Securities Master - Add instrument details (sector, ISIN)
  5. Join Full Dataset - Combine all data sources
  6. Execution Quality by Sector - Latency and volume analysis
  7. Symbol-Level VWAP - Top 10 by volume with rolling averages
  8. Latency Distribution - Bucket analysis of execution speeds

#### **Enhanced Flask Test Server**
- **`/securities` endpoint** - Securities master data (ticker, sector, exchange, ISIN)
- **`/fix_messages` endpoint** - Simulated FIX execution reports with timing/latency data
- **`/parent_orders` endpoint** - Parent/child order hierarchy (ready for future examples)

### 🔧 Technical Improvements

#### **Temp Table Registration**
- `execute_statement_with_temp_tables()` now properly registers temp tables after execution
- Uses `materialize_view()` to convert DataView to DataTable
- Made `materialize_view()` public in QueryEngine (was private)
- Temp tables accessible to all subsequent statements in execution chain

#### **WEB CTE Enhancements**
- Fixed "Column not found" errors - WEB CTEs must SELECT FROM the CTE name
- Added empty BODY '{}' support for endpoints that don't need parameters
- Proper error handling for HTTP endpoints

#### **GROUP BY with CASE Expressions**
- Fixed "must appear in GROUP BY clause" errors
- GROUP BY now supports full CASE expressions (not just column aliases)
- Repeated CASE expression in GROUP BY clause for proper aggregation

### 📊 Performance

Multi-stage pipeline execution is extremely fast:
```
8 statements (5 dependencies analyzed, 3 skipped)
Total execution time: 105.96ms
✅ All temp tables properly created and chained
```

### 🐛 Bug Fixes

**Nvim Plugin (`nvim-plugin/lua/sql-cli/executor.lua`)**:
- Fixed statement counting logic - was counting GOs before cursor instead of finding which block contains cursor
- Added case-insensitive GO matching with `.upper()` method
- Fixed "executing statement #1" when cursor was actually on statement #2

**Main CLI (`src/main.rs`)**:
- Fixed "no data file provided" error for WEB CTE queries
- Use DUAL table when no data file specified (lines 803-814)
- Fixed temp tables not being registered (lines 833-856)
- Capture `into_table` name and register after successful execution

**Query Engine (`src/data/query_engine.rs`)**:
- Made `materialize_view()` public (line 1073) for temp table creation

### 📚 Examples & Testing

**Usage**:
```bash
# Run complete analysis pipeline
./target/release/sql-cli -f examples/hedge_fund_execution_analysis.sql

# Execute specific stage with dependencies
./target/release/sql-cli -f examples/hedge_fund_execution_analysis.sql --execute-statement 8

# In Neovim: \sq (run all) or \sx (run statement at cursor)
```

**Features Demonstrated**:
- WEB CTEs for HTTP data fetching
- Multi-stage temp table pipeline (`#fix_messages` → `#enriched_fix` → `#full_dataset`)
- Window functions (LAG, AVG OVER, SUM OVER, ROW_NUMBER)
- Complex joins and aggregations
- CASE expressions in SELECT and GROUP BY
- Dependency analysis and optimal execution order

### 🎉 User Feedback

Real user validation:
> "mind blowing! so i have a query file where end query has 5 intermediary steps, 5 dependencies... and the final query works!"

> "this is amazing"

The feature has been tested with real production queries involving complex multi-stage pipelines with temp table dependencies.

## [1.59.0] - 2025-10-11

### 🚀 Critical Performance Fix & Infrastructure Improvements

This release fixes a severe performance regression introduced by the alias resolution feature and adds important SQL formatting and preprocessing improvements.

### ⚡ Performance Improvements

#### **WHERE Clause Evaluation Optimization - 23x Faster!**
- **Fixed critical regression** - `RecursiveWhereEvaluator` was being created inside the row loop instead of once before the loop
- **Impact on complex queries**:
  - 3-condition WHERE on 5,000 rows: **1277ms → 54ms** (23x speedup)
  - 2-condition WHERE on 5,000 rows: **46ms → unchanged**
  - Simple WHERE on 5,000 rows: **17ms → unchanged**
- **Root cause**: Alias resolution feature (v1.58.0) moved evaluator creation inside the loop, causing 5000+ unnecessary object instantiations
- **Solution**:
  - Added `with_both_contexts()` method to support both alias resolution AND regex caching
  - Moved evaluator creation outside the loop
  - Reuses single evaluator instance for all rows

### 🏗️ SQL Formatting & Preprocessing

#### **SELECT INTO Syntax Support**
- **Proper SQL Server syntax** - `SELECT col1, col2 INTO #temp FROM table WHERE ...`
- **AST-based formatting** - INTO clause now properly formatted in query output
- **Parser fix** - INTO recognized in correct position (after SELECT, before FROM)
- **Example formatting** works correctly: `examples/tmp_table.sql`

#### **AST-Based INTO Clause Removal**
- **New preprocessing module** - `IntoClauseRemover` follows CTEHoister pattern
- **Replaced regex hack** - Brittle regex removal replaced with proper AST manipulation
- **Recursive handling** - Removes INTO from all nested subqueries
- **Maintainable architecture** - Clean separation of parsing, preprocessing, and execution

#### **Comment-Aware Tokenization Foundation**
- **Dual-path lexer** - New `next_token_with_comments()` preserves comments, old `next_token()` skips them
- **Token types added** - `Token::LineComment` and `Token::BlockComment`
- **Backwards compatible** - Parser unchanged, uses comment-skipping path
- **Future-ready** - Foundation for Prettier/Rustfmt-style comment preservation in formatters

### 🔧 Technical Details

**Performance Fix Files**:
- `src/data/recursive_where_evaluator.rs` - Added `with_both_contexts()` method (lines 55-68)
- `src/data/query_engine.rs` - Moved evaluator creation outside loop (lines 1169-1174)

**Formatting Files**:
- `src/sql/parser/ast_formatter.rs` - Added INTO clause formatting
- `src/sql/recursive_parser.rs` - Fixed INTO parsing position
- `src/query_plan/into_clause_remover.rs` - New AST-based preprocessor
- `src/sql/parser/lexer.rs` - Added comment token support

### 📊 Benchmark Results

Testing complex WHERE clause (3 conditions) on 5,000 rows:
```
Before fix:  1277.48ms  ❌
After fix:   54.65ms    ✅  (23x faster)
```

### 🐛 Bug Fixes
- Fixed SELECT INTO being dropped during formatting
- Fixed execution of SELECT INTO statements in scripts
- Fixed text_navigation.rs compilation after comment tokens added

### 📚 Documentation
- Updated `examples/tmp_table.sql` with correct SQL Server syntax
- Added `src/query_plan/into_clause_remover.rs` with comprehensive doc comments

## [1.58.0] - 2025-10-09

### 🎯 Qualified Column Names and Table Alias Support

This release brings comprehensive table alias support across all SQL clauses, enabling more readable and maintainable queries with qualified column references.

### ✨ New Features

#### **Qualified Column Names (table.column syntax)**
- **Table aliases now work throughout queries** - Use `t.column_name` syntax in WHERE, SELECT, ORDER BY, and GROUP BY clauses
  - Example: `SELECT t.id, t.amount FROM data t WHERE t.amount > 100 ORDER BY t.amount DESC`
  - Works with CTEs, nested queries, and complex multi-level aliases
  - Full support for method calls on qualified names: `WHERE t.classification.Contains('value')`

#### **Implementation Phases (All Complete)**
- **Phase 1-2**: ExecutionContext infrastructure and unified column resolution
- **Phase 3**: WHERE clause alias resolution - Qualified names in filtering conditions
- **Phase 4**: SELECT clause alias resolution - Qualified names in projections
- **Phase 5**: ORDER BY clause alias resolution - Qualified names in sorting
- **Phase 6**: GROUP BY clause - Already working with expression-based parsing

### 🏗️ Architecture Highlights

#### **Non-Breaking Design**
- All changes maintain backward compatibility
- Fallback behavior when ExecutionContext unavailable
- Unqualified column names continue to work as before

#### **ExecutionContext System**
- Tracks table alias mappings during query execution
- Resolves qualified names (table.column) to actual column indices
- Provides "did you mean?" suggestions using edit distance algorithm

#### **ArithmeticEvaluator Enhancement**
- Resolves table aliases in column references
- Tries qualified lookup first, falls back to unqualified
- Supports both qualified and unqualified names in same query

### 📚 Documentation & Examples

#### **New Example File: `examples/qualified_names.sql`**
- 7 comprehensive examples demonstrating alias support
- Basic CTE with aliases in WHERE/SELECT
- ORDER BY with qualified column names
- GROUP BY with qualified columns and aggregations
- Nested CTEs with multi-level aliases
- String method calls with qualified names
- Complex queries combining all clauses

### 🔧 Technical Implementation

**Key Files Modified:**
- `src/data/query_engine.rs` - ExecutionContext, resolve_column_index, apply_select_items integration
- `src/data/recursive_where_evaluator.rs` - WHERE clause alias resolution
- `src/data/arithmetic_evaluator.rs` - evaluate_column_ref with table_aliases
- `src/sql/recursive_parser.rs` - ORDER BY parser for qualified names

**Commit History:**
- `125679e` - Phase 1: ExecutionContext infrastructure
- `7771912` - Phase 2: Unified resolve_column helper
- `9a42c0b` - Phase 3: WHERE clause resolution
- `abd5b8d` - Phase 4: SELECT clause resolution
- `8000cad` - Phase 5: ORDER BY resolution
- `722ee33` - Documentation and examples

### 📝 Known Limitations
- HAVING clause currently requires unqualified column references
- Temp tables (#table) with aliases have limited support (investigation ongoing)

### 🎉 Use Cases Enabled
```sql
-- Nested CTEs with aliases throughout
WITH base AS (
    SELECT value as id, value % 5 as bucket
    FROM RANGE(1, 25)
),
enriched AS (
    SELECT b.id, b.bucket, b.id * 10 as score
    FROM base b
    WHERE b.bucket IN (0, 1, 2)
)
SELECT e.bucket, COUNT(*), SUM(e.score)
FROM enriched e
GROUP BY e.bucket
ORDER BY e.bucket;
```

## [1.57.0] - 2025-10-06

### 🗓️ Flexible Date Parsing & Nvim Plugin UX Improvements

This release introduces powerful date parsing functions with custom format support and improves the Nvim plugin user experience by removing unnecessary prompts.

### ✨ New Features

#### **Flexible Date Parsing Functions**
- **PARSE_DATETIME(date_string, format_string)** - Parse dates with custom chrono format strings
  - Supports European (DD/MM/YYYY), American (MM/DD/YYYY), ISO 8601, FIX Protocol formats
  - Text month names (Jan, January)
  - Millisecond precision support
  - Full chrono strftime format compatibility
  - Example: `SELECT PARSE_DATETIME('15/01/2024', '%d/%m/%Y')`

- **PARSE_DATETIME_UTC(date_string, [format_string])** - Parse datetime explicitly as UTC
  - 1 argument: auto-detects format (includes FIX Protocol)
  - 2 arguments: uses custom format string
  - Example: `SELECT PARSE_DATETIME_UTC('20240115-14:30:45.567')` (auto-detects FIX)

- **DATETIME(year, month, day, [hour], [minute], [second])** - Construct datetime from components
  - 3-6 arguments supported
  - All times interpreted as UTC
  - Handles leap years and month boundaries
  - Example: `SELECT DATETIME(2024, 1, 15, 14, 30, 45)`

#### **FIX Protocol Support**
- Verified compatibility with FIX timestamp format (YYYYMMDD-HH:MM:SS.sss)
- Millisecond precision maintained throughout parsing pipeline
- Works seamlessly with existing date functions (DATEDIFF, DATEADD, etc.)

### 🛠️ Nvim Plugin UX Improvements

#### **Instant Query Execution**
- `\sx` (execute at cursor) now executes immediately without parameter prompts
- Removed debug notification that caused "Press ENTER" prompts
- Parameter resolution skipped for cursor-based execution for faster workflow
- Debug mode (`vim.g.sql_cli_debug = true`) still available when needed

### 📚 Documentation & Testing
- **New Examples**: `examples/parse_datetime.sql` - Comprehensive date parsing guide with format reference
- **Test Suite**: 18 new Python tests for date parsing functions (all passing)
- **Format Reference**: Inline chrono strftime format documentation
- Verified with real FIX timestamp data (`data/fix_timestamps.csv`)

### 🔧 Technical Details
- Built on existing chrono dependency (no new dependencies)
- Parser currently supports up to 6 function arguments (7th argument support planned)
- All datetime values stored with millisecond precision
- Consistent UTC interpretation across all new functions

## [1.56.0] - 2025-10-02

### 🎯 Smart Column Intelligence & Cardinality Analysis

This release brings intelligent column expansion and powerful data cardinality analysis to the Nvim plugin, making data exploration significantly faster.

### ✨ New Features

#### **Smart Star Expansion (`\sE`)**
- Execute queries with `LIMIT 1` to discover actual column names from CTEs and subqueries
- No longer limited to static CSV file hints - works with any query context
- Supports both array-of-objects and object-with-columns JSON formats
- Auto-inserts column hint comments for Nvim's built-in Ctrl+N completion
- Configurable via `smart_expansion.enabled` and `auto_insert_column_hints` settings
- Graceful fallback to static file hints when query execution isn't possible

#### **Distinct Values Analysis (`\srD`)**
- New `--distinct-column <column>` CLI flag for instant cardinality analysis
- Automatically detects and preserves WEB CTE context (HTTP data sources)
- Intelligent CTE extraction using parenthesis depth tracking
- Displays top 100 distinct values with counts in floating window
- Works seamlessly with CTEs, subqueries, files, and HTTP endpoints
- Proper handling of nested CTEs and complex query structures

### 🛠️ Technical Improvements
- CTE-aware query rewriting in Rust with proper parenthesis tracking
- Simplified Nvim plugin to call CLI and parse CSV output
- Proper async handling with `vim.schedule()` for event context
- Clear separation: Rust handles heavy lifting, Lua orchestrates display
- Enhanced column state tracking across buffers

### 📚 Documentation
- Added comprehensive smart expansion guide: `SMART_EXPANSION_README.md`
- Design document for column completion: `NVIM_SMART_COLUMN_COMPLETION.md`
- Example queries showcasing smart expansion features

## [1.55.0] - 2025-09-27

### 🎉 Windows Nvim Export Support & Performance Documentation

This release brings full Windows compatibility for the Nvim plugin's export features and showcases impressive performance benchmarks.

### ✨ New Features

#### **Windows Export Compatibility**
- Fixed TSV/CSV export clipboard handling on Windows (proper CRLF line endings)
- Browser HTML export now works on Windows (using rundll32 url.dll)
- Direct sql-cli calls for clean exports without table formatting artifacts
- Proper temp file handling for Windows (%TEMP% directory)
- Clear notifications showing export source and success status

#### **Performance Documentation**
- Added comprehensive PERFORMANCE.md with detailed benchmarks
- 100K row benchmarks added to test suite
- Results show:
  - Simple SELECT: 8ms at 100K rows
  - JOINs: Under 40ms for all types
  - GROUP BY: 433ms-2.49s (improved from 12s!)
  - Window functions: ~1.2s at 100K rows

### 🐛 Bug Fixes
- Fixed nil 'lines' error in table_nav.lua when using \se export
- Fixed export buffer detection (now checks output buffer correctly)
- HTML export now fetches data directly from sql-cli with proper CSV parsing
- Browser export on Windows no longer opens terminal window

### 🛠️ Technical Improvements
- Refactored export.lua to call sql-cli directly for clean data
- Added proper CSV parsing for quoted fields with commas
- Improved error handling and debug messages for exports
- Export functions now work immediately after \sx query execution

## [1.54.0] - 2025-09-24

### 🚀 Major Performance Improvements & JOIN Enhancements

This release delivers massive GROUP BY performance improvements and adds support for multiple JOIN conditions.

### ✨ New Features

#### **Multiple JOIN Conditions**
- Support for multiple conditions in JOIN clauses connected by AND
- Example: `INNER JOIN table ON a.id = b.id AND a.status = b.status`
- Works with all join types (INNER, LEFT, RIGHT)
- Supports mixed operators (equality and inequality)

#### **Enhanced Execution Plan**
- Added detailed phase breakdown for GROUP BY operations
- Shows timing for each phase: group building, aggregation, HAVING filter
- Use `--execution-plan` flag to see detailed performance metrics

### 🎯 Performance Improvements

#### **GROUP BY Optimization - 6x Faster!**
- **Fixed major inefficiency**: ArithmeticEvaluator was being created for every row
- **Results**:
  - 30,000 rows: 2,421ms → 402ms (6x faster)
  - 50,000 rows: 3,808ms → 633ms (6x faster)
  - Group building phase alone is 12x faster
- **Impact**: All GROUP BY queries will see significant performance gains

### 🛠️ Technical Details
- Reused ArithmeticEvaluator instances instead of creating 30,000+ times
- Pre-allocated and reused vectors in hot paths
- Added GroupByPhaseInfo for detailed performance tracking

## [1.53.0] - 2025-09-23

### 🎯 String Utilities & Type Conversions

This release adds essential utility functions for character code operations, type conversions, and encoding/decoding capabilities.

### ✨ New Features

#### **Character Code Functions**
- **`ASCII(string)`** - Get ASCII/Unicode code point of first character (supports full Unicode)
- **`ORD(string)`** - Alias for ASCII function
- **`CHAR(code)`** - Convert Unicode code point to character (supports codes beyond ASCII 0-255)
- **`UNICODE(string)`** - Get all Unicode code points as comma-separated list

#### **Type Conversion Functions**
- **`TO_INT(value)`** - Convert string or float to integer (truncates decimals)
- **`TO_DECIMAL(value)`** - Convert string or integer to decimal/float
- **`TO_STRING(value)`** - Convert any value to string representation

#### **Encoding/Decoding Functions**
- **`ENCODE(string, format)`** - Encode string to base64 or hex format
- **`DECODE(string, format)`** - Decode base64 or hex encoded strings

### 📊 Examples
```sql
-- Character codes
SELECT ASCII('€');  -- Returns 8364
SELECT CHAR(8364);  -- Returns '€'
SELECT UNICODE('ABC');  -- Returns '65,66,67'

-- Type conversions
SELECT TO_INT('123.45');  -- Returns 123
SELECT TO_DECIMAL('123');  -- Returns 123.0

-- Encoding
SELECT ENCODE('Hello', 'base64');  -- Returns 'SGVsbG8='
SELECT DECODE('SGVsbG8=', 'base64');  -- Returns 'Hello'
```

## [1.52.0] - 2025-09-21

### 🔢 Arbitrary Precision Arithmetic & Bit Manipulation

This release adds comprehensive support for arbitrary precision integer arithmetic, bit manipulation operations, and base conversions using Rust's `num-bigint` library.

### ✨ New Features

#### **BigInt Arithmetic Functions**
- **`BIGINT(value)`** - Convert numbers or strings to arbitrary precision integers
- **`BIGADD(a, b)`** - Add two arbitrary precision integers
- **`BIGMUL(a, b)`** - Multiply large numbers (tested with 30-digit numbers)
- **`BIGPOW(base, exp)`** - Calculate powers like 2^256 or 99^99
- **`BIGFACT(n)`** - Calculate factorials up to 10000! (1000! has 2568 digits)

#### **Bit Manipulation Operations**
- **`BITAND(a, b)`** - Bitwise AND on arbitrary precision integers
- **`BITOR(a, b)`** - Bitwise OR operations
- **`BITXOR(a, b)`** - Bitwise XOR for large numbers
- **`BITSHIFT(n, shift)`** - Bit shifting left (positive) or right (negative)

#### **Base Conversion Functions**
- **`TO_BINARY(n)`** - Convert numbers to binary strings
- **`FROM_BINARY(s)`** - Parse binary strings to decimal
- **`TO_HEX(n)`** - Convert to hexadecimal representation
- **`FROM_HEX(s)`** - Parse hex strings (handles 0x prefix)

### 📊 Example Calculations
```sql
-- Calculate 2^100
SELECT BIGPOW('2', 100);  -- 1267650600228229401496703205376

-- 100 factorial
SELECT BIGFACT(100);  -- 93326215443944152681699238856266700490715968264381621468592963895217599993229915608941463976156518286253697920827223758251185210916864000000000000000000000000

-- Convert 2^256 to binary
SELECT TO_BINARY(BITSHIFT('1', 256));  -- 1 followed by 256 zeros

-- Bit operations
SELECT BITXOR(FROM_BINARY('1010'), FROM_BINARY('1100'));  -- 6
```

### 🔧 Dependencies
- Added `num-bigint = "0.4"` for arbitrary precision arithmetic
- Added `num-traits = "0.2"` for numeric trait implementations

## [1.51.0] - 2025-09-21

### 🎯 Qualified Column Resolution & Scoping

This release fixes critical issues with qualified column name resolution throughout the query pipeline, ensuring proper column scoping in JOINs, CTEs, and generator functions.

### ✨ Major Fixes

#### **Qualified Column Name Resolution**
- **Fixed parser distinction** between method calls (`column.Method()`) and qualified columns (`table.column`)
- **Strict validation** of table prefixes - invalid prefixes like `wrongtable.column` now properly fail
- **Preserved qualified names through JOINs** - All columns maintain their source table information
- **Standard CTE column enrichment** - CTEs from generators (RANGE, TRIANGULAR, SQUARES) now support qualified references

#### **Column Scoping Improvements**
- **JOIN operations** - Columns from both tables preserve their qualified names (`messages.field_name`, `fields.number`)
- **WEB CTEs** - Proper column enrichment with table prefix (`messages.message_name`)
- **Standard CTEs** - Generator functions now support qualified references (`tri.value`, `data1.num`)
- **Materialized views** - Qualified names preserved through view materialization

### 🔧 Technical Improvements

#### **Parser Enhancements**
- Method call detection now checks for parentheses after dot notation
- `resolve_select_columns` validates qualified names instead of ignoring prefixes
- Proper SQL syntax enforcement (single quotes for strings, double quotes for identifiers)

#### **Debug Capabilities**
- Added column scoping debug output for JOIN operations
- Enhanced logging shows which columns have qualified names
- Better error messages for column resolution failures

### 🐛 Bug Fixes
- Fixed `Type.Contains("value")` being incorrectly parsed as qualified column
- Corrected SQL examples using wrong quote types for string literals
- Fixed qualified name resolution in simple SELECT queries

## [1.50.0] - 2025-09-21

### 🚀 CTE Testing & SQL Enhancement Release

This release introduces powerful CTE (Common Table Expression) testing capabilities in the Neovim plugin, significant SQL formatter improvements, and enhanced parser features for better quoted identifier handling.

### ✨ New Features

#### **CTE Testing Framework (Neovim Plugin)**
- **Interactive CTE testing** - Test CTEs incrementally with `<leader>sC` keybinding
- **Query preview modal** - See exact SQL before execution with options to Execute/Yank/Cancel
- **Smart cursor detection** - Automatically detects which CTE the cursor is in
- **CLI-based parser** - Uses `--cte-info` flag for robust CTE structure analysis
- **CTE analysis popup** - View CTE dependencies and structure with `<leader>sA`
- **RANGE() query support** - Properly handles CTEs that use RANGE() without external data

#### **SQL Formatter Enhancements**
- **Quoted identifier preservation** - Maintains double quotes and brackets throughout AST
- **ColumnRef with QuoteStyle** - New AST structure preserves quote information
- **Improved reformatting** - WHERE clauses now preserve quoted column names
- **Better GO handling** - Case-insensitive terminator detection (GO/go/Go)

#### **Query Rewriter Framework**
- **Expression hoisting suggestions** - Analyzes queries for unsupported expressions
- **CTE transformation patterns** - Suggests moving complex expressions to CTEs
- **`--analyze-rewrite` flag** - New CLI flag for query rewrite analysis

#### **Parser Improvements**
- **CTE name handling** - Fixed parsing of CTE names with underscores (dates_1, inventory_2)
- **Comment handling** - Better handling of queries with leading comments
- **Query boundary detection** - Improved detection between CTEs and main SELECT

### 🔧 Technical Improvements

#### **AST Restructuring**
- **QuoteStyle enum** - Tracks None, DoubleQuotes, or Brackets
- **ColumnRef struct** - Replaces String for column references
- **Formatter updates** - All formatters updated to preserve quote styles
- **Parser updates** - Parser now captures and preserves quote information

#### **CTE Parser Enhancements**
- **JSON null handling** - Fixed Lua errors with null columns from CLI parser
- **Parenthesis tracking** - Accurate CTE boundary detection
- **WITH clause variations** - Handles WITH on same line as CTE or separate line
- **Main SELECT detection** - Stops at main SELECT to avoid including entire query

### 🐛 Bug Fixes

- **Fixed CTE test execution** - Corrected argument count in executor.execute_query call
- **Fixed DUAL table fallback** - RANGE queries no longer show "DUMMY X"
- **Fixed lowercase GO** - Terminator detection now case-insensitive
- **Fixed CTE name patterns** - Underscores in CTE names now properly recognized
- **Fixed query extraction** - Comments before WITH no longer included in test query
- **Fixed formatter jumping** - SELECT no longer jumps to previous GO line

### 📚 Documentation

- **CLAUDE.md updates** - Added agent delegation guidelines
- **Function roadmap** - Updated with completed and pending functions
- **Test examples** - Added complex CTE testing examples

## [1.49.0] - 2025-09-20

### 🎯 Aggregate Registry Migration & Developer Experience

This release continues the migration of aggregate functions to the new registry system, adds comprehensive benchmarking tools, and significantly improves the Neovim plugin experience with unified help system.

### ✨ New Features

#### **Unified Help System**
- **`--item-help` command** - Single CLI switch that checks functions, aggregates, and generators
- **Neovim K mapping** - Press K on any SQL function/aggregate/generator for instant help
- **Automatic type detection** - CLI automatically determines if item is a function, aggregate, or generator

#### **String Case Conversion Functions**
- **TO_SNAKE_CASE()** - Convert text to snake_case
- **TO_CAMEL_CASE()** - Convert text to camelCase
- **TO_PASCAL_CASE()** - Convert text to PascalCase
- **TO_KEBAB_CASE()** - Convert text to kebab-case
- **TO_CONSTANT_CASE()** - Convert text to CONSTANT_CASE
- **Intelligent word splitting** - Handles transitions between uppercase/lowercase/numbers correctly

#### **Performance Benchmarking Suite**
- **Python benchmark script** (`scripts/benchmark_all.py`) - Comprehensive performance testing
- **Scaling analysis** - Automatically determines O(1), O(n), O(n log n), O(n²) complexity
- **Performance comparison** - Compare results against baseline for regression detection
- **LIKE operator optimization** - Documented 7-14ms performance for 25K rows
- **GROUP BY optimization** - Confirmed O(n) scaling, not O(n²) as initially feared

### 🔧 Technical Improvements

#### **Aggregate Function Migration**
- **COUNT/COUNT_STAR migrated** - Moved from hardcoded to new registry system
- **Unified registry checking** - ArithmeticEvaluator now checks new registry for all aggregates
- **Sample vs Population variance** - VARIANCE now correctly returns sample variance (n-1 denominator)
- **Test suite updates** - Python tests updated to expect sample variance/stddev

### 🐛 Bug Fixes

- **Fixed case conversion tests** - Corrected expectations for word-splitting behavior
- **Fixed STDDEV/VARIANCE tests** - Updated to use sample variance calculations
- **Fixed Neovim K mapping** - Now properly bound and working for all item types
- **Fixed generator help** - No longer shows function error before checking generators

### 📚 Documentation

- **Performance metrics in README** - Added benchmarks showing exceptional LIKE performance
- **Migration documentation** - Updated docs for aggregate function migration process

## [1.48.0] - 2025-09-18

### 🚀 Major Performance Improvements & Data Generation

This release delivers massive performance improvements to the LIKE operator and introduces powerful data generation and benchmarking capabilities for performance testing and development.

### ✨ New Features

#### **Data Generation System**
- **Virtual table generator** - Create tables with configurable rows/columns on the fly
- **GENERATE() function** - SQL function to create test data: `SELECT * FROM GENERATE(1000, 5)`
- **Multiple table types**:
  - Narrow tables (3 columns)
  - Wide tables (20 columns)
  - Very wide tables (50 columns)
  - Mixed data tables (various data types)
  - Aggregation-optimized tables
  - Window function test tables
- **Neovim plugin integration** - Generator discovery and help system

#### **Comprehensive Benchmarking System**
- **45+ benchmark queries** across 5 categories (basic, aggregation, sorting, window, complex)
- **Progressive benchmarking** - Test performance from 10K to 100K+ rows
- **Detailed metrics** - Parse time, execution time, rows/sec throughput
- **Multiple output formats** - CSV export and markdown reports
- **Category-specific testing** - Focus on specific query types

### 🔧 Performance Optimizations

#### **LIKE Operator Optimization - 900x+ Faster**
- **Before**: O(n²) performance, 5.7 seconds for 20K rows
- **After**: O(n) linear performance, 7.5ms for 20K rows
- **How**: Introduced `EvaluationContext` with regex caching
- **Impact**: Interactive queries now possible on 100K+ row datasets
- Regex patterns compiled once and cached across all row evaluations
- Dramatic reduction in memory allocations

### 🐛 Bug Fixes
- Fixed test compilation issues with mutable evaluator references
- Resolved throughput calculation showing 0 rows/sec in metrics
- Fixed HAVING clause column resolution in benchmarks

### 📊 Performance Baseline
With this release, we've established performance baselines:
- Simple SELECT: < 20ms for 100K rows ✅
- LIKE patterns: < 20ms for 50K rows ✅ (was 14+ seconds)
- ORDER BY: < 30ms for 50K rows ✅
- GROUP BY: ~3 seconds for 50K rows (next optimization target)

## [1.47.0] - 2025-09-14

### 🎨 ASCII Chart Visualizations & Date Functions

This release brings powerful ASCII chart visualizations to the Neovim plugin and essential date extraction functions, making data analysis more visual and date handling much simpler.

### ✨ New Features

#### **ASCII Chart Visualizations for Neovim**
- **Bar charts** - Horizontal bar charts with customizable width and character styles
- **Pie charts** - ASCII pie charts with configurable radius (5-30, default 15)
- **Histograms** - Frequency distribution visualizations with binning support
- **Scatter plots** - 2D point plotting with density indicators
- **Sparklines** - Compact trend visualizations
- **Box plots** - Statistical summaries with quartiles and outliers

#### **Chart Integration & Commands**
- **Query-at-cursor visualization** - Instant charts with keybindings:
  - `<leader>sB` - Bar chart from query at cursor
  - `<leader>sP` - Pie chart from query at cursor
  - `<leader>sH` - Histogram from query at cursor
  - `<leader>sS` - Scatter plot from query at cursor
  - `<leader>sl` - Sparkline from query at cursor
- **Debug mode** - `:SqlChartDebug on/off` shows detailed parsing info
- **Configurable sizing** - `:SqlPieRadius <size>` to adjust pie chart size
- **Smart CSV parsing** - Handles quoted values and preserves text labels

#### **Essential Date Functions**
- **YEAR(date)** - Extract year as number (e.g., 2024)
- **MONTH(date)** - Extract month as number (1-12)
- **DAY(date)** - Extract day of month (1-31)
- These complement existing DAYOFWEEK(), DAYNAME(), MONTHNAME() functions

### 🔧 Improvements
- **Simplified date queries** - Replaced verbose CASE statements with clean function calls
- **Dollar sign handling** - Fixed parameter substitution issue (use `\$` to escape)
- **Enhanced debug output** - Debug info appears directly below charts for easy analysis
- **Better error messages** - Clear feedback when chart data requirements aren't met

### 🐛 Bug Fixes
- Fixed dollar signs in string literals being interpreted as parameters
- Resolved buffer modifiable errors when creating chart buffers
- Fixed CSV parsing to correctly preserve text labels vs numeric values
- Corrected DataValue type usage in new date functions

### 📚 Documentation
- Added comprehensive bar chart examples with escaped dollar signs
- Documented available date functions in examples
- Added debug workflow documentation for chart troubleshooting

## [1.46.0] - 2025-09-14

### 🚀 Multi-Table Navigation & Web Data Integration

This release introduces powerful multi-table navigation in the Neovim plugin and comprehensive web data fetching capabilities, making it easier than ever to work with complex SQL scripts and remote data sources.

### ✨ New Features

#### **Enhanced Neovim Plugin - Multi-Table Navigation**
- **Multi-table result navigation** - Navigate between multiple query results from scripts with GO separators
- **Intuitive keybindings**:
  - `]t` and `[t` - Navigate to next/previous table
  - `<leader>s1`, `<leader>s2`, `<leader>s3` - Jump directly to specific tables
  - `<leader>sI` - Show current table info with position and row count
- **Smart table detection** - Automatically detects ASCII, box, and pipe table formats
- **Viewport centering** - Tables are automatically centered in view with context
- **Status line integration** - Shows current table position (e.g., "📊2/16")
- **Comprehensive debug tools** - `<leader>sI` provides detailed table registry with clipboard export

#### **Web Data Integration & Environment Variables**
- **WEB CTE with custom headers** - Fetch data from REST APIs with authentication
- **Environment variable injection** - Use `${VAR_NAME}` syntax in queries for dynamic values
- **Flexible header configuration** - Set custom HTTP headers for API authentication
- **Seamless data integration** - Web data treated as first-class tables in SQL queries

#### **JOIN & Query Enhancements**
- **Qualified column names** - Support for `table.column` syntax in SELECT and WHERE clauses
- **Multiple WEB CTE support** - Fetch from multiple endpoints in single query
- **Enhanced column resolution** - Intelligent handling of ambiguous column references
- **Improved parser robustness** - Better handling of complex JOIN conditions

#### **String Method Extensions**
- **TrimStart()** and **TrimEnd()** methods - Remove leading/trailing whitespace
- **Enhanced method chaining** - Support for `column.Method1().Method2()` patterns
- **Consistent string operations** - Unified string manipulation across all data types

### 🔧 Internal Improvements
- **Centralized navigation logic** - Eliminated code duplication in table navigation
- **Registry-based table detection** - Pre-processed table lookup for improved performance
- **State conflict resolution** - Fixed navigation jumping between single/multi-table modes
- **Enhanced AST formatting** - Better representation of complex query structures

### 🐛 Bug Fixes
- Fixed state conflicts between single-table and multi-table navigation modes
- Resolved cursor positioning issues when jumping between tables
- Fixed viewport scrolling to keep navigated tables visible
- Corrected column name resolution in complex JOIN scenarios

## [1.45.0] - 2025-09-13

### 🚀 Enhanced CTE Support & Detailed Execution Plans

This release brings major improvements to CTE (Common Table Expression) handling and introduces comprehensive execution plan analysis for better query performance insights.

### ✨ New Features

#### **Advanced CTE Context Propagation**
- **Nested CTE support** - CTEs can now reference other CTEs within the same WITH clause
- **Subquery CTE access** - Subqueries can access CTEs defined in parent query scope
- **Complex analytical queries** - Enables sophisticated multi-level data transformations
- **Proper scope resolution** - CTE context correctly propagated through entire query tree

#### **Comprehensive Execution Plan Analysis**
- **Detailed step breakdown** with hierarchical tree visualization
- **CTE execution statistics**:
  - Processing time for each CTE
  - Result set size (rows and columns)
  - Source table and filter information
- **JOIN execution details**:
  - Join type and condition details
  - Left/right table row counts
  - Result set size and timing
- **Subquery tracking**:
  - Identifies and tracks subquery evaluation
  - Shows materialization of subquery results
- **Operation-level metrics**:
  - WHERE clause filtering (input → output rows)
  - GROUP BY aggregation statistics
  - ORDER BY sort timing
  - DISTINCT deduplication metrics
  - LIMIT/OFFSET row reduction
- **Visual execution tree** - Beautiful ASCII art visualization of query execution flow

#### **AST Formatter Enhancements**
- **Method call support** - Properly formats `Column.Method()` syntax (e.g., `Type.Contains('Noble')`)
- **Chained method calls** - Handles `column.Method1().Method2()` patterns
- **Preserves original syntax** - No more debug output for method expressions

### 🐛 Bug Fixes
- Fixed AST formatter outputting debug representations for MethodCall and ChainedMethodCall expressions
- Resolved CTE reference errors in complex nested queries
- Fixed subquery execution within CTE contexts

### 📚 Documentation
- Added `docs/CTE_LIMITATIONS.md` - Current CTE implementation limitations
- Added `docs/GROUP_BY_LIMITATIONS.md` - GROUP BY expression support status

### 🔧 Technical Improvements
- Refactored QueryEngine to support CTE context threading
- Enhanced SubqueryExecutor with CTE context awareness
- Improved execution plan builder with new step types (CTE, Subquery, Aggregate, Distinct)
- Better timing instrumentation throughout query execution pipeline

## [1.44.0] - 2025-09-13

### 🚀 SQL Parser Modularization & Enhanced Nvim Plugin

This major release introduces a complete SQL parser refactoring for better maintainability, significant improvements to the Neovim plugin's query boundary detection, and new execution plan functionality.

### ✨ New Features

#### **SQL Parser Modularization (Phase 1)**
- **Modular parser architecture** - Refactored monolithic parser into specialized modules
- **Expression parser modules**:
  - `expressions/arithmetic.rs` - Arithmetic operations and math functions
  - `expressions/case.rs` - CASE/WHEN expressions
  - `expressions/comparison.rs` - Comparison operators and predicates  
  - `expressions/logical.rs` - AND, OR, NOT logical operations
- **Centralized type system** - Unified comparison logic with proper NULL handling
- **Improved maintainability** - Easier to extend and debug parser functionality
- **Enhanced error handling** - Better error messages and recovery

#### **Enhanced Neovim Plugin Query Detection**
- **Improved boundary detection** - Completely rewritten query-at-cursor logic
- **Consistent behavior** - All cursor functions now use same boundary detection:
  - `\sx` - Execute query at cursor
  - `\sX` - Execute query with detailed execution plan (NEW!)
  - `\sy` - Copy query to clipboard  
  - `\s/` - Smart comment toggle
  - `\s=` - Format query
  - `\sv` - Visual select query
  - `\sP` - Preview query in floating window
- **Smart terminator detection** - Properly handles `;` and `GO` statement separators
- **Multi-statement support** - Works with WITH, SELECT, INSERT, UPDATE, DELETE, CREATE, DROP, ALTER
- **Enhanced navigation** - `]q` and `[q` now find any SQL statement type, not just SELECT

#### **Execution Plan Integration**
- **New `\sX` keymap** - Execute query with detailed timing breakdown
- **Performance insights**:
  - Parse time, data loading time, query execution time
  - Row processing statistics (loaded, filtered, returned)
  - Column counts and memory usage
  - Total execution time breakdown
- **Same boundary detection** - Uses identical query detection as `\sx`

#### **Smart Comment Toggle**
- **Documentation-aware** - Preserves documentation comments when toggling
- **Distinguishes comment types**:
  - Documentation comments (preserved): `-- This explains the query`
  - Commented-out SQL code (toggled): `-- SELECT * FROM table`
- **Consistent boundaries** - Comments exactly what `\sx` would execute

### 🐛 Bug Fixes & Improvements

#### **Parser Stability**
- **Fixed compilation errors** - Resolved all test suite compilation issues
- **Removed unused imports** - Clean codebase with proper dependency management
- **Better error recovery** - Parser handles malformed queries more gracefully
- **Type system consistency** - Unified comparison logic across all data types

#### **Date and Boolean Handling**
- **Enhanced date parsing** - Better support for various date formats
- **Boolean type improvements** - Consistent boolean literal handling
- **NULL comparison fixes** - Proper three-valued logic implementation

#### **Plugin Reliability**
- **Boundary detection edge cases** - Handles queries at start/end of file
- **Empty line handling** - Proper trimming of whitespace in query boundaries
- **Comment preservation** - Never damages documentation when toggling comments

### 🏗️ Internal Improvements

#### **Code Organization**
- **Modular parser structure** - Easier to maintain and extend
- **Centralized type system** - Single source of truth for data type handling
- **Consistent APIs** - Unified interfaces across parser modules
- **Better test coverage** - All 342 tests passing with improved reliability

#### **Performance**
- **Execution plan insights** - Detailed performance metrics for query optimization
- **Memory efficiency** - Better data structure reuse in joins and aggregations
- **Parsing performance** - Modular structure enables better optimization

### 📚 Examples & Documentation

#### **New SQL Examples**
- **Chemistry examples** - JOIN queries with periodic table data
- **Complex CTEs** - Multi-CTE queries with joins and aggregations  
- **Execution plan demos** - Example files showing performance analysis

#### **JOIN Implementation Examples**
- **Working JOIN examples** in `examples/chemistry.sql`:
  - CTE with aggregations joined back to main table
  - LEFT JOIN examples with NULL handling
  - Self-joins and complex conditions
- **Documented limitations** - Current JOIN implementation constraints
- **Performance insights** - Using `\sX` to analyze JOIN performance

## [1.43.0] - 2025-09-10

### 🚀 Parser & Plugin Enhancements

This release introduces SQL JOIN parser support, significantly improves the Neovim plugin autocomplete, and fixes critical terminal handling issues.

### ✨ New Features

#### **SQL JOIN Parser Support**
- **Complete JOIN grammar implementation** - Parser now supports all standard SQL JOIN types
- **Supported JOIN types**: INNER, LEFT, RIGHT, FULL OUTER, CROSS
- **Table aliasing** - Full support for table aliases in FROM and JOIN clauses
- **Complex ON conditions** - Support for various comparison operators in JOIN conditions
- **CTE with JOINs** - CTEs can now be used in JOIN operations
- **Subquery JOINs** - Support for joining with subqueries
- Note: Parser only - execution implementation coming in future release

#### **Enhanced Neovim Plugin Autocomplete**
- **New `--schema-json` flag** - Clean JSON output for schema information without ANSI colors
- **Improved completion system** - Plugin now uses JSON parsing instead of regex for reliability
- **Better keybindings**:
  - `Alt+;` or `Alt+.` - Trigger column-specific completion
  - `Ctrl+Space` - General SQL completion
  - `Tab`/`Shift+Tab` - Navigate completion menu
  - `Enter` - Accept selected completion
  - `1-9` - Quick select numbered completion item
- **Smart schema detection** - Automatically loads schema from data file hints in SQL comments
- **Context-aware completions** - Shows columns, SQL functions, and keywords based on context

### 🐛 Bug Fixes & Improvements

#### **Terminal Corruption Fix**
- **Fixed terminal corruption on TUI crash** - Terminal now properly restores on errors
- **Added panic hook** - Automatically restores terminal state on panic
- **Enhanced error handling** - Terminal cleanup happens even when TUI fails to start
- **File validation timing** - Files are now validated before entering raw terminal mode
- Prevents the need to open new terminal when TUI fails with invalid file paths

#### **Parser Improvements**
- **Fixed function scope errors** in recursive_parser.rs
- **Added missing JOIN token patterns** in text navigation
- **Proper TableSource handling** for derived tables and subqueries
- **CROSS JOIN support** - Correctly handles CROSS JOIN without ON clause

### 📚 Examples

#### JOIN Parser Examples
```sql
-- Simple INNER JOIN
SELECT * FROM users JOIN orders ON users.id = orders.user_id;

-- LEFT JOIN with table aliases
SELECT * FROM users u LEFT JOIN orders o ON u.id = o.user_id;

-- Multiple JOINs
SELECT * FROM users 
JOIN orders ON users.id = orders.user_id
JOIN products ON orders.product_id = products.id;

-- JOIN with CTE
WITH active_users AS (SELECT * FROM users WHERE active = 1)
SELECT * FROM active_users JOIN orders ON active_users.id = orders.user_id;
```

#### Neovim Plugin Usage
```vim
" In your .vimrc or init.vim
" The plugin auto-detects data files from comments:
" -- #!data: data/sales.csv

" Then in insert mode:
" Type 'SELECT ' then press Alt+; to see column completions
" Use Tab to navigate, Enter to accept, or 1-9 for quick select
```

### 🔧 Technical Details
- JOIN AST structures: `JoinType`, `JoinOperator`, `JoinCondition`, `JoinClause`
- SelectStatement now includes `joins: Vec<JoinClause>` field
- Parser correctly handles table.column vs object.method() disambiguation in most cases
- Known limitation: WHERE clauses after JOINs may misinterpret table.column as method calls

## [1.42.0] - 2025-09-09

### 🚀 Major Performance & Functionality Improvements

This release delivers critical performance optimizations, powerful new aggregate capabilities, and smarter script execution.

### ✨ New Features

#### **COUNT(DISTINCT) and DISTINCT Aggregates**
- **Full DISTINCT support** for all aggregate functions: COUNT, SUM, AVG, MIN, MAX
- **COUNT(DISTINCT column)** - Count unique values within groups
- Works seamlessly with GROUP BY clauses
- Example: `SELECT region, COUNT(DISTINCT customer_id) FROM sales GROUP BY region`

#### **GROUP_NUM() Function**
- **Value enumeration function** - Assigns unique sequential numbers (0-based) to distinct values
- Maintains consistency across entire query execution
- Alternative to JOINs for creating unique identifiers
- Example: `SELECT order_id, GROUP_NUM(order_id) as order_num FROM orders`

#### **Data File Hint System**
- **Script data hints** - Specify data file in SQL scripts with `-- #!data: path/to/file.csv`
- Supports relative paths (resolved from script location)
- Command-line arguments override script hints
- Examples:
  - `-- #!data: ../data/sales.csv`
  - `-- #!datafile: /absolute/path/to/data.csv`

### 🐛 Bug Fixes & Improvements

#### **Performance Optimization**
- **Fixed severe performance issue** with script execution on large files
- Scripts no longer clone entire DataTable for each GO block
- Creates Arc<DataTable> once and reuses for all statements
- Dramatic speedup on 50k+ row datasets

#### **DUAL Script Support**
- **Scripts using only DUAL now work** without requiring a data file
- Automatically detects when scripts use DUAL, RANGE(), or no FROM clause
- Only requires data file when script references actual tables
- Fixes issue with pure SQL calculation scripts like chemical_formulas.sql

### 📚 Examples
```sql
-- COUNT(DISTINCT) in action
SELECT 
    root_order_id,
    COUNT(DISTINCT security_id) as unique_securities,
    SUM(DISTINCT quantity) as unique_quantities
FROM trades
GROUP BY root_order_id;

-- GROUP_NUM for enumeration
SELECT 
    customer,
    GROUP_NUM(customer) as customer_num,
    total_sales
FROM sales_summary
ORDER BY customer_num;

-- Script with data hint
-- #!data: ../data/production.csv
SELECT * FROM production WHERE status = 'active';
```

### 🔧 Technical Details
- Added `distinct` flag to SqlExpression::FunctionCall in parser
- Implemented evaluate_aggregate_distinct() for efficient unique value tracking
- Global memoization for GROUP_NUM using lazy_static
- Smart script analysis to determine data file requirements

## [1.41.0] - 2025-09-08

### 🚀 Major Enhancements

This release brings significant improvements to the SQL engine with new operators, window functions, and mathematical capabilities.

### ✨ New Features

#### **Operators & Expression Support**
- **Modulo operator (%)** - Now supports `value % 5` as an alias to `MOD(value, 5)`
- **OR operator in WHERE** - Fixed support for OR conditions like `WHERE (col = 'A' OR col = 'B')`
- **DISTINCT keyword** - Full support for `SELECT DISTINCT` to remove duplicate rows

#### **Window Functions**
- **SUM() window function** - Calculate sums over partitions: `SUM(amount) OVER (PARTITION BY category)`
- **COUNT() window function** - Enhanced with COUNT(*) support: `COUNT(*) OVER (PARTITION BY group)`
- **COUNT(column)** - Count non-null values in partitions

#### **Mathematical Functions**
- **SUM_N(n)** - Calculate triangular numbers (sum of first n natural numbers)
  - Formula: n * (n + 1) / 2
  - Example: `SUM_N(10)` returns 55

#### **RANGE Function & CTEs**
- **Comprehensive examples** - Added three new example files showcasing RANGE with CTEs:
  - `range_statistical_analysis.sql` - Statistical calculations
  - `range_test_data_generation.sql` - Mock data generation
  - `range_mathematical_sequences.sql` - Mathematical patterns

### 🐛 Bug Fixes
- Fixed OR operator not working in WHERE clauses with parentheses
- Fixed COUNT(*) not working as a window function (was parsed as StringLiteral instead of Column)
- Updated Python tests to match actual system capabilities

### 📚 Examples
```sql
-- Modulo operator
SELECT value, value % 3 AS remainder FROM RANGE(1, 10);

-- DISTINCT rows
SELECT DISTINCT category, status FROM products;

-- SUM window function with PARTITION BY
SELECT 
    region,
    sales_amount,
    SUM(sales_amount) OVER (PARTITION BY region) AS region_total
FROM sales_data;

-- Triangular numbers
SELECT n, SUM_N(n) AS triangular FROM RANGE(1, 10);
-- Returns: 1→1, 2→3, 3→6, 4→10, 5→15, etc.
```

### 🔧 Known Limitations
- CASE WHEN doesn't support AND/OR operators (use mathematical workarounds)
- GROUP BY only supports column names, not expressions (use CTEs as workaround)
- Cross-joins with multiple RANGE CTEs have column resolution issues
- No ROWS BETWEEN support in window functions yet

## [1.40.0] - 2025-09-07

### 🚀 Common Table Expressions (CTEs) Support

This release introduces full CTE (WITH clause) support, enabling powerful multi-stage queries and solving the "can't use alias in WHERE" limitation. CTEs can reference previous CTEs in the chain, unlocking advanced SQL patterns.

### ✨ New Features

#### **Common Table Expressions**
- **`WITH` clause** - Define named temporary result sets
- **CTE chaining** - Each CTE can reference ALL previous CTEs
- **Column aliasing** - Optional column list syntax `WITH cte_name (col1, col2) AS ...`
- **Window functions in CTEs** - Enables "top N per group" patterns
- **Materialized execution** - CTEs are evaluated once and cached

#### **NULL Handling Improvements**
- **NULL literal** - Proper NULL support in SQL expressions
- **IS NULL / IS NOT NULL** - Standard SQL null checking operators
- **CASE with NULL** - Correct NULL handling in CASE expressions
- **Arithmetic with NULL** - Operations with NULL correctly return NULL

#### **Function Registry Updates**
- **CONVERT()** - Moved from special handling to proper function registry
- **Physics constants** - Fixed function names (K, AVOGADRO, etc.)

### 📚 Examples
```sql
-- CTEs with chaining - each references the previous
WITH 
    filtered AS (SELECT * FROM data WHERE value > 100),
    aggregated AS (SELECT category, AVG(value) as avg_val FROM filtered GROUP BY category),
    top_categories AS (SELECT * FROM aggregated WHERE avg_val > 500)
SELECT * FROM top_categories ORDER BY avg_val DESC;

-- Top N per group using window functions in CTEs
WITH ranked AS (
    SELECT *, ROW_NUMBER() OVER (PARTITION BY category ORDER BY sales DESC) as rank
    FROM products
)
SELECT * FROM ranked WHERE rank <= 3;
```

### 🔧 Improvements
- **Subquery foundation** - CTE architecture enables future subquery support
- **Query optimization** - CTEs evaluated once, results cached
- **Examples** - Added comprehensive CTE cookbook and chaining examples

## [1.39.0] - 2025-09-06

### 🪟 Window Functions, Hash Functions & Geometry Formulas

This release adds powerful SQL window functions for analytics, cryptographic hash functions for data integrity, and mathematical geometry formulas for calculations.

### ✨ New Features

#### **Window Functions**
- **`LAG(column, offset)`** - Access previous row values within partition
- **`LEAD(column, offset)`** - Access next row values within partition
- **`ROW_NUMBER()`** - Assign sequential numbers within partition
- **`FIRST_VALUE(column)`** - Get first value in partition
- **`LAST_VALUE(column)`** - Get last value in partition
- Full support for `OVER (PARTITION BY ... ORDER BY ...)` clause
- Enables ranking, running totals, and trend analysis

#### **Hash Functions**
- **`MD5(value)`** - Calculate MD5 hash (32 chars)
- **`SHA1(value)`** - Calculate SHA1 hash (40 chars)
- **`SHA256(value)`** - Calculate SHA256 hash (64 chars)
- **`SHA512(value)`** - Calculate SHA512 hash (128 chars)
- Auto-converts numbers to strings for hashing
- Returns NULL for NULL inputs

#### **Geometry Functions**
- **`PYTHAGORAS(a, b)`** - Calculate hypotenuse using Pythagorean theorem
- **`CIRCLE_AREA(radius)`** - Calculate area of circle (πr²)
- **`CIRCLE_CIRCUMFERENCE(radius)`** - Calculate circumference (2πr)
- **`SPHERE_VOLUME(radius)`** - Calculate sphere volume (4/3πr³)
- **`SPHERE_SURFACE_AREA(radius)`** - Calculate sphere surface area (4πr²)
- **`TRIANGLE_AREA(a, b, c)`** - Calculate triangle area using Heron's formula
- **`DISTANCE_2D(x1, y1, x2, y2)`** - Calculate 2D Euclidean distance

### 🔧 Improvements
- **NULL Arithmetic Handling** - Any arithmetic operation with NULL now correctly returns NULL
- **WindowContext** - Efficient partitioned data management for window functions
- **Test Coverage** - Comprehensive Python test suite for all new functions
- **Examples** - Added window function SQL examples and sales_data.csv sample

### 📚 Examples
```sql
-- Window functions for analytics
SELECT salesperson, month, sales_amount,
       LAG(sales_amount, 1) OVER (PARTITION BY salesperson ORDER BY month) as prev_month,
       ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as rank
FROM sales_data;

-- Hash functions for data integrity
SELECT email, MD5(email) as email_hash, SHA256(password) as password_hash
FROM users;

-- Geometry calculations
SELECT PYTHAGORAS(3, 4) as hypotenuse,  -- Returns 5
       CIRCLE_AREA(10) as area,         -- Returns 314.159...
       TRIANGLE_AREA(3, 4, 5) as triangle_area;  -- Returns 6
```

## [1.38.0] - 2025-09-05

### 🔢 Prime Number Functions & Self-Documenting Registry

This release adds comprehensive prime number support with pre-computed tables for lightning-fast operations, plus a self-documenting function registry that automatically generates documentation.

### ✨ New Features

#### **Prime Number Functions**
- **`PRIME(n)`** - Returns the nth prime number (1-indexed)
  - Pre-computed 100,000 primes at compile time
  - O(1) access up to the 100,000th prime (1,299,709)
- **`IS_PRIME(n)`** - Tests if a number is prime
  - O(1) for numbers up to 1.3 million via HashSet lookup
  - Miller-Rabin algorithm for larger numbers
- **`PRIME_COUNT(n)`** - Returns count of primes ≤ n (π(n) function)
- **`NEXT_PRIME(n)`** - Returns smallest prime ≥ n
- **`PREV_PRIME(n)`** - Returns largest prime ≤ n

#### **Self-Documenting Function Registry**
- **`--list-functions`** - List all available SQL functions with descriptions
- **`--function-help <name>`** - Show detailed help for a specific function
- **`--generate-docs`** - Auto-generate markdown reference documentation
- All function metadata (description, arguments, examples) now in one place

#### **Prime Number Examples**
```sql
-- Get the 100th prime
SELECT PRIME(100);  -- Returns 541

-- Test primality
SELECT IS_PRIME(17), IS_PRIME(100);  -- true, false

-- Count primes up to 1000
SELECT PRIME_COUNT(1000);  -- Returns 168

-- Find twin primes (gap of 2)
SELECT n, PRIME(n), PRIME(n+1) 
FROM numbers WHERE PRIME(n+1) - PRIME(n) = 2;

-- Navigate primes
SELECT NEXT_PRIME(100), PREV_PRIME(100);  -- 101, 97
```

### 🚀 Performance
- Pre-computed prime tables use only ~400KB memory
- Instant access to first 100,000 primes
- Efficient primality testing via compile-time generation

### 🧪 Testing
- Comprehensive Python test suite for prime functions
- Tests include twin primes, Goldbach's conjecture, Sophie Germain primes
- Prime analysis demonstration script

### 📚 Documentation  
- Auto-generated FUNCTION_REFERENCE.md from registry
- Function help available directly from CLI
- Examples embedded in function signatures

## [1.37.0] - 2025-09-04

### 🎨 String Functions & Mathematical Constants

### ✨ New Features

#### **String Functions**
- **`MID(string, start, length)`** - Extract substring (1-indexed like SQL)
- **`UPPER(string)`** - Convert to uppercase
- **`LOWER(string)`** - Convert to lowercase  
- **`TRIM(string)`** - Remove leading/trailing whitespace

#### **Mathematical Constants**
- **`PI()`** - Returns π (3.14159...)
- **`E()`** - Returns Euler's number (2.71828...)

## [1.36.0] - 2025-09-02

### 🌌 Astronomical Constants & Solar System Calculations

This release transforms SQL CLI into a powerful scientific calculator with comprehensive astronomical constants for astrophysics and solar system calculations.

### ✨ New Features

#### **Astronomical Constants**
- **Particle Radii** - `RE()`, `RP()`, `RN()` for electron, proton, and neutron radii
- **Solar System Masses** - All planets, Sun, and Moon masses in kg
  - `MASS_SUN()` - 1.989×10³⁰ kg
  - `MASS_EARTH()` - 5.972×10²⁴ kg  
  - `MASS_MOON()` - 7.342×10²² kg
  - `MASS_MERCURY()` through `MASS_NEPTUNE()` for all planets
- **Orbital Distances** - Precise distances from Sun in meters
  - `DIST_MERCURY()` through `DIST_NEPTUNE()`
  - `AU()` - Astronomical Unit (1.496×10¹¹ m)
- **Distance Units** - `PARSEC()` and `LIGHTYEAR()` constants

#### **Scientific Calculations Now Possible**
```sql
-- Calculate Earth's surface gravity (9.82 m/s²)
SELECT G() * MASS_EARTH() / POWER(6.371e6, 2) FROM DUAL;

-- Escape velocity from Moon
SELECT SQRT(2 * G() * MASS_MOON() / 1.737e6) FROM DUAL;

-- Schwarzschild radius of the Sun
SELECT 2 * G() * MASS_SUN() / (C() * C()) FROM DUAL;

-- Kepler's Third Law orbital periods
SELECT SQRT(4*PI()*PI()*POWER(DIST_MARS(),3)/(G()*MASS_SUN()))/(365.25*24*3600) FROM DUAL;
```

### 🧪 Testing
- Added comprehensive test suite with 21 tests for astronomical calculations
- Tests cover Kepler's laws, escape velocities, gravitational forces, and planetary densities
- All 243 Python tests passing

### 📚 Documentation
- Updated README with dedicated astronomical constants section
- Added examples for astrophysics calculations
- Documented all available constants with scientific notation

## [1.33.0] - 2025-08-31

### 🐛 Critical Bug Fixes
- **Fixed SQL Parser Method Call Handling** - Resolved parser failures with string methods containing spaces
  - Fixed issue where `name.Trim()` would fail if followed by spaces in SELECT clause
  - Parser now correctly handles whitespace after method calls like `IndexOf(' ')`
  - Ensures proper tokenization of method calls with arguments
- **Fixed TEXTJOIN Function** - Corrected argument handling to require ignore_empty flag
  - Syntax: `TEXTJOIN(delimiter, ignore_empty, value1, value2, ...)`
  - Fixed to properly handle 3+ arguments as required
- **Fixed Date Function Syntax** - Standardized date function argument order
  - DATEDIFF: `DATEDIFF('unit', date1, date2)` 
  - DATEADD: `DATEADD('unit', amount, date)`
  - Unit parameter must be a string literal

### ✅ Testing Improvements
- Added comprehensive test coverage for 40+ SQL functions
- Created `test_math_date_functions.py` with 23 test cases
- Updated `run_python_tests.sh` to include all test suites
- All 69 tests passing (66 pass, 3 skipped for unimplemented aggregates)

## [1.32.0] - 2025-08-31

### 🎯 Major Features
- **Non-Interactive Query Mode** - Execute SQL queries directly from command line for scripting and automation
  - Run queries with `-q "SELECT ..."` or from file with `-f query.sql`
  - Multiple output formats: CSV, JSON, TSV, and pretty tables
  - Output to file with `-O results.csv`
  - Perfect for data pipelines and batch processing
  - Ultra-fast execution with microsecond response times

### 🚀 Features
- **Viewport Lock Mode** - Press Space to anchor scrolling position, data scrolls while cursor stays fixed
- **Dynamic Column Sizing** - Columns automatically adjust width based on visible viewport data
- **Compact Mode** - Press 'C' to reduce padding and fit more columns on screen
- **Auto-Execute for Files** - CSV/JSON files show data immediately on load with pre-filled query
- **Multi-Source Data Proxy** - Query SQL Server, APIs, and files seamlessly through unified interface
- **Visual Source Indicators** - Shows data source with colored icons (📦 Cache, 📁 File, 🌐 API, 🗄️ SQL)
- **Named Cache System** - Save queries with custom IDs like `:cache save trades_2024`
- **Rainbow Parentheses** - Visual matching for nested SQL queries
- **String.IsNullOrEmpty()** - LINQ-style null/empty checking in WHERE clauses
- **Schema-Aware History** - Command history with intelligent suggestions based on query context

### 🐛 Bug Fixes
- Fixed GitHub Actions deprecation warnings by updating to v4
- Fixed cache save to support named IDs
- Fixed formatting issues in CI/CD pipeline

### 📚 Documentation
- Comprehensive README with keyboard shortcuts
- Enhanced F1 help screen with all features
- Added MULTI_SOURCE_PROXY.md documentation
- Added tips section in help for feature discovery

### 🔧 Infrastructure
- Cross-platform CI/CD for Linux, Windows, macOS (x64 and ARM64)
- Automated release workflow with version bumping
- Pre-commit hooks for code formatting
- GitHub Actions permissions properly configured

## [1.0.0] - 2024-01-06

### Initial Release
- Full SQL parser with LINQ support
- Context-aware tab completion
- Professional TUI interface with split-view design
- Vim-like navigation and search
- Command history with search (Ctrl+R)
- CSV/JSON file support
- REST API integration
- Multi-line editor mode (F3)
- Export to CSV (Ctrl+S)
- Column sorting and filtering
- Cache management system

### Supported LINQ Methods
- String.Contains()
- String.StartsWith()
- String.EndsWith()
- String.IsNullOrEmpty()
- Property name normalization

### Platform Support
- Linux x64
- Windows x64
- macOS x64 (Intel)
- macOS ARM64 (Apple Silicon)