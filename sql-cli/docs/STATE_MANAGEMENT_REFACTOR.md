# State Management Refactoring Strategy

## Vision
Transform the application from scattered state management to a centralized, React-like state store where:
- All state lives in `AppStateContainer` (the "Store")
- TUI components only interact with state through the Store API
- Components can subscribe to state slices they care about
- All state mutations are logged and traceable
- State changes trigger automatic re-renders of affected components

## Current Architecture Problems
1. **State is scattered** across EnhancedTuiApp, Buffer, and various widgets
2. **No clear data flow** - components directly modify state they don't own
3. **Debugging is difficult** - state changes happen in many places
4. **Testing is hard** - components are tightly coupled to implementation

## Target Architecture (React-like Pattern)

```
┌─────────────────────────────────────────┐
│          AppStateContainer              │
│            (The Store)                  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │        State Slices:              │  │
│  │  - Input State                    │  │
│  │  - Search/Filter State            │  │
│  │  - Navigation State               │  │
│  │  - Buffer/Results State           │  │
│  │  - Mode State                     │  │
│  │  - Widget States                  │  │
│  └──────────────────────────────────┘  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │        Actions/Mutations:         │  │
│  │  - All state changes go through   │  │
│  │    defined methods                │  │
│  │  - Each mutation is logged        │  │
│  │  - Validation before changes      │  │
│  └──────────────────────────────────┘  │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │        Subscriptions:             │  │
│  │  - Components register interest   │  │
│  │  - Notified on relevant changes   │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
                    ↑ ↓
        ┌───────────────────────┐
        │   TUI Components       │
        │  (Pure View Layer)     │
        └───────────────────────┘
```

## Refactoring Phases

### Phase 1: Complete State Consolidation ✅ (v10-v13)
- [x] Create AppStateContainer structure
- [x] Add debug service integration
- [x] Implement key press history with coalescing
- [x] Add platform-aware key normalization
- [x] Basic state logging infrastructure

### Phase 2: Search/Filter State (v14) 🚧
**Goal**: Move all search/filter logic into AppStateContainer

#### Current State (Scattered):
```rust
// In EnhancedTuiApp:
search_pattern: String,
search_matches: Vec<(usize, usize)>,
current_search_match: Option<usize>,
filter_pattern: String,
filtered_rows: Vec<usize>,
column_search_pattern: String,
column_search_matches: Vec<(usize, String)>,
```

#### Target State (Consolidated):
```rust
// In AppStateContainer:
pub struct SearchState {
    pattern: String,
    matches: Vec<SearchMatch>,
    current_match: usize,
    history: VecDeque<SearchQuery>,
    search_type: SearchType,
}

pub struct FilterState {
    pattern: String,
    filtered_indices: Vec<usize>,
    is_active: bool,
    case_insensitive: bool,
    history: VecDeque<FilterQuery>,
}
```

#### Actions to Implement:
- `search_in_results(pattern: &str) -> SearchResult`
- `apply_filter(pattern: &str) -> FilterResult`
- `clear_search()`
- `clear_filter()`
- `next_search_match()`
- `previous_search_match()`

#### Logging Points:
- Pattern changes with old → new
- Match count changes
- Navigation through matches
- Filter application/removal
- Performance metrics (search time, rows filtered)

### Phase 3: History Search State (v15) ✅
**Goal**: Migrate Ctrl+R history search to AppStateContainer

**Completed**:
- Added HistorySearchState to AppStateContainer
- Implemented history search operations (start, update, navigate, accept, cancel)
- Integrated with fuzzy matching using SkimMatcherV2
- Updated enhanced_tui to use AppStateContainer for history search
- Added comprehensive logging for all history search operations
- Tested with existing history data

### Phase 4: Navigation State (v16) ✅
**Goal**: Consolidate all table navigation and viewport management

#### Current State (Scattered):
```rust
// In EnhancedTuiApp:
table_state: TableState,
selected_row: Option<usize>,
selected_column: usize,
scroll_offset: (usize, usize),
last_visible_rows: usize,
selection_mode: SelectionMode,
```

#### Target State (Consolidated):
```rust
pub struct NavigationState {
    viewport: Viewport,
    selection: Selection,
    scroll_position: ScrollPosition,
    navigation_history: VecDeque<NavigationEntry>,
}

pub struct Viewport {
    visible_rows: Range<usize>,
    visible_columns: Range<usize>,
    total_rows: usize,
    total_columns: usize,
}
```

#### Actions to Implement:
- `navigate_to(row: usize, col: usize)`
- `scroll_by(delta_row: i32, delta_col: i32)`
- `select_cell(row: usize, col: usize)`
- `expand_selection(direction: Direction)`
- `jump_to_row(row: usize)`
- `jump_to_column(col: usize)`

