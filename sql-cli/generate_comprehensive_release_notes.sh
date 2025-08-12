#!/bin/bash

# Enhanced release notes generator that captures ALL improvements
# Usage: ./generate_comprehensive_release_notes.sh [VERSION]

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)}"
echo "Generating comprehensive release notes for v${VERSION}..."

# Get last tag
LAST_TAG=$(git tag --sort=-version:refname | grep -v "^v$VERSION$" | head -n 1 || echo "")

# Output file
OUTPUT="RELEASE_NOTES_v${VERSION}.md"

# Header
cat > "$OUTPUT" << EOF
# SQL CLI v${VERSION} Release Notes

**Release Date:** $(date +'%B %d, %Y')

EOF

# Get all commits since last tag with full commit messages
if [ -z "$LAST_TAG" ]; then
    COMMITS=$(git log --pretty=format:"%H|%s|%b" || echo "")
    COMMIT_COUNT=$(git rev-list --count HEAD 2>/dev/null || echo "0")
else
    COMMITS=$(git log ${LAST_TAG}..HEAD --pretty=format:"%H|%s|%b" || echo "")
    COMMIT_COUNT=$(git rev-list --count ${LAST_TAG}..HEAD 2>/dev/null || echo "0")
fi

echo "## 📊 Release Statistics" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "- **Commits since last release:** $COMMIT_COUNT" >> "$OUTPUT"
echo "- **Files changed:** $(git diff --stat ${LAST_TAG}..HEAD 2>/dev/null | tail -1 | awk '{print $1}' || echo "N/A")" >> "$OUTPUT"
echo "- **Lines added:** $(git diff --stat ${LAST_TAG}..HEAD 2>/dev/null | tail -1 | awk '{print $4}' || echo "N/A")" >> "$OUTPUT"
echo "- **Lines removed:** $(git diff --stat ${LAST_TAG}..HEAD 2>/dev/null | tail -1 | awk '{print $6}' || echo "N/A")" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Analyze commits for features
echo "## ✨ Major Features & Improvements" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Extract feature-related commits with better parsing
FEATURES_FOUND=false

# Check for visual enhancements
if echo "$COMMITS" | grep -qi "cell.*render\|cell.*highlight\|visual.*enhance\|key.*indicator\|key.*fade"; then
    echo "### 🎨 Visual Enhancements" >> "$OUTPUT"
    echo "- **Cell Highlighting**: Configurable cell selection with multiple render modes (corners, borders, blocks)" >> "$OUTPUT"
    echo "- **Key Press Indicator**: Visual feedback showing recent key presses with fade effect (F12 to toggle)" >> "$OUTPUT"
    echo "- **Improved Cell Rendering**: Better visual distinction between row/cell/column selection modes" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    FEATURES_FOUND=true
fi

# Check for logging improvements
if echo "$COMMITS" | grep -qi "dual.*log\|logging.*system\|debug.*log"; then
    echo "### 🔍 Enhanced Debugging & Logging" >> "$OUTPUT"
    echo "- **Dual Logging System**: Logs to both file and in-memory ring buffer" >> "$OUTPUT"
    echo "- **Cross-platform Log Support**: Works on both Windows and Linux without admin rights" >> "$OUTPUT"
    echo "- **F5 Debug Mode**: Comprehensive state dump showing all internal state" >> "$OUTPUT"
    echo "- **Improved Log Output**: Better formatted logs with timestamps and categories" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    FEATURES_FOUND=true
fi

# Check for state management improvements
if echo "$COMMITS" | grep -qi "state.*container\|state.*migration\|appstate\|selection.*state"; then
    echo "### 🏗️ State Management Refactoring" >> "$OUTPUT"
    echo "- **Centralized State**: Migrated state management to AppStateContainer" >> "$OUTPUT"
    echo "- **Transaction-like Updates**: Multiple state changes now happen atomically in blocks" >> "$OUTPUT"
    echo "- **Better State Synchronization**: Improved coordination between navigation and selection" >> "$OUTPUT"
    echo "- **Reduced Coupling**: TUI layer now purely handles orchestration" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    FEATURES_FOUND=true
fi

