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

  -- Table output settings
  table_output = {
    max_col_width = 50,      -- Maximum column width (0 = unlimited, default: 50)
    col_sample_rows = 100,   -- Rows to sample for column width (0 = all rows, default: 100)
    auto_hide_empty = false, -- Automatically hide columns that are all NULL/empty (default: false)
    style = "default",       -- Table style: default, ascii, markdown, utf8, etc. (see --list-table-styles)
  },

  -- Formatting preferences
  format = {
    lowercase = false,     -- Use lowercase keywords
    compact = false,       -- Compact formatting
    tabs = false,         -- Use tabs instead of spaces
  },

  -- Redis cache settings
  cache = {
    enabled = false,       -- Enable Redis cache (set true to use cache)
    redis_url = nil,      -- Custom Redis URL (nil = use default localhost:6379)
    show_notifications = true, -- Show cache hit/miss notifications
  },

  -- Auto-detect features
  auto_detect = {
    csv_files = true,      -- Auto-detect when editing CSV files
    data_hints = true,     -- Auto-detect -- #!data: hints in SQL files
  },

  -- Smart expansion features
  smart_expansion = {
    enabled = true,                    -- Enable smart * expansion (executes query to get columns)
    auto_insert_column_hints = true,   -- Auto-insert column hint comments for completion
    auto_sync_column_hints = true,     -- Sync columns from results buffer to query buffer
    fallback_to_static = true,         -- Fall back to static schema if query execution fails
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
    reset_layout = "<leader>sR",    -- Reset/recover output layout (capital R for reset)
    set_data_file = "<leader>sD",   -- Set data file (capital D)
    clear_data_file = nil, -- Removed - conflicted with refactoring
    show_plan = "<leader>sp",       -- Show query plan
    format_query = "<leader>sf",    -- Format query at cursor (primary mapping)
    open_data_file = "<leader>sV",  -- View data file (capital V to avoid conflict)
    next_query = "]q",              -- Jump to next query
    prev_query = "[q",              -- Jump to previous query
    toggle_comment = "<leader>s/",  -- Toggle comment for query at cursor
    save_results_csv = "<leader>sW", -- Write results to CSV file (capital W)
    results_to_buffer = nil, -- Removed - conflicts with export browser
    function_help = "K",            -- Show function help at cursor
    list_functions = "<leader>sL",  -- List all SQL functions (capital L)
    list_generators = "<leader>sG",  -- List all generator functions (capital G)
    search_functions = "<leader>sF", -- Search SQL functions
    show_schema = "<leader>sh",     -- Show table schema
    column_help = "<leader>sk",     -- Smart column/function detection at cursor
    expand_star = "<leader>sE",     -- Expand SELECT * to column names (capital E)
    copy_query = "<leader>sy",       -- Copy query at cursor to clipboard (y for yank)
    copy_query_shell = "<leader>sY", -- Copy query for shell execution (capital Y)
    toggle_table_nav = "<leader>sn", -- Toggle table navigation mode (n for nav) - legacy
    -- Table navigation (works in results buffer only, \sT prefix for which-key)
    table_toggle = "<leader>sTt",       -- Toggle table navigation on/off (t for toggle)
    table_nav_left = "<leader>sTh",     -- Navigate left (h in table nav)
    table_nav_down = "<leader>sTj",     -- Navigate down (j in table nav)
    table_nav_up = "<leader>sTk",       -- Navigate up (k in table nav)
    table_nav_right = "<leader>sTl",    -- Navigate right (l in table nav)
    table_nav_first_col = "<leader>sT0", -- Jump to first column (0)
    table_nav_last_col = "<leader>sT$",  -- Jump to last column ($)
    table_nav_first_row = "<leader>sTgg", -- Jump to first row (gg)
    table_nav_last_row = "<leader>sTG",  -- Jump to last row (G)
    table_yank_cell = "<leader>sTy",    -- Yank current cell
    table_yank_row = "<leader>sTY",     -- Yank entire row
    table_yank_column = "<leader>sTc",  -- Yank entire column
    table_yank_column_json = "<leader>sTJ", -- Yank column as JSON array (capital J)
    table_diagnostic = "<leader>sTd",   -- Show table diagnostic info
    -- Multi-table navigation (for scripts with GO statements, \sT prefix)
    table_next = "<leader>sTn",     -- Jump to next result table (n for next)
    table_prev = "<leader>sTp",     -- Jump to previous result table (p for prev)
    table_goto_1 = "<leader>sT1",   -- Jump to result table 1
    table_goto_2 = "<leader>sT2",   -- Jump to result table 2
    table_goto_3 = "<leader>sT3",   -- Jump to result table 3
    table_info = "<leader>sTi",     -- Show current table info (i for info)
    -- Cache management
    cache_purge = "<leader>sCP",    -- Purge all cache entries (capital CP)
    -- Export keymaps (when in table navigation mode)
    export_menu = "<leader>se",     -- Export menu (all formats)
    export_browser = nil,  -- Removed - use export menu instead
    -- CTE testing and analysis
    test_cte = "<leader>sC",         -- Test CTE at cursor (C for CTE)
    test_cte_new = "<leader>sN",     -- Test CTE in new buffer (N for new)
    cte_analysis = "<leader>sA",     -- Show CTE analysis popup (A for Analysis)
    analyze_query = "<leader>sJ",    -- Show raw query analysis JSON (J for JSON)
    export_markdown = "<leader>sm", -- Export as Markdown
    export_tsv = "<leader>sT",      -- Export as Tab-separated (Excel) - capital T to avoid template conflict
    -- Chart visualizations (execute query at cursor and visualize)
    bar_chart_at_cursor = "<leader>sB",     -- Bar chart from query at cursor (capital B)
    pie_chart_at_cursor = nil,     -- Removed - conflicts with preview
    histogram_at_cursor = "<leader>sH",     -- Histogram from query at cursor (capital H)
    scatter_at_cursor = nil,       -- Removed - conflicts with selection
    sparkline_at_cursor = "<leader>sl",     -- Sparkline from query at cursor (lowercase l for line)
    -- Parameter management
    cycle_param_next = "<leader>spn",    -- Next parameter value (p for param, n for next)
    cycle_param_prev = "<leader>spp",    -- Previous parameter value (p for param, p for prev)
    save_param_set = "<leader>sps",      -- Save parameter set (p for param, s for save)
    -- Query history
    query_history = "<leader>sQ",        -- Query history recall (capital Q for query)
    export_history = "<leader>se",       -- Export query history (lowercase e for export)
    import_history = "<leader>si",       -- Import query history (lowercase i for import)
    -- Column statistics
    sum_column = "<leader>sS",           -- Sum/statistics for column at cursor (capital S for Sum)
    show_totals = nil,                   -- Show totals row (disabled by default, use command)
  },

  -- Output window settings
  output = {
    focus_on_run = false,  -- Focus output window after execution
    clear_on_run = true,   -- Clear output before each run
    wrap = false,          -- Line wrap in output
    number = false,        -- Show line numbers in output
  },

  -- Query history settings
  query_history = {
    persist = true,        -- Automatically save/load history
    max_items = 100,       -- Maximum number of queries to keep
    auto_save = true,      -- Save after each new query
  },

  -- Debug settings (usually false)
  debug = false,           -- Enable debug logging for the plugin
  debug_format = false,    -- Show debug messages for formatting

  -- Logging settings
  logging = {
    enabled = true,        -- Enable file logging
    level = "INFO",        -- Log level: TRACE, DEBUG, INFO, WARN, ERROR
    max_files = 20,        -- Maximum number of log files to keep
    buffer_size = 100,     -- Number of messages to buffer before flush
    auto_flush_interval = 5000, -- Auto-flush interval in milliseconds
  },

  -- Custom syntax highlighting rules
  syntax = {
    -- Path to custom syntax file (Lua or TOML)
    -- Example: "~/.config/sql-cli/syntax.lua"
    custom_file = nil,

    -- Built-in custom patterns (can be overridden by custom_file)
    patterns = {
      -- Trading patterns (example)
      -- { pattern = [[\<Buy\>]], group = "SqlCliBuy", color = { gui = "#50fa7b", cterm = "Green", bold = true } },
      -- { pattern = [[\<Sell\>]], group = "SqlCliSell", color = { gui = "#ff5555", cterm = "Red", bold = true } },
    }
  },
}

-- Validate and merge config
function M.setup(user_config)
  return vim.tbl_deep_extend("force", M.defaults, user_config or {})
end

return M