# SQL CLI v1.76.0

**Release Date:** May 23, 2026

## 📊 Release Overview
- **Commits in this release:** 7
- **Files updated:** 17

## ✨ Highlights

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

### 💾 Data Protection
- **History Recovery**: Automatic recovery from corrupted files
- **Atomic Writes**: Safer file operations to prevent data loss

## 📝 Changes by Category

### 🚀 New Features
- add DELIMITER clause for non-comma CSV sources
- --delimiter flag + extension auto-detect for sql-cli file.psv
- extension auto-detect and explicit delimiter override
- add CsvReadOptions infrastructure for per-call delimiter

### 📚 Documentation
- add iris_tsv.sql showcasing TSV delimiter surfaces
- capture READ_CSV delimiter arg as next-session item

<details>
<summary>📋 View all commits</summary>

- docs(examples): add iris_tsv.sql showcasing TSV delimiter surfaces (TimelordUK)
- feat(web-cte): add DELIMITER clause for non-comma CSV sources (TimelordUK)
- feat(cli): --delimiter flag + extension auto-detect for sql-cli file.psv (TimelordUK)
- feat(read_csv): extension auto-detect and explicit delimiter override (TimelordUK)
- test(history): fix flaky integration test via #[serial] (TimelordUK)
- feat(csv): add CsvReadOptions infrastructure for per-call delimiter (TimelordUK)
- docs(roadmap): capture READ_CSV delimiter arg as next-session item (TimelordUK)

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