# Check for history improvements
if echo "$COMMITS" | grep -qi "history.*protect\|history.*corrupt\|history.*recovery\|atomic.*write"; then
    echo "### 💾 History Protection & Recovery" >> "$OUTPUT"
    echo "- **Automatic Corruption Recovery**: Detects and recovers from corrupted history files" >> "$OUTPUT"
    echo "- **Atomic Writes**: Prevents history corruption with temp file + rename pattern" >> "$OUTPUT"
    echo "- **Backup System**: Automatic backups before significant changes" >> "$OUTPUT"
    echo "- **History Validation**: Validates writes to prevent data loss" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    FEATURES_FOUND=true
fi

# Check for navigation improvements
if echo "$COMMITS" | grep -qi "navigation\|viewport.*lock\|cursor.*lock\|scroll"; then
    echo "### 🧭 Navigation Improvements" >> "$OUTPUT"
    echo "- **Dual Lock Modes**: Viewport lock (Space) and cursor lock (Shift+Space)" >> "$OUTPUT"
    echo "- **Better Scroll Behavior**: Improved viewport scrolling and position tracking" >> "$OUTPUT"
    echo "- **Navigation State**: Centralized navigation tracking with history" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    FEATURES_FOUND=true
fi

# Categorized commits (traditional format)
echo "## 📝 Categorized Changes" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Features
FEAT_COMMITS=$(echo "$COMMITS" | grep -E "^[a-f0-9]+\|feat(\(.*\))?:" | cut -d'|' -f2 | grep -v "bump version" || echo "")
if [ ! -z "$FEAT_COMMITS" ]; then
    echo "### 🚀 Features" >> "$OUTPUT"
    echo "$FEAT_COMMITS" | sed 's/^feat\(.*\): /- /' >> "$OUTPUT"
    echo "" >> "$OUTPUT"
fi

# Bug Fixes
FIX_COMMITS=$(echo "$COMMITS" | grep -E "^[a-f0-9]+\|fix(\(.*\))?:" | cut -d'|' -f2 | grep -v "bump version" || echo "")
if [ ! -z "$FIX_COMMITS" ]; then
    echo "### 🐛 Bug Fixes" >> "$OUTPUT"
    echo "$FIX_COMMITS" | sed 's/^fix\(.*\): /- /' >> "$OUTPUT"
    echo "" >> "$OUTPUT"
fi

# Refactoring
REFACTOR_COMMITS=$(echo "$COMMITS" | grep -E "^[a-f0-9]+\|refactor(\(.*\))?:" | cut -d'|' -f2 | grep -v "bump version" || echo "")
if [ ! -z "$REFACTOR_COMMITS" ]; then
    echo "### 🔧 Refactoring" >> "$OUTPUT"
    echo "$REFACTOR_COMMITS" | sed 's/^refactor\(.*\): /- /' >> "$OUTPUT"
    echo "" >> "$OUTPUT"
fi

# Documentation
DOCS_COMMITS=$(echo "$COMMITS" | grep -E "^[a-f0-9]+\|docs(\(.*\))?:" | cut -d'|' -f2 | grep -v "bump version" || echo "")
if [ ! -z "$DOCS_COMMITS" ]; then
    echo "### 📚 Documentation" >> "$OUTPUT"
    echo "$DOCS_COMMITS" | sed 's/^docs\(.*\): /- /' >> "$OUTPUT"
    echo "" >> "$OUTPUT"
fi

# Technical details section
echo "## 🔬 Technical Details" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "<details>" >> "$OUTPUT"
echo "<summary>Click to see technical improvements</summary>" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# List modified files by category
echo "### Files Modified by Category" >> "$OUTPUT"
echo "" >> "$OUTPUT"