### Phase 5: Buffer/Results State (v17) ✅
**Goal**: Move all query results data into AppStateContainer

**Completed**:
- Created comprehensive ResultsState structure with performance tracking
- Implemented LRU cache with memory management and statistics
- Added CachedResult and QueryPerformance tracking structures
- Integrated results state with comprehensive debug logging
- Updated enhanced_tui to use AppStateContainer for results and caching
- Added performance metrics tracking (execution time, cache hit rate, memory usage)
- Implemented smart cache eviction based on memory limits and LRU policy

#### Implemented State:
```rust
pub struct ResultsState {
    current_results: Option<QueryResponse>,
    results_cache: HashMap<String, CachedResult>,
    max_cache_size: usize,
    total_memory_usage: usize,
    memory_limit: usize,
    last_query: String,
    last_execution_time: Duration,
    query_performance_history: VecDeque<QueryPerformance>,
    from_cache: bool,
    last_modified: Instant,
}

pub struct CachedResult {
    response: QueryResponse,
    cached_at: Instant,
    access_count: u32,
    last_access: Instant,
    memory_size: usize,
}

pub struct QueryPerformance {
    query: String,
    execution_time: Duration,
    row_count: usize,
    from_cache: bool,
    memory_usage: usize,
    executed_at: Instant,
}
```

### Phase 6: Input State Completion (v18) 📋
**Goal**: Complete input management migration with undo/redo

#### Target State:
```rust
pub struct InputState {
    text: String,
    cursor_position: usize,
    selection: Option<Range<usize>>,
    undo_stack: Vec<InputSnapshot>,
    redo_stack: Vec<InputSnapshot>,
    input_history: VecDeque<String>,
}
```

### Phase 7: Subscription System (v19) 📋
**Goal**: Implement component subscription to state slices

#### Concept:
```rust
pub trait StateListener {
    fn on_state_change(&mut self, change: StateChange);
}

pub enum StateChange {
    SearchUpdated(SearchState),
    FilterUpdated(FilterState),
    NavigationUpdated(NavigationState),
    ResultsUpdated(ResultsState),
    ModeChanged(AppMode),
}

impl AppStateContainer {
    pub fn subscribe(&mut self, listener: Box<dyn StateListener>, slices: Vec<StateSlice>);
    pub fn notify_listeners(&self, change: StateChange);
}
```

### Phase 8: Widget State Binding (v20) 📋
**Goal**: Widgets automatically update from state changes

```rust
impl Widget for SearchModesWidget {
    fn bind_to_state(&mut self, state: Arc<AppStateContainer>) {
        self.state = state;
    }
    
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let search_state = self.state.search();
        // Render based on current state
    }
}
```

## Benefits of This Approach

1. **Single Source of Truth**: All state in one place
2. **Predictable State Updates**: All changes go through defined actions
3. **Complete Audit Trail**: Every state change is logged
4. **Easier Testing**: Pure functions and mockable state
5. **Better Performance**: Can optimize re-renders based on what changed
6. **Cleaner Code**: Separation of concerns between state and UI
7. **Time Travel Debugging**: With complete history, can replay state changes

## Implementation Guidelines

### For Each State Migration:
1. Define the state structure in AppStateContainer
2. Create action methods for all mutations
3. Add comprehensive logging at each mutation point
4. Update TUI to use new API
5. Remove old state from TUI
6. Add tests for state transitions
7. Document the state slice

### Logging Requirements:
- Log all state mutations with before/after values
- Include timestamp and caller context
- Track performance metrics where relevant
- Make logs queryable/filterable

### Testing Strategy:
- Unit test each state slice independently
- Test state transitions and validations
- Test subscription notifications
- Integration test TUI → State interactions

## Success Metrics
- [ ] TUI has no direct state (only references AppStateContainer)
- [ ] All state changes are logged
- [ ] State can be serialized/deserialized
- [ ] Can replay user session from logs
- [ ] Widgets update automatically on state changes
- [ ] Performance improves (fewer unnecessary re-renders)

## Next Steps (v14)
1. Create new branch `refactor-v14-search-filter-state`
2. Implement SearchState with full operations
3. Add comprehensive logging for search operations
4. Migrate EnhancedTuiApp to use new search API
5. Remove old search state from TUI
6. Test on Linux and Windows
7. Document search state slice

## Long-term Vision
Eventually, the TUI becomes a thin layer that:
- Handles key events → dispatches actions to Store
- Renders current state → pure function of State
- Has no business logic → all in Store
- Can be swapped out → for different UI frameworks

This architecture would allow:
- Alternative frontends (web UI, native GUI)
- State persistence and replay
- Collaborative features (shared state)
- Undo/redo for any operation
- Time-travel debugging