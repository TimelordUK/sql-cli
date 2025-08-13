# Memory Usage Analysis for 100k Row CSV

## Current Data Duplication Issue

When loading a 100k row CSV file, the data is stored multiple times:

### 1. CsvDataSource (src/data/csv_datasource.rs)
- Stores data as `Vec<serde_json::Value>` 
- Each row is a JSON object with field names duplicated

### 2. QueryResponse (src/api_client.rs) 
- Contains `data: Vec<Value>` - another copy of the JSON data
- Stored in Buffer.results

### 3. Buffer.filtered_data (optional)
- When filtering: `Vec<Vec<String>>` - string representation of filtered rows

### 4. Buffer.cached_data (optional)
- Another `Vec<serde_json::Value>` for caching

## Memory Overhead Calculation

For a typical trade record with 7 fields:
```
{
  "id": 12345,
  "symbol": "AAPL", 
  "price": 150.25,
  "quantity": 100,
  "timestamp": "2024-01-15T10:30:00Z",
  "side": "BUY",
  "exchange": "NASDAQ"
}
```

### JSON Object Overhead:
- Field names: ~50 bytes × 100k rows = 5MB
- serde_json::Value enum tags: 8 bytes × 7 fields × 100k = 5.6MB  
- HashMap overhead: ~40 bytes × 100k = 4MB
- String allocations: Each string value has its own allocation

### Total Memory Usage:
- Raw data: ~100 bytes × 100k = 10MB
- JSON representation: ~300 bytes × 100k = 30MB
- Multiple copies: 30MB × 2-3 = 60-90MB minimum
- Plus heap fragmentation and allocator overhead

**Result: 10MB of actual data becomes 100MB+ in memory**

## Solution Options

### Short-term Fix (V46)
1. Remove duplicate storage of cached_data when not needed
2. Use indices instead of copying filtered data
3. Clear unused data after loading

### Long-term Fix (V50+)
1. Migrate to DataTable with columnar storage
2. Store data only once in efficient format
3. Use views/indices for filtering and sorting
4. Lazy loading for large datasets

## Immediate Recommendation

For V46, we should:
1. Avoid storing `cached_data` unless actually caching
2. Use filter indices instead of `filtered_data` copies  
3. Implement streaming for large CSV files
4. Consider compression for string columns