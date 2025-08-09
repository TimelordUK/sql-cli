#!/bin/bash

# Script to generate release notes for sql-cli
# Usage: ./generate_release_notes.sh [VERSION]

VERSION="${1:-1.11.4}"
echo "# SQL CLI v${VERSION}" > RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md

# Add release date
echo "**Release Date:** $(date +'%B %d, %Y')" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md

# Get commits since last tag (excluding the current tag)
LAST_TAG=$(git tag --sort=-version:refname | grep -v "^v$VERSION$" | head -n 1 || echo "")

echo "Debug: Current version: v$VERSION"
echo "Debug: Last tag found: $LAST_TAG"

# Generate categorized changelog
echo "## ✨ What's New" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md

# Helper function to safely grep commits
safe_grep_commits() {
    local pattern="$1"
    local commits="$2"
    echo "$commits" | grep -E "$pattern" | grep -v "^chore: bump version" || echo ""
}

# Get all commits since last tag
if [ -z "$LAST_TAG" ]; then
    ALL_COMMITS=$(git log --pretty=format:"%s" || echo "")
else
    # Use HEAD instead of HEAD^ to include all commits
    ALL_COMMITS=$(git log ${LAST_TAG}..HEAD --pretty=format:"%s" || echo "")
fi

# Features
FEAT_COMMITS=$(safe_grep_commits "^feat(\(.*\))?:" "$ALL_COMMITS")
if [ ! -z "$FEAT_COMMITS" ]; then
    echo "### 🚀 Features" >> RELEASE_NOTES.md
    echo "$FEAT_COMMITS" | sed 's/^feat\(.*\): /- /' >> RELEASE_NOTES.md
    echo "" >> RELEASE_NOTES.md
fi

# Fixes
FIX_COMMITS=$(safe_grep_commits "^fix(\(.*\))?:" "$ALL_COMMITS")
if [ ! -z "$FIX_COMMITS" ]; then
    echo "### 🐛 Bug Fixes" >> RELEASE_NOTES.md
    echo "$FIX_COMMITS" | sed 's/^fix\(.*\): /- /' >> RELEASE_NOTES.md
    echo "" >> RELEASE_NOTES.md
fi

# Refactoring
REFACTOR_COMMITS=$(safe_grep_commits "^refactor(\(.*\))?:" "$ALL_COMMITS")
if [ ! -z "$REFACTOR_COMMITS" ]; then
    echo "### 🔧 Refactoring" >> RELEASE_NOTES.md
    echo "$REFACTOR_COMMITS" | sed 's/^refactor\(.*\): /- /' >> RELEASE_NOTES.md
    echo "" >> RELEASE_NOTES.md
fi

# Documentation
DOCS_COMMITS=$(safe_grep_commits "^docs(\(.*\))?:" "$ALL_COMMITS")
if [ ! -z "$DOCS_COMMITS" ]; then
    echo "### 📚 Documentation" >> RELEASE_NOTES.md
    echo "$DOCS_COMMITS" | sed 's/^docs\(.*\): /- /' >> RELEASE_NOTES.md
    echo "" >> RELEASE_NOTES.md
fi

# If no categorized commits found, show uncategorized
if [ -z "$FEAT_COMMITS" ] && [ -z "$FIX_COMMITS" ] && [ -z "$REFACTOR_COMMITS" ] && [ -z "$DOCS_COMMITS" ]; then
    echo "### Recent Changes" >> RELEASE_NOTES.md
    if [ ! -z "$ALL_COMMITS" ]; then
        echo "$ALL_COMMITS" | grep -v "^chore: bump version" | sed 's/^/- /' >> RELEASE_NOTES.md || true
    else
        echo "- Minor updates and improvements" >> RELEASE_NOTES.md
    fi
    echo "" >> RELEASE_NOTES.md
fi

# All commits (detailed)
echo "## 📝 All Changes" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md
echo "<details>" >> RELEASE_NOTES.md
echo "<summary>Click to expand full commit list</summary>" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md

if [ -z "$LAST_TAG" ]; then
    git log --pretty=format:"- %s (%an)" | grep -v "^- chore: bump version" || echo "- Initial release"
else
    git log ${LAST_TAG}..HEAD --pretty=format:"- %s (%an)" | grep -v "^- chore: bump version" || echo "- Minor updates"
fi

echo "" >> RELEASE_NOTES.md
echo "</details>" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md

# Key highlights
echo "## 🎯 Highlights" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md

# Dynamic highlights based on recent changes
if echo "$ALL_COMMITS" | grep -q -i "history"; then
    echo "- **History Protection**: Enhanced protection against accidental history loss" >> RELEASE_NOTES.md
fi
if echo "$ALL_COMMITS" | grep -q -i "widget"; then
    echo "- **Widget Extraction**: Modularized UI components for better maintainability" >> RELEASE_NOTES.md
fi
if echo "$ALL_COMMITS" | grep -q -i "navigation\|shift.*g\|goto"; then
    echo "- **Navigation Fix**: Fixed Shift-G navigation to last row in results view" >> RELEASE_NOTES.md
fi

# Standard highlights
echo "- **Dynamic Column Sizing**: Columns automatically adjust width based on visible data" >> RELEASE_NOTES.md
echo "- **Compact Mode**: Press 'C' to reduce padding and fit more columns" >> RELEASE_NOTES.md
echo "- **Viewport Lock**: Press Space to anchor scrolling position" >> RELEASE_NOTES.md
echo "- **Auto-Execute**: CSV/JSON files show data immediately on load" >> RELEASE_NOTES.md
echo "- **Visual Source Indicators**: See where your data comes from (📦 📁 🌐 🗄️)" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md

echo "## 📦 Installation" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md
echo "Download the appropriate binary for your platform from the assets below." >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md
echo "### Supported Platforms" >> RELEASE_NOTES.md
echo "- **Linux x64**: \`sql-cli-linux-x64.tar.gz\`" >> RELEASE_NOTES.md
echo "- **Windows x64**: \`sql-cli-windows-x64.zip\`" >> RELEASE_NOTES.md
echo "- **macOS x64** (Intel): \`sql-cli-macos-x64.tar.gz\`" >> RELEASE_NOTES.md
echo "- **macOS ARM64** (Apple Silicon): \`sql-cli-macos-arm64.tar.gz\`" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md
echo "### Quick Start" >> RELEASE_NOTES.md
echo "\`\`\`bash" >> RELEASE_NOTES.md
echo "# Load a CSV file with instant preview" >> RELEASE_NOTES.md
echo "sql-cli data/customers.csv" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md
echo "# Connect to API" >> RELEASE_NOTES.md
echo "sql-cli --url http://localhost:5000" >> RELEASE_NOTES.md
echo "\`\`\`" >> RELEASE_NOTES.md

echo ""
echo "Release notes generated successfully in RELEASE_NOTES.md"