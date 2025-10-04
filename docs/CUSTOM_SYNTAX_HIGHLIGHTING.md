# Custom Syntax Highlighting

The SQL-CLI Neovim plugin supports custom syntax highlighting patterns for query results. This allows you to highlight specific values, keywords, or patterns in your data with custom colors.

## Overview

Custom syntax highlighting is applied to the output buffer after query execution. You can define patterns either:
1. **Inline** - In your Neovim plugin configuration
2. **External file** - In a separate Lua file that returns a pattern table

Patterns are applied using Vim's syntax highlighting engine, which provides visual overlays without modifying buffer content.

## Configuration

### Basic Setup (Inline Patterns)

Add patterns directly to your `sql-cli.lua` plugin config:

```lua
require('sql-cli').setup({
  -- ... other config ...

  syntax = {
    patterns = {
      {
        pattern = [[\<Buy\>]],           -- Vim regex pattern
        group = "Buy",                    -- Highlight group name
        color = {
          gui = "#50fa7b",                -- GUI hex color
          cterm = "Green",                -- Terminal color name
          bold = true                     -- Optional: make bold
        }
      },
      {
        pattern = [[\<Sell\>]],
        group = "Sell",
        color = {
          gui = "#ff5555",
          cterm = "Red",
          bold = true
        }
      },
    }
  }
})
```

### External File Setup

Create a separate syntax file and reference it in your config:

**~/.config/sql-cli/my_syntax.lua:**
```lua
return {
  { pattern = [[\<Buy\>]], group = "Buy", color = { gui = "#50fa7b", cterm = "Green", bold = true } },
  { pattern = [[\<Sell\>]], group = "Sell", color = { gui = "#ff5555", cterm = "Red", bold = true } },
  -- ... more patterns ...
}
```

**Plugin config:**
```lua
require('sql-cli').setup({
  syntax = {
    custom_file = "~/.config/sql-cli/my_syntax.lua"
  }
})
```

### Combined Approach

You can use both inline patterns and an external file. They will be merged:

```lua
require('sql-cli').setup({
  syntax = {
    custom_file = "~/.config/sql-cli/common_patterns.lua",
    patterns = {
      -- Additional patterns specific to this project
      { pattern = [[\<URGENT\>]], group = "Urgent", color = { gui = "#ff0000", cterm = "Red", bold = true } },
    }
  }
})
```

## Pattern Specification

Each pattern is a Lua table with these fields:

| Field | Required | Description |
|-------|----------|-------------|
| `pattern` | Yes | Vim regex pattern (use `\<` and `\>` for word boundaries) |
| `group` | Yes | Unique name for the highlight group (will be prefixed with "SqlCli") |
| `color` | Yes | Color specification table |
| `color.gui` | No | Hex color for GUI Neovim (e.g., `"#50fa7b"`) |
| `color.cterm` | No | Terminal color name (e.g., `"Green"`, `"Red"`, `"Cyan"`) |
| `color.bold` | No | Boolean - make text bold |
| `color.italic` | No | Boolean - make text italic |
| `color.underline` | No | Boolean - underline text |

## Pattern Examples

### Word Boundaries
Use `\<` and `\>` to match whole words only:

```lua
-- Matches "Buy" but not "Buyer" or "BuyMore"
{ pattern = [[\<Buy\>]], group = "Buy", color = { gui = "#50fa7b" } }
```

### Case-Insensitive Matching
Use `\c` in the pattern:

```lua
-- Matches "error", "ERROR", "Error"
{ pattern = [[\c\<error\>]], group = "Error", color = { gui = "#ff5555" } }
```

### Multiple Alternatives
Use `\|` for OR:

```lua
-- Matches "USD", "EUR", or "GBP"
{ pattern = [[\<\(USD\|EUR\|GBP\)\>]], group = "Currency", color = { gui = "#f1fa8c" } }
```

### Number Patterns
Match specific number formats:

```lua
-- Match negative numbers
{ pattern = [[\v<-\d+>]], group = "Negative", color = { gui = "#ff5555" } }

-- Match percentages
{ pattern = [[\v<\d+(\.\d+)?%>]], group = "Percent", color = { gui = "#8be9fd" } }
```

### Status Indicators
Common status patterns:

```lua
{
  pattern = [[\<\(SUCCESS\|PASSED\|OK\)\>]],
  group = "Success",
  color = { gui = "#50fa7b", cterm = "Green", bold = true }
},
{
  pattern = [[\<\(FAILED\|ERROR\|CRITICAL\)\>]],
  group = "Error",
  color = { gui = "#ff5555", cterm = "Red", bold = true }
},
{
  pattern = [[\<\(PENDING\|WAITING\|IN_PROGRESS\)\>]],
  group = "Warning",
  color = { gui = "#f1fa8c", cterm = "Yellow" }
}
```

## Complete Example: Trading Data

See `examples/syntax_configs/trading_syntax.lua` for a complete example highlighting trading-specific data:

- **Buy/Sell** - Green/Red with bold
- **Instrument types** (NDS, NFD, CDS, IRS) - Different colors per type

## Color Reference

### Common Terminal Colors (cterm)
- `"Black"`, `"Red"`, `"Green"`, `"Yellow"`
- `"Blue"`, `"Magenta"`, `"Cyan"`, `"White"`
- Use color numbers: `"0"` to `"255"`

### Popular GUI Colors (gui)
- **Green**: `"#50fa7b"` (Dracula green)
- **Red**: `"#ff5555"` (Dracula red)
- **Yellow**: `"#f1fa8c"` (Dracula yellow)
- **Cyan**: `"#8be9fd"` (Dracula cyan)
- **Purple**: `"#bd93f9"` (Dracula purple)
- **Pink**: `"#ff79c6"` (Dracula pink)

## Troubleshooting

### Pattern Not Matching

1. **Check word boundaries**: Add `\<` and `\>` to match whole words
   ```lua
   -- Won't match "ORD012234" numbers:
   { pattern = [[\<\d+\>]], ... }
   ```

2. **Test pattern in Vim**:
   ```vim
   :syntax match TestPattern /\<Buy\>/
   ```

### Colors Not Showing

1. Ensure the pattern is being loaded:
   ```lua
   debug = true  -- Enable debug mode in config
   ```

2. Check if another syntax rule is overriding:
   - More specific patterns should come after general ones
   - String patterns have lowest priority by default

### Pattern Priority

Patterns are applied in order. Later patterns can override earlier ones if they match the same text. Order your patterns from most general to most specific.

## Best Practices

1. **Use word boundaries** to avoid partial matches in alphanumeric IDs
2. **Group related patterns** in external files for reusability
3. **Test patterns** on sample data before deploying
4. **Use descriptive group names** for maintainability
5. **Prefer GUI colors** for better appearance in modern terminals

## Advanced: Vim Regex Reference

- `\<` - Start of word boundary
- `\>` - End of word boundary
- `\|` - OR (alternative)
- `\( \)` - Grouping
- `\d` - Digit (0-9)
- `\w` - Word character (alphanumeric + underscore)
- `\s` - Whitespace
- `\v` - Very magic mode (less escaping needed)
- `\c` - Case insensitive

See `:help pattern` in Neovim for complete reference.
