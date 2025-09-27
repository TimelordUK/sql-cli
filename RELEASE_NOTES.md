# SQL CLI v1.55.0

**Release Date:** September 27, 2025

## 📊 Release Overview
- **Commits in this release:** 41
- **Files updated:** 86

## ✨ Highlights

### 🎨 Visual Improvements

### 🔍 Enhanced Debugging
- **Better Diagnostics**: Improved error messages and state dumps

## 📝 Changes by Category

### 🚀 New Features
- Add enhanced query history UI with split-pane preview
- Update Flask test server to accept SQL-CLI Web CTE format
- Add query history system with telescope-style recall
- Release v1.55.0 - Windows Nvim export support & performance docs
- Improve export notifications and clean output
- Add 100K rows to benchmark suite
- Add JSON selector proxy server with FORM_FILE upload and JSON prettification
- Add comprehensive bitwise operations and binary visualization functions
- Complete rewrite of Neovim template system with major improvements

### 🐛 Bug Fixes
- Correct macro format in web_query template
- Use macro for API URL to avoid colon parsing issue
- Update web_query template for Flask test server compatibility
- Export clean TSV/CSV data directly from sql-cli
- Use Parser:: instead of Self:: in free functions
- Resolve circular reference and nil handling in template system

### 🔧 Refactoring
- Remove dead get_binary_op and get_arithmetic_op methods
- Extract is_valid_identifier helper to eliminate identifier validation duplication
- Remove dead parse_select_list method
- Extract update_paren_depth helper to eliminate parenthesis tracking duplication
- Remove more dead code from incomplete tokenization work
- Remove unused is_reserved_keyword method and clarify string-based checking
- Extract parse_argument_list helper to eliminate argument parsing duplication
- Extract find_quote_start helper to eliminate quote parsing duplication
- Extract handle_method_call_context helper to eliminate more duplication
- Extract check_after_comparison_operator helper to eliminate large duplication
- Extract check_balanced_parentheses helper to eliminate final duplication
- Extract parse_optional_alias helper to eliminate more code duplication
- Extract common CTE parsing logic to eliminate code duplication
- Extract WEB CTE parsing into dedicated module
- Remove string literals from recursive parser
- Optimize parser keyword handling - reduce string allocations

### 📚 Documentation
- Add comprehensive performance documentation
- Design structured data selector for hierarchical object querying
- Add FIX engine integration design document
- Document ROW_NUMBER() window function bug with CTEs

<details>
<summary>📋 View all commits</summary>

- demo server to publish trades from a template cte in nvim (TimelordUK)
- feat: Add enhanced query history UI with split-pane preview (TimelordUK)
- fix: Correct macro format in web_query template (TimelordUK)
- fix: Use macro for API URL to avoid colon parsing issue (TimelordUK)
- fix: Update web_query template for Flask test server compatibility (TimelordUK)
- feat: Update Flask test server to accept SQL-CLI Web CTE format (TimelordUK)
- feat: Add query history system with telescope-style recall (TimelordUK)
- feat: Release v1.55.0 - Windows Nvim export support & performance docs (TimelordUK)
- windows export browser (TimelordUK)
- windows clip board (TimelordUK)
- open browser from windows in nvim (TimelordUK)
- feat(nvim): Improve export notifications and clean output (TimelordUK)
- fix(nvim): Export clean TSV/CSV data directly from sql-cli (TimelordUK)
- docs: Add comprehensive performance documentation (TimelordUK)
- feat: Add 100K rows to benchmark suite (TimelordUK)
- refactor: Remove dead get_binary_op and get_arithmetic_op methods (TimelordUK)
- fix: Use Parser:: instead of Self:: in free functions (TimelordUK)
- refactor: Extract is_valid_identifier helper to eliminate identifier validation duplication (TimelordUK)
- refactor: Remove dead parse_select_list method (TimelordUK)
- refactor: Extract update_paren_depth helper to eliminate parenthesis tracking duplication (TimelordUK)
- refactor: Remove more dead code from incomplete tokenization work (TimelordUK)
- refactor: Remove unused is_reserved_keyword method and clarify string-based checking (TimelordUK)
- refactor: Extract parse_argument_list helper to eliminate argument parsing duplication (TimelordUK)
- refactor: Extract find_quote_start helper to eliminate quote parsing duplication (TimelordUK)
- refactor: Extract handle_method_call_context helper to eliminate more duplication (TimelordUK)
- refactor: Extract check_after_comparison_operator helper to eliminate large duplication (TimelordUK)
- refactor: Extract check_balanced_parentheses helper to eliminate final duplication (TimelordUK)
- refactor: Extract parse_optional_alias helper to eliminate more code duplication (TimelordUK)
- refactor: Extract common CTE parsing logic to eliminate code duplication (TimelordUK)
- refactor: Extract WEB CTE parsing into dedicated module (TimelordUK)
- chore: Update .gitignore for .NET build artifacts (TimelordUK)
- feat: Add JSON selector proxy server with FORM_FILE upload and JSON prettification (TimelordUK)
- docs: Design structured data selector for hierarchical object querying (TimelordUK)
- docs: Add FIX engine integration design document (TimelordUK)
- feat: Add comprehensive bitwise operations and binary visualization functions (TimelordUK)
- docs: Document ROW_NUMBER() window function bug with CTEs (TimelordUK)
- refactor: Remove string literals from recursive parser (TimelordUK)
- Add test_trades_multi.csv data file (TimelordUK)
- fix: Resolve circular reference and nil handling in template system (TimelordUK)
- refactor: Optimize parser keyword handling - reduce string allocations (TimelordUK)
- feat: Complete rewrite of Neovim template system with major improvements (TimelordUK)

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
