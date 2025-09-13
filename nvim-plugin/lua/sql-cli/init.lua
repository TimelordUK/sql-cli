-- SQL CLI Neovim Plugin
-- Main entry point that coordinates all modules

local M = {}

-- Load modules
local config = require('sql-cli.config')
local state = require('sql-cli.state')
local utils = require('sql-cli.utils')
local executor = require('sql-cli.executor')
local formatter = require('sql-cli.formatter')
local ui = require('sql-cli.ui')
local navigation = require('sql-cli.navigation')
local functions = require('sql-cli.functions')
local results = require('sql-cli.results')

-- Plugin configuration
M.config = {}

-- Plugin state
M.state = nil

-- Setup function
function M.setup(opts)
  -- Initialize configuration
  M.config = config.setup(opts)

  -- Initialize state
  M.state = state.new()

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

  vim.api.nvim_create_user_command("SqlCliSetDataFile", function(opts)
    M.set_data_file(opts.args)
  end, { nargs = 1, complete = "file", desc = "Set data file for SQL queries" })

  vim.api.nvim_create_user_command("SqlCliClearDataFile", function()
    M.clear_data_file()
  end, { desc = "Clear data file" })

  vim.api.nvim_create_user_command("SqlCliToggleOutput", function()
    ui.toggle_output_window(M.config, M.state)
  end, { desc = "Toggle SQL output window" })

  vim.api.nvim_create_user_command("SqlCliShowPlan", function()
    M.show_query_plan()
  end, { desc = "Show query execution plan" })

  vim.api.nvim_create_user_command("SqlCliSelectQuery", function()
    navigation.select_query_at_cursor()
  end, { desc = "Select SQL query at cursor" })

  vim.api.nvim_create_user_command("SqlCliPreviewQuery", function()
    ui.preview_query_at_cursor(M.config)
  end, { desc = "Preview SQL query at cursor in floating window" })

  vim.api.nvim_create_user_command("SqlCliFormatQuery", function()
    formatter.format_query_at_cursor(M.config, M.state)
  end, { desc = "Format SQL query at cursor" })

  vim.api.nvim_create_user_command("SqlCliListFunctions", function()
    functions.list_functions(M.config)
  end, { desc = "List all SQL functions" })

  vim.api.nvim_create_user_command("SqlCliSearchFunctions", function(opts)
    functions.search_functions(opts.args, M.config)
  end, { nargs = 1, desc = "Search SQL functions" })

  vim.api.nvim_create_user_command("SqlCliSaveResults", function(opts)
    results.save_results_csv(opts.args, M.state)
  end, { nargs = "?", complete = "file", desc = "Save query results to CSV" })

  vim.api.nvim_create_user_command("SqlCliResultsToBuffer", function()
    results.results_to_buffer(M.state)
  end, { desc = "Open query results in new buffer" })

  vim.api.nvim_create_user_command("SqlCliExpandStar", function()
    results.expand_star_columns(M.config, M.state)
  end, { desc = "Expand SELECT * to column names" })
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

  if keymaps.set_data_file then
    vim.keymap.set("n", keymaps.set_data_file, function()
      M.set_data_file(nil)
    end, { desc = "Set SQL data file", silent = true })
  end

  if keymaps.clear_data_file then
    vim.keymap.set("n", keymaps.clear_data_file, M.clear_data_file,
      { desc = "Clear SQL data file", silent = true })
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

  if keymaps.results_to_buffer then
    vim.keymap.set("n", keymaps.results_to_buffer, function()
      results.results_to_buffer(M.state)
    end, { desc = "Results to buffer", silent = true })
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

  if keymaps.search_functions then
    vim.keymap.set("n", keymaps.search_functions, function()
      vim.ui.input({ prompt = "Search functions: " }, function(query)
        if query and query ~= "" then
          functions.search_functions(query, M.config)
        end
      end)
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
      results.expand_star_columns(M.config, M.state)
    end, { desc = "Expand SELECT *", silent = true })

    vim.keymap.set("v", keymaps.expand_star, function()
      results.expand_star_visual(M.config, M.state)
    end, { desc = "Expand SELECT * in selection", silent = true })
  end

  if keymaps.copy_query then
    vim.keymap.set("n", keymaps.copy_query, navigation.copy_query_at_cursor,
      { desc = "Copy SQL query at cursor to clipboard", silent = true })
  end

  if keymaps.format_query then
    vim.keymap.set("n", keymaps.format_query, function()
      formatter.format_query_at_cursor(M.config, M.state)
    end, { desc = "Format SQL query at cursor", silent = true })
  end
end

-- Setup autocommands
function M.setup_autocmds()
  local group = vim.api.nvim_create_augroup("SqlCli", { clear = true })

  -- Auto-detect CSV files
  if M.config.auto_detect.csv_files then
    vim.api.nvim_create_autocmd("BufEnter", {
      group = group,
      pattern = "*.csv",
      callback = function(ev)
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
  M.state:set_data_file(nil)
  vim.notify("Data file cleared", vim.log.levels.INFO)
end

-- Show query plan
function M.show_query_plan()
  executor.execute_at_cursor_with_plan(M.config, M.state)
end

-- Open data file
function M.open_data_file()
  local data_file = M.state:get_data_file()
  if data_file then
    vim.cmd("edit " .. vim.fn.fnameescape(data_file))
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
  formatter.test_formatter(M.config)
end

return M