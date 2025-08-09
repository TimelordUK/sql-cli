# State Manager Integration Guide

## Quick Integration Steps

### 1. Add StateManager to EnhancedTuiApp

```rust
use sql_cli::state_manager::StateManager;

pub struct EnhancedTuiApp {
    // ... existing fields ...
    state_manager: StateManager,
}

impl EnhancedTuiApp {
    pub fn new() -> Self {
        Self {
            // ... existing initialization ...
            state_manager: StateManager::new(),
        }
    }
}
```

### 2. Replace Direct Mode Switches with State Stack

Instead of:
```rust
self.buffer_mut().set_mode(AppMode::Search);
```

Use:
```rust
self.state_manager.push_mode(AppMode::Search, self.buffer_mut());
```

### 3. Handle ESC Key with State Stack

```rust
KeyCode::Esc => {
    // Pop back to previous mode with state restoration
    if !self.state_manager.pop_mode(self.buffer_mut()) {
        // No previous state - we're at the root
        self.clear_input();
    }
}
```

### 4. Example Mode Transitions

```rust
// Entering search from results mode
fn start_search(&mut self) {
    // State automatically saved before transition
    self.state_manager.push_mode(AppMode::Search, self.buffer_mut());
    self.sync_input_state();
}

// Returning from search to results
fn exit_search(&mut self) {
    // Previous state (cursor position, selection, etc.) restored
    self.state_manager.pop_mode(self.buffer_mut());
    self.sync_input_state();
}

// Sync app-level input with buffer
fn sync_input_state(&mut self) {
    let buffer = self.buffer();
    self.input = Input::new(buffer.get_input_text())
        .with_cursor(buffer.get_input_cursor_position());
}
```

### 5. Debug View Integration

Add state manager info to debug view:

```rust
// In F5 handler
debug_info.push_str("\n");
debug_info.push_str(&self.state_manager.format_debug_info());
```

## Benefits

1. **Preserves Context**: When you go Results → Search → ESC, you return to exact same position
2. **Nested Modes**: Can go Results → Filter → Search → Debug and unwind correctly
3. **Buffer Switching**: State preserved per-buffer when switching
4. **Debug Visibility**: Stack depth shown in debug view
5. **Memory Safe**: Bounded stack size prevents infinite growth

## Migration Strategy

1. Start with most problematic transitions (Search, Filter, FuzzyFilter)
2. Gradually replace all `set_mode()` calls with state manager
3. Test nested workflows thoroughly
4. Add custom state preservation for mode-specific data as needed