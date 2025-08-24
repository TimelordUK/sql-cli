# VimSearchManager vs VimSearchAdapter Architecture

## Separation of Concerns

### VimSearchManager (Core Search Logic)
**Responsibilities:**
- Finding matches in DataView
- Maintaining match list and current index
- Navigating between matches (next_match, previous_match)
- Highlighting current match
- Managing search pattern
- Updating viewport to show matches

**What stays here:**
```rust
impl VimSearchManager {
    // Core search functionality
    pub fn find_matches(&self, pattern: &str, dataview: &DataView) -> Vec<SearchMatch>
    pub fn next_match(&mut self, viewport: &mut ViewportManager)
    pub fn previous_match(&mut self, viewport: &mut ViewportManager)
    pub fn confirm_search(&mut self, dataview: &DataView, viewport: &mut ViewportManager)
    pub fn update_pattern(&mut self, pattern: String, dataview: &DataView, viewport: &mut ViewportManager)
    
    // These stay as core functionality
    fn navigate_to_match(&mut self, match: &SearchMatch, viewport: &mut ViewportManager)
    fn find_first_match_from(&self, row: usize, col: usize, pattern: &str, dataview: &DataView)
}
```

### VimSearchAdapter (State Coordination)
**Responsibilities:**
- Listening to state events from dispatcher
- Determining when VimSearchManager should be active
- Checking Buffer state to decide if keys should be handled
- Clearing VimSearchManager when search ends
- NOT duplicating search logic

**What the adapter does:**
```rust
impl VimSearchAdapter {
    // State coordination only
    pub fn should_handle_key(&self, buffer: &Buffer) -> bool {
        // Check Buffer state, not search logic
        buffer.mode == AppMode::Search || !buffer.search_state.pattern.is_empty()
    }
    
    // Delegates to VimSearchManager
    pub fn handle_key(&mut self, key: KeyCode, dataview: &DataView, viewport: &mut ViewportManager, buffer: &Buffer) -> bool {
        if !self.should_handle_key(buffer) {
            return false; // Let N key toggle line numbers
        }
        
        // Delegate actual search operations
        match key {
            KeyCode::Char('n') => {
                self.manager.next_match(viewport);
                true
            }
            KeyCode::Char('N') => {
                self.manager.previous_match(viewport);
                true
            }
            _ => false
        }
    }
}
```

## The Key Fix for N Toggle

The bug happens because VimSearchManager's internal `is_active()` method doesn't know when search mode has ended. The adapter fixes this by:

1. **Checking Buffer state** instead of internal state
2. **Listening to StateEvents** to know when to clear
3. **Delegating search operations** to VimSearchManager

## Data Flow

### Current (Broken):
```
User presses 'N' in Results mode after search
  → TUI checks VimSearchManager.is_active() 
  → Returns true (doesn't know search ended)
  → N handled as "previous match"
  → Line numbers don't toggle ❌
```

### With Adapter (Fixed):
```
User presses 'N' in Results mode after search
  → TUI checks VimSearchAdapter.should_handle_key(buffer)
  → Checks buffer.mode and buffer.search_state.pattern
  → Both indicate no active search
  → Returns false
  → N toggles line numbers ✅
```

## Implementation in EnhancedTui

```rust
// In enhanced_tui.rs
pub struct EnhancedTui {
    // OLD: vim_search_manager: VimSearchManager,
    // NEW: Use adapter
    vim_search_adapter: VimSearchAdapter,
    state_dispatcher: StateDispatcher,
    // ...
}

impl EnhancedTui {
    fn handle_key(&mut self, key: KeyCode) {
        // For N key
        if key == KeyCode::Char('N') {
            // Check adapter, not manager directly
            if !self.vim_search_adapter.should_handle_key(&self.buffer) {
                // Toggle line numbers
                self.buffer.show_row_numbers = !self.buffer.show_row_numbers;
            } else {
                // Let adapter handle previous match
                self.vim_search_adapter.handle_key(
                    key, 
                    &self.dataview, 
                    &mut self.viewport,
                    &self.buffer
                );
            }
        }
    }
    
    fn handle_escape(&mut self) {
        // Dispatch state change
        self.state_dispatcher.dispatch_mode_change(
            self.buffer.mode,
            AppMode::Results
        );
        // Adapter will be notified and clear VimSearchManager
    }
}
```

## Summary

- **VimSearchManager**: Keeps ALL search logic (finding, navigating, highlighting)
- **VimSearchAdapter**: ONLY handles state coordination and activation
- **No duplication**: Adapter delegates to manager for actual search operations
- **Clean separation**: Search logic stays in manager, state logic in adapter
- **N key fix**: Adapter checks Buffer state, not internal flags