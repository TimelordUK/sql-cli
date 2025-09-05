# SQL CLI v1.38.0

**Release Date:** September 05, 2025

## 📊 Release Overview
- **Commits in this release:** 19
- **Files updated:** 32

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

### 💾 Data Protection
- **History Recovery**: Automatic recovery from corrupted files
- **Atomic Writes**: Safer file operations to prevent data loss

## 📝 Changes by Category

### 🚀 New Features
- Move TEXTJOIN to function registry
- Add self-documenting function registry with CLI options
- Add comprehensive prime number functions with pre-computed tables
- Add comparison functions (GREATEST, LEAST, COALESCE, NULLIF)
- Create registry-based string methods system
- Add molecular formula parsing to ATOMIC_MASS function

### 🐛 Bug Fixes
- Move string functions (MID, UPPER, LOWER, TRIM) to function registry

### 🔧 Refactoring
- Migrate 19 math functions from arithmetic_evaluator to function registry

### 📚 Documentation
- Add comparison functions and string functions to README
- Add GROUP BY architecture design document
- Update README with molecular formula capabilities

<details>
<summary>📋 View all commits</summary>

- add get chemical formula function (TimelordUK)
- fix maths tests (TimelordUK)
- refactor chemistry to more easily add new molecules (TimelordUK)
- migrate functions (TimelordUK)
- refactor: Migrate 19 math functions from arithmetic_evaluator to function registry (TimelordUK)
- feat: Move TEXTJOIN to function registry (TimelordUK)
- fix: Move string functions (MID, UPPER, LOWER, TRIM) to function registry (TimelordUK)
- chore: Bump version to 1.38.0 for prime functions release (TimelordUK)
- feat: Add self-documenting function registry with CLI options (TimelordUK)
- feat: Add comprehensive prime number functions with pre-computed tables (TimelordUK)
- fix tests (TimelordUK)
- fix unit tests (TimelordUK)
- add least_label, greatest_label (TimelordUK)
- docs: Add comparison functions and string functions to README (TimelordUK)
- feat: Add comparison functions (GREATEST, LEAST, COALESCE, NULLIF) (TimelordUK)
- feat: Create registry-based string methods system (TimelordUK)
- docs: Add GROUP BY architecture design document (TimelordUK)
- docs: Update README with molecular formula capabilities (TimelordUK)
- feat: Add molecular formula parsing to ATOMIC_MASS function (TimelordUK)

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
