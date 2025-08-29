# State Duplication Analysis: Buffer vs AppStateContainer

## Current Architecture Problem
We have significant state duplication between Buffer and AppStateContainer. Now that Arc is removed, we can make AppStateContainer a thin proxy that delegates to the Buffer.

## Duplicated State Fields

### 1. Navigation State
**Buffer has:**
- `selected_row: Option<usize>`
- `current_column: usize`
- `scroll_offset: (usize, usize)`
- `last_results_row: Option<usize>`
- `last_scroll_offset: (usize, usize)`

**AppStateContainer.navigation has:**
- `selected_row: usize`
- `selected_column: usize`
- `scroll_offset: (usize, usize)`
- `viewport_lock: bool`
- `viewport_lock_row: Option<usize>`

**Resolution:** Keep in Buffer, AppStateContainer delegates

### 2. Search State
**Buffer has:**
- `search_state.pattern: String`
- `search_state.matches: Vec<(usize, usize)>`
- `search_state.current_match: Option<(usize, usize)>`
- `search_state.match_index: usize`

**AppStateContainer.search has:**
- `pattern: String`
- `matches: Vec<(usize, usize, usize, usize)>` (extended format)
- `current_match: usize`
- `is_active: bool`

**Resolution:** Keep in Buffer, extend Buffer's format if needed

### 3. Filter State
**Buffer has:**
- `filter_pattern: String`
- `filter_active: bool`
- `fuzzy_filter_pattern: String`
- `fuzzy_filter_active: bool`
- `fuzzy_filter_indices: Vec<usize>`

**AppStateContainer.filter has:**
- `pattern: String`
- `filtered_indices: Vec<usize>`
- `filtered_data: Option<Vec<Vec<String>>>`
- `is_active: bool`
- `case_insensitive: bool`

**Resolution:** Keep in Buffer, merge fuzzy and regular filter

### 4. Sort State
**Buffer has:**
- `sort_state.column: Option<usize>`
- `sort_state.order: SortOrder`

**AppStateContainer.sort has:**
- `column: Option<usize>`
- `column_name: Option<String>`
- `order: SortOrder`

**Resolution:** Keep in Buffer, add column_name if needed

### 5. Display Options
**Buffer has:**
- `compact_mode: bool`
- `show_row_numbers: bool`
- `viewport_lock: bool`
- `viewport_lock_row: Option<usize>`
- `case_insensitive: bool`

**AppStateContainer has:**
- Various display-related fields scattered

**Resolution:** Keep in Buffer

## Proposed Architecture

### Phase 1: Create Delegation Methods
```rust
impl AppStateContainer {
    // Navigation delegation
    pub fn selected_row(&self) -> Option<usize> {
        self.current_buffer()?.get_selected_row()
    }
    
    pub fn set_selected_row(&mut self, row: Option<usize>) {
        if let Some(buffer) = self.current_buffer_mut() {
            buffer.set_selected_row(row);
        }
    }
    
    // Search delegation
    pub fn search_pattern(&self) -> String {
        self.current_buffer()
            .map(|b| b.get_search_pattern())
            .unwrap_or_default()
    }
    
    // Continue for all duplicated fields...
}
```

### Phase 2: Remove Duplicate Fields
1. Remove NavigationState struct from AppStateContainer
2. Remove SearchState struct from AppStateContainer  
3. Remove FilterState struct from AppStateContainer
4. Remove SortState struct from AppStateContainer
5. Keep only non-duplicated state like:
   - BufferManager
   - History management
   - Column operations
   - Schema/parser state

### Phase 3: Update All References
Update all code that accesses `state_container.navigation()`, `state_container.search()`, etc. to use the new delegation methods.

## Benefits
1. **Single Source of Truth**: No more state synchronization issues
2. **Reduced Memory**: No duplicate data structures
3. **Simpler Code**: No need to keep two states in sync
4. **Better Encapsulation**: Buffer owns its state completely
5. **Easier Testing**: Test Buffer in isolation

## Implementation Order
1. Start with NavigationState (most used)
2. Then SearchState
3. Then FilterState  
4. Then SortState
5. Finally clean up remaining duplications

## Risks
- Large number of call sites to update
- Need to ensure Buffer is always available when accessing state
- Some state might legitimately belong in AppStateContainer (e.g., history)