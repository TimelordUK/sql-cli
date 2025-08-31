# Rendering with DataView Instead of DataProvider

## Current Setup (DataProvider)
```rust
fn render_table_with_provider(&self, f: &mut Frame, area: Rect, provider: &dyn DataProvider) {
    let row_count = provider.get_row_count();
    let headers = provider.get_column_names();
    // Get rows as Vec<String>
    let row_data = provider.get_row(index);
}
```

## Better Setup (DataView)
```rust
fn render_table_with_dataview(&self, f: &mut Frame, area: Rect, dataview: &DataView) {
    let row_count = dataview.row_count();
    let headers = dataview.column_names();
    
    // Get typed row data
    if let Some(row) = dataview.get_row(index) {
        // Convert to strings for display
        let row_strings: Vec<String> = row.values.iter()
            .map(|v| v.to_string())
            .collect();
    }
}
```

## Why DataView is Better

### DataProvider (Limited)
- Only provides string data
- No awareness of filtering/sorting state
- No column visibility info beyond what BufferAdapter provides
- Just a "dumb" data accessor

### DataView (Complete)
- Provides typed data (can format better)
- Knows about active filters (can show filter indicator)
- Knows about column search (can highlight matches)
- Knows about sort state (can show sort arrows)
- Has column visibility built-in
- Can provide row indices for jump-to-row

## Migration Path

### Option 1: Replace DataProvider Usage
```rust
// Instead of getting a DataProvider
if let Some(provider) = self.get_data_provider() {
    self.render_table_with_provider(f, area, provider.as_ref());
}

// Get DataView directly
if let Some(dataview) = self.buffer().get_dataview() {
    self.render_table_with_dataview(f, area, dataview);
}
```

### Option 2: Make DataView implement DataProvider
```rust
impl DataProvider for DataView {
    fn get_row(&self, index: usize) -> Option<Vec<String>> {
        self.get_row(index).map(|row| {
            row.values.iter().map(|v| v.to_string()).collect()
        })
    }
    
    fn get_column_names(&self) -> Vec<String> {
        self.column_names()
    }
    
    fn get_row_count(&self) -> usize {
        self.row_count()
    }
    
    fn get_column_count(&self) -> usize {
        self.column_count()
    }
}
```

Then you can pass DataView where DataProvider is expected.

### Option 3: Remove DataProvider Completely
Since DataView has everything we need, we can:
1. Remove the DataProvider trait
2. Remove BufferAdapter 
3. Render directly from DataView

## Recommendation

**Short term:** Make DataView implement DataProvider (Option 2) to get everything compiling quickly.

**Long term:** Remove DataProvider completely (Option 3) and render directly from DataView. This gives you:
- Better type information for formatting
- Access to all view state for UI indicators
- Simpler architecture (one less abstraction layer)

## Benefits of Direct DataView Rendering

1. **Show filter status**: `if dataview.has_filter() { show_indicator() }`
2. **Show column search**: `if let Some(match) = dataview.get_current_column_match() { highlight(match) }`
3. **Show sort arrows**: Check sort state directly
4. **Better formatting**: Use DataValue types for proper number/date formatting
5. **Performance**: One less abstraction layer

## Example Implementation

```rust
fn render_table_from_dataview(&self, f: &mut Frame, area: Rect) {
    let Some(dataview) = self.buffer().get_dataview() else {
        // Render empty state
        return;
    };
    
    // Get view information
    let row_count = dataview.row_count();
    let headers = dataview.column_names();
    let has_filter = dataview.has_filter();
    let column_search = dataview.column_search_pattern();
    
    // Build header with indicators
    let mut header_cells = vec![];
    for (idx, name) in headers.iter().enumerate() {
        let mut cell_text = name.clone();
        
        // Add sort indicator
        if let Some(sort_info) = self.get_sort_info(idx) {
            cell_text.push_str(sort_info);
        }
        
        // Highlight if column search match
        if let Some(pattern) = column_search {
            if name.contains(pattern) {
                // Highlight this header
            }
        }
        
        header_cells.push(Cell::from(cell_text));
    }
    
    // Render with full context
    // ...
}
```