# State Migration Audit - Remaining Work

## Audit Date: 2025-08-12

### Summary
Based on the git history analysis and code review, the state migration from `enhanced_tui.rs` to `AppStateContainer` is approximately **85% complete**. The major state components have been successfully migrated, but several smaller state items remain.

## Completed Migrations ✅

### From Git History (V-branches):
- **V1-V10**: Initial widget extraction and modularization
- **V11-V16**: AppStateContainer foundation
- **V17**: ResultsState migration (with caching and performance tracking)
- **V18-V24**: SearchState, FilterState migrations  
- **V25**: NavigationState migration with dual-lock modes (viewport + cursor lock)
- **V26**: SortState migration (completed with fixes)
- **V28**: HistorySearchState migration (Ctrl+R functionality)
- **V29**: HelpState migration

### Recently Completed (app_state_non_optional_v1 branch):
- Removed `Option<AppStateContainer>` wrapper - now mandatory
- Removed ~1000 lines of legacy fallback code
- Added history corruption recovery
- Added dual logging system
- Added visual enhancements (cell highlighting, key indicator)

## State Already in AppStateContainer

### Fully Migrated:
1. **SelectionState** - Basic structure exists but not fully utilized
   - `SelectionMode` enum (Row, Cell, Column)
   - Selected indices tracking
   - Mode toggling

2. **ClipboardState** - Structure exists but operations scattered
   - Yank history tracking
   - Multiple clipboard formats
   - Operation statistics

3. **ColumnSearchState** - Fully migrated
   - Pattern matching
   - Match navigation
   - History tracking

### Partially Migrated:
1. **InputState** - Core exists but some operations in TUI
2. **BufferManager** - Exists but some direct access patterns remain

## Remaining Work 🚧

### V27: Complete SelectionState Migration
**Current Issues:**
- Selection operations still handled in enhanced_tui.rs
- Cell/row/column selection logic scattered
- Visual selection rendering in TUI

**Required Actions:**
- Move all selection operations to AppStateContainer methods
- Centralize selection bounds calculation
- Move visual selection state management

### V28: Complete ClipboardState Migration  
**Current Issues:**
- Clipboard operations use direct `arboard::Clipboard` in TUI
- Yank operations partially in TUI (handle_yank_chord)
- System clipboard integration scattered

**Required Actions:**
- Move all clipboard operations to AppStateContainer
- Centralize system clipboard access
- Move yank chord handling to state container

### V29: Verify ColumnSearchState Complete
**Status:** Appears complete but needs verification
- Pattern management ✅
- Match navigation ✅
- Integration with column highlighting ✅

### V30: Add ChordState to AppStateContainer
**Current State:** No ChordState exists - chord handling is ad-hoc

**Required Structure:**
```rust
pub struct ChordState {
    first_key: Option<char>,
    waiting_for_second: bool,
    timeout: Instant,
    chord_map: HashMap<String, ChordAction>,
}
```

**Required Actions:**
- Create ChordState structure
- Move chord detection logic from TUI
- Implement chord timeout handling
- Define ChordAction enum

### V31: Final Cleanup
**Remaining in enhanced_tui.rs:**
- `scroll_offset: (usize, usize)` - Should move to NavigationState
- `current_column: usize` - Should move to NavigationState  
- `show_help: bool` - Already noted as TODO
- `table_state: TableState` - Ratatui widget state, may need wrapper
- Direct field access patterns
- Temporary fallback states

## Code Locations

### Files to Update:
1. **src/enhanced_tui.rs** (lines ~140-180)
   - Remove remaining state fields
   - Update all direct state access
   - Remove fallback states

2. **src/app_state_container.rs** 
   - Add ChordState structure (new)
   - Enhance SelectionState methods
   - Complete ClipboardState integration

## Migration Strategy

### Approach per V-branch:
1. **Create branch** (e.g., `refactor-v27-selection-complete`)
2. **Move state field** to AppStateContainer
3. **Add accessor methods** with logging
4. **Update TUI** to use new methods
5. **Test thoroughly** - especially edge cases
6. **Merge to main** quickly to avoid conflicts

### Testing Checklist per Migration:
- [ ] Basic operations work
- [ ] Edge cases handled
- [ ] F5 debug view shows state
- [ ] No regressions in existing features
- [ ] Performance unchanged

## Risk Areas

### High Risk:
- **Clipboard integration** - System-specific behavior
- **Chord timing** - Needs careful timeout handling

### Medium Risk:
- **Selection bounds** - Complex calculation logic
- **TableState** - Tightly coupled to ratatui

### Low Risk:
- **scroll_offset** - Simple position tracking
- **current_column** - Simple index

## Estimated Timeline

- **V27 (SelectionState)**: 2-3 hours
- **V28 (ClipboardState)**: 3-4 hours  
- **V29 (ColumnSearch verify)**: 1 hour
- **V30 (ChordState)**: 4-5 hours (new implementation)
- **V31 (Cleanup)**: 2-3 hours

**Total: ~13-16 hours of focused work**

## Success Metrics

### When Complete:
1. `enhanced_tui.rs` contains NO state fields (only service references)
2. All state accessible via AppStateContainer methods
3. F5 debug dump shows complete application state
4. No direct state mutations outside AppStateContainer
5. Comprehensive state logging for debugging

## Next Steps

1. **Start with V27** - SelectionState is partially done
2. **Test each migration** thoroughly before moving on
3. **Keep branches small** - one state item per branch
4. **Document patterns** discovered during migration
5. **Update this audit** after each V-branch completion