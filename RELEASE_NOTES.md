# SQL CLI v1.41.0

**Release Date:** September 07, 2025

## 📊 Release Overview
- **Commits in this release:** 17
- **Files updated:** 33

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🚀 New Features
- Add SUM_N function for triangular numbers
- Add SUM() and COUNT() as window functions
- Add DISTINCT support for SELECT queries
- Add % modulo operator support as alias to MOD function

### 🐛 Bug Fixes
- Support COUNT(*) OVER (PARTITION BY ...) window function
- Update examples to work around CASE WHEN limitations
- Add support for OR operator in WHERE clause

### 📚 Documentation
- Add example SQL files showcasing RANGE with CTEs and window functions

<details>
<summary>📋 View all commits</summary>

- chore: Release version 1.41.0 (TimelordUK)
- fix: Support COUNT(*) OVER (PARTITION BY ...) window function (TimelordUK)
- fix: Update examples to work around CASE WHEN limitations (TimelordUK)
- feat: Add SUM_N function for triangular numbers (TimelordUK)
- docs: Add example SQL files showcasing RANGE with CTEs and window functions (TimelordUK)
- feat: Add SUM() and COUNT() as window functions (TimelordUK)
- test: Update Python tests to match actual system capabilities (TimelordUK)
- fix the example suite (TimelordUK)
- fix: Add support for OR operator in WHERE clause (TimelordUK)
- add distinct (TimelordUK)
- feat: Add DISTINCT support for SELECT queries (TimelordUK)
- feat: Add % modulo operator support as alias to MOD function (TimelordUK)
- fix format (TimelordUK)
- primes (TimelordUK)
- add prime finder sql (TimelordUK)
- trade example using cte (TimelordUK)
- add range conversions (TimelordUK)

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
