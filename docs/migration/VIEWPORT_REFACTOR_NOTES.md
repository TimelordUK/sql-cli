# ViewportManager Refactoring Notes
## Date: 2025-08-21

## Issue: Duplicate Column Width Calculation

### Current Problem
The TUI's `calculate_viewport_column_widths()` function (line 5599 in enhanced_tui.rs) is duplicating work that ViewportManager should be handling exclusively.

### What TUI is Currently Doing (Should NOT be):
```rust
fn calculate_viewport_column_widths(&mut self, viewport_start: usize, viewport_end: usize)
```
1. Directly accessing DataView
2. Calculating column widths based on visible rows
3. Checking terminal size
4. Applying compact mode logic
5. Setting widths on the buffer

### What ViewportManager Already Provides:
```rust
// In viewport_manager.rs
pub fn get_column_widths(&mut self) -> &[u16]
fn recalculate_column_widths(&mut self)
pub fn get_column_widths_for(&mut self, column_indices: &[usize]) -> Vec<u16>
```

ViewportManager already:
- Tracks terminal dimensions
- Calculates optimal column widths
- Handles packing modes (DataFocus, HeaderFocus, Balanced)
- Manages column visibility and viewport

## Refactoring Plan

### Step 1: Remove TUI's Column Width Calculation
- Delete `calculate_viewport_column_widths()` from enhanced_tui.rs
- Remove `calculate_optimal_column_widths()` if it exists
- Remove any direct column width logic from TUI

### Step 2: Use ViewportManager Exclusively
Replace all calls to TUI's width calculation with:
```rust
let widths = {
    let mut viewport_manager = self.viewport_manager.borrow_mut();
    if let Some(ref mut vm) = *viewport_manager {
        vm.get_column_widths().to_vec()
    } else {
        vec![]
    }
};
```

### Step 3: Handle Compact Mode
- Move compact mode logic into ViewportManager's packing modes
- Or create a new packing mode for compact display

### Step 4: Clean Up Buffer
- Remove `set_column_widths()` from Buffer if ViewportManager handles it
- Buffer should query ViewportManager when it needs widths

## Benefits of This Refactoring
1. **Single Source of Truth**: ViewportManager becomes the sole authority on column widths
2. **Better Performance**: No duplicate calculations
3. **Cleaner Architecture**: TUI focuses on rendering, not data calculations
4. **Easier Testing**: Column width logic isolated in one place
5. **More Features**: ViewportManager's packing modes are more sophisticated

## Related Issues
- TUI still has too much awareness of DataView internals
- Buffer stores column widths that ViewportManager also tracks
- Multiple places calculate "what fits on screen"

## Priority
**HIGH** - This is core to the viewport/display separation and affects performance on large datasets

## Estimated Effort
2-3 hours to refactor and test thoroughly

## Testing Required
- Verify column widths display correctly after refactor
- Test all packing modes (DataFocus, HeaderFocus, Balanced)
- Test compact mode behavior
- Test with very wide and very narrow terminals
- Test with many columns (100+)
- Test with long data values