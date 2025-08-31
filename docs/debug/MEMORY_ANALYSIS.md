# Memory Analysis - Where is the memory going?

## Current Observations
- 10k rows = ~350MB (35KB per row!)
- 20k rows = ~680MB (34KB per row)
- Linear scaling suggests constant overhead per row

## Data Structure Analysis

### For 10k trade rows with ~50 fields each:

#### 1. CsvDataSource (JSON Storage)
```
Vec<Value> where each Value is a JSON object
- Each row: HashMap with 50 entries
- Each entry: String key (field name) + Value
- Field names: 50 fields × 20 chars avg × 10k rows = 10MB just for field names!
- Values: 50 fields × 20 chars avg × 10k rows = 10MB for data
- HashMap overhead: ~40 bytes per entry × 50 × 10k = 20MB
- serde_json::Value enum overhead: 24 bytes × 50 × 10k = 12MB
Total: ~52MB minimum
```

#### 2. QueryResponse (Another JSON Copy)
```
Same as above: ~52MB
```

#### 3. DataTable (Our new format)
```
- Column metadata: 50 × ~100 bytes = 5KB (negligible)
- Row storage: 10k × 50 × DataValue size
- DataValue enum: ~32 bytes (discriminant + largest variant)
- Actual data: 10k × 50 × 20 bytes avg = 10MB
Total: ~26MB (more efficient!)
```

#### 4. Buffer.filtered_data (When filtering)
```
Vec<Vec<String>> - all data as strings
- 10k × 50 × 20 chars = 10MB minimum
- String overhead: 24 bytes × 50 × 10k = 12MB
Total: ~22MB
```

### Total So Far
52MB + 52MB + 26MB + 22MB = 152MB

But we're seeing 350MB! Where's the other 200MB?

## Hidden Memory Consumers

### 1. String Allocations
Every String in Rust has:
- Pointer (8 bytes)
- Capacity (8 bytes)  
- Length (8 bytes)
- Actual data (variable)

So a 20-char string takes 24 + 20 = 44 bytes!

### 2. Vector Overhead
Vectors allocate with growth factor (usually 2x):
- If you push 10k items, vector might allocate space for 16k

### 3. Memory Fragmentation
- Heap fragmentation from many small allocations
- Rust's allocator (jemalloc/system) overhead

### 4. Ratatui Rendering
Every frame (60fps):
- Creates Row objects for visible rows
- Creates Cell objects (50 fields × 30 visible rows = 1500 Cells)
- Each Cell contains a cloned String
- Style information per cell

Even if transient, these allocations fragment memory.

### 5. Hidden Copies
- Input buffer
- History storage
- Undo/redo stacks
- Clipboard/kill ring
- Various caches

## Experiment: Limit Data to TUI

Let's test if Ratatui is the problem by limiting what we pass to rendering:

```rust
// In BufferAdapter::get_row_count()
fn get_row_count(&self) -> usize {
    // EXPERIMENT: Only tell TUI about first 1000 rows
    let actual_count = /* normal calculation */;
    actual_count.min(1000)  // Cap at 1000
}
```

If memory drops significantly, we know Ratatui is creating objects for all rows.

## Solutions

### Short Term
1. **Limit rendered rows**: Only pass visible rows to Ratatui
2. **Remove duplicate storage**: V50 - eliminate JSON after DataTable creation
3. **String interning**: Reuse field name strings

### Long Term  
1. **True virtual scrolling**: Don't create ANY objects for non-visible rows
2. **Custom allocator**: Use arena allocator for temporary objects
3. **Streaming**: Don't load entire CSV, stream chunks
4. **Memory-mapped files**: Use mmap for large CSVs

## Testing Plan
1. Load 10k CSV, press F6, note process memory
2. Modify get_row_count() to return min(1000, actual)
3. Reload, press F6, compare memory
4. If significantly lower, Ratatui is the culprit