# ViewportManager and Crosshair Status

## Checkpoint: 2025-08-17

### ✅ What's Working
1. **Column Navigation** - Moving left/right through columns works correctly
2. **Viewport Scrolling** - Viewport properly scrolls when navigating past visible columns
3. **Hidden Columns** - Navigation and crosshair work correctly with hidden columns
4. **Crosshair Synchronization** - Header and cell highlighting now stay in sync
5. **ViewportManager as Source of Truth** - All display position mapping centralized in ViewportManager

### 🔧 What Still Needs Work
1. **Pinned Columns** - Pin functionality not yet working correctly
2. **Coordinate System** - Still using DataTable indices instead of visual coordinates

### Key Improvements Made
1. **ViewportManager initialization** - Now properly initialized with `set_current_column(0)` on file load
2. **Centralized position conversion** - Added `get_display_position_for_datatable_column()` method
3. **Consistent crosshair logic** - Both headers and cells use ViewportManager for position mapping
4. **Debug logging** - Added extensive debug output for viewport state tracking

### Architecture Notes

#### Current Approach (DataTable Indices)
- Buffer stores DataTable column index (e.g., 5 for "commission")
- ViewportManager converts to display position for rendering
- Complex with hidden/pinned columns

#### Proposed Future Approach (Visual Coordinates)
- Buffer would store display position directly (e.g., 4 for 5th visible column)
- ViewportManager handles all DataTable ↔ Display mappings
- Simpler, more maintainable

### Next Steps
1. Fix pinned columns functionality
2. Consider refactoring to visual coordinate system
3. Add comprehensive tests for viewport scenarios

### Files Modified
- `src/ui/enhanced_tui.rs` - Crosshair highlighting logic
- `src/ui/viewport_manager.rs` - Position mapping and debug logging

### Branch
- `pin_attempt_v3` - Current working branch with improvements