# DataTable/DataView Implementation Strategy

## Executive Summary

This document outlines our **incremental, non-breaking** strategy to migrate from the current buffer/results system to a clean DataTable/DataView architecture. The key principle is to **transform the system from the inside out** while keeping the TUI layer stable and unaware of the changes.

## Core Philosophy

> "Leave the TUI thinking it's talking to the CSVClient whilst under covers we are changing it all to DataTables"

This approach ensures:
- No breaking changes to the TUI
- Continuous working state (can ship at any point)
- Easy rollback if issues arise
- Small, reviewable PRs (~20 branches expected)

## The Problem We're Solving

Currently:
- TUI directly manipulates data through Buffer/CSVClient
- Multiple filtering layers cause confusion (WHERE, LIMIT, fuzzy filter)
- Data and view logic are intertwined
- Row counts are incorrect due to multiple filter states
- The TUI knows too much about data implementation

Goal:
- TUI only knows about views (what to display)
- DataTable holds immutable source data
- DataView provides filtered/sorted/projected presentation
- Clean separation between data and presentation

## Implementation Strategy

### Phase 1: Define the Contract (V34-V36)

Define traits that represent what the TUI actually needs, not how data is stored.

```rust
// Core trait - what any data source must provide
pub trait DataProvider: Send + Sync {
    // Basic data access
    fn get_row(&self, index: usize) -> Option<Vec<String>>;
    fn get_column_names(&self) -> Vec<String>;
    fn get_row_count(&self) -> usize;
    fn get_column_count(&self) -> usize;
    
    // For rendering
    fn get_visible_rows(&self, start: usize, count: usize) -> Vec<Vec<String>>;
    fn get_column_widths(&self) -> Vec<usize>;
    
    // For display
    fn get_cell_value(&self, row: usize, col: usize) -> Option<String>;
    fn get_display_value(&self, row: usize, col: usize) -> String;
    
    // For statistics
    fn get_column_stats(&self, col_index: usize) -> Option<ColumnStats>;
    fn get_column_type(&self, col_index: usize) -> DataType;
}

// Extended trait for views that can be modified
pub trait DataViewProvider: DataProvider {
    // Filtering
    fn apply_filter(&mut self, filter: FilterSpec) -> Result<()>;
    fn clear_filters(&mut self);
    fn get_active_filters(&self) -> Vec<FilterSpec>;
    fn get_filtered_count(&self) -> usize;
    
    // Sorting  
    fn sort_by(&mut self, column: usize, order: SortOrder) -> Result<()>;
    fn clear_sort(&mut self);
    fn get_sort_state(&self) -> Option<(usize, SortOrder)>;
    
    // Selection/Navigation
    fn is_row_visible(&self, row: usize) -> bool;
    fn map_to_source_row(&self, view_row: usize) -> Option<usize>;
    fn map_to_view_row(&self, source_row: usize) -> Option<usize>;
}

// Filter specification
pub enum FilterSpec {
    WhereClause(String),        // SQL WHERE
    FuzzySearch(String),        // Fuzzy text search
    Regex(String),              // Regex pattern
    ColumnFilter {              // Simple column filter
        column: String,
        operator: FilterOp,
        value: String,
    },
}
```

### Phase 2: Adapter Layer (V37-V39)

Create adapters that make existing components implement the new traits **without changing them**.

```rust
// Makes Buffer look like a DataProvider
pub struct BufferAdapter<'a> {
    buffer: &'a Buffer,
}

impl<'a> DataProvider for BufferAdapter<'a> {
    fn get_row(&self, index: usize) -> Option<Vec<String>> {
        self.buffer.get_results()
            .and_then(|r| r.data.get(index))
            .map(|row| {
                // Convert existing JSON format to Vec<String>
                row.as_object()
                    .map(|obj| obj.values().map(|v| v.to_string()).collect())
                    .unwrap_or_default()
            })
    }
    
    fn get_column_names(&self) -> Vec<String> {
        self.buffer.get_column_names()
    }
    
    // ... implement other required methods using Buffer's existing API
}

// Makes CSVClient look like a DataProvider  
pub struct CSVClientAdapter {
    client: CSVClient,
    cached_results: Option<QueryResponse>,
}

impl DataProvider for CSVClientAdapter {
    // Similar implementation wrapping CSVClient
}
```

### Phase 3: Update TUI to Use Traits (V40-V45)

Gradually change TUI methods to use traits instead of concrete types.

```rust
// Before (TUI knows about Buffer):
impl EnhancedTuiApp {
    fn render_table(&self, buffer: &Buffer) {
        let results = buffer.get_results();
        // ... direct manipulation
    }
}

// After (TUI only knows about traits):
impl EnhancedTuiApp {
    fn render_table(&self, data_view: &dyn DataViewProvider) {
        let row_count = data_view.get_row_count();
        let visible_rows = data_view.get_visible_rows(start, count);
        // ... work with trait methods only
    }
}
```

