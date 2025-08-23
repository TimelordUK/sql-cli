# Key Migration Status and Next Steps

## Current State (2025-08-15)

### What We Just Fixed
1. **Column Sorting Issues** 
   - Fixed mismatch between view and source column indices when sorting
   - Added `apply_sort_internal()` method to work with source indices
   - Sorting now works correctly on the column under the cursor

2. **Key History Display**
   - Reverted from 50 keys back to 10 keys max capacity
   - Keys fade after 2 seconds on status line
   - Prevents status line overflow

3. **Pinned Columns Architecture**
   - Fixed duplicate pinned columns in display
   - Added `rebuild_visible_columns()` to maintain proper column order
   - `visible_columns` is now the single source of truth (includes pinned columns first)
   - Fixed `column_count()` and column movement methods to work with unified array

4. **Enhanced Debug View (F5)**
   - Added DataView internal state visibility
   - Shows `visible_columns` array, pinned columns, and sort state

## Where We Were: Key Migration Project

### Context
We were in the middle of migrating key handling from the main TUI loop into a dedicated action system. This work was partially complete when we temporarily diverted to attempt a widget refactor (which we then reverted).

### Key Migration Progress

#### Already Completed (from previous commits)
1. **Vim-style modes implemented**:
   - Insert mode ('i' key)
   - Append modes ('a', 'A') with SQL-aware cursor positioning
   - Command mode editing actions extracted to action system

2. **Action System Infrastructure**:
   - `src/action.rs` - Core action definitions
   - `src/handlers/mod.rs` - Handler module structure
   - Key handlers partially extracted from main loop

#### Still in TUI Main Loop (needs migration)
Based on our enhanced_tui.rs, these key handlers still need extraction:

1. **Navigation Keys**:
   - Arrow keys (Up, Down, Left, Right)
   - Page Up/Down
   - Home/End
   - 'g'/'G' (top/bottom)
   - 'h','j','k','l' (vim navigation)

2. **Column Operations**:
   - 'p' (pin/unpin column)
   - 'H' (hide column) 
   - Shift+Left/Right (move columns)
   - '/' (column search)
   - Tab/Shift+Tab (column navigation)

3. **Data Operations**:
   - 's' (sort toggle)
   - 'f' (filter)
   - 'F' (fuzzy filter)
   - Ctrl+F (SQL filter)
   - 'e' (export)
   - 'E' (export filtered)

4. **View Operations**:
   - F5 (debug view)
   - 'q'/'Q' (quit)
   - Esc (cancel operations)
   - Enter (various context-dependent actions)

## Next Steps for Key Migration

### Phase 1: Complete Handler Structure
1. Create dedicated handler files:
   - `src/handlers/navigation.rs` - All movement keys
   - `src/handlers/columns.rs` - Column operations (pin, hide, move, search)
   - `src/handlers/data.rs` - Data operations (sort, filter, export)
   - `src/handlers/view.rs` - View operations (debug, quit)

### Phase 2: Extract Key Processing
1. Move key matching logic from `handle_key_event()` to appropriate handlers
2. Each handler should:
   - Take `(&mut self, key: KeyEvent) -> Result<bool>`
   - Return true if key was handled
   - Update state through action system

### Phase 3: Centralize State Updates
1. All state changes go through actions
2. Remove direct state manipulation from TUI
3. Enable better testing and debugging of key handling

## Architecture Notes

### Current Issues
- TUI is still tightly coupled to DataView ("the tui is still v closely aware of the view")
- Viewport manager would help but is "rabbit hole" to implement now
- Key handling mixed with rendering logic

### Future Improvements (post key migration)
1. **Viewport Manager**: Separate view management from TUI rendering
2. **State Management**: Redux-like pattern for all state changes
3. **Widget System**: Revisit widget refactor with better foundation

## Branch History
- `tui_widgets_v1` (current) - Fixed sorting/pinning issues, reverted widget refactor
- Previous widget refactor branch (stashed) - Had some fixes we recovered
- Main branch - Stable baseline

## Testing Checklist
When resuming work:
1. Run `cargo test --test data_view_trades_test` - All should pass
2. Test on Windows before merging
3. Verify column operations:
   - Pin/unpin columns
   - Hide/show columns  
   - Sort with pinned columns
   - Move columns left/right

## Important Files
- `/src/ui/enhanced_tui.rs` - Main TUI with key handling to extract
- `/src/action.rs` - Action system to expand
- `/src/handlers/mod.rs` - Handler module to populate
- `/src/data/data_view.rs` - Fixed DataView with proper column handling
- `/src/key_indicator.rs` - Fixed key history display

## Commands to Remember
```bash
# Run tests
cargo test --test data_view_trades_test

# Build release
cargo build --release

# Format before commit
cargo fmt

# Check what keys are still in main loop
grep -n "KeyCode::" src/ui/enhanced_tui.rs
```

## Resume Point
Start with extracting navigation keys (arrows, page up/down, vim keys) into `src/handlers/navigation.rs` as they're the most straightforward and will establish the pattern for other handlers.