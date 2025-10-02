# Data-View-Viewport Synchronization Issues and Solutions

## The Problem

We have a classic synchronization issue between three interconnected systems:
- **Data**: The actual query results (rows and columns)
- **View**: What subset of data is visible (filters, sorting)
- **Viewport**: What portion of the view fits on screen

### Current Symptoms
- Cursor disappears after complex navigation sequences
- Selected row shows as `None` while NavigationState shows row 21
- Viewport gets misaligned after filter operations
- "Pressing 'g' fixes it" - a reset to known state helps

### Example Failure Sequence
1. Query data (10,000 rows)
2. Navigate around (row 21, column 5)
3. Apply fuzzy filter (reduces to 100 visible rows)
4. Remove filter (back to 10,000 rows)
5. Switch between Edit and Results modes
6. **Result**: Cursor lost, viewport misaligned, selection state inconsistent

## Root Cause Analysis

### State Scattered Across Multiple Systems

```
Current State Distribution:
├── NavigationState (AppStateContainer)
│   ├── selected_row: 21
│   ├── selected_column: 0
│   └── scroll_offset: (0, 0)
├── Buffer
│   ├── table_state: None (!)
│   ├── current_column: 0
│   └── scroll_offset: (0, 0)
├── SelectionState (AppStateContainer)
│   ├── selected_row: Some(21)
│   └── mode: Cell
└── FilterState
    └── Can change what rows mean
```

### The Core Issue
Different parts of the system have different ideas about the current state:
- **NavigationState** thinks we're at row 21
- **Buffer's table_state** thinks we have no selection
- **SelectionState** has its own idea
- No single source of truth

## Why 'g' (Go to Top) Fixes It

The 'g' command works as a band-aid because it:
1. Sets selection to row 0 (known position)
2. Sets viewport scroll to (0,0) (known viewport)
3. Forces all systems to resync from this known state
4. Essentially a "reset to factory defaults" for navigation

## The Solution: DataTable/DataView Pattern

### Phase 1: Separate Data from View (DataTable/DataView)

```rust
// Immutable data container
struct DataTable {
    rows: Vec<Row>,        // Never changes after query
    columns: Vec<Column>,  // Never changes
    query: String,         // Original query that produced this
}

// Mutable view on the data
struct DataView {
    table: Arc<DataTable>,      // Shared, immutable reference
    
    // View-specific state
    filter: Option<Filter>,     // What rows are visible
    sort: Option<Sort>,         // How rows are ordered
    
    // These MUST stay synchronized
    visible_rows: Vec<usize>,   // Indices into table.rows
    selection: Selection,       // ALWAYS relative to visible_rows
    viewport: Viewport,         // ALWAYS relative to visible_rows
}
```

### Key Insight
**Selection and viewport are properties of the VIEW, not the data**

When filter changes:
- `visible_rows` updates
- `selection` must be revalidated
- `viewport` must be adjusted

All three MUST update together or we get desync.

### Phase 2: Redux/Reducer Pattern for State Consistency

Every state change goes through a reducer that maintains invariants:

```rust
enum Action {
    SetData(DataTable),
    ApplyFilter(String),
    RemoveFilter,
    NavigateToRow(usize),
    SortByColumn(usize),
    ToggleViewportLock,
}

fn reducer(state: AppState, action: Action) -> AppState {
    match action {
        Action::RemoveFilter => {
            let mut new_state = state.clone();
            
            // 1. Update the filter
            new_state.view.filter = None;
            
            // 2. MUST recompute visible rows
            new_state.view.visible_rows = compute_visible_rows(&new_state);
            
            // 3. MUST validate selection still makes sense
            new_state.view.selection = validate_selection(
                &new_state.view.selection,
                &new_state.view.visible_rows
            );
            
            // 4. MUST adjust viewport to keep selection visible
            new_state.view.viewport = ensure_selection_visible(
                &new_state.view.selection,
                &new_state.view.viewport,
                &new_state.view.visible_rows
            );
            
            new_state
        }
        // ... other actions follow same pattern
    }
}
```

### Invariants That Must Be Maintained

1. **Selection Invariant**: Selected row must be within visible_rows
2. **Viewport Invariant**: If selection exists, it should be visible in viewport
3. **Filter Invariant**: visible_rows must match current filter
4. **Sort Invariant**: visible_rows order must match current sort

