# Changelog

All notable changes to SQL CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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