# SQL CLI v1.16.0

**Release Date:** August 11, 2025

## ✨ What's New

### 🚀 Major Milestone Release

This release completes two major state migrations as part of our ongoing refactoring effort:

- **V26: SortState Migration** - Column sorting now managed centrally in AppStateContainer
  - Fixed sort cycling (None → Ascending ↑ → Descending ↓ → None)
  - Resolved infinite recursive loops
  - Fixed mutable static safety issues
  - Improved state synchronization

- **V27: CompletionState Migration** - Tab completion now managed centrally
  - Tab completion for column names and methods
  - Completion statistics tracking
  - Context-aware suggestions
  - Cycle through multiple suggestions with Tab

### 🐛 Bug Fixes
- Complete V26 SortState migration - resolve all remaining issues
- Fixed infinite recursive loop in sort_results_data()
- Fixed mutable static safety issues with fallback_filter_state
- Fixed double RefCell borrow panic
- Fixed event handler return values

### 📚 Technical Improvements
- Centralized state management in AppStateContainer
- Better encapsulation with dedicated state methods
- Improved debug logging for state changes
- Maintains backward compatibility with fallback modes

## 📦 Installation

Download the appropriate binary for your platform from the assets below.

### Supported Platforms
- **Linux x64**: `sql-cli-linux-x64.tar.gz`
- **Windows x64**: `sql-cli-windows-x64.zip`
- **macOS x64** (Intel): `sql-cli-macos-x64.tar.gz`
- **macOS ARM64** (Apple Silicon): `sql-cli-macos-arm64.tar.gz`

### Quick Start
```bash
# Load a CSV file with instant preview
sql-cli data/customers.csv

# Load JSON data
sql-cli trades.json

# Connect to API
sql-cli --url http://localhost:5000
```
