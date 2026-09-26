# SQL CLI v1.85.11

**Release Date:** September 26, 2026

## 📊 Release Overview
- **Commits in this release:** 8
- **Files updated:** 12

## ✨ Highlights

## 📝 Changes by Category

### 🐛 Bug Fixes
- search functions read NULL as NULL, a number as its text, and follow --case-insensitive (R13 slice 5)

### 🔧 Refactoring
- WHERE's method calls delegate; its method arms and readers go (R13 slice 5)

### 📚 Documentation
- merge an orphaned doc comment left by the deletion

<details>
<summary>📋 View all commits</summary>

- Merge pull request #93 from TimelordUK/refactor/r13-slice5-method-calls (TimelordUK)
- docs(r13): merge an orphaned doc comment left by the deletion (TimelordUK)
- refactor(r13): WHERE's method calls delegate; its method arms and readers go (R13 slice 5) (TimelordUK)
- test(r13): pin methods WHERE rejects on the left of a comparison; file P57 (R13 slice 5) (TimelordUK)
- fix(r13): search functions read NULL as NULL, a number as its text, and follow --case-insensitive (R13 slice 5) (TimelordUK)
- test(r13): pin the search functions' function forms under D3 (R13 slice 5) (TimelordUK)
- test(r13): pin method calls before they move onto the registry; file P56, D3 (R13 slice 5) (TimelordUK)
- add a maths equation csv (stephen james)

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
