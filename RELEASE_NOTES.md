# SQL CLI v1.57.1

**Release Date:** October 06, 2025

## 📊 Release Overview
- **Commits in this release:** 36
- **Files updated:** 103

## ✨ Highlights

### 🎨 Visual Improvements

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🚀 New Features
- Add flexible date parsing functions and improve Nvim plugin UX
- Add FIX Protocol timestamp support and millisecond precision
- Implement UNNEST row expansion for delimited strings
- Add isolated row expansion system with UNNEST implementation
- Add UNNEST parser support and test data for FIX allocations
- Add UNNEST foundation - AST, lexer, and formatter support
- Add FIX timestamp, currency codes, and PendingCancel status highlighting
- Enhanced FIX message syntax highlighting with case-insensitive matching
- Add FIX message syntax highlighting examples for nvim plugin
- Add configurable table styles and custom syntax highlighting
- Add --cache-purge flag for easy cache management
- Complete data model navigation with Neovim-native scrolling
- Add data model architecture for structured query results
- Add raw query analysis JSON viewer (\sJ)
- Add CACHE, FORM_FILE, and FORM_FIELD support to CTE extraction
- Add CLI analysis foundation + CTE Tester V3

### 🐛 Bug Fixes
- Cache purge pattern - use sql-cli: instead of sql_cli:
- Cache purge command path and which-key discoverability
- Complete CTE extraction improvements for WEB CTEs
- Correct WEB CTE HEADERS syntax (colon + commas)
- Extract ALL CTEs up to target for executable queries

### 🔧 Refactoring
- Use jq for JSON pretty printing if available
- Remove old CTE tester versions and consolidate to CLI-based analysis
- Unify expression formatting to eliminate code duplication

### 📚 Documentation
- Add comprehensive UNNEST performance analysis
- Add UNNEST examples with FIX allocations demo
- Add comprehensive UNNEST implementation guide
- Add project summary and overview
- Add plugin data model evolution architecture
- Add START_HERE guide for new contributors
- Organize documentation and create 2025 roadmap

<details>
<summary>📋 View all commits</summary>

- feat: Add flexible date parsing functions and improve Nvim plugin UX (TimelordUK)
- fix test showing delta with lag (TimelordUK)
- feat: Add FIX Protocol timestamp support and millisecond precision (TimelordUK)
- docs: Add comprehensive UNNEST performance analysis (TimelordUK)
- docs: Add UNNEST examples with FIX allocations demo (TimelordUK)
- feat: Implement UNNEST row expansion for delimited strings (TimelordUK)
- docs: Add comprehensive UNNEST implementation guide (TimelordUK)
- feat: Add isolated row expansion system with UNNEST implementation (TimelordUK)
- feat: Add UNNEST parser support and test data for FIX allocations (TimelordUK)
- feat: Add UNNEST foundation - AST, lexer, and formatter support (TimelordUK)
- feat: Add FIX timestamp, currency codes, and PendingCancel status highlighting (TimelordUK)
- feat: Enhanced FIX message syntax highlighting with case-insensitive matching (TimelordUK)
- color syntax for fix (TimelordUK)
- feat: Add FIX message syntax highlighting examples for nvim plugin (TimelordUK)
- fix: Cache purge pattern - use sql-cli: instead of sql_cli: (TimelordUK)
- fix: Cache purge command path and which-key discoverability (TimelordUK)
- feat: Add configurable table styles and custom syntax highlighting (TimelordUK)
- feat: Add --cache-purge flag for easy cache management (TimelordUK)
- revert: Remove data model architecture - return to text-based navigation (TimelordUK)
- feat: Complete data model navigation with Neovim-native scrolling (TimelordUK)
- feat: Add data model architecture for structured query results (TimelordUK)
- refactor: Use jq for JSON pretty printing if available (TimelordUK)
- feat: Add raw query analysis JSON viewer (\sJ) (TimelordUK)
- refactor: Remove old CTE tester versions and consolidate to CLI-based analysis (TimelordUK)
- feat: Add CACHE, FORM_FILE, and FORM_FIELD support to CTE extraction (TimelordUK)
- refactor: Unify expression formatting to eliminate code duplication (TimelordUK)
- fix: Complete CTE extraction improvements for WEB CTEs (TimelordUK)
- fix: Correct WEB CTE HEADERS syntax (colon + commas) (TimelordUK)
- fix: Extract ALL CTEs up to target for executable queries (TimelordUK)
- feat: Add CLI analysis foundation + CTE Tester V3 (TimelordUK)
- fitlering functions (TimelordUK)
- fix example tester (TimelordUK)
- docs: Add project summary and overview (TimelordUK)
- docs: Add plugin data model evolution architecture (TimelordUK)
- docs: Add START_HERE guide for new contributors (TimelordUK)
- docs: Organize documentation and create 2025 roadmap (TimelordUK)

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
