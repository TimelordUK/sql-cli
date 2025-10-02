# Neovim Plugin: Smart Column Completion and Intelligent * Expansion

## Problem Statement

Currently the nvim plugin has several limitations around column completion and * expansion:

### Current Issues

1. **No column completion across buffers**: When query results are displayed in a separate buffer with all columns visible, nvim's default word completion doesn't include those columns in the SQL editor buffer.

2. **Limited * expansion (`\sE`)**: The current `expand_star_columns()` only works with:
   - Data file hint (`-- #! path/to/file.csv`)
   - Directly opened CSV files
   - It does NOT handle CTEs or dynamic queries

3. **Lost \srD keybinding**: The distinct values/cardinality feature (`show_distinct_values()`) exists but may have been overridden or is not easily accessible.

4. **Manual effort required**: Users must manually copy column names from results buffer or remember them from previous queries.

## Proposed Solutions

### 1. Intelligent * Expansion with Query Execution

Instead of relying on static data file hints, execute the query to get actual columns:

**File:** `nvim-plugin/lua/sql-cli/results.lua`

```lua
-- New function: Smart * expansion that runs the query
function M.expand_star_smart(config, state)
  -- Get current query (without the SELECT * itself)
  local query = M.get_query_for_expansion()

  if not query then
    vim.notify("Could not determine query context", vim.log.levels.WARN)
    return
  end

  -- Execute a preview query to get column names
  -- This handles CTEs, joins, subqueries, etc.
  local preview_query = string.gsub(query, "SELECT%s+%*", "SELECT * LIMIT 0")

  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    vim.notify(err, vim.log.levels.ERROR)
    return
  end

  -- Execute with --schema-json to get columns from the query
  Job:new({
    command = command_path,
    args = {
      '-q', preview_query,
      '--schema-json'
    },
    on_exit = function(j, return_val)
      vim.schedule(function()
        if return_val == 0 then
          local result = table.concat(j:result(), '\n')
          local ok, schema = pcall(vim.json.decode, result)

          if ok and schema and schema.columns then
            -- Extract column names
            local columns = {}
            for _, col in ipairs(schema.columns) do
              table.insert(columns, col.name)
            end

            -- Perform the replacement
            M.replace_star_with_columns(columns)

            -- BONUS: Add column hint comment for completion
            M.add_column_hint_comment(columns, state)
          else
            vim.notify("Could not parse schema", vim.log.levels.ERROR)
          end
        else
          vim.notify("Query execution failed: " .. table.concat(j:stderr_result(), '\n'),
                     vim.log.levels.ERROR)
        end
      end)
    end
  }):start()
end

-- Helper: Get query context for expansion
function M.get_query_for_expansion()
  local cursor = vim.api.nvim_win_get_cursor(0)
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)

  -- Find query boundaries (GO statements or CTEs)
  local start_line = 1
  local end_line = #lines

  -- Look backward for WITH or previous GO
  for i = cursor[1] - 1, 1, -1 do
    if lines[i]:upper():match('^GO%s*$') then
      start_line = i + 1
      break
    elseif lines[i]:upper():match('^WITH%s+') then
      start_line = i
      break
    end
  end

  -- Look forward for GO
  for i = cursor[1], #lines do
    if lines[i]:upper():match('^GO%s*$') then
      end_line = i - 1
      break
    end
  end

  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end

  return table.concat(query_lines, '\n')
end

-- Helper: Replace * with column list
function M.replace_star_with_columns(columns)
  local line = vim.api.nvim_get_current_line()
  local cursor_pos = vim.api.nvim_win_get_cursor(0)

  -- Find SELECT *
  local select_pattern = "SELECT%s+%*"
  local select_start, select_end = line:find(select_pattern)

  if not select_start then
    local line_lower = line:lower()
    select_start, select_end = line_lower:find("select%s+%*")
  end

  if not select_start then
    vim.notify("No SELECT * found on current line", vim.log.levels.WARN)
    return
  end

  -- Quote columns if needed
  local column_names = {}
  for _, col in ipairs(columns) do
    if col:match("[%-%. ]") then
      table.insert(column_names, '"' .. col .. '"')
    else
      table.insert(column_names, col)
    end
  end

  -- Build expanded SELECT
  local after_star = line:sub(select_end + 1)
  local total_length = #("SELECT " .. table.concat(column_names, ", ")) + #after_star
  local use_multiline = #column_names > 5 or total_length > 100

  if use_multiline then
    local lines_to_insert = {"SELECT"}
    for i, col in ipairs(column_names) do
      local prefix = i == 1 and "    " or "  , "
      table.insert(lines_to_insert, prefix .. col)
    end

    -- Add FROM clause if present
    if after_star:match("^%s+FROM") or after_star:match("^%s+from") then
      table.insert(lines_to_insert, after_star:match("^%s*(.*)"))
    end

    local row = cursor_pos[1]
    vim.api.nvim_buf_set_lines(0, row - 1, row, false, lines_to_insert)

    vim.notify("Expanded * to " .. #column_names .. " columns (from query execution)",
               vim.log.levels.INFO)
  else
    local expanded = "SELECT " .. table.concat(column_names, ", ") .. after_star
    vim.api.nvim_set_current_line(expanded)

    vim.notify("Expanded * to " .. #column_names .. " columns", vim.log.levels.INFO)
  end
end
```

