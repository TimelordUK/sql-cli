# TUI DataView Migration Guide

## Overview
We've removed legacy methods from the Buffer trait. The TUI needs to be updated to use DataView instead of the removed methods.

## Removed Methods Count (from compilation errors)
- `get_filtered_data` - 26 calls
- `is_csv_mode` - 21 calls  
- `is_cache_mode` - 19 calls
- `get_table_name` - 15 calls
- `get_csv_client` - 15 calls
- `get_hidden_columns` - 9 calls
- `set_filtered_data` - 7 calls
- Others: `set_table_name`, `set_csv_mode`, `set_csv_client`, `get_cached_data`, etc.

## Migration Patterns

### Pattern 1: Getting Row Count
**Old:**
```rust
self.buffer().get_filtered_data().map(|d| d.len()).unwrap_or(0)
```

**New:**
```rust
self.buffer().get_dataview().map(|v| v.row_count()).unwrap_or(0)
```

### Pattern 2: Checking for Data / Getting Schema
**Old:**
```rust
if self.buffer().is_csv_mode() {
    if let Some(csv_client) = self.buffer().get_csv_client() {
        // Get schema from csv_client
    }
} else if self.buffer().is_cache_mode() {
    // Handle cache mode
}
```

**New:**
```rust
if let Some(dataview) = self.buffer().get_dataview() {
    let columns = dataview.column_names(); // Gets visible columns
    let table_name = Some(dataview.source().name.clone()); // Gets table name
    (columns, table_name)
} else {
    (vec![], None)
}
```

### Pattern 3: Column Visibility (Hide/Unhide)
**Old:**
```rust
// Hide column
self.buffer_mut().add_hidden_column(col_name.clone());

// Get hidden columns
let hidden_columns = self.buffer().get_hidden_columns();

// Clear hidden columns
self.buffer_mut().clear_hidden_columns();
```

**New:**
```rust
// Hide column
if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
    dataview.hide_column_by_name(&col_name);
}

// Unhide all columns
if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
    dataview.unhide_all_columns();
}

// Check if columns are hidden
if let Some(dataview) = self.buffer().get_dataview() {
    let all_columns = dataview.source().column_names();
    let visible_columns = dataview.column_names();
    let has_hidden = all_columns.len() > visible_columns.len();
}
```

### Pattern 4: Filtered Data
**Old:**
```rust
self.buffer_mut().set_filtered_data(Some(filtered.clone()));
```

**New:**
DataView handles filtering internally - no need to set filtered data explicitly.

### Pattern 5: Getting Table Name
**Old:**
```rust
self.buffer().get_table_name()
```

**New:**
```rust
if let Some(dataview) = self.buffer().get_dataview() {
    dataview.source().name.clone()
} else {
    "data".to_string() // or some default
}
```

### Pattern 6: Checking if Data Exists
**Old:**
```rust
if self.buffer().is_csv_mode() || self.buffer().is_cache_mode() {
    // Has data
}
```

**New:**
```rust
if self.buffer().get_dataview().is_some() {
    // Has data
}
```

### Pattern 7: Getting Cached Data
**Old:**
```rust
if let Some(cached_data) = self.buffer().get_cached_data() {
    // Use cached_data
}
```

**New:**
This is legacy - DataView/DataTable is the universal format now. Access data through DataView's iteration methods.

## Key Principles
1. **DataView** is the single source of truth for:
   - Column visibility
   - Filtering
   - Row access
   - Column ordering

2. **DataTable** (via `dataview.source()`) provides:
   - Raw data
   - Table name
   - Original column definitions

3. **No more CSV/Cache modes** - everything uses DataTable/DataView

## Files to Update
- `src/ui/enhanced_tui.rs` - 123 errors
- `src/data/adapters/buffer_adapter.rs` - 4 errors
- Other files have fewer errors

## Testing After Changes
After making changes, run:
```bash
cargo check --all-targets
cargo test
cargo clippy
```