**Key: Do this one method at a time across multiple branches**

### Phase 4: Introduce DataTable Behind the Scenes (V46-V50)

Build DataTable while keeping everything working through adapters.

```rust
pub struct DataTable {
    /// Table name/identifier
    name: String,
    
    /// Column definitions (simple at first)
    columns: Vec<DataColumn>,
    
    /// Rows of data (simple at first)
    rows: Vec<Vec<DataValue>>,
    
    /// Source metadata
    source: DataSource,
}

pub struct DataColumn {
    name: String,
    data_type: DataType,
    // Start simple, add more later
}

pub enum DataValue {
    Null,
    String(String),
    Number(f64),
    Boolean(bool),
    // Start simple, add more types later
}

impl DataTable {
    /// Convert from existing format
    pub fn from_query_response(response: &QueryResponse) -> Result<Self> {
        // Transform existing data to DataTable format
        // This is where we normalize CSV/JSON/API responses
    }
    
    /// Convert from CSV
    pub fn from_csv_data(data: &CSVData) -> Result<Self> {
        // Specific CSV conversion
    }
}

// Make DataTable implement DataProvider
impl DataProvider for DataTable {
    fn get_row(&self, index: usize) -> Option<Vec<String>> {
        self.rows.get(index)
            .map(|row| row.iter().map(|v| v.to_string()).collect())
    }
    // ... implement other methods
}
```

### Phase 5: Introduce DataView as Filtering Layer (V51-V55)

DataView starts as a simple wrapper that doesn't modify data.

```rust
pub struct DataView {
    /// Reference to underlying data (immutable)
    source: Arc<dyn DataProvider>,
    
    /// View configuration (start simple)
    filters: Vec<FilterSpec>,
    sort: Option<(usize, SortOrder)>,
    
    /// Cached view state (computed lazily)
    visible_rows: RefCell<Option<Vec<usize>>>,
}

impl DataView {
    pub fn new(source: Arc<dyn DataProvider>) -> Self {
        Self {
            source,
            filters: Vec::new(),
            sort: None,
            visible_rows: RefCell::new(None),
        }
    }
    
    /// Compute visible rows based on filters
    fn compute_visible_rows(&self) -> Vec<usize> {
        let total = self.source.get_row_count();
        let mut indices: Vec<usize> = (0..total).collect();
        
        // Apply filters (start with simple implementation)
        for filter in &self.filters {
            indices = self.apply_single_filter(filter, indices);
        }
        
        // Apply sort if needed
        if let Some((col, order)) = self.sort {
            self.sort_indices(&mut indices, col, order);
        }
        
        indices
    }
}

impl DataProvider for DataView {
    fn get_row(&self, index: usize) -> Option<Vec<String>> {
        let visible = self.visible_rows.borrow();
        let rows = visible.as_ref()
            .unwrap_or_else(|| {
                let computed = self.compute_visible_rows();
                *self.visible_rows.borrow_mut() = Some(computed.clone());
                self.visible_rows.borrow()
            });
        
        rows.get(index)
            .and_then(|&source_idx| self.source.get_row(source_idx))
    }
    
    fn get_row_count(&self) -> usize {
        self.visible_rows.borrow()
            .as_ref()
            .map(|rows| rows.len())
            .unwrap_or_else(|| self.compute_visible_rows().len())
    }
    // ... other methods delegate through visible_rows mapping
}

impl DataViewProvider for DataView {
    fn apply_filter(&mut self, filter: FilterSpec) -> Result<()> {
        self.filters.push(filter);
        self.visible_rows.borrow_mut().take(); // Invalidate cache
        Ok(())
    }
    // ... implement other view methods
}
```

### Phase 6: Integration with AppStateContainer (V56-V60)

Update AppStateContainer to manage DataTables and DataViews.

```rust
impl AppStateContainer {
    /// Current data source (polymorphic)
    data_source: Arc<dyn DataProvider>,
    
    /// Active view
    active_view: RefCell<DataView>,
    
    /// Load data (works with any source)
    pub fn load_data(&mut self, source: DataSource) -> Result<()> {
        let provider: Arc<dyn DataProvider> = match source {
            DataSource::CSV(path) => {
                // For now, use adapter
                let client = CSVClient::new(path)?;
                Arc::new(CSVClientAdapter::new(client))
            },
            DataSource::QueryResult(result) => {
                // Convert to DataTable
                let table = DataTable::from_query_response(&result)?;
                Arc::new(table)
            },
            // ... other sources
        };
        
        self.data_source = provider.clone();
        self.active_view = RefCell::new(DataView::new(provider));
        Ok(())
    }
    
    /// Get current view for TUI
    pub fn current_view(&self) -> Ref<dyn DataViewProvider> {
        self.active_view.borrow()
    }
    
    /// Apply filter (TUI doesn't know this creates a view layer)
    pub fn apply_where_clause(&mut self, sql: &str) -> Result<()> {
        self.active_view.borrow_mut()
            .apply_filter(FilterSpec::WhereClause(sql.to_string()))
    }
}
```

