# SQL CLI Neovim Plugin

A Neovim plugin for the SQL CLI tool that provides seamless integration for executing SQL queries directly from your editor.

## Features

- 🚀 Execute SQL queries/scripts directly from Neovim
- 📊 Split pane output window with results
- 🔍 Auto-detect CSV files and data hints
- ⌨️ Customizable keymaps
- 📝 Visual selection execution
- 🎯 Query plan visualization
- 🔄 Async execution with live output
- 🎨 Syntax highlighting for output (tables, numbers, errors, etc.)

## Installation

### Using [lazy.nvim](https://github.com/folke/lazy.nvim)

**IMPORTANT**: Since the plugin is in a subdirectory of the main repo, you need to use one of these approaches:

**Option 1 - Clone and use locally (Recommended):**
```lua
{
  dir = vim.fn.expand("~/dev/sql-cli/nvim-plugin"),
  name = "sql-cli.nvim",
  lazy = false,
  config = function()
    require('sql-cli').setup({
      command = vim.fn.expand("~/dev/sql-cli/target/release/sql-cli"),
      output_format = "table",
    })
  end,
}
```

**Option 2 - Manual installation:**
```bash
# Clone the repo
git clone https://github.com/TimelordUK/sql-cli.git ~/sql-cli-temp

# Copy plugin files to your Neovim config
cp -r ~/sql-cli-temp/nvim-plugin/* ~/.config/nvim/

# Then in your lazy.nvim config, just require it:
```
```lua
require('sql-cli').setup({
  command = "sql-cli",  -- Or full path to executable
})
```

**Option 3 - Symlink approach:**
```bash
# Create a symlink in your local lazy.nvim directory
ln -s ~/dev/sql-cli/nvim-plugin ~/.local/share/nvim/lazy/sql-cli.nvim
```

Then configure normally:
```lua
{
  "sql-cli.nvim",
  config = function()
    require('sql-cli').setup()
  end,
}
```

### Using [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use {
  '/path/to/sql-cli/nvim-plugin',
  config = function()
    require('sql-cli').setup()
  end
}
```

## Configuration

```lua
require('sql-cli').setup({
  -- Path to sql-cli executable
  command = "sql-cli",
  
  -- Split configuration
  split = {
    direction = "vertical", -- "vertical" or "horizontal"
    size = 0.5,            -- Size as fraction (0.5 = 50%)
  },
  
  -- Default output format
  output_format = "table", -- "table", "csv", "json", "tsv"
  
  -- Auto-detect features
  auto_detect = {
    csv_files = true,      -- Auto-use CSV files as data source
    data_hints = true,     -- Auto-detect -- #!data: hints
  },
  
  -- Keymaps (set to false to disable)
  keymaps = {
    execute = "<leader>sq",         -- Execute entire buffer
    execute_selection = "<leader>ss", -- Execute visual selection
    toggle_output = "<leader>so",   -- Toggle output window
    set_data_file = "<leader>sd",   -- Set data file
    clear_data_file = "<leader>sc", -- Clear data file
    show_plan = "<leader>sp",       -- Show query plan
  },
  
  -- Output window settings
  output = {
    focus_on_run = false,  -- Focus output after execution
    clear_on_run = true,   -- Clear output before each run
    wrap = false,          -- Line wrap in output
    number = false,        -- Show line numbers
  }
})
```

## Usage

### Basic Commands

- `:SqlCliExecute` - Execute current buffer as SQL
- `:SqlCliSetData <file>` - Set data file for queries
- `:SqlCliClearData` - Clear data file setting
- `:SqlCliShowPlan` - Show query execution plan
- `:SqlCliToggleOutput` - Toggle output window

### Default Keymaps

- `<leader>sq` - Execute SQL query (normal mode)
- `<leader>ss` - Execute selected SQL (visual mode)
- `<leader>so` - Toggle output window
- `<leader>sd` - Set data file
- `<leader>sc` - Clear data file
- `<leader>sp` - Show query plan

### Data File Hints

Add hints in your SQL files to specify the data source:

```sql
-- #!data: ../data/sales.csv
SELECT * FROM sales WHERE amount > 1000;
```

The plugin will automatically detect and use these hints.

### Script Execution

Scripts with `GO` separators are automatically detected and executed properly:

```sql
-- #!data: data/test.csv

SELECT * FROM test WHERE id < 10;
GO

SELECT COUNT(*) FROM test;
GO
```

### Working with CSV Files

When you open a CSV file, it's automatically set as the data source. You can then write queries in another buffer and execute them against this CSV.

## Examples

### Quick Analysis Workflow

1. Open a CSV file: `:e data/sales.csv`
2. Open a new SQL buffer: `:vnew query.sql`
3. Write your query:
   ```sql
   SELECT 
     region,
     COUNT(*) as sales_count,
     SUM(amount) as total
   FROM sales
   GROUP BY region
   ```
4. Execute with `<leader>sq`

### Visual Selection Execution

1. Write multiple queries
2. Visually select the one you want
3. Press `<leader>ss` to execute only the selection

### Using Query Plans

1. Write a complex query
2. Press `<leader>sp` to see the execution plan
3. Optimize based on the plan output

## Tips

- The plugin remembers your last query, so toggling the output window will re-run it
- Use `:SqlCliSetData` with tab completion to quickly switch data files
- Add `-- #!data:` hints to your SQL files for reproducible queries
- The output window is a regular buffer, so you can search, copy, etc.
- Syntax highlighting is automatically applied to output for better readability
- SQL files get additional buffer-local keymaps (`<LocalLeader>r` to run, `<LocalLeader>p` for plan)

## Statusline Integration

Add to your statusline to show the current data file:

```lua
-- For lualine
{
  function() return require('sql-cli').statusline() end,
  cond = function() return vim.bo.filetype == 'sql' end,
}

-- For custom statusline
vim.o.statusline = vim.o.statusline .. ' %{luaeval("require(\"sql-cli\").statusline()")}'
```

## Troubleshooting

### Plugin not loading

If you get `attempt to call field 'setup' (a nil value)`, ensure:

1. The plugin is properly installed:
   ```vim
   :Lazy show sql-cli
   ```

2. Check plugin health:
   ```vim
   :checkhealth sql-cli
   ```

3. Verify the Lua module can be loaded:
   ```vim
   :lua print(vim.inspect(require('sql-cli')))
   ```

4. For local development, ensure the path is correct:
   ```lua
   {
     dir = vim.fn.expand("~/dev/sql-cli/nvim-plugin"),
     name = "sql-cli.nvim",
     lazy = false,  -- Load immediately
     config = function()
       require('sql-cli').setup({
         command = vim.fn.expand("~/dev/sql-cli/target/release/sql-cli"),
       })
     end,
   }
   ```

## License

Same as SQL CLI - MIT