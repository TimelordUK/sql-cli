# SQL CLI v1.62.0

**Release Date:** October 22, 2025

## 📊 Release Overview
- **Commits in this release:** 24
- **Files updated:** 66

## ✨ Highlights

### 🎨 Visual Improvements

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🚀 New Features
- v1.62.0 - JOIN expression support & scoped star expansion
- Add test CSV files for scoped expansion examples
- Implement scoped table star expansion (SELECT table.*)
- Add vibrant gradient ASCII art showcase to README
- Complete comment preservation for SQL formatter with CTE support
- Add Flask + Kerberos Docker example with auto-renewal
- Add parser support for qualified star (table.*) syntax (Phase 1)
- Add helpful error for aggregate functions in HAVING clause
- Improve error messages for reserved keyword collisions
- Add opt-in ParserMode for comment preservation (Phase 2)
- Add ANSI terminal color and formatting functions
- Add file-level variable substitution with @SET syntax

### 🐛 Bug Fixes
- Strip cache and diagnostic messages from exports
- Export only the focused table in multi-table results
- Apply @SET variable substitution in \sE (expand SELECT *)

### 📚 Documentation
- Deep analysis of HAVING clause support and limitations
- Add comprehensive CROSS JOIN examples and documentation
- Add glorious rainbow color showcase to README 🌈

<details>
<summary>📋 View all commits</summary>

- feat: v1.62.0 - JOIN expression support & scoped star expansion (TimelordUK)
- format (TimelordUK)
- test: Add automated tests for scoped table star expansion (TimelordUK)
- feat(data): Add test CSV files for scoped expansion examples (TimelordUK)
- feat(sql): Implement scoped table star expansion (SELECT table.*) (TimelordUK)
- feat(docs): Add vibrant gradient ASCII art showcase to README (TimelordUK)
- feat: Complete comment preservation for SQL formatter with CTE support (TimelordUK)
- feat: Add Flask + Kerberos Docker example with auto-renewal (TimelordUK)
- test: Update Python test to use HAVING alias pattern (TimelordUK)
- test: Update integration tests to use HAVING alias pattern (TimelordUK)
- fix(nvim): Strip cache and diagnostic messages from exports (TimelordUK)
- test: Fix HAVING clause test to use alias pattern (TimelordUK)
- feat: Add parser support for qualified star (table.*) syntax (Phase 1) (TimelordUK)
- fix(nvim): Export only the focused table in multi-table results (TimelordUK)
- fix(nvim): Apply @SET variable substitution in \sE (expand SELECT *) (TimelordUK)
- feat: Add helpful error for aggregate functions in HAVING clause (TimelordUK)
- docs: Deep analysis of HAVING clause support and limitations (TimelordUK)
- feat: Improve error messages for reserved keyword collisions (TimelordUK)
- docs: Add comprehensive CROSS JOIN examples and documentation (TimelordUK)
- feat: Add opt-in ParserMode for comment preservation (Phase 2) (TimelordUK)
- add more examples (TimelordUK)
- docs: Add glorious rainbow color showcase to README 🌈 (TimelordUK)
- feat(sql): Add ANSI terminal color and formatting functions (TimelordUK)
- feat(nvim): Add file-level variable substitution with @SET syntax (TimelordUK)

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
