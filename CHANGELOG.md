# Changelog

All notable changes to SQL CLI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### 🚀 Features
- **Viewport Lock Mode** - Press Space to anchor scrolling position, data scrolls while cursor stays fixed
- **Dynamic Column Sizing** - Columns automatically adjust width based on visible viewport data
- **Compact Mode** - Press 'C' to reduce padding and fit more columns on screen
- **Auto-Execute for Files** - CSV/JSON files show data immediately on load with pre-filled query
- **Multi-Source Data Proxy** - Query SQL Server, APIs, and files seamlessly through unified interface
- **Visual Source Indicators** - Shows data source with colored icons (📦 Cache, 📁 File, 🌐 API, 🗄️ SQL)
- **Named Cache System** - Save queries with custom IDs like `:cache save trades_2024`
- **Rainbow Parentheses** - Visual matching for nested SQL queries
- **String.IsNullOrEmpty()** - LINQ-style null/empty checking in WHERE clauses
- **Schema-Aware History** - Command history with intelligent suggestions based on query context

### 🐛 Bug Fixes
- Fixed GitHub Actions deprecation warnings by updating to v4
- Fixed cache save to support named IDs
- Fixed formatting issues in CI/CD pipeline

### 📚 Documentation
- Comprehensive README with keyboard shortcuts
- Enhanced F1 help screen with all features
- Added MULTI_SOURCE_PROXY.md documentation
- Added tips section in help for feature discovery

### 🔧 Infrastructure
- Cross-platform CI/CD for Linux, Windows, macOS (x64 and ARM64)
- Automated release workflow with version bumping
- Pre-commit hooks for code formatting
- GitHub Actions permissions properly configured

## [1.0.0] - 2024-01-06

### Initial Release
- Full SQL parser with LINQ support
- Context-aware tab completion
- Professional TUI interface with split-view design
- Vim-like navigation and search
- Command history with search (Ctrl+R)
- CSV/JSON file support
- REST API integration
- Multi-line editor mode (F3)
- Export to CSV (Ctrl+S)
- Column sorting and filtering
- Cache management system

### Supported LINQ Methods
- String.Contains()
- String.StartsWith()
- String.EndsWith()
- String.IsNullOrEmpty()
- Property name normalization

### Platform Support
- Linux x64
- Windows x64
- macOS x64 (Intel)
- macOS ARM64 (Apple Silicon)