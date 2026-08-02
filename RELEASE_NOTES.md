# SQL CLI v1.82.2

**Release Date:** August 02, 2026

## 📊 Release Overview
- **Commits in this release:** 12
- **Files updated:** 28

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

### 💾 Data Protection
- **History Recovery**: Automatic recovery from corrupted files
- **Atomic Writes**: Safer file operations to prevent data loss

## 📝 Changes by Category

### 🐛 Bug Fixes
- reject trailing input instead of silently discarding it (P13 stage 1)
- evaluate window functions after WHERE (P21)

### 📚 Documentation
- decide P20 (propagate NULL through ||); promote the rule
- record the P17 decision — follow the reference engine on NULL ordering

<details>
<summary>📋 View all commits</summary>

- Merge pull request #44 from TimelordUK/fix/p13-reject-trailing-input (TimelordUK)
- fix(parser): reject trailing input instead of silently discarding it (P13 stage 1) (TimelordUK)
- Merge pull request #43 from TimelordUK/fix/p21-window-after-where (TimelordUK)
- test(examples): re-capture four expectations that had encoded P21 (TimelordUK)
- fix(engine): evaluate window functions after WHERE (P21) (TimelordUK)
- Merge pull request #42 from TimelordUK/parity/corpus-tiers-08-10 (TimelordUK)
- build(parity): pin DuckDB at 1.5.5; close the discovery phase (TimelordUK)
- test(parity): build out tier 09 (window functions); file P21-P26 (TimelordUK)
- docs(parity): decide P20 (propagate NULL through ||); promote the rule (TimelordUK)
- docs(parity): record the P17 decision — follow the reference engine on NULL ordering (TimelordUK)
- test(parity): build out tier 08 with a NULL fixture; file P16-P20 (TimelordUK)
- test(parity): add corpus tiers 08-10; file P13-P15 (TimelordUK)

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
