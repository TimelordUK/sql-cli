# V49: Direct DataTable Creation from CSV/JSON

## What We Implemented

### CsvDataSource Changes
1. **Added `to_datatable()` method**
   - Converts directly from CSV data to DataTable
   - No JSON intermediate step
   - Column types inferred during conversion

2. **Added helper `json_value_to_data_value()`**
   - Maps JSON values to typed DataValues
   - Detects dates, numbers, booleans

### CsvApiClient Changes
1. **Added `get_datatable()` method**
   - Returns DataTable directly from datasource
   - Called after CSV/JSON loading
   - Bypasses QueryResponse conversion

### Enhanced TUI Changes
1. **Updated CSV loading**
   - Tries to get DataTable directly from CsvApiClient
   - Falls back to JSON conversion if needed

2. **Updated JSON loading**
   - Same as CSV - direct DataTable creation
   - Maintains backward compatibility

## The New Flow

### Before (V48):
```
CSV → CsvDataSource (JSON) → QueryResponse → DataTable
```

### Now (V49):
```
CSV → CsvDataSource → DataTable (direct!)
                    ↘
                      QueryResponse (still available for compatibility)
```

## Benefits
1. **Less Memory**: No duplicate JSON conversion step
2. **Faster Loading**: Direct CSV to DataTable
3. **Better Types**: Types detected during initial parse
4. **Still Compatible**: QueryResponse still works if needed

## What's Still Using JSON
- Query execution still returns QueryResponse
- Filtering still uses JSON values
- This is OK for now - we're taking small steps!

## Next Steps (V50+)
- Make query execution return DataTable
- Remove JSON storage from Buffer
- Complete the migration