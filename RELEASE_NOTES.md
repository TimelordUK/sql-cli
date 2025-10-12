# SQL CLI v1.60.0

**Release Date:** October 12, 2025

## 📊 Release Overview
- **Commits in this release:** 30
- **Files updated:** 39

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🚀 New Features
- Add comprehensive hedge fund execution analysis example
- Add FIX execution endpoint to mock API server
- Add dependency-aware statement execution for scripts
- Add PI_DIGIT(n) function for Nth decimal digit lookup
- Add PI_DIGITS() function for arbitrary precision π
- Add script dependency analyzer for temp table tracking

### 🐛 Bug Fixes
- Register temp tables after executing INTO statements
- Fix statement counting and case-insensitive GO matching for \sx
- Correct statement number counting in Neovim plugin for dependency-aware execution
- Support alternative SQL Server INTO syntax and refactor to AST-based dependency analysis
- Only show 'rows affected' message for table output format
- Add missing ColumnRef import in window_functions/mod.rs

### 🔧 Refactoring
- Extract classic console mode to function (~98 lines)
- Extract key debugger and config generation handlers (115 lines)
- Create argument parsing context object (Python argparse style)
- Extract non-interactive query mode to local function (132 lines removed)
- Extract schema handlers to main_handlers.rs (174 lines removed)
- Extract benchmark handler to main_handlers.rs (87 lines removed)
- Extract distinct column handler to main_handlers.rs (113 lines removed)
- Extract documentation handlers to main_handlers.rs (255 lines removed)
- Extract handler functions from main() (92 lines removed)
- Extract CLI handlers to organized cli module (299 lines removed from main.rs)

### 📚 Documentation
- Move WEB CTE temp tables test to examples with ABS demo
- Add π digits example to README showcasing RANGE query
- Add --execute-statement feature plan

<details>
<summary>📋 View all commits</summary>

- chore: Bump version to 1.60.0 (TimelordUK)
- feat: Add comprehensive hedge fund execution analysis example (TimelordUK)
- feat: Add FIX execution endpoint to mock API server (TimelordUK)
- docs: Move WEB CTE temp tables test to examples with ABS demo (TimelordUK)
- test: Add WEB CTE with temp tables integration test (TimelordUK)
- fix: Register temp tables after executing INTO statements (TimelordUK)
- fix: Fix statement counting and case-insensitive GO matching for \sx (TimelordUK)
- fix: Correct statement number counting in Neovim plugin for dependency-aware execution (TimelordUK)
- fix: Support alternative SQL Server INTO syntax and refactor to AST-based dependency analysis (TimelordUK)
- feat: Add dependency-aware statement execution for scripts (TimelordUK)
- refactor: Extract classic console mode to function (~98 lines) (TimelordUK)
- refactor: Extract key debugger and config generation handlers (115 lines) (TimelordUK)
- add a temp chart examples (TimelordUK)
- first 10k places of PI and find nth place of pi (TimelordUK)
- docs: Add π digits example to README showcasing RANGE query (TimelordUK)
- feat: Add PI_DIGIT(n) function for Nth decimal digit lookup (TimelordUK)
- feat: Add PI_DIGITS() function for arbitrary precision π (TimelordUK)
- add prime examples (TimelordUK)
- docs: Add --execute-statement feature plan (TimelordUK)
- refactor: Create argument parsing context object (Python argparse style) (TimelordUK)
- refactor: Extract non-interactive query mode to local function (132 lines removed) (TimelordUK)
- refactor: Extract schema handlers to main_handlers.rs (174 lines removed) (TimelordUK)
- refactor: Extract benchmark handler to main_handlers.rs (87 lines removed) (TimelordUK)
- refactor: Extract distinct column handler to main_handlers.rs (113 lines removed) (TimelordUK)
- refactor: Extract documentation handlers to main_handlers.rs (255 lines removed) (TimelordUK)
- refactor: Extract handler functions from main() (92 lines removed) (TimelordUK)
- refactor: Extract CLI handlers to organized cli module (299 lines removed from main.rs) (TimelordUK)
- feat: Add script dependency analyzer for temp table tracking (TimelordUK)
- fix: Only show 'rows affected' message for table output format (TimelordUK)
- fix: Add missing ColumnRef import in window_functions/mod.rs (TimelordUK)

</details>

## 🎯 Key Features

- **Instant Data Preview**: CSV/JSON files load immediately
- **Visual Feedback**: Key press indicator, cell highlighting
- **Advanced Navigation**: Vim-style keys, viewport/cursor lock
- **Powerful Search**: Regular search (Ctrl+F), fuzzy filter (Ctrl+/)
- **Data Export**: Save as CSV or JSON
- **Debug Mode**: Press F5 for comprehensive state information

## 📦 Installation

Download the binary for your platform from the assets below.

---
**Thank you for using SQL CLI!** 🎉

Report issues: [GitHub Issues](https://github.com/TimelordUK/sql-cli/issues)
