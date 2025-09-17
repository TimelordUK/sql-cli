-- SQL CLI Configuration Module
-- Default configuration and validation

local M = {}

-- Default configuration
M.defaults = {
  -- Path to sql-cli executable
  -- Set this to the full path if sql-cli is not in PATH
  -- e.g., command = "/home/user/sql-cli/target/release/sql-cli"
  command = "sql-cli",

  -- Split configuration
  split = {
    direction = "vertical", -- "vertical" or "horizontal"
    size = 0.5,            -- Size as fraction (0.5 = 50%)
  },

  -- Default output format
  output_format = "table",

  -- Formatting preferences
  format = {
    lowercase = false,     -- Use lowercase keywords
    compact = false,       -- Compact formatting
    tabs = false,         -- Use tabs instead of spaces
  },

  -- Auto-detect features
  auto_detect = {
    csv_files = true,      -- Auto-detect when editing CSV files
    data_hints = true,     -- Auto-detect -- #!data: hints in SQL files
  },

  -- Table navigation settings
  table_navigation = {
    enabled_by_default = false,  -- Don't auto-enable table navigation (requires manual toggle)
    hijack_hjkl = true,          -- When enabled, use hjkl for table navigation (set false for normal vim movement)
  },

  -- Keymaps (set to false to disable)
  keymaps = {
    execute = "<leader>sq",         -- Execute query
    execute_selection = "<leader>ss", -- Execute visual selection
    execute_at_cursor = "<leader>sx", -- Execute query at cursor
    execute_with_plan = "<leader>sX", -- Execute query at cursor with execution plan
    select_query = "<leader>sv",    -- Visually select query at cursor
    preview_query = "<leader>sP",   -- Preview query in floating window
    toggle_output = "<leader>sO",   -- Toggle output window (capital O)
    toggle_orientation = "<leader>so", -- Toggle split orientation (lowercase o)
    set_data_file = "<leader>sd",   -- Set data file
    clear_data_file = "<leader>sc", -- Clear data file
    show_plan = "<leader>sp",       -- Show query plan
    format_query = "<leader>sf",    -- Format query at cursor (primary mapping)
    open_data_file = "<leader>sV",  -- View data file (capital V to avoid conflict)
    next_query = "]q",              -- Jump to next query
    prev_query = "[q",              -- Jump to previous query
    toggle_comment = "<leader>s/",  -- Toggle comment for query at cursor
    save_results_csv = "<leader>sw", -- Write results to CSV file
    results_to_buffer = "<leader>sb", -- Results to new buffer
    function_help = "K",            -- Show function help at cursor
    list_functions = "<leader>sL",  -- List all SQL functions (capital L)
    list_generators = "<leader>sG",  -- List all generator functions (capital G)
    search_functions = "<leader>sF", -- Search SQL functions
    show_schema = "<leader>sh",     -- Show table schema
    column_help = "<leader>sk",     -- Smart column/function detection at cursor
    expand_star = "<leader>sE",     -- Expand SELECT * to column names (capital E)
    copy_query = "<leader>sy",       -- Copy query at cursor to clipboard (y for yank)
    toggle_table_nav = "<leader>sn", -- Toggle table navigation mode (n for nav)
    -- Multi-table navigation (for scripts with GO statements)
    next_table = "]t",              -- Jump to next result table
    prev_table = "[t",              -- Jump to previous result table
    goto_table_1 = "<leader>s1",    -- Jump to first table
    goto_table_2 = "<leader>s2",    -- Jump to second table
    goto_table_3 = "<leader>s3",    -- Jump to third table
    table_info = "<leader>sI",      -- Show current table info (capital I to avoid conflict)
    -- Export keymaps (when in table navigation mode)
    export_menu = "<leader>se",     -- Export menu (all formats)
    export_browser = "<leader>sb",  -- Open in Browser for Gmail/Teams
    export_markdown = "<leader>sm", -- Export as Markdown
    export_tsv = "<leader>st",      -- Export as Tab-separated (Excel)
    -- Chart visualizations (execute query at cursor and visualize)
    bar_chart_at_cursor = "<leader>sB",     -- Bar chart from query at cursor (capital B)
    pie_chart_at_cursor = "<leader>sP",     -- Pie chart from query at cursor (capital P)
    histogram_at_cursor = "<leader>sH",     -- Histogram from query at cursor (capital H)
    scatter_at_cursor = "<leader>sS",       -- Scatter plot from query at cursor (capital S)
    sparkline_at_cursor = "<leader>sl",     -- Sparkline from query at cursor (lowercase l for line)
  },

  -- Output window settings
  output = {
    focus_on_run = false,  -- Focus output window after execution
    clear_on_run = true,   -- Clear output before each run
    wrap = false,          -- Line wrap in output
    number = false,        -- Show line numbers in output
  },

  -- Debug settings (usually false)
  debug = false,           -- Enable debug logging for the plugin
  debug_format = false,    -- Show debug messages for formatting
}

-- Validate and merge config
function M.setup(user_config)
  return vim.tbl_deep_extend("force", M.defaults, user_config or {})
end

return M