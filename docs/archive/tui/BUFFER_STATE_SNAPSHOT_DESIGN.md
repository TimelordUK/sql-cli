# Buffer State Snapshot Design

## Goal
Enable true buffer switching where switching back to a buffer restores the EXACT state as if you never left - cursor position, viewport, mode, search state, everything.

## Key Insight
The Buffer becomes a complete "memento" that contains a snapshot of all view state. The state manager coordinates saving/restoring this snapshot transparently.

## Architecture

### 1. Buffer as State Container
```rust
pub struct Buffer {
    // ... existing data fields ...
    
    // State snapshot - populated when switching away
    state_snapshot: Option<BufferStateSnapshot>,
}

pub struct BufferStateSnapshot {
    // Mode and navigation
    mode: AppMode,
    cursor_position: (usize, usize),
    viewport_offset: (usize, usize),
    cursor_lock: bool,
    viewport_lock: bool,
    
    // Search state
    search_state: SearchStateSnapshot,
    
    // Filter state
    filter_pattern: Option<String>,
    fuzzy_filter_pattern: Option<String>,
    filter_active: bool,
    fuzzy_filter_active: bool,
    
    // Column state
    hidden_columns: Vec<String>,
    pinned_columns: Vec<String>,
    column_order: Vec<String>,
    sort_column: Option<usize>,
    sort_ascending: bool,
    
    // Input state (for command mode)
    input_text: String,
    input_cursor: usize,
    
    // Timestamp for debugging
    saved_at: std::time::Instant,
}

pub struct SearchStateSnapshot {
    vim_search: Option<VimSearchState>,
    column_search: Option<ColumnSearchState>,
    search_matches: Vec<(usize, usize)>,
    current_match: usize,
}
```

### 2. State Manager as Coordinator

The ShadowStateManager becomes responsible for:

```rust
impl ShadowStateManager {
    /// Save current state to buffer before switching away
    pub fn save_to_buffer(&self, buffer: &mut Buffer, viewport_manager: &ViewportManager) {
        let snapshot = BufferStateSnapshot {
            mode: self.get_mode(),
            cursor_position: viewport_manager.get_cursor_position(),
            viewport_offset: viewport_manager.get_viewport_offset(),
            cursor_lock: viewport_manager.is_cursor_locked(),
            viewport_lock: viewport_manager.is_viewport_locked(),
            // ... collect all state ...
            saved_at: std::time::Instant::now(),
        };
        
        buffer.set_state_snapshot(Some(snapshot));
        info!("Saved state snapshot to buffer");
    }
    
    /// Restore state from buffer when switching to it
    pub fn restore_from_buffer(&mut self, buffer: &Buffer, viewport_manager: &mut ViewportManager) {
        if let Some(snapshot) = buffer.get_state_snapshot() {
            // Restore mode
            self.set_state_from_mode(snapshot.mode);
            
            // Restore viewport
            viewport_manager.set_cursor_position(snapshot.cursor_position);
            viewport_manager.set_viewport_offset(snapshot.viewport_offset);
            viewport_manager.set_cursor_lock(snapshot.cursor_lock);
            viewport_manager.set_viewport_lock(snapshot.viewport_lock);
            
            // Restore search state
            if let Some(vim_search) = &snapshot.search_state.vim_search {
                self.restore_vim_search(vim_search);
            }
            
            // ... restore all state ...
            
            info!("Restored state snapshot from buffer (age: {:?})", snapshot.saved_at.elapsed());
        } else {
            // No snapshot - initialize default state for this buffer
            self.initialize_default_state(buffer);
        }
    }
}
```

### 3. TUI Integration (Transparent)

The TUI doesn't need to know about snapshots:

```rust
impl EnhancedTUI {
    /// Switch to a different buffer
    pub fn switch_to_buffer(&mut self, buffer_id: usize) {
        // Save current state to current buffer
        if let Some(current_buffer) = self.get_current_buffer_mut() {
            self.shadow_state.borrow().save_to_buffer(
                current_buffer,
                &self.viewport_manager.borrow()
            );
        }
        
        // Switch buffer
        self.set_current_buffer(buffer_id);
        
        // Restore state from new buffer
        if let Some(new_buffer) = self.get_current_buffer() {
            self.shadow_state.borrow_mut().restore_from_buffer(
                new_buffer,
                &mut self.viewport_manager.borrow_mut()
            );
        }
    }
}
```

## Implementation Phases

### Phase 1: Define Snapshot Structure
1. Create `BufferStateSnapshot` struct
2. Add `state_snapshot` field to Buffer
3. Implement getters/setters

### Phase 2: Implement Save Logic
1. Add `save_to_buffer` to ShadowStateManager
2. Collect state from all sources:
   - Shadow state itself
   - ViewportManager
   - AppStateContainer (search states)
   - DataView (column states)

### Phase 3: Implement Restore Logic
1. Add `restore_from_buffer` to ShadowStateManager
2. Distribute state to all targets:
   - Update shadow state
   - Update ViewportManager
   - Update AppStateContainer
   - Update DataView

### Phase 4: Wire Up Buffer Switching
1. Add buffer switching commands (e.g., Ctrl+Tab, :bnext, :bprev)
2. Implement switch_to_buffer in TUI
3. Test state preservation

## Benefits

1. **Perfect State Restoration**: Every aspect of the view is preserved
2. **Transparent to TUI**: TUI just calls switch_to_buffer()
3. **Centralized Logic**: State manager handles all complexity
4. **Debugging**: Can inspect saved snapshots
5. **Future: Undo/Redo**: Snapshots can be used for undo stack

## Considerations

### What State to Save?

**Always Save:**
- Mode
- Cursor position
- Viewport position
- Active filters
- Active searches
- Column visibility/order
- Sort state

**Maybe Save:**
- Undo history (could be large)
- Clipboard/yank buffer (might want global)
- Error messages (probably clear on switch)

**Don't Save:**
- Transient UI state (animations, tooltips)
- Debug overlays
- Performance metrics

### Memory Usage

Each snapshot might be ~1-2KB. With 100 buffers, that's only ~200KB - negligible.

### Snapshot Timing

Save snapshot when:
- Switching to another buffer
- Before executing a new query (save "Results" view)
- Optionally: Periodically for recovery

## Migration Path

Since we're already migrating to shadow state as the source of truth, we can:

1. First: Complete shadow state migration (current work)
2. Then: Add snapshot capability to shadow state
3. Finally: Implement buffer switching with state restoration

This means continuing our current path is the right approach - making shadow state authoritative prepares us for this snapshot system.

## Example User Experience

```
1. User runs query: SELECT * FROM users
   - Results shown, user navigates to row 50, column 3
   - Applies filter for "active" status
   - Sorts by created_date
   
2. User runs new query: SELECT * FROM orders  
   - State snapshot saved to buffer #1
   - New buffer #2 created
   - User browses orders
   
3. User switches back to buffer #1 (Ctrl+Tab)
   - State restored: Same row 50, column 3
   - Filter still active
   - Sort still applied
   - Feels like they never left!
```

## Next Steps

1. Continue current shadow state migration (make it authoritative)
2. Design BufferStateSnapshot struct with all needed fields
3. Implement save/restore methods
4. Add buffer switching commands
5. Test with multiple complex buffers