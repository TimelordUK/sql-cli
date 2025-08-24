# State Transition Mapping: The Current Chaos

## Overview

Found **57 set_mode() calls** and **15+ search state operations** scattered across the codebase. This documents the current transition triggers and their required side effects.

## Mode Transition Triggers (57 locations!)

### 1. **Command → Results** (Query Execution)
**Trigger Locations**:
- `ui/enhanced_tui.rs:2251` - Execute query from command mode
- `ui/enhanced_tui.rs:2914` - Resume from results
- `ui/enhanced_tui.rs:2986` - Various result transitions
- `action_handler.rs:65` - Action system result mode
- Multiple other locations...

**Required Side Effects**:
```rust
// CURRENT: Scattered manual coordination
self.buffer_mut().set_mode(AppMode::Results);
// Missing: Clear search states, reset viewport, update status

// NEEDED: Coordinated transition
state_manager.transition(StateTransition::ExecuteQuery {
    query: query_text
});
// Should automatically:
// - Clear all search states (vim, column, fuzzy)  
// - Reset viewport to (0,0)
// - Update key mapping context
// - Reset status line to results mode
// - Initialize navigation state
```

### 2. **Results → Command** (Back to Input)
**Trigger Locations**:
- `ui/enhanced_tui.rs:463,771` - Escape key handlers
- `action_handler.rs:60,69,99` - Action system
- Multiple exit paths...

**Required Side Effects**:
```rust
// CURRENT: Only mode change
self.buffer_mut().set_mode(AppMode::Command);

// NEEDED: Full restoration
state_manager.transition(StateTransition::ReturnToCommand);
// Should restore:
// - Previous query text in input
// - Cursor position in input
// - Clear all results-mode state
// - Update status line
```

### 3. **Search Mode Entries** (Multiple Types)
**Trigger Locations**:
- `ui/enhanced_tui.rs:2706,3000,3019` - Column search entries
- `action_handler.rs:202` - General search
- `vim_search_manager.rs:45` - Vim search start

**Current Problems**:
```rust
// SCATTERED: Each search type managed separately
buffer.set_mode(AppMode::ColumnSearch);           // Column search
vim_search_manager.start_search();                // Vim search  
state_container.start_search(pattern);           // Regular search

// CONFLICTS: Multiple search states can be active!
// RESULT: 'N' key bug - system doesn't know which search is active
```

## Search State Transition Chaos

### Current Search State Locations
```rust
// 1. VimSearchManager - /search functionality
enum VimSearchState {
    Inactive,
    Typing { pattern: String },
    Navigating { matches, current_index },
}

// 2. ColumnSearchState - Column name search  
struct ColumnSearchState {
    is_active: bool,
    pattern: String,
    matching_columns: Vec<(usize, String)>,
}

// 3. Regular SearchState - Data search
struct SearchState {
    is_active: bool, 
    pattern: String,
    matches: Vec<SearchMatch>,
}

// 4. FilterState - Fuzzy filtering
struct FilterState {
    is_active: bool,
    pattern: String, 
    // ... filter logic
}

// PROBLEM: All can be active simultaneously!
```

### Search Transition Problems
```rust
// TRIGGER: User presses '/' for vim search
vim_search_manager.start_search();
// MISSING: Clear other search states!

// TRIGGER: User presses Escape
vim_search_manager.cancel_search();  
// MISSING: Update action context, key mappings, status line

// TRIGGER: Execute new query  
self.state_container.clear_search();
self.state_container.clear_column_search(); 
self.vim_search_manager.borrow_mut().cancel_search();
// FRAGILE: Manual coordination, easy to miss one
```

## Required State Coordination Matrix

| Transition | Buffer Mode | Search States | Viewport | Keys | Status |
|-----------|-------------|---------------|----------|------|--------|
| Execute Query | → Results | Clear ALL | Reset (0,0) | Navigation | Results info |
| Enter Vim Search | Same | Clear others → Vim | Preserve | Search nav | Search UI |
| Exit Search | Same | Clear current | Restore | Restore nav | Normal mode |
| Return to Command | → Command | Clear ALL | N/A | Input keys | Command UI |
| Switch Search Type | Same | Clear old → New | Adjust | New search | New search UI |

## Critical Coordination Points

### 1. **Search State Conflicts** (The 'N' key bug)
```rust
// CURRENT PROBLEM: Action context checks all sources
has_search: !buffer.get_search_pattern().is_empty() 
    || self.vim_search_manager.borrow().is_active()
    || self.state_container.column_search().is_active

// SOLUTION NEEDED: Single source of truth
app_state.current_search_type() -> Option<SearchType>
```

### 2. **Mode Transition Side Effects**
```rust
// CURRENT: Manual, inconsistent
set_mode(AppMode::Results);
// Sometimes clears search, sometimes doesn't
// Sometimes updates viewport, sometimes doesn't

// NEEDED: Automatic side effects
state_manager.transition(EnterResultsMode);
// Always clears search, always resets viewport, always updates UI
```

### 3. **Key Mapping Context**
```rust  
// CURRENT: Complex boolean logic in action context
ActionContext {
    has_search: /* 3-way check */,
    mode: /* buffer mode */,
    has_results: /* buffer check */,
    // ... more scattered flags
}

// NEEDED: Derived from central state
ActionContext::from_app_state(&state_manager.current_state())
```

## Implementation Strategy

### Phase 1: Search State Unification
1. Create `enum SearchType { None, Vim, Column, Data, Fuzzy }`
2. Single `current_search: SearchType` in state manager  
3. Replace all search state checks with single source
4. **Fix 'N' key bug immediately**

### Phase 2: Mode Transition Coordination  
1. Replace direct `set_mode()` calls with state transitions
2. Implement automatic side effects for each transition
3. Add state validation (prevent impossible combinations)

### Phase 3: Full State Centralization
1. Move all navigation, selection, filter state to manager
2. Implement complete state history and debugging
3. Remove all scattered state management

## Next Step: Start Small

Begin with **search state unification** - it's the highest pain point and most isolated. The 'N' key bug fix will validate the approach before expanding to full mode management.