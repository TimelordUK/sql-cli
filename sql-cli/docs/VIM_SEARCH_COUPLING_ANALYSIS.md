# VimSearchAdapter Coupling Analysis

## Current Problems

### 1. Direct Buffer Dependency
The VimSearchAdapter still directly depends on BufferAPI:
```rust
pub fn should_handle_key(&self, buffer: &dyn BufferAPI) -> bool {
    let in_search_mode = buffer.get_mode() == AppMode::Search;
    let has_pattern = !buffer.get_search_pattern().is_empty();
    // ...
}

pub fn handle_key(
    &mut self,
    key: KeyCode,
    dataview: &DataView,
    viewport: &mut ViewportManager,
    buffer: &dyn BufferAPI,  // <- Still needs Buffer!
) -> bool
```

### 2. DataView Dependency
Requires direct DataView access instead of going through state:
```rust
pub fn update_pattern(
    &mut self,
    pattern: String,
    dataview: &DataView,  // <- Direct DataView access
    viewport: &mut ViewportManager,
)
```

### 3. ViewportManager Dependency
Directly manipulates ViewportManager instead of dispatching actions:
```rust
self.manager.next_match(viewport);  // <- Direct manipulation
```

### 4. TUI Still Manages Search State
The TUI is still orchestrating search operations:
```rust
// In enhanced_tui.rs
if self.vim_search_adapter.borrow().is_active() {
    self.vim_search_adapter.borrow_mut().exit_navigation();
    self.state_container.set_status_message("Search mode exited".to_string());
}
```

## What It Should Be

### Ideal Architecture
```
User Input → TUI → Action → StateContainer → VimSearchState → Events → TUI (render)
```

### VimSearchAdapter Should:
1. **Only interact with AppStateContainer** - No direct Buffer/DataView access
2. **Dispatch actions** - Not directly manipulate state
3. **Receive state updates** - Through subscriptions/events
4. **Be data-only** - No UI/rendering logic

## Refactoring Plan

### Step 1: Move Search State to AppStateContainer
```rust
// In AppStateContainer
pub struct SearchState {
    mode: SearchMode,
    pattern: String,
    matches: Vec<(usize, usize)>,
    current_match: usize,
    is_active: bool,
}

impl AppStateContainer {
    pub fn search_state(&self) -> &SearchState { ... }
    pub fn search_state_mut(&mut self) -> &mut SearchState { ... }
    
    pub fn start_vim_search(&mut self) { ... }
    pub fn update_search_pattern(&mut self, pattern: String) { ... }
    pub fn next_search_match(&mut self) { ... }
    pub fn previous_search_match(&mut self) { ... }
    pub fn exit_search(&mut self) { ... }
}
```

### Step 2: Convert VimSearchAdapter to State Manager
```rust
impl VimSearchAdapter {
    // Instead of:
    pub fn should_handle_key(&self, buffer: &dyn BufferAPI) -> bool
    
    // Should be:
    pub fn should_handle_key(&self, state: &AppStateContainer) -> bool {
        let search_state = state.search_state();
        search_state.is_active || !search_state.pattern.is_empty()
    }
    
    // Instead of:
    pub fn handle_key(&mut self, key: KeyCode, dataview: &DataView, viewport: &mut ViewportManager, buffer: &dyn BufferAPI)
    
    // Should be:
    pub fn handle_key(&mut self, key: KeyCode, state: &mut AppStateContainer) -> bool {
        match key {
            KeyCode::Char('n') => {
                state.next_search_match();
                true
            }
            KeyCode::Char('N') => {
                state.previous_search_match();
                true
            }
            // ...
        }
    }
}
```

### Step 3: TUI Only Handles Rendering
```rust
// In enhanced_tui.rs
fn render_search_status(&self, f: &mut Frame, area: Rect) {
    let search_state = self.state_container.search_state();
    if search_state.is_active {
        // Render search UI based on state
    }
}
```

## Benefits of Proper Decoupling

1. **Testability** - Can test search logic without UI
2. **Reusability** - Search logic can be used by other UIs
3. **Clarity** - Clear separation of concerns
4. **State Consistency** - Single source of truth
5. **Redux Pattern** - Actions → State → View

## Current State Summary

❌ **Failed as Redux Pattern** - Still tightly coupled to UI components
❌ **Direct Dependencies** - Buffer, DataView, ViewportManager
❌ **State Management** - Split between adapter and TUI
❌ **Action Dispatch** - Direct manipulation instead of actions

This was meant to be our first proper decoupled component following Redux patterns, but it ended up being another tightly coupled UI component.