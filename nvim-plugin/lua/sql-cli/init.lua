-- SQL CLI Neovim Plugin
-- Main entry point that coordinates all modules

local M = {}

-- Load modules
local config = require('sql-cli.config')
local logger = require('sql-cli.logger')
local state = require('sql-cli.state')
local utils = require('sql-cli.utils')
local executor = require('sql-cli.executor')
local formatter = require('sql-cli.formatter')
local ui = require('sql-cli.ui')
local navigation = require('sql-cli.navigation')
local functions = require('sql-cli.functions')
local results = require('sql-cli.results')
local table_nav = require('sql-cli.table_nav')
local multi_table_nav = require('sql-cli.multi_table_nav')
local visualize = require('sql-cli.visualize')
local cte_tester = require('sql-cli.cte_tester')  -- Uses CLI analysis instead of regex
local cte_parser_cli = require('sql-cli.cte_parser_cli')  -- CTE analysis popup
local refactoring = require('sql-cli.refactoring')
local window_functions = require('sql-cli.window_functions')

-- Plugin configuration
M.config = {}

-- Plugin state
M.state = nil

-- Setup function
function M.setup(opts)
  -- Initialize configuration
  M.config = config.setup(opts)

  -- Initialize logging system
  if M.config.logging and M.config.logging.enabled then
    local log_config = {
      enabled = M.config.logging.enabled,
      level = logger.levels[M.config.logging.level] or logger.levels.INFO,
      max_files = M.config.logging.max_files or 20,
      buffer_size = M.config.logging.buffer_size or 100,
      auto_flush_interval = M.config.logging.auto_flush_interval or 5000,
    }
    logger.init(log_config)
    logger.info('init', 'SQL CLI plugin initialized')
    logger.info('init', 'Neovim version: ' .. vim.version().major .. '.' .. vim.version().minor .. '.' .. vim.version().patch)
  end

  -- Initialize state
  M.state = state.new()

  -- Initialize query history with config
  require('sql-cli.query_history').init(M.config)

  -- Add UI callbacks to config
  M.config.ui_callbacks = {
    create_output_window = function()
      ui.create_output_window(M.config, M.state)
    end,
    is_output_window_valid = function()
      return ui.is_output_window_valid(M.state)
    end,
  }

  -- Add load schema callback to config
  M.config.load_schema_callback = function()
    functions.load_schema_for_completion(M.config, M.state)
  end

  -- Setup commands
  M.create_commands()

  -- Setup keymaps
  M.setup_keymaps()

  -- Setup autocommands
  M.setup_autocmds()

  -- Load completion schema if available
  if M.config.auto_detect.csv_files then
    functions.load_schema_for_completion(M.config, M.state)
  end

  -- Setup visualization commands
  visualize.setup()

  -- Setup refactoring keymaps
  refactoring.setup_keymaps(M.config)

  -- Setup window function keymaps
  window_functions.setup_keymaps(M.config)

  -- Setup new template system
  local template = require('sql-cli.template')
  template.setup(M.config.template or {})
  template.create_commands()
  template.create_keymaps()

  -- Setup token manager
  local token_manager = require('sql-cli.token_manager')
  token_manager.setup(M.config.token or {})
  token_manager.create_commands()
end

