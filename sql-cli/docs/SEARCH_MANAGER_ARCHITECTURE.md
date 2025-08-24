# SearchManager Architecture

## Overview
The SearchManager encapsulates all search logic for the TUI, providing a clean separation between search functionality and UI rendering. This solves the previous issues where search logic was scattered throughout the TUI code and breaking easily with changes.

## Key Problems Solved

### 1. Case Sensitivity Issues
- **Problem**: Search wasn't properly handling case variations (e.g., "Unconfirmed" vs "unconfirmed" vs "UNCONFIRMED")
- **Solution**: SearchManager has configurable case sensitivity that correctly handles all variations
- **Code**: `SearchConfig.case_sensitive` flag controls behavior

### 2. Coordinate Mapping
- **Problem**: Mapping between data cell coordinates and render cell coordinates was inconsistent
- **Solution**: SearchManager works directly with data coordinates, ViewportManager handles the mapping to render coordinates
- **Separation of Concerns**: Search logic doesn't need to know about rendering

### 3. Debounced Search Navigation
- **Problem**: Debounced search wasn't navigating to first match
- **Solution**: `execute_search_action` now properly uses SearchManager to get match coordinates and navigate

### 4. Testability
- **Problem**: Search logic was embedded in TUI, making it impossible to test independently
- **Solution**: SearchManager can be tested in isolation with unit tests

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     EnhancedTuiApp                       │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │SearchManager │  │ViewportMgr   │  │StateContainer│ │
│  │              │  │              │  │              │ │
│  │ - search()   │  │ - set_viewport│ │ - matches    │ │
│  │ - navigate() │  │ - crosshair  │  │ - selection  │ │
│  │ - matches    │  │              │  │              │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘

Data Flow:
1. User types search pattern
2. SearchModesWidget debounces input
3. EnhancedTuiApp calls SearchManager.search()
4. SearchManager finds matches in data coordinates
5. ViewportManager maps to render coordinates
6. UI navigates to first match
```

## Core Components

### SearchManager
- **Purpose**: Find matches in data
- **Input**: Pattern, data rows, visible columns
- **Output**: List of SearchMatch objects with coordinates
- **Features**:
  - Case sensitive/insensitive search
  - Regex support
  - Visible columns filtering
  - Match iteration and navigation

### SearchMatch
```rust
pub struct SearchMatch {
    pub row: usize,        // Data row index
    pub column: usize,     // Data column index  
    pub value: String,     // The matching value
    pub highlight_range: (usize, usize), // For highlighting
}
```

### SearchConfig
```rust
pub struct SearchConfig {
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub visible_columns_only: bool,
    pub wrap_around: bool,
}
```

## Integration Points

### 1. perform_search()
```rust
// Convert DataView to searchable data
let data: Vec<Vec<String>> = /* ... */;

// Use SearchManager
let match_count = self.search_manager.borrow_mut()
    .search(&pattern, &data, visible_columns);

// Navigate to first match
if let Some(first_match) = self.search_manager.borrow().first_match() {
    // Update viewport and navigate
}
```

### 2. execute_search_action()
```rust
// Get first match from SearchManager
let (row, col) = {
    let search_manager = self.search_manager.borrow();
    if let Some(first_match) = search_manager.first_match() {
        (first_match.row, first_match.column)
    } else {
        (0, 0)
    }
};

// Navigate using ViewportManager
viewport_manager.set_viewport(/* ... */);
viewport_manager.set_crosshair(row, col);
```

### 3. Navigation (n/N keys)
```rust
// Next match
let next = self.search_manager.borrow_mut().next_match();

// Previous match  
let prev = self.search_manager.borrow_mut().previous_match();
```

## Testing

The SearchManager includes comprehensive unit tests:

1. **test_case_insensitive_search**: Verifies case-insensitive matching
2. **test_case_sensitive_search**: Verifies exact case matching
3. **test_navigation**: Tests next/previous/wrap-around navigation
4. **test_visible_columns_filter**: Tests searching only visible columns
5. **test_scroll_offset_calculation**: Tests viewport scrolling logic
6. **test_find_from_position**: Tests finding next/previous from cursor

## Future Enhancements

1. **ViewportManager Integration**: 
   - SearchManager could be owned by ViewportManager
   - Automatic coordinate mapping

2. **Incremental Search**:
   - Cache data for faster re-searching
   - Only search changed rows

3. **Search History**:
   - Track recent searches
   - Quick recall with up/down arrows

4. **Multi-file Search**:
   - Search across multiple buffers
   - Global search results panel

5. **Advanced Patterns**:
   - Column-specific search (e.g., "status:pending")
   - Boolean operators (AND, OR, NOT)

## Migration Status

✅ **Completed**:
- SearchManager class created
- Integration with perform_search()
- Integration with execute_search_action()
- Unit tests
- Case sensitivity handling

🔄 **In Progress**:
- Full ViewportManager integration
- Coordinate mapping refinement

⏳ **Pending**:
- Column search migration
- Filter search migration
- Search history integration

## Benefits

1. **Maintainability**: Search logic is isolated and easy to modify
2. **Testability**: Can test search without UI dependencies
3. **Performance**: Optimized search algorithms in one place
4. **Reliability**: Fewer bugs from scattered logic
5. **Extensibility**: Easy to add new search features

## Usage Examples

### Basic Search
```rust
let mut manager = SearchManager::new();
manager.set_case_sensitive(false);
let count = manager.search("pattern", &data, None);
```

### Navigate Matches
```rust
// Go to first match
if let Some(first) = manager.first_match() {
    navigate_to(first.row, first.column);
}

// Cycle through matches
manager.next_match();
manager.previous_match();
```

### Configure Search
```rust
let config = SearchConfig {
    case_sensitive: false,
    use_regex: true,
    visible_columns_only: true,
    wrap_around: true,
};
let manager = SearchManager::with_config(config);
```