# Smart Column Expansion and Completion Features

## New Features

### 1. Smart * Expansion (`\sE`)

The `\sE` keybinding now intelligently expands `SELECT *` by **executing the query** to get the actual column schema, instead of relying on static data file hints.

#### What's New

**Before:**
- Only worked with `-- #! data_file.csv` hints
- Couldn't handle CTEs, subqueries, or joins
- Limited to direct table access

**After:**
- Works with CTEs, subqueries, joins, and complex queries
- Executes `LIMIT 0` version of your query to get schema
- Falls back to static schema if execution fails
- Can be toggled on/off in config

#### Usage

1. Write a query with `SELECT *`:
   ```sql
   WITH sales AS (
     SELECT * FROM data WHERE amount > 100
   )
   SELECT * FROM sales
   ```

2. Put cursor on the `SELECT *` line you want to expand

3. Press `\sE` (or `:SqlCliExpandStar`)

4. The plugin:
   - Captures the full query context (including the CTE)
   - Executes it with `LIMIT 0` to get column names
   - Replaces `*` with actual columns
   - Adds a column hint comment for completion (see below)

#### Result

```sql
-- Columns: order_id, customer_id, amount, date, region

WITH sales AS (
  SELECT * FROM data WHERE amount > 100
)
SELECT
    order_id
  , customer_id
  , amount
  , date
  , region
FROM sales
```

### 2. Auto Column Hint Comments

After expanding `*`, a special comment is automatically inserted at the top of your buffer:

```sql
-- Columns: order_id, customer_id, amount, date, region
```

#### Why This Matters

Neovim's default word completion (`Ctrl+N`) looks for words in the **current buffer**. Since query results are in a different buffer, column names weren't available for completion.

The hint comment solves this by adding column names to your SQL buffer!

#### Usage

1. After expanding *, the hint comment is added automatically

2. Start typing a column name and press `Ctrl+N`:
   ```sql
   SELECT ord<Ctrl+N>
   ```

3. Neovim suggests: `order_id` (from the hint comment!)

#### Configuration

```lua
require('sql-cli').setup({
  smart_expansion = {
    enabled = true,                    -- Enable smart expansion
    auto_insert_column_hints = true,   -- Auto-add hint comments
    auto_sync_column_hints = true,     -- Sync after query execution
    fallback_to_static = true,         -- Fall back if query fails
  }
})
```

### 3. Column Hint Syncing

When you execute a query and get results, the plugin can automatically update the column hint comment with the result columns.

This keeps your completion suggestions in sync with your actual query results!

#### Configuration

Set `auto_sync_column_hints = true` in config (enabled by default).

### 4. Distinct Values Display (`\srD`)

This feature already existed but may not have been easily discoverable.

#### Usage

1. Put cursor on a column name in your query
2. Press `\srD`
3. A floating window shows distinct values and their counts

#### Example

Cursor on `region`:

```
╔═══ Distinct values for 'region' ═══╗
║ Total distinct values: 4            ║
║                                     ║
║ Value         Count                 ║
║ ─────────────────────────            ║
║ East          1,234                 ║
║ West          1,156                 ║
║ North         892                   ║
║ South         743                   ║
╚═════════════════════════════════════╝
```

## Configuration

### Full Config Example

```lua
require('sql-cli').setup({
  -- Smart expansion features
  smart_expansion = {
    enabled = true,                    -- Use smart expansion (query execution)
    auto_insert_column_hints = true,   -- Add column hints for completion
    auto_sync_column_hints = true,     -- Sync hints after query execution
    fallback_to_static = true,         -- Fall back to --schema-json if query fails
  },

  -- Keymaps
  keymaps = {
    expand_star = '<leader>sE',  -- Smart * expansion
    -- ... other keymaps
  }
})
```

### Disable Smart Expansion

To use the old static expansion:

```lua
smart_expansion = {
  enabled = false,  -- Use old expand_star_columns behavior
}
```

### Disable Column Hints

To expand * without adding hint comments:

```lua
smart_expansion = {
  enabled = true,
  auto_insert_column_hints = false,  -- Don't add hints
}
```

## Keybindings Summary

| Key       | Description                          |
|-----------|--------------------------------------|
| `\sE`     | Smart SELECT * expansion             |
| `\srD`    | Show distinct values for column      |
| `Ctrl+N`  | Autocomplete (use with column hints) |

## Technical Details

### How Smart Expansion Works

1. **Query Capture**: Finds the complete query context (including CTEs, subqueries)
2. **Preview Execution**: Runs query with `LIMIT 0` to get schema
3. **Column Extraction**: Parses JSON result to get column names
4. **Replacement**: Replaces `*` with formatted column list
5. **Hint Addition**: Adds comment for completion (if enabled)

### Query Context Detection

The plugin looks backward/forward to find query boundaries:
- Stops at `GO` statements
- Captures `WITH` clauses at the start
- Includes everything up to the next `GO`

This means it correctly handles:
- Multiple CTEs
- Nested subqueries
- JOIN operations
- Complex WHERE clauses

### Fallback Behavior

If smart expansion fails (query error, syntax issue):
1. Shows error message
2. Falls back to static `--schema-json` if `fallback_to_static = true`
3. Uses data file hint or opened CSV as before

## Testing

See `test_smart_expansion.sql` for examples:

```bash
# Open in neovim
nvim test_smart_expansion.sql

# Try \sE on each SELECT * line
# Try \srD on column names
# Try Ctrl+N after expansion to test completion
```

## Troubleshooting

### "Could not determine query context"

- Make sure your cursor is on or near a `SELECT *` line
- Check that the query is properly formatted (not in middle of CTE)

### "Query execution failed"

- Query may have syntax errors
- Data file might not be set (use `:SqlCliSetData`)
- Try the query manually first with `:SqlCliExecute`

### Column hints not appearing

- Check `smart_expansion.auto_insert_column_hints = true`
- Verify expansion actually ran (check for notification)
- Look for `-- Columns:` comment at top of buffer

### Completion not working

- Press `Ctrl+N` after typing partial column name
- Make sure column hint comment exists in buffer
- Neovim's completion looks at **current buffer** only

## Future Enhancements

Possible improvements:
- LSP integration for true cross-buffer completion
- Column hints from results buffer without expansion
- Intelligent column ordering (frequently used first)
- Type hints in completion (String, Integer, etc.)
- Schema caching to avoid repeated executions

## Related Documentation

- Main plugin README: `README.md`
- Keybindings: `KEYBINDINGS.md`
- Design doc: `../docs/NVIM_SMART_COLUMN_COMPLETION.md`
