# SQL CLI v1.53.0

**Release Date:** September 23, 2025

## 📊 Release Overview
- **Commits in this release:** 33
- **Files updated:** 45

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🚀 New Features
- Add utility functions for character codes, type conversions, and encoding
- Add JWT token refresh system for nvim plugin
- Add generic macro template system with function patterns for nvim plugin
- Add recursive macro expansion support
- File-based configuration for template system
- Enhanced template system with inline SQL macros
- Add SQL template/macro system for Neovim plugin
- Enhance WEB CTE with HTTP methods, JSON path extraction, and request body support
- Add VALUES and ARRAY generators for literal value tables
- Add big_math.sql example and parser refactoring TODO

### 🐛 Bug Fixes
- Handle window keywords in ORDER BY clause
- Handle window keywords as column names in expression context
- Complete WEB CTE formatter support and nvim macro improvements
- Skip formatting for WEB CTE queries in nvim plugin
- Resolve environment variables in nvim macro CONFIG section
- Preserve buffer content when expanding macros
- Update TODAYS_TRADES macro to include SOURCE variable
- Handle multi-line macro expansions correctly
- Allow underscores in macro names for @MACRO pattern
- Keep table headers visible when navigating large result sets in Neovim
- Use localhost URLs in WEB CTE tests to avoid CI timeouts

### 🔧 Refactoring
- Replace string literals with token-based parsing in cursor context analysis
- Reorganize Neovim keybindings for better organization
- Replace hardcoded SQL keywords with proper Token matching in parser
- Major parser improvements - remove hardcoded logic, add tracing

### 📚 Documentation
- Add token management documentation

<details>
<summary>📋 View all commits</summary>

- fix format (TimelordUK)
- failing test sql (TimelordUK)
- fix: Handle window keywords in ORDER BY clause (TimelordUK)
- fix: Handle window keywords as column names in expression context (TimelordUK)
- feat: Add utility functions for character codes, type conversions, and encoding (TimelordUK)
- docs: Add token management documentation (TimelordUK)
- feat: Add JWT token refresh system for nvim plugin (TimelordUK)
- feat: Add generic macro template system with function patterns for nvim plugin (TimelordUK)
- refactor: Replace string literals with token-based parsing in cursor context analysis (TimelordUK)
- fix: Complete WEB CTE formatter support and nvim macro improvements (TimelordUK)
- Revert "fix: Skip formatting for WEB CTE queries in nvim plugin" (TimelordUK)
- fix: Skip formatting for WEB CTE queries in nvim plugin (TimelordUK)
- fix: Resolve environment variables in nvim macro CONFIG section (TimelordUK)
- fix: Preserve buffer content when expanding macros (TimelordUK)
- fix: Update TODAYS_TRADES macro to include SOURCE variable (TimelordUK)
- feat: Add recursive macro expansion support (TimelordUK)
- fix: Handle multi-line macro expansions correctly (TimelordUK)
- feat: File-based configuration for template system (TimelordUK)
- fix: Allow underscores in macro names for @MACRO pattern (TimelordUK)
- feat: Enhanced template system with inline SQL macros (TimelordUK)
- feat: Add SQL template/macro system for Neovim plugin (TimelordUK)
- refactor: Reorganize Neovim keybindings for better organization (TimelordUK)
- fix: Keep table headers visible when navigating large result sets in Neovim (TimelordUK)
- fix: Use localhost URLs in WEB CTE tests to avoid CI timeouts (TimelordUK)
- refactor: Replace hardcoded SQL keywords with proper Token matching in parser (TimelordUK)
- feat: Enhance WEB CTE with HTTP methods, JSON path extraction, and request body support (TimelordUK)
- update documents around progress in parser and tokenisation (TimelordUK)
- add a generator example for a literal array of numbers (TimelordUK)
- feat: Add VALUES and ARRAY generators for literal value tables (TimelordUK)
- refactor: Major parser improvements - remove hardcoded logic, add tracing (TimelordUK)
- improve big math (TimelordUK)
- add big_math example (TimelordUK)
- feat: Add big_math.sql example and parser refactoring TODO (TimelordUK)

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
