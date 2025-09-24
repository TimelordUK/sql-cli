# Test Consolidation Plan

## Current Problem
- 64 integration test files in `tests/`
- 14 binary files in `src/bin/`
- Each creates a separate executable that needs linking
- Significantly increases build time

## Proposed Solution

### 1. Consolidate Integration Tests
Instead of 64 separate test files, group them into logical categories:

```
tests/
├── lib.rs                    # Main test module
├── parser_tests.rs           # All parser-related tests
├── data_tests.rs            # DataTable, DataView tests
├── query_tests.rs           # Query execution tests
├── function_tests.rs        # Function registry tests
├── window_tests.rs          # Window function tests
├── cte_tests.rs             # CTE tests
├── performance_tests.rs     # Performance benchmarks
└── ui_tests.rs              # TUI-related tests
```

### 2. Move Test Binaries to Test Modules
Convert test binaries in `src/bin/` to test modules:
- `test_visual_simple.rs` → Move to `tests/ui_tests.rs`
- `test_linq_parse.rs` → Move to `tests/parser_tests.rs`
- `test_viewport_efficiency.rs` → Move to `tests/performance_tests.rs`
- etc.

### 3. Keep Only Essential Binaries
Keep in `src/bin/`:
- Main application binary
- Essential debugging tools (if actively used)

Move to tests or examples:
- Performance test binaries
- Debug utilities
- Test harnesses

### 4. Use Workspace Test Configuration
In `Cargo.toml`:
```toml
[profile.test]
opt-level = 1  # Some optimization for faster test execution
lto = false    # Disable LTO for faster linking

[[test]]
name = "integration"
path = "tests/lib.rs"
```

## Benefits
- Reduce from ~78 executables to ~10
- Significantly faster incremental builds
- Faster CI/CD pipelines
- Easier to run all tests

## Implementation Steps
1. Create test module structure
2. Move and consolidate test files
3. Update imports and module declarations
4. Clean up unused binaries
5. Update CI configuration if needed