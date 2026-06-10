# SQL CLI v1.79.0

**Release Date:** June 10, 2026

## 📊 Release Overview
- **Commits in this release:** 4
- **Files updated:** 4

## ✨ Highlights

## 📝 Changes by Category

### 🐛 Bug Fixes
- make join-key coercion type-aware (opt out by casting to string)
- coerce equi-join keys so string keys match numeric keys
- fall back to unqualified lookup for aliased SELECT columns

<details>
<summary>📋 View all commits</summary>

- Merge pull request #21 from TimelordUK/fix/temp-table-qualified-and-join-coercion (TimelordUK)
- fix(join): make join-key coercion type-aware (opt out by casting to string) (TimelordUK)
- fix(join): coerce equi-join keys so string keys match numeric keys (TimelordUK)
- fix(query): fall back to unqualified lookup for aliased SELECT columns (TimelordUK)

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
