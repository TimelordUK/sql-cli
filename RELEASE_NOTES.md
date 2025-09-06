# SQL CLI v1.39.0

**Release Date:** September 06, 2025

## 📊 Release Overview
- **Commits in this release:** 20
- **Files updated:** 266

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

### 💾 Data Protection
- **History Recovery**: Automatic recovery from corrupted files
- **Atomic Writes**: Safer file operations to prevent data loss

## 📝 Changes by Category

### 🚀 New Features
- v1.39.0 - Add hash functions and geometry formulas
- Add window functions support (LAG, LEAD, ROW_NUMBER, FIRST_VALUE, LAST_VALUE)
- add window functions parsing and WindowContext infrastructure
- implement STDDEV and VARIANCE aggregate functions
- add comprehensive SQL-CLI feature showcase and test script
- implement FACTORIAL function and clean up examples
- add PRIME_PI and NTH_PRIME functions, improve script parser
- implement HAVING clause for GROUP BY filtering
- implement GROUP BY with aggregate function support

### 🐛 Bug Fixes
- clean up string_functions.sql to only show implemented functions
- correct function names in physics_constants.sql
- correct function names in chemical examples
- apply clippy pedantic fixes and code formatting
- resolve clippy warnings and improve code quality

### 🔧 Refactoring
- proxy string methods through function registry
- move date functions from arithmetic_evaluator to function registry
- move constants from arithmetic_evaluator to function registry

### 📚 Documentation
- update README with GROUP BY feature documentation

<details>
<summary>📋 View all commits</summary>

- feat: v1.39.0 - Add hash functions and geometry formulas (TimelordUK)
- feat: Add window functions support (LAG, LEAD, ROW_NUMBER, FIRST_VALUE, LAST_VALUE) (TimelordUK)
- feat: add window functions parsing and WindowContext infrastructure (TimelordUK)
- refactor: proxy string methods through function registry (TimelordUK)
- feat: implement STDDEV and VARIANCE aggregate functions (TimelordUK)
- feat: add comprehensive SQL-CLI feature showcase and test script (TimelordUK)
- fix: clean up string_functions.sql to only show implemented functions (TimelordUK)
- fix: correct function names in physics_constants.sql (TimelordUK)
- feat: implement FACTORIAL function and clean up examples (TimelordUK)
- fix: correct function names in chemical examples (TimelordUK)
- feat: add PRIME_PI and NTH_PRIME functions, improve script parser (TimelordUK)
- fix: apply clippy pedantic fixes and code formatting (TimelordUK)
- fix: resolve clippy warnings and improve code quality (TimelordUK)
- feat: implement HAVING clause for GROUP BY filtering (TimelordUK)
- docs: update README with GROUP BY feature documentation (TimelordUK)
- feat: implement GROUP BY with aggregate function support (TimelordUK)
- refactor: move date functions from arithmetic_evaluator to function registry (TimelordUK)
- refactor: move constants from arithmetic_evaluator to function registry (TimelordUK)
- edit distance function (TimelordUK)
- fix python tests (TimelordUK)

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
