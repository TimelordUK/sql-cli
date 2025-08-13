# V48: Use DataTable for Rendering

## What We've Implemented

### BufferAdapter Changes (`src/data/adapters/buffer_adapter.rs`)

1. **get_row()** - Now checks for DataTable first:
   - If DataTable exists, reads directly from it
   - Converts DataValue to String for display
   - Falls back to JSON if no DataTable

2. **get_column_names()** - Uses DataTable columns:
   - Returns column names from DataTable if available
   - Falls back to JSON extraction

3. **get_row_count()** - Uses DataTable row count:
   - Returns DataTable row count if available
   - Respects filters (fuzzy, regex)
   - Falls back to JSON count

4. **get_column_type()** - Uses DataTable column types:
   - Maps DataTable types to DataProvider types
   - No string parsing needed - types are known!
   - Falls back to type detection for JSON

5. **get_column_types()** - Returns all column types:
   - Direct mapping from DataTable column metadata
   - Much more accurate than string parsing

## How It Works

```rust
// When rendering, BufferAdapter now does:
if let Some(datatable) = self.buffer.get_datatable() {
    // Use DataTable - faster, typed access
    return datatable_row;
} else {
    // Fall back to JSON - backward compatible
    return json_row;
}
```

## Benefits

1. **Performance**: No JSON parsing on each row access
2. **Type Safety**: Column types are definitive, not guessed
3. **Memory**: Direct access to typed values
4. **Backward Compatible**: JSON fallback ensures nothing breaks

## Testing

To test V48 rendering:
1. Load a CSV file: `./target/release/sql-cli test.csv`
2. Run a query: `select * from data`
3. Press F6 to ensure DataTable is created
4. Data should display normally
5. Check logs for "V48: Using DataTable" messages

## Type Mapping

DataTable Type → DataProvider Type:
- String → Text
- Integer → Integer  
- Float → Float
- Boolean → Boolean
- DateTime → Date
- Null → Unknown
- Mixed → Text

## Next Steps (V49)

Once we confirm V48 is working:
- Remove JSON storage entirely
- Buffer will only have DataTable
- Significant memory savings
- Simpler codebase