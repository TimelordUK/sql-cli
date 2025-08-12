# How to Update the Release Action

## Location
The file to update is in the root repository:
`.github/workflows/manual-release.yml` (or `release.yml`)

## What to Replace

### Find this section (around line 56-98):
```yaml
      - name: Generate release notes
        id: notes
        run: |
          VERSION="${{ steps.version.outputs.new_version }}"
          LAST_TAG=$(git tag --sort=-version:refname | head -n 1 || echo "")
          
          {
            echo "# SQL CLI v${VERSION}"
            # ... basic release notes generation ...
          } > RELEASE_NOTES.md
```

### Replace with this enhanced version:
```yaml
      - name: Generate comprehensive release notes
        id: notes
        run: |
          VERSION="${{ steps.version.outputs.new_version }}"
          LAST_TAG=$(git tag --sort=-version:refname | head -n 1 || echo "")
          
          # Get commit statistics
          if [ -n "$LAST_TAG" ]; then
            COMMIT_COUNT=$(git rev-list --count ${LAST_TAG}..HEAD 2>/dev/null || echo "0")
            FILES_CHANGED=$(git diff --name-only ${LAST_TAG}..HEAD 2>/dev/null | wc -l || echo "0")
            COMMITS=$(git log ${LAST_TAG}..HEAD --pretty=format:"%H|%s|%b" 2>/dev/null || echo "")
          else
            COMMIT_COUNT=$(git rev-list --count HEAD 2>/dev/null || echo "0")
            FILES_CHANGED=$(git ls-files | wc -l || echo "0")
            COMMITS=$(git log --pretty=format:"%H|%s|%b" 2>/dev/null || echo "")
          fi
          
          {
            echo "# SQL CLI v${VERSION}"
            echo ""
            echo "**Release Date:** $(date +'%B %d, %Y')"
            echo ""
            
            # Add custom notes if provided
            if [ -n "${{ github.event.inputs.release_notes }}" ]; then
              echo "## 📢 Release Notes"
              echo "${{ github.event.inputs.release_notes }}"
              echo ""
            fi
            
            # Add statistics
            echo "## 📊 Release Overview"
            echo "- **Commits in this release:** $COMMIT_COUNT"
            echo "- **Files updated:** $FILES_CHANGED"
            echo ""
            
            # Detect and highlight features
            echo "## ✨ Highlights"
            echo ""
            
            # Check for visual enhancements
            if echo "$COMMITS" | grep -qi "cell.*render\|visual\|key.*indicator\|fade\|theme"; then
              echo "### 🎨 Visual Improvements"
              if echo "$COMMITS" | grep -qi "key.*indicator"; then
                echo "- **Key Press Indicator**: Visual feedback for key presses with fade effects (F12 to toggle)"
              fi
              if echo "$COMMITS" | grep -qi "cell.*highlight"; then
                echo "- **Enhanced Cell Selection**: Multiple rendering modes for better visual feedback"
              fi
              echo ""
            fi
            
            # Check for debugging improvements
            if echo "$COMMITS" | grep -qi "debug\|log\|diagnostic"; then
              echo "### 🔍 Enhanced Debugging"
              if echo "$COMMITS" | grep -qi "dual.*log"; then
                echo "- **Dual Logging**: Simultaneous file and in-memory logging"
              fi
              echo "- **Better Diagnostics**: Improved error messages and state dumps"
              echo ""
            fi
            
            # Check for state management
            if echo "$COMMITS" | grep -qi "state.*container\|refactor.*v[0-9]"; then
              echo "### 🏗️ Architecture Improvements"
              echo "- **State Management**: Continued migration to centralized AppStateContainer"
              echo "- **Code Quality**: Transaction-like state updates for better consistency"
              echo ""
            fi
            
            # Check for data integrity
            if echo "$COMMITS" | grep -qi "history.*protect\|corrupt\|atomic"; then
              echo "### 💾 Data Protection"
              echo "- **History Recovery**: Automatic recovery from corrupted files"
              echo "- **Atomic Writes**: Safer file operations to prevent data loss"
              echo ""
            fi
            
            # Traditional categorized changes
            echo "## 📝 Changes by Category"
            echo ""
            
            # Features
            if [ -n "$LAST_TAG" ]; then
              FEATURES=$(git log ${LAST_TAG}..HEAD --pretty=format:"%s" | grep -E "^feat(\(.*\))?:" | sed 's/^feat[^:]*: //' | grep -v "bump version" || true)
              if [ -n "$FEATURES" ]; then
                echo "### 🚀 New Features"
                echo "$FEATURES" | while IFS= read -r line; do
                  [ -n "$line" ] && echo "- $line"
                done
                echo ""
              fi
              
              # Bug Fixes
              FIXES=$(git log ${LAST_TAG}..HEAD --pretty=format:"%s" | grep -E "^fix(\(.*\))?:" | sed 's/^fix[^:]*: //' | grep -v "bump version" || true)
              if [ -n "$FIXES" ]; then
                echo "### 🐛 Bug Fixes"
                echo "$FIXES" | while IFS= read -r line; do
                  [ -n "$line" ] && echo "- $line"
                done
                echo ""
              fi
              
              # Refactoring
              REFACTORS=$(git log ${LAST_TAG}..HEAD --pretty=format:"%s" | grep -E "^refactor(\(.*\))?:" | sed 's/^refactor[^:]*: //' | grep -v "bump version" || true)
              if [ -n "$REFACTORS" ]; then
                echo "### 🔧 Refactoring"
                echo "$REFACTORS" | while IFS= read -r line; do
                  [ -n "$line" ] && echo "- $line"
                done
                echo ""
              fi
              
              # Documentation
              DOCS=$(git log ${LAST_TAG}..HEAD --pretty=format:"%s" | grep -E "^docs(\(.*\))?:" | sed 's/^docs[^:]*: //' | grep -v "bump version" || true)
              if [ -n "$DOCS" ]; then
                echo "### 📚 Documentation"
                echo "$DOCS" | while IFS= read -r line; do
                  [ -n "$line" ] && echo "- $line"
                done
                echo ""
              fi
            fi
            
            # Collapsible full commit list
            echo "<details>"
            echo "<summary>📋 View all commits</summary>"
            echo ""
            if [ -n "$LAST_TAG" ]; then
              git log ${LAST_TAG}..HEAD --pretty=format:"- %s (%an)" | grep -v "bump version"
            else
              git log --pretty=format:"- %s (%an)" | head -20
            fi
            echo ""
            echo "</details>"
            echo ""
            
            # Key features section
            echo "## 🎯 Key Features"
            echo ""
            echo "- **Instant Data Preview**: CSV/JSON files load immediately"
            echo "- **Visual Feedback**: Key press indicator, cell highlighting"
            echo "- **Advanced Navigation**: Vim-style keys, viewport/cursor lock"
            echo "- **Powerful Search**: Regular search (Ctrl+F), fuzzy filter (Ctrl+/)"
            echo "- **Data Export**: Save as CSV or JSON"
            echo "- **Debug Mode**: Press F5 for comprehensive state information"
            echo ""
            
            # Installation
            echo "## 📦 Installation"
            echo ""
            echo "Download the binary for your platform from the assets below."
            echo ""
            
            echo "---"
            echo "**Thank you for using SQL CLI!** 🎉"
            echo ""
            echo "Report issues: [GitHub Issues](https://github.com/TimelordUK/sql-cli/issues)"
            
          } > RELEASE_NOTES.md
          
          # Save to output
          echo "notes<<EOF" >> $GITHUB_OUTPUT
          cat RELEASE_NOTES.md >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT
```

## Summary of Changes

The enhanced version adds:
1. **Statistics**: Commit count, files changed
2. **Feature Detection**: Scans commits for keywords to detect:
   - Visual improvements (key indicator, cell highlighting)
   - Debugging enhancements (dual logging, F5 mode)
   - Architecture improvements (state management)
   - Data protection (history recovery)
3. **Better Formatting**: Organized into meaningful sections
4. **Collapsible Details**: Full commit list in expandable section
5. **Key Features**: Highlights main capabilities

## Benefits

With this change, your release notes will automatically detect and highlight:
- ✅ Key press indicator and visual feedback features
- ✅ Dual logging system
- ✅ State management refactoring progress
- ✅ History protection improvements
- ✅ Transaction-like updates
- ✅ All the "hidden" work that doesn't show in commit prefixes

Instead of just seeing "refactor: ..." you'll get meaningful descriptions of what was actually improved!