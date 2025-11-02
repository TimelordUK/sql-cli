# SQL CLI v1.66.0

**Release Date:** November 02, 2025

## 📊 Release Overview
- **Commits in this release:** 12
- **Files updated:** 42

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🚀 New Features
- Add SELECT * EXCLUDE syntax support (DuckDB compatibility)
- Add complete line geometry analysis toolkit
- Add comprehensive vector mathematics support
- Add ILIKE operator support (PostgreSQL compatibility)

### 🐛 Bug Fixes
- Fix test_examples.py capture for multi-statement JSON output
- Remove REPLACE keyword to avoid conflict with REPLACE() function

### 📚 Documentation
- Update roadmap to reflect completed features
- Add temp table example to README showing multi-stage -q queries

<details>
<summary>📋 View all commits</summary>

- chore: Bump version to 1.66.0 and update CHANGELOG (TimelordUK)
- fix: Fix test_examples.py capture for multi-statement JSON output (TimelordUK)
- test: Add formal test for SELECT * EXCLUDE feature (TimelordUK)
- feat: Add SELECT * EXCLUDE syntax support (DuckDB compatibility) (TimelordUK)
- feat(sql): Add complete line geometry analysis toolkit (TimelordUK)
- feat(sql): Add comprehensive vector mathematics support (TimelordUK)
- ci: Add performance benchmarking to GitHub Actions workflow (TimelordUK)
- fix: Remove REPLACE keyword to avoid conflict with REPLACE() function (TimelordUK)
- chore: Add EXCLUDE and REPLACE tokens to lexer (TimelordUK)
- feat: Add ILIKE operator support (PostgreSQL compatibility) (TimelordUK)
- docs: Update roadmap to reflect completed features (TimelordUK)
- docs: Add temp table example to README showing multi-stage -q queries (TimelordUK)

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
