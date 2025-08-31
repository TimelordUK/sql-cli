# Centralized State Management Design

## Problem Statement

Currently, application state is scattered across multiple components:
- Search state: Buffer, VimSearchManager, ColumnSearchState
- Navigation state: ViewportManager, Buffer navigation
- Mode state: Buffer mode, various widget states
- Action context: Multiple boolean flags computed ad-hoc

This leads to:
- **Inconsistent state**: 'N' key stuck in search mode after clearing search
- **Complex coordination**: Action context needs to check multiple sources
- **State synchronization bugs**: Components get out of sync
- **Hard to debug**: State changes happen in many places

## Proposed Solution: Redux-Style State Manager

Create a central `AppStateManager` that owns all application state and publishes state transitions.

### Architecture

```rust
pub struct AppStateManager {
    // Core state
    mode: AppMode,
    search_state: SearchState,
    navigation_state: NavigationState,
    
    // Subscribers that get notified of state changes
    subscribers: Vec<Box<dyn StateSubscriber>>,
}

pub enum StateTransition {
    EnterSearchMode { search_type: SearchType },
    ExitSearchMode,
    ExecuteQuery { query: String },
    NavigateToCell { row: usize, col: usize },
    // ... other transitions
}

pub trait StateSubscriber {
    fn on_state_change(&mut self, old_state: &AppState, new_state: &AppState);
}
```

### Consolidated State

Instead of checking multiple sources:

```rust
// BEFORE: Scattered checks
has_search: !buffer.get_search_pattern().is_empty() 
    || self.vim_search_manager.borrow().is_active()
    || self.state_container.column_search().is_active

// AFTER: Single source of truth
has_search: app_state_manager.is_search_active()
```

### State Transitions

All state changes flow through the central manager:

```rust
impl AppStateManager {
    pub fn transition(&mut self, transition: StateTransition) -> Result<()> {
        let old_state = self.current_state.clone();
        
        // Apply transition
        match transition {
            StateTransition::EnterSearchMode { search_type } => {
                self.search_state = SearchState::Active { search_type };
                self.mode = AppMode::Search;
            }
            StateTransition::ExitSearchMode => {
                self.search_state = SearchState::Inactive;
                self.mode = self.previous_mode;
            }
            // ... other transitions
        }
        
        // Notify all subscribers
        let new_state = &self.current_state;
        for subscriber in &mut self.subscribers {
            subscriber.on_state_change(&old_state, new_state);
        }
        
        Ok(())
    }
}
```

### Integration Points

1. **Action System**: Gets state from single source
2. **TUI Components**: Subscribe to relevant state changes
3. **Key Mapping**: Context computed from central state
4. **Render Pipeline**: Single state source for all rendering decisions

## Implementation Plan

### Phase 1: Core Infrastructure
- [ ] Create `AppStateManager` struct
- [ ] Define `StateTransition` enum
- [ ] Implement subscriber pattern
- [ ] Add basic search state consolidation

### Phase 2: Migration
- [ ] Migrate search state from scattered locations
- [ ] Update action context to use central state
- [ ] Convert key handlers to use state transitions
- [ ] Update TUI rendering to subscribe to state

### Phase 3: Expansion
- [ ] Add navigation state management
- [ ] Add mode transition management
- [ ] Add filter/sort state management
- [ ] Remove obsolete state coordination code

## Benefits

1. **Single Source of Truth**: All state in one place
2. **Predictable Updates**: All changes go through transition system
3. **Easy Debugging**: State history and logging in one place
4. **Consistent Behavior**: No more state synchronization bugs
5. **Testable**: Easy to unit test state transitions

## Example: Search State Fix

The current bug where 'N' key stays in search mode would be fixed because:

```rust
// When executing a query
app_state_manager.transition(StateTransition::ExecuteQuery { query });

// StateManager automatically:
// 1. Clears all search states
// 2. Notifies action system 
// 3. Updates key mapping context
// 4. Ensures 'N' key maps to toggle_line_numbers
```

This ensures the action system always has the correct context without manual coordination.

## Next Steps

Start with Phase 1 in a new branch after completing current Phase 2 refactoring work.