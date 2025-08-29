# State Delegation Migration Plan

## Current Situation
- AppStateContainer has multiple state structs: NavigationState, SelectionState, SearchState, FilterState, SortState
- Buffer has duplicate fields for most of these
- Currently they are NOT synchronized - changes to one don't reflect in the other

## Migration Strategy

### Phase 1: Add Synchronization (Current Step)
Instead of immediately removing state and breaking everything, first ensure state stays synchronized:

1. ✅ Created delegation methods with `delegated_` prefix
2. **TODO**: Modify existing setter methods to update BOTH places
3. **TODO**: Add tests to verify synchronization works

### Phase 2: Gradual Migration
Replace internal usage one system at a time:

1. Start with less critical systems (e.g., sort state)
2. Update internal methods to use Buffer state
3. Keep public API unchanged initially
4. Verify with tests at each step

### Phase 3: Public API Migration
Once internal usage is migrated:

1. Deprecate old methods
2. Point them to delegation methods
3. Update all callers
4. Remove deprecated methods

### Phase 4: Remove Duplicate State
Finally remove the duplicate state structs:

1. Remove NavigationState from AppStateContainer
2. Remove SelectionState from AppStateContainer
3. Remove SearchState from AppStateContainer
4. Remove FilterState from AppStateContainer
5. Remove SortState from AppStateContainer

## Key Methods to Migrate

### Navigation/Selection
- `get_selected_row()` -> delegates to Buffer
- `set_table_selected_row()` -> updates Buffer
- `get_selected_column()` / `get_current_column()` -> delegates to Buffer
- `set_current_column()` -> updates Buffer
- `get_current_position()` -> uses Buffer

### Search
- `start_search()` -> updates Buffer
- `perform_search()` -> uses Buffer state
- `clear_search()` -> clears Buffer state

### Filter
- `set_filter()` -> updates Buffer
- `clear_filter()` -> clears Buffer
- `is_filter_active()` -> checks Buffer

### Sort
- `sort_by_column()` -> updates Buffer
- `clear_sort()` -> clears Buffer
- `get_sort_state()` -> reads from Buffer

## Benefits of Gradual Approach
1. No breaking changes initially
2. Can test synchronization at each step
3. Can rollback if issues found
4. Maintains working system throughout migration