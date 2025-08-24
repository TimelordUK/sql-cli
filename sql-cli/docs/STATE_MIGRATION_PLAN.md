# Practical State Migration Plan: AppStateContainer → Buffer

## State Categories Analysis

### Category 1: Buffer-Persistent State (MUST migrate to Buffer)
These must be saved for perfect buffer restoration:

1. **Query & Data State**
   - `command_input` - Query text and cursor position
   - `results` - DataView with query results
   - `results_cache` - Cached query results

2. **View Configuration**
   - `column_search` - Which columns are visible/searched
   - `filter` - Active filters on data
   - `sort` - Sort configuration per column
   - `selection` - Selected rows/cells
   - `navigation` - Cursor position in results
   - `scroll` - Viewport position

3. **Search States**
   - `search` - Vim search pattern and matches
   - `history_search` - History search state

4. **Mode State**
   - `mode_stack` - Current mode and mode history

### Category 2: TUI-Transient State (can stay in AppStateContainer)
These are UI-only and don't need buffer persistence:

1. **UI Widgets**
   - `widgets` - Help, stats, debug widget states
   - `column_stats` - Temporary stats display
   - `jump_to_row` - Temporary jump dialog

2. **History & Debug**
   - `command_history` - Shared across all buffers
   - `key_press_history` - Debug/diagnostic only

3. **System State**
   - `clipboard` - System clipboard integration
   - `chord` - Key chord detection (UI event handling)
   - `completion` - Tab completion state (regenerated on demand)

4. **Undo/Redo**
   - `undo_redo` - Could be either, depends on requirements

## Migration Strategy

### Phase 1: Create BufferState Struct (Week 1)
```rust
// In buffer.rs
pub struct BufferState {
    // Start with most critical state
    pub query: QueryState,
    pub data: DataState,
    pub view: ViewState,
}

pub struct QueryState {
    pub text: String,
    pub cursor_position: usize,
    pub last_executed: Option<String>,
}

pub struct DataState {
    pub data_view: Option<DataView>,
    pub cache_key: Option<String>,
}

pub struct ViewState {
    pub cursor: (usize, usize),
    pub viewport: ViewportState,
    pub columns: ColumnState,
    pub filters: FilterState,
}
```

### Phase 2: Dual-Write Pattern (Week 2)
Keep AppStateContainer working while gradually migrating:

```rust
impl AppStateContainer {
    // Start writing to both places
    pub fn set_query(&mut self, query: String) {
        // Old location (keep working)
        self.command_input.borrow_mut().text = query.clone();
        
        // New location (build up)
        self.get_current_buffer()
            .borrow_mut()
            .state
            .query
            .text = query;
    }
    
    // Gradually switch reads to Buffer
    pub fn get_query(&self) -> String {
        // During migration: read from Buffer if available
        if let Some(buffer_query) = self.try_get_from_buffer() {
            buffer_query
        } else {
            // Fallback to old location
            self.command_input.borrow().text.clone()
        }
    }
}
```

### Phase 3: Component by Component Migration

#### Step 1: DataView (Highest Priority)
```rust
// Move DataView ownership to Buffer
impl Buffer {
    pub fn set_data_view(&mut self, view: DataView) {
        self.state.data.data_view = Some(view);
        self.state.data.version += 1;
    }
}

// AppStateContainer becomes a pass-through
impl AppStateContainer {
    pub fn get_data_view(&self) -> Option<&DataView> {
        self.get_current_buffer()
            .borrow()
            .state
            .data
            .data_view
            .as_ref()
    }
}
```

#### Step 2: Search States
```rust
// Unify all search states in Buffer
pub struct SearchState {
    pub vim: Option<VimSearchState>,
    pub column: Option<ColumnSearchState>,
    pub fuzzy: Option<String>,
    pub data: Option<String>,
}

// Migrate VimSearchManager to use Buffer
impl VimSearchManager {
    pub fn update(&mut self, buffer: &Buffer) {
        if let Some(vim_state) = &buffer.state.search.vim {
            self.pattern = vim_state.pattern.clone();
            self.matches = vim_state.matches.clone();
        }
    }
}
```

#### Step 3: View Configuration
Move column visibility, sorting, filtering to Buffer.

### Phase 4: Remove from AppStateContainer
Once dual-write is stable:
1. Remove state fields from AppStateContainer
2. Convert AppStateContainer methods to dispatch actions
3. Make AppStateContainer purely a coordinator

## Implementation Checklist

### Week 1: Foundation
- [ ] Create BufferState struct hierarchy
- [ ] Add state field to Buffer
- [ ] Implement state versioning
- [ ] Add debug logging for state changes

### Week 2: DataView Migration
- [ ] Move DataView to BufferState
- [ ] Dual-write DataView updates
- [ ] Switch reads to Buffer
- [ ] Test buffer switching preserves DataView
- [ ] Remove DataView from AppStateContainer

### Week 3: Search State Consolidation
- [ ] Create unified SearchState in Buffer
- [ ] Migrate VimSearchManager to read from Buffer
- [ ] Move column search to Buffer
- [ ] Test N key issue is fixed
- [ ] Remove search states from AppStateContainer

### Week 4: View State Migration
- [ ] Move viewport/cursor to Buffer
- [ ] Move column configuration to Buffer
- [ ] Move filter state to Buffer
- [ ] Test full view restoration on buffer switch

### Week 5: Cleanup
- [ ] Remove migrated fields from AppStateContainer
- [ ] Convert AppStateContainer to dispatcher pattern
- [ ] Update all components to read from Buffer
- [ ] Add comprehensive tests

## Testing Strategy

Create test to verify buffer switching:
```rust
#[test]
fn test_buffer_state_restoration() {
    // Setup buffer 1 with specific state
    let buffer1 = create_test_buffer();
    buffer1.state.query.text = "SELECT * FROM users";
    buffer1.state.data.data_view = Some(test_data_view());
    buffer1.state.view.cursor = (5, 10);
    
    // Switch to buffer 2
    let buffer2 = create_test_buffer();
    buffer2.state.query.text = "SELECT * FROM orders";
    
    // Switch back to buffer 1
    // Assert all state is restored
    assert_eq!(buffer1.state.query.text, "SELECT * FROM users");
    assert_eq!(buffer1.state.view.cursor, (5, 10));
    assert!(buffer1.state.data.data_view.is_some());
}
```

## Success Criteria

1. **Buffer switching works perfectly** - All visual state restored
2. **N key issue fixed** - Search states properly managed
3. **No duplicate state** - Single source of truth
4. **Performance maintained** - No excessive cloning
5. **Backwards compatible** - Gradual migration without breakage

## Key Principles

1. **Dual-write during migration** - Keep both systems working
2. **Migrate by feature** - DataView first, then search, then view
3. **Test each migration** - Verify buffer switching works
4. **Version everything** - Avoid unnecessary re-renders
5. **Keep transient state separate** - Not everything goes in Buffer

## What NOT to Migrate

Keep these in AppStateContainer or TUI:
- Widget states (help, debug panels)
- Key press history (diagnostic only)
- Chord detection (event handling)
- System clipboard (OS integration)
- Command history (shared across buffers)