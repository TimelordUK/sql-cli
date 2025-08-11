# DataTable/DataView Architecture Proposal

## Executive Summary

This proposal outlines a comprehensive refactoring of our data management layer to adopt a DataTable/DataView architecture pattern, similar to C#/.NET's DataTable/DataView model. This architecture will provide a clean separation between data storage and data presentation, solving current issues with row counts, filtering, and state management while enabling powerful new features.

## Current Problems

1. **Row Count Confusion**: Multiple filtering layers (WHERE clause, LIMIT, fuzzy filter) each maintain their own filtered data, leading to incorrect row counts
2. **Duplicated Logic**: Filtering, sorting, and data transformation logic is scattered across TUI, Buffer, and various state objects
3. **Memory Inefficiency**: Multiple copies of data exist for different views (filtered_data, fuzzy_filter_indices, etc.)
4. **Complex State Management**: No single source of truth for what data is currently visible to the user
5. **Limited Composability**: Can't easily chain multiple filters or transformations

## Proposed Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      DataSource Layer                        │
│  (CSV, JSON, SQL Query Results, API responses)              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                         DataTable                            │
│  - Immutable raw data storage                               │
│  - Column definitions with types                            │
│  - Row storage with type-safe values                        │
│  - Statistics and metadata                                  │
│  - Memory-efficient storage                                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                         DataView                             │
│  - Virtual view over DataTable                              │
│  - Zero-copy where possible                                 │
│  - Composable filters and transformations                   │
│  - Lazy evaluation                                          │
│  - Maintains view state (sort, filter, visible rows)        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    AppStateContainer                         │
│  - Manages active DataView                                  │
│  - Publishes view change events                             │
│  - Coordinates view updates                                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                         TUI Layer                            │
│  - Binds to current DataView                                │
│  - Re-renders on view changes                               │
│  - No data manipulation logic                               │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. DataTable

```rust
pub struct DataTable {
    /// Table name/identifier
    pub name: String,
    
    /// Column definitions
    pub columns: Vec<DataColumn>,
    
    /// Rows of data
    pub rows: Vec<DataRow>,
    
    /// Table-level metadata
    pub metadata: TableMetadata,
    
    /// Indexes for fast lookups (optional)
    indexes: HashMap<String, BTreeMap<DataValue, Vec<usize>>>,
}

pub struct DataColumn {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub unique: bool,
    pub indexed: bool,
    pub statistics: ColumnStatistics,
}

pub struct DataRow {
    pub values: Vec<DataValue>,
    pub row_id: usize,  // Internal row identifier
}

pub enum DataValue {
    Null,
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Date(chrono::NaiveDate),
    DateTime(chrono::DateTime<Utc>),
    Json(serde_json::Value),
}
```

### 2. DataView

```rust
pub struct DataView {
    /// Reference to underlying DataTable
    source: Arc<DataTable>,
    
    /// View configuration
    config: ViewConfig,
    
    /// Cached view state (lazy evaluated)
    state: RefCell<ViewState>,
}

pub struct ViewConfig {
    /// Active filters (applied in order)
    filters: Vec<Box<dyn ViewFilter>>,
    
    /// Sort configuration
    sort: Option<SortConfig>,
    
    /// Row limit (for LIMIT clause)
    limit: Option<usize>,
    
    /// Row offset (for pagination)
    offset: usize,
    
    /// Column visibility
    visible_columns: Option<Vec<String>>,
    
    /// Computed columns
    computed_columns: Vec<ComputedColumn>,
}

pub struct ViewState {
    /// Visible row indices (after all filters)
    visible_rows: Vec<usize>,
    
    /// Total row count (for display)
    total_rows: usize,
    
    /// Filtered row count
    filtered_rows: usize,
    
    /// Last evaluation timestamp
    last_updated: Instant,
    
    /// Dirty flag for re-evaluation
    needs_update: bool,
}

/// Trait for view filters
pub trait ViewFilter: Send + Sync {
    /// Apply filter to row indices
    fn apply(&self, table: &DataTable, rows: &[usize]) -> Vec<usize>;
    
    /// Human-readable description
    fn description(&self) -> String;
    
    /// Can this filter be combined with another?
    fn can_merge(&self, other: &dyn ViewFilter) -> bool;
}
```

