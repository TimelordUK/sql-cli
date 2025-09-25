# SQL-CLI Current Keymap Reference

Complete list of all current keymaps in SQL-CLI, organized by function.

## Template Operations - `\st` prefix
**New template system - all under `\st` to avoid conflicts**

| Keymap | Action | Description |
|--------|--------|-------------|
| `\ste` | Expand | Expand template/macro at cursor |
| `\sts` | Select | Select template from file |
| `\stl` | List | List all available templates |
| `\str` | Reload | Reload template cache |
| `\stm` | Macro | Quick macro expand |

## Query Execution - Mixed prefixes

| Keymap | Action | Config Key | Description |
|--------|--------|-----------|-------------|
| `\sq` | Execute | `execute` | Execute entire query |
| `\ss` | Execute Selection | `execute_selection` | Execute visual selection |
| `\sx` | Execute at Cursor | `execute_at_cursor` | Execute query at cursor position |
| `\sv` | Select Query | `select_query` | Visually select query at cursor |
| `\sp` | Show Plan | `show_plan` | Show query execution plan |

## Formatting & Display

| Keymap | Action | Config Key | Description |
|--------|--------|-----------|-------------|
| `\sf` | Format | `format_query` | Format query at cursor |
| `\so` | Orientation | `toggle_orientation` | Toggle split orientation |
| `\sh` | Schema | `show_schema` | Show table schema |
| `\sk` | Column Help | `column_help` | Smart column/function detection |

## Export Operations

| Keymap | Action | Config Key | Description |
|--------|--------|-----------|-------------|
| `\se` | Export Menu | `export_menu` | Show export format menu |
| `\sm` | Markdown | `export_markdown` | Export as Markdown |
| `\sT` | TSV | `export_tsv` | Export as Tab-separated (capital T) |

## CTE Operations

| Keymap | Action | Config Key | Description |
|--------|--------|-----------|-------------|
| `\sC` | Test CTE | `test_cte` | Test CTE at cursor |
| `\sN` | New Buffer | `test_cte_new` | Test CTE in new buffer |
| `\sA` | Analysis | `cte_analysis` | Show CTE analysis popup |

## Visualization Charts

| Keymap | Action | Config Key | Description |
|--------|--------|-----------|-------------|
| `\sB` | Bar Chart | `bar_chart_at_cursor` | Bar chart from query |
| `\sH` | Histogram | `histogram_at_cursor` | Histogram from query |
| `\sl` | Sparkline | `sparkline_at_cursor` | Sparkline from query |

## Navigation & Utility

| Keymap | Action | Config Key | Description |
|--------|--------|-----------|-------------|
| `\sn` | Navigate | `toggle_table_nav` | Toggle table navigation mode |
| `\sy` | Yank/Copy | `copy_query` | Copy query to clipboard |

## Potential Conflicts & Solutions

### Resolved Conflicts:
- `\st` was TSV export → Changed to `\sT` (capital T)
- Templates now safely use `\st` prefix

### Current Namespace Usage:
- `\st_` - Templates (5 mappings)
- `\s[qsxvp]` - Query operations (5 mappings)
- `\s[fohk]` - Formatting/Help (4 mappings)
- `\s[emT]` - Export (3 mappings)
- `\s[CNA]` - CTE operations (3 mappings)
- `\s[BHl]` - Charts (3 mappings)
- `\s[ny]` - Navigation/Utility (2 mappings)

### Available Single Letters:
Lower case available: `a, b, c, d, g, i, j, r, u, w, z`
Upper case available: `D, E, F, G, I, J, K, L, M, O, P, Q, R, S, U, V, W, X, Y, Z`

## Recommended Future Prefixes

To maintain consistency and avoid conflicts:

| Prefix | Reserved For | Status |
|--------|-------------|---------|
| `\st` | Templates | ✅ In use |
| `\sr` | Refactoring | 📋 Planned |
| `\sc` | CTEs (lowercase) | 🔄 Partially used |
| `\sw` | WHERE builder | 📋 Planned |
| `\sd` | Debugging | 📋 Available |
| `\sg` | Git integration | 📋 Available |
| `\si` | Import operations | 📋 Available |
| `\su` | Utilities | 📋 Available |

## Configuration

Users can customize all keymaps in their config:

```lua
require('sql-cli').setup({
  keymaps = {
    execute = "<leader>sq",           -- Change any mapping
    export_tsv = "<leader>sT",        -- Now capital T
    -- Set to false to disable
    toggle_table_nav = false,         -- Disable specific mapping
  }
})

-- Template system has its own setup
require('sql-cli.template').setup({
  keymap_prefix = "<leader>st"       -- Change template prefix
})
```

## Quick Reference by Frequency

Most commonly used operations:

| Operation | Current Key | Remember As |
|-----------|------------|-------------|
| Execute query | `\sq` | **Q**uery |
| Execute selection | `\ss` | **S**election |
| Format SQL | `\sf` | **F**ormat |
| Export menu | `\se` | **E**xport |
| Expand template | `\ste` | **T**emplate **E**xpand |
| Show schema | `\sh` | Sc**h**ema |
| Copy query | `\sy` | **Y**ank |