### 2. Auto-Insert Column Hint Comments for Completion

Add column names as a comment in the buffer to enable nvim's default word completion:

```lua
-- Add column hint comment for nvim completion
function M.add_column_hint_comment(columns, state)
  -- Check if user wants this feature
  local config = M.config or require('sql-cli.config').defaults
  if not config.auto_insert_column_hints then
    return  -- Feature disabled
  end

  -- Store columns in state for future queries
  state:set_last_query_columns(columns)

  -- Create hint comment
  local hint = "-- Columns: " .. table.concat(columns, ", ")

  -- Insert at top of buffer (after data file hint if present)
  local lines = vim.api.nvim_buf_get_lines(0, 0, 5, false)
  local insert_line = 0

  -- Skip past #! hint if present
  for i, line in ipairs(lines) do
    if line:match("^%s*$") or line:match("^%-%-%s*#!") then
      insert_line = i
    else
      break
    end
  end

  -- Check if hint already exists
  local existing_hint = false
  for i = 0, math.min(insert_line + 2, #lines) do
    if lines[i] and lines[i]:match("^%-%- Columns:") then
      existing_hint = true
      -- Update existing hint
      vim.api.nvim_buf_set_lines(0, i - 1, i, false, {hint})
      break
    end
  end

  if not existing_hint then
    vim.api.nvim_buf_set_lines(0, insert_line, insert_line, false, {hint, ""})
  end

  vim.notify("Column hint added for completion (press Ctrl+N to autocomplete)",
             vim.log.levels.INFO)
end
```

### 3. Copy Columns from Results Buffer

When results are displayed, automatically update the column hint in the query buffer:

```lua
-- Called after executing a query and displaying results
function M.sync_columns_to_query_buffer(query_bufnr, columns)
  local config = M.config or require('sql-cli.config').defaults
  if not config.auto_sync_column_hints then
    return
  end

  -- Update column hint in the query buffer
  vim.api.nvim_buf_call(query_bufnr, function()
    M.add_column_hint_comment(columns, require('sql-cli.state'))
  end)
end
```

### 4. Restore \srD for Distinct Values

Ensure the keybinding is properly set up:

**File:** `nvim-plugin/lua/sql-cli/init.lua`

```lua
-- In setup_keymaps function, ensure we have:
vim.keymap.set('n', '<leader>srD', function()
  require('sql-cli.refactoring').show_distinct_values()
end, {
  desc = 'Show distinct values for column under cursor',
  silent = true
})
```

### 5. Configuration Options

Add new config options:

```lua
-- In config.lua defaults
{
  -- Auto-insert column hint comments for completion
  auto_insert_column_hints = true,

  -- Sync columns from results buffer to query buffer
  auto_sync_column_hints = true,

  -- Smart * expansion: execute query to get columns (vs static schema)
  smart_star_expansion = true,

  -- Keybindings
  keymaps = {
    expand_star = '<leader>sE',           -- Smart * expansion
    show_distinct_values = '<leader>srD', -- Show value distribution
    -- ... other keymaps
  }
}
```

## Implementation Plan

### Phase 1: Smart * Expansion (High Priority)
1. ✅ Implement `expand_star_smart()` that executes query to get columns
2. ✅ Handle CTEs, subqueries, joins correctly
3. ✅ Update `\sE` keybinding to use new function
4. ✅ Add config option to toggle smart vs static expansion

