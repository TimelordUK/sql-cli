#!/bin/bash

# Enhanced GitHub release notes generator for the workflow
# This script generates comprehensive release notes by analyzing:
# - Commit messages (conventional commits)
# - File changes to detect features
# - Commit body messages for details
# Usage: ./generate_github_release_notes.sh [LAST_TAG]

LAST_TAG="${1:-$(git tag --sort=-version:refname | head -n 1 || echo "")}"
VERSION="${2:-$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)}"

# Helper function to extract detailed features from commits
analyze_commits() {
    local since_ref="$1"
    
    if [ -z "$since_ref" ]; then
        git log --pretty=format:"%H|%s|%b" 2>/dev/null || echo ""
    else
        git log ${since_ref}..HEAD --pretty=format:"%H|%s|%b" 2>/dev/null || echo ""
    fi
}

# Start generating release notes
echo "# SQL CLI v${VERSION}"
echo ""
echo "**Release Date:** $(date +'%B %d, %Y')"
echo ""

# Get commits for analysis
COMMITS=$(analyze_commits "$LAST_TAG")

# Count statistics
if [ -n "$LAST_TAG" ]; then
    COMMIT_COUNT=$(git rev-list --count ${LAST_TAG}..HEAD 2>/dev/null || echo "0")
    FILES_CHANGED=$(git diff --name-only ${LAST_TAG}..HEAD 2>/dev/null | wc -l || echo "0")
else
    COMMIT_COUNT=$(git rev-list --count HEAD 2>/dev/null || echo "0")
    FILES_CHANGED=$(git ls-files | wc -l || echo "0")
fi

echo "## 📊 Release Overview"
echo "- **Commits in this release:** $COMMIT_COUNT"
echo "- **Files updated:** $FILES_CHANGED"
echo ""

# Detect and highlight major features
echo "## ✨ Highlights"
echo ""

# Feature detection based on commit content and file changes
HIGHLIGHTS_ADDED=false

# Visual enhancements
if echo "$COMMITS" | grep -qi "cell.*render\|cell.*highlight\|visual\|key.*indicator\|fade\|theme\|color"; then
    echo "### 🎨 Visual Improvements"
    if echo "$COMMITS" | grep -qi "key.*indicator\|key.*fade"; then
        echo "- **Key Press Indicator**: See your key presses with visual feedback and fade effects (F12 to toggle)"
    fi
    if echo "$COMMITS" | grep -qi "cell.*highlight\|cell.*render"; then
        echo "- **Enhanced Cell Selection**: Multiple rendering modes for cell selection (corners, borders, blocks)"
    fi
    if echo "$COMMITS" | grep -qi "theme\|color"; then
        echo "- **Improved Theming**: Better color contrast and visual clarity"
    fi
    echo ""
    HIGHLIGHTS_ADDED=true
fi

# Debugging improvements
if echo "$COMMITS" | grep -qi "debug\|log\|trace\|diagnostic"; then
    echo "### 🔍 Debugging & Diagnostics"
    if echo "$COMMITS" | grep -qi "dual.*log"; then
        echo "- **Dual Logging System**: Simultaneous file and in-memory logging for better debugging"
    fi
    if echo "$COMMITS" | grep -qi "f5\|debug.*mode\|debug.*dump"; then
        echo "- **Enhanced F5 Debug Mode**: Comprehensive state dumps for troubleshooting"
    fi
    echo "- **Better Error Messages**: More informative error reporting and recovery"
    echo ""
    HIGHLIGHTS_ADDED=true
fi

# State management improvements
if echo "$COMMITS" | grep -qi "state\|container\|refactor.*v[0-9]\|migration"; then
    echo "### 🏗️ Architecture Improvements"
    echo "- **State Management**: Continued migration to centralized AppStateContainer"
    echo "- **Reduced Coupling**: Better separation between UI and business logic"
    echo "- **Transaction-like Updates**: Multiple state changes now happen atomically"
    echo ""
    HIGHLIGHTS_ADDED=true
fi

# Data integrity
if echo "$COMMITS" | grep -qi "history\|corrupt\|recovery\|atomic\|backup"; then
    echo "### 💾 Data Integrity"
    echo "- **History Protection**: Automatic recovery from corrupted history files"
    echo "- **Atomic Operations**: Safe file writes preventing data corruption"
    echo "- **Automatic Backups**: Protection against accidental data loss"
    echo ""
    HIGHLIGHTS_ADDED=true
fi

# Navigation improvements
if echo "$COMMITS" | grep -qi "navigation\|viewport\|scroll\|lock\|cursor"; then
    echo "### 🧭 Navigation Enhancements"
    if echo "$COMMITS" | grep -qi "viewport.*lock\|cursor.*lock"; then
        echo "- **Dual Lock Modes**: Viewport lock (Space) and cursor lock (Shift+Space)"
    fi
    if echo "$COMMITS" | grep -qi "scroll"; then
        echo "- **Improved Scrolling**: Better viewport management and position tracking"
    fi
    echo ""
    HIGHLIGHTS_ADDED=true
fi

