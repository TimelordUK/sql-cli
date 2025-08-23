# Column Operations Simplification - TODO

## Target Methods for Today's Work

Based on navigation method simplification success, apply same pattern to column operations:

### Methods to Simplify:
1. `move_column_right()` - src/ui/enhanced_tui.rs
2. `move_column_left()` - src/ui/enhanced_tui.rs  
3. `unhide_all_columns()` - src/ui/enhanced_tui.rs
4. `hide_current_column()` - src/ui/enhanced_tui.rs:171 (user selected)

## Objective

**Goal**: Prepare these column operations for trait extraction by:
- Identifying minimal state dependencies needed
- Creating helper methods similar to `apply_row_navigation_result()`
- Reducing complex internal state management to simple patterns
- Eliminating duplicated code and ViewportManager fallbacks

## Expected Pattern

Based on navigation method success, target pattern:
```rust
fn column_operation_method(&mut self) {
    // Use ViewportManager for operation
    let operation_result = {
        let mut viewport_borrow = self.viewport_manager.borrow_mut();
        viewport_borrow.as_mut().map(|vm| vm.column_operation())
    };

    if let Some(operation_result) = operation_result {
        self.apply_column_operation_result(operation_result);
    }
}
```

## Analysis Needed

1. **Current State Dependencies**: What internal state do these methods currently access?
2. **ViewportManager Support**: Which column operations are already supported by ViewportManager?
3. **Helper Method Design**: What would `apply_column_operation_result()` need to handle?
4. **Result Types**: What data structure should column operation results return?

## Success Criteria

- [ ] All 4 column methods follow consistent ~8-line pattern
- [ ] Single helper method handles all state synchronization  
- [ ] No ViewportManager fallback code
- [ ] All tests continue to pass
- [ ] Methods ready for trait extraction with minimal dependencies

## Context: Navigation Method Success

Recently completed navigation method simplification:
- Reduced 9 navigation methods from 40+ lines each to 8 lines each
- Eliminated ~300 lines of duplicated/dead code
- Single `apply_row_navigation_result()` helper handles all state updates
- All methods ready for trait extraction
- Pattern: ViewportManager → Helper Method → Done

## Next Phase Preparation

After column operations are simplified:
- Both navigation and column operations ready for trait extraction
- Traits will only need access to helper methods and ViewportManager
- Modular TUI architecture without spiraling dependencies
- Foundation for further method extractions

---
**Status**: Ready to start after PC restart
**Branch**: Create new branch for column operations work
**Tests**: Ensure all 198 tests continue passing