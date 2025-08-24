# Phase 1 Implementation: Consolidate DataView in Buffer

## Current State Analysis

### Buffer Already Has:
```rust
pub struct Buffer {
    // Good - these are already here:
    pub dataview: Option<DataView>,
    pub datatable: Option<DataTable>,
    pub mode: AppMode,
    pub search_state: SearchState,
    pub filter_state: FilterState,
    pub fuzzy_filter_state: FuzzyFilterState,
    // ... lots more
}
```

### AppStateContainer Has Duplicates:
```rust
pub struct AppStateContainer {
    search: RefCell<SearchState>,      // DUPLICATE of buffer.search_state
    filter: RefCell<FilterState>,      // DUPLICATE of buffer.filter_state
    results: RefCell<ResultsState>,    // Contains DataView - DUPLICATE
    // ... more duplicates
}
```

## The Core Problem
We have the SAME state in TWO places! This is why the N key breaks - the search state in Buffer might be cleared but AppStateContainer's search state isn't.

## Immediate Fix: Route Through Buffer

### Step 1: Make AppStateContainer Use Buffer's State
Instead of having its own SearchState, AppStateContainer should route to Buffer:

```rust
impl AppStateContainer {
    // OLD - uses its own search state
    pub fn get_search_pattern(&self) -> String {
        self.search.borrow().pattern.clone()
    }
    
    // NEW - uses Buffer's search state
    pub fn get_search_pattern(&self) -> String {
        self.current_buffer()
            .map(|b| b.search_state.pattern.clone())
            .unwrap_or_default()
    }
}
```

### Step 2: Start with VimSearchManager
The N key issue is specifically with VimSearchManager. Let's fix that first:

```rust
// In enhanced_tui.rs
impl EnhancedTui {
    fn handle_escape(&mut self) -> Result<()> {
        // Current: only clears mode
        self.shadow_state.borrow_mut().exit_to_results(&mut self.buffer);
        
        // Add: clear search states in Buffer
        self.buffer.search_state = SearchState::default();
        self.buffer.filter_state = FilterState::default();
        
        // Add: notify VimSearchManager
        self.vim_search_manager.clear();
        
        Ok(())
    }
}
```

## Concrete Implementation Plan

### 1. Create a State Coordinator Trait
```rust
// src/state/coordinator.rs
pub trait StateCoordinator {
    /// Called when exiting any search mode
    fn on_exit_search(&mut self);
    
    /// Called when entering search mode
    fn on_enter_search(&mut self, search_type: SearchType);
    
    /// Called when mode changes
    fn on_mode_change(&mut self, from: AppMode, to: AppMode);
}

impl StateCoordinator for Buffer {
    fn on_exit_search(&mut self) {
        // Clear all search states
        self.search_state = SearchState::default();
        self.filter_state = FilterState::default();
        self.fuzzy_filter_state = FuzzyFilterState::default();
        
        // Log for debugging
        debug!("Buffer: Cleared all search states on exit");
    }
    
    fn on_enter_search(&mut self, search_type: SearchType) {
        // Clear other search types
        match search_type {
            SearchType::Vim => {
                self.filter_state = FilterState::default();
                self.fuzzy_filter_state = FuzzyFilterState::default();
            }
            SearchType::Filter => {
                self.search_state = SearchState::default();
                self.fuzzy_filter_state = FuzzyFilterState::default();
            }
            _ => {}
        }
    }
    
    fn on_mode_change(&mut self, from: AppMode, to: AppMode) {
        // Handle mode-specific cleanup
        match (from, to) {
            (AppMode::Search, AppMode::Results) |
            (AppMode::Filter, AppMode::Results) |
            (AppMode::FuzzyFilter, AppMode::Results) => {
                self.on_exit_search();
            }
            _ => {}
        }
    }
}
```

### 2. Use Coordinator in Shadow State
```rust
impl ShadowStateManager {
    pub fn set_mode_with_coordinator(
        &mut self, 
        mode: AppMode, 
        buffer: &mut Buffer, 
        trigger: &str
    ) {
        let old_mode = buffer.mode.clone();
        
        // Update mode
        self.set_mode(mode.clone(), buffer, trigger);
        
        // Trigger coordination
        buffer.on_mode_change(old_mode, mode);
    }
}
```

### 3. Fix the N Key Issue Specifically
```rust
// In enhanced_tui.rs, when handling Escape:
KeyCode::Esc => {
    let current_mode = self.shadow_state.borrow().get_mode();
    
    // Use the coordinator pattern
    self.shadow_state.borrow_mut()
        .set_mode_with_coordinator(
            AppMode::Results, 
            &mut self.buffer.borrow_mut(),
            "escape_key"
        );
    
    // VimSearchManager will check Buffer's search_state
    // and see it's been cleared, so N key will work
}
```

### 4. Make VimSearchManager Check Buffer State
```rust
impl VimSearchManager {
    pub fn is_active(&self, buffer: &Buffer) -> bool {
        // Check Buffer's state, not internal state
        !buffer.search_state.pattern.is_empty() || 
        buffer.mode == AppMode::Search
    }
    
    pub fn handle_key(&mut self, key: KeyCode, buffer: &Buffer) -> bool {
        if !self.is_active(buffer) {
            return false; // N key goes to line numbers
        }
        
        // Handle vim search keys
        match key {
            KeyCode::Char('n') => self.next_match(),
            KeyCode::Char('N') => self.prev_match(),
            _ => return false,
        }
        true
    }
}
```

## Testing the Fix

Create a test that reproduces the N key issue:
```rust
#[test]
fn test_n_key_after_search() {
    let mut tui = create_test_tui();
    let mut buffer = Buffer::new();
    
    // N key should toggle line numbers initially
    tui.handle_key(KeyCode::Char('N'), &mut buffer);
    assert!(buffer.show_row_numbers);
    
    // Enter search mode
    tui.handle_key(KeyCode::Char('/'), &mut buffer);
    assert_eq!(buffer.mode, AppMode::Search);
    
    // Type search pattern
    tui.handle_key(KeyCode::Char('t'), &mut buffer);
    tui.handle_key(KeyCode::Char('e'), &mut buffer);
    tui.handle_key(KeyCode::Char('s'), &mut buffer);
    tui.handle_key(KeyCode::Char('t'), &mut buffer);
    
    // Exit search mode
    tui.handle_key(KeyCode::Esc, &mut buffer);
    assert_eq!(buffer.mode, AppMode::Results);
    
    // N key should toggle line numbers again (THIS CURRENTLY FAILS)
    tui.handle_key(KeyCode::Char('N'), &mut buffer);
    assert!(!buffer.show_row_numbers); // Should toggle off
}
```

## Next Steps After This Works

Once we have the coordinator pattern working for the N key:

1. **Extend to all search types** - Column search, fuzzy filter, etc.
2. **Remove duplicate state** - Delete SearchState from AppStateContainer
3. **Add state versioning** - Avoid re-renders when state hasn't changed
4. **Create action system** - Replace direct mutations with dispatched actions

## Why This Approach Works

1. **Minimal changes** - We're not rewriting everything
2. **Fixes the bug** - N key will work because VimSearchManager checks Buffer
3. **Sets foundation** - Coordinator pattern can be extended
4. **Testable** - Can write tests for each state transition
5. **Gradual migration** - Can do one component at a time