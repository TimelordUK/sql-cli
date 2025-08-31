# Critical Architecture Problem: Multiple State Update Paths

## The Problem

We have THREE different ways state gets updated:

```
1. TUI → Buffer (direct)
2. TUI → AppStateContainer → Internal State
3. TUI → AppStateContainer → Buffer (what we're trying to add)
```

This creates a triangle of state management where:
- TUI directly calls `self.buffer_mut().set_mode()`, `set_status_message()`, etc.
- TUI also calls `self.state_container.set_table_selected_row()`
- AppStateContainer has its own duplicate state
- Buffer has its own state
- **These are NEVER synchronized!**

## Examples of Direct Buffer Access in TUI

```rust
// Setting mode directly on Buffer
self.buffer_mut().set_mode(AppMode::Command);

// Setting status directly on Buffer
self.buffer_mut().set_status_message(msg.to_string());

// Setting navigation state directly on Buffer
self.buffer_mut().set_last_results_row(Some(selected));
self.buffer_mut().set_last_scroll_offset(scroll_offset);

// Reading from Buffer directly
let input_text = self.buffer().get_input_text();
let scroll_offset = self.buffer().get_scroll_offset();
```

## The Correct Architecture

Everything should route through AppStateContainer:

```
TUI → AppStateContainer → Buffer
```

AppStateContainer should be the ONLY way to modify state. This gives us:
1. **Single entry point** for all state changes
2. **Centralized logging** and debugging
3. **State validation** in one place
4. **Side effects** management (e.g., clearing search when changing modes)
5. **No synchronization issues**

## Migration Strategy

### Option 1: Big Bang (Risky)
1. Remove `buffer()` and `buffer_mut()` from TUI
2. Add all Buffer methods to AppStateContainer as proxies
3. Update all TUI code at once
4. Fix all compilation errors

### Option 2: Gradual Migration (Safer)
1. **Phase 1**: Add proxy methods to AppStateContainer for all Buffer operations
2. **Phase 2**: Mark TUI's `buffer()` and `buffer_mut()` as deprecated
3. **Phase 3**: Gradually replace each direct Buffer call with AppStateContainer call
4. **Phase 4**: Remove deprecated methods
5. **Phase 5**: Remove duplicate state from AppStateContainer

### Option 3: Facade Pattern (Recommended)
1. Make Buffer private in AppStateContainer
2. AppStateContainer becomes a complete facade/API for all state
3. TUI can ONLY interact through AppStateContainer methods
4. AppStateContainer manages the complexity internally

## Key Insight

The current architecture violates the **Single Responsibility Principle**:
- TUI is managing state directly
- AppStateContainer is managing state
- Buffer is managing state

We need AppStateContainer to be the **single source of truth** and the **single point of control**.

## Implementation Order

1. **First**: Create AppStateContainer proxy methods for all Buffer operations
2. **Second**: Update TUI to use AppStateContainer exclusively  
3. **Third**: Remove duplicate state from AppStateContainer
4. **Fourth**: Buffer becomes a private implementation detail of AppStateContainer

This is a fundamental architectural fix that will prevent countless bugs.