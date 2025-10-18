# SQL CLI v1.61.0

**Release Date:** October 18, 2025

## 📊 Release Overview
- **Commits in this release:** 15
- **Files updated:** 49

## ✨ Highlights

### 🎨 Visual Improvements

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🚀 New Features
- Make fuzzy filter interactive with navigation and lock mode
- Add opt-in comment preservation foundation (Phase 1)
- Add dependency-aware column expansion for \sX with temp tables
- Add terminal colored table output with YAML style configuration

### 🐛 Bug Fixes
- Exit insert mode and add persistent ESC handler for locked filters
- Fuzzy filter now filters currently focused table, not first table
- Remove cursor jump when navigating between tables with \sTn
- Improve table navigation UX with nearest table and smooth scrolling
- Fix line number mapping in --get-columns-at for \sE expansion
- Execute target statement to get schema in --get-columns-at

<details>
<summary>📋 View all commits</summary>

- chore: Bump version to 1.61.0 with Neovim plugin UX improvements (TimelordUK)
- fix(nvim): Exit insert mode and add persistent ESC handler for locked filters (TimelordUK)
- feat(nvim): Make fuzzy filter interactive with navigation and lock mode (TimelordUK)
- fix(nvim): Fuzzy filter now filters currently focused table, not first table (TimelordUK)
- fix(nvim): Remove cursor jump when navigating between tables with \sTn (TimelordUK)
- fix(nvim): Improve table navigation UX with nearest table and smooth scrolling (TimelordUK)
- feat: Add opt-in comment preservation foundation (Phase 1) (TimelordUK)
- fix: Fix line number mapping in --get-columns-at for \sE expansion (TimelordUK)
- format (TimelordUK)
- fix: Execute target statement to get schema in --get-columns-at (TimelordUK)
- feat: Add dependency-aware column expansion for \sX with temp tables (TimelordUK)
- correct README (TimelordUK)
- move test files from root (TimelordUK)
- styled example (TimelordUK)
- feat: Add terminal colored table output with YAML style configuration (TimelordUK)

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
