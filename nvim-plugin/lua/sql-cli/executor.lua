-- SQL CLI Query Executor Module
-- Functions for executing SQL queries and managing execution

local utils = require('sql-cli.utils')
local table_nav = require('sql-cli.table_nav')

local M = {}

-- Helper function to expand environment variables in text
local function expand_env_variables(text)
  -- Replace ${VAR_NAME} patterns with environment variable values
  local expanded = text:gsub('%${([%w_]+)}', function(var)
    local value = vim.env[var]
    if value then
      return value
    else
      -- Keep the original placeholder if no env var found
      return '${' .. var .. '}'
    end
  end)
  return expanded
end

-- Execute query from buffer or provided string
function M.execute_query(query, config, state, skip_params)
  -- Get query from buffer if not provided
  if not query or query == "" then
    local bufnr = vim.api.nvim_get_current_buf()
    local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    query = table.concat(lines, "\n")

    -- Auto-detect data file from hints
    if config.auto_detect.data_hints and not state:get_data_file() then
      -- Get the directory of the current buffer
      local buf_path = vim.api.nvim_buf_get_name(bufnr)
      local buf_dir = nil
      if buf_path and buf_path ~= "" then
        buf_dir = vim.fn.fnamemodify(buf_path, ":h")
      end
      local data_file = utils.detect_data_hint(lines, buf_dir)
      if data_file then
        state:set_data_file(data_file)
        -- Load schema for completion would need to be called from main module
        if config.load_schema_callback then
          config.load_schema_callback()
        end
      end
    end

    -- Auto-detect if current buffer is a CSV
    if config.auto_detect.csv_files and not state:get_data_file() then
      local filename = vim.api.nvim_buf_get_name(bufnr)
      if filename:match("%.csv$") then
        state:set_data_file(filename)
        -- Load schema for completion would need to be called from main module
        if config.load_schema_callback then
          config.load_schema_callback()
        end
      end
    end
  end

  -- Check for parameters and resolve them unless skipped
  if not skip_params then
    local params = require('sql-cli.params')
    local extracted = params.extract_parameters(query)

    if #extracted > 0 then
      -- Query has parameters, resolve them
      params.resolve_parameters(query, function(resolved_query)
        if resolved_query then
          -- Trim the resolved query
          resolved_query = resolved_query:gsub("^%s+", ""):gsub("%s+$", "")

          -- Show confirmation dialog with resolved query
          local lines = vim.split(resolved_query, '\n')

          -- Create a preview buffer
          local buf = vim.api.nvim_create_buf(false, true)
          vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
          vim.api.nvim_buf_set_option(buf, 'filetype', 'sql')

          -- Calculate window size
          local width = math.min(80, math.max(40, vim.o.columns - 20))
          local height = math.min(30, math.max(10, #lines + 4))

          -- Create floating window
          local win = vim.api.nvim_open_win(buf, true, {
            relative = 'editor',
            width = width,
            height = height,
            col = math.floor((vim.o.columns - width) / 2),
            row = math.floor((vim.o.lines - height) / 2),
            style = 'minimal',
            border = 'rounded',
            title = ' Resolved Query - Press ENTER to execute, ESC to cancel ',
            title_pos = 'center'
          })

          -- Set up keymaps for the preview window
          local function close_and_execute()
            vim.api.nvim_win_close(win, true)
            -- Expand environment variables (for tokens like ${JWT_TOKEN})
            resolved_query = expand_env_variables(resolved_query)
            -- Save the resolved query
            state:set_last_query(resolved_query)
            -- Add to history for recall
            require('sql-cli.query_history').add_to_history(resolved_query)
            -- Execute with parameters resolved
            M.run_command(resolved_query, false, config, state)
          end

          local function close_and_cancel()
            vim.api.nvim_win_close(win, true)
            vim.notify("Query execution cancelled", vim.log.levels.INFO)
          end

          -- Key mappings for the floating window
          vim.api.nvim_buf_set_keymap(buf, 'n', '<CR>', '', {
            callback = close_and_execute,
            noremap = true,
            silent = true
          })

          vim.api.nvim_buf_set_keymap(buf, 'n', '<Esc>', '', {
            callback = close_and_cancel,
            noremap = true,
            silent = true
          })

          vim.api.nvim_buf_set_keymap(buf, 'n', 'q', '', {
            callback = close_and_cancel,
            noremap = true,
            silent = true
          })

          -- Add yank functionality
          vim.api.nvim_buf_set_keymap(buf, 'n', 'y', '', {
            callback = function()
              vim.fn.setreg('+', resolved_query)
              vim.fn.setreg('"', resolved_query)
              vim.notify("Query yanked to clipboard", vim.log.levels.INFO)
            end,
            noremap = true,
            silent = true
          })

          -- Add a status line at the top
          vim.api.nvim_buf_set_lines(buf, 0, 0, false, {
            "-- Parameters resolved successfully",
            "-- Press ENTER to execute, ESC/q to cancel, y to yank",
            "-- " .. string.rep("-", width - 6),
            ""
          })

          -- Now make it read-only
          vim.api.nvim_buf_set_option(buf, 'modifiable', false)
        else
          vim.notify("Parameter resolution cancelled", vim.log.levels.INFO)
        end
      end)
      return
    end
  end

  -- Expand environment variables (for tokens like ${JWT_TOKEN})
  query = expand_env_variables(query)

  -- Save the query
  state:set_last_query(query)

  -- Add to history (for queries without parameters)
  require('sql-cli.query_history').add_to_history(query)

  -- Execute
  M.run_command(query, false, config, state)
end

-- Execute query with execution plan
function M.execute_query_with_plan(query, config, state, skip_params)
  -- Get query from buffer if not provided
  if not query or query == "" then
    local bufnr = vim.api.nvim_get_current_buf()
    local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    query = table.concat(lines, "\n")

    -- Auto-detect data file from hints
    if config.auto_detect.data_hints and not state:get_data_file() then
      -- Get the directory of the current buffer
      local buf_path = vim.api.nvim_buf_get_name(bufnr)
      local buf_dir = nil
      if buf_path and buf_path ~= "" then
        buf_dir = vim.fn.fnamemodify(buf_path, ":h")
      end
      local data_file = utils.detect_data_hint(lines, buf_dir)
      if data_file then
        state:set_data_file(data_file)
        -- Load schema for completion would need to be called from main module
        if config.load_schema_callback then
          config.load_schema_callback()
        end
      end
    end

    -- Auto-detect if current buffer is a CSV
    if config.auto_detect.csv_files and not state:get_data_file() then
      local filename = vim.api.nvim_buf_get_name(bufnr)
      if filename:match("%.csv$") then
        state:set_data_file(filename)
        -- Load schema for completion would need to be called from main module
        if config.load_schema_callback then
          config.load_schema_callback()
        end
      end
    end
  end

  -- Check for parameters and resolve them unless skipped
  if not skip_params then
    local params = require('sql-cli.params')
    local extracted = params.extract_parameters(query)

    if #extracted > 0 then
      -- Query has parameters, resolve them
      params.resolve_parameters(query, function(resolved_query)
        if resolved_query then
          -- Trim the resolved query
          resolved_query = resolved_query:gsub("^%s+", ""):gsub("%s+$", "")

          -- Show same preview dialog as execute_query
          local lines = vim.split(resolved_query, '\n')
          local buf = vim.api.nvim_create_buf(false, true)
          vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
          vim.api.nvim_buf_set_option(buf, 'filetype', 'sql')

          local width = math.min(80, math.max(40, vim.o.columns - 20))
          local height = math.min(30, math.max(10, #lines + 4))

          local win = vim.api.nvim_open_win(buf, true, {
            relative = 'editor',
            width = width,
            height = height,
            col = math.floor((vim.o.columns - width) / 2),
            row = math.floor((vim.o.lines - height) / 2),
            style = 'minimal',
            border = 'rounded',
            title = ' Resolved Query - Press ENTER to execute, ESC to cancel ',
            title_pos = 'center'
          })

          local function close_and_execute()
            vim.api.nvim_win_close(win, true)
            -- Expand environment variables (for tokens like ${JWT_TOKEN})
            resolved_query = expand_env_variables(resolved_query)
            state:set_last_query(resolved_query)
            -- Add to history for recall
            require('sql-cli.query_history').add_to_history(resolved_query)
            M.run_command(resolved_query, true, config, state)
          end

          local function close_and_cancel()
            vim.api.nvim_win_close(win, true)
            vim.notify("Query execution cancelled", vim.log.levels.INFO)
          end

          vim.api.nvim_buf_set_keymap(buf, 'n', '<CR>', '', {
            callback = close_and_execute,
            noremap = true,
            silent = true
          })

          vim.api.nvim_buf_set_keymap(buf, 'n', '<Esc>', '', {
            callback = close_and_cancel,
            noremap = true,
            silent = true
          })

          vim.api.nvim_buf_set_keymap(buf, 'n', 'q', '', {
            callback = close_and_cancel,
            noremap = true,
            silent = true
          })

          -- Add yank functionality
          vim.api.nvim_buf_set_keymap(buf, 'n', 'y', '', {
            callback = function()
              vim.fn.setreg('+', resolved_query)
              vim.fn.setreg('"', resolved_query)
              vim.notify("Query yanked to clipboard", vim.log.levels.INFO)
            end,
            noremap = true,
            silent = true
          })

          vim.api.nvim_buf_set_lines(buf, 0, 0, false, {
            "-- Parameters resolved successfully",
            "-- Press ENTER to execute with plan, ESC/q to cancel, y to yank",
            "-- " .. string.rep("-", width - 6),
            ""
          })

          -- Now make it read-only
          vim.api.nvim_buf_set_option(buf, 'modifiable', false)
        else
          vim.notify("Parameter resolution cancelled", vim.log.levels.INFO)
        end
      end)
      return
    end
  end

  -- Expand environment variables (for tokens like ${JWT_TOKEN})
  query = expand_env_variables(query)

  -- Save the query
  state:set_last_query(query)

  -- Execute with execution plan
  M.run_command(query, true, config, state)
end

-- Execute visual selection
function M.execute_selection(config, state)
  -- Save current window and cursor position to restore later
  local original_win = vim.api.nvim_get_current_win()
  local original_cursor = vim.api.nvim_win_get_cursor(original_win)

  -- Get the current visual selection properly
  -- Save the current register content
  local save_reg = vim.fn.getreg('"')
  local save_regtype = vim.fn.getregtype('"')

  -- Yank the current visual selection
  vim.cmd('normal! y')
  local query = vim.fn.getreg('"')

  -- Restore the register
  vim.fn.setreg('"', save_reg, save_regtype)

  if not query or query == "" then
    vim.notify("No selection", vim.log.levels.WARN)
    return
  end

  -- Store the original window and cursor to restore after execution
  state.original_win = original_win
  state.original_cursor = original_cursor

  M.execute_query(query, config, state)
end

-- Execute query at cursor
function M.execute_at_cursor(config, state)
  -- Save current window and cursor position to restore later
  local original_win = vim.api.nvim_get_current_win()
  local original_cursor = vim.api.nvim_win_get_cursor(original_win)

  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Check if this is a script with GO separators
  local script_text = table.concat(lines, "\n")
  local has_go_separator = script_text:match("%sGO%s") or script_text:match("^GO%s") or script_text:match("%sGO$") or script_text:match("^GO$")

  -- If we have GO separators, delegate to execute_statement_at_cursor for proper dependency handling
  if has_go_separator then
    vim.notify("Script with GO separators detected - using dependency-aware execution", vim.log.levels.INFO)
    M.execute_statement_at_cursor(config, state)
    return
  end

  -- Find query boundaries
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end

  -- Optionally highlight the query briefly before execution
  if vim.g.sql_cli_highlight_before_execute then
    -- Save current position
    local save_cursor = vim.fn.getpos('.')

    -- Highlight the query
    vim.cmd('normal! ' .. start_line .. 'G')
    vim.cmd('normal! V')
    vim.cmd('normal! ' .. end_line .. 'G')
    vim.cmd('redraw')

    -- Brief pause to show selection
    vim.cmd('sleep 200m')

    -- Exit visual mode and restore cursor
    vim.cmd('normal! <Esc>')
    vim.fn.setpos('.', save_cursor)
  end

  -- Extract the query, skipping comments and terminators
  local query_lines = {}
  for i = start_line, end_line do
    local line = lines[i]
    -- Skip GO terminators and comment lines
    if not line:match("^%s*GO%s*$") and not line:match("^%s*%-%-") then
      table.insert(query_lines, line)
    end
  end

  local query = table.concat(query_lines, "\n")

  -- Auto-detect data file if needed
  if config.auto_detect.data_hints and not state:get_data_file() then
    local buf_path = vim.api.nvim_buf_get_name(bufnr)
    local buf_dir = nil
    if buf_path and buf_path ~= "" then
      buf_dir = vim.fn.fnamemodify(buf_path, ":h")
    end
    local data_file = utils.detect_data_hint(lines, buf_dir)
    if data_file then
      state:set_data_file(data_file)
      if config.load_schema_callback then
        config.load_schema_callback()
      end
    end
  end

  -- Auto-detect if current buffer is a CSV
  if config.auto_detect.csv_files and not state:get_data_file() then
    local filename = vim.api.nvim_buf_get_name(bufnr)
    if filename:match("%.csv$") then
      state:set_data_file(filename)
      if config.load_schema_callback then
        config.load_schema_callback()
      end
    end
  end

  -- Store the original window and cursor to restore after execution
  state.original_win = original_win
  state.original_cursor = original_cursor

  -- Skip parameter resolution for quick execution at cursor
  M.execute_query(query, config, state, true)
end

-- Execute statement at cursor with dependency awareness (\sx command)
-- Uses --execute-statement flag to run only the target statement and its dependencies
function M.execute_statement_at_cursor(config, state)
  -- Save current window and cursor position to restore later
  local original_win = vim.api.nvim_get_current_win()
  local original_cursor = vim.api.nvim_win_get_cursor(original_win)

  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Check if this is a script with GO separators
  local script_text = table.concat(lines, "\n")
  local has_go_separator = script_text:match("%sGO%s") or script_text:match("^GO%s") or script_text:match("%sGO$") or script_text:match("^GO$")

  if not has_go_separator then
    -- Not a script, this shouldn't happen since execute_at_cursor delegates to us
    -- But handle it gracefully anyway
    vim.notify("No GO separators found - executing as single query", vim.log.levels.INFO)
    -- Fall back to normal query execution (avoiding circular call)
    local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)
    if start_line then
      local query_lines = {}
      for i = start_line, end_line do
        table.insert(query_lines, lines[i])
      end
      local query = table.concat(query_lines, "\n")
      state.original_win = original_win
      state.original_cursor = original_cursor
      M.execute_query(query, config, state, true)
    else
      vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    end
    return
  end

  -- Count statement number at cursor by finding which statement block contains the cursor
  -- Statement blocks are separated by GO terminators
  -- Statement 1 starts at line 1 and ends at the first GO
  -- Statement 2 starts after the first GO and ends at the second GO, etc.
  local statement_num = 1
  local current_stmt_start = 1

  for i = 1, #lines do
    if lines[i]:match("^%s*GO%s*$") then
      -- Found a GO terminator
      if i < cursor_line then
        -- GO is before cursor, so cursor is in next statement
        statement_num = statement_num + 1
        current_stmt_start = i + 1
      else
        -- GO is at or after cursor, so cursor is in current statement
        break
      end
    end
  end

  -- Auto-detect data file if needed
  if config.auto_detect.data_hints and not state:get_data_file() then
    local buf_path = vim.api.nvim_buf_get_name(bufnr)
    local buf_dir = nil
    if buf_path and buf_path ~= "" then
      buf_dir = vim.fn.fnamemodify(buf_path, ":h")
    end
    local data_file = utils.detect_data_hint(lines, buf_dir)
    if data_file then
      state:set_data_file(data_file)
      if config.load_schema_callback then
        config.load_schema_callback()
      end
    end
  end

  -- Auto-detect if current buffer is a CSV
  if config.auto_detect.csv_files and not state:get_data_file() then
    local filename = vim.api.nvim_buf_get_name(bufnr)
    if filename:match("%.csv$") then
      state:set_data_file(filename)
      if config.load_schema_callback then
        config.load_schema_callback()
      end
    end
  end

  -- Store the original window and cursor to restore after execution
  state.original_win = original_win
  state.original_cursor = original_cursor

  -- Notify user which statement we're executing
  vim.notify(string.format("Executing statement #%d with dependencies...", statement_num), vim.log.levels.INFO)

  -- Execute using --execute-statement flag
  M.run_statement_with_dependencies(script_text, statement_num, config, state)
end

-- Run a specific statement from a script with dependency analysis
function M.run_statement_with_dependencies(script, statement_num, config, state)
  local ui_callbacks = config.ui_callbacks or {}

  -- Create output window if needed
  if ui_callbacks.is_output_window_valid and not ui_callbacks.is_output_window_valid() then
    if ui_callbacks.create_output_window then
      ui_callbacks.create_output_window()
    else
      vim.notify("No output window callback available", vim.log.levels.ERROR)
      return
    end
  end

  -- Clear output if configured
  local output_buf = state:get_output_buf()
  if config.output.clear_on_run and output_buf then
    -- Clear table navigation state if active on this buffer
    local table_nav = require('sql-cli.table_nav')
    if table_nav.is_active() then
      table_nav.clear_navigation()
    end

    -- Make buffer modifiable to clear it
    vim.bo[output_buf].modifiable = true
    vim.bo[output_buf].readonly = false
    vim.api.nvim_buf_set_lines(output_buf, 0, -1, false, {})
  end

  -- Save script to temp file
  local temp_file = vim.fn.tempname() .. ".sql"
  local file = io.open(temp_file, "w")
  if not file then
    vim.notify("Failed to create temporary file", vim.log.levels.ERROR)
    return
  end
  file:write(script)
  file:close()

  -- Build command with --execute-statement flag
  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    vim.notify(err, vim.log.levels.ERROR)
    return
  end

  local cmd_parts = { command_path }

  -- Add data file if set
  local data_file = state:get_data_file()
  if data_file then
    table.insert(cmd_parts, vim.fn.shellescape(data_file))
  end

  -- Add script file
  table.insert(cmd_parts, "-f")
  table.insert(cmd_parts, vim.fn.shellescape(temp_file))

  -- Add --execute-statement flag
  table.insert(cmd_parts, "--execute-statement")
  table.insert(cmd_parts, tostring(statement_num))

  -- Add output format
  table.insert(cmd_parts, "-o")
  table.insert(cmd_parts, config.output_format)

  -- Add table output settings if using table format
  if config.output_format == "table" and config.table_output then
    if config.table_output.style then
      table.insert(cmd_parts, "--table-style")
      table.insert(cmd_parts, config.table_output.style)
    end
    if config.table_output.max_col_width then
      table.insert(cmd_parts, "--max-col-width")
      table.insert(cmd_parts, tostring(config.table_output.max_col_width))
    end
    if config.table_output.col_sample_rows then
      table.insert(cmd_parts, "--col-sample-rows")
      table.insert(cmd_parts, tostring(config.table_output.col_sample_rows))
    end
    if config.table_output.auto_hide_empty then
      table.insert(cmd_parts, "--auto-hide-empty")
    end
  end

  local cmd = table.concat(cmd_parts, " ")
  if vim.g.sql_cli_debug then
    vim.notify(string.format("Executing command: %s", cmd:sub(1, 500)), vim.log.levels.INFO)
  end

  -- Add header
  local header = {
    "-- SQL CLI Output (Statement #" .. statement_num .. " with dependencies) --",
    "-- Command: " .. cmd,
    "-- " .. os.date("%Y-%m-%d %H:%M:%S"),
    string.rep("-", 60),
    "",
  }
  if output_buf then
    vim.bo[output_buf].modifiable = true
    vim.bo[output_buf].readonly = false
    vim.api.nvim_buf_set_lines(output_buf, 0, 0, false, header)
  end

  -- Run command asynchronously
  local output_lines = {}
  local csv_lines = {}

  local job_id = vim.fn.jobstart(cmd, {
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            table.insert(output_lines, line)
            -- Try to detect CSV format
            if line:match("^[^|]*,[^|]*") or line:match("^%d+,") or line:match('^"[^"]*",') then
              table.insert(csv_lines, line)
            end
          end
        end
      end
    end,
    on_stderr = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            if line:match("^#") or line:match("Executing statement") or line:match("completed") then
              -- Informational output
              table.insert(output_lines, line)
            elseif line:match("^Error:") or line:match("^ERROR:") then
              table.insert(output_lines, "ERROR: " .. line)
            else
              table.insert(output_lines, line)
            end
          end
        end
      end
    end,
    on_exit = function(_, exit_code)
      vim.schedule(function()
        -- Append output
        if output_buf then
          vim.bo[output_buf].modifiable = true
          vim.bo[output_buf].readonly = false
          vim.api.nvim_buf_set_lines(output_buf, -1, -1, false, output_lines)
        end

        -- Store results
        if #csv_lines > 0 then
          state:set_last_results(csv_lines)
        else
          state:set_last_results(output_lines)
        end

        -- Add footer
        local footer = {
          "",
          string.rep("-", 60),
          "-- Exit code: " .. exit_code,
        }
        if output_buf then
          vim.bo[output_buf].modifiable = true
          vim.bo[output_buf].readonly = false
          vim.api.nvim_buf_set_lines(output_buf, -1, -1, false, footer)
        end

        -- Scroll to top
        local output_win = state:get_output_win()
        if output_win and vim.api.nvim_win_is_valid(output_win) then
          vim.api.nvim_win_set_cursor(output_win, {1, 0})
          vim.api.nvim_win_call(output_win, function()
            vim.cmd('normal! zt')
          end)
        end

        -- Focus output window if configured
        if config.output.focus_on_run and ui_callbacks.is_output_window_valid and ui_callbacks.is_output_window_valid() and output_win then
          vim.api.nvim_set_current_win(output_win)

          -- Restore original window after a delay
          if state.original_win and vim.api.nvim_win_is_valid(state.original_win) then
            vim.defer_fn(function()
              vim.api.nvim_set_current_win(state.original_win)
              if state.original_cursor then
                vim.api.nvim_win_set_cursor(state.original_win, state.original_cursor)
              end
              state.original_win = nil
              state.original_cursor = nil
            end, 100)
          end
        end

        -- Notify completion
        if exit_code == 0 then
          vim.notify(string.format("Statement #%d executed successfully", statement_num), vim.log.levels.INFO)

          -- Enable table navigation
          if output_buf and vim.api.nvim_buf_is_valid(output_buf) then
            vim.defer_fn(function()
              if not vim.api.nvim_buf_is_valid(output_buf) then
                return
              end

              local table_nav = require('sql-cli.table_nav')
              if config.table_navigation and config.table_navigation.enabled_by_default then
                local lines = vim.api.nvim_buf_get_lines(output_buf, 0, -1, false)
                if #lines > 5 then
                  if table_nav.is_active() then
                    table_nav.clear_navigation()
                  end

                  local nav_output_win = state:get_output_win()
                  if table_nav.init_navigation(output_buf, nav_output_win) then
                    table_nav.setup_keymaps(output_buf, config)
                    if nav_output_win and vim.api.nvim_win_is_valid(nav_output_win) then
                      vim.api.nvim_win_set_cursor(nav_output_win, {1, 0})
                    end
                    local nav_keys = config.table_navigation and config.table_navigation.hijack_hjkl == false and "arrow keys" or "h/j/k/l"
                    vim.notify("Table navigation enabled (" .. nav_keys .. " to move)", vim.log.levels.INFO)
                  end
                end
              else
                vim.notify("Use <leader>sn to enable table navigation", vim.log.levels.INFO)
              end
            end, 500)
          end
        else
          vim.notify(string.format("Statement #%d failed with exit code: %d", statement_num, exit_code), vim.log.levels.ERROR)
        end
      end)
    end,
  })

  if job_id <= 0 then
    vim.notify("Failed to start SQL CLI", vim.log.levels.ERROR)
  end
end

-- Execute query at cursor with execution plan
function M.execute_at_cursor_with_plan(config, state)
  -- Save current window and cursor position to restore later
  local original_win = vim.api.nvim_get_current_win()
  local original_cursor = vim.api.nvim_win_get_cursor(original_win)

  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end

  -- Optionally highlight the query briefly before execution
  if vim.g.sql_cli_highlight_before_execute then
    -- Save current position
    local save_cursor = vim.fn.getpos('.')

    -- Highlight the query
    vim.cmd('normal! ' .. start_line .. 'G')
    vim.cmd('normal! V')
    vim.cmd('normal! ' .. end_line .. 'G')
    vim.cmd('redraw')

    -- Brief pause to show selection
    vim.cmd('sleep 200m')

    -- Exit visual mode and restore cursor
    vim.cmd('normal! <Esc>')
    vim.fn.setpos('.', save_cursor)
  end

  -- Extract the query, skipping comments and terminators
  local query_lines = {}
  for i = start_line, end_line do
    local line = lines[i]
    -- Skip GO terminators and comment lines
    if not line:match("^%s*GO%s*$") and not line:match("^%s*%-%-") then
      table.insert(query_lines, line)
    end
  end

  local query = table.concat(query_lines, "\n")

  -- Auto-detect data file if needed
  if config.auto_detect.data_hints and not state:get_data_file() then
    local buf_path = vim.api.nvim_buf_get_name(bufnr)
    local buf_dir = nil
    if buf_path and buf_path ~= "" then
      buf_dir = vim.fn.fnamemodify(buf_path, ":h")
    end
    local data_file = utils.detect_data_hint(lines, buf_dir)
    if data_file then
      state:set_data_file(data_file)
      if config.load_schema_callback then
        config.load_schema_callback()
      end
    end
  end

  -- Auto-detect if current buffer is a CSV
  if config.auto_detect.csv_files and not state:get_data_file() then
    local filename = vim.api.nvim_buf_get_name(bufnr)
    if filename:match("%.csv$") then
      state:set_data_file(filename)
      if config.load_schema_callback then
        config.load_schema_callback()
      end
    end
  end

  -- Store the original window and cursor to restore after execution
  state.original_win = original_win
  state.original_cursor = original_cursor

  M.execute_query_with_plan(query, config, state)
end

-- Build command line for SQL CLI execution
function M.build_command(query, show_plan, config, state)
  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    vim.notify(err, vim.log.levels.ERROR)
    return nil
  end

  local cmd_parts = { command_path }

  -- Add data file if set
  local data_file = state:get_data_file()
  if data_file then
    table.insert(cmd_parts, vim.fn.shellescape(data_file))
  end

  -- Check if it's a script (has GO separator) or multi-line query
  local is_script = query:match("%sGO%s") or query:match("^GO%s") or query:match("%sGO$")
  local is_multiline = query:find("\n") ~= nil

  if is_script or is_multiline then
    -- Save to temp file for script execution or multi-line queries
    local temp_file = vim.fn.tempname() .. ".sql"
    local file = io.open(temp_file, "w")
    if not file then
      vim.notify("Failed to create temporary file", vim.log.levels.ERROR)
      return nil
    end
    file:write(query)
    file:close()

    table.insert(cmd_parts, "-f")
    table.insert(cmd_parts, vim.fn.shellescape(temp_file))
  else
    -- Direct single-line query
    table.insert(cmd_parts, "-q")
    table.insert(cmd_parts, vim.fn.shellescape(query))
  end

  -- Add output format
  table.insert(cmd_parts, "-o")
  table.insert(cmd_parts, config.output_format)

  -- Add table output settings if using table format
  if config.output_format == "table" and config.table_output then
    -- Add table style
    if config.table_output.style then
      table.insert(cmd_parts, "--table-style")
      table.insert(cmd_parts, config.table_output.style)
    end

    -- Add max column width
    if config.table_output.max_col_width then
      table.insert(cmd_parts, "--max-col-width")
      table.insert(cmd_parts, tostring(config.table_output.max_col_width))
    end

    -- Add column sample rows
    if config.table_output.col_sample_rows then
      table.insert(cmd_parts, "--col-sample-rows")
      table.insert(cmd_parts, tostring(config.table_output.col_sample_rows))
    end

    -- Add auto-hide empty columns
    if config.table_output.auto_hide_empty then
      table.insert(cmd_parts, "--auto-hide-empty")
    end
  end

  -- Add query plan flag if requested
  if show_plan then
    table.insert(cmd_parts, "--execution-plan")
  end

  local cmd = table.concat(cmd_parts, " ")
  -- Only show command in debug mode (won't trigger "Press ENTER" prompt)
  if vim.g.sql_cli_debug then
    vim.notify(string.format("Executing command: %s", cmd:sub(1, 500)), vim.log.levels.INFO)
  end
  return cmd
end

-- Run SQL CLI command
function M.run_command(query, show_plan, config, state)
  local ui_callbacks = config.ui_callbacks or {}

  -- Create output window if needed
  if ui_callbacks.is_output_window_valid and not ui_callbacks.is_output_window_valid() then
    if ui_callbacks.create_output_window then
      ui_callbacks.create_output_window()
    else
      vim.notify("No output window callback available", vim.log.levels.ERROR)
      return
    end
  end

  -- Clear output if configured
  local output_buf = state:get_output_buf()
  if config.output.clear_on_run and output_buf then
    -- Clear table navigation state if active on this buffer
    local table_nav = require('sql-cli.table_nav')
    if table_nav.is_active() then
      table_nav.clear_navigation()
    end

    -- Make buffer modifiable to clear it (table_nav may have made it readonly)
    vim.bo[output_buf].modifiable = true
    vim.bo[output_buf].readonly = false
    vim.api.nvim_buf_set_lines(output_buf, 0, -1, false, {})
  end

  -- Build command
  local cmd = M.build_command(query, show_plan, config, state)
  if not cmd then
    return
  end

  -- Add header
  local header = {
    "-- SQL CLI Output --",
    "-- Command: " .. cmd,
    "-- " .. os.date("%Y-%m-%d %H:%M:%S"),
    string.rep("-", 60),
    "",
  }
  if output_buf then
    -- Ensure buffer is modifiable before writing
    vim.bo[output_buf].modifiable = true
    vim.bo[output_buf].readonly = false
    vim.api.nvim_buf_set_lines(output_buf, 0, 0, false, header)
  end

  -- Run command asynchronously
  local output_lines = {}
  local cache_messages = {}  -- Collect cache messages separately
  local csv_lines = {}  -- Store CSV format results

  -- Prepare environment with cache settings if configured
  local env = nil
  if config.cache and config.cache.enabled then
    env = vim.fn.environ()
    env["SQL_CLI_CACHE"] = "true"
    if config.cache.redis_url then
      env["SQL_CLI_REDIS_URL"] = config.cache.redis_url
    end
  end

  local job_id = vim.fn.jobstart(cmd, {
    stdout_buffered = true,
    stderr_buffered = true,
    env = env,  -- Pass environment if cache is enabled
    on_stdout = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            table.insert(output_lines, line)

            -- Try to detect and store CSV-formatted data
            if line:match("^[^|]*,[^|]*") or line:match("^%d+,") or line:match('^"[^"]*",') then
              table.insert(csv_lines, line)
            elseif line:match("^│") or line:match("^|") then
              -- Convert table format to CSV (basic conversion)
              local csv_line = line:gsub("^[│|]%s*", ""):gsub("%s*[│|]%s*", ","):gsub("%s*[│|]$", "")
              if not csv_line:match("^[─┬┴┼├┤┌┐└┘]+") then
                table.insert(csv_lines, csv_line)
              end
            end
          end
        end
      end
    end,
    on_stderr = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            -- Check for cache status messages
            if line:match("Cache HIT") then
              -- Highlight cache hits
              table.insert(cache_messages, "✅ " .. line)
              -- Also notify for immediate visibility
              vim.schedule(function()
                vim.notify("🚀 " .. line, vim.log.levels.INFO)
              end)
            elseif line:match("Cache MISS") then
              -- Highlight cache misses
              table.insert(cache_messages, "⚠️  " .. line)
              vim.schedule(function()
                vim.notify("🔍 " .. line, vim.log.levels.INFO)
              end)
            elseif line:match("Cached .* for %d+ seconds") then
              -- Show when data is cached
              table.insert(cache_messages, "💾 " .. line)
              vim.schedule(function()
                vim.notify("💾 " .. line, vim.log.levels.INFO)
              end)
            -- Check if it's actually an error or just info
            elseif line:match("^#") or line:match("Query completed:") or line:match("rows in") then
              -- This is informational output (like "# Query completed: 10 rows in 906.909µs")
              table.insert(output_lines, line)
            elseif line:match("^Error:") or line:match("^ERROR:") or line:match("failed") then
              -- This is an actual error
              table.insert(output_lines, "ERROR: " .. line)
            else
              -- Default to showing as-is (could be warnings or other info)
              table.insert(output_lines, line)
            end
          end
        end
      end
    end,
    on_exit = function(_, exit_code)
      vim.schedule(function()
        -- Combine cache messages and output, with cache messages at the top
        local final_output = {}

        -- Add cache messages first (if any)
        if #cache_messages > 0 then
          vim.list_extend(final_output, cache_messages)
          table.insert(final_output, "")  -- Add blank line separator
        end

        -- Add the actual query output
        vim.list_extend(final_output, output_lines)

        -- Append combined output
        if output_buf then
          -- Ensure buffer is modifiable before writing
          vim.bo[output_buf].modifiable = true
          vim.bo[output_buf].readonly = false
          vim.api.nvim_buf_set_lines(output_buf, -1, -1, false, final_output)
        end

        -- Store results for saving (prefer CSV format if available)
        if #csv_lines > 0 then
          state:set_last_results(csv_lines)
        else
          state:set_last_results(output_lines)
        end

        -- Add footer
        local footer = {
          "",
          string.rep("-", 60),
          "-- Exit code: " .. exit_code,
        }
        if output_buf then
          -- Ensure buffer is modifiable before writing footer
          vim.bo[output_buf].modifiable = true
          vim.bo[output_buf].readonly = false
          vim.api.nvim_buf_set_lines(output_buf, -1, -1, false, footer)
        end

        -- Always scroll output window to top to show new results
        local output_win = state:get_output_win()
        if output_win and vim.api.nvim_win_is_valid(output_win) then
          -- Move cursor to the top to show results from beginning
          vim.api.nvim_win_set_cursor(output_win, {1, 0})
          -- Ensure we're scrolled to the top
          vim.api.nvim_win_call(output_win, function()
            vim.cmd('normal! zt')
          end)
        end

        -- Focus output window if configured
        if config.output.focus_on_run and ui_callbacks.is_output_window_valid and ui_callbacks.is_output_window_valid() and output_win then
          vim.api.nvim_set_current_win(output_win)

          -- Restore original window and cursor position if saved
          if state.original_win and vim.api.nvim_win_is_valid(state.original_win) then
            vim.defer_fn(function()
              vim.api.nvim_set_current_win(state.original_win)
              if state.original_cursor then
                vim.api.nvim_win_set_cursor(state.original_win, state.original_cursor)
              end
              -- Clear the saved position
              state.original_win = nil
              state.original_cursor = nil
            end, 100) -- Small delay to ensure output window is properly set up first
          end
        end

        -- Notify completion
        if exit_code == 0 then
          vim.notify("Query executed successfully", vim.log.levels.INFO)

          -- Enable table navigation for results
          if output_buf and vim.api.nvim_buf_is_valid(output_buf) then
            vim.defer_fn(function()
              -- Double-check buffer still exists and has content
              if not vim.api.nvim_buf_is_valid(output_buf) then
                return
              end

              -- Only enable table navigation if configured to do so
              if config.table_navigation and config.table_navigation.enabled_by_default then
                local lines = vim.api.nvim_buf_get_lines(output_buf, 0, -1, false)
                if #lines > 5 then -- Make sure we have some content
                  -- Clear any existing navigation state before re-initializing
                  if table_nav.is_active() then
                    table_nav.clear_navigation()
                  end

                  -- Pass the output window to init_navigation
                  local nav_output_win = state:get_output_win()
                  if table_nav.init_navigation(output_buf, nav_output_win) then
                    table_nav.setup_keymaps(output_buf, config)

                    -- Scroll to top to show table header
                    if nav_output_win and vim.api.nvim_win_is_valid(nav_output_win) then
                      vim.api.nvim_win_set_cursor(nav_output_win, {1, 0})
                    end

                    local nav_keys = config.table_navigation and config.table_navigation.hijack_hjkl == false and "arrow keys" or "h/j/k/l"
                    vim.notify("Table navigation enabled (" .. nav_keys .. " to move, yy to yank, <leader>sn to toggle)", vim.log.levels.INFO)
                  else
                    -- Debug: show what we're seeing
                    if config.debug then
                      vim.notify("Table nav failed. First 10 lines:", vim.log.levels.WARN)
                      for i = 1, math.min(10, #lines) do
                        vim.notify("  Line " .. i .. ": " .. lines[i]:sub(1, 50), vim.log.levels.WARN)
                      end
                    end
                  end
                end
              else
                -- Table navigation disabled by default, remind user how to enable
                vim.notify("Use <leader>sn to enable table navigation", vim.log.levels.INFO)
              end
            end, 500) -- Increased delay to ensure buffer is fully populated
          end
        else
          vim.notify("Query failed with exit code: " .. exit_code, vim.log.levels.ERROR)
        end
      end)
    end,
  })

  if job_id <= 0 then
    vim.notify("Failed to start SQL CLI", vim.log.levels.ERROR)
  end
end

-- Execute query from history
function M.execute_from_history(config, state)
  local history = require('sql-cli.query_history')

  history.show_history_picker(function(query)
    if query then
      -- Trim the query
      query = query:gsub("^%s+", ""):gsub("%s+$", "")

      -- Show preview dialog before executing
      local lines = vim.split(query, '\n')

      -- Create a preview buffer
      local buf = vim.api.nvim_create_buf(false, true)
      vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
      vim.api.nvim_buf_set_option(buf, 'filetype', 'sql')

      -- Calculate window size
      local width = math.min(80, math.max(40, vim.o.columns - 20))
      local height = math.min(30, math.max(10, #lines + 4))

      -- Create floating window
      local win = vim.api.nvim_open_win(buf, true, {
        relative = 'editor',
        width = width,
        height = height,
        col = math.floor((vim.o.columns - width) / 2),
        row = math.floor((vim.o.lines - height) / 2),
        style = 'minimal',
        border = 'rounded',
        title = ' Query from History - Press ENTER to execute, ESC to cancel ',
        title_pos = 'center'
      })

      -- Set up keymaps
      local function close_and_execute()
        vim.api.nvim_win_close(win, true)
        state:set_last_query(query)
        -- Move to front of history
        history.add_to_history(query)
        M.run_command(query, false, config, state)
      end

      local function close_and_cancel()
        vim.api.nvim_win_close(win, true)
        vim.notify("Query execution cancelled", vim.log.levels.INFO)
      end

      vim.api.nvim_buf_set_keymap(buf, 'n', '<CR>', '', {
        callback = close_and_execute,
        noremap = true,
        silent = true
      })

      vim.api.nvim_buf_set_keymap(buf, 'n', '<Esc>', '', {
        callback = close_and_cancel,
        noremap = true,
        silent = true
      })

      vim.api.nvim_buf_set_keymap(buf, 'n', 'q', '', {
        callback = close_and_cancel,
        noremap = true,
        silent = true
      })

      -- Add yank functionality
      vim.api.nvim_buf_set_keymap(buf, 'n', 'y', '', {
        callback = function()
          vim.fn.setreg('+', query)
          vim.fn.setreg('"', query)
          vim.notify("Query yanked to clipboard", vim.log.levels.INFO)
        end,
        noremap = true,
        silent = true
      })

      -- Add status line at the top
      vim.api.nvim_buf_set_lines(buf, 0, 0, false, {
        "-- Query from history",
        "-- Press ENTER to execute, ESC/q to cancel, y to yank",
        "-- " .. string.rep("-", width - 6),
        ""
      })

      -- Now make it read-only
      vim.api.nvim_buf_set_option(buf, 'modifiable', false)
    end
  end)
end

return M