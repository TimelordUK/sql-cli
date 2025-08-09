# Refactoring Status - Enhanced TUI Decomposition

## Problem Statement
- `enhanced_tui.rs` is 8,269 lines - a massive monolith
- Contains 207 methods mixing UI, data, and business logic
- Difficult to maintain, test, and extend
- Buffer refactoring incomplete - data handling still mixed with UI

## Work Completed

### ✅ Created `cursor_manager.rs`
- Extracted all cursor/navigation logic
- Word navigation (move_word_forward, move_word_backward)
- Table navigation (up/down/left/right, page up/down)
- Scroll management (horizontal/vertical)
- Token/word utilities for completion
- ~200 lines of focused, testable code

### ✅ Created `data_manager.rs` (Partial)
- Column width calculations
- Filter operations
- Search operations
- Data transformation utilities
- Statistics calculation
- **Issue**: Built for wrong data structure (assumed columns/rows format)

### ✅ Created `REFACTORING_PLAN.md`
- Comprehensive architecture design
- Clear separation of concerns
- Modular component structure
- Migration strategy

## Current Blockers

### Data Structure Mismatch
The `QueryResponse` structure is:
```rust
pub struct QueryResponse {
    pub data: Vec<Value>,  // JSON values, not rows/columns
    pub count: usize,
    pub query: QueryInfo,
    // ...
}
```

But we need a tabular format with columns and rows for display. The enhanced_tui must have conversion logic somewhere.

## Next Steps

### Option 1: Find Existing Conversion Logic
- Search for where enhanced_tui converts `Vec<Value>` to table format
- Extract that logic into a `ResultsProcessor`
- Adapt `DataManager` to use processed results

### Option 2: Create New Data Processing Layer
- Build `ResultsProcessor` to convert `QueryResponse` to tabular format
- Extract columns from first data item (if JSON object)
- Convert each Value to a row of strings
- Feed processed data to `DataManager`

### Option 3: Use DataTable Architecture
Since we already have:
- `DataTable` - structured data container
- `DataTableView` - view operations
- `DataTableLoaders` - loading from various sources

We could:
1. Convert `QueryResponse` to `DataTable`
2. Use `DataTableView` for all view operations
3. Migrate enhanced_tui to use DataTable internally

## Recommended Approach

**Use DataTable as the common data format:**

1. **Phase 1**: Create `QueryResponseToDataTable` converter
2. **Phase 2**: Replace direct data handling in enhanced_tui with DataTable
3. **Phase 3**: Extract all remaining data operations to DataManager
4. **Phase 4**: Extract rendering to ViewSystem
5. **Phase 5**: Slim down enhanced_tui to orchestrator only

This leverages existing work and provides a clean architecture.

## Benefits of DataTable Approach

1. **Already built**: DataTable/DataTableView infrastructure exists
2. **Consistent**: Same data structure for enhanced and modern TUI
3. **Feature-rich**: Sorting, filtering, searching already implemented
4. **Clean separation**: Data (DataTable) vs View (DataTableView)
5. **Testable**: Each component can be tested independently

## Implementation Priority

1. **Immediate**: Fix data structure issues in DataManager
2. **Next**: Create ResultsProcessor to handle QueryResponse
3. **Then**: Start migrating enhanced_tui to use DataTable
4. **Finally**: Complete extraction of UI logic to ViewSystem

## Success Metrics

- [ ] Enhanced_tui.rs reduced from 8,269 to < 1,000 lines
- [ ] All data operations in DataManager/ResultsProcessor
- [ ] All rendering in ViewSystem
- [ ] Clear module boundaries
- [ ] No performance regression
- [ ] All features preserved