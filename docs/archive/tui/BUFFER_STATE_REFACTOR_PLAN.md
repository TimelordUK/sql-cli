# Buffer State Refactor Plan

## Problem Statement
We have view state (crosshair position, scroll offset, selection) duplicated across:
- ViewportManager (in TUI)
- NavigationState (in AppStateContainer)  
- SelectionState (in AppStateContainer)
- Buffer (partial storage)

This causes synchronization bugs and makes buffer switching unreliable.

## Goal
Make Buffer the single source of truth for all its view state, with everything else being proxies or caches.

## Phase 1: Proxy Pattern for AppStateContainer

### Current (Broken)
```rust
AppStateContainer {
    navigation: RefCell<NavigationState>,  // Duplicate state!
    selection: RefCell<SelectionState>,    // Duplicate state!
    buffers: BufferManager,
}
```

### Target (Proxy Pattern)
```rust
AppStateContainer {
    buffers: BufferManager,
    // No navigation or selection fields!
}

impl AppStateContainer {
    pub fn navigation(&self) -> NavigationProxy {
        NavigationProxy::new(self.buffers.current())
    }
    
    pub fn selection(&self) -> SelectionProxy {
        SelectionProxy::new(self.buffers.current())
    }
}
```

### Implementation Steps

1. **Create ViewState struct in Buffer**
```rust
// In buffer.rs
pub struct ViewState {
    // Position
    pub crosshair_row: usize,
    pub crosshair_col: usize,
    pub scroll_offset: (usize, usize),
    
    // Selection
    pub selection_mode: SelectionMode,
    pub selected_cells: Vec<(usize, usize)>,
    pub selection_anchor: Option<(usize, usize)>,
    
    // Viewport config
    pub viewport_lock: bool,
    pub cursor_lock: bool,
}

impl Buffer {
    pub view_state: ViewState,  // Replaces individual fields
}
```

2. **Create Proxy structs**
```rust
// In app_state_container.rs
pub struct NavigationProxy<'a> {
    buffer: Option<&'a Buffer>,
}

impl NavigationProxy<'_> {
    pub fn selected_row(&self) -> usize {
        self.buffer
            .map(|b| b.view_state.crosshair_row)
            .unwrap_or(0)
    }
    
    pub fn selected_column(&self) -> usize {
        self.buffer
            .map(|b| b.view_state.crosshair_col)
            .unwrap_or(0)
    }
    
    pub fn scroll_offset(&self) -> (usize, usize) {
        self.buffer
            .map(|b| b.view_state.scroll_offset)
            .unwrap_or((0, 0))
    }
}

pub struct NavigationProxyMut<'a> {
    buffer: Option<&'a mut Buffer>,
}

impl NavigationProxyMut<'_> {
    pub fn set_selected_row(&mut self, row: usize) {
        if let Some(buffer) = &mut self.buffer {
            buffer.view_state.crosshair_row = row;
        }
    }
    // etc...
}
```

3. **Update all callers**
- Change `state.navigation().selected_row` to work with proxy
- Change `state.navigation_mut().selected_row = x` to use proxy setter

## Phase 2: ViewportManager Integration

### Option A: Single ViewportManager (Current approach)
- On buffer switch: Load buffer's ViewState into ViewportManager
- On any change: Save ViewportManager state back to buffer's ViewState
- Pro: Simple, less memory
- Con: Need save/restore logic

### Option B: ViewportManager per Buffer
- Each Buffer owns its ViewportManager
- On buffer switch: Just change which ViewportManager is active
- Pro: No save/restore needed
- Con: More memory, bigger refactor

### Recommendation: Start with Option A
We already have save/restore working. We can migrate to Option B later if needed.

## Phase 3: Cleanup

1. **Remove duplicate fields from Buffer**
   - `current_column` -> use `view_state.crosshair_col`
   - `scroll_offset` -> use `view_state.scroll_offset`
   - `table_state.selected()` -> use `view_state.crosshair_row`

2. **Remove NavigationState and SelectionState structs entirely**
   - They become just proxy types

3. **Ensure ViewportManager syncs with Buffer's ViewState**
   - On every navigation operation
   - On buffer switch

## Benefits

1. **Single source of truth** - Buffer owns all its view state
2. **Reliable buffer switching** - State travels with buffer
3. **No synchronization bugs** - Can't get out of sync if there's only one copy
4. **Cleaner architecture** - Clear ownership and data flow

## Migration Strategy

1. Start with Phase 1 (proxy pattern) - Less invasive
2. Test thoroughly with buffer switching
3. Move to Phase 2 once stable
4. Clean up in Phase 3

## Testing Plan

1. Create test with 3 buffers
2. Navigate to different positions in each
3. Switch between them rapidly
4. Verify position is preserved
5. Test with different selection modes
6. Test with hidden/pinned columns

## Risk Assessment

- **High Risk**: Breaking navigation during migration
  - Mitigation: Keep old code paths until new ones proven
  
- **Medium Risk**: Performance impact from proxy indirection
  - Mitigation: Measure and optimize hot paths
  
- **Low Risk**: Memory increase from ViewState struct
  - Mitigation: Negligible per buffer

## Success Criteria

1. Buffer switching preserves exact position
2. No duplicate state fields
3. All navigation/selection operations work
4. Performance unchanged or better
5. Code is simpler and more maintainable