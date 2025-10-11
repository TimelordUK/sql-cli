# SQL CLI v1.59.0

**Release Date:** October 11, 2025

## 📊 Release Overview
- **Commits in this release:** 13
- **Files updated:** 27

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🚀 New Features
- Add comment-aware tokenization foundation
- Add UNION (with deduplication) support
- Add UNION ALL support for combining SELECT query results

### 🐛 Bug Fixes
- Correct SELECT INTO syntax in tmp_table.sql example
- Support SELECT INTO #temp formatting
- Restore normal buffer navigation after toggling out of table mode

### 🔧 Refactoring
- Replace regex-based INTO removal with AST preprocessing

### 📚 Documentation
- Mark CODE CTE feature as not proceeding
- Add session summary for 2025-01-11 CODE CTE investigation
- Add lexer and parser considerations analysis
- Add CODE CTE design document for programmable data transformations

<details>
<summary>📋 View all commits</summary>

- chore: Bump version to v1.59.0 (TimelordUK)
- perf: Fix 23x performance regression in WHERE clause evaluation (TimelordUK)
- feat: Add comment-aware tokenization foundation (TimelordUK)
- refactor: Replace regex-based INTO removal with AST preprocessing (TimelordUK)
- fix: Correct SELECT INTO syntax in tmp_table.sql example (TimelordUK)
- fix: Support SELECT INTO #temp formatting (TimelordUK)
- fix: Restore normal buffer navigation after toggling out of table mode (TimelordUK)
- feat: Add UNION (with deduplication) support (TimelordUK)
- feat: Add UNION ALL support for combining SELECT query results (TimelordUK)
- docs: Mark CODE CTE feature as not proceeding (TimelordUK)
- docs: Add session summary for 2025-01-11 CODE CTE investigation (TimelordUK)
- docs: Add lexer and parser considerations analysis (TimelordUK)
- docs: Add CODE CTE design document for programmable data transformations (TimelordUK)

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
