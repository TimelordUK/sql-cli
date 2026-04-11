# SQL CLI v1.67.1

**Release Date:** April 11, 2026

## 📊 Release Overview
- **Commits in this release:** 23
- **Files updated:** 71

## ✨ Highlights

### 🎨 Visual Improvements
- **Key Press Indicator**: Visual feedback for key presses with fade effects (F12 to toggle)

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🚀 New Features
- Add FILE CTE parser scaffold (parser-only, execution stubbed)
- Migrate query execution to use unified from_source field - PIVOT now works end-to-end!
- Add PivotExpander transformer (Phase 4 - partial)
- Add PIVOT parser implementation
- Add PIVOT AST structures and pattern matching
- Add PIVOT/UNPIVOT lexer tokens and test data
- Implement SQL standard implicit window frames
- Make batch window evaluation the default and fix expression handling
- Optimize window aggregates for UNBOUNDED PRECEDING frames
- Implement batch evaluation for window functions - 86% performance improvement\!
- Window function optimization Phase 2 + Step 0 prep for batch evaluation
- Add Phase 1 window function profiling infrastructure

### 🐛 Bug Fixes
- Set from_source in ExpressionLifter to fix QUALIFY regression
- Update Python tests to use assertions instead of return values

### 🔧 Refactoring
- Improve CTE handling in source table extraction
- Add unified from_source field to SelectStatement and integrate PIVOT expansion

### 📚 Documentation
- Document undocumented-but-working key bindings
- Remove F7/F12 from help — bindings don't fire at runtime
- Fix stale entries in native TUI help page
- Add comprehensive PIVOT examples with best practices

<details>
<summary>📋 View all commits</summary>

- ci: Publish to crates.io in the manual release workflow (TimelordUK)
- docs(tui): Document undocumented-but-working key bindings (TimelordUK)
- ci: Guard uv venv against existing cached .venv (TimelordUK)
- docs(tui): Remove F7/F12 from help — bindings don't fire at runtime (TimelordUK)
- docs(tui): Fix stale entries in native TUI help page (TimelordUK)
- feat: Add FILE CTE parser scaffold (parser-only, execution stubbed) (TimelordUK)
- fix: Set from_source in ExpressionLifter to fix QUALIFY regression (TimelordUK)
- refactor: Improve CTE handling in source table extraction (TimelordUK)
- docs: Add comprehensive PIVOT examples with best practices (TimelordUK)
- feat: Migrate query execution to use unified from_source field - PIVOT now works end-to-end! (TimelordUK)
- refactor: Add unified from_source field to SelectStatement and integrate PIVOT expansion (TimelordUK)
- feat(pivot): Add PivotExpander transformer (Phase 4 - partial) (TimelordUK)
- feat(pivot): Add PIVOT parser implementation (TimelordUK)
- feat(pivot): Add PIVOT AST structures and pattern matching (TimelordUK)
- feat(pivot): Add PIVOT/UNPIVOT lexer tokens and test data (TimelordUK)
- feat: Implement SQL standard implicit window frames (TimelordUK)
- chore: Bump version to v1.67.0 (TimelordUK)
- feat: Make batch window evaluation the default and fix expression handling (TimelordUK)
- fix: Update Python tests to use assertions instead of return values (TimelordUK)
- feat: Optimize window aggregates for UNBOUNDED PRECEDING frames (TimelordUK)
- feat: Implement batch evaluation for window functions - 86% performance improvement\! (TimelordUK)
- feat: Window function optimization Phase 2 + Step 0 prep for batch evaluation (TimelordUK)
- feat: Add Phase 1 window function profiling infrastructure (TimelordUK)

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
