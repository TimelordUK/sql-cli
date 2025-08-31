# Column Search Fix Verification

## Problem
Column search was not working with DataTableBuffer (used for CSV files) because:
1. `DataTableBuffer::get_results()` returns `None` (it doesn't use QueryResponse)
2. The `search_columns()` function only looked for columns in JSON results
3. Without columns, no matches could be found and the cursor wouldn't move

## Solution
1. Added `get_column_names()` method to `BufferAPI` trait
2. Implemented it in both `Buffer` (for JSON results) and `DataTableBuffer` (for DataTable)
3. Updated `search_columns()` to use the unified `get_column_names()` method

## Changes Made

### File: src/buffer.rs
- Added `fn get_column_names(&self) -> Vec<String>` to BufferAPI trait
- Implemented it to extract column names from JSON results

### File: src/datatable_buffer.rs
- Implemented `get_column_names()` to return column names from DataTable:
```rust
fn get_column_names(&self) -> Vec<String> {
    self.view.table().columns.iter()
        .map(|col| col.name.clone())
        .collect()
}
```

### File: src/enhanced_tui.rs
- Simplified `search_columns()` to use the new unified method:
```rust
let column_names = self.buffer().get_column_names();
let mut columns = Vec::new();
for (index, col_name) in column_names.iter().enumerate() {
    columns.push((col_name.clone(), index));
}
```

## Testing
To test the fix:
1. Run: `./target/release/sql-cli data/instruments.csv`
2. Press Enter to go to Results mode
3. Press '\' to enter column search mode
4. Type a partial column name (e.g., "inst" for "instrument_id")
5. The cursor should now move to the matched column

## Result
Column search now works correctly with:
- JSON API responses (standard mode)
- CSV files with CsvApiClient
- DataTableBuffer (new DataTable architecture)

The fix ensures that regardless of the data source, column names are consistently retrieved and searchable.