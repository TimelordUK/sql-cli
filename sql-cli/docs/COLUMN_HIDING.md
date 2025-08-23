# Column Hiding Feature

The column hiding feature allows you to temporarily hide columns from the results view without modifying your query or the underlying data.

## Keybindings

Due to terminal compatibility issues with Ctrl+H (often interpreted as backspace), we provide multiple keybindings:

### Hide Current Column
- **`-` (minus key)** - Recommended, works in all terminals
- **`Alt+H`** - Alternative binding
- **`Ctrl+H`** - May not work in some terminals

### Unhide All Columns
- **`+` or `=`** - Recommended
- **`Ctrl+Shift+H`** - Alternative binding

## How It Works

1. Navigate to the column you want to hide using arrow keys
2. Press `-` (minus) to hide the current column
3. The column is removed from view but data remains intact
4. Continue hiding other columns as needed
5. Press `+` or `=` to unhide all columns

## Technical Details

- Hidden columns are maintained in the `hidden_columns` list
- When a query is re-executed, the QueryEngine applies the hidden columns filter
- The DataView excludes hidden columns from the visible_columns indices
- The underlying DataTable remains immutable

## Limitations

- You cannot hide the last visible column
- Hidden columns are reset when switching between queries
- Column hiding is only available in Results mode