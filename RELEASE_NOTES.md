# SQL CLI v1.64.0

**Release Date:** November 01, 2025

## 📊 Release Overview
- **Commits in this release:** 37
- **Files updated:** 113

## ✨ Highlights

### 🎨 Visual Improvements

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

### 🏗️ Architecture Improvements
- **State Management**: Continued migration to centralized AppStateContainer
- **Code Quality**: Transaction-like state updates for better consistency

## 📝 Changes by Category

### 🚀 New Features
- Add \st keymap for SQL transformation pipeline debug
- Add QUALIFY clause support for window function filtering
- Add ORDER BY expression support with aggregate transformer
- Add transformer debugging with --show-transformations flag
- Complete Phase 3 of Execution Mode Unification
- Complete Phase 2 of execution mode unification
- Complete Phase 1 of execution mode unification
- Add unified execution module foundation (Phase 0)
- Improve test output to clearly show JSON validation
- Add data file hint support to examples test framework
- Add examples test suite to CI pipeline
- Add correlated subquery analyzer (Phase 1 - Detection)
- Enable preprocessing pipeline in script execution (Quick Win #4)
- Add GROUP BY clause alias expansion (Quick Win #3)
- Add WHERE clause alias expansion (Quick Win #2)
- Add HAVING auto-aliasing transformer (Quick Win #1)
- Add AST preprocessing pipeline infrastructure (Phase 0)

### 🐛 Bug Fixes
- Restore temp table persistence in --execute-statement mode
- Change transformations keymap from \st to \sz
- Add actual UNION ALL subquery example
- Enable transformers in dependency-aware execution mode
- Resolve qualified names regression and add transformer debugging
- Make preprocessing pipeline run by default

### 🔧 Refactoring
- Use Python examples test runner in run_all_tests.sh

### 📚 Documentation
- Update roadmap with ORDER BY completion
- Add UNION ALL subquery examples
- Add ORDER BY expressions example file
- Update CLAUDE.md with examples test framework commands

<details>
<summary>📋 View all commits</summary>

- chore: Bump version to 1.64.0 and update CHANGELOG (TimelordUK)
- fix: Restore temp table persistence in --execute-statement mode (TimelordUK)
- fix(nvim): Change transformations keymap from \st to \sz (TimelordUK)
- feat(nvim): Add \st keymap for SQL transformation pipeline debug (TimelordUK)
- feat: Add QUALIFY clause support for window function filtering (TimelordUK)
- docs: Update roadmap with ORDER BY completion (TimelordUK)
- fix: Add actual UNION ALL subquery example (TimelordUK)
- docs: Add UNION ALL subquery examples (TimelordUK)
- test: Add formal test for ORDER BY expressions example (TimelordUK)
- fix: Enable transformers in dependency-aware execution mode (TimelordUK)
- docs: Add ORDER BY expressions example file (TimelordUK)
- feat: Add ORDER BY expression support with aggregate transformer (TimelordUK)
- feat: Add transformer debugging with --show-transformations flag (TimelordUK)
- feat: Complete Phase 3 of Execution Mode Unification (TimelordUK)
- feat: Complete Phase 2 of execution mode unification (TimelordUK)
- feat: Complete Phase 1 of execution mode unification (TimelordUK)
- more formal test captures (TimelordUK)
- feat: Add unified execution module foundation (Phase 0) (TimelordUK)
- examples and how to generate time series data (TimelordUK)
- expectation tests (TimelordUK)
- more expectations for examples (TimelordUK)
- feat: Improve test output to clearly show JSON validation (TimelordUK)
- feat: Add data file hint support to examples test framework (TimelordUK)
- docs: Update CLAUDE.md with examples test framework commands (TimelordUK)
- refactor: Use Python examples test runner in run_all_tests.sh (TimelordUK)
- feat: Add examples test suite to CI pipeline (TimelordUK)
- fix: Resolve qualified names regression and add transformer debugging (TimelordUK)
- feat: Add correlated subquery analyzer (Phase 1 - Detection) (TimelordUK)
- feat: Enable preprocessing pipeline in script execution (Quick Win #4) (TimelordUK)
- test: Ignore test_orchestrator_preserves_original requiring history file (TimelordUK)
- feat: Add GROUP BY clause alias expansion (Quick Win #3) (TimelordUK)
- feat: Add WHERE clause alias expansion (Quick Win #2) (TimelordUK)
- sql demo using bit string math add to readme (TimelordUK)
- format (TimelordUK)
- fix: Make preprocessing pipeline run by default (TimelordUK)
- feat: Add HAVING auto-aliasing transformer (Quick Win #1) (TimelordUK)
- feat: Add AST preprocessing pipeline infrastructure (Phase 0) (TimelordUK)

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