### Phase 7: Gradual CSVClient Replacement (V61-V65)

Once everything works through traits, gradually replace CSVClient internals with DataTable.

```rust
// Old CSVClient gradually becomes a thin wrapper
pub struct CSVClient {
    // Instead of storing raw data
    // data: Vec<Vec<String>>,
    
    // Store as DataTable
    table: DataTable,
}

impl CSVClient {
    pub fn load_file(&mut self, path: &str) -> Result<()> {
        // Parse CSV into DataTable instead of custom format
        self.table = DataTable::from_csv_file(path)?;
        Ok(())
    }
    
    // Old API methods now delegate to DataTable
    pub fn get_results(&self) -> Option<&QueryResponse> {
        // Convert DataTable back to old format for compatibility
        // This conversion goes away once TUI fully uses traits
    }
}
```

## Migration Checkpoints

Each checkpoint is a stable, shippable state:

1. **V34-36**: Traits defined, no functional changes
2. **V40**: TUI renders through traits (using adapters)
3. **V45**: All TUI data access through traits
4. **V50**: DataTable exists but hidden behind adapters
5. **V55**: DataView provides filtering/sorting
6. **V60**: AppStateContainer manages views
7. **V65**: CSVClient internally uses DataTable
8. **V70**: Remove adapters, pure DataTable/DataView

## Testing Strategy

Each branch must:
1. Pass all existing tests
2. Add tests for new trait implementations
3. Verify no performance regression
4. Test on both Linux and Windows

```rust
#[test]
fn test_adapter_compatibility() {
    let buffer = create_test_buffer();
    let adapter = BufferAdapter::new(&buffer);
    
    // Verify adapter provides same data as direct buffer access
    assert_eq!(adapter.get_row_count(), buffer.get_row_count());
    assert_eq!(adapter.get_column_names(), buffer.get_column_names());
}

#[test]
fn test_view_filtering() {
    let table = DataTable::from_csv_data(&test_data);
    let mut view = DataView::new(Arc::new(table));
    
    let original_count = view.get_row_count();
    view.apply_filter(FilterSpec::WhereClause("age > 25".to_string()));
    
    assert!(view.get_row_count() < original_count);
    assert!(view.get_filtered_count() > 0);
}
```

## Benefits of This Approach

### Immediate
- No breaking changes
- Can ship at any point
- Easy to review (small PRs)
- Easy to rollback

### Long-term
- Clean architecture
- TUI becomes a pure view layer
- WHERE clauses naturally become view filters
- Multiple views of same data possible
- Correct row counts (single source of truth)

## What Success Looks Like

When complete, the TUI will:
- Only interact with `dyn DataViewProvider`
- Not know about Buffer, CSVClient, or DataTable
- Not manipulate data directly
- Just render what the view provides

Example of final state:
```rust
impl EnhancedTuiApp {
    fn handle_where_clause(&mut self, sql: &str) -> Result<()> {
        // TUI just asks for a filtered view
        self.state_container.apply_where_clause(sql)?;
        
        // Re-render with new view
        self.render_current_view()
    }
    
    fn render_current_view(&self) -> Result<()> {
        let view = self.state_container.current_view();
        
        // TUI only knows about the view interface
        let row_count = view.get_row_count();
        let visible_rows = view.get_visible_rows(self.viewport.start, self.viewport.height);
        
        // Render the data
        self.render_table(visible_rows);
        self.render_status(format!("Showing {} rows", row_count));
        
        Ok(())
    }
}
```

## Key Principles

1. **Never break the TUI** - It should always work
2. **Incremental changes** - One method, one branch
3. **Traits as contracts** - TUI depends on traits, not implementations
4. **Adapters as bridges** - Keep old code working while building new
5. **Test continuously** - Each branch must be stable
6. **Document decisions** - Future us will thank current us

## Conclusion

This strategy allows us to completely rebuild the data layer without the TUI knowing or caring. By using traits as contracts and adapters as bridges, we can incrementally transform the system while maintaining stability. The key insight is that WHERE clauses, filtering, and sorting are naturally view operations - they don't modify data, they just change what we see.

The DataTable/DataView separation gives us the clean architecture we need while the incremental approach ensures we don't "spiral out of control" as originally feared.