# SQL CLI v1.85.6

**Release Date:** September 16, 2026

## 📊 Release Overview
- **Commits in this release:** 10
- **Files updated:** 4

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🐛 Bug Fixes
- WHERE column operands keep their resolved index; inner evaluator gets aliases (R13 slice 4)

### 🔧 Refactoring
- WHERE's IN / NOT IN delegate to the value evaluator (R13 slice 4)
- WHERE's BETWEEN delegates to the value evaluator (R13 slice 4)

### 📚 Documentation
- record the column-resolution logging follow-up
- record slice 4 first part (BETWEEN / IN delegate, logging cost)

<details>
<summary>📋 View all commits</summary>

- Merge pull request #88 from TimelordUK/perf/where-path-per-row-logging (TimelordUK)
- docs(r13): record the column-resolution logging follow-up (TimelordUK)
- perf: drop per-row logging from column resolution (TimelordUK)
- Merge pull request #87 from TimelordUK/refactor/r13-slice4-where-arms-delegate (TimelordUK)
- docs(r13): record slice 4 first part (BETWEEN / IN delegate, logging cost) (TimelordUK)
- refactor(r13): WHERE's IN / NOT IN delegate to the value evaluator (R13 slice 4) (TimelordUK)
- refactor(r13): WHERE's BETWEEN delegates to the value evaluator (R13 slice 4) (TimelordUK)
- perf(r13): drop per-node logging from the value evaluator (R13 slice 4) (TimelordUK)
- fix(r13): WHERE column operands keep their resolved index; inner evaluator gets aliases (R13 slice 4) (TimelordUK)
- test(r13): pin window-operand guard and alias resolution for BETWEEN / IN (R13 slice 4) (TimelordUK)

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
