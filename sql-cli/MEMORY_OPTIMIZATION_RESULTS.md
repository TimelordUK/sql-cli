# Memory Optimization Results

## Problem
Loading a 20k row CSV file (trades_20k.csv with 53 columns) was using excessive memory:
- **Linux**: 49 MB → 156 MB (107 MB increase) after cloning DataTable
- **Windows**: Reported as 120 MB total (up from ~40 MB baseline)

## Root Cause
The `Buffer` was cloning the entire `DataTable` when storing it, causing memory duplication:
```rust
// OLD CODE - This was cloning the entire DataTable!
buffer.set_datatable(Some((*source_table).clone()));
```

## Solution
Converted `Buffer` to use `Arc<DataTable>` for shared ownership:

1. Changed Buffer fields:
```rust
pub datatable: Option<Arc<DataTable>>,
pub original_source: Option<Arc<DataTable>>,
```

2. Updated set_datatable to accept Arc:
```rust
fn set_datatable(&mut self, datatable: Option<Arc<DataTable>>)
```

3. Fixed the clone to share Arc instead:
```rust
// NEW CODE - Just increments Arc reference count!
buffer.set_datatable(Some(source_table));  // where source_table is Arc<DataTable>
```

## Results
**Memory usage for 20k row file (53 columns):**
- **Before**: 48 MB → 156 MB (108 MB increase) 
- **After**: 48 MB → 49 MB (1 MB increase) ✅

**Savings: 107 MB (69% reduction in memory usage)**

## Files Modified
- `src/buffer.rs` - Changed DataTable fields to Arc<DataTable>
- `src/ui/enhanced_tui.rs` - Use Arc sharing instead of cloning
- `src/data/data_view.rs` - Added source_arc() method
- `src/data/datatable_buffer.rs` - Updated trait implementation
- `src/ui/debug/context.rs` - Fixed buffer access

## Testing
Verified with Linux memory tracking:
```
[MEMORY[before_arc_share]: 48 MB
[MEMORY[after_arc_share]: 49 MB (+1 MB)
```

The 1 MB increase is just for the Arc wrapper and DataView metadata - the actual DataTable (37.73 MB) is now properly shared!