# SQL CLI v1.56.0

**Release Date:** October 02, 2025

## 📊 Release Overview
- **Commits in this release:** 63
- **Files updated:** 173

## ✨ Highlights

### 🎨 Visual Improvements

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

### 💾 Data Protection
- **History Recovery**: Automatic recovery from corrupted files
- **Atomic Writes**: Safer file operations to prevent data loss

## 📝 Changes by Category

### 🚀 New Features
- Add smart column expansion and distinct values analysis
- Add auto_hide_empty option to table_output config
- Add configurable column width and sampling for table output
- Add yank column as JSON array for WEB CTE workflow
- Add JSON pretty-printing for WEB CTE BODY clause
- Add format.sql example for testing JSON pretty-printing in WEB CTEs
- Add column sum/statistics calculator
- Add Telescope-style fuzzy filter for SQL results
- Add multi-token JWT management system
- Add statusline integration for token refresh notifications
- Add multi-token manager with auto-refresh support
- Add multi-token management system with refresh commands
- Add bitwise string manipulation functions
- Integrate Redis cache config into existing config system
- Add Redis cache management tools and nvim notifications
- Add opt-in Redis caching for Web CTEs
- Add execution and action hints to query history window titles
- Add preview pane scrolling for long queries in history UI
- Add persistence and import/export for query history
- Add direct execution from query history (x key)

### 🐛 Bug Fixes
- Remove SQL comments from CTE test queries
- Add WEB CTE support to query generation phase
- Rewrite CTE cursor detection logic to fix off-by-one bug
- Preserve $JSON$ delimiters in FORM_FIELD values
- Trim whitespace in column JSON array yank
- Add table navigation diagnostics and improve boundary handling
- Fix null exceptions and add layout reset recovery
- Preserve window function frames and add $JSON$ delimiter support
- Fix token corruption from multi-line script output
- Trim whitespace from tokens in multi_token_manager
- Use leader-based keybindings for fuzzy filter
- Fix parameter input handling for dynamic queries
- Improve parameter resolution for dynamic queries
- Make Web CTE caching independent of query context
- Fix Redis cache key collision for same-named Web CTEs
- Fix table navigation issues with query result boundaries
- Handle PowerShell execution policy restrictions
- Fix PowerShell token command path expansion
- Clean up Rust build warnings
- Prevent duplicate token refresh on startup
- Ensure cache messages always appear at top of output
- Simplify window switching in query history UI
- Prevent query history windows from closing when switching focus with Space
- Improve delete functionality in query history UI
- Separate interactive params {{}} from environment vars ${}
- Trim leading/trailing whitespace from query history entries
- Use execute_query instead of non-existent execute_buffer

### 🔧 Refactoring
- Clean up unused field warnings by prefixing with underscore

### 📚 Documentation
- Add development roadmap for next week's features
- Add Web CTE caching system design for next session

<details>
<summary>📋 View all commits</summary>

- chore: Bump version to 1.56.0 and update CHANGELOG (TimelordUK)
- feat(nvim): Add smart column expansion and distinct values analysis (TimelordUK)
- refactor: Clean up unused field warnings by prefixing with underscore (TimelordUK)
- new doc for HAVING_CTE_HOISTING (TimelordUK)
- feat(nvim): Add auto_hide_empty option to table_output config (TimelordUK)
- fix(nvim): Remove SQL comments from CTE test queries (TimelordUK)
- fix(nvim): Add WEB CTE support to query generation phase (TimelordUK)
- debug(nvim): Add verbose CTE search logging to diagnose WEB CTE detection (TimelordUK)
- fix(nvim): Rewrite CTE cursor detection logic to fix off-by-one bug (TimelordUK)
- debug(nvim): Add verbose cursor position debugging for CTE detection (TimelordUK)
- debug(nvim): Add SQL comment debug output to CTE tester (TimelordUK)
- feat: Add configurable column width and sampling for table output (TimelordUK)
- debug for cte tester (TimelordUK)
- fix(formatter): Preserve $JSON$ delimiters in FORM_FIELD values (TimelordUK)
- fix(nvim): Trim whitespace in column JSON array yank (TimelordUK)
- fix(nvim): Add table navigation diagnostics and improve boundary handling (TimelordUK)
- feat(nvim): Add yank column as JSON array for WEB CTE workflow (TimelordUK)
- fix(nvim): Fix null exceptions and add layout reset recovery (TimelordUK)
- fix(formatter): Preserve window function frames and add $JSON$ delimiter support (TimelordUK)
- fix(nvim): Fix token corruption from multi-line script output (TimelordUK)
- fix(nvim): Trim whitespace from tokens in multi_token_manager (TimelordUK)
- feat: Add JSON pretty-printing for WEB CTE BODY clause (TimelordUK)
- feat: Add format.sql example for testing JSON pretty-printing in WEB CTEs (TimelordUK)
- formatting (TimelordUK)
- feat(nvim): Add column sum/statistics calculator (TimelordUK)
- fix(nvim): Use leader-based keybindings for fuzzy filter (TimelordUK)
- feat(nvim): Add Telescope-style fuzzy filter for SQL results (TimelordUK)
- fix(nvim): Fix parameter input handling for dynamic queries (TimelordUK)
- fix(nvim): Improve parameter resolution for dynamic queries (TimelordUK)
- fix: Make Web CTE caching independent of query context (TimelordUK)
- fix: Fix Redis cache key collision for same-named Web CTEs (TimelordUK)
- fix(nvim): Fix table navigation issues with query result boundaries (TimelordUK)
- docs: Add development roadmap for next week's features (TimelordUK)
- add windows and linux working multi token config (TimelordUK)
- fix(nvim): Handle PowerShell execution policy restrictions (TimelordUK)
- fix(nvim): Fix PowerShell token command path expansion (TimelordUK)
- fix: Clean up Rust build warnings (TimelordUK)
- feat(nvim): Add multi-token JWT management system (TimelordUK)
- fix(nvim): Prevent duplicate token refresh on startup (TimelordUK)
- feat(nvim): Add statusline integration for token refresh notifications (TimelordUK)
- feat(nvim): Add multi-token manager with auto-refresh support (TimelordUK)
- feat: Add multi-token management system with refresh commands (TimelordUK)
- feat: Add bitwise string manipulation functions (TimelordUK)
- fix(nvim): Ensure cache messages always appear at top of output (TimelordUK)
- feat: Integrate Redis cache config into existing config system (TimelordUK)
- feat: Add Redis cache management tools and nvim notifications (TimelordUK)
- feat: Add opt-in Redis caching for Web CTEs (TimelordUK)
- add ascii art generator (TimelordUK)
- add a xml parser allowing cds data to be extracted into a flattened csv/json (TimelordUK)
- add swagger support (TimelordUK)
- move files from root (TimelordUK)
- feat: Add execution and action hints to query history window titles (TimelordUK)
- fix: Simplify window switching in query history UI (TimelordUK)
- fix: Prevent query history windows from closing when switching focus with Space (TimelordUK)
- feat: Add preview pane scrolling for long queries in history UI (TimelordUK)
- fix: Improve delete functionality in query history UI (TimelordUK)
- fix: Separate interactive params {{}} from environment vars ${} (TimelordUK)
- demo config lua for pluygin (TimelordUK)
- docs: Add Web CTE caching system design for next session (TimelordUK)
- feat: Add persistence and import/export for query history (TimelordUK)
- fix: Trim leading/trailing whitespace from query history entries (TimelordUK)
- fix: Use execute_query instead of non-existent execute_buffer (TimelordUK)
- feat: Add direct execution from query history (x key) (TimelordUK)

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
