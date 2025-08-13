# Memory Investigation Results

## Problem
When loading 20k rows, memory usage jumps from 178 MB to 700 MB and is never returned.

## Root Causes Found

### 1. QueryResponse with JSON Intermediate Format
- CSV data is converted to `serde_json::Value` objects (154 MB for 20k rows)
- This is ~3x larger than the raw data due to JSON overhead
- JSON tree structure with heap allocations for every value

### 2. Multiple Clones of QueryResponse
The execute_query function creates multiple clones:
```rust
// Line 2945 - Clone for AppStateContainer
self.state_container.set_results(response.clone(), duration, from_cache)

// Line 2955 - Clone for caching
self.state_container.cache_results(query_key, response.clone())  

// Previously also had duplicate buffer updates (now fixed)
```

### 3. Memory Never Released
- AppStateContainer keeps the full QueryResponse in memory
- Cache keeps another full copy of QueryResponse
- These are never freed until new query replaces them

## Memory Breakdown (20k rows)
```
Before query: 178 MB
After CSV query (JSON created): 332 MB (+154 MB) 
After first conversion: 485 MB (+153 MB from clone)
After DataTable creation: 546 MB (+61 MB)
Final steady state: 700 MB

Total: 522 MB for data that should be ~50 MB
```

## Solutions

### Immediate Fixes
1. ✅ Remove duplicate buffer updates (saves 154 MB)
2. Pass QueryResponse by value instead of cloning where possible
3. Clear/drop QueryResponse after DataTable conversion

### Architectural Fixes (for DataView phase)
1. **Skip JSON intermediate** - Load CSV directly to DataTable
2. **Cache DataTable, not QueryResponse** - Much more memory efficient
3. **Use references/views** - DataView should reference DataTable, not copy

### Code Locations
- `src/ui/enhanced_tui.rs:2837` - CSV creates QueryResponse with JSON
- `src/ui/enhanced_tui.rs:2945` - Clone for AppStateContainer  
- `src/ui/enhanced_tui.rs:2955` - Clone for cache
- `src/app_state_container.rs` - Stores full QueryResponse
- `src/data/csv_datasource.rs` - Creates JSON intermediate

## Expected Memory Usage After Fix
With DataTable-only approach:
- Raw data: ~50 MB (20k rows × 50 cols × 50 bytes average)
- DataTable overhead: ~10-20 MB
- Total: ~70 MB instead of 700 MB (10x reduction!)

## Next Steps
1. V51-55: Implement DataView layer
2. V56+: Refactor to eliminate QueryResponse/JSON intermediate
3. Direct CSV → DataTable conversion
4. Cache DataTables instead of QueryResponse