# Memory Optimization Plan - DataTable Duplication Issue

## Problem Summary
Loading a 20K row file with 50 columns increased memory usage from ~40MB to ~120MB. This is a 3x increase when it should be closer to 1.5x at most.

## Root Cause Analysis

### The Duplication
1. **DataView Creation**: When loading data, we create a DataView that holds an `Arc<DataTable>` to the original data
2. **Buffer Storage**: In `enhanced_tui.rs:1596`, we clone the entire DataTable: `buffer.set_datatable(Some((*source_table).clone()))`
3. **Original Preservation**: In `buffer.rs:531`, we clone again: `self.original_source = datatable.clone()`

**Result**: We have 3 copies of the same data!
- DataView's Arc<DataTable> (shared reference - good!)
- Buffer's datatable field (full clone - bad!)
- Buffer's original_source field (another full clone - bad!)

### Why We Need the Original
When users run queries with computed columns (e.g., `SELECT *, price * quantity as total`), we:
1. Create a new DataTable with the computed results
2. Need the original DataTable for the next query
3. Currently solve this by keeping a full clone

## Solution: Use Arc<DataTable> Everywhere

### Immediate Fix (Quick Win)
Change Buffer to store `Arc<DataTable>` instead of `DataTable`:

```rust
// buffer.rs
pub struct Buffer {
    pub datatable: Option<Arc<DataTable>>,     // Changed from Option<DataTable>
    pub original_source: Option<Arc<DataTable>>, // Changed from Option<DataTable>
    pub dataview: Option<DataView>,
    // ...
}
```

### Implementation Steps

1. **Update Buffer Structure**:
```rust
// In buffer.rs
impl BufferAPI for Buffer {
    fn set_datatable(&mut self, datatable: Option<Arc<DataTable>>) {
        // Just store the Arc, no cloning
        if datatable.is_some() && self.original_source.is_none() {
            self.original_source = datatable.clone(); // Arc clone is cheap!
        }
        self.datatable = datatable;
    }
}
```

2. **Update DataView Creation**:
```rust
// In enhanced_tui.rs
pub fn new_with_dataview(dataview: DataView, source_name: &str) -> Result<Self> {
    // ...
    let source_table = dataview.source(); // This is already Arc<DataTable>
    buffer.set_datatable(Some(source_table.clone())); // Arc clone, not data clone!
    // ...
}
```

3. **Update Query Engine**:
```rust
// When creating computed views, wrap new DataTable in Arc
let computed_table = DataTable::new("query_result");
// ... add computed columns ...
Ok(DataView::new(Arc::new(computed_table)))
```

## Expected Memory Savings

### Current Memory Usage (20K rows × 50 columns)
- Original DataTable: ~40MB
- Clone for buffer.datatable: ~40MB
- Clone for buffer.original_source: ~40MB
- **Total: ~120MB**

### After Optimization
- Original DataTable: ~40MB
- Arc references: ~16 bytes each
- **Total: ~40MB**

**Savings: 80MB (67% reduction!)**

## Additional Optimizations

### 1. String Interning
Many columns have repeated values (e.g., product names, categories). String interning could save 20-50% on string columns.

### 2. Column Compression
For columns with low cardinality, use dictionary encoding internally.

### 3. Lazy Loading
Only load columns that are actually used in queries.

## Testing Plan

1. Add memory benchmarks before changes
2. Implement Arc<DataTable> changes
3. Verify memory reduction with trades_20k.csv
4. Ensure query functionality still works:
   - Computed columns
   - Multiple queries
   - Filters and sorting

## Implementation Priority

1. **High Priority**: Fix DataTable cloning (immediate 67% memory savings)
2. **Medium Priority**: Add memory monitoring to F5 debug view
3. **Low Priority**: String interning and compression (additional 20-30% savings)

## Code Changes Required

### Files to Modify:
1. `src/buffer.rs` - Change DataTable fields to Arc<DataTable>
2. `src/ui/enhanced_tui.rs` - Update to use Arc clones
3. `src/data/query_engine.rs` - Wrap computed tables in Arc
4. `src/services/query_execution_service.rs` - Handle Arc<DataTable>
5. `src/app_state_container.rs` - Update buffer access methods

### Estimated Effort: 
- 2-3 hours for core changes
- 1 hour for testing
- 1 hour for cleanup and documentation

## Backwards Compatibility
All changes can be made internally without affecting the public API. The BufferAPI trait methods can remain the same, just handling Arc internally.