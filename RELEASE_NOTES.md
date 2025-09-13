# SQL CLI v1.45.0

**Release Date:** September 13, 2025

## 📊 Release Overview
- **Commits in this release:** 34
- **Files updated:** 85

## ✨ Highlights

### 🎨 Visual Improvements

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

<details>
<summary>📋 View all commits</summary>

- Release v1.45.0: Enhanced CTE support and comprehensive execution plans (TimelordUK)
- Fix AST formatter to properly handle method call expressions (TimelordUK)
- Fix CTE context propagation to subqueries for proper name resolution (TimelordUK)
- Add AST-based SQL formatter with full nvim integration (TimelordUK)
- Add GROUP BY expression support (TimelordUK)
- Add time-based aggregation functions for financial data (TimelordUK)
- Add comprehensive subquery support (scalar, IN, NOT IN) (TimelordUK)
- Implement subquery parsing infrastructure for SQL CLI (TimelordUK)
- Merge implement_execution_plan into main (TimelordUK)
- Release v1.44.0: SQL Parser Modularization & Enhanced Nvim Plugin (TimelordUK)
- Add --execution-plan flag for query debugging (TimelordUK)
- Extract recursive_parser tests to separate module (Phase 2.7) (TimelordUK)
- add to trade-rec (TimelordUK)
- Extract formatting functions into separate module (Phase 2.6) (TimelordUK)
- Update refactoring plan - Phase 2 complete! (TimelordUK)
- Extract CASE expression parsing (Phase 2.5) (TimelordUK)
- Temporarily skip failing SQL examples (TimelordUK)
- Fix integration test for new WHERE clause structure (TimelordUK)
- Update CASE WHEN test to reflect newly supported AND/OR operators (TimelordUK)
- Update refactoring plan - Phase 2.4 complete (TimelordUK)
- Extract logical expression parsing (Phase 2.4) (TimelordUK)
- Extract comparison expression parsing (Phase 2.3) (TimelordUK)
- Extract arithmetic expression parsing (Phase 2.2) (TimelordUK)
- Extract primary expression parsing to modular structure (TimelordUK)
- Fix parser refactoring Phase 1 issues (TimelordUK)
- Refactor SQL parser into modular structure (Phase 1) (TimelordUK)
- Create unified type system with centralized comparison logic (TimelordUK)
- Fix date parsing and comparison issues in WHERE clauses (TimelordUK)
- add is_date, type based functions and add features to nvim plugin (TimelordUK)
- add frequency function (TimelordUK)
- reformat (TimelordUK)
- add expand * to all columns in nvim plugin. (TimelordUK)
- Fix DateDiff issue with datetime columns in TUI mode (TimelordUK)
- add new functions (TimelordUK)

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
