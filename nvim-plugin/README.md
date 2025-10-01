# SQL CLI Neovim Plugin

A Neovim plugin for the SQL CLI tool that provides seamless integration for executing SQL queries directly from your editor.

## Features

- 🚀 Execute SQL queries/scripts directly from Neovim
- 📊 Split pane output window with results
- 🔄 Dynamic split orientation toggle (vertical/horizontal)
- 🎯 Execute query at cursor (smart SELECT to GO/semicolon detection)
- 📝 Visual selection execution
- 📄 View data files directly in buffer
- 🔍 Auto-detect CSV files and data hints
- ⌨️ Customizable keymaps
- 🎯 Query plan visualization
- 🔄 Async execution with live output
- 🎨 Syntax highlighting for output (tables, numbers, errors, success messages)
- 📚 Built-in function help system (LSP-like documentation)
- 🔎 Function search and discovery
- 🔤 Intelligent autocompletion for column names, SQL functions, and keywords
- ⚡ Generator function discovery and help (FIBONACCI, GENERATE_PRIMES, etc.)

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
  -- If formatting doesn't work, try setting the full path:
  -- command = "/home/user/sql-cli/target/release/sql-cli",
  command = "sql-cli",
  
  -- Split configuration
  split = {
    direction = "vertical", -- "vertical" or "horizontal"
    size = 0.5,            -- Size as fraction (0.5 = 50%)
  },
  
  -- Default output format
  output_format = "table",

  -- Table output settings (when output_format = "table")
  table_output = {
    max_col_width = 50,      -- Maximum column width (0 = unlimited, default: 50)
    col_sample_rows = 100,   -- Rows to sample for width (0 = all rows, default: 100)
  },

  -- SQL Formatting preferences (NEW!)
  format = {
    lowercase = false,     -- Use lowercase keywords
    compact = false,       -- Compact formatting
    tabs = false,         -- Use tabs instead of spaces
  }, -- "table", "csv", "json", "tsv"
  
  -- Auto-detect features
  auto_detect = {
    csv_files = true,      -- Auto-use CSV files as data source
    data_hints = true,     -- Auto-detect -- #!data: hints
  },
  
  -- Keymaps (set to false to disable)
  keymaps = {
    execute = "<leader>sq",         -- Execute entire buffer
    execute_selection = "<leader>ss", -- Execute visual selection
    execute_at_cursor = "<leader>sx", -- Execute query at cursor
    toggle_output = "<leader>so",   -- Toggle output window
    toggle_orientation = "<leader>st", -- Toggle split orientation
    set_data_file = "<leader>sd",   -- Set data file
    clear_data_file = "<leader>sc", -- Clear data file
    show_plan = "<leader>sp",       -- Show query plan
    open_data_file = "<leader>sv",  -- View data file in buffer
    function_help = "K",            -- Show function/generator help at cursor (Shift+K)
    list_functions = "<leader>sL",  -- List all SQL functions
    list_generators = "<leader>sG", -- List all generator functions
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

### Function and Generator Help

The plugin provides intelligent help for both SQL functions and generator functions:

- **Shift+K** on any function or generator name shows detailed help in a floating window
- Works with both regular functions (MEDIAN, SQRT, etc.) and generators (FIBONACCI, GENERATE_PRIMES, etc.)
- Provides signature, description, examples, and argument information
- If the word under cursor isn't found, suggests similar functions/generators

Example:
```sql
-- Place cursor on FIBONACCI and press Shift+K to see help
SELECT * FROM FIBONACCI(20);

-- Place cursor on MEDIAN and press Shift+K to see help
SELECT MEDIAN(salary) FROM employees;
```

### Basic Commands

- `:SqlCliExecute` - Execute current buffer as SQL
- `:SqlCliSetData <file>` - Set data file for queries
- `:SqlCliClearData` - Clear data file setting
- `:SqlCliShowPlan` - Show query execution plan
- `:SqlCliToggleOutput` - Toggle output window
- `:SqlCliCopyQuery` - Copy query at cursor to clipboard
- `:SqlCliFormatQuery` - Format/prettify SQL query at cursor

### Default Keymaps

#### Query Execution
- `<leader>sq` - Execute SQL query (normal mode)
- `<leader>ss` - Execute selected SQL (visual mode)
- `<leader>sx` - Execute query at cursor (from SELECT to GO/semicolon)
- `<leader>sp` - Show query plan
- `<leader>sy` - Copy query at cursor to clipboard
- `<leader>sY` - Copy query as shell command (formats as `sql-cli -q "query"` with proper escaping)
- `<leader>sf` - Format/prettify query at cursor (uses AST-based formatter)

#### Navigation
- `]q` - Jump to next query
- `[q` - Jump to previous query
- `<leader>s/` - Toggle comment for query at cursor

#### Output Management
- `<leader>so` - Toggle output window
- `<leader>st` - Toggle split orientation (vertical/horizontal)
- `<leader>sw` - Save results to CSV file
- `<leader>sb` - Open results in new buffer

#### Data Files
- `<leader>sd` - Set data file
- `<leader>sc` - Clear data file
- `<leader>sv` - View data file in buffer

#### Function Help (LSP-like features)
- `K` - Show help for SQL function under cursor
- `<leader>sL` - List all available SQL functions
- `<leader>sF` - Search SQL functions interactively

#### Schema & Column Information
- `<leader>sh` - Show table schema (columns and types)
- `<leader>sk` - Smart detection at cursor (shows column info or function help)

#### Multi-Table Navigation
- `]t` - Jump to next result table
- `[t` - Jump to previous result table
- `<leader>s1` - Jump to first result table
- `<leader>s2` - Jump to second result table
- `<leader>s3` - Jump to third result table
- `<leader>sI` - Show current table info (debug modal)

#### SQL Refactoring & CASE Generation
- `<leader>sD` - Show distinct values for column under cursor (preview in floating window)
- `<leader>sC` - Generate CASE statement for column under cursor (auto-detects data file)
- `<leader>sr` - Generate CASE for numeric range (e.g., for RANGE() function)
- `<leader>sd` - Generate CASE from data file with column picker
- `<leader>sb` - Generate banding CASE statement
- `<leader>sp` - Pivot/conditional aggregation builder

#### Window Function Helpers
- `<leader>sw` - Window function wizard (interactive menu)
- `<leader>swr` - Add ROW_NUMBER() ranking with partition
- `<leader>swl` - Add LAG/LEAD for delta calculations
- `<leader>swa` - Add windowed aggregates (running totals, moving averages)
- `<leader>swk` - Add RANK functions (RANK, DENSE_RANK, PERCENT_RANK, NTILE)
- `<leader>swv` - Add FIRST_VALUE/LAST_VALUE

#### Autocompletion
- `<C-Space>` - Trigger SQL autocompletion (in INSERT mode)
  - Completes column names from current data file
  - Suggests SQL functions with descriptions
  - Includes SQL keywords
  - Sorted by relevance: columns → functions → keywords
- `<C-x><C-o>` - Alternative way to trigger completion (Vim's standard omnicompletion)

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

## Window Function Helpers

The plugin provides interactive wizards to quickly generate window functions, which are often complex to write:

### Quick Access with `<leader>sw`
Press `<leader>sw` to open an interactive menu with all window function patterns:
- ROW_NUMBER() for ranking
- LAG/LEAD for delta calculations
- Windowed aggregates (running totals, moving averages)
- RANK functions (RANK, DENSE_RANK, PERCENT_RANK, NTILE)
- FIRST/LAST VALUE for boundary values

### Common Patterns

#### Add Ranking (`<leader>swr`)
Place cursor on a column and press `<leader>swr` to add ROW_NUMBER():
```sql
-- Before: cursor on 'region'
SELECT region, salesperson, sales_amount FROM test;

-- After: adds ranking partitioned by region
SELECT region, salesperson, sales_amount,
    ROW_NUMBER() OVER (PARTITION BY region ORDER BY sales_amount DESC) as region_rank
FROM test;
```

#### Calculate Deltas (`<leader>swl`)
Place cursor on a numeric column and press `<leader>swl` to add LAG/LEAD:
```sql
-- Before: cursor on 'sales_amount'
SELECT salesperson, month, sales_amount FROM test;

-- After: adds previous value and change calculation
SELECT salesperson, month, sales_amount,
    LAG(sales_amount, 1) OVER (ORDER BY month) as prev_sales_amount,
    sales_amount - LAG(sales_amount, 1) OVER (ORDER BY month) as sales_amount_change
FROM test;
```

#### Running Totals (`<leader>swa`)
Add cumulative or moving window aggregates:
```sql
-- Running total example
SELECT month, sales_amount,
    SUM(sales_amount) OVER (ORDER BY month ROWS UNBOUNDED PRECEDING) as running_sum
FROM test;

-- Moving average example (3-row window)
SELECT month, sales_amount,
    AVG(sales_amount) OVER (ORDER BY month ROWS BETWEEN 1 PRECEDING AND 1 FOLLOWING) as moving_avg
FROM test;
```

### Interactive Prompts
Each wizard guides you through:
1. **Column selection** - Which column to analyze
2. **Partitioning** - Group calculations by specific columns
3. **Ordering** - Sort order for window calculations
4. **Aliases** - Custom names for result columns

The functions are inserted inline at your cursor position with proper SQL formatting.

## SQL Refactoring & Code Generation

The plugin includes powerful SQL refactoring tools to speed up query development:

### CASE Statement Generation

#### Explore Column Values (`<leader>sD`)
Place your cursor on any column name and press `<leader>sD` to see distinct values:
- Shows top 20 distinct values by frequency
- Displays total distinct count
- Helps understand data distribution before writing CASE statements
- Works with any data file (CSV, or file CTEs)

#### Generate CASE from Column (`<leader>sC`)
Place cursor on a column name and press `<leader>sC` to automatically generate a CASE statement:
- Auto-detects data file from context
- For categorical columns (≤20 distinct values): Creates CASE with exact values
- For numeric columns (>20 values): Generates smart range-based bands
- Inserts inline after the column in SELECT statements

Example:
```sql
-- Before: cursor on ocean_proximity
SELECT ocean_proximity FROM house_prices;

-- After pressing \sC:
SELECT ocean_proximity,
    CASE
        WHEN ocean_proximity = '<1H OCEAN' THEN '<1H OCEAN'
        WHEN ocean_proximity = 'INLAND' THEN 'INLAND'
        WHEN ocean_proximity = 'ISLAND' THEN 'ISLAND'
        WHEN ocean_proximity = 'NEAR BAY' THEN 'NEAR BAY'
        WHEN ocean_proximity = 'NEAR OCEAN' THEN 'NEAR OCEAN'
        ELSE 'Other'
    END AS ocean_proximity_category
FROM house_prices;
```

#### Generate CASE for Ranges (`<leader>sr`)
Perfect for synthetic data from RANGE() or numeric columns:
- Prompts for column name, range (e.g., 1-100), and number of bands
- Offers preset labels (Very Low/Low/Medium/High/Very High, Level 1-5, etc.)
- Generates equal-width bands automatically

Example with RANGE():
```sql
-- Before:
SELECT * FROM RANGE(1, 100);

-- After \sr with 5 bands:
SELECT *,
    CASE
        WHEN value <= 20 THEN 'Level 1'
        WHEN value > 20 AND value <= 40 THEN 'Level 2'
        WHEN value > 40 AND value <= 60 THEN 'Level 3'
        WHEN value > 60 AND value <= 80 THEN 'Level 4'
        WHEN value > 80 THEN 'Level 5'
    END AS value_band
FROM RANGE(1, 100);
```

#### Generate CASE from Data File (`<leader>sd`)
Interactive CASE generation with column picker:
- Shows all columns with their data types
- Automatically determines best CASE style (values vs ranges)
- Smart data file detection from hints or current context

### Window Functions & Aggregations

#### Window Function Wizard (`<leader>sw`)
Interactive builder for window functions:
- ROW_NUMBER(), RANK(), DENSE_RANK()
- LAG/LEAD for change detection
- Running totals and moving averages
- Guides through PARTITION BY and ORDER BY setup

#### Conditional Aggregation Builder (`<leader>sp`)
Generate pivot-style conditional aggregations:
- Transforms rows to columns
- Creates SUM(CASE WHEN...) patterns
- Useful for pivot tables and cross-tabs

## Multi-Table Navigation

When executing SQL scripts with multiple queries separated by `GO` statements, the plugin now supports navigation between result tables:

### Navigation Keys
- `]t` - Jump to next result table
- `[t` - Jump to previous result table
- `<leader>s1` - Jump directly to first table
- `<leader>s2` - Jump directly to second table
- `<leader>s3` - Jump directly to third table
- `<leader>sI` - Show current table info (debug modal) (e.g., "Table 2/4 (10 rows)")

### Example Workflow

1. Open a multi-query SQL file: `:e examples/chemistry.sql`
2. Execute with `<leader>sx` to run all queries
3. Use `]t` to navigate forward through result tables
4. Use `[t` to navigate backward through result tables
5. Use `<leader>s1` to jump back to the first table
6. Use `<leader>sI` to see which table you're currently viewing

The navigation wraps around - pressing `]t` on the last table takes you to the first, and `[t` on the first table takes you to the last.

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

## Token Management (JWT Auto-Refresh)

For APIs with short-lived tokens (e.g., 15-minute expiry), the plugin can automatically refresh tokens:

### Configuration

```lua
require('sql-cli').setup({
  -- ... other config ...

  token = {
    -- Your token endpoint URL
    token_endpoint = 'http://localhost:5000/token',

    -- Auto-refresh before expiry
    auto_refresh = true,

    -- Refresh every 14 minutes (for 15-min tokens)
    refresh_interval = 14 * 60,

    -- JSON path to extract token
    token_path = 'token',  -- For {"token": "xxx"}
  }
})
```

### Commands

- `:TokenRefresh` - Manually refresh token
- `:TokenShow` - Display current token (truncated)
- `:TokenConfig <url>` - Set token endpoint

### Usage in Macros

```sql
-- MACRO: CONFIG
-- API_TOKEN = ${JWT_TOKEN}  -- Auto-uses fresh token
-- END MACRO
```

The token refreshes automatically, no need to exit Neovim!

## Tips

- The plugin remembers your last query, so toggling the output window will re-run it
- Use `:SqlCliSetData` with tab completion to quickly switch data files
- Add `-- #!data:` hints to your SQL files for reproducible queries
- The output window is a regular buffer, so you can search, copy, etc.
- Syntax highlighting is automatically applied to output for better readability
- SQL files get additional buffer-local keymaps (`<LocalLeader>r` to run, `<LocalLeader>p` for plan)
- Autocompletion works best after setting a data file - it reads the schema automatically
- Use `<C-Space>` in INSERT mode to trigger intelligent SQL completion

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