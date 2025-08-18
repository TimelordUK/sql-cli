# Out-of-Order Column Navigation Investigation

## Issue Description
When columns are selected out of order in a DataView (e.g., `SELECT col11, col0, col5, col3, ...`), the crosshair cursor reportedly jumps around incorrectly during navigation, instead of moving sequentially through visual positions.

## Test Created
Created comprehensive unit test `test_out_of_order_column_navigation` in `src/ui/viewport_manager.rs:3335` that:
1. Creates a DataTable with 12 columns (col0 through col11)
2. Creates a DataView with columns selected out of order: `[11, 0, 5, 3, 8, 1, 10, 2, 7, 4, 9, 6]`
3. Tests navigation right and left through all columns
4. Verifies crosshair moves sequentially through visual positions (0,1,2,3...11)
5. Tests column hiding to ensure navigation still works correctly

## Test Results
**The test PASSES** - ViewportManager correctly handles out-of-order column navigation:
- Crosshair moves sequentially through visual positions (0,1,2,3...)
- DataTable indices are correctly mapped (11,0,5,3...)
- Navigation methods (`navigate_column_right` and `navigate_column_left`) properly:
  - Accept visual position as input
  - Update crosshair to next/previous visual position
  - Return corresponding DataTable index

## Architecture Analysis

### ViewportManager Design (CORRECT)
The ViewportManager correctly implements the separation of concerns:
- **Input**: Takes visual/display position (0,1,2,3...)
- **Internal**: Uses DataView's `get_display_columns()` to map visual position to DataTable index
- **Output**: Returns DataTable index for other components
- **Crosshair**: Maintains visual position in `crosshair_col`

### DataView Column Mapping (CORRECT)
DataView maintains proper column mapping via:
- `visible_columns`: Array of DataTable indices in display order
- `get_display_columns()`: Returns visual order of columns
- `column_names()`: Returns column names in visual order

## Hypothesis: The Issue is NOT in ViewportManager

Since the unit test passes, the navigation issue must be elsewhere in the system. Possible locations:

### 1. **Renderer Issue** (Most Likely)
The renderer might be incorrectly mapping visual positions when:
- Drawing the crosshair on screen
- Calculating column positions for display
- Using DataTable indices where visual indices should be used

### 2. **TUI Event Handling**
The EnhancedTuiApp might be:
- Passing wrong position to ViewportManager
- Incorrectly interpreting the result from ViewportManager
- Mixing visual and DataTable indices

### 3. **State Synchronization**
There might be inconsistency between:
- ViewportManager's crosshair position
- Buffer's cursor position
- Display rendering position

## Next Steps

1. **Add Debug Logging**: Enable viewport_manager debug logs to see actual values during runtime
2. **Check Renderer**: Look for places where column indices are used for rendering
3. **Trace Navigation Path**: Follow the complete path from keypress to screen update
4. **Create Integration Test**: Test the full stack (not just ViewportManager) with out-of-order columns

## Key Code Locations

- `src/ui/viewport_manager.rs:1689-1799` - `navigate_column_left` implementation
- `src/ui/viewport_manager.rs:1817-1940` - `navigate_column_right` implementation
- `src/ui/enhanced_tui.rs:4188-4241` - TUI calling navigation methods
- `src/data/data_view.rs:571-575` - `get_display_columns` implementation

## Running the Test

```bash
cargo test --lib test_out_of_order_column_navigation -- --nocapture
```

## Bug Found!

### The Issue
The bug is in the ambiguous use of column indices throughout the system:

1. **ViewportManager navigation methods** (`navigate_column_right`, `navigate_column_left`):
   - Input: visual column position 
   - Internal: Updates crosshair to new visual position correctly
   - Output: Returns DataTable index in `column_position`

2. **After navigation in EnhancedTuiApp** (lines 4203, 4258, etc.):
   - Takes the DataTable index from `nav_result.column_position`
   - Stores it in `buffer.current_column`
   - Stores it in `state_container.navigation.selected_column`

3. **The bug occurs when rendering/using the stored column**:
   - `buffer.get_current_column()` returns what was stored (DataTable index)
   - But code using it often assumes it's a visual index!
   - Example: Line 4613 uses it as index into `headers` array (visual order)
   - Example: Line 6706 uses it to index into row data (visual order)

### Root Cause
The `buffer.current_column` field is used ambiguously:
- Sometimes stores visual indices
- Sometimes stores DataTable indices  
- No clear contract about which type it should contain
- Code reading from it makes inconsistent assumptions

## Solution Implemented

The fix clarifies what type of index each component should store:

1. **ViewportManager**: Already correct - tracks visual position in `crosshair_col`
2. **Buffer**: Now stores visual index (to match how it's used in rendering)
3. **Navigation state**: Now stores visual index (for UI consistency)

### Changes Made

Modified navigation methods in `EnhancedTuiApp` (`move_column_left`, `move_column_right`, `goto_first_column`, `goto_last_column`):

**Before**: 
```rust
self.buffer_mut().set_current_column(nav_result.column_position);
self.state_container.navigation_mut().selected_column = nav_result.column_position;
```

**After**:
```rust
// Get the visual position from ViewportManager after navigation
let visual_position = {
    let viewport_manager_borrow = self.viewport_manager.borrow();
    viewport_manager_borrow
        .as_ref()
        .map(|vm| vm.get_crosshair_col())
        .unwrap_or(0)
};

// Apply navigation result to TUI state (using visual position)
self.buffer_mut().set_current_column(visual_position);
self.state_container.navigation_mut().selected_column = visual_position;
```

This ensures that after navigation:
- ViewportManager's `get_crosshair_col()` provides the visual position
- Buffer stores the visual position (not DataTable index)
- Navigation state stores the visual position
- Rendering code correctly interprets the stored position as visual index

## Conclusion

The ViewportManager is working correctly. The bug is in EnhancedTuiApp's handling of navigation results - it's storing DataTable indices where visual indices are expected. This causes the crosshair to "jump around" when columns are selected out of order because the rendering code interprets DataTable indices as visual positions.