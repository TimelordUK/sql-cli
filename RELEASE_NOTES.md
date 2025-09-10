# SQL CLI v1.43.1

**Release Date:** September 10, 2025

## 📊 Release Overview
- **Commits in this release:** 18
- **Files updated:** 26

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

### 💾 Data Protection
- **History Recovery**: Automatic recovery from corrupted files
- **Atomic Writes**: Safer file operations to prevent data loss

## 📝 Changes by Category

<details>
<summary>📋 View all commits</summary>

- Release v1.43.0: JOIN parser, Neovim plugin improvements, terminal fixes (TimelordUK)
- fix unit tests (TimelordUK)
- add AST for joins for CTEs (TimelordUK)
- fix auto complete (TimelordUK)
- Fix strip_ansi_codes function scope error in Neovim plugin (TimelordUK)
- Fix SQL autocompletion to work with partial text (TimelordUK)
- Fix ORDER BY with aggregate column aliases after GROUP BY (TimelordUK)
- format (TimelordUK)
- Add Ctrl+Space keybinding for SQL autocompletion (TimelordUK)
- Fix ANSI escape sequences in Neovim schema floating window (TimelordUK)
- Add intelligent SQL autocompletion to Neovim plugin (TimelordUK)
- add a neutrons function to chemistry (TimelordUK)
- add K to get function definition and other features for nvim plugin (TimelordUK)
- navigate back and forth a query toggle comment a query and save results to buffer (TimelordUK)
- add switch orientation and execute at cursor and load data file (TimelordUK)
- fix data file for solar system (TimelordUK)
- fix the data file hint so we run in nvim the correct data (TimelordUK)
- fix lua plugin show success in green and highlight submit (TimelordUK)

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
