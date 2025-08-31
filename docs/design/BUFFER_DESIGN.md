# Buffer System Design for SQL CLI

## Overview

The buffer system allows multiple files to be loaded simultaneously, with each maintaining its own independent state. This is similar to vim's buffer system or browser tabs.

## Key Features

### 1. Independent State Per Buffer
Each buffer maintains:
- Loaded data (CSV/JSON)
- Current SQL query
- All filters and searches
- Cursor position and scroll state
- Sort order
- Column pins
- View preferences (compact mode, row numbers, etc.)

### 2. Quick Switching
Switch between buffers without losing any state:
- Your position in the data
- Active filters
- Current query
- Everything is preserved

## Keybindings

### Buffer Navigation (Vim-style)
- `gt` or `Ctrl+Tab` - Next buffer
- `gT` or `Ctrl+Shift+Tab` - Previous buffer  
- `{n}gt` - Go to buffer n (e.g., `2gt` for second buffer)
- `Alt+1` through `Alt+9` - Quick jump to buffer 1-9

### Buffer Management
- `:e filename.csv` - Open file in new buffer
- `:edit filename.json` - Same as :e
- `:bd` or `:bdelete` - Close current buffer
- `:ls` or `:buffers` - List all open buffers
- `:b {n}` or `:buffer {n}` - Switch to buffer n
- `:bn` or `:bnext` - Next buffer
- `:bp` or `:bprev` - Previous buffer

## Visual Design

### Tab Bar
```
┌─────────────────────────────────────────────────────┐
│ [1:trades.csv] [2:products.json*] [3:customers.csv] │
├─────────────────────────────────────────────────────┤
│ SELECT * FROM products WHERE price > 100            │
└─────────────────────────────────────────────────────┘
```

- Current buffer is highlighted
- `*` indicates modified/filtered state
- Number prefix for quick jumping

### Status Line Addition
```
[NAV] Row 45/1000 | 📁 FILE | Buffer 2/3 | Enter:Run
```

## Implementation Strategy

### Phase 1: Basic Buffer Support
1. Extract current state into Buffer struct
2. Add BufferManager to handle multiple buffers
3. Implement basic switching (gt/gT)

### Phase 2: File Operations
1. Add :e command to open files
2. Implement :bd to close buffers
3. Add duplicate file detection

### Phase 3: Visual Polish
1. Add tab bar widget
2. Show buffer indicators in status line
3. Add buffer list view (:ls)

## Use Cases

### Comparing Data
```
1. Load trades.csv
2. Filter to specific date range
3. :e reference_data.csv
4. Search for specific values
5. gt to switch back to trades
6. Your filter is still active!
```

### Multi-File Analysis
```
1. Open main dataset
2. :e lookup_table.csv for reference
3. :e config.json for settings
4. Use Alt+1, Alt+2, Alt+3 to quickly jump between them
```

## Benefits

1. **No Context Loss** - Each file keeps its state
2. **Quick Comparison** - Rapidly switch between datasets
3. **Familiar Interface** - Vim users will feel at home
4. **Efficient Workflow** - No need to restart for new files

## Technical Considerations

### Memory Management
- Each buffer holds its own data copy
- Consider max buffer limit for large files
- Lazy loading possible for unused buffers

### State Preservation
- All UI state moves into Buffer struct
- App becomes primarily a buffer manager
- Clean separation of concerns

### Ratatui Compatibility
- Fully compatible with ratatui's architecture
- Tab bar is just another widget
- State management remains straightforward

## Future Enhancements

### Advanced Features (Not Initial Scope)
- Split panes (view 2 buffers side by side)
- Buffer groups/sessions
- Save/restore buffer sessions
- Quick buffer switching with fuzzy finder

### Integration Ideas
- Open query results in new buffer
- Compare buffers with diff view
- Transfer filters between buffers
- Linked scrolling for comparison

## Command Examples

```sql
-- In buffer 1 (trades.csv)
SELECT * FROM trades WHERE date > '2024-01-01'

-- User presses :e products.csv
-- Now in buffer 2 (products.csv)
SELECT * FROM products

-- User presses gt
-- Back in buffer 1, filter still active

-- User types :ls
-- Output:
1: trades.csv (filtered)
2: products.csv
3: customers.csv

-- User types :b3
-- Switches to customers.csv
```

This design keeps the tool focused on its core purpose (simple SQL queries over flat files) while adding powerful multi-file capabilities that don't complicate the interface.