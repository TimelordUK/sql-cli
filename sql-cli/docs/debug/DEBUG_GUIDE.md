# DataView Debugging Guide

## Quick Start

### 1. Compile with Debug Symbols
```bash
rustc -g test_dataview_debug.rs -o test_dataview_debug
```

### 2. Run Normally (Interactive)
```bash
./test_dataview_debug
```
This version has pause points and debug output to see what's happening.

## Debugging Methods

### Method 1: GDB (GNU Debugger)
```bash
# Start GDB
rust-gdb ./test_dataview_debug

# In GDB:
(gdb) break DataView::search_columns    # Set breakpoint
(gdb) break DataView::apply_text_filter # Another breakpoint
(gdb) run                                # Start program
(gdb) print self                         # Inspect DataView
(gdb) print pattern                      # Inspect variables
(gdb) continue                           # Continue execution
(gdb) step                               # Step into function
(gdb) next                               # Step over line
```

### Method 2: LLDB 
```bash
# Start LLDB
rust-lldb ./test_dataview_debug

# In LLDB:
(lldb) b DataView::search_columns       # Set breakpoint
(lldb) run                               # Start program
(lldb) print self                        # Inspect DataView
(lldb) frame variable                    # Show all variables
(lldb) continue                          # Continue
```

### Method 3: VSCode
1. Open `test_dataview_debug.rs` in VSCode
2. Click in the gutter to add breakpoints (red dots)
3. Add this `launch.json` to `.vscode/`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug DataView",
            "program": "${workspaceFolder}/test_dataview_debug",
            "args": [],
            "cwd": "${workspaceFolder}",
            "preLaunchTask": "cargo build"
        }
    ]
}
```

4. Press F5 to start debugging

### Method 4: Print Debugging
The debug version already includes helpful output:
- `debug_state()` - Shows internal state
- `debug_visible_data()` - Shows current visible data
- Automatic logging in search/filter/sort operations

## Key Breakpoint Locations

### Column Search
```rust
// Line ~170 in search_columns()
if col_name.to_lowercase().contains(&pattern_lower) {
    // BREAKPOINT HERE - See when columns match
}
```

### Filtering
```rust
// Line ~280 in apply_text_filter()
if text.contains(&pattern_lower) {
    // BREAKPOINT HERE - See when rows match
    return true;
}
```

### State Changes
```rust
// Any line that modifies:
self.visible_rows = ...      // Row visibility changes
self.visible_columns = ...   // Column visibility changes
self.matching_columns = ...  // Search results change
```

## What to Inspect

### DataView State
- `visible_rows` - Which row indices are visible
- `visible_columns` - Which column indices are visible  
- `base_rows` - Original rows (preserved through filters)
- `matching_columns` - Column search results
- `filter_pattern` - Active filter

### During Column Search
1. Watch `pattern_lower` get created
2. See each column name comparison
3. Watch `matching_columns` vector build up
4. See `current_column_match` reset to 0

### During Filtering
1. Watch each row get tested
2. See which values match the pattern
3. Watch `visible_rows` shrink

## Example Debug Session

```bash
rust-gdb ./test_dataview_debug

(gdb) break DataView::search_columns
(gdb) run
# Program pauses at column search
(gdb) print pattern
$1 = "a"
(gdb) next
# Step through and watch matches
(gdb) print self.matching_columns
$2 = Vec([(1, "name"), (2, "amount"), (3, "category"), (4, "active")])
(gdb) continue
```

## Tips

1. **Use the pause points** - The interactive version stops at key moments
2. **Watch the debug output** - It shows you what's happening internally
3. **Set conditional breakpoints** - Break only when pattern == "specific"
4. **Use watchpoints** - Break when visible_rows changes
5. **Print the entire view** - `print *self` in GDB shows everything

## Testing Specific Issues

### "Why isn't my column search finding anything?"
1. Set breakpoint in `search_columns` at the filter_map
2. Check if `pattern_lower` is correct
3. Check each `col_name.to_lowercase()` comparison
4. Verify `visible_columns` has the expected indices

### "Why is my filter not working?"
1. Set breakpoint in `apply_text_filter` inside the filter closure
2. Check each row's values
3. Verify the pattern matching logic
4. Check if `base_rows` has the expected data

### "Why is sort not preserving through filter?"
1. Check that sort updates `base_rows` after sorting
2. Verify filter starts from `base_rows`, not original indices
3. Ensure clear_filter restores to `base_rows`