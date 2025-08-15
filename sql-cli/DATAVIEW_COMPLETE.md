# DataView - Complete View State Management

## Overview
DataView is now the single source of truth for ALL view operations, completely encapsulating view state management away from the TUI.

## Complete Feature Set

### 1. Row Filtering
- **Text Filter**: `apply_text_filter(pattern, case_sensitive)`
  - Case-sensitive/insensitive substring matching
  - Searches across all cell values in a row
  - Maintains filter pattern for restoration
  
- **Fuzzy Filter**: `apply_fuzzy_filter(pattern, case_insensitive)`
  - Fuzzy matching using SkimMatcherV2
  - Exact mode with `'` prefix
  - Scores and filters based on match quality

- **Clear/Restore**: `clear_filter()`
  - Restores to `base_rows` (preserves sort order)
  - Filter pattern tracking for reapplication

### 2. Sorting
- **Apply Sort**: `apply_sort(column_index, ascending)`
  - Sorts visible rows by column
  - Updates `base_rows` to preserve sort through filter changes
  - Type-aware comparison (Integer, Float, String, Boolean, DateTime)

- **Clear Sort**: `clear_sort()`
  - Restores original row order
  - Reapplies any active filter

### 3. Column Visibility
- **Hide Columns**: 
  - `hide_column(index)` - Hide by index
  - `hide_column_by_name(name)` - Hide by name
  - `hide_all_columns()` - Hide all

- **Show Columns**:
  - `unhide_all_columns()` - Restore to base columns
  - Tracks `base_columns` for original projection

- **Column Reordering**:
  - `move_column_left(index)` - With wraparound
  - `move_column_right(index)` - With wraparound
  - Also available by name

- **Query Methods**:
  - `has_hidden_columns()`
  - `get_hidden_column_names()`
  - `is_column_visible(index)`

### 4. Column Search (NEW!)
- **Search Operations**:
  - `search_columns(pattern)` - Search column names
  - `clear_column_search()` - Clear search
  - Case-insensitive substring matching

- **Navigation**:
  - `next_column_match()` - Go to next match
  - `prev_column_match()` - Go to previous match
  - Circular navigation (wraps around)

- **Query Methods**:
  - `column_search_pattern()` - Get current pattern
  - `get_matching_columns()` - Get all matches
  - `get_current_column_match()` - Get current match
  - `has_column_search()` - Check if active

### 5. Export Capabilities
- **JSON Export**: `to_json()`
  - Exports visible data as JSON array
  - Preserves data types

- **CSV Export**: `to_csv()`
  - Proper escaping for commas, quotes, newlines
  - Headers included

- **TSV Export**: `to_tsv()`
  - Tab-separated for Excel compatibility
  - Clean format without escaping

### 6. Pagination
- **Limit/Offset**: `with_limit(limit, offset)`
  - Virtual pagination over visible rows
  - Transparent to all operations

### 7. Data Access
- **Row Access**:
  - `get_row(index)` - Get single row
  - `get_rows()` - Get all visible rows
  - `row_count()` - Count of visible rows

- **Column Access**:
  - `column_names()` - Names of visible columns
  - `column_count()` - Count of visible columns
  - `visible_column_indices()` - Raw indices

- **Source Access**:
  - `source()` - Get underlying DataTable (immutable)

## Architecture Benefits

### Single Source of Truth
```rust
// All view state in one place
pub struct DataView {
    source: Arc<DataTable>,           // Immutable data
    visible_rows: Vec<usize>,         // Current visible rows
    visible_columns: Vec<usize>,      // Current visible columns
    base_rows: Vec<usize>,            // Preserved through filters
    base_columns: Vec<usize>,         // Original projection
    filter_pattern: Option<String>,   // Active filter
    column_search_pattern: Option<String>,  // Column search
    matching_columns: Vec<(usize, String)>, // Search results
    current_column_match: usize,      // Current selection
    // ... pagination
}
```

### Clean Separation
- **DataTable**: Immutable data storage
- **DataView**: All view operations and state
- **TUI**: Pure presentation, no data logic

### Performance
- Arc-based sharing (no data copying)
- Index-based operations (O(1) access)
- Lazy evaluation where possible

### Testability
- DataView can be tested in isolation
- No UI dependencies
- Clear input/output semantics

## Migration from TUI

### Before (TUI manages state)
```rust
// Column search in TUI
self.state_container.start_column_search(pattern);
self.state_container.update_column_search_matches(&columns, &pattern);
let column_search = self.state_container.column_search();
if !column_search.matching_columns.is_empty() {
    // Navigate matches
}
```

### After (DataView manages state)
```rust
// Column search in DataView
if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
    dataview.search_columns(&pattern);
    if let Some(col_idx) = dataview.get_current_column_match() {
        // Jump to column
    }
}
```

## Next Steps
With DataView now handling ALL view state:
1. Remove ColumnSearchState from AppStateContainer
2. Update TUI to use DataView's column search
3. Consider moving search highlighting to DataView
4. Add undo/redo support at DataView level
5. Prepare for Redux-style immutable updates