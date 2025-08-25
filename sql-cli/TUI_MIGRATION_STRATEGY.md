# TUI to AppStateContainer Migration Strategy

## Scale of the Problem
- **Original**: 335 total direct Buffer accesses (223 buffer_mut, 112 buffer)
- **Current**: 72 total remaining (58 buffer_mut, 14 buffer)
- **Progress**: 79% complete (263 calls migrated)

## Migration Approach

### Phase 1: Add Proxy Methods ✅ (DONE)
Added proxy methods to AppStateContainer for:
- set_mode() / get_mode()
- set_status_message() / get_status_message()
- set_dataview() / get_dataview()
- set_last_results_row()
- set_last_scroll_offset()
- is_buffer_modified() / set_buffer_modified()

### Phase 2: Gradual Migration ✅ (79% COMPLETE)
Successfully migrated most common operations:

#### Most Common Patterns to Replace:
```rust
// BEFORE:
self.buffer_mut().set_mode(AppMode::Command)
// AFTER:
self.state_container.set_mode(AppMode::Command)

// BEFORE:
self.buffer_mut().set_status_message(msg)
// AFTER:
self.state_container.set_status_message(msg)

// BEFORE:
self.buffer().get_input_text()
// AFTER:
self.state_container.get_input_text()
```

### Phase 3: Automation Script ✅ (DONE)
Successfully used sed for mechanical replacements:

```bash
# Completed replacements:
sed -i 's/self\.buffer()\.get_mode()/self.state_container.get_mode()/g'
sed -i 's/self\.buffer()\.get_input_text()/self.state_container.get_input_text()/g'
sed -i 's/self\.buffer_mut()\.set_mode(/self.state_container.set_mode(/g'
sed -i 's/self\.buffer_mut()\.set_status_message(/self.state_container.set_status_message(/g'
sed -i 's/self\.buffer_mut()\.set_dataview(/self.state_container.set_dataview(/g'
# And many more...
```

### Phase 4: Complex Cases (IN PROGRESS)
Remaining 72 calls need manual intervention:
- Trait implementations that require BufferAPI (58 calls)
- Stats widget render that needs buffer reference (1 call)
- VimSearchAdapter integration (1 call)
- sync_all_input_states and other internal methods (12 calls)

### Phase 5: Remove Direct Access
Once all migrations are done:
1. Remove `buffer()` and `buffer_mut()` from TUI
2. Make buffers field private in AppStateContainer
3. Buffer becomes implementation detail

## New Grouped Operations Added
Created these grouped operations in AppStateContainer to reduce calls:
- `insert_char_at_cursor()` - Combines save_undo, get/set text, get/set cursor
- `set_input_text_with_cursor()` - Sets both text and cursor position
- `clear_search_state()` - Clears matches and sets status message
- `set_last_state()` - Sets both last_results_row and last_scroll_offset
- `clear_line()` - Save undo state and clear input
- `move_input_cursor_left/right()` - Cursor movement with bounds checking
- `backspace()` - Delete char with undo state
- `delete()` - Delete at cursor with undo state
- `reset_navigation_state()` - Reset all navigation-related state
- `clear_fuzzy_filter_state()` - Clear all fuzzy filter state

## Benefits After Migration
1. **Single entry point** - All state changes go through AppStateContainer
2. **Centralized logging** - Can add debug logging in one place
3. **State validation** - Can validate state changes
4. **Side effects** - Can trigger side effects (e.g., clear search on mode change)
5. **No sync issues** - Single source of truth

## Risk Mitigation
- Do this incrementally
- Test after each batch of changes
- Keep git commits small
- Can revert if issues found

## Order of Operations
1. Start with mode changes (most critical)
2. Then status messages
3. Then input operations
4. Then scroll/navigation
5. Finally, complex state operations