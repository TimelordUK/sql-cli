# SQL CLI v1.85.15

**Release Date:** October 03, 2026

## 📊 Release Overview
- **Commits in this release:** 12
- **Files updated:** 13

## ✨ Highlights

## 📝 Changes by Category

### 🐛 Bug Fixes
- NULL join keys never pair, on the hash and nested-loop paths (P52, R13 slice 6)
- HAVING and IIF read a value's truth with the one rule (D4, R13 slice 6)

### 🔧 Refactoring
- delete the unused single-condition nested-loop builders (R13 slice 6)

### 📚 Documentation
- record slice 6 JOIN (P52 closed); file P63
- record slice 6 HAVING / IIF (P62 finished); reopen R5 as a workstream

<details>
<summary>📋 View all commits</summary>

- Merge pull request #103 from TimelordUK/refactor/r13-slice6c-join-null-keys (TimelordUK)
- docs(r13): record slice 6 JOIN (P52 closed); file P63 (TimelordUK)
- fix(r13): NULL join keys never pair, on the hash and nested-loop paths (P52, R13 slice 6) (TimelordUK)
- Merge pull request #102 from TimelordUK/refactor/r13-slice6b-having-iif (TimelordUK)
- docs(r13): record slice 6 HAVING / IIF (P62 finished); reopen R5 as a workstream (TimelordUK)
- refactor(joins): delete the unused single-condition nested-loop builders (R13 slice 6) (TimelordUK)
- fix(r13): HAVING and IIF read a value's truth with the one rule (D4, R13 slice 6) (TimelordUK)
- Merge pull request #101 from TimelordUK/refactor/r13-slice6-pin (TimelordUK)
- test(r13): pin HAVING / IIF truth values and P52 on every join path (R13 slice 6) (TimelordUK)
- Merge pull request #100 from TimelordUK/chore/choco-icon-jsdelivr (TimelordUK)
- add county query (TimelordUK)
- chore(choco): serve iconUrl from jsdelivr, not raw.githubusercontent (TimelordUK)

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
