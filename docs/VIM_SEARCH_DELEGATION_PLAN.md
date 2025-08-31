# VimSearchAdapter Event Delegation Plan

## Current Problem: Escape Key in Results Mode

The most problematic behavior is:
1. In Results mode with active search
2. User presses Escape
3. TUI checks vim_search_adapter.is_active() directly
4. Either exits search OR switches to Command mode

This decision should be made by VimSearchAdapter, not TUI!

## Event Flow Architecture

### Current (Broken):
```
KeyEvent → TUI.try_handle_mode_dispatch → Mode Handler → Direct Logic
                                                          ↓
                                              Check vim_search_adapter.is_active()
                                              Make decision in TUI
```

### Target (Clean):
```
KeyEvent → TUI.try_handle_mode_dispatch → VimSearchAdapter.should_handle?
                                          ↓ Yes            ↓ No
                                   VimSearchAdapter     Mode Handler
                                   .handle_key()
```

## Implementation Plan

### Step 1: Add Early Check in try_handle_chord_processing
```rust
fn try_handle_chord_processing(&mut self, key: KeyEvent) -> Result<bool> {
    // FIRST: Let VimSearchAdapter handle if it wants to
    if self.vim_search_adapter.borrow().should_handle_key(&self.state_container) {
        let handled = self.vim_search_adapter
            .borrow_mut()
            .handle_key(key.code, &mut self.state_container);
        if handled {
            return Ok(false); // Key was handled, don't exit
        }
    }
    
    // THEN: Continue with normal chord processing
    // ...
}
```

### Step 2: VimSearchAdapter Handles Escape Logic
```rust
impl VimSearchAdapter {
    pub fn handle_key(&mut self, key: KeyCode, state: &mut AppStateContainer) -> bool {
        match (state.get_mode(), key) {
            // In Results mode + Escape = Exit search, stay in Results
            (AppMode::Results, KeyCode::Esc) if self.is_active => {
                state.set_status_message("Search mode exited".to_string());
                self.clear();
                true // We handled it
            }
            
            // In Search mode + Escape = Exit to Results
            (AppMode::Search, KeyCode::Esc) => {
                state.exit_vim_search();
                self.clear();
                true
            }
            
            // Navigation keys
            (_, KeyCode::Char('n')) if self.is_active => {
                self.next_match(state);
                true
            }
            
            (_, KeyCode::Char('N')) if self.is_active => {
                self.previous_match(state);
                true
            }
            
            _ => false // Not our key
        }
    }
}
```

### Step 3: Remove Direct Checks from TUI
Remove all these from enhanced_tui.rs:
```rust
// DELETE THIS:
if self.vim_search_adapter.borrow().is_active() {
    self.vim_search_adapter.borrow_mut().exit_navigation();
    self.state_container.set_status_message("Search mode exited".to_string());
    return;
}
```

## Key Insight: Push Points

The TUI should "push" events to VimSearchAdapter at these points:

1. **try_handle_chord_processing** (Results mode) - FIRST check
2. **handle_search_modes_input** (Search/Filter modes) - For typing
3. **start_vim_search** (When '/' is pressed) - To initialize

That's it! Three push points, clean delegation.

## Benefits

1. **TUI doesn't know about search logic** - Just pushes events
2. **Adapter owns all search behavior** - Including tricky Escape logic
3. **Clean separation** - TUI renders, Adapter manages state
4. **Testable** - Can test search behavior without UI

## The Escape Key Solution

The problematic Escape behavior becomes simple:
- VimSearchAdapter checks: "Am I active and in Results mode?"
- If yes: "I'll handle this - exit search, stay in Results"
- If no: "Not mine, let TUI handle mode switching"

This removes ALL the complex conditional logic from the TUI!