# V48: Use DataTable for Rendering

## Goal
Modify the rendering pipeline to use DataTable instead of JSON QueryResponse, while keeping JSON as a fallback for safety.

## Current Flow
1. BufferAdapter::get_row() reads from `buffer.get_results().data` (JSON)
2. Converts JSON values to Vec<String> for display
3. Type detection happens by parsing strings

## New Flow (V48)
1. BufferAdapter::get_row() checks if DataTable exists
2. If yes: Read from DataTable (typed values)
3. If no: Fall back to JSON (backward compatibility)
4. Type information comes directly from DataTable columns

## Implementation Steps

### 1. Modify BufferAdapter::get_row()
```rust
fn get_row(&self, index: usize) -> Option<Vec<String>> {
    // V48: Try DataTable first
    if let Some(datatable) = self.buffer.get_datatable() {
        // Read from DataTable
        return datatable.get_row(index);
    }
    
    // Fallback to JSON
    // ... existing code ...
}
```

### 2. Add DataTable methods to Buffer
- `get_datatable_row(index)` - Get row with filtering support
- `get_datatable_column_names()` - Get column names from DataTable
- `get_datatable_row_count()` - Get row count from DataTable

### 3. Update column type detection
- Use DataTable's column types directly
- No need to parse strings to detect types

### 4. Handle filtering with DataTable
- Fuzzy filter indices still work (they're just row indices)
- Regex filter needs to work with DataTable rows

## Benefits
1. **Performance**: No JSON parsing, direct typed access
2. **Memory**: No string duplication for numbers
3. **Type Safety**: Column types are known, not guessed
4. **Backward Compatible**: JSON fallback ensures nothing breaks

## Testing Strategy
1. Load CSV with F6 to confirm DataTable exists
2. Verify rendering shows same data
3. Test filtering still works
4. Check performance with large datasets

## Success Criteria
- ✅ Data displays correctly from DataTable
- ✅ All filters work (regex, fuzzy)
- ✅ Column types are correct
- ✅ No visual differences to user
- ✅ Performance improvement measurable