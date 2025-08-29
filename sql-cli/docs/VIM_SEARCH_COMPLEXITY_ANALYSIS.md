# VimSearch Methods Complexity Analysis

## The Problem

The TUI's vim search methods are doing WAY too much:

### Current vim_search_next() - 90+ lines of complexity:
```rust
fn vim_search_next(&mut self) {
    // 1. Check if navigating
    if !self.vim_search_adapter.borrow().is_navigating() {
        // 2. Get viewport
        // 3. Get dataview
        // 4. Call adapter.resume_last_search()
        // 5. Handle status messages
    }
    
    // 6. Get viewport again
    // 7. Call adapter.next_match()
    // 8. Get match info
    // 9. Update selected row
    // 10. Update selected column
    // 11. Navigate viewport
    // 12. Sync with other widgets
    // 13. Update status message with match count
}
```

## What It Should Be

```rust
fn vim_search_next(&mut self) {
    self.vim_search_adapter
        .borrow_mut()
        .navigate_next(&mut self.state_container);
}
```

That's it! One line!

## Why Is This Bad?

1. **TUI knows implementation details** - How to resume search, check navigation state
2. **Direct viewport manipulation** - TUI shouldn't touch viewport directly
3. **Complex borrow juggling** - Multiple RefCell borrows causing complexity
4. **Duplicated logic** - Same code in next() and previous()
5. **Status message management** - Scattered across TUI instead of centralized

## The Proper Architecture

### All logic moves to VimSearchAdapter:
```rust
impl VimSearchAdapter {
    pub fn navigate_next(&mut self, state: &mut AppStateContainer) -> bool {
        // Check if we need to resume
        if !self.is_navigating() {
            if !self.resume_search(state) {
                state.set_status_message("No previous search pattern".to_string());
                return false;
            }
        }
        
        // Get next match (adapter knows how)
        if let Some(search_match) = self.get_next_match(state) {
            // Update state (through container)
            state.set_selected_row(Some(search_match.row));
            state.set_selected_column(Some(search_match.col));
            
            // Update status
            let (current, total) = self.get_match_info();
            state.set_status_message(format!(
                "Match {}/{} at row {} col {}",
                current, total, search_match.row + 1, search_match.col + 1
            ));
            
            true
        } else {
            state.set_status_message("No more matches".to_string());
            false
        }
    }
}
```

## The Pattern We Keep Seeing

Every TUI method is like an iceberg:
- **Visible**: Simple method name like `vim_search_next()`
- **Hidden**: 50-100 lines of complex orchestration

This is backwards! The TUI should be thin, the adapter should be smart.

## Refactoring Steps

1. **Move ALL logic to VimSearchAdapter**
   - Resume search logic
   - Navigation logic
   - Status message updates
   - State updates

2. **VimSearchAdapter uses AppStateContainer**
   - Get dataview through state
   - Update selection through state
   - Set status messages through state

3. **TUI becomes a thin delegation layer**
   ```rust
   fn vim_search_next(&mut self) {
       self.vim_search_adapter.borrow_mut().navigate_next(&mut self.state_container);
   }
   
   fn vim_search_previous(&mut self) {
       self.vim_search_adapter.borrow_mut().navigate_previous(&mut self.state_container);
   }
   
   fn start_vim_search(&mut self) {
       self.vim_search_adapter.borrow_mut().start(&mut self.state_container);
   }
   ```

## Benefits

1. **Testability** - Can test search without UI
2. **Clarity** - TUI methods become self-documenting
3. **Reusability** - Search logic can be used elsewhere
4. **Less coupling** - TUI doesn't know HOW search works
5. **Easier debugging** - Logic is in one place

## The Real Issue

We keep thinking the TUI needs to orchestrate everything. It doesn't!
The TUI should just:
1. Receive user input
2. Delegate to appropriate component
3. Render the result

All the "how" should be in the components, not the TUI.