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

## Phase 1 Status: ✅ COMPLETED
- [x] ColumnWidthCalculator successfully extracted
- [x] All functionality preserved, tests pass
- [x] 150 lines moved to focused component
- [x] Comprehensive unit tests added
- [x] Merged to main branch

## Phase 2 Prep: Comprehensive Testing Strategy

**⚠️ CRITICAL: Test-First Approach Required**

Given the complexity of viewport management and recent navigation issues, we must write comprehensive tests BEFORE attempting Phase 2 refactoring.

### Why Test-First is Essential:
1. **Complex Domain**: Viewport involves coordinate transformations, boundary calculations, state sync
2. **Recent Bug History**: Just fixed major scrolling issues - likely more edge cases exist
3. **High-Risk Refactoring**: Core TUI functionality - breaking these methods breaks everything
4. **Hard to Debug**: Viewport issues are visual and difficult to reproduce consistently

### Phase 2 Scope (ViewportBoundsCalculator)
Methods to extract:
- `calculate_visible_column_indices()`
- `calculate_visible_column_indices_with_offset()`  
- `calculate_scroll_offset_for_visual_column()` *(recently fixed!)*
- `determine_viewport_bounds()`

### Comprehensive Test Plan

#### 1. Crosshair Placement Tests
```rust
// Test crosshair positioning with pinned columns
test_crosshair_with_pinned_columns()

// Test crosshair at viewport boundaries (first/last visible column)
test_crosshair_at_viewport_boundaries()

// Test crosshair persistence during scrolling
test_crosshair_persistence_during_scroll()

// Test coordinate space conversions (visual ↔ DataTable)
test_crosshair_coordinate_conversions()
```

#### 2. Viewport Scrolling Tests
```rust
// Test smooth scrolling vs jumping behavior
test_smooth_scrolling_behavior()

// Test boundary conditions (first column, last column)
test_viewport_boundary_conditions()

// Test scroll offset calculations with different column widths
test_scroll_offset_calculations()

// Test the exact scenario we just fixed (column 20 → 21)
test_navigation_column_20_to_21_no_jumping()
```

#### 3. Pinned Columns Integration Tests
```rust
// Test navigation with pinned columns
test_pinned_columns_navigation()

// Test viewport boundaries with mixed pinned/scrollable columns
test_mixed_pinned_scrollable_boundaries()

// Edge case: more pinned columns than viewport width
test_excessive_pinned_columns()
```

#### 4. Coordinate System Tests
```rust
// Test visual index ↔ DataTable index conversions
test_coordinate_system_conversions()

// Test display columns array handling
test_display_columns_handling()

// Test column visibility calculations
test_column_visibility_calculations()
```

#### 5. Edge Case Tests
```rust
// Test very wide columns (exceed viewport)
test_oversized_columns()

// Test very narrow columns (sub-pixel widths)
test_narrow_columns()

// Test empty datasets
test_empty_dataset_handling()

// Test single column datasets
test_single_column_dataset()

// Test terminal resize scenarios
test_terminal_resize_handling()
```

### Testing Architecture

**Integration Tests** (most valuable for viewport):
- Test complete user scenarios (navigation, scrolling, resizing)
- Verify no regression in recently fixed bugs
- Test crosshair placement accuracy
- Test coordinate system consistency

**Unit Tests** (for extracted components):
- Test individual methods in isolation
- Test boundary conditions and edge cases
- Test mathematical correctness of calculations

### Implementation Strategy

#### Day 1: Test Foundation
1. **Morning**: Write comprehensive viewport integration tests
2. **Afternoon**: Run tests, fix any revealed bugs  
3. **Evening**: Ensure 100% test coverage for target methods

#### Day 2: Phase 2 Refactoring
1. Extract ViewportBoundsCalculator with confidence
2. Run full test suite after each change
3. Refactor fearlessly knowing tests catch regressions

### Benefits of Test-First Approach
1. **Safety Net**: Comprehensive coverage before refactoring
2. **Regression Prevention**: Catch Phase 2 breaking changes immediately
3. **Documentation**: Tests serve as executable specification  
4. **Confidence**: Refactor fearlessly with test protection
5. **Better Design**: Testing often reveals design flaws early

## Next Steps
1. ✅ Phase 1 Complete - ColumnWidthCalculator extracted
2. 🧪 **NEXT: Write comprehensive viewport tests** 
3. 🔧 Phase 2: Extract ViewportBoundsCalculator (test-driven)
4. 🚀 Continue with remaining phases

---

*This refactoring will significantly improve the maintainability and testability of the viewport management system while preserving all existing functionality.*