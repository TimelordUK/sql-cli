# Buffer-Only Design for TUI

## Problem Statement
The TUI currently has a mix of:
- Direct input field access for special modes (Search, Filter, etc.)
- Buffer-based access for Command/Results modes
- Legacy fallbacks everywhere

This creates complexity and bugs. We need EVERYTHING to go through buffers.

## Proposed Solution

### 1. Special Mode Buffers
Create lightweight buffers for each special mode:
- `SearchBuffer` - handles search input
- `FilterBuffer` - handles filter pattern input  
- `FuzzyFilterBuffer` - handles fuzzy filter input
- `ColumnSearchBuffer` - handles column search input

These are managed by the BufferManager just like regular buffers.

### 2. BufferManager Enhancement
```rust
enum BufferType {
    Query(Buffer),           // Normal query buffers
    Search(SearchBuffer),    // Search mode buffer
    Filter(FilterBuffer),    // Filter mode buffer
    // etc...
}

impl BufferManager {
    fn current_buffer(&self) -> &dyn BufferAPI;
    fn get_search_buffer(&mut self) -> &mut SearchBuffer;
    fn get_filter_buffer(&mut self) -> &mut FilterBuffer;
    // etc...
}
```

### 3. Unified Input Handling
All modes route through the same input pipeline:
```rust
fn handle_input(&mut self, key: KeyEvent) {
    match self.mode {
        AppMode::Command => self.buffer_manager.current().handle_input(key),
        AppMode::Search => self.buffer_manager.search_buffer().handle_input(key),
        AppMode::Filter => self.buffer_manager.filter_buffer().handle_input(key),
        // etc...
    }
}
```

### 4. Mode Transitions
When entering a special mode:
1. Save current buffer state
2. Switch to special mode buffer
3. Clear/prepare the special buffer
4. On exit, restore previous buffer

## Benefits
1. **Consistency**: All input goes through buffers
2. **No Fallbacks**: No more "if buffer exists" checks
3. **Clean Architecture**: Clear separation of concerns
4. **Testability**: Each buffer type can be tested independently
5. **Future Proof**: Easy to add new modes

## Implementation Steps

### Phase 1: Create Special Buffers
- [ ] Define `SearchBuffer`, `FilterBuffer`, etc.
- [ ] Implement minimal BufferAPI for each
- [ ] Add to BufferManager

### Phase 2: Remove TUI Input Fields
- [ ] Remove `input` field from TUI
- [ ] Remove `textarea` field from TUI
- [ ] Remove `edit_mode` field from TUI
- [ ] Remove ALL fallback code

### Phase 3: Route Everything Through Buffers
- [ ] Update all handle_*_input methods
- [ ] Update all mode transitions
- [ ] Update rendering to use buffer state

### Phase 4: Cleanup
- [ ] Remove all wrapper methods with fallbacks
- [ ] Ensure panic if no buffer (fail fast)
- [ ] Add debug assertions

## Alternative: Single Buffer with Modes

Instead of multiple buffer types, use a single Buffer with internal mode state:

```rust
struct Buffer {
    mode: BufferMode,
    query_state: QueryState,
    search_state: SearchState,
    filter_state: FilterState,
    // ...
}

enum BufferMode {
    Query,
    Search,
    Filter,
    // ...
}
```

This might be simpler but less flexible.

## Decision Needed

Which approach:
1. **Multiple specialized buffers** (more code, cleaner separation)
2. **Single buffer with modes** (less code, more complex state)
3. **Hybrid**: Regular buffers + one SpecialModeBuffer for all special modes

## Recommendation

Go with **Option 1** (Multiple specialized buffers) because:
- Cleaner separation of concerns
- Each buffer type is simple and focused
- Easier to test
- More maintainable long-term
- Aligns with single responsibility principle