### 3. Filter Implementations

```rust
/// SQL WHERE clause filter
pub struct WhereFilter {
    expression: SqlExpression,
    compiled: CompiledExpression,
}

/// Fuzzy text search filter
pub struct FuzzyFilter {
    pattern: String,
    matcher: SkimMatcherV2,
    columns: Vec<String>,  // Which columns to search
    threshold: i64,        // Match score threshold
}

/// Regular expression filter
pub struct RegexFilter {
    pattern: Regex,
    columns: Vec<String>,
    case_insensitive: bool,
}

/// Column value filter
pub struct ColumnFilter {
    column: String,
    operator: FilterOperator,
    value: DataValue,
}

/// Composite filter (AND/OR)
pub struct CompositeFilter {
    operator: LogicalOperator,
    filters: Vec<Box<dyn ViewFilter>>,
}
```

### 4. DataView Operations

```rust
impl DataView {
    /// Create a new view over a DataTable
    pub fn new(source: Arc<DataTable>) -> Self;
    
    /// Add a filter to the view
    pub fn add_filter(&mut self, filter: Box<dyn ViewFilter>) -> &mut Self;
    
    /// Clear all filters
    pub fn clear_filters(&mut self) -> &mut Self;
    
    /// Set sort configuration
    pub fn sort_by(&mut self, column: &str, order: SortOrder) -> &mut Self;
    
    /// Set row limit
    pub fn limit(&mut self, limit: usize) -> &mut Self;
    
    /// Set row offset
    pub fn offset(&mut self, offset: usize) -> &mut Self;
    
    /// Get visible row count
    pub fn row_count(&self) -> usize;
    
    /// Get row at index (in view coordinates)
    pub fn get_row(&self, index: usize) -> Option<&DataRow>;
    
    /// Iterate over visible rows
    pub fn rows(&self) -> DataViewIterator;
    
    /// Clone this view (cheap - shares underlying data)
    pub fn clone_view(&self) -> Self;
    
    /// Create a new view with additional filter
    pub fn filter(&self, filter: Box<dyn ViewFilter>) -> Self;
    
    /// Export view to various formats
    pub fn export(&self, format: ExportFormat) -> Result<Vec<u8>>;
}
```

### 5. Integration with AppStateContainer

```rust
pub struct DataState {
    /// Currently loaded DataTables
    tables: HashMap<String, Arc<DataTable>>,
    
    /// Active view
    active_view: Option<DataView>,
    
    /// View history for undo/redo
    view_history: VecDeque<DataView>,
    
    /// Event subscribers
    view_listeners: Vec<Box<dyn ViewListener>>,
}

impl AppStateContainer {
    /// Load data into a new DataTable
    pub fn load_data(&mut self, source: DataSource, table_name: String) -> Result<()>;
    
    /// Set the active view
    pub fn set_view(&mut self, view: DataView);
    
    /// Get the current view
    pub fn current_view(&self) -> Option<&DataView>;
    
    /// Apply a filter to current view
    pub fn apply_filter(&mut self, filter: Box<dyn ViewFilter>);
    
    /// Clear all filters
    pub fn clear_filters(&mut self);
    
    /// Notify listeners of view change
    fn notify_view_change(&self, change: ViewChangeEvent);
}

pub enum ViewChangeEvent {
    ViewUpdated { old_count: usize, new_count: usize },
    FilterAdded { filter: String },
    FilterRemoved,
    SortChanged { column: String, order: SortOrder },
    DataLoaded { table: String, rows: usize },
}
```

