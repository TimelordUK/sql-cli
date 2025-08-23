# TUI Refactoring Progress

## Status as of 2025-08-23

### Current State
- **File**: `src/ui/enhanced_tui.rs`
- **Lines**: 7,700 (down from ~10,000 at start of day)
- **Reduction**: 2,300 lines removed (23%)

### Architecture Established
- **Visitor Pattern**: `action_handlers.rs` (486 lines)
  - NavigationActionHandler ✅
  - ColumnActionHandler ✅
  - ExportActionHandler ✅
  - YankActionHandler ✅
  - UIActionHandler (partial - ShowHelp only)
- **Search Operations**: `search_operations.rs` (180 lines) - foundation laid

### What's Been Moved to Visitor Pattern
These actions are now handled by the visitor pattern and removed from the switch:
- ✅ All Navigate() actions (Up/Down/Left/Right/PageUp/PageDown/Home/End/FirstColumn/LastColumn/JumpToRow/JumpToColumn)
- ✅ NextColumn, PreviousColumn
- ✅ ToggleColumnPin, HideColumn, UnhideAllColumns, ClearAllPins
- ✅ ExportToCsv, ExportToJson (stub implementations)
- ✅ All Yank() operations (Cell/Row/Column/All/Query)
- ✅ ShowHelp (partial - mode change only)

### What Remains in the Switch (line ~272 onwards)
Actions still in the legacy switch that could be migrated:

#### Simple Actions (Easy to Move)
- [ ] Quit, ForceQuit
- [ ] ToggleSelectionMode
- [ ] ToggleRowNumbers
- [ ] ToggleCompactMode
- [ ] ToggleCaseInsensitive
- [ ] ToggleKeyIndicator
- [ ] ToggleCursorLock
- [ ] ToggleViewportLock
- [ ] RefreshView
- [ ] ClearFilter
- [ ] Sort (basic version)

#### Medium Complexity (Need Some Work)
- [ ] StartJumpToRow
- [ ] StartSearch
- [ ] StartColumnSearch
- [ ] StartFilter
- [ ] StartFuzzyFilter
- [ ] NextMatch, PreviousMatch
- [ ] NextSearchMatch, PreviousSearchMatch
- [ ] NavigateToViewportTop/Middle/Bottom
- [ ] HideEmptyColumns
- [ ] MoveColumnLeft, MoveColumnRight
- [ ] SwitchMode, SwitchModeWithCursor
- [ ] ExitCurrentMode

#### Complex Actions (Keep in Legacy for Now)
- ⚠️ ShowDebugInfo (needs toggle_debug_mode() - complex state generation)
- ⚠️ ExecuteQuery (very complex, involves parser, data loading, etc.)
- ⚠️ ApplyFilter (complex data manipulation)
- ⚠️ LoadFromHistory (complex state management)
- ⚠️ StartHistorySearch (complex UI interaction)
- ⚠️ Yank operations that need context (complex clipboard ops)

### Tomorrow's Plan

1. **Create More Specialized Handlers**
   ```rust
   - ModeActionHandler (mode switching, exit operations)
   - DisplayActionHandler (toggle row numbers, compact mode, etc.)
   - SearchActionHandler (start search/filter modes)
   - ViewportActionHandler (viewport navigation and locking)
   ```

2. **Migration Strategy**
   - Start with the "Simple Actions" list above
   - Each handler should be ~50-100 lines
   - Keep complex initialization in legacy switch
   - Focus on actions that are pure state changes

3. **Expected Outcome**
   - Move 20-30 more actions to visitor pattern
   - Remove another 200-300 lines from the switch
   - Target: Reduce switch from ~800 lines to ~400 lines
   - Overall file target: 7,700 → ~7,000 lines

### Big Extraction Opportunities (Future)
After finishing the switch migration:

1. **Search/Filter System** (~1000 lines)
   - All the search modes logic
   - Filter application
   - Match navigation

2. **Rendering Methods** (~1000 lines)
   - render_table_immutable
   - render_results_table
   - render_command_area
   - etc.

3. **Mode-Specific Input Handlers** (~500 lines)
   - handle_command_input
   - handle_results_input
   - handle_help_input
   - etc.

4. **Data Operations** (~500 lines)
   - Sorting
   - Filtering
   - Column operations

### Notes
- The visitor pattern is working excellently
- ActionHandlerContext trait provides clean abstraction
- Gradual migration is successful - no regressions
- Dead code removal is accelerating the cleanup

### Commands for Tomorrow
```bash
# Check what's still in the switch
grep -n "^            [A-Z]" src/ui/enhanced_tui.rs | head -50

# Find complex dependencies
grep -A5 "ShowDebugInfo\|ExecuteQuery" src/ui/enhanced_tui.rs

# Test after each migration
cargo test action_handlers
cargo build --release
```