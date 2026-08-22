# SQL CLI v1.83.0

**Release Date:** August 22, 2026

## 📊 Release Overview
- **Commits in this release:** 9
- **Files updated:** 12

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🐛 Bug Fixes
- evaluate NULL predicates as UNKNOWN, not false (P18/P19)
- key the Python venv cache on the interpreter version

### 🔧 Refactoring
- evaluate WHERE in Trilean, not bool (R10 slice 1b)
- add Trilean, the SQL three-valued truth type (R10 slice 1a)

<details>
<summary>📋 View all commits</summary>

- Merge pull request #54 from TimelordUK/fix/p18-p19-three-valued-logic (TimelordUK)
- fix(engine): evaluate NULL predicates as UNKNOWN, not false (P18/P19) (TimelordUK)
- Merge pull request #53 from TimelordUK/refactor/r10-trilean-wire-evaluator (TimelordUK)
- refactor(engine): evaluate WHERE in Trilean, not bool (R10 slice 1b) (TimelordUK)
- Merge pull request #51 from TimelordUK/refactor/r10-trilean-type (TimelordUK)
- Merge branch 'main' into refactor/r10-trilean-type (TimelordUK)
- Merge pull request #52 from TimelordUK/fix/ci-python-venv-cache (TimelordUK)
- fix(ci): key the Python venv cache on the interpreter version (TimelordUK)
- refactor(engine): add Trilean, the SQL three-valued truth type (R10 slice 1a) (TimelordUK)

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