-- Create user commands
function M.create_commands()
  vim.api.nvim_create_user_command("SqlCliExecute", function(opts)
    executor.execute_query(opts.args, M.config, M.state)
  end, { nargs = "?", desc = "Execute SQL query" })

  vim.api.nvim_create_user_command("SqlCliExecuteSelection", function()
    executor.execute_selection(M.config, M.state)
  end, { desc = "Execute selected SQL query" })

  vim.api.nvim_create_user_command("SqlCliExecuteAtCursor", function()
    executor.execute_at_cursor(M.config, M.state)
  end, { desc = "Execute SQL query at cursor" })

  vim.api.nvim_create_user_command("SqlCliExecuteStatementAtCursor", function()
    executor.execute_statement_at_cursor(M.config, M.state)
  end, { desc = "Execute statement at cursor with dependencies" })

  vim.api.nvim_create_user_command("SqlCliSetDataFile", function(opts)
    M.set_data_file(opts.args)
  end, { nargs = 1, complete = "file", desc = "Set data file for SQL queries" })

  vim.api.nvim_create_user_command("SqlCliClearDataFile", function()
    M.clear_data_file()
  end, { desc = "Clear data file" })

  vim.api.nvim_create_user_command("SqlCliToggleOutput", function()
    ui.toggle_output_window(M.config, M.state)
  end, { desc = "Toggle SQL output window" })

  vim.api.nvim_create_user_command("SqlCliResetLayout", function()
    ui.reset_output_layout(M.config, M.state)
  end, { desc = "Reset/recover output window layout (fixes corrupted state)" })

  vim.api.nvim_create_user_command("SqlCliShowPlan", function()
    M.show_query_plan()
  end, { desc = "Show query execution plan" })

  vim.api.nvim_create_user_command("SqlCliSelectQuery", function()
    navigation.select_query_at_cursor()
  end, { desc = "Select SQL query at cursor" })

  vim.api.nvim_create_user_command("SqlCliCopyQuery", function()
    navigation.copy_query_at_cursor()
  end, { desc = "Copy SQL query at cursor to clipboard" })

  vim.api.nvim_create_user_command("SqlCliCopyQueryShell", function()
    navigation.copy_query_for_shell()
  end, { desc = "Copy SQL query for shell execution" })

  vim.api.nvim_create_user_command("SqlCliPreviewQuery", function()
    ui.preview_query_at_cursor(M.config)
  end, { desc = "Preview SQL query at cursor in floating window" })

  vim.api.nvim_create_user_command("SqlCliFormatQuery", function()
    formatter.format_query_at_cursor(M.config, M.state)
  end, { desc = "Format SQL query at cursor" })

  vim.api.nvim_create_user_command("SqlCliListFunctions", function()
    functions.list_functions(M.config)
  end, { desc = "List all SQL functions" })

  vim.api.nvim_create_user_command("SqlCliListGenerators", function()
    functions.list_generators(M.config)
  end, { desc = "List all SQL generators" })

  vim.api.nvim_create_user_command("SqlCliSearchFunctions", function(opts)
    functions.search_functions(opts.args, M.config)
  end, { nargs = 1, desc = "Search SQL functions" })

  vim.api.nvim_create_user_command("SqlCliSaveResults", function(opts)
    results.save_results_csv(opts.args, M.state)
  end, { nargs = "?", complete = "file", desc = "Save query results to CSV" })

  vim.api.nvim_create_user_command("SqlCliTestCte", function()
    cte_tester.test_cte_at_cursor(M.config, M.state)
  end, { desc = "Test CTE at cursor position" })

  vim.api.nvim_create_user_command("SqlCliTestCteNewBuffer", function()
    cte_tester.test_cte_in_new_buffer(M.config, M.state)
  end, { desc = "Test CTE in new buffer" })

  vim.api.nvim_create_user_command("SqlCliCteAnalysis", function()
    cte_parser_cli.show_cte_analysis_popup()
  end, { desc = "Show CTE analysis popup" })

  vim.api.nvim_create_user_command("SqlCliAnalyzeQuery", function()
    M.analyze_query_raw()
  end, { desc = "Show raw query analysis JSON" })

  vim.api.nvim_create_user_command("SqlCliResultsToBuffer", function()
    results.results_to_buffer(M.state)
  end, { desc = "Open query results in new buffer" })

  vim.api.nvim_create_user_command("SqlCliExpandStar", function()
    results.expand_star_smart(M.config, M.state)
  end, { desc = "Expand SELECT * to column names (smart)" })

  vim.api.nvim_create_user_command("SqlCliToggleTableNav", function()
    table_nav.toggle_navigation(nil, M.config)
  end, { desc = "Toggle table navigation mode in current buffer" })

  vim.api.nvim_create_user_command("SqlCliExportTable", function()
    local export = require('sql-cli.export')
    local bufnr = vim.api.nvim_get_current_buf()

    -- Parse table structure
    local table_nav_module = require('sql-cli.table_nav')
    local success = table_nav_module.init_navigation(bufnr)
    if success then
      local table_info = table_nav_module.get_table_info()
      export.show_export_menu(bufnr, table_info)
      table_nav_module.disable_navigation()
    else
      vim.notify("No table found in buffer", vim.log.levels.WARN)
    end
  end, { desc = "Export table in various formats" })

  -- Fuzzy filter commands
  vim.api.nvim_create_user_command("SqlCliFuzzyFilter", function()
    local fuzzy = require('sql-cli.fuzzy_filter')
    fuzzy.open_fuzzy_finder()
  end, { desc = "Open fuzzy finder for result filtering" })

  vim.api.nvim_create_user_command("SqlCliResetFilter", function()
    local fuzzy = require('sql-cli.fuzzy_filter')
    fuzzy.reset_filter()
  end, { desc = "Reset result filter" })

  -- Column sum commands
  vim.api.nvim_create_user_command("SqlCliSumColumn", function()
    local column_sum = require('sql-cli.column_sum')
    column_sum.sum_column_at_cursor()
  end, { desc = "Calculate sum/statistics for column at cursor" })

  vim.api.nvim_create_user_command("SqlCliShowTotals", function()
    local column_sum = require('sql-cli.column_sum')
    column_sum.show_all_totals()
  end, { desc = "Add totals row to table" })

  -- Logging commands
  vim.api.nvim_create_user_command("SqlCliOpenLog", function()
    logger.open_log()
  end, { desc = "Open plugin log file" })

  vim.api.nvim_create_user_command("SqlCliTailLog", function()
    logger.tail_log()
  end, { desc = "Tail plugin log file (live updates)" })

  vim.api.nvim_create_user_command("SqlCliLogPath", function()
    local log_path = logger.get_log_path()
    if log_path then
      vim.notify('Log file: ' .. log_path, vim.log.levels.INFO)
      vim.fn.setreg('+', log_path)  -- Copy to clipboard
    else
      vim.notify('Logging not initialized', vim.log.levels.WARN)
    end
  end, { desc = "Show and copy log file path" })
