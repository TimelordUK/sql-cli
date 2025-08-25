# Viewport State Refactor Plan

## Problem Statement
We currently have multiple sources of truth for viewport and navigation state that require manual synchronization, leading to bugs where different components get out of sync. This was highlighted by the vim search navigation bug where the ViewportManager had the correct viewport position but the UI showed wrong cells because NavigationState wasn't synced.

## Current State (Multiple Sources of Truth)

### 1. ViewportManager (`src/ui/viewport_manager.rs`)
- `viewport_rows: Range<usize>` - which rows are visible
- `viewport_cols: Range<usize>` - which columns are visible  
- `crosshair_row: usize` - absolute row position
- `crosshair_col: usize` - absolute column position
- `terminal_width/height` - terminal dimensions

### 2. NavigationState (`src/app_state_container.rs`)
- `selected_row: usize` - current row position
- `selected_column: usize` - current column position
- `scroll_offset: (usize, usize)` - viewport scroll position
- `viewport_height/width` - viewport dimensions

### 3. Buffer (`src/buffer.rs`)
- `current_column: usize` - selected column
- `selected_row: Option<usize>` - selected row
- `scroll_offset: (usize, usize)` - scroll position

### 4. TableWidgetManager (`src/ui/table_widget_manager.rs`)
- `position: TablePosition` - crosshair position
- `scroll_offset: (usize, usize)` - viewport scroll

### 5. SelectionState (`src/app_state_container.rs`)
- `selected_row: usize`
- `selected_column: usize`

## The Synchronization Problem

When vim search navigates to a match, it must update:
1. ViewportManager's viewport and crosshair
2. NavigationState's selected_row, selected_column, and scroll_offset
3. Buffer's current_column and selected_row
4. TableWidgetManager's position
5. SelectionState's selected_column

Any missed update causes display issues where the crosshair lands on wrong cells.

## Example of Current Fix (Band-aid Solution)
```rust
// From enhanced_tui.rs - vim_search_next()
// After updating ViewportManager, we manually sync everything else:
if let Some(ref viewport) = *self.viewport_manager.borrow() {
    let viewport_rows = viewport.get_viewport_rows();
    let viewport_cols = viewport.viewport_cols();
    
    // Update navigation scroll offset to match viewport
    self.state_container.navigation_mut().scroll_offset = (viewport_rows.start, viewport_cols.start);
    
    // Also update buffer scroll offset
    self.state_container.set_scroll_offset((viewport_rows.start, viewport_cols.start));
}
```

## Proposed Solution: Single Source of Truth

### Option 1: ViewportManager as Central Authority
Make ViewportManager the single source of truth for all viewport and position state:

```rust
impl ViewportManager {
    // All position/viewport queries go through here
    pub fn get_selected_row(&self) -> usize { self.crosshair_row }
    pub fn get_selected_column(&self) -> usize { self.crosshair_col }
    pub fn get_scroll_offset(&self) -> (usize, usize) {
        (self.viewport_rows.start, self.viewport_cols.start)
    }
}
```

Then update other components to be thin proxies:
```rust
impl NavigationState {
    pub fn selected_row(&self) -> usize {
        self.viewport_manager.get_selected_row()
    }
}
```

### Option 2: Unified State Manager
Create a new `UnifiedNavigationState` that combines all navigation/viewport state:

```rust
pub struct UnifiedNavigationState {
    // Single source of truth
    crosshair: (usize, usize),
    viewport: (Range<usize>, Range<usize>),
    terminal_size: (u16, u16),
    
    // Computed properties
    fn scroll_offset(&self) -> (usize, usize) {
        (self.viewport.0.start, self.viewport.1.start)
    }
}
```

### Option 3: Event-Driven Synchronization
Keep separate states but use an event system to ensure updates propagate:

```rust
enum NavigationEvent {
    CrosshairMoved { row: usize, col: usize },
    ViewportScrolled { rows: Range<usize>, cols: Range<usize> },
}

// Single update point that notifies all subscribers
fn update_navigation(event: NavigationEvent) {
    // Automatically sync all dependent states
}
```

## Implementation Steps

### Phase 1: Audit and Document Dependencies
1. Map all places that read navigation/viewport state
2. Map all places that write navigation/viewport state  
3. Identify which components actually need their own state vs. which could reference shared state

### Phase 2: Consolidate State
1. Choose approach (recommend Option 1 - ViewportManager as central)
2. Create migration plan to avoid breaking existing code
3. Add compatibility layer during transition

### Phase 3: Refactor Components
1. Update NavigationState to proxy to ViewportManager
2. Update Buffer to remove redundant position tracking
3. Update TableWidgetManager to use ViewportManager
4. Update SelectionState to reference ViewportManager

### Phase 4: Remove Synchronization Code
1. Remove all manual sync code like the vim search fix
2. Remove duplicate state fields
3. Update tests

## Benefits of Refactor
1. **Eliminates sync bugs** - Single source of truth means no sync needed
2. **Simpler code** - Remove hundreds of lines of sync logic
3. **Easier debugging** - One place to check for position/viewport state
4. **Better performance** - Less state copying and updating
5. **Clearer architecture** - Obvious where navigation state lives

## Files to Modify
- `src/ui/viewport_manager.rs` - Enhance to be central authority
- `src/app_state_container.rs` - Remove duplicate state from NavigationState
- `src/buffer.rs` - Remove position tracking
- `src/ui/table_widget_manager.rs` - Use ViewportManager for position
- `src/ui/enhanced_tui.rs` - Remove sync code
- `src/ui/vim_search_manager.rs` - Simplify to just update ViewportManager

## Testing Strategy
1. Create integration tests for navigation consistency
2. Test vim search, column operations, scrolling
3. Verify F5 debug shows consistent state
4. Performance test to ensure no regression

## Estimated Effort
- 2-3 days for full refactor
- Can be done incrementally without breaking existing functionality
- Start with ViewportManager enhancements, then migrate components one by one

## Next Steps
1. Review this plan and decide on approach
2. Create feature branch `viewport-refactor`
3. Start with Phase 1 audit to understand full scope
4. Implement chosen solution incrementally