## Immediate Mitigations (Before Full Refactor)

### 1. Add State Synchronization Points

```rust
impl EnhancedTuiApp {
    /// Call after any operation that might desync state
    fn sync_all_state(&mut self) {
        let (row, col) = self.state_container.get_current_position();
        
        // Sync buffer's idea of selection
        self.buffer_mut().set_selected_row(Some(row));
        self.buffer_mut().set_current_column(col);
        
        // Ensure viewport contains selection
        self.ensure_selection_visible(row, col);
        
        // Sync selection state with navigation
        self.state_container.sync_selection_with_navigation();
    }
}
```

### 2. Add State Validation

```rust
#[derive(Debug)]
enum StateIssue {
    SelectionMismatch { nav: usize, buffer: Option<usize> },
    SelectionNotVisible { row: usize, viewport: (usize, usize) },
    FilterStateMismatch,
}

fn validate_state(&self) -> Vec<StateIssue> {
    let mut issues = vec![];
    
    // Check selection consistency
    let nav_row = self.state_container.navigation().selected_row;
    let buffer_row = self.buffer().get_selected_row();
    
    if Some(nav_row) != buffer_row {
        issues.push(StateIssue::SelectionMismatch {
            nav: nav_row,
            buffer: buffer_row,
        });
    }
    
    // Check if selection is visible
    let viewport = self.buffer().get_viewport_range();
    if !viewport.contains(nav_row) {
        issues.push(StateIssue::SelectionNotVisible {
            row: nav_row,
            viewport,
        });
    }
    
    issues
}
```

### 3. Call Sync After Key Operations

- After removing filters
- After changing modes
- After sorting
- After applying filters
- After window resize

## Implementation Timeline

### Already Complete ✅
- V1-V26: Basic state migration to AppStateContainer
- V27: SelectionState consolidation

### Remaining AppStateContainer Work
- V28: ClipboardState
- V29: ColumnSearchState verification
- V30: ChordState
- V31: Final cleanup

### Next Major Phase: DataTable/DataView
**Goal**: Separate immutable data from mutable view state
- Create DataTable structure for immutable query results
- Create DataView for filters/sorting/selection
- Migrate current buffer system to use DataTable/DataView
- Ensure all view operations maintain synchronization

### Final Phase: Redux-Style State Management
**Goal**: Guarantee state consistency through controlled mutations
- Implement Action enum for all state changes
- Create reducer function with invariant maintenance
- Add middleware for logging/debugging state changes
- Optional: Time-travel debugging

## Success Metrics

1. **No more "lost cursor"** - Selection always visible and consistent
2. **No more 'g' workarounds** - State stays synchronized
3. **Predictable behavior** - Same action = same result
4. **Debuggable** - Can trace exactly how state got into any configuration

## Testing Strategy

### Chaos Testing Sequence
```
1. Load large dataset (10k+ rows)
2. Navigate to middle (row 5000)
3. Apply filter (reduces to 100 rows)
4. Sort by column
5. Navigate in filtered view
6. Remove filter
7. Verify: Cursor visible, selection consistent, viewport correct
8. Apply fuzzy filter
9. Change modes (Edit → Results → Command)
10. Verify: State remains consistent
```

### Invariant Testing
After each operation, verify:
- `assert!(buffer.selected_row == navigation.selected_row)`
- `assert!(viewport.contains(selection))`
- `assert!(visible_rows.len() matches filter expectation)`

## Conclusion

The current issues stem from managing interconnected state in multiple places without ensuring they stay synchronized. The solution is a combination of:

1. **Better state organization** (DataTable/DataView)
2. **Controlled state mutations** (Redux/Reducer)
3. **Invariant maintenance** (Always sync related state together)

This will transform our "mostly works but sometimes doesn't" navigation into a rock-solid system where state consistency is guaranteed by design rather than hoped for.

## References

- [C# DataTable/DataView Pattern](https://docs.microsoft.com/en-us/dotnet/api/system.data.dataview)
- [Redux Principles](https://redux.js.org/understanding/thinking-in-redux/three-principles)
- [Elm Architecture](https://guide.elm-lang.org/architecture/) - Similar pattern in pure functional style