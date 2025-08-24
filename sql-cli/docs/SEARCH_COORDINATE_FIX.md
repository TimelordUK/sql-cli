# Search Coordinate Mismatch Fix

## Problem
When searching for "emerging" in a 20k row dataset, the search finds matches but navigates to wrong cells. For example, searching "emerging" lands on row 0, column 2 which shows "Derivatives" instead of a cell containing "emerging".

## Root Cause
The SearchManager is searching through DataView data which is already filtered/sorted/column-reordered, and returning coordinates in that space. However, there may be a mismatch between:

1. **Data collected for search**: Using `dataview.get_row()` which returns data in display order
2. **Navigation coordinates**: Using the same coordinates but potentially misaligned with viewport

## Key Findings

### DataView Transformations
```rust
// DataView.get_row() applies:
// 1. Row filtering (visible_rows)
// 2. Column reordering (display_columns = pinned + visible)
// 3. Offset/limit
let row = dataview.get_row(index);
```

### Search Data Collection
```rust
// We collect data respecting all DataView transformations
let data: Vec<Vec<String>> = (0..dataview.row_count())
    .filter_map(|i| dataview.get_row(i))
    .map(|row| row.values.iter().map(|v| v.to_string()).collect())
    .collect();
```

### Coordinate Spaces
1. **DataTable space**: Raw data, all rows/columns
2. **DataView space**: Filtered rows, reordered columns
3. **Viewport space**: Currently visible portion
4. **Visual space**: What user sees on screen

## The Fix

### Immediate Solution
Add comprehensive logging to understand the transformation at each level:

```rust
// When searching
info!("Search data row {}: {:?}", row_index, row_data);

// When navigating  
info!("Navigate to ({}, {}), value at target: '{}'", row, col, value);

// Compare what we searched vs what we land on
```

### Long-term Solution
Create a unified coordinate mapper:

```rust
pub struct CoordinateMapper {
    // Maps between different coordinate spaces
    dataview_to_viewport: fn(row: usize, col: usize) -> (usize, usize),
    viewport_to_visual: fn(row: usize, col: usize) -> (usize, usize),
}
```

## Verification Steps

1. Create test file with known "emerging" positions
2. Search for "emerging" 
3. Log:
   - Data being searched (first few rows)
   - Matches found with values
   - Navigation target coordinates
   - Actual value at navigation target
4. Compare logged values to identify mismatch

## Hypothesis
The most likely issue is that when we have many columns (53 in trades_20k.csv), the column ordering might be different between:
- The data we collect for search (DataView's display order)
- The column index we use for navigation (might need viewport adjustment)

## Next Steps
1. Add detailed logging as shown above
2. Test with small dataset to verify coordinates
3. Test with large dataset (trades_20k.csv) to reproduce issue
4. Implement CoordinateMapper if needed