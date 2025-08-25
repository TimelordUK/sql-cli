# TUI to AppStateContainer Migration Strategy

## Scale of the Problem
- **223** calls to `self.buffer_mut()`
- **112** calls to `self.buffer()`
- **335 total** direct Buffer accesses to migrate!

## Migration Approach

### Phase 1: Add Proxy Methods ✅ (DONE)
Added proxy methods to AppStateContainer for:
- set_mode() / get_mode()
- set_status_message() / get_status_message()
- set_dataview() / get_dataview()
- set_last_results_row()
- set_last_scroll_offset()
- is_buffer_modified() / set_buffer_modified()

### Phase 2: Gradual Migration (Current)
Start with the most common operations:

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

### Phase 3: Automation Script
Create a sed/awk script to help with mechanical replacements:

```bash
# For simple getters
sed -i 's/self\.buffer()\.get_mode()/self.state_container.get_mode()/g'
sed -i 's/self\.buffer()\.get_input_text()/self.state_container.get_input_text()/g'

# For simple setters (requires mut state_container)
sed -i 's/self\.buffer_mut()\.set_mode(/self.state_container.set_mode(/g'
```

### Phase 4: Complex Cases
Some cases need manual intervention:
- Where buffer() is stored in a variable
- Where buffer methods are chained
- Where BufferAPI trait is used directly

### Phase 5: Remove Direct Access
Once all migrations are done:
1. Remove `buffer()` and `buffer_mut()` from TUI
2. Make buffers field private in AppStateContainer
3. Buffer becomes implementation detail

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