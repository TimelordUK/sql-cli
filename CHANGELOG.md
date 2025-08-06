# Changelog

All notable changes to SQL CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2024-01-06

### Added
- **LINQ String Functions**: Full support for string operations in WHERE clauses
  - `Contains()` - Check if string contains substring
  - `StartsWith()` - Check if string starts with prefix
  - `EndsWith()` - Check if string ends with suffix
  - `Length()` - Compare string length
  - `ToUpper()/ToLower()` - Case-insensitive comparisons
  - `IsNullOrEmpty()` - Check for null or empty strings
- **Rainbow Parentheses**: Nested parentheses now display in different colors for better readability
- **Schema-Aware History**: Command history now tracks and prioritizes based on:
  - Data source (CSV file, API endpoint)
  - Column names used in queries
  - Query metadata (tables, functions, WHERE columns)
- **Cross-Platform Support**: Full Windows, Linux, and macOS (Intel & ARM) support
- **Enhanced Autocomplete**: Context-aware suggestions for columns and LINQ methods
- **Virtual Table Rendering**: Efficient scrolling for large datasets
- **Multi-Line Query Editor**: F3 key for complex query editing
- **Comprehensive Test Suite**: Unit and integration tests for all features

### Changed
- Autocomplete now uses single quotes for string literals (SQL standard)
- Improved performance for large CSV files (100k+ rows)
- Better error messages with query context

### Fixed
- Virtual viewport performance issues with large datasets
- Quoted identifier handling in column names with spaces
- Parser precedence for complex WHERE clauses
- Memory efficiency when caching large result sets

## [0.1.0] - 2023-12-01

### Added
- Initial release
- Basic SQL SELECT support
- CSV file parsing
- WHERE clause filtering
- ORDER BY sorting
- Basic TUI interface