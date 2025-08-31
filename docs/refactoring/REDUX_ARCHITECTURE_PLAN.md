# Redux Architecture Plan

## Current State (What we have now)

```rust
// In handle_results_input() and handle_command_input()
let action = self.key_mapper.map_key(key, &context);
if let Some(action) = action {
    self.try_handle_action(action, &context)  // Direct handling
}
```

The action system is working but still directly mutating state through method calls.

## Target State (Redux Pattern)

```rust
// Future: In handle_input()
let action = self.key_mapper.map_key(key, &context);
if let Some(action) = action {
    self.store.dispatch(action);  // Just dispatch, don't handle
}

// Somewhere else - the Store
impl Store {
    fn dispatch(&mut self, action: Action) {
        let old_state = self.state.clone();
        let new_state = reducer(&old_state, action);
        self.state = new_state;
        self.notify_subscribers();
    }
}

// Pure reducer function
fn reducer(state: &AppState, action: Action) -> AppState {
    match action {
        Action::Navigate(NavigateAction::Down(n)) => {
            let mut new_state = state.clone();
            new_state.navigation.selected_row += n;
            new_state
        }
        // ... handle all actions
    }
}

// TUI subscribes to state changes
impl EnhancedTuiApp {
    fn on_state_change(&mut self, new_state: &AppState) {
        // Update UI based on new state
        self.render();
    }
}
```

## Benefits of This Architecture

1. **Pure Functions**: Reducers are pure - same input always produces same output
2. **Testable**: Can test state transitions without UI
3. **Time Travel**: Can implement undo/redo by storing state history
4. **Debugging**: Can log every action and state change
5. **Predictable**: State changes only through actions
6. **Decoupled**: UI just subscribes to state, doesn't manage it

## Migration Path

### Phase 1: Create State and Store (Current Branch Status)
- ✅ Action enum exists
- ✅ KeyMapper maps keys to actions
- ✅ Actions are being dispatched (but handled directly)

### Phase 2: Create Central State Structure
```rust
struct AppState {
    navigation: NavigationState,
    mode: AppMode,
    selection_mode: SelectionMode,
    buffer: BufferState,
    search: SearchState,
    filter: FilterState,
    // ... all app state in one place
}
```

### Phase 3: Create Reducer
```rust
fn reducer(state: &AppState, action: Action) -> AppState {
    // Pure function that returns new state
}
```

### Phase 4: Create Store
```rust
struct Store {
    state: AppState,
    reducer: fn(&AppState, Action) -> AppState,
    subscribers: Vec<Box<dyn Subscriber>>,
}

impl Store {
    fn dispatch(&mut self, action: Action) {
        let new_state = (self.reducer)(&self.state, action);
        if new_state != self.state {
            self.state = new_state;
            self.notify_subscribers();
        }
    }
}
```

### Phase 5: Make TUI a Subscriber
```rust
impl Subscriber for EnhancedTuiApp {
    fn on_state_change(&mut self, state: &AppState) {
        // Update internal references
        // Trigger re-render
    }
}
```

## Example: Navigation in Redux Style

### Current (Imperative)
```rust
fn next_row(&mut self) {
    let nav = self.state_container.navigation_mut();
    nav.selected_row += 1;
    self.buffer_mut().set_selected_row(nav.selected_row);
    self.update_viewport();
}
```

### Future (Redux)
```rust
// Just dispatch the action
store.dispatch(Action::Navigate(NavigateAction::Down(1)));

// Reducer handles it purely
fn reducer(state: &AppState, action: Action) -> AppState {
    match action {
        Action::Navigate(NavigateAction::Down(n)) => {
            AppState {
                navigation: NavigationState {
                    selected_row: min(
                        state.navigation.selected_row + n,
                        state.data.row_count - 1
                    ),
                    ..state.navigation
                },
                ..state
            }
        }
        _ => state
    }
}
```

## Middleware Opportunities

With Redux, we can add middleware:

```rust
// Logging middleware
fn logging_middleware(action: &Action, state: &AppState) {
    debug!("Action: {:?}", action);
    debug!("State before: {:?}", state);
}

// Async middleware for API calls
fn api_middleware(action: &Action) -> Option<Future<Action>> {
    match action {
        Action::ExecuteQuery(sql) => {
            Some(async { 
                let result = api_client.query(sql).await;
                Action::QueryComplete(result)
            })
        }
        _ => None
    }
}

// Undo/Redo middleware
fn history_middleware(action: &Action, history: &mut StateHistory) {
    if action.is_undoable() {
        history.push(state.clone());
    }
}
```

## Testing Benefits

```rust
#[test]
fn test_navigation() {
    let initial_state = AppState::default();
    
    // Test moving down
    let new_state = reducer(&initial_state, Action::Navigate(NavigateAction::Down(5)));
    assert_eq!(new_state.navigation.selected_row, 5);
    
    // Test boundary
    let state_at_end = AppState {
        navigation: NavigationState { selected_row: 99, ..default() },
        data: DataState { row_count: 100, ..default() },
        ..default()
    };
    let new_state = reducer(&state_at_end, Action::Navigate(NavigateAction::Down(5)));
    assert_eq!(new_state.navigation.selected_row, 99); // Clamped at boundary
}
```

## Timeline

1. **Now**: Action system integrated, handling actions imperatively
2. **Next**: Create AppState structure consolidating all state
3. **Then**: Create reducer function for pure state transitions
4. **Then**: Create Store with dispatch and subscribe
5. **Finally**: Convert TUI to subscriber model

This will make the codebase much more maintainable and testable!