# Clean Fuzzy Filter Implementation with DataView

## The Problem
The current `apply_fuzzy_filter()` function is 130+ lines of complex state management, trying to:
- Apply fuzzy filter on top of regex filters
- Manage filtered_data separately
- Sync multiple states
- Handle clearing filters

## The Solution: DataView Handles Everything

### Add fuzzy filter support to DataView

First, extend DataView to support fuzzy filtering:

```rust
// In src/data/data_view.rs

impl DataView {
    /// Apply a fuzzy filter to the view
    /// This filters on top of any existing filters
    pub fn apply_fuzzy_filter(&mut self, pattern: &str, matcher: &SkimMatcherV2) {
        if pattern.is_empty() {
            // Clear fuzzy filter but keep other filters
            self.clear_fuzzy_filter();
            return;
        }
        
        // Filter visible_rows based on fuzzy matching
        self.visible_rows = self.visible_rows
            .iter()
            .copied()
            .filter(|&row_idx| {
                // Get row text for matching
                let row_text = self.source.get_row_as_string(row_idx);
                matcher.fuzzy_match(&row_text, pattern).is_some()
            })
            .collect();
    }
    
    /// Clear only the fuzzy filter (restore to previous filter state)
    pub fn clear_fuzzy_filter(&mut self) {
        // This would restore visible_rows to the state before fuzzy filter
        // You'd need to track this, or reapply other filters
        // For now, let's assume we rebuild from other filters
        self.rebuild_filters();
    }
    
    fn rebuild_filters(&mut self) {
        // Start with all rows
        self.visible_rows = (0..self.source.row_count()).collect();
        
        // Reapply any other filters (regex, etc.)
        // This would use stored filter state
    }
}
```

### Simplified TUI Function

```rust
fn apply_fuzzy_filter(&mut self) {
    let pattern = self.buffer().get_fuzzy_filter_pattern();
    
    // Apply or clear fuzzy filter on the DataView
    if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
        if pattern.is_empty() {
            debug!(target: "fuzzy", "Clearing fuzzy filter");
            dataview.clear_fuzzy_filter();
            self.buffer_mut().set_fuzzy_filter_active(false);
            self.buffer_mut().set_status_message("Fuzzy filter cleared".to_string());
        } else {
            debug!(target: "fuzzy", "Applying fuzzy filter: {}", pattern);
            let matcher = SkimMatcherV2::default();
            dataview.apply_fuzzy_filter(&pattern, &matcher);
            self.buffer_mut().set_fuzzy_filter_active(true);
            
            let visible_count = dataview.row_count();
            self.buffer_mut().set_status_message(
                format!("Fuzzy filter: {} matches", visible_count)
            );
        }
    }
    
    // That's it! No state syncing needed.
    // The DataView is the single source of truth.
}
```

## Benefits

1. **130+ lines → ~20 lines**
2. **No state synchronization** - DataView IS the state
3. **No filtered_data management** - DataView tracks visible rows
4. **Composable filters** - Fuzzy on top of regex just works
5. **Easy to clear** - Just reset the DataView filter

## How It Works

1. **DataView tracks all filters** - regex, fuzzy, column filters, etc.
2. **Filters compose naturally** - Each filter reduces visible_rows further
3. **Clearing is simple** - Remove one filter layer, keep others
4. **TUI just displays** - Whatever rows DataView says are visible

## Implementation Strategy

Instead of fixing the complex `apply_fuzzy_filter()` function, we should:

1. Add filter methods to DataView
2. Track filter state in DataView (so we can clear individual filters)
3. Replace the complex TUI function with the simple version
4. Remove all the state sync code

This aligns with the Redux philosophy - the view layer (TUI) shouldn't manage state, it should just display what the data layer (DataView) provides.

## Filter State in DataView

Consider adding a filter stack to DataView:

```rust
pub struct DataView {
    source: Arc<DataTable>,
    visible_rows: Vec<usize>,
    visible_columns: Vec<usize>,
    
    // Filter stack - can pop individual filters
    filters: Vec<Filter>,
}

enum Filter {
    Regex(String),
    Fuzzy(String),
    Column { column: usize, predicate: Box<dyn Fn(&DataValue) -> bool> },
}
```

This way you can easily add, remove, or clear specific filters without losing others.