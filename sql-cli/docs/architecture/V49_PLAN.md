# V49: CsvApiClient Returns DataTable Directly

## Goal
Modify CsvApiClient to create DataTable directly when loading CSV/JSON, avoiding the JSON intermediate format.

## Current Flow
```
CSV File → CsvDataSource (Vec<Value>) → QueryResponse (JSON) → DataTable
```

## New Flow (V49)
```
CSV File → CsvDataSource → DataTable (direct creation)
         → QueryResponse (optional, for compatibility)
```

## Implementation Steps

### 1. Add DataTable creation to CsvDataSource
```rust
impl CsvDataSource {
    pub fn to_datatable(&self) -> DataTable {
        // Convert directly to DataTable
        // Headers → DataColumns
        // Records → DataRows with typed DataValues
    }
}
```

### 2. Add method to CsvApiClient
```rust
impl CsvApiClient {
    pub fn get_datatable(&self, table_name: &str) -> Option<DataTable> {
        self.datasource.as_ref().map(|ds| ds.to_datatable())
    }
}
```

### 3. Update Buffer to prefer DataTable
- When loading CSV, create DataTable immediately
- Store DataTable as primary data
- Keep QueryResponse for backward compatibility (for now)

### 4. Benefits
- **Less memory**: No intermediate JSON conversion
- **Faster loading**: Direct CSV → DataTable
- **Better types**: Types detected during CSV parsing
- **Backward compatible**: Still supports QueryResponse if needed

## Testing
1. Load CSV file
2. Verify DataTable is created directly
3. Check memory usage is lower
4. Ensure rendering still works