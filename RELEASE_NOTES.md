# SQL CLI v1.83.6

**Release Date:** September 05, 2026

## 📊 Release Overview
- **Commits in this release:** 15
- **Files updated:** 48

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🐛 Bug Fixes
- NULL ordering in ORDER BY — P17 + P13 stage 2

### 📚 Documentation
- acceptance criteria for P13 stage 2; re-scope P27; queue P17
- file P40 — generator args are evaluated against DUAL
- file P37/P38, restore skipped examples, triage the smoke suite

<details>
<summary>📋 View all commits</summary>

- Merge pull request #64 from TimelordUK/fix/p17-p13-null-ordering (TimelordUK)
- fix(parity): NULL ordering in ORDER BY — P17 + P13 stage 2 (TimelordUK)
- docs(parity): acceptance criteria for P13 stage 2; re-scope P27; queue P17 (TimelordUK)
- docs(parity): file P40 — generator args are evaluated against DUAL (TimelordUK)
- add new issue relating to parity of divide by 0, null etc (TimelordUK)
- Merge pull request #63 from TimelordUK/docs/p37-p38-and-example-triage (TimelordUK)
- docs(parity): file P37/P38, restore skipped examples, triage the smoke suite (TimelordUK)
- ignore divide by 0 test (TimelordUK)
- fix a example sql (TimelordUK)
- Merge branch 'main' of https://github.com/TimelordUK/sql-cli (TimelordUK)
- fix 3 smoke tests (TimelordUK)
- Merge pull request #62 from TimelordUK/fix/ci-examples-generated-test-data (TimelordUK)
- ci: give the examples job the test data it needs (TimelordUK)
- Merge pull request #61 from TimelordUK/tui_completer_schema (TimelordUK)
- change the enhanced tui so it derives the schema from the data table directly rather than stripping out just names and then guessing. (TimelordUK)

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
