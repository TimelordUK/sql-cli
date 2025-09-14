# Changelog

All notable changes to SQL CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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