# SQL CLI v1.81.0

**Release Date:** July 19, 2026

## 📊 Release Overview
- **Commits in this release:** 20
- **Files updated:** 17

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🐛 Bug Fixes
- rewrite aggregates nested in BETWEEN / IN / CASE (P9)
- resolve sql-cli.exe on Windows in the examples runner

### 🔧 Refactoring
- make the boundary-crossing forms the primitives
- migrate cte_hoister onto walk helpers
- migrate ilike_to_like onto walk::map_children
- migrate into_clause_remover onto walk::map_children
- add exhaustive SqlExpression traversal helpers
- keep from_source and legacy FROM fields in sync

### 📚 Documentation
- refresh R2 status after #35
- record PR number for the R2 crossing entry
- file P9-P12, found while surveying transformers for R2
- add ENGINE_REFACTORING.md, an R-numbered structural debt log

<details>
<summary>📋 View all commits</summary>

- Merge pull request #37 from TimelordUK/fix/p9-having-walk-migration (TimelordUK)
- fix(having): rewrite aggregates nested in BETWEEN / IN / CASE (P9) (TimelordUK)
- Merge pull request #36 from TimelordUK/docs/r2-status (TimelordUK)
- docs(refactoring): refresh R2 status after #35 (TimelordUK)
- Merge pull request #35 from TimelordUK/refactor/walk-crossing (TimelordUK)
- docs(refactoring): record PR number for the R2 crossing entry (TimelordUK)
- refactor(walk): make the boundary-crossing forms the primitives (TimelordUK)
- Merge pull request #33 from TimelordUK/refactor/walk-migration-hybrids (TimelordUK)
- Merge pull request #34 from TimelordUK/docs/parity-p9-p12 (TimelordUK)
- docs(parity): file P9-P12, found while surveying transformers for R2 (TimelordUK)
- refactor(query_plan): migrate cte_hoister onto walk helpers (TimelordUK)
- refactor(query_plan): migrate ilike_to_like onto walk::map_children (TimelordUK)
- refactor(query_plan): migrate into_clause_remover onto walk::map_children (TimelordUK)
- Merge pull request #32 from TimelordUK/docs/engine-refactoring-tracker (TimelordUK)
- Merge pull request #31 from TimelordUK/refactor/expression-walkers (TimelordUK)
- docs: add ENGINE_REFACTORING.md, an R-numbered structural debt log (TimelordUK)
- refactor(parser): add exhaustive SqlExpression traversal helpers (TimelordUK)
- Merge pull request #30 from TimelordUK/chore/p3-step0-from-source-sync (TimelordUK)
- fix(tests): resolve sql-cli.exe on Windows in the examples runner (TimelordUK)
- refactor(query_plan): keep from_source and legacy FROM fields in sync (TimelordUK)

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