## Migration Plan

### Phase 1: Core Infrastructure (v20)
1. Implement DataTable and DataColumn structures
2. Implement DataValue enum with type conversions
3. Create DataTableConverter for existing data sources
4. Add unit tests for DataTable operations

### Phase 2: Basic DataView (v21)
1. Implement DataView with basic filtering
2. Create WhereFilter for SQL WHERE clauses
3. Implement view state caching
4. Add view iterator for efficient row access

### Phase 3: Filter Implementations (v22)
1. Implement FuzzyFilter using existing SkimMatcherV2
2. Create RegexFilter for pattern matching
3. Implement CompositeFilter for AND/OR operations
4. Add ColumnFilter for simple comparisons

### Phase 4: State Integration (v23)
1. Add DataState to AppStateContainer
2. Implement view change notifications
3. Create ViewListener trait and implementations
4. Update TUI to use DataView instead of raw results

### Phase 5: Advanced Features (v24)
1. Add computed columns
2. Implement view persistence/serialization
3. Add index support for faster filtering
4. Create view composition/chaining API

### Phase 6: Performance Optimization (v25)
1. Implement lazy evaluation for large datasets
2. Add parallel filter evaluation
3. Optimize memory usage with zero-copy where possible
4. Add view caching and memoization

## Benefits

### Immediate Benefits
1. **Correct Row Counts**: Single source of truth for visible rows
2. **Clean Architecture**: Clear separation of concerns
3. **Memory Efficiency**: Single copy of data with virtual views
4. **Composability**: Easy to chain and combine filters

### Future Benefits
1. **Multiple Views**: Can have multiple views of same data
2. **Undo/Redo**: Easy to implement with view history
3. **Performance**: Can optimize filter evaluation
4. **Extensibility**: Easy to add new filter types
5. **Testing**: Pure functions, easy to test
6. **Debugging**: Clear data flow, easy to trace

## Example Usage

```rust
// Load data
let table = DataTable::from_csv("data.csv")?;
let table_arc = Arc::new(table);

// Create a view with multiple filters
let view = DataView::new(table_arc.clone())
    .filter(Box::new(WhereFilter::new("age > 25")?))
    .filter(Box::new(FuzzyFilter::new("John", vec!["name", "email"])))
    .sort_by("age", SortOrder::Descending)
    .limit(100);

// Get row count (correct, accounting for all filters)
println!("Showing {} of {} rows", view.row_count(), view.total_rows());

// Iterate over visible rows
for row in view.rows() {
    println!("{:?}", row);
}

// Create a new view with additional filter (shares underlying data)
let filtered_view = view.filter(Box::new(ColumnFilter::new(
    "department",
    FilterOperator::Equals,
    DataValue::String("Engineering".to_string())
)));
```

## Implementation Notes

1. **Zero-Copy Where Possible**: Views should reference the underlying DataTable without copying data
2. **Lazy Evaluation**: Filters should only be evaluated when needed
3. **Caching**: View state should be cached and only re-evaluated when config changes
4. **Thread Safety**: DataTable should be immutable and thread-safe for sharing
5. **Error Handling**: All operations should return Result types with clear error messages

## Testing Strategy

1. **Unit Tests**: Each component tested in isolation
2. **Integration Tests**: Full data flow from source to view
3. **Performance Tests**: Benchmark filter evaluation and view operations
4. **Memory Tests**: Verify zero-copy and memory efficiency
5. **Regression Tests**: Ensure no breaking changes to existing functionality

## Conclusion

This DataTable/DataView architecture will provide a solid foundation for data management in the application. It solves current issues while enabling powerful new features. The migration can be done incrementally without breaking existing functionality.

The key insight is that by separating data storage (DataTable) from data presentation (DataView), we gain flexibility, performance, and correctness. The TUI becomes a pure rendering layer that simply displays whatever the current DataView contains, making the entire system more maintainable and testable.