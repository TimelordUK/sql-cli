# Changelog

All notable changes to SQL CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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