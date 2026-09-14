# SQL CLI v1.85.5

**Release Date:** September 14, 2026

## 📊 Release Overview
- **Commits in this release:** 10
- **Files updated:** 12

## ✨ Highlights

### 🎨 Visual Improvements

## 📝 Changes by Category

### 🚀 New Features
- add lrt, an ls -alrt built on sql-cli

### 🐛 Bug Fixes
- honour case-insensitive mode in the value evaluator (R13 slice 3b)
- align ANSI-coloured cells under --table-style

### 🔧 Refactoring
- one ArithmeticEvaluator constructor (R13 slice 3)
- build evaluator registries once and share them (R13 slice 3)

### 📚 Documentation
- record slice 3 and 3b

<details>
<summary>📋 View all commits</summary>

- Merge pull request #86 from TimelordUK/refactor/r13-slice3-one-construction-path (TimelordUK)
- docs(r13): record slice 3 and 3b (TimelordUK)
- fix(r13): honour case-insensitive mode in the value evaluator (R13 slice 3b) (TimelordUK)
- test(r13): pin case-insensitive divergence between evaluators (R13 slice 3b) (TimelordUK)
- refactor(r13): one ArithmeticEvaluator constructor (R13 slice 3) (TimelordUK)
- refactor(r13): build evaluator registries once and share them (R13 slice 3) (TimelordUK)
- Merge pull request #85 from TimelordUK/fix/table-style-ansi-width (TimelordUK)
- feat(scripts): add lrt, an ls -alrt built on sql-cli (TimelordUK)
- fix(output): align ANSI-coloured cells under --table-style (TimelordUK)
- tweak example (stephen james)

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