# If no specific highlights detected, show general improvements
if [ "$HIGHLIGHTS_ADDED" = "false" ]; then
    echo "- Performance improvements and bug fixes"
    echo "- Code quality enhancements"
    echo "- Better error handling"
    echo ""
fi

# Traditional categorized commits
echo "## 📝 Changes by Category"
echo ""

# Features
FEATURES=$(echo "$COMMITS" | grep -E "^[a-f0-9]+\|feat(\(.*\))?:" | cut -d'|' -f2 | sed 's/^feat[^:]*: //' | grep -v "bump version" || echo "")
if [ -n "$FEATURES" ]; then
    echo "### 🚀 New Features"
    echo "$FEATURES" | while IFS= read -r line; do
        [ -n "$line" ] && echo "- $line"
    done
    echo ""
fi

# Bug Fixes
FIXES=$(echo "$COMMITS" | grep -E "^[a-f0-9]+\|fix(\(.*\))?:" | cut -d'|' -f2 | sed 's/^fix[^:]*: //' | grep -v "bump version" || echo "")
if [ -n "$FIXES" ]; then
    echo "### 🐛 Bug Fixes"
    echo "$FIXES" | while IFS= read -r line; do
        [ -n "$line" ] && echo "- $line"
    done
    echo ""
fi

# Refactoring
REFACTORS=$(echo "$COMMITS" | grep -E "^[a-f0-9]+\|refactor(\(.*\))?:" | cut -d'|' -f2 | sed 's/^refactor[^:]*: //' | grep -v "bump version" || echo "")
if [ -n "$REFACTORS" ]; then
    echo "### 🔧 Refactoring"
    echo "$REFACTORS" | while IFS= read -r line; do
        [ -n "$line" ] && echo "- $line"
    done
    echo ""
fi

# Documentation
DOCS=$(echo "$COMMITS" | grep -E "^[a-f0-9]+\|docs(\(.*\))?:" | cut -d'|' -f2 | sed 's/^docs[^:]*: //' | grep -v "bump version" || echo "")
if [ -n "$DOCS" ]; then
    echo "### 📚 Documentation"
    echo "$DOCS" | while IFS= read -r line; do
        [ -n "$line" ] && echo "- $line"
    done
    echo ""
fi

# Performance
PERF=$(echo "$COMMITS" | grep -E "^[a-f0-9]+\|perf(\(.*\))?:" | cut -d'|' -f2 | sed 's/^perf[^:]*: //' | grep -v "bump version" || echo "")
if [ -n "$PERF" ]; then
    echo "### ⚡ Performance"
    echo "$PERF" | while IFS= read -r line; do
        [ -n "$line" ] && echo "- $line"
    done
    echo ""
fi

# All other commits
OTHER=$(echo "$COMMITS" | grep -vE "^[a-f0-9]+\|(feat|fix|refactor|docs|perf|chore)(\(.*\))?:" | cut -d'|' -f2 | grep -v "bump version" | grep -v "^$" || echo "")
if [ -n "$OTHER" ]; then
    echo "### 📦 Other Changes"
    echo "$OTHER" | while IFS= read -r line; do
        [ -n "$line" ] && echo "- $line"
    done
    echo ""
fi

# Full commit list in collapsible section
echo "<details>"
echo "<summary>📋 Full Commit List</summary>"
echo ""
echo "| Commit | Author | Message |"
echo "|--------|--------|---------|"
if [ -n "$LAST_TAG" ]; then
    git log ${LAST_TAG}..HEAD --pretty=format:"| %h | %an | %s |" | grep -v "bump version"
else
    git log --pretty=format:"| %h | %an | %s |" | head -20
fi
echo ""
echo "</details>"
echo ""

# Installation section
echo "## 📦 Installation"
echo ""
echo "### Download"
echo "Download the appropriate binary for your platform:"
echo "- **Linux x64**: \`sql-cli-linux-x64.tar.gz\`"
echo "- **Windows x64**: \`sql-cli-windows-x64.zip\`"
echo "- **macOS x64** (Intel): \`sql-cli-macos-x64.tar.gz\`"
echo "- **macOS ARM64** (Apple Silicon): \`sql-cli-macos-arm64.tar.gz\`"
echo ""
echo "### Quick Start"
echo "\`\`\`bash"
echo "# View CSV file"
echo "./sql-cli data.csv"
echo ""
echo "# Connect to API"
echo "./sql-cli --url http://localhost:5000"
echo ""
echo "# Enable debug mode"
echo "SQL_CLI_DEBUG=1 ./sql-cli data.csv"
echo "\`\`\`"
echo ""

# What's next section
echo "## 🔮 What's Next"
echo ""
echo "We're actively working on:"
echo "- Completing state management migration to AppStateContainer"
echo "- Implementing DataTable/DataView pattern for better data handling"
echo "- Redux-style state management for guaranteed consistency"
echo "- Enhanced mathematical expression support in SQL"
echo ""

echo "---"
echo ""
echo "**Thank you for using SQL CLI!** 🎉"
echo ""
echo "Found an issue? [Report it on GitHub](https://github.com/TimelordUK/sql-cli/issues)"