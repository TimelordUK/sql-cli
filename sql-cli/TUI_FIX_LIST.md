# TUI Compilation Fixes Needed

## DataView is Working!
The standalone test confirms DataView has all the functionality needed:
- Column search ✓
- Text filtering ✓ 
- Sorting ✓
- Column visibility ✓
- All combined operations ✓

## Errors to Fix (42 total)

### 1. Missing methods on BufferAPI (most common)
- `get_filtered_data()` - Use `get_dataview()` instead
- `get_hidden_columns()` - Use `get_dataview()?.get_hidden_column_names()`
- `is_csv_mode()` - Remove, no longer needed
- `is_cache_mode()` - Remove, no longer needed
- `get_table_name()` - Use `get_dataview()?.source().name` or similar
- `get_csv_client()` - Remove, no longer needed

### 2. DataView method issues
- `len()` not found - Use `row_count()` instead

### 3. Borrow checker issues (8 total)
- Multiple mutable borrows - Need to restructure code to get values before mutating
- Pattern: Get case_insensitive before getting mutable dataview

### 4. Removed AppMode variants
- `AppMode::CacheList` - Remove these branches

### 5. Method signature changes
- Some methods now take different arguments

## Quick Fix Strategy

1. **For removed Buffer methods**: Replace with DataView equivalents
2. **For borrow checker**: Extract values to variables before mutable operations
3. **For removed modes**: Delete the match arms
4. **For DataView**: Use the correct method names (row_count not len)

## Example Fixes

### Before (get_filtered_data)
```rust
if let Some(filtered) = self.buffer().get_filtered_data() {
    // use filtered
}
```

### After (use DataView)
```rust
if let Some(dataview) = self.buffer().get_dataview() {
    let row_count = dataview.row_count();
    // use dataview
}
```

### Before (borrow checker issue)
```rust
if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
    dataview.apply_text_filter(pattern, !self.buffer().is_case_insensitive());
}
```

### After (extract value first)
```rust
let case_insensitive = self.buffer().is_case_insensitive();
if let Some(dataview) = self.buffer_mut().get_dataview_mut() {
    dataview.apply_text_filter(pattern, !case_insensitive);
}
```

## The Good News
- DataView is fully functional and tested
- Only 42 compilation errors left
- Most are simple method replacements
- Once these are fixed, the TUI will work with the new DataView architecture!