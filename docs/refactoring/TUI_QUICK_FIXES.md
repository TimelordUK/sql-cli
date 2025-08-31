# Quick TUI Compilation Fixes

## 1. Hidden Column Methods
**Replace:**
```rust
self.buffer_mut().add_hidden_column(col_name.clone());
```
**With:**
```rust
if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
    dataview.hide_column_by_name(&col_name);
}
```

**Replace:**
```rust
self.buffer_mut().clear_hidden_columns();
```
**With:**
```rust
if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
    dataview.unhide_all_columns();
}
```

**Replace:**
```rust
buffer.get_hidden_columns()
```
**With:**
```rust
buffer.get_dataview().map(|v| v.get_hidden_column_names()).unwrap_or_default()
```

## 2. DataView len() → row_count()
**Replace:**
```rust
dataview.len()
```
**With:**
```rust
dataview.row_count()
```

## 3. Filter/Sort Method Signatures
**Replace:**
```rust
dataview.apply_text_filter(pattern)  // Missing case_sensitive
```
**With:**
```rust
let case_sensitive = !self.buffer().is_case_insensitive();
dataview.apply_text_filter(pattern, case_sensitive)
```

**Replace:**
```rust
dataview.apply_fuzzy_filter(pattern)  // Missing case_insensitive
```
**With:**
```rust
let case_insensitive = self.buffer().is_case_insensitive();
dataview.apply_fuzzy_filter(pattern, case_insensitive)
```

## 4. Remove CacheList Mode
**Delete these match arms:**
```rust
AppMode::CacheList => { /* delete entire block */ }
```

## 5. Remove CSV/Cache methods
**Delete calls to:**
- `is_csv_mode()`
- `is_cache_mode()`
- `get_csv_client()`
- `get_table_name()` - Use `get_dataview()?.source().name` instead
- `handle_cache_command()`

## 6. Borrow Checker Fixes
**Pattern - extract value before mutable borrow:**
```rust
// WRONG - borrows self twice
if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
    dataview.apply_text_filter(pattern, !self.buffer().is_case_insensitive());
}

// RIGHT - get value first
let case_insensitive = self.buffer().is_case_insensitive();
if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
    dataview.apply_text_filter(pattern, !case_insensitive);
}
```

## 7. Column Search Migration
**Old (AppStateContainer):**
```rust
self.state_container.start_column_search(pattern);
self.state_container.column_search().matching_columns
```

**New (DataView):**
```rust
if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
    dataview.search_columns(&pattern);
    let matches = dataview.get_matching_columns();
}
```

## Common Patterns

### Get row count
```rust
let row_count = self.buffer()
    .get_dataview()
    .map(|v| v.row_count())
    .unwrap_or(0);
```

### Get column names
```rust
let columns = self.buffer()
    .get_dataview()
    .map(|v| v.column_names())
    .unwrap_or_default();
```

### Check if filtered
```rust
let is_filtered = self.buffer()
    .get_dataview()
    .map(|v| v.has_filter())
    .unwrap_or(false);
```

### Get table name
```rust
let table_name = self.buffer()
    .get_dataview()
    .map(|v| v.source().name.clone())
    .unwrap_or_else(|| "data".to_string());
```