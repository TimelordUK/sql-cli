# SQL CLI Project

Vim-like terminal SQL editor with in-memory query engine for ultra-fast navigation and data exploration. Built in Rust using ratatui.

## Core Philosophy
- **Vim-inspired**: Modal editing, vim keybindings (hjkl navigation, i/a/A insert modes)
- **In-memory queries**: All data loaded to memory for instant filtering/sorting
- **Fast navigation**: Optimized for keyboard-only workflow with minimal latency
- **Power user focused**: Built for rapid data exploration without mouse

## Tech Stack
- **Language**: Rust 1.26.2
- **TUI Framework**: ratatui + crossterm (high-performance terminal UI)
- **SQL Parser**: Custom recursive descent parser with AST (in-memory evaluation)
- **Data Sources**: CSV, JSON files, REST API (cached in memory)

## Build & Test Commands
```bash
# Build
cargo build --releaseds

# Run tests
cargo test
cargo test --test data_view_trades_test  # Important DataView tests

# IMPORTANT: Always run before committing!
cargo fmt  # Required - formats all code to project standards

# Lint
cargo clippy

# Run application
./target/release/sql-cli <file.csv>
./target/release/sql-cli --enhanced <file.json>
```

## agents
- use the rust build fixer to fix any compilation issues
- use the unit test fixer to correct unit test breaks

## Project Structure
- `src/ui/enhanced_tui.rs` - Main TUI interface (key handling to be migrated)
- `src/app_state_container.rs` - Central state management
- `src/data/data_view.rs` - DataView with column operations
- `src/handlers/` - Event handlers (migration in progress)
- `src/action.rs` - Action system for state updates
- `src/sql/` - SQL parsing and AST evaluation
- `integration_tests/` - Integration test suite
- `integration_tests/test_scripts/` - Shell script test suite

## Current Work: Key Handler Migration
Migrating key handling from TUI main loop to dedicated action system. See KEY_MIGRATION_STATUS.md for details.

**Branch**: key_migration_v2 (based on tui_widgets_v1)

**Recently Fixed**:
- Column sorting with pinned columns
- Unified visible_columns architecture
- Key history display (10 keys max, 2s fade)

## Vim-like Features
- **Modal editing**: Insert (i), Append (a/A), Command mode
- **Vim navigation**: hjkl for movement, g/G for top/bottom
- **Fast column ops**: Pin (p), Hide (H), Sort (s) - single keystrokes
- **Search modes**: `/` for column search, `?` for data search, n/N to navigate
- **Visual feedback**: Key history display, mode indicators

## Performance Features
- **In-memory operations**: All queries run on cached data
- **Virtual scrolling**: Handle 100K+ rows smoothly
- **Instant filtering**: Fuzzy search, regex, SQL WHERE - all sub-second
- **Zero-latency navigation**: Optimized keyboard response
- **Smart caching**: Query results cached for instant re-filtering

## Performance Targets
- 10K-100K rows: Interactive queries (50-200ms)
- Complex queries on 100K rows: ~100-200ms
- Memory: ~50MB for 100K rows

## Testing Scripts
```bash
# Column operations
./integration_tests/test_scripts/test_column_ops.sh

# Sorting with cycles
./integration_tests/test_scripts/test_sort_cycles.sh

# TUI sort fixes
./integration_tests/test_scripts/test_tui_sort_fix.sh
```

## Important Notes
- **ALWAYS run `cargo fmt` before committing** - This is required for all commits
- Windows compatibility required before merge
- Direct DataView state manipulation being removed
- Action system handles all state changes
- F5 shows debug view with internal state

## Documentation
Extensive docs in `docs/` folder covering architecture, refactoring plans, and feature designs.