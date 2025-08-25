# VimSearchAdapter Refactoring Plan

## Current Architecture
```
TUI → VimSearchAdapter → VimSearchManager
         ↓                    ↓
      Buffer              DataView
      ViewportManager
```

## Target Architecture
```
TUI → VimSearchAdapter → AppStateContainer → VimSearchManager
                              ↓
                          All State
```

## Keep VimSearchManager as Core Logic
VimSearchManager contains the actual search algorithms and should remain unchanged.
It's the adapter layer that needs refactoring.

## Refactoring Steps

### Step 1: Add Search Methods to AppStateContainer
```rust
impl AppStateContainer {
    /// Check if vim search should handle key
    pub fn vim_search_should_handle_key(&self) -> bool {
        let mode = self.get_mode();
        let pattern = self.get_search_pattern();
        mode == AppMode::Search || !pattern.is_empty()
    }
    
    /// Start vim search mode
    pub fn start_vim_search(&mut self) {
        self.set_mode(AppMode::Search);
        self.set_input_text(String::new());
        self.set_input_cursor_position(0);
    }
    
    /// Update vim search pattern and find matches
    pub fn update_vim_search(&mut self, pattern: String) {
        if let Some(dataview) = self.get_buffer_dataview() {
            // VimSearchManager would find matches here
            // Store matches in state
            self.set_search_pattern(pattern);
        }
    }
    
    /// Navigate to next vim search match
    pub fn vim_search_next(&mut self) {
        // Update viewport through state
        // VimSearchManager calculates position
    }
    
    /// Navigate to previous vim search match  
    pub fn vim_search_previous(&mut self) {
        // Update viewport through state
        // VimSearchManager calculates position
    }
    
    /// Exit vim search mode
    pub fn exit_vim_search(&mut self) {
        self.set_mode(AppMode::Results);
        self.clear_search_state();
    }
}
```

### Step 2: Simplify VimSearchAdapter
```rust
impl VimSearchAdapter {
    // BEFORE: Takes buffer, dataview, viewport
    pub fn should_handle_key(&self, buffer: &dyn BufferAPI) -> bool
    
    // AFTER: Takes AppStateContainer
    pub fn should_handle_key(&self, state: &AppStateContainer) -> bool {
        state.vim_search_should_handle_key()
    }
    
    // BEFORE: Takes multiple dependencies
    pub fn handle_key(&mut self, key: KeyCode, dataview: &DataView, viewport: &mut ViewportManager, buffer: &dyn BufferAPI) -> bool
    
    // AFTER: Only takes state container
    pub fn handle_key(&mut self, key: KeyCode, state: &mut AppStateContainer) -> bool {
        match key {
            KeyCode::Char('n') => {
                state.vim_search_next();
                true
            }
            KeyCode::Char('N') => {
                state.vim_search_previous();
                true
            }
            KeyCode::Enter => {
                state.confirm_vim_search();
                true
            }
            KeyCode::Esc => {
                state.exit_vim_search();
                true
            }
            _ => false
        }
    }
}
```

### Step 3: Update TUI Usage
```rust
// BEFORE
if self.vim_search_adapter.borrow().should_handle_key(self.buffer()) {
    // Complex logic
}

// AFTER  
if self.vim_search_adapter.borrow().should_handle_key(&self.state_container) {
    self.vim_search_adapter.borrow_mut().handle_key(key, &mut self.state_container);
}
```

## Benefits

1. **Single dependency** - VimSearchAdapter only needs AppStateContainer
2. **Cleaner API** - Methods take fewer parameters
3. **State consistency** - All state changes go through one place
4. **Easier testing** - Can test without UI components
5. **Maintains VimSearchManager** - Core logic stays unchanged

## Implementation Order

1. Add proxy methods to AppStateContainer for vim search operations
2. Update VimSearchAdapter to use AppStateContainer methods
3. Remove direct Buffer/DataView/ViewportManager dependencies from adapter
4. Update TUI to pass AppStateContainer instead of multiple dependencies
5. Test vim search functionality

## Key Insight

The VimSearchAdapter should be a thin layer that:
- Translates key events to state actions
- Routes everything through AppStateContainer
- Doesn't directly manipulate UI components

The VimSearchManager remains the brain with search algorithms, but accessed through AppStateContainer.