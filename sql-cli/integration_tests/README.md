# Integration Tests

This directory contains all integration and test files for the SQL CLI project.

## Directory Structure

```
integration_tests/
├── test_scripts/       # Shell scripts for testing features
├── test_data/         # CSV and other data files for tests
└── *.rs              # Rust integration test files
```

## Organization

### Shell Scripts (`test_scripts/`)
- `test_all_fixes.sh` - Comprehensive test suite for all fixes
- `test_buffer_switch.sh` - Tests buffer switching functionality
- `test_column_search.sh` - Tests column search feature
- `test_datetime.sh` - Tests datetime handling
- `test_enhancements.sh` - Tests various enhancements
- `test_filter_clearing.sh` - Tests filter clearing functionality
- `test_fuzzy_filter.sh` - Tests fuzzy filtering
- `test_history_protection.sh` - Tests history protection mechanisms
- `test_history_search.sh` - Tests Ctrl+R history search functionality
- `test_multi_files.sh` - Tests multiple file handling
- `test_multiline.sh` - Tests multiline query support
- `test_navigation.sh` - Tests navigation features
- `test_query_flow.sh` - Tests query execution flow
- `test_release.sh` - Release testing script

### Rust Test Files (*.rs)
These are standalone test programs that can be compiled and run individually:

#### Core Functionality
- `test_cache.rs` - Cache system tests
- `test_csv.rs` - CSV parsing and handling
- `test_json_cli.rs` - JSON data source tests
- `test_datetime.rs` - DateTime parsing tests

#### Parser Tests
- `test_linq_methods.rs` - LINQ method parsing
- `test_parser_errors.rs` - Parser error handling
- `test_quoted_debug.rs`, `test_quoted_debug2.rs` - Quoted string parsing
- `test_quoted_method_completion.rs` - Completion in quoted contexts
- `test_paren_method_completion.rs` - Parenthesis method completion
- `test_order_by_completion.rs` - ORDER BY clause completion

#### Query Tests
- `test_csv_queries.rs` - CSV-specific queries
- `test_csv_issues.rs` - Known CSV query issues
- `test_not_contains.rs` - NOT Contains operator
- `test_order_by.rs`, `test_order_by_csv.rs` - ORDER BY functionality
- `test_where_order_by_issue.rs` - WHERE + ORDER BY combinations
- `test_trades_equality.rs` - Equality comparisons

#### UI/Navigation Tests
- `test_column_autofit.rs` - Column width auto-fitting
- `test_column_navigation.rs` - Column navigation
- `test_column_pinning.rs` - Column pinning feature
- `test_shift_g.rs` - Shift+G navigation
- `test_sorting.rs` - Data sorting

#### Performance Tests
- `test_large_dataset_perf.rs` - Large dataset performance

#### Other Tests
- `test_case_insensitive_csv.rs` - Case-insensitive CSV handling
- `test_null_handling.rs` - NULL value handling
- `test_history_debug.rs`, `test_history_unit.rs` - History system tests
- `test_state_init.rs` - State initialization

### Test Data (`test_data/`)
- Sample CSV files with various data types and structures
- Query result exports for regression testing
- Test fixtures for specific scenarios

## Running Tests

### Shell Scripts
```bash
# From project root
./integration_tests/test_scripts/test_history_search.sh

# Or for version-specific tests
./integration_tests/test_scripts/test_v46_datatable.sh
```

### Rust Test Files
```bash
# Run all integration tests
cargo test --test '*'

# Run specific test
cargo test --test test_csv

# With debug output
RUST_LOG=debug cargo test --test test_name -- --nocapture
```

## Version Tests

Tests are versioned to match our DataTable migration strategy:
- **V40-V45**: Trait-based migration (✅ complete)
- **V46-V50**: DataTable introduction (🚧 in progress)
- **V51-V60**: DataView implementation (📋 planned)
- **V61-V70**: Full migration completion (📋 planned)

## Note
These tests were moved from the main project directory to keep it clean and organized.
Test scripts may need path adjustments if test data locations have changed.