if [ ! -z "$LAST_TAG" ]; then
    # Core files
    CORE_FILES=$(git diff --name-only ${LAST_TAG}..HEAD | grep -E "^src/(main|lib|app_state|enhanced_tui)\.rs$" || echo "")
    if [ ! -z "$CORE_FILES" ]; then
        echo "**Core Components:**" >> "$OUTPUT"
        echo "$CORE_FILES" | sed 's/^/- /' >> "$OUTPUT"
        echo "" >> "$OUTPUT"
    fi
    
    # State management files
    STATE_FILES=$(git diff --name-only ${LAST_TAG}..HEAD | grep -E "state|container" | grep -v target || echo "")
    if [ ! -z "$STATE_FILES" ]; then
        echo "**State Management:**" >> "$OUTPUT"
        echo "$STATE_FILES" | sed 's/^/- /' >> "$OUTPUT"
        echo "" >> "$OUTPUT"
    fi
    
    # UI/Widget files
    UI_FILES=$(git diff --name-only ${LAST_TAG}..HEAD | grep -E "widget|render|ui|tui" | grep -v target || echo "")
    if [ ! -z "$UI_FILES" ]; then
        echo "**UI Components:**" >> "$OUTPUT"
        echo "$UI_FILES" | sed 's/^/- /' >> "$OUTPUT"
        echo "" >> "$OUTPUT"
    fi
fi

echo "</details>" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Key features that are always relevant
echo "## 🎯 Key Features (Always Available)" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "- **Instant Data Preview**: CSV/JSON files load immediately with auto-execute" >> "$OUTPUT"
echo "- **Dynamic Column Sizing**: Intelligent column width adjustment" >> "$OUTPUT"
echo "- **Compact Mode**: Press 'C' to fit more columns on screen" >> "$OUTPUT"
echo "- **Visual Source Indicators**: Clear icons showing data source (📦 📁 🌐 🗄️)" >> "$OUTPUT"
echo "- **Vim-style Navigation**: j/k for rows, h/l for columns, g/G for top/bottom" >> "$OUTPUT"
echo "- **Advanced Search**: Ctrl+F for search, Ctrl+/ for fuzzy filter" >> "$OUTPUT"
echo "- **Column Operations**: Pin columns (P), search columns (Ctrl+Shift+F)" >> "$OUTPUT"
echo "- **Export Options**: Save results as CSV (Ctrl+E) or JSON" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Installation section
echo "## 📦 Installation" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "Download the appropriate binary for your platform from the assets below." >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "### Supported Platforms" >> "$OUTPUT"
echo "- **Linux x64**: \`sql-cli-linux-x64.tar.gz\`" >> "$OUTPUT"
echo "- **Windows x64**: \`sql-cli-windows-x64.zip\`" >> "$OUTPUT"
echo "- **macOS x64** (Intel): \`sql-cli-macos-x64.tar.gz\`" >> "$OUTPUT"
echo "- **macOS ARM64** (Apple Silicon): \`sql-cli-macos-arm64.tar.gz\`" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Quick start
echo "## 🚀 Quick Start" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "\`\`\`bash" >> "$OUTPUT"
echo "# View CSV with instant preview" >> "$OUTPUT"
echo "sql-cli data.csv" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "# Query API endpoint" >> "$OUTPUT"
echo "sql-cli --url http://api.example.com" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "# Enable debug logging" >> "$OUTPUT"
echo "SQL_CLI_DEBUG=1 sql-cli data.csv" >> "$OUTPUT"
echo "\`\`\`" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Testing improvements
echo "## 🧪 Testing & Quality" >> "$OUTPUT"
echo "" >> "$OUTPUT"
if [ -f "test_v27_selection.sh" ] || [ -f "test_state_logging.sh" ]; then
    echo "- Added comprehensive test scripts for state management" >> "$OUTPUT"
fi
echo "- Improved error handling and recovery mechanisms" >> "$OUTPUT"
echo "- Better cross-platform compatibility (Windows/Linux/macOS)" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Contributors section (if multiple)
CONTRIBUTORS=$(git log ${LAST_TAG}..HEAD --pretty=format:"%an" 2>/dev/null | sort -u | wc -l || echo "1")
if [ "$CONTRIBUTORS" -gt "1" ]; then
    echo "## 👥 Contributors" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    git log ${LAST_TAG}..HEAD --pretty=format:"%an" | sort -u | sed 's/^/- /' >> "$OUTPUT"
    echo "" >> "$OUTPUT"
fi

# Footer
echo "---" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "Thank you for using SQL CLI! 🎉" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "For issues or feedback: [GitHub Issues](https://github.com/yourusername/sql-cli/issues)" >> "$OUTPUT"

echo "✅ Comprehensive release notes generated in $OUTPUT"
echo ""
echo "Preview:"
echo "========"
head -30 "$OUTPUT"