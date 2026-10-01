# SQL CLI v1.85.14

**Release Date:** October 01, 2026

## 📊 Release Overview
- **Commits in this release:** 12
- **Files updated:** 17

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🐛 Bug Fixes
- compute PHI() instead of writing the golden ratio literal
- one rule for a value's truth, DuckDB's cast to boolean (D4, R13 slice 4)
- accept quoted column after alias qualifier (alias."col name")

### 🔧 Refactoring
- WHERE's NOT / AND / OR delegate; its own arms go (R13 slice 4)
- WHERE's CASE delegates to the value evaluator (R13 slice 4)

### 📚 Documentation
- record slice 4 fourth part (AND / OR / NOT / CASE; P61, P62, D4)
- file P58-P60, R14, T19 from the qualified quoted column work

<details>
<summary>📋 View all commits</summary>

- Merge pull request #99 from TimelordUK/refactor/r13-slice4d-and-or-not-case (TimelordUK)
- docs(r13): record slice 4 fourth part (AND / OR / NOT / CASE; P61, P62, D4) (TimelordUK)
- refactor(r13): WHERE's NOT / AND / OR delegate; its own arms go (R13 slice 4) (TimelordUK)
- fix(clippy): compute PHI() instead of writing the golden ratio literal (TimelordUK)
- refactor(r13): WHERE's CASE delegates to the value evaluator (R13 slice 4) (TimelordUK)
- fix(r13): one rule for a value's truth, DuckDB's cast to boolean (D4, R13 slice 4) (TimelordUK)
- test(r13): pin values used as truth values; file P61, P62, D4 (R13 slice 4) (TimelordUK)
- docs: file P58-P60, R14, T19 from the qualified quoted column work (TimelordUK)
- Merge pull request #97 from TimelordUK/fix/qualified-quoted-columns (TimelordUK)
- fix(parser): accept quoted column after alias qualifier (alias."col name") (TimelordUK)
- Merge pull request #96 from TimelordUK/chore/chocolatey-resubmit (TimelordUK)
- chore(choco): address moderator feedback, publish manually (TimelordUK)

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
