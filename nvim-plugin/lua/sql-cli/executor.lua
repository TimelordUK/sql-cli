-- SQL CLI Query Executor Module
-- Functions for executing SQL queries and managing execution

local utils = require('sql-cli.utils')
local table_nav = require('sql-cli.table_nav')

local M = {}

-- Execute query from buffer or provided string
function M.execute_query(query, config, state)
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

  -- Save the query
  state:set_last_query(query)

  -- Execute
  M.run_command(query, false, config, state)
end

-- Execute query with execution plan
function M.execute_query_with_plan(query, config, state)
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

  M.execute_query(query, config, state)
end

-- Execute query at cursor with execution plan
function M.execute_at_cursor_with_plan(config, state)
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

  -- Add query plan flag if requested
  if show_plan then
    table.insert(cmd_parts, "--execution-plan")
  end

  return table.concat(cmd_parts, " ")
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
  local csv_lines = {}  -- Store CSV format results

  local job_id = vim.fn.jobstart(cmd, {
    stdout_buffered = true,
    stderr_buffered = true,
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
            -- Check if it's actually an error or just info
            if line:match("^#") or line:match("Query completed:") or line:match("rows in") then
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
        -- Append output
        if output_buf then
          -- Ensure buffer is modifiable before writing
          vim.bo[output_buf].modifiable = true
          vim.bo[output_buf].readonly = false
          vim.api.nvim_buf_set_lines(output_buf, -1, -1, false, output_lines)
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

        -- Focus output window if configured
        local output_win = state:get_output_win()
        if config.output.focus_on_run and ui_callbacks.is_output_window_valid and ui_callbacks.is_output_window_valid() and output_win then
          vim.api.nvim_set_current_win(output_win)
          -- Move cursor to end
          local line_count = vim.api.nvim_buf_line_count(output_buf)
          vim.api.nvim_win_set_cursor(output_win, {line_count, 0})

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
                  -- Pass the output window to init_navigation
                  local output_win = state:get_output_win()
                  if table_nav.init_navigation(output_buf, output_win) then
                    table_nav.setup_keymaps(output_buf, config)
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

return M