### Phase 2: Column Hint Comments (Medium Priority)
1. ✅ Implement `add_column_hint_comment()`
2. ✅ Add config options for auto-insertion
3. ✅ Update hint when query results change
4. ✅ Handle hint placement intelligently (after #! hint)

### Phase 3: Results Buffer Sync (Medium Priority)
1. ✅ Implement `sync_columns_to_query_buffer()`
2. ✅ Hook into query execution flow
3. ✅ Store last query columns in state
4. ✅ Allow manual sync with keybinding

### Phase 4: Restore \srD and Enhance (Low Priority)
1. ✅ Verify `\srD` keybinding is set
2. ✅ Consider adding `\srD` as alias for distinct values
3. ✅ Add visual indicator when cardinality is shown

## Usage Examples

### Example 1: Expanding CTE Query

```sql
-- User types:
WITH sales AS (
  SELECT * FROM data WHERE amount > 100
)
SELECT * FROM sales

-- User presses \sE on second SELECT * line
-- Plugin executes: WITH sales AS (...) SELECT * FROM sales LIMIT 0
-- Gets columns: [OrderID, CustomerID, Amount, Date]
-- Replaces with:
WITH sales AS (
  SELECT * FROM data WHERE amount > 100
)
SELECT
    OrderID
  , CustomerID
  , Amount
  , Date
FROM sales

-- Auto-adds hint comment at top:
-- Columns: OrderID, CustomerID, Amount, Date
```

### Example 2: Using Column Hints for Completion

```sql
-- Columns: TradeID, Symbol, Quantity, Price, Side, TradeDate

-- User types: SELECT Tra<Ctrl+N>
-- Nvim suggests: TradeID, TradeDate (from the hint comment)

SELECT TradeID, Symbol, Price
FROM trades
WHERE Side = 'Buy'
```

### Example 3: Checking Distinct Values

```sql
-- User has cursor on "Side" column
-- Presses \srD

-- Floating window appears:
╔═══ Distinct values for 'Side' ═══╗
║ Total distinct values: 2          ║
║                                   ║
║ Value              Count          ║
║ ──────────────────────────        ║
║ Buy                1,234          ║
║ Sell               1,156          ║
╚═══════════════════════════════════╝
```

## Benefits

1. **Works with CTEs**: No longer limited to static data files
2. **Cross-buffer completion**: Column names available via hints
3. **Accurate schema**: Gets actual query result schema, not just table schema
4. **Better UX**: Fewer manual steps to get column names
5. **Preserved functionality**: \srD still available for value distribution

## Backward Compatibility

- Keep old `expand_star_columns()` as fallback
- Make smart expansion opt-in via config
- Preserve existing `--schema-json` CLI flag behavior

## CLI Requirements

The smart expansion requires the CLI to support:

```bash
# Get schema from a query (not just a file)
sql-cli -q "SELECT * FROM data LIMIT 0" --schema-json

# Should return:
{
  "columns": [
    {"name": "Col1", "type": "String"},
    {"name": "Col2", "type": "Integer"}
  ]
}
```

This already exists if the CLI can execute queries with `LIMIT 0` and return schema.

## Alternative: Use LSP for Completion

For future enhancement, consider implementing an LSP server that:
- Maintains schema cache per data file
- Provides completion based on query context
- Offers signature help for SQL functions
- Real-time validation of column names

This would be more robust than comment hints but requires more infrastructure.

## Testing Checklist

- [ ] Smart * expansion with simple SELECT
- [ ] Smart * expansion with CTE
- [ ] Smart * expansion with multiple CTEs
- [ ] Smart * expansion with subquery
- [ ] Smart * expansion with JOIN
- [ ] Column hint insertion at correct location
- [ ] Column hint update when results change
- [ ] \srD shows distinct values
- [ ] Completion works with column hints
- [ ] Config toggles work correctly
- [ ] Backward compatibility with old expansion

## Related Files

- `nvim-plugin/lua/sql-cli/results.lua` - Main expansion logic
- `nvim-plugin/lua/sql-cli/refactoring.lua` - Distinct values feature
- `nvim-plugin/lua/sql-cli/init.lua` - Keybinding setup
- `nvim-plugin/lua/sql-cli/config.lua` - Configuration
- `nvim-plugin/lua/sql-cli/state.lua` - State management
