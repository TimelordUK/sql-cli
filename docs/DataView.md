# DataView Architecture

## Overview
A complete separation of data management from presentation, inspired by C#/.NET's DataTable/DataView pattern.

## Core Components

### DataTable
- **Purpose**: Raw data storage and management
- **Responsibilities**:
  - Store original query results
  - Maintain data integrity
  - Provide data access interface
  - Handle data types and schemas
- **Key Features**:
  - Immutable after load (read-only)
  - Schema-aware
  - Memory efficient

### DataView
- **Purpose**: Filtered/sorted/projected view of DataTable
- **Responsibilities**:
  - Apply filters without modifying source
  - Sort without modifying source
  - Column projection/hiding
  - Virtual scrolling support
- **Key Features**:
  - Multiple views per table
  - Lazy evaluation
  - Stateful (maintains position, selection)

### DataSet
- **Purpose**: Container for multiple related DataTables
- **Responsibilities**:
  - Manage table relationships
  - Cross-table queries
  - Unified schema management
- **Use Cases**:
  - Multiple query results
  - Related data from different sources
  - Join operations

### Query Optimizer
- **Purpose**: Intelligent query execution and caching
- **Responsibilities**:
  - Query plan generation
  - Cache management
  - Result reuse detection
  - Branch cache optimization
- **Key Features**:
  - AST-based query analysis
  - Incremental query optimization
  - Smart cache invalidation

## Implementation Phases

### Phase 1: Complete State Migration (Current)
- [x] ClipboardState → AppStateContainer (V20)
- [x] ColumnSearchState → AppStateContainer (V21)
- [x] SortState → AppStateContainer (V22)
- [ ] SelectionState → AppStateContainer
- [ ] FilterState → AppStateContainer  
- [ ] NavigationState → AppStateContainer
- [ ] ViewportState → AppStateContainer

### Phase 2: DataTable Implementation
- [ ] Extract results storage from Buffer
- [ ] Create DataTable struct with schema
- [ ] Implement data access interface
- [ ] Add type-aware column handling
- [ ] Memory optimization for large datasets

### Phase 3: DataView Implementation
- [ ] Create DataView abstraction
- [ ] Implement filter chains
- [ ] Add sort without data modification
- [ ] Virtual scrolling with viewport
- [ ] Column projection and pinning

### Phase 4: Query Optimizer
- [ ] AST-based query analysis
- [ ] Cache key generation
- [ ] Branch cache for partial matches
- [ ] Query rewrite for optimization
- [ ] Incremental result computation

### Phase 5: DataSet and Relations
- [ ] Multi-table container
- [ ] Table relationships
- [ ] Cross-table queries
- [ ] Join optimization

## Technical Decisions

### Read-Only Design
- All data modifications create new views
- Original data never mutated
- Enables aggressive caching
- Simplifies concurrency

### Lazy Evaluation
- Filters/sorts computed on-demand
- Viewport-based rendering
- Memory efficient for large datasets

### Branch Caching
- Cache intermediate results
- Detect common sub-expressions
- Example: `WHERE x > 10` can reuse `WHERE x > 10 AND y < 5`

## Current Issues to Address

### Column Navigation with Pinned Columns
- **Problem**: Viewport calculation confused when mixing pinned/scrollable columns
- **Root Cause**: Current code mixes data position with view position
- **Solution**: DataView will properly separate these concerns

### Sort State Restoration
- **Problem**: Original order lost after sorting
- **Root Cause**: Sorting modifies results data instead of creating view
- **Solution**: DataTable remains immutable, DataView handles sorting

### Filter Performance
- **Problem**: Filters re-evaluated on every render
- **Solution**: DataView caches filtered indices

## Migration Strategy

1. **State First**: Move all state to AppStateContainer (current phase)
2. **Data Extraction**: Pull data management out of Buffer/TUI
3. **View Layer**: Implement DataView for presentation
4. **Optimization**: Add caching and query optimization
5. **Advanced Features**: DataSet, relations, joins

## Benefits

### Immediate
- Cleaner architecture
- Easier testing
- Better performance
- Fix navigation issues

### Long-term
- Multiple views of same data
- Advanced filtering/sorting
- Query optimization
- Memory efficiency
- Potential for parallelization

## Example Usage (Future API)

```rust
// DataTable holds raw data
let table = DataTable::from_query_results(results);

// DataView provides filtered/sorted view
let mut view = DataView::new(&table);
view.add_filter("age > 25");
view.sort_by("name", SortOrder::Ascending);
view.pin_columns(vec![0, 1]);

// TUI only interacts with view
let visible_data = view.get_viewport(start_row, end_row, start_col, end_col);

// Query optimizer handles caching
let optimizer = QueryOptimizer::new();
let results = optimizer.execute(query, &dataset)?; // Returns cached if available

// Branch caching example
let q1 = "SELECT * FROM users WHERE age > 25";
let q2 = "SELECT * FROM users WHERE age > 25 AND city = 'London'";
// q2 can reuse q1's results and filter further
```

## Notes

- This refactor should only begin after ALL state is migrated to AppStateContainer
- Each phase should be mergeable and not break existing functionality
- Extensive testing required at each phase
- Performance benchmarks before/after each phase