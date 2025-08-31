# Column Operations Analysis

## Current State Analysis

### 1. Hide Column Operation (`hide_current_column`)
**Location**: src/ui/enhanced_tui.rs:172-247
**Current Structure**:
- **Mode Check**: Returns early if not in Results mode
- **ViewportManager Interaction**: 
  - Gets visual column index from crosshair
  - Calls `viewport_manager.hide_column(visual_col_idx)`
  - Clones updated DataView
- **State Updates**:
  - Syncs DataView back to Buffer
  - Sets status message
  - Adjusts cursor position if needed
- **Dependencies**:
  - `self.viewport_manager` (RefCell)
  - `self.buffer_mut()`
  - `self.cursor_manager`
- **Lines of Code**: ~75 lines

### 2. Unhide All Columns (`unhide_all_columns`)
**Location**: src/ui/enhanced_tui.rs:251-282
**Current Structure**:
- **ViewportManager Interaction**:
  - Gets hidden column count
  - Calls `viewport_manager.unhide_all_columns()`
  - Clones updated DataView
- **State Updates**:
  - Syncs DataView back to Buffer
  - Sets status message
- **Dependencies**:
  - `self.viewport_manager` (RefCell)
  - `self.buffer_mut()`
- **Lines of Code**: ~32 lines

### 3. Move Column Left (`move_current_column_left`)
**Location**: src/ui/enhanced_tui.rs:285-332
**Current Structure**:
- **Mode Check**: Returns early if not in Results mode
- **ViewportManager Interaction**:
  - Gets current column from crosshair
  - Calls `viewport_manager.reorder_column_left(current_col)`
  - Returns `ColumnReorderResult` with success flag, new position, description
  - Gets new viewport and updated DataView
- **State Updates**:
  - Syncs DataView back to Buffer
  - Updates NavigationState (selected_column, scroll_offset)
  - Sets current column on Buffer
  - Sets status message
- **Dependencies**:
  - `self.viewport_manager` (RefCell)
  - `self.buffer_mut()`
  - `self.state_container.navigation_mut()`
- **Lines of Code**: ~48 lines

### 4. Move Column Right (`move_current_column_right`)
**Location**: src/ui/enhanced_tui.rs:335-382
**Current Structure**:
- **Mode Check**: Returns early if not in Results mode
- **ViewportManager Interaction**:
  - Gets current column from crosshair
  - Calls `viewport_manager.reorder_column_right(current_col)`
  - Returns `ColumnReorderResult` with success flag, new position, description
  - Gets new viewport and updated DataView
- **State Updates**:
  - Syncs DataView back to Buffer
  - Updates NavigationState (selected_column, scroll_offset)
  - Sets current column on Buffer
  - Sets status message
- **Dependencies**:
  - `self.viewport_manager` (RefCell)
  - `self.buffer_mut()`
  - `self.state_container.navigation_mut()`
- **Lines of Code**: ~48 lines

## Common Patterns Identified

### 1. ViewportManager as Single Source of Truth
All operations delegate to ViewportManager which:
- Manages the DataView
- Tracks crosshair position
- Returns operation results
- Maintains viewport state

### 2. Result Pattern
Operations return structured results containing:
- Success flag
- New position/state
- Description for status message
- Updated DataView

### 3. State Synchronization Pattern
After ViewportManager operation:
1. Clone updated DataView
2. Sync to Buffer
3. Update NavigationState if needed
4. Set status message
5. Update cursor if needed

### 4. Mode Guard Pattern
Most operations check `AppMode::Results` before proceeding

## Proposed Unified Result Structure

```rust
pub struct ColumnOperationResult {
    pub success: bool,
    pub description: String,
    pub updated_dataview: Option<Arc<DataView>>,
    pub new_column_position: Option<usize>,
    pub new_viewport: Option<Range<usize>>,
    pub hidden_count: Option<usize>,
}
```

## Proposed Helper Method

```rust
fn apply_column_operation_result(&mut self, result: ColumnOperationResult) {
    if !result.success {
        if !result.description.is_empty() {
            self.buffer_mut().set_status_message(result.description);
        }
        return;
    }

    // Sync DataView if updated
    if let Some(dataview) = result.updated_dataview {
        self.buffer_mut().set_dataview(Some(dataview));
    }

    // Update navigation state if column position changed
    if let Some(new_col) = result.new_column_position {
        let mut nav = self.state_container.navigation_mut();
        nav.selected_column = new_col;
        
        // Update scroll offset if viewport changed
        if let Some(viewport) = result.new_viewport {
            let pinned_count = self.buffer()
                .get_dataview()
                .map(|dv| dv.get_pinned_columns().len())
                .unwrap_or(0);
            nav.scroll_offset.1 = viewport.start.saturating_sub(pinned_count);
        }
        
        self.buffer_mut().set_current_column(new_col);
    }

    // Set status message
    self.buffer_mut().set_status_message(result.description);
}
```

## Simplification Strategy

### Phase 1: Create Helper Method
1. Add `ColumnOperationResult` struct
2. Implement `apply_column_operation_result()` helper
3. Test with one method first

### Phase 2: Refactor Each Method
Transform each method to ~8 line pattern:
```rust
pub fn hide_current_column(&mut self) {
    if self.buffer().get_mode() != AppMode::Results {
        return;
    }
    
    let result = self.viewport_manager.borrow_mut()
        .as_mut()
        .map(|vm| vm.hide_current_column_with_result())
        .unwrap_or_else(|| ColumnOperationResult::failure("No viewport manager"));
    
    self.apply_column_operation_result(result);
}
```

### Phase 3: ViewportManager Updates
Update ViewportManager methods to return unified result:
- `hide_current_column_with_result()` 
- `unhide_all_columns_with_result()`
- `reorder_column_left_with_result()`
- `reorder_column_right_with_result()`

## Dependencies to Maintain in TUI

These should remain in the TUI as they're core responsibilities:
- `buffer_mut()` - Buffer management
- `state_container` - Central state
- `viewport_manager` - View management
- Mode checking

## Benefits of This Approach

1. **Consistency**: All column operations follow same pattern
2. **Reduced Code**: ~200 lines reduced to ~50 lines
3. **Single Point of Truth**: ViewportManager handles all column logic
4. **Easy Testing**: Result structs are easy to test
5. **Trait Ready**: Methods have minimal dependencies
6. **Maintainability**: Changes to state sync happen in one place

## Risk Mitigation

1. **Test Coverage**: Run existing tests after each change
2. **Incremental Changes**: One method at a time
3. **Preserve Behavior**: All existing functionality maintained
4. **Clear Result Types**: Self-documenting return values

## Next Steps

1. ✅ Complete analysis
2. Create `ColumnOperationResult` struct
3. Implement `apply_column_operation_result()` helper
4. Refactor `hide_current_column()` first as test case
5. If successful, apply pattern to remaining methods
6. Update ViewportManager methods to return unified results
7. Run full test suite
8. Document changes

## Testing Commands

```bash
# Run specific column operation tests
cargo test column_ops
cargo test hide_column
cargo test move_column

# Integration tests
./integration_tests/test_scripts/test_column_ops.sh

# Full test suite
cargo test
```