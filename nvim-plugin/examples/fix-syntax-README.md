# FIX Message Syntax Highlighting Examples

This directory contains examples for adding custom syntax highlighting for FIX protocol messages in sql-cli.nvim output.

## Approaches

### 1. Lua Config (Recommended for Simple Patterns)

Use the `syntax.patterns` option in your plugin setup:

```lua
-- In your lazy.nvim config
require('sql-cli').setup({
  syntax = {
    patterns = {
      { pattern = [[\<ExecutionReport\>]], group = "FixExecutionReport",
        color = { gui = "#50fa7b", cterm = "Green", bold = true } },
      { pattern = [[\<Buy\>]], group = "FixBuy",
        color = { gui = "#50fa7b", cterm = "Green", bold = true } },
      { pattern = [[\<Sell\>]], group = "FixSell",
        color = { gui = "#ff5555", cterm = "Red", bold = true } },
    }
  },
})
```

**See:** `fix-trading-syntax.lua` for complete example

### 2. VimScript Syntax File (For Complex Patterns)

For more advanced syntax highlighting with keywords, regions, and complex matching:

**Option A: Auto-load on all sql-cli buffers**
Place the file at:
```
~/.config/nvim/after/syntax/sql-cli-output.vim
```

**Option B: Manual sourcing**
```vim
:source ~/path/to/fix-custom-syntax.vim
```

**Option C: Auto-source in your init.lua**
```lua
vim.api.nvim_create_autocmd("FileType", {
  pattern = "sql-cli-output",
  callback = function()
    vim.cmd("source ~/.config/nvim/syntax/fix-custom-syntax.vim")
  end,
})
```

**See:** `fix-custom-syntax.vim` for complete example

## What Gets Highlighted

### Message Types
- **ExecutionReport** (Green) - Execution confirmations
- **AllocationReport** (Cyan) - Allocation instructions
- **NewOrderSingle** (Yellow) - New order submissions
- **Reject** (Red) - Rejections

### Order Status
- **PendingNew** (Yellow) - Pending states
- **New**, **PartiallyFilled** (Green) - Active orders
- **Filled** (Bold Green) - Completed
- **Canceled**, **Rejected** (Red/Gray) - Inactive

### Side
- **Buy** (Bold Green)
- **Sell** (Bold Red)

### Exchanges
- **NYSE**, **NASDAQ** (Magenta)
- **LSE**, **XETRA** (Pink)

### Instrument Types
- **NDS**, **NFD**, **CDS**, **IRS** (Various colors)

## Color Schemes

The examples use Dracula-inspired colors:
- Green (`#50fa7b`) - Positive actions (Buy, New, Filled)
- Red (`#ff5555`) - Negative actions (Sell, Reject, Cancel)
- Yellow (`#f1fa8c`) - Pending/Warning states
- Cyan (`#8be9fd`) - Informational
- Magenta (`#bd93f9`) - Exchanges, metadata

## Customizing for Your Data

### Pattern Syntax (Lua)

```lua
{
  pattern = [[\<YourValue\>]],  -- \< \> = word boundaries
  group = "UniqueGroupName",     -- Must be unique
  color = {
    gui = "#hexcolor",           -- For GUI Neovim
    cterm = "ColorName",         -- For terminal (Red, Green, etc.)
    bold = true,                 -- Optional: bold text
    italic = true,               -- Optional: italic text
  }
}
```

### Common Pattern Examples

```lua
-- Exact word match
{ pattern = [[\<EXACT\>]], ... }

-- Case-insensitive
{ pattern = [[\c\<value\>]], ... }

-- Multiple values (regex OR)
{ pattern = [[\<\(Buy\|Sell\)\>]], ... }

-- Tag=value format
{ pattern = [[35=\w\+]], ... }

-- Number ranges
{ pattern = [[\<[0-9]\{1,3\}\>]], ... }
```

## Limitations

### What Won't Work

1. **Generic values** - "8" or "AS" are too common and would highlight everywhere
   - ✅ Solution: Only highlight resolved enum values (ExecutionReport, not "8")

2. **Context-dependent highlighting** - Can't color "8" only when it's in MsgType column
   - ✅ Solution: Use your SQL queries to resolve enums to readable names

3. **Column-specific colors** - Can't highlight "PendingNew" only in Status column
   - ✅ Solution: Use unique naming (prepend column name if needed)

## Best Practices

### 1. Resolve Enums in SQL

Instead of showing raw FIX tag values, resolve them in your query:

```sql
SELECT
  CASE msg_type
    WHEN '8' THEN 'ExecutionReport'
    WHEN 'D' THEN 'NewOrderSingle'
    ELSE msg_type
  END as msg_type,
  CASE side
    WHEN '1' THEN 'Buy'
    WHEN '2' THEN 'Sell'
    ELSE side
  END as side
FROM fix_messages
```

This makes the output much more readable and colorable.

### 2. Use Prefixes for Ambiguous Values

If "New" appears in multiple contexts:

```sql
SELECT
  CONCAT('Status:', order_status) as status,
  CONCAT('Type:', msg_type) as type
FROM orders
```

Then highlight with:
```lua
{ pattern = [[Status:New]], group = "StatusNew", ... }
{ pattern = [[Type:New]], group = "TypeNew", ... }
```

### 3. Test Patterns Incrementally

Start with a few patterns, verify they work, then add more:

```bash
# Test your query
./target/release/sql-cli your_data.db -q "SELECT * FROM fix_messages LIMIT 10"

# Then add syntax highlighting incrementally
```

## Example Queries for FIX Data

```sql
-- Execution reports with resolved enums
SELECT
  CASE msg_type WHEN '8' THEN 'ExecutionReport' ELSE msg_type END as msg_type,
  symbol,
  CASE side WHEN '1' THEN 'Buy' WHEN '2' THEN 'Sell' END as side,
  CASE ord_status
    WHEN '0' THEN 'New'
    WHEN '1' THEN 'PartiallyFilled'
    WHEN '2' THEN 'Filled'
    WHEN '4' THEN 'Canceled'
    WHEN '8' THEN 'Rejected'
  END as status,
  last_qty,
  last_px
FROM fix_messages
WHERE msg_type = '8'
ORDER BY transact_time DESC
LIMIT 50;
```

## Testing Your Syntax

1. **Visual check**: Run a query and verify colors appear
2. **Inspect groups**: In Neovim, place cursor on highlighted text and run:
   ```vim
   :Inspect
   ```
   This shows which highlight group is active

3. **Reload syntax**: After changes:
   ```vim
   :set syntax=sql-cli-output
   ```

## Complete Working Example

See `fix-trading-syntax.lua` for a complete, copy-paste ready configuration that works with resolved FIX enum values.

The key is: **resolve numeric FIX tags to readable names in your SQL queries**, then highlight those readable names with custom colors.
