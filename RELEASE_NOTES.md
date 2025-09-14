# SQL CLI v1.46.1

**Release Date:** September 14, 2025

## 📊 Release Overview
- **Commits in this release:** 39
- **Files updated:** 52

## ✨ Highlights

### 🎨 Visual Improvements

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

### 💾 Data Protection
- **History Recovery**: Automatic recovery from corrupted files
- **Atomic Writes**: Safer file operations to prevent data loss

## 📝 Changes by Category

<details>
<summary>📋 View all commits</summary>

- Release v1.46.0: Multi-table navigation and web data integration (TimelordUK)
- Centralize multi-table navigation logic to fix state conflicts (TimelordUK)
- Add comprehensive multi-table navigation system for SQL results (TimelordUK)
- Add environment variable support for WEB CTE headers (TimelordUK)
- Add TrimStart and TrimEnd string methods (TimelordUK)
- Add qualified column name resolution for JOINs (TimelordUK)
- Fix parser to handle qualified column names and multiple WEB CTEs (TimelordUK)
- Add convenient string extraction functions (TimelordUK)
- Add INSTR/IndexOf string functions and improve function registry (TimelordUK)
- Add WEB CTE support for fetching data from HTTP/HTTPS endpoints (TimelordUK)
- Refactor CSV/JSON loading to use stream-based approach (TimelordUK)
- Add missing CLI help options and example writing guide (TimelordUK)
- Add WEB CTE parser support for fetching data from URLs (TimelordUK)
- Improve SQL formatter readability with better line breaking (TimelordUK)
- Add accounting format for negative numbers in parentheses (TimelordUK)
- Fix decimal number splitting in nvim output highlighting (TimelordUK)
- Fix decimal number highlighting in nvim output buffer (TimelordUK)
- Improve nvim plugin output buffer styling (TimelordUK)
- Add demo data and examples for formatting functions (TimelordUK)
- Add flexible RENDER_NUMBER and FORMAT_CURRENCY functions (TimelordUK)
- Fix nvim plugin cursor and readonly buffer issues (TimelordUK)
- Improve browser export for WSL users (TimelordUK)
- Add global export keymaps with descriptions for which-key (TimelordUK)
- Change export keymaps to use \s prefix to avoid conflicts (TimelordUK)
- Add browser export option for Gmail/Teams compatibility (TimelordUK)
- Add multiple export formats for query results (TimelordUK)
- Fix table navigation parser for ASCII table format (TimelordUK)
- Improve table navigation with status display and toggle command (TimelordUK)
- Fix table navigation initialization and add features roadmap (TimelordUK)
- Add Excel-like table navigation for query results (TimelordUK)
- Fix open_data_file to use split instead of replacing buffer (TimelordUK)
- Fix output window not showing when executing queries (TimelordUK)
- Fix state parameter passing in all plugin modules (TimelordUK)
- Fix state module to use proper object-oriented pattern (TimelordUK)
- Fix nil state errors in Neovim plugin autocommands (TimelordUK)
- Refactor Neovim plugin into modular architecture (TimelordUK)
- Add window function formatting support to AST formatter (TimelordUK)
- chemistry examples (TimelordUK)
- Add recursive CTE implementation plan (TimelordUK)

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
