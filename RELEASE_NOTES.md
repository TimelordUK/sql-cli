# SQL CLI v1.83.1

**Release Date:** August 29, 2026

## 📊 Release Overview
- **Commits in this release:** 5
- **Files updated:** 15

## ✨ Highlights

### 🎨 Visual Improvements

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

### 🏗️ Architecture Improvements
- **State Management**: Continued migration to centralized AppStateContainer
- **Code Quality**: Transaction-like state updates for better consistency

### 💾 Data Protection
- **History Recovery**: Automatic recovery from corrupted files
- **Atomic Writes**: Safer file operations to prevent data loss

## 📝 Changes by Category

### 🚀 New Features
- add Ctrl+L redraw, fix navigation debug index, add sample data

### 🐛 Bug Fixes
- stop writing to stderr while the alternate screen is active
- translate DataTable indices to visual positions for column widths

<details>
<summary>📋 View all commits</summary>

- Merge pull request #55 from TimelordUK/fix/projection-column-width-index (TimelordUK)
- style: apply cargo fmt (TimelordUK)
- feat(tui): add Ctrl+L redraw, fix navigation debug index, add sample data (TimelordUK)
- fix(tui): stop writing to stderr while the alternate screen is active (TimelordUK)
- fix(viewport): translate DataTable indices to visual positions for column widths (TimelordUK)

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
