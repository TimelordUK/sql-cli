# Unified State Architecture - Buffer-Centric Redux Pattern

## Current State Mess
We have THREE competing state systems:
1. **AppStateContainer** - Holds DataView, filters, column state, etc.
2. **ShadowStateManager** - Observes/coordinates mode transitions
3. **Buffer** - Should eventually hold ALL state for buffer switching

This is unsustainable and will lead to more synchronization bugs.

## The Goal: Buffer-Centric State
When we switch buffers, we should restore EVERYTHING:
- Query text and cursor position
- DataView with current query results  
- All filters and search states
- Column visibility, sorting, pinning
- Viewport position and locks
- Current mode
- Search patterns and matches

## Proposed Architecture

### 1. Buffer Becomes the Redux Store
```rust
pub struct Buffer {
    // Core state slices
    pub query_state: QueryState,
    pub data_state: DataState,        // DataView lives here
    pub ui_state: UIState,            // Mode, viewport, etc.
    pub search_state: SearchState,    // All search types
    pub column_state: ColumnState,    // Hide, pin, sort
    
    // State version for change detection
    version: u64,
    
    // Subscribers interested in state changes
    subscribers: Vec<StateSubscriberId>,
}
```

### 2. AppStateContainer Becomes the Dispatcher
Instead of holding state, it coordinates:
```rust
pub struct AppStateContainer {
    // Reference to current buffer
    current_buffer: Rc<RefCell<Buffer>>,
    
    // All available buffers
    buffers: HashMap<BufferId, Rc<RefCell<Buffer>>>,
    
    // The reducer that processes actions
    reducer: StateReducer,
    
    // Component registry for side effects
    components: ComponentRegistry,
}

impl AppStateContainer {
    pub fn dispatch(&mut self, action: Action) -> Result<()> {
        // 1. Get current buffer state
        let buffer = self.current_buffer.borrow();
        
        // 2. Process action through reducer (pure function)
        let (new_state, side_effects) = self.reducer.process(&buffer, action);
        
        // 3. Update buffer with new state
        buffer.apply_state_change(new_state);
        
        // 4. Notify components about side effects
        for effect in side_effects {
            self.components.handle_effect(effect);
        }
        
        Ok(())
    }
}
```

### 3. Shadow State Becomes the Reducer
Transform shadow state into a pure function reducer:
```rust
pub struct StateReducer {
    // No state! Just logic
}

impl StateReducer {
    pub fn process(&self, state: &Buffer, action: Action) 
        -> (StateChange, Vec<SideEffect>) {
        
        match action {
            Action::ExitSearchMode => {
                let change = StateChange {
                    ui_state: Some(UIState { mode: AppMode::Results }),
                    search_state: Some(SearchState::cleared()),
                    ..Default::default()
                };
                
                let effects = vec![
                    SideEffect::RestoreNavigationKeys,
                    SideEffect::ClearSearchHighlights,
                ];
                
                (change, effects)
            }
            // ... other actions
        }
    }
}
```

### 4. Avoiding Clones with Smart References

#### Option A: State Versioning
```rust
pub struct DataState {
    data_view: Arc<DataView>,  // Immutable, shared
    version: u64,               // Increment on change
}

// Components keep version and check if stale
pub struct TableRenderer {
    data_version: u64,
    cached_render: Option<RenderOutput>,
}

impl TableRenderer {
    fn render(&mut self, data_state: &DataState) -> RenderOutput {
        if self.data_version != data_state.version {
            // Re-render only if data changed
            self.cached_render = Some(self.do_render(&data_state.data_view));
            self.data_version = data_state.version;
        }
        self.cached_render.clone().unwrap()
    }
}
```

#### Option B: Slice References
```rust
// Components get references to specific slices they care about
pub trait Component {
    type StateSlice;
    
    fn get_slice<'a>(&self, buffer: &'a Buffer) -> &'a Self::StateSlice;
    fn handle_change(&mut self, old: &Self::StateSlice, new: &Self::StateSlice);
}

impl Component for VimSearchManager {
    type StateSlice = SearchState;
    
    fn get_slice<'a>(&self, buffer: &'a Buffer) -> &'a SearchState {
        &buffer.search_state
    }
    
    fn handle_change(&mut self, old: &SearchState, new: &SearchState) {
        if old.vim_pattern != new.vim_pattern {
            self.update_pattern(&new.vim_pattern);
        }
    }
}
```

### 5. Pub-Sub Without Cloning

Use indices and weak references:
```rust
pub struct EventBus {
    // Events are just indices into a ring buffer
    events: RingBuffer<Event>,
    
    // Subscribers get notified with event index
    subscribers: HashMap<EventType, Vec<SubscriberId>>,
}

pub struct Event {
    action: Action,
    state_before: StateSnapshot,  // Just version numbers
    state_after: StateSnapshot,
}

pub struct StateSnapshot {
    query_version: u64,
    data_version: u64,
    ui_version: u64,
    // ... other version numbers
}

// Components check if they care about the change
impl VimSearchManager {
    fn on_event(&mut self, event_id: EventId, bus: &EventBus) {
        let event = bus.get_event(event_id);
        
        // Only react if search state changed
        if event.state_before.search_version != event.state_after.search_version {
            self.handle_search_change();
        }
    }
}
```

## Migration Path

### Phase 1: Move DataView to Buffer
```rust
// Move from AppStateContainer to Buffer
impl Buffer {
    pub fn get_data_view(&self) -> &DataView {
        &self.data_state.data_view
    }
}
```

### Phase 2: Consolidate Search State
```rust
// Unify all search types in Buffer
pub struct SearchState {
    vim_search: Option<VimSearchData>,
    column_search: Option<ColumnSearchData>,
    fuzzy_filter: Option<String>,
    data_filter: Option<String>,
}
```

### Phase 3: Make AppStateContainer a Dispatcher
- Remove all state storage from AppStateContainer
- Add dispatch() method
- Convert all mutations to actions

### Phase 4: Implement State Versioning
- Add version numbers to each state slice
- Components cache based on version
- Only re-compute when version changes

## Benefits

1. **Single Source of Truth**: Everything in Buffer
2. **Buffer Switching Works**: Can save/restore entire UI state
3. **No Redundant Clones**: Version checking and smart references
4. **Clear Data Flow**: Action → Reducer → State → Side Effects
5. **Testable**: Pure reducer functions
6. **Debuggable**: Can replay actions

## Specific Solution for N Key Issue

With this architecture:
```rust
// User presses Escape in search mode
dispatch(Action::ExitSearchMode)
  → Reducer returns (StateChange { mode: Results }, [ClearSearches])
  → Buffer updates search_state
  → VimSearchManager notified via side effect
  → VimSearchManager clears itself
  → N key now works properly
```

## Next Steps

1. **Don't add more state systems** - We have enough!
2. **Start moving state to Buffer** - DataView first
3. **Convert ShadowStateManager to Reducer** - Pure functions
4. **Make AppStateContainer a dispatcher** - Not a state holder
5. **Implement versioning** - Avoid unnecessary clones

## Key Principles

1. **Buffer owns all state** - It's the Redux store
2. **AppStateContainer dispatches** - It's the event bus
3. **Reducer is pure** - No state, just logic
4. **Components subscribe to slices** - Not whole state
5. **Use versions not clones** - Check if changed before reacting