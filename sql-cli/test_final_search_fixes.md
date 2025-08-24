# Search Navigation Fixes Implemented

## Summary
Fixed two remaining issues with search navigation:
1. **'g' key resets search to first match** - Vim-like behavior when in search navigation mode
2. **Column scrolling for off-screen matches** - Initial search now scrolls horizontally to show matches

## Changes Made

### 1. Added reset_to_first_match method to VimSearchManager
- File: `src/ui/vim_search_manager.rs`
- Added new method that resets current_index to 0 and navigates to first match
- Logs the reset action for debugging

### 2. Modified goto_first_row to handle search reset
- File: `src/ui/enhanced_tui.rs` (line 6981)
- Checks if VimSearchManager is in navigating mode
- If yes, calls reset_to_first_match instead of normal first row navigation
- Updates TableWidgetManager to trigger proper re-render

### 3. Fixed column scrolling in debounced search
- File: `src/ui/enhanced_tui.rs` (line 2668)
- Added column offset calculation (was hardcoded to 0)
- Calculates if match column is off-screen left or right
- Centers column with some context when scrolling right
- Updates both row and column in viewport and navigation state

## Key Code Changes

### Column Scrolling Fix:
```rust
// Calculate column scroll if needed
let current_col_scroll = self.state_container.navigation().scroll_offset.1;
let new_col_offset = if col < current_col_scroll {
    col // Match is to the left, scroll left
} else if col >= current_col_scroll + viewport_width.saturating_sub(1) {
    let centered = col.saturating_sub(viewport_width / 4);
    centered // Match is to the right, scroll with context
} else {
    current_col_scroll // Already visible
};

// Update both row and column offsets
nav.scroll_offset.0 = new_row_offset;
nav.scroll_offset.1 = new_col_offset;
```

### 'g' Key Reset:
```rust
fn goto_first_row(&mut self) {
    if vim_search_borrow.is_navigating() {
        // Reset to first match in search results
        vim_search_mut.reset_to_first_match(viewport);
        table_widget_manager.navigate_to_search_match(first_match.row, first_match.col);
    } else {
        // Normal goto first row behavior
        <Self as NavigationBehavior>::goto_first_row(self);
    }
}
```

## Testing
Created test scripts:
- `test_search_fixes.sh` - Tests both 'g' key reset and column scrolling
- `test_column_scroll.sh` - Specific test for off-screen column matches

## Result
- Search navigation now properly scrolls both rows and columns on initial match
- 'g' key provides vim-like reset to first match during search navigation
- 'n' and 'N' continue to work for forward/backward navigation with proper scrolling