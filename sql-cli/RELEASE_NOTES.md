# SQL CLI v1.11.4

**Release Date:** August 09, 2025

## ✨ What's New

### 🐛 Bug Fixes
- Fix Shift-G navigation regression in results view

## 📝 All Changes

<details>
<summary>Click to expand full commit list</summary>


</details>

## 🎯 Highlights

- **Navigation Fix**: Fixed Shift-G navigation to last row in results view
- **Dynamic Column Sizing**: Columns automatically adjust width based on visible data
- **Compact Mode**: Press 'C' to reduce padding and fit more columns
- **Viewport Lock**: Press Space to anchor scrolling position
- **Auto-Execute**: CSV/JSON files show data immediately on load
- **Visual Source Indicators**: See where your data comes from (📦 📁 🌐 🗄️)

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

# Connect to API
sql-cli --url http://localhost:5000
```
