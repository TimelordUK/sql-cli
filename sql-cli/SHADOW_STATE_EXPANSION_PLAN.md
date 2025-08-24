# Shadow State Manager Expansion Plan

## Current State
The shadow state manager is currently observing state transitions but not yet serving as the source of truth. State is scattered across multiple locations.

## Goal
Transform the shadow state manager from an observer to the central state authority, eliminating duplicate state storage throughout the TUI.

## Phase 1: State Inventory (Current Reality)

### State Currently Stored in Multiple Places:

1. **AppMode (Primary mode)**
   - Buffer: `self.buffer().get_mode()` / `set_mode()`
   - Shadow State: `self.shadow_state.borrow().get_state()`
   - Status: Duplicate storage, Buffer is authoritative

2. **Search State**
   - VimSearchManager: Tracks vim search pattern and matches
   - ColumnSearchState: In AppStateContainer
   - Buffer: Search pattern strings
   - Shadow State: Observes but doesn't control
   - Status: Highly fragmented

3. **Filter State**
   - Buffer: `filter_pattern`, `fuzzy_filter_pattern`, `filter_active`, `fuzzy_filter_active`
   - AppStateContainer: FilterState
   - DataView: Internal filter state
   - Status: Triple duplication

4. **Editor/Input State**
   - Buffer: `input_text`, cursor position
   - EditorWidget: Internal state
   - Status: Partially migrated to widgets

5. **Navigation State**
   - ViewportManager: cursor position, viewport, locks
   - Buffer: Some cursor state
   - Status: Mostly in ViewportManager

6. **Query State**
   - Buffer: `last_query`, `original_query`
   - Status: Only in Buffer

## Phase 2: Migration Strategy

### Step 1: Make Shadow State the Reader Interface
Instead of immediately removing state from Buffer, first make all reads go through shadow state:

```rust
// Instead of:
if self.buffer().get_mode() == AppMode::Results {
    // ...
}

// Use:
if self.shadow_state.borrow().is_in_results_mode() {
    // ...
}
```

### Step 2: Shadow State as Write-Through Cache
Shadow state becomes authoritative but still updates Buffer for compatibility:

```rust
impl ShadowStateManager {
    pub fn switch_to_mode(&mut self, mode: AppMode, buffer: &mut Buffer) {
        self.set_state(mode.clone());
        buffer.set_mode(mode); // Keep buffer in sync for now
    }
}
```

### Step 3: Remove Buffer State
Once all reads go through shadow state, remove the duplicate storage from Buffer.

## Phase 3: Implementation Order

### Priority 1: AppMode (Simplest, High Impact)
1. Add read methods to ShadowStateManager:
   - `is_in_results_mode()`
   - `is_in_command_mode()`
   - `is_in_search_mode()`
   - `get_current_mode()`

2. Replace all `buffer().get_mode()` calls with shadow state calls

3. Make shadow state authoritative for mode changes

4. Remove mode from Buffer

### Priority 2: Search State (Most Fragmented)
1. Consolidate search state in ShadowStateManager:
   ```rust
   pub struct UnifiedSearchState {
       vim_search: Option<VimSearchPattern>,
       column_search: Option<ColumnSearchPattern>,
       fuzzy_filter: Option<String>,
       text_filter: Option<String>,
       active_search: Option<SearchType>,
   }
   ```

2. Migrate VimSearchManager functionality to shadow state

3. Move column search from AppStateContainer to shadow state

4. Unify all search operations through shadow state

### Priority 3: Filter State
1. Move all filter state to shadow state
2. Remove from Buffer and AppStateContainer
3. Keep DataView as the executor but shadow state as the controller

## Phase 4: State Access Patterns

### Current (Scattered):
```
TUI -> Buffer -> get_mode()
TUI -> VimSearchManager -> get_pattern()
TUI -> AppStateContainer -> column_search()
TUI -> ViewportManager -> get_cursor()
```

### Target (Centralized):
```
TUI -> ShadowState -> mode()
TUI -> ShadowState -> search_state()
TUI -> ShadowState -> filter_state()
TUI -> ViewportManager -> get_cursor() // Keep navigation separate
```

## Phase 5: Benefits

1. **Single Source of Truth**: No more state synchronization bugs
2. **Easier Testing**: Mock one state manager instead of many
3. **Better Debugging**: All state transitions in one place
4. **Cleaner Architecture**: Clear separation of concerns
5. **Preparation for Undo/Redo**: State snapshots become trivial

## Implementation Checklist

### Week 1: AppMode Migration
- [ ] Add comprehensive read methods to ShadowStateManager
- [ ] Create compatibility layer for gradual migration
- [ ] Replace 50% of get_mode() calls
- [ ] Replace remaining get_mode() calls
- [ ] Make shadow state authoritative for mode
- [ ] Remove mode from Buffer

### Week 2: Search State Consolidation
- [ ] Design UnifiedSearchState structure
- [ ] Migrate VimSearchManager to shadow state
- [ ] Migrate column search from AppStateContainer
- [ ] Unify search interfaces
- [ ] Remove old search state storage

### Week 3: Filter State & Cleanup
- [ ] Migrate filter state to shadow
- [ ] Remove filter state from Buffer
- [ ] Clean up AppStateContainer
- [ ] Update all tests
- [ ] Documentation update

## Success Metrics

1. **Before**: 57+ scattered set_mode() calls
2. **After**: All state changes go through shadow state
3. **Before**: State bugs from synchronization issues
4. **After**: Single source of truth eliminates sync bugs
5. **Before**: Difficult to track state transitions
6. **After**: Complete state history in shadow state

## Next Immediate Steps

1. Create a new branch: `shadow_state_expansion`
2. Start with AppMode migration (simplest, highest impact)
3. Add comprehensive read methods to ShadowStateManager
4. Begin replacing `buffer().get_mode()` calls one file at a time