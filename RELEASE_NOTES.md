# SQL CLI v1.85.9

**Release Date:** September 19, 2026

## 📊 Release Overview
- **Commits in this release:** 9
- **Files updated:** 10

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🐛 Bug Fixes
- rename LIKE test data out of a .gitignore pattern
- LIKE matcher without exponential backtracking (R13 slice 4)

### 🔧 Refactoring
- remove the LIKE regex cache and its plumbing (R13 slice 4)
- WHERE's LIKE delegates; one `sql_like` for both evaluators (R13 slice 4)
- WHERE's IS NULL / IS NOT NULL delegate to the value evaluator (R13 slice 4)

### 📚 Documentation
- record slice 4 third part (IS NULL, LIKE; P55, D2)

<details>
<summary>📋 View all commits</summary>

- Merge pull request #91 from TimelordUK/refactor/r13-slice4c-like-is-null (TimelordUK)
- fix(parity): rename LIKE test data out of a .gitignore pattern (TimelordUK)
- docs(r13): record slice 4 third part (IS NULL, LIKE; P55, D2) (TimelordUK)
- refactor(r13): remove the LIKE regex cache and its plumbing (R13 slice 4) (TimelordUK)
- refactor(r13): WHERE's LIKE delegates; one `sql_like` for both evaluators (R13 slice 4) (TimelordUK)
- test(r13): pin LIKE over a number as the decided behaviour; record D2 (R13 slice 4) (TimelordUK)
- fix(r13): LIKE matcher without exponential backtracking (R13 slice 4) (TimelordUK)
- test(r13): pin LIKE before it delegates; file P55 (R13 slice 4) (TimelordUK)
- refactor(r13): WHERE's IS NULL / IS NOT NULL delegate to the value evaluator (R13 slice 4) (TimelordUK)

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
