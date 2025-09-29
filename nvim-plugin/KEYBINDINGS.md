# SQL-CLI Neovim Plugin Keybindings

## Organization

All keybindings use the `<leader>s` prefix, organized into logical groups:

## Core SQL Execution
| Keybinding | Description |
|------------|-------------|
| `\sq` | Execute entire query buffer |
| `\ss` | Execute visual selection |
| `\sx` | Execute query at cursor position |
| `\sX` | Execute query at cursor with execution plan |

## Navigation & Selection
| Keybinding | Description |
|------------|-------------|
| `\sv` | Visually select query at cursor |
| `\sn` | Toggle table navigation mode |
| `]q` | Jump to next query |
| `[q` | Jump to previous query |
| `]t` | Jump to next result table |
| `[t` | Jump to previous result table |
| `\s1` | Jump to first table |
| `\s2` | Jump to second table |
| `\s3` | Jump to third table |

## CTE Testing & Analysis
| Keybinding | Description |
|------------|-------------|
| `\sC` | Test CTE at cursor (preview and run) |
| `\sN` | Test CTE in new buffer |
| `\sA` | Show CTE analysis popup |

## Query Management
| Keybinding | Description |
|------------|-------------|
| `\sf` | Format query at cursor |
| `\sp` | Show query plan |
| `\sP` | Preview query in floating window |
| `\s/` | Toggle comment for query at cursor |
| `\sy` | Copy query at cursor to clipboard |
| `\sY` | Copy query for shell execution |

## Data Files
| Keybinding | Description |
|------------|-------------|
| `\sD` | Set data file (capital D) |
| `\sV` | View data file |

## Functions & Help
| Keybinding | Description |
|------------|-------------|
| `K` | Show function help at cursor |
| `\sL` | List all SQL functions |
| `\sG` | List all generator functions |
| `\sF` | Search SQL functions |
| `\sh` | Show table schema |
| `\sk` | Smart column/function detection |
| `\sE` | Expand SELECT * to column names |
| `\sI` | Show current table info |

## Export & Output
| Keybinding | Description |
|------------|-------------|
| `\se` | Export menu (all formats) |
| `\sm` | Export as Markdown |
| `\st` | Export as Tab-separated (Excel) |
| `\sW` | Write results to CSV file |
| `\sO` | Toggle output window |
| `\so` | Toggle split orientation |

## Visualizations
| Keybinding | Description |
|------------|-------------|
| `\sB` | Bar chart from query at cursor |
| `\sH` | Histogram from query at cursor |
| `\sl` | Sparkline from query at cursor |

## Templates (`\sT` prefix)

SQL query templates with variable substitution:

| Keybinding | Description |
|------------|-------------|
| `\sT` | Select and apply a template |
| `\sTq` | Quick substitute variables in current query |
| `\sTs` | Save current query as a template |
| `\sTd` | Insert a quick date picker |
| `\sTS` | Insert a trade source selector |

### Template Features
- **Variable Substitution**: Use `${VARIABLE}` or `${VARIABLE:default}` in queries
- **Auto-escaping**: Automatically handles quote escaping for JSON bodies
- **Date Helpers**: Quick date selection (today, yesterday, last business day)
- **Source Selection**: Quick picker for common trade sources
- **Environment Variables**: Automatically substitutes env vars like `${JWT_TOKEN}`

### Example Template Usage
```sql
WITH WEB trades AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    BODY '{
        "Where": "Source = \"${SOURCE}\" and TradeDate = ${TRADE_DATE}",
        "Limit": ${LIMIT:100}
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT * FROM trades
```

When you run `\sTq` on this query:
1. Prompts for SOURCE (with quick selector)
2. Prompts for TRADE_DATE (with date picker)
3. Uses default of 100 for LIMIT
4. Substitutes all variables and handles escaping

## Refactoring (`\sr` prefix)

All refactoring operations are grouped under `\sr`:

### Case Statement Generation
| Keybinding | Description |
|------------|-------------|
| `\src` | Generate CASE from data analysis (full) |
| `\srd` | Generate CASE from data (quick/inline) |
| `\srC` | Generate CASE for column under cursor |
| `\srD` | Show distinct values for column |
| `\srr` | Generate CASE for range (inline) |
| `\srb` | Generate banding CASE statement |

### SQL Patterns & Transformations
| Keybinding | Description |
|------------|-------------|
| `\sre` | Extract selection to CTE (visual mode) |
| `\srp` | Pivot/conditional aggregation |
| `\sri` | Insert SQL pattern menu |

### Window Functions (`\srw` prefix)
| Keybinding | Description |
|------------|-------------|
| `\srw` | Window function wizard |
| `\srwr` | Add ROW_NUMBER() ranking |
| `\srwl` | Add LAG/LEAD delta |
| `\srwa` | Add windowed aggregate |
| `\srwk` | Add RANK functions |
| `\srwv` | Add FIRST/LAST VALUE |

## Result Filtering

When viewing SQL results in the output buffer:

| Keybinding | Description |
|------------|-------------|
| `\sz` | Open fuzzy filter (zoom/search through results) |
| `/` | Open fuzzy filter (alternative - natural search key) |
| `\sZ` | Reset filter (zoom out - show all results again) |

### In Filter Window:
| Keybinding | Description |
|------------|-------------|
| `<Esc>` | Close filter and restore original results |
| `<Enter>` | Apply filter and close window |
| `<C-l>` | Clear filter input |
| `<C-1>` to `<C-9>` | Filter specific column only |

## Table Navigation Mode

When in table navigation mode (activated with `\sn`):

### Movement (when hijack_hjkl = true)
| Keybinding | Description |
|------------|-------------|
| `h` | Move left one column |
| `j` | Move down one row |
| `k` | Move up one row |
| `l` | Move right one column |
| `0` | Go to first column |
| `$` | Go to last column |
| `gg` | Go to first row |
| `G` | Go to last row |
| `Tab` | Next cell (wraps to next row) |
| `Shift-Tab` | Previous cell |

### Movement (when hijack_hjkl = false)
Use arrow keys instead of hjkl for navigation.

### Operations
| Keybinding | Description |
|------------|-------------|
| `yy` | Yank current cell value |
| `Y` | Yank entire row as CSV |
| `yc` | Yank entire column |

### Export (in table navigation mode)
| Keybinding | Description |
|------------|-------------|
| `\se` | Show export menu |
| `\sb` | Open as HTML in browser |
| `\sH` | Yank as HTML code |
| `\sm` | Yank as Markdown table |
| `\st` | Yank as TSV (for Excel) |
| `\si` | Generate INSERT statements |
| `\sC` | Generate CREATE TABLE |

## Configuration

To customize keybindings, override them in your config:

```lua
require('sql-cli').setup({
  keymaps = {
    execute = "<your-key>",
    -- Set to false or nil to disable
    preview_query = false,
  }
})
```

## Tips

1. **Refactoring Operations**: All refactoring features are now under `\sr` to keep them organized
2. **CTE Testing**: `\sC` provides a preview before execution - press Enter to run
3. **Table Navigation**: Great for exploring large result sets - headers stay visible when scrolling
4. **Export Menu**: `\se` in table mode gives you all export options in one place
5. **Window Functions**: Use `\srw` for the wizard to build complex window functions interactively