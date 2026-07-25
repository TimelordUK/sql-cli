# SQL CLI v1.82.0

**Release Date:** July 25, 2026

## 📊 Release Overview
- **Commits in this release:** 5
- **Files updated:** 13

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

### 💾 Data Protection
- **History Recovery**: Automatic recovery from corrupted files
- **Atomic Writes**: Safer file operations to prevent data loss

## 📝 Changes by Category

### 🐛 Bug Fixes
- close P11 — SELECT alias on the LHS of an IN-subquery
- close P10 (HAVING NOT), P6 (INTERSECT/EXCEPT), P12 (WITH in expr)

<details>
<summary>📋 View all commits</summary>

- Merge pull request #39 from TimelordUK/fix/p11-alias-in-in-subquery (TimelordUK)
- fix(parity): close P11 — SELECT alias on the LHS of an IN-subquery (TimelordUK)
- Merge pull request #38 from TimelordUK/fix/parity-p10-p6-p12-and-test-reliability (TimelordUK)
- test(history): make history_protection_integration reliable cross-platform (TimelordUK)
- fix(parity): close P10 (HAVING NOT), P6 (INTERSECT/EXCEPT), P12 (WITH in expr) (TimelordUK)

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
