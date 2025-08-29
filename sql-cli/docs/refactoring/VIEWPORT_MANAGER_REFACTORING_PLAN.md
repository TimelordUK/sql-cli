# ViewportManager Refactoring Plan

## Overview
The `ViewportManager` has grown to **4,531 lines** with **108 methods**, making it a monolithic class that violates the Single Responsibility Principle. This document outlines a systematic refactoring plan to extract logical subsystems into focused, testable components.

## Current State Analysis
- **File**: `src/ui/viewport_manager.rs`
- **Size**: 4,531 lines
- **Methods**: 108 methods
- **Primary Issues**:
  - Too many responsibilities
  - Difficult to test individual features
  - Hard to understand and maintain
  - Performance optimizations limited by coupling

## Proposed Subsystem Extraction

### Phase 1: Column Width Calculator (`ColumnWidthCalculator`)
**Target: ~500-800 lines**
```rust
// Methods to extract:
- calculate_column_widths()
- calculate_optimal_column_width() 
- sample_column_data_for_width()
- calculate_width_with_packing_mode()
- get_column_width_debug_info()
```

**Benefits**: Most self-contained subsystem, easiest to test, clear interface

### Phase 2: Viewport Bounds Calculator (`ViewportBoundsCalculator`) 
**Target: ~400-600 lines**
```rust
// Methods to extract:
- calculate_visible_column_indices()
- calculate_visible_column_indices_with_offset()
- calculate_scroll_offset_for_visual_column() // Recently fixed!
- determine_viewport_bounds()
```

**Benefits**: Contains recent navigation fix, well-defined mathematical operations

### Phase 3: Navigation Engine (`NavigationEngine`)
**Target: ~800-1000 lines**
```rust
// Methods to extract:
- navigate_column_left/right()
- navigate_to_first/last_column()
- navigate_row_up/down()
- page_up/page_down()
- goto_line()
- All NavigationResult building logic
```

**Benefits**: Largest reduction in main class size, clear user-facing API

### Phase 4: Crosshair Manager (`CrosshairManager`)
**Target: ~300-400 lines**
```rust
// Methods to extract:
- get_crosshair_column()
- get_crosshair_row() 
- set_crosshair_position()
- cursor_lock/viewport_lock logic
- crosshair coordinate translations
```

**Benefits**: Centralized cursor management, easier debugging

### Phase 5: Cache Manager (`ViewportCacheManager`)
**Target: ~200-300 lines**
```rust
// Methods to extract:
- cache invalidation logic
- visible row caching
- state hashing for cache validity
- cache_dirty management
```

**Benefits**: Performance optimizations can be focused and improved

### Phase 6: Visual Display Builder (`VisualDisplayBuilder`)
**Target: ~400-500 lines**
```rust
// Methods to extract:
- get_visual_display()
- get_visible_column_headers()
- coordinate space conversions
- display data assembly
```

**Benefits**: Rendering logic separation, easier testing of display output

## Refactored Architecture

### New ViewportManager Structure
```rust
pub struct ViewportManager {
    // Core state (minimal)
    dataview: Arc<DataView>,
    terminal_width: u16,
    terminal_height: u16,
    
    // Extracted subsystems
    width_calculator: ColumnWidthCalculator,
    bounds_calculator: ViewportBoundsCalculator, 
    navigation: NavigationEngine,
    crosshair: CrosshairManager,
    cache: ViewportCacheManager,
    display_builder: VisualDisplayBuilder,
    
    // Minimal remaining state
    viewport_cols: Range<usize>,
    viewport_rows: Range<usize>,
}
```

### Expected Size Reduction
- **Current**: 4,531 lines
- **Refactored Core**: ~1,000-1,500 lines (orchestration only)
- **Individual Subsystems**: 200-800 lines each (focused functionality)

## Benefits of Refactoring

### 1. Testability
- Each subsystem can be unit tested independently
- Easier to write targeted tests for specific behavior
- Better test coverage and faster test execution

### 2. Maintainability
- Easier to understand and modify specific behavior
- Clear separation of concerns
- Reduced cognitive load for developers

### 3. Performance
- Cache and width calculations can be optimized separately
- Subsystem-specific performance profiling
- Targeted performance improvements

### 4. Reusability
- Components could be used in other UI contexts
- Easier to adapt for different rendering backends
- Cleaner APIs for external integration

### 5. Debugging
- Clearer separation for troubleshooting issues
- Focused logging per subsystem
- Easier to isolate bugs to specific components

### 6. Code Review
- Much easier to review focused, smaller modules
- Clearer change impact analysis
- Faster code review cycles

## Implementation Strategy

### Phase 1: Proof of Concept (ColumnWidthCalculator)
1. Create new module `src/ui/viewport/column_width_calculator.rs`
2. Extract width calculation methods
3. Update ViewportManager to use the new component
4. Ensure all tests pass
5. Validate performance is maintained

### Phase 2-6: Iterative Extraction
- Extract one subsystem per iteration
- Maintain full test coverage
- Ensure no performance regressions
- Update documentation as we go

### Rollback Plan
- Each phase is a separate branch
- Full test suite validation at each step
- Easy rollback if issues discovered
- Incremental merge to main branch

## Success Criteria
- [ ] All existing functionality preserved
- [ ] Test suite continues to pass
- [ ] No performance regressions
- [ ] Code is more maintainable and testable
- [ ] ViewportManager reduced to orchestration role
- [ ] Each subsystem has clear, focused responsibility

## Next Steps
1. Create branch for Phase 1 (ColumnWidthCalculator extraction)
2. Implement proof of concept
3. Validate approach and adjust plan as needed
4. Continue with subsequent phases

---

*This refactoring will significantly improve the maintainability and testability of the viewport management system while preserving all existing functionality.*