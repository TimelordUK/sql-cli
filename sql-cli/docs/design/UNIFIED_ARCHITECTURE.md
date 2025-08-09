# Unified Architecture: Buffers, DataTableView, and TUI

## The Complete Picture

```
┌─────────────────────────────────────────────────────┐
│                      TUI                             │
│  (Pure presentation - no state, only renders)        │
└─────────────────────────────────────────────────────┘
                          │
                    Interacts with
                          ▼
┌─────────────────────────────────────────────────────┐
│                  DataTableView                       │
│  (Sorted, filtered, paginated view of data)         │
│  - Handles all data presentation logic               │
│  - Column widths, formatting, etc.                   │
└─────────────────────────────────────────────────────┘
                          │
                     Backed by
                          ▼
┌─────────────────────────────────────────────────────┐
│                    DataTable                         │
│  (Raw data with schema, types, metadata)            │
│  - Column definitions                                │
│  - Data rows                                         │
│  - Statistics                                        │
└─────────────────────────────────────────────────────┘
                          │
                    Produced by
                          ▼
┌─────────────────────────────────────────────────────┐
│                     Buffer                           │
│  (Query execution context)                           │
│  - Input management (query text)                     │
│  - Query execution                                   │
│  - Produces DataTable from results                   │
└─────────────────────────────────────────────────────┘
```

## The Problem with Special Mode Buffers

If we create SearchBuffer, FilterBuffer, etc., they don't fit well with DataTableView because:
- They don't produce DataTables
- They operate on existing DataTableViews
- They're really just UI state, not data sources

## Better Solution: View State vs Data State

### 1. Buffers Produce Data
```rust
trait Buffer {
    fn execute_query(&mut self) -> Result<DataTable>;
    fn get_input_manager(&self) -> &dyn InputManager;
    // ... other buffer concerns
}
```

### 2. DataTableView Handles Presentation
```rust
struct DataTableView {
    source: DataTable,
    
    // View state
    sort: Option<SortConfig>,
    filter: Option<FilterConfig>,
    search: Option<SearchState>,
    
    // Presentation
    visible_rows: Range<usize>,
    column_widths: Vec<u16>,
    
    // These ARE the special "buffers" we were thinking of
    filter_input: String,      // Filter pattern being typed
    search_input: String,       // Search pattern being typed
    column_search_input: String, // Column search pattern
}

impl DataTableView {
    fn apply_filter(&mut self, pattern: &str);
    fn apply_search(&mut self, pattern: &str);
    fn apply_sort(&mut self, column: usize, order: SortOrder);
    
    // Input handling for view operations
    fn handle_filter_input(&mut self, key: KeyEvent);
    fn handle_search_input(&mut self, key: KeyEvent);
}
```

### 3. TUI Delegates Everything
```rust
impl EnhancedTuiApp {
    fn handle_input(&mut self, key: KeyEvent) {
        match self.mode {
            AppMode::Command => {
                // Delegate to buffer's input manager
                self.current_buffer().handle_input(key);
            }
            AppMode::Results => {
                // Delegate to view's navigation
                self.current_view().handle_navigation(key);
            }
            AppMode::Filter => {
                // Delegate to view's filter input
                self.current_view().handle_filter_input(key);
            }
            AppMode::Search => {
                // Delegate to view's search input
                self.current_view().handle_search_input(key);
            }
            // ... etc
        }
    }
    
    fn render(&self, frame: &mut Frame) {
        // TUI just asks view what to render
        let view = self.current_view();
        
        // Render input area
        let input_widget = view.get_input_widget();
        frame.render_widget(input_widget, input_area);
        
        // Render results
        let table_widget = view.get_table_widget();
        frame.render_widget(table_widget, results_area);
        
        // Render status
        let status_widget = view.get_status_widget();
        frame.render_widget(status_widget, status_area);
    }
}
```

## The Key Insight

**Special modes aren't separate buffers - they're operations on the DataTableView!**

- **Search**: Finding within the current view
- **Filter**: Filtering the current view
- **Sort**: Sorting the current view
- **Column Search**: Navigating columns in the view

These all operate on the DataTableView, not on separate buffers.

## Revised Architecture

```rust
struct BufferManager {
    buffers: Vec<Buffer>,  // Query execution contexts
    current: usize,
}

struct Buffer {
    input_manager: Box<dyn InputManager>,  // Query input
    data_source: DataSource,               // CSV, API, Cache
    last_result: Option<DataTable>,        // Last query result
    view: DataTableView,                   // View of last_result
}

struct DataTableView {
    // Data
    table: DataTable,
    
    // View state
    mode: ViewMode,
    filter: FilterState,
    search: SearchState,
    sort: SortState,
    
    // View-specific input (not queries)
    filter_input: SimpleInput,    // Just a string + cursor
    search_input: SimpleInput,    // Just a string + cursor
    
    // Derived data
    visible_rows: Vec<usize>,     // Indices after filter/sort
    column_widths: Vec<u16>,
}

enum ViewMode {
    Normal,
    Filtering,    // User is typing a filter
    Searching,    // User is typing a search
    Sorting,      // User is selecting sort column
}
```

## Benefits of This Approach

1. **Clean Separation**: 
   - Buffer = Query execution
   - DataTable = Raw data
   - DataTableView = Presentation
   - TUI = Pure rendering

2. **No Special Buffers Needed**: Filter/Search are view operations

3. **Consistent Data Flow**: 
   - Buffer executes query → produces DataTable
   - DataTable wrapped in DataTableView
   - TUI renders DataTableView

4. **Future Proof**: Easy to add new view operations without new buffer types

## Migration Path

1. **Phase 1**: Create DataTable and DataTableView structures
2. **Phase 2**: Move filter/search/sort state into DataTableView
3. **Phase 3**: Remove filter/search/sort from TUI
4. **Phase 4**: TUI becomes pure presentation layer

## Conclusion

Instead of creating special buffer types for Search/Filter/etc., we should:
1. Keep Buffer focused on query execution and producing DataTables
2. Put all view operations (filter, search, sort) in DataTableView
3. Make TUI a pure presentation layer

This aligns much better with the DataTable abstraction you mentioned and creates a cleaner architecture overall.