end

-- Setup keymaps
function M.setup_keymaps()
  local keymaps = M.config.keymaps

  if keymaps.execute then
    vim.keymap.set("n", keymaps.execute, function()
      executor.execute_query(nil, M.config, M.state)
    end, { desc = "Execute SQL query", silent = true })
  end

  if keymaps.execute_selection then
    vim.keymap.set("v", keymaps.execute_selection, function()
      executor.execute_selection(M.config, M.state)
    end, { desc = "Execute selected SQL", silent = true })
  end

  if keymaps.execute_at_cursor then
    vim.keymap.set("n", keymaps.execute_at_cursor, function()
      executor.execute_at_cursor(M.config, M.state)
    end, { desc = "Execute SQL at cursor", silent = true })
  end

  if keymaps.execute_statement_at_cursor then
    vim.keymap.set("n", keymaps.execute_statement_at_cursor, function()
      executor.execute_statement_at_cursor(M.config, M.state)
    end, { desc = "Execute statement with dependencies", silent = true })
  end

  if keymaps.execute_with_plan then
    vim.keymap.set("n", keymaps.execute_with_plan, function()
      executor.execute_at_cursor_with_plan(M.config, M.state)
    end, { desc = "Execute SQL at cursor with plan", silent = true })
  end

  if keymaps.select_query then
    vim.keymap.set("n", keymaps.select_query, navigation.select_query_at_cursor,
      { desc = "Select SQL query at cursor", silent = true })
  end

  if keymaps.preview_query then
    vim.keymap.set("n", keymaps.preview_query, function()
      ui.preview_query_at_cursor(M.config)
    end, { desc = "Preview SQL query", silent = true })
  end

  if keymaps.toggle_output then
    vim.keymap.set("n", keymaps.toggle_output, function()
      ui.toggle_output_window(M.config, M.state)
    end, { desc = "Toggle SQL output", silent = true })
  end

  if keymaps.toggle_orientation then
    vim.keymap.set("n", keymaps.toggle_orientation, function()
      ui.toggle_split_orientation(M.config, M.state)
    end, { desc = "Toggle split orientation", silent = true })
  end

  if keymaps.reset_layout then
    vim.keymap.set("n", keymaps.reset_layout, function()
      ui.reset_output_layout(M.config, M.state)
    end, { desc = "Reset/recover output layout", silent = true })
  end

  if keymaps.set_data_file then
    vim.keymap.set("n", keymaps.set_data_file, function()
      M.set_data_file(nil)
    end, { desc = "Set SQL data file", silent = true })
  end

  if keymaps.clear_data_file then
    vim.keymap.set("n", keymaps.clear_data_file, M.clear_data_file,
      { desc = "Clear SQL data file", silent = true })
  end

  if keymaps.cache_purge then
    vim.keymap.set("n", keymaps.cache_purge, function()
      M.purge_cache()
    end, { desc = "Purge all cache entries", silent = true })
  end

  if keymaps.show_plan then
    vim.keymap.set("n", keymaps.show_plan, M.show_query_plan,
      { desc = "Show SQL query plan", silent = true })
  end

  if keymaps.open_data_file then
    vim.keymap.set("n", keymaps.open_data_file, M.open_data_file,
      { desc = "Open SQL data file", silent = true })
  end

  if keymaps.next_query then
    vim.keymap.set("n", keymaps.next_query, navigation.next_query,
      { desc = "Next SQL query", silent = true })
  end

  if keymaps.prev_query then
    vim.keymap.set("n", keymaps.prev_query, navigation.prev_query,
      { desc = "Previous SQL query", silent = true })
  end

  if keymaps.toggle_comment then
    vim.keymap.set("n", keymaps.toggle_comment, navigation.toggle_comment_query,
      { desc = "Toggle SQL comment", silent = true })
  end

  if keymaps.save_results_csv then
    vim.keymap.set("n", keymaps.save_results_csv, function()
      results.save_results_csv(nil, M.state)
    end, { desc = "Save results to CSV", silent = true })
  end

  -- Parameter management keymaps
  if keymaps.cycle_param_next then
    vim.keymap.set("n", keymaps.cycle_param_next, function()
      require('sql-cli.params').cycle_parameter("next")
    end, { desc = "Next parameter value", silent = true })
  end

  if keymaps.cycle_param_prev then
    vim.keymap.set("n", keymaps.cycle_param_prev, function()
      require('sql-cli.params').cycle_parameter("prev")
    end, { desc = "Previous parameter value", silent = true })
  end

  if keymaps.save_param_set then
    vim.keymap.set("n", keymaps.save_param_set, function()
      require('sql-cli.params').save_parameter_set()
    end, { desc = "Save parameter set", silent = true })
  end

  -- Query history
  if keymaps.query_history then
    vim.keymap.set("n", keymaps.query_history, function()
      local executor = require('sql-cli.executor')
      local params = require('sql-cli.params')

      require('sql-cli.query_history').show_history_picker(
        -- Insert callback
        function(query)
          -- Insert query into current buffer
          local lines = vim.split(query, '\n')

          -- Trim leading/trailing empty lines for cleaner insertion
          while #lines > 0 and lines[1]:match("^%s*$") do
            table.remove(lines, 1)
          end
          while #lines > 0 and lines[#lines]:match("^%s*$") do
            table.remove(lines)
          end

          local row, _ = unpack(vim.api.nvim_win_get_cursor(0))
          vim.api.nvim_buf_set_lines(0, row - 1, row - 1, false, lines)
          -- Move cursor to end of inserted text
          vim.api.nvim_win_set_cursor(0, {row + #lines - 1, #lines[#lines] - 1})
        end,
        -- Execute callback
        function(query)
          -- Check if query has parameters
          if query:match("{{[^}]+}}") then
            -- Resolve parameters first
            params.resolve_parameters(query, function(resolved_query)
              if resolved_query then
                -- Execute the resolved query
                executor.execute_query(resolved_query, M.config, M.state, true)  -- skip_params = true since already resolved
              end
            end)
          else
            -- No parameters, execute directly
            executor.execute_query(query, M.config, M.state, false)
          end
        end
      )
    end, { desc = "Recall query from history", silent = true })
  end

  -- Export query history
  if keymaps.export_history then
    vim.keymap.set("n", keymaps.export_history, function()
      require('sql-cli.query_history').export_history()
    end, { desc = "Export query history to JSON", silent = true })
  end

  -- Import query history
  if keymaps.import_history then
    vim.keymap.set("n", keymaps.import_history, function()
      -- Ask whether to merge or replace
      vim.ui.select({'Replace existing history', 'Merge with existing history'}, {
        prompt = 'Import mode:',
      }, function(choice)
        if choice then
          local merge = choice:match('Merge') ~= nil
          require('sql-cli.query_history').import_history(nil, merge)
        end
      end)
    end, { desc = "Import query history from JSON", silent = true })
  end

  if keymaps.results_to_buffer then
    vim.keymap.set("n", keymaps.results_to_buffer, function()
      results.results_to_buffer(M.state)
    end, { desc = "Results to buffer", silent = true })
  end

  if keymaps.test_cte then
    vim.keymap.set("n", keymaps.test_cte, function()
      cte_tester.test_cte_at_cursor(M.config, M.state)
    end, { desc = "Test CTE at cursor", silent = true })
  end

  if keymaps.test_cte_new then
    vim.keymap.set("n", keymaps.test_cte_new, function()
      cte_tester.test_cte_in_new_buffer(M.config, M.state)
    end, { desc = "Test CTE in new buffer", silent = true })
  end

  if keymaps.cte_analysis then
    vim.keymap.set("n", keymaps.cte_analysis, function()
      -- Use CLI parser by default as it's more robust
      cte_parser_cli.show_cte_analysis_popup()
    end, { desc = "Show CTE analysis", silent = true })
  end

  if keymaps.analyze_query then
    vim.keymap.set("n", keymaps.analyze_query, function()
      M.analyze_query_raw()
    end, { desc = "Show raw query analysis JSON", silent = true })
  end

  if keymaps.function_help then
    vim.keymap.set("n", keymaps.function_help, function()
      functions.function_help_at_cursor(M.config)
    end, { desc = "SQL function help", silent = true })
  end

  if keymaps.list_functions then
    vim.keymap.set("n", keymaps.list_functions, function()
      functions.list_functions(M.config)
    end, { desc = "List SQL functions", silent = true })
  end

  if keymaps.list_generators then
    vim.keymap.set("n", keymaps.list_generators, function()
      functions.list_generators(M.config)
    end, { desc = "List SQL generators", silent = true })
  end

  if keymaps.search_functions then
    vim.keymap.set("n", keymaps.search_functions, function()
      functions.search_functions(M.config)
    end, { desc = "Search SQL functions", silent = true })
  end

  if keymaps.show_schema then
    vim.keymap.set("n", keymaps.show_schema, function()
      functions.show_schema(M.config, M.state)
    end, { desc = "Show table schema", silent = true })
  end

  if keymaps.column_help then
    vim.keymap.set("n", keymaps.column_help, function()
      functions.column_help_at_cursor(M.config, M.state)
    end, { desc = "Column/function help", silent = true })
  end

  if keymaps.expand_star then
    vim.keymap.set("n", keymaps.expand_star, function()
      results.expand_star_smart(M.config, M.state)
    end, { desc = "Expand SELECT * (smart)", silent = true })

    vim.keymap.set("v", keymaps.expand_star, function()
      results.expand_star_visual(M.config, M.state)
    end, { desc = "Expand SELECT * in selection", silent = true })
  end

  if keymaps.copy_query then
    vim.keymap.set("n", keymaps.copy_query, navigation.copy_query_at_cursor,
      { desc = "Copy SQL query at cursor to clipboard", silent = true })
  end

  if keymaps.copy_query_shell then
    vim.keymap.set("n", keymaps.copy_query_shell, navigation.copy_query_for_shell,
      { desc = "Copy SQL query for shell execution", silent = true })
  end

  if keymaps.format_query then
    vim.keymap.set("n", keymaps.format_query, function()
      formatter.format_query_at_cursor(M.config, M.state)
    end, { desc = "Format SQL query at cursor", silent = true })
  end

  if keymaps.toggle_table_nav then
    vim.keymap.set("n", keymaps.toggle_table_nav, function()
      table_nav.toggle_navigation(nil, M.config)
    end, { desc = "Toggle table navigation mode", silent = true })
  end

  -- Helper to check if in results buffer and auto-enable table navigation
  local function require_results_buffer()
    local bufnr = vim.api.nvim_get_current_buf()
    local output_bufnr = M.state:get_output_buf()

    if not output_bufnr or bufnr ~= output_bufnr then
      vim.notify("This command only works in the results buffer", vim.log.levels.WARN)
      return false
    end

    -- Auto-enable table navigation if not already enabled
    if not table_nav.is_active() then
      if table_nav.init_navigation(bufnr) then
        table_nav.setup_keymaps(bufnr, M.config)
      else
        vim.notify("Could not find a table in the buffer", vim.log.levels.WARN)
        return false
      end
    end

    return true
  end

  -- Table toggle (works in results buffer, also accessible via legacy \sn)
  if keymaps.table_toggle then
    vim.keymap.set("n", keymaps.table_toggle, function()
      table_nav.toggle_navigation(nil, M.config)
    end, { desc = "Table: Toggle navigation mode", silent = true })
  end

  -- Table navigation keymaps (\sT prefix - all discoverable in which-key)
  if keymaps.table_nav_left then
    vim.keymap.set("n", keymaps.table_nav_left, function()
      if require_results_buffer() then
        table_nav.move_left()
      end
    end, { desc = "Table: Navigate left", silent = true })
  end

  if keymaps.table_nav_down then
    vim.keymap.set("n", keymaps.table_nav_down, function()
      if require_results_buffer() then
        table_nav.move_down()
      end
    end, { desc = "Table: Navigate down", silent = true })
  end

  if keymaps.table_nav_up then
    vim.keymap.set("n", keymaps.table_nav_up, function()
      if require_results_buffer() then
        table_nav.move_up()
      end
    end, { desc = "Table: Navigate up", silent = true })
  end

  if keymaps.table_nav_right then
    vim.keymap.set("n", keymaps.table_nav_right, function()
      if require_results_buffer() then
        table_nav.move_right()
      end
    end, { desc = "Table: Navigate right", silent = true })
  end

  if keymaps.table_nav_first_col then
    vim.keymap.set("n", keymaps.table_nav_first_col, function()
      if require_results_buffer() then
        table_nav.go_to_first_column()
      end
    end, { desc = "Table: First column", silent = true })
  end

  if keymaps.table_nav_last_col then
    vim.keymap.set("n", keymaps.table_nav_last_col, function()
      if require_results_buffer() then
        table_nav.go_to_last_column()
      end
    end, { desc = "Table: Last column", silent = true })
  end

  if keymaps.table_nav_first_row then
    vim.keymap.set("n", keymaps.table_nav_first_row, function()
      if require_results_buffer() then
        table_nav.go_to_first_row()
      end
    end, { desc = "Table: First row", silent = true })
  end

  if keymaps.table_nav_last_row then
    vim.keymap.set("n", keymaps.table_nav_last_row, function()
      if require_results_buffer() then
        table_nav.go_to_last_row()
      end
    end, { desc = "Table: Last row", silent = true })
  end

  if keymaps.table_yank_cell then
    vim.keymap.set("n", keymaps.table_yank_cell, function()
      if require_results_buffer() then
        table_nav.yank_cell()
      end
    end, { desc = "Table: Yank cell", silent = true })
  end

  if keymaps.table_yank_row then
    vim.keymap.set("n", keymaps.table_yank_row, function()
      if require_results_buffer() then
        table_nav.yank_row()
      end
    end, { desc = "Table: Yank row", silent = true })
  end

  if keymaps.table_yank_column then
    vim.keymap.set("n", keymaps.table_yank_column, function()
      if require_results_buffer() then
        table_nav.yank_column()
      end
    end, { desc = "Table: Yank column", silent = true })
  end

  if keymaps.table_yank_column_json then
    vim.keymap.set("n", keymaps.table_yank_column_json, function()
      if require_results_buffer() then
        table_nav.yank_column_as_json()
      end
    end, { desc = "Table: Yank column as JSON", silent = true })
  end

  if keymaps.table_diagnostic then
    vim.keymap.set("n", keymaps.table_diagnostic, function()
      if require_results_buffer() then
        table_nav.show_table_diagnostic()
      end
    end, { desc = "Table: Show diagnostic", silent = true })
  end

  -- Multi-table navigation keymaps (\sT prefix)
  -- These don't need table nav to be enabled, just need to be in results buffer
  local function in_results_buffer()
    local bufnr = vim.api.nvim_get_current_buf()
    local output_bufnr = M.state:get_output_buf()

    -- Debug info
    if M.logger then
      M.logger.info('init', string.format(
        'in_results_buffer check: current_buf=%d, output_bufnr=%s',
        bufnr,
        tostring(output_bufnr)
      ))
    end

    if not output_bufnr or bufnr ~= output_bufnr then
      if M.logger then
        M.logger.warn('init', 'Buffer check failed')
      end
      vim.notify(string.format(
        "This command only works in the results buffer (current=%d, expected=%s)",
        bufnr,
        tostring(output_bufnr)
      ), vim.log.levels.WARN)
      return false
    end
    return true
  end

  if keymaps.table_next then
    vim.keymap.set("n", keymaps.table_next, function()
      if in_results_buffer() then
        multi_table_nav.goto_next_table(M.state)
      end
    end, { desc = "Table: Next result table", silent = true })
  end

  if keymaps.table_prev then
    vim.keymap.set("n", keymaps.table_prev, function()
      if in_results_buffer() then
        multi_table_nav.goto_prev_table(M.state)
      end
    end, { desc = "Table: Previous result table", silent = true })
  end

  if keymaps.table_goto_1 then
    vim.keymap.set("n", keymaps.table_goto_1, function()
      if in_results_buffer() then
        multi_table_nav.goto_table(M.state, 1)
      end
    end, { desc = "Table: Go to table 1", silent = true })
  end

  if keymaps.table_goto_2 then
    vim.keymap.set("n", keymaps.table_goto_2, function()
      if in_results_buffer() then
        multi_table_nav.goto_table(M.state, 2)
      end
    end, { desc = "Table: Go to table 2", silent = true })
  end

  if keymaps.table_goto_3 then
    vim.keymap.set("n", keymaps.table_goto_3, function()
      if in_results_buffer() then
        multi_table_nav.goto_table(M.state, 3)
      end
    end, { desc = "Table: Go to table 3", silent = true })
  end

  if keymaps.table_info then
    vim.keymap.set("n", keymaps.table_info, function()
      if in_results_buffer() then
        multi_table_nav.debug_tables(M.state)
      end
    end, { desc = "Table: Show table info", silent = true })
  end

  -- Export keymaps (auto-enable table nav if needed)
  local function try_export(format_type)
    return function()
      local bufnr = vim.api.nvim_get_current_buf()
      local export = require('sql-cli.export')
      local table_nav_module = require('sql-cli.table_nav')

      -- Check if table nav is already active
      if table_nav_module.is_active() then
        local table_info = table_nav_module.get_table_info()
        if format_type == "menu" then
          export.show_export_menu(bufnr, table_info)
        elseif format_type == "browser" then
          export.yank_as_html(bufnr, table_info, true)
        elseif format_type == "markdown" then
          export.yank_as_markdown(bufnr, table_info)
        elseif format_type == "tsv" then
          export.yank_as_tsv(bufnr, table_info)
        end
      else
        -- First check if this is the output buffer
        local output_bufnr = M.state:get_output_buf()
        local target_bufnr = bufnr

        -- If we're not in the output buffer, switch to it if it exists
        if output_bufnr and output_bufnr ~= bufnr and vim.api.nvim_buf_is_valid(output_bufnr) then
          target_bufnr = output_bufnr
        end

        -- Try to activate table nav temporarily on the target buffer
        table_nav_module.init_navigation(target_bufnr)
        local table_info = table_nav_module.get_table_info()

        -- Check if we found a table
        if table_info and table_info.data_start then
          if format_type == "menu" then
            export.show_export_menu(target_bufnr, table_info)
          elseif format_type == "browser" then
            export.yank_as_html(target_bufnr, table_info, true)
          elseif format_type == "markdown" then
            export.yank_as_markdown(target_bufnr, table_info)
          elseif format_type == "tsv" then
            export.yank_as_tsv(target_bufnr, table_info)
          end
          -- Disable nav after export
          table_nav_module.disable_navigation()
        else
          vim.notify("No table found. Run a query first (\\sx) or ensure you're in the output buffer", vim.log.levels.WARN)
        end
      end
    end
  end

  if keymaps.export_menu then
    vim.keymap.set("n", keymaps.export_menu, try_export("menu"),
      { desc = "Export table (all formats)", silent = true })
  end

  if keymaps.export_browser then
    vim.keymap.set("n", keymaps.export_browser, try_export("browser"),
      { desc = "Export to browser (Gmail/Teams)", silent = true })
  end

  if keymaps.export_markdown then
    vim.keymap.set("n", keymaps.export_markdown, try_export("markdown"),
      { desc = "Export as Markdown table", silent = true })
  end

  if keymaps.export_tsv then
    vim.keymap.set("n", keymaps.export_tsv, try_export("tsv"),
      { desc = "Export as TSV (Excel)", silent = true })
  end

  -- Chart visualization keymaps
  local visualize = require('sql-cli.visualize')

  if keymaps.bar_chart_at_cursor then
    vim.keymap.set("n", keymaps.bar_chart_at_cursor, function()
      visualize.bar_chart_at_cursor()
    end, { desc = "Bar chart from query at cursor", silent = true })
  end

  if keymaps.pie_chart_at_cursor then
    vim.keymap.set("n", keymaps.pie_chart_at_cursor, function()
      visualize.pie_chart_at_cursor()
    end, { desc = "Pie chart from query at cursor", silent = true })
  end

  if keymaps.histogram_at_cursor then
    vim.keymap.set("n", keymaps.histogram_at_cursor, function()
      visualize.histogram_at_cursor()
    end, { desc = "Histogram from query at cursor", silent = true })
  end

  if keymaps.scatter_at_cursor then
    vim.keymap.set("n", keymaps.scatter_at_cursor, function()
      visualize.scatter_plot_at_cursor()
    end, { desc = "Scatter plot from query at cursor", silent = true })
  end

  if keymaps.sparkline_at_cursor then
    vim.keymap.set("n", keymaps.sparkline_at_cursor, function()
      visualize.sparkline_at_cursor()
    end, { desc = "Sparkline from query at cursor", silent = true })
  end

  -- Column sum/statistics
  if keymaps.sum_column then
    vim.keymap.set("n", keymaps.sum_column, function()
      local column_sum = require('sql-cli.column_sum')
      column_sum.sum_column_at_cursor()
    end, { desc = "Sum/statistics for column at cursor", silent = true })
  end

  if keymaps.show_totals then
    vim.keymap.set("n", keymaps.show_totals, function()
      local column_sum = require('sql-cli.column_sum')
      column_sum.show_all_totals()
    end, { desc = "Add totals row to table", silent = true })
  end
end

-- Setup autocommands
function M.setup_autocmds()
  local group = vim.api.nvim_create_augroup("SqlCli", { clear = true })

  -- Debug logging
  local debug = M.config.debug or false

  -- Auto-detect CSV files
  if M.config.auto_detect.csv_files then
    vim.api.nvim_create_autocmd("BufEnter", {
      group = group,
      pattern = "*.csv",
      callback = function(ev)
        if debug then
          vim.notify("[SQL-CLI Debug] CSV autocmd triggered, M.state = " .. tostring(M.state), vim.log.levels.DEBUG)
        end

        -- Check if plugin is initialized
        if not M.state then
          if debug then
            vim.notify("[SQL-CLI Debug] State not initialized, skipping CSV detection", vim.log.levels.DEBUG)
          end
          return
        end

        if not M.state:get_data_file() then
          M.state:set_data_file(ev.file)
          vim.notify("Data file set to: " .. ev.file, vim.log.levels.INFO)
          -- Load schema for completion
          functions.load_schema_for_completion(M.config, M.state)
        end
      end,
    })
  end

  -- Auto-detect data hints in SQL files
  if M.config.auto_detect.data_hints then
    vim.api.nvim_create_autocmd({"BufRead", "BufNewFile"}, {
      group = group,
      pattern = "*.sql",
      callback = function(ev)
        if debug then
          vim.notify("[SQL-CLI Debug] SQL autocmd triggered for " .. ev.file .. ", M.state = " .. tostring(M.state), vim.log.levels.DEBUG)
        end

        -- Check if plugin is initialized
        if not M.state then
          if debug then
            vim.notify("[SQL-CLI Debug] State not initialized, skipping data hint detection", vim.log.levels.DEBUG)
          end
          return
        end

        -- Check state methods are accessible
        if not M.state.get_data_file then
          if debug then
            vim.notify("[SQL-CLI Debug] State exists but get_data_file method missing!", vim.log.levels.ERROR)
            vim.notify("[SQL-CLI Debug] State type: " .. type(M.state), vim.log.levels.ERROR)
            if type(M.state) == "table" then
              local methods = {}
              for k, v in pairs(M.state) do
                table.insert(methods, k .. "=" .. type(v))
              end
              vim.notify("[SQL-CLI Debug] State contents: " .. table.concat(methods, ", "), vim.log.levels.ERROR)
            end
          end
          return
        end

        if not M.state:get_data_file() then
          local lines = vim.api.nvim_buf_get_lines(ev.buf, 0, 5, false)
          local dir = vim.fn.fnamemodify(ev.file, ":h")
          local data_file = utils.detect_data_hint(lines, dir)
          if data_file then
            M.state:set_data_file(data_file)
            vim.notify("Data file detected: " .. data_file, vim.log.levels.INFO)
            -- Load schema for completion
            functions.load_schema_for_completion(M.config, M.state)
          end
        end
      end,
    })
  end
end

-- Set data file
function M.set_data_file(file)
  if not M.state then
    vim.notify("SQL CLI plugin not initialized. Run :lua require('sql-cli').setup()", vim.log.levels.ERROR)
    return
  end

  if not file or file == "" then
    -- Use file picker
    vim.ui.input({
      prompt = "Data file path: ",
      default = M.state:get_data_file() or "",
      completion = "file"
    }, function(input)
      if input and input ~= "" then
        M.state:set_data_file(input)
        vim.notify("Data file set to: " .. input, vim.log.levels.INFO)
        -- Load schema for completion
        functions.load_schema_for_completion(M.config, M.state)
      end
    end)
  else
    M.state:set_data_file(file)
    vim.notify("Data file set to: " .. file, vim.log.levels.INFO)
    -- Load schema for completion
    functions.load_schema_for_completion(M.config, M.state)
  end
end

-- Clear data file
function M.clear_data_file()
  if not M.state then
    vim.notify("SQL CLI plugin not initialized. Run :lua require('sql-cli').setup()", vim.log.levels.ERROR)
    return
  end
  M.state:set_data_file(nil)
  vim.notify("Data file cleared", vim.log.levels.INFO)
end

-- Purge all cache entries
function M.purge_cache()
  local utils = require('sql-cli.utils')
  local command_path, err = utils.get_command_path(M.config.command)
  if not command_path then
    vim.notify("Failed to find sql-cli command: " .. (err or "unknown error"), vim.log.levels.ERROR)
    return
  end

  local cmd = string.format("%s --cache-purge", command_path)

  vim.fn.jobstart(cmd, {
    on_stdout = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            vim.notify(line, vim.log.levels.INFO)
          end
        end
      end
    end,
    on_stderr = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            vim.notify(line, vim.log.levels.ERROR)
          end
        end
      end
    end,
  })
end

-- Show query plan
function M.show_query_plan()
  if not M.state then
    vim.notify("SQL CLI plugin not initialized. Run :lua require('sql-cli').setup()", vim.log.levels.ERROR)
    return
  end
  executor.execute_at_cursor_with_plan(M.config, M.state)
end

-- Open data file
function M.open_data_file()
  if not M.state then
    vim.notify("SQL CLI plugin not initialized. Run :lua require('sql-cli').setup()", vim.log.levels.ERROR)
    return
  end
  local data_file = M.state:get_data_file()
  if data_file then
    -- Create a new split for the data file
    local split_cmd = M.config.split.direction == "vertical" and "vsplit" or "split"
    vim.cmd(split_cmd .. " " .. vim.fn.fnameescape(data_file))

    -- Optionally resize the window
    if M.config.split.direction == "vertical" then
      local size = math.floor(vim.o.columns * M.config.split.size)
      vim.api.nvim_win_set_width(0, size)
    else
      local size = math.floor(vim.o.lines * M.config.split.size)
      vim.api.nvim_win_set_height(0, size)
    end
  else
    vim.notify("No data file set", vim.log.levels.WARN)
  end
end

-- Get data file (for external use)
function M.get_data_file()
  return M.state and M.state:get_data_file() or nil
end

-- Statusline function
function M.statusline()
  if not M.state then
    return ""
  end
  return ui.statusline(M.state)
end

-- Test formatter (for debugging)
function M.test_formatter()
  if not M.config then
    vim.notify("SQL CLI plugin not initialized. Run :lua require('sql-cli').setup()", vim.log.levels.ERROR)
    return
  end
  formatter.test_formatter(M.config)
end

-- Show raw query analysis JSON
function M.analyze_query_raw()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL query found at cursor", vim.log.levels.WARN)
    return
  end

  -- Extract query
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end
  local query = table.concat(query_lines, '\n')

  -- Show raw analysis
  local cli_analyzer = require('sql-cli.cli_analyzer')
  cli_analyzer.show_raw_analysis_popup(query)
end

-- Expose logger module for use by other modules
M.logger = logger

return M