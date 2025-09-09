-- SQL CLI Neovim Plugin
-- A companion plugin for the SQL CLI tool

local M = {}

-- Default configuration
M.config = {
  -- Path to sql-cli executable
  command = "sql-cli",
  
  -- Split configuration
  split = {
    direction = "vertical", -- "vertical" or "horizontal"
    size = 0.5,            -- Size as fraction (0.5 = 50%)
  },
  
  -- Default output format
  output_format = "table",
  
  -- Auto-detect features
  auto_detect = {
    csv_files = true,      -- Auto-detect when editing CSV files
    data_hints = true,     -- Auto-detect -- #!data: hints in SQL files
  },
  
  -- Keymaps (set to false to disable)
  keymaps = {
    execute = "<leader>sq",         -- Execute query
    execute_selection = "<leader>ss", -- Execute visual selection
    execute_at_cursor = "<leader>sx", -- Execute query at cursor
    toggle_output = "<leader>so",   -- Toggle output window
    toggle_orientation = "<leader>st", -- Toggle split orientation
    set_data_file = "<leader>sd",   -- Set data file
    clear_data_file = "<leader>sc", -- Clear data file
    show_plan = "<leader>sp",       -- Show query plan
    open_data_file = "<leader>sv",  -- View data file
    next_query = "]q",              -- Jump to next query
    prev_query = "[q",              -- Jump to previous query
    toggle_comment = "<leader>s/",  -- Toggle comment for query at cursor
    save_results_csv = "<leader>sw", -- Write results to CSV file
    results_to_buffer = "<leader>sb", -- Results to new buffer
  },
  
  -- Output window settings
  output = {
    focus_on_run = false,  -- Focus output window after execution
    clear_on_run = true,   -- Clear output before each run
    wrap = false,          -- Line wrap in output
    number = false,        -- Show line numbers in output
  }
}

-- Plugin state
local state = {
  data_file = nil,
  output_buf = nil,
  output_win = nil,
  last_query = nil,
  last_results = nil,  -- Store last query results for saving
  query_markers = {},  -- Track query positions in output
}

-- Setup function
function M.setup(opts)
  M.config = vim.tbl_deep_extend("force", M.config, opts or {})
  
  -- Create commands
  M.create_commands()
  
  -- Setup keymaps if enabled
  if M.config.keymaps then
    M.setup_keymaps()
  end
  
  -- Setup autocommands
  M.setup_autocmds()
end

-- Create user commands
function M.create_commands()
  vim.api.nvim_create_user_command("SqlCliExecute", function(args)
    M.execute_query(args.args)
  end, { nargs = "?", desc = "Execute SQL query" })
  
  vim.api.nvim_create_user_command("SqlCliSetData", function(args)
    M.set_data_file(args.args)
  end, { nargs = 1, complete = "file", desc = "Set data file for queries" })
  
  vim.api.nvim_create_user_command("SqlCliClearData", function()
    M.clear_data_file()
  end, { desc = "Clear data file setting" })
  
  vim.api.nvim_create_user_command("SqlCliShowPlan", function()
    M.show_query_plan()
  end, { desc = "Show query execution plan" })
  
  vim.api.nvim_create_user_command("SqlCliToggleOutput", function()
    M.toggle_output_window()
  end, { desc = "Toggle output window" })
end

-- Setup keymaps
function M.setup_keymaps()
  local keymaps = M.config.keymaps
  
  if keymaps.execute then
    vim.keymap.set("n", keymaps.execute, M.execute_query, 
      { desc = "Execute SQL query", silent = true })
  end
  
  if keymaps.execute_selection then
    vim.keymap.set("v", keymaps.execute_selection, M.execute_selection,
      { desc = "Execute selected SQL", silent = true })
  end
  
  if keymaps.toggle_output then
    vim.keymap.set("n", keymaps.toggle_output, M.toggle_output_window,
      { desc = "Toggle SQL output", silent = true })
  end
  
  if keymaps.set_data_file then
    vim.keymap.set("n", keymaps.set_data_file, function()
      vim.ui.input({ prompt = "Data file: ", completion = "file" }, function(input)
        if input then M.set_data_file(input) end
      end)
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
  
  if keymaps.execute_at_cursor then
    vim.keymap.set("n", keymaps.execute_at_cursor, M.execute_at_cursor,
      { desc = "Execute SQL query at cursor", silent = true })
  end
  
  if keymaps.toggle_orientation then
    vim.keymap.set("n", keymaps.toggle_orientation, M.toggle_split_orientation,
      { desc = "Toggle split orientation", silent = true })
  end
  
  if keymaps.open_data_file then
    vim.keymap.set("n", keymaps.open_data_file, M.open_data_file,
      { desc = "Open data file in buffer", silent = true })
  end
  
  if keymaps.next_query then
    vim.keymap.set("n", keymaps.next_query, M.next_query,
      { desc = "Jump to next query", silent = true })
  end
  
  if keymaps.prev_query then
    vim.keymap.set("n", keymaps.prev_query, M.prev_query,
      { desc = "Jump to previous query", silent = true })
  end
  
  if keymaps.toggle_comment then
    vim.keymap.set("n", keymaps.toggle_comment, M.toggle_comment_query,
      { desc = "Toggle comment for query at cursor", silent = true })
  end
  
  if keymaps.save_results_csv then
    vim.keymap.set("n", keymaps.save_results_csv, M.save_results_csv,
      { desc = "Save results to CSV file", silent = true })
  end
  
  if keymaps.results_to_buffer then
    vim.keymap.set("n", keymaps.results_to_buffer, M.results_to_buffer,
      { desc = "Open results in new buffer", silent = true })
  end
end

-- Setup autocommands
function M.setup_autocmds()
  local group = vim.api.nvim_create_augroup("SqlCli", { clear = true })
  
  -- Auto-detect CSV files
  if M.config.auto_detect.csv_files then
    vim.api.nvim_create_autocmd({ "BufRead", "BufNewFile" }, {
      group = group,
      pattern = "*.csv",
      callback = function(ev)
        -- Set this CSV as the data file if none is set
        if not state.data_file then
          state.data_file = ev.file
          vim.notify("SQL CLI: Using " .. ev.file .. " as data file", vim.log.levels.INFO)
        end
      end,
    })
  end
  
  -- Clean up on exit
  vim.api.nvim_create_autocmd("VimLeavePre", {
    group = group,
    callback = function()
      M.cleanup()
    end,
  })
end

-- Execute query from current buffer or provided string
function M.execute_query(query)
  -- Get query from buffer if not provided
  if not query or query == "" then
    local bufnr = vim.api.nvim_get_current_buf()
    local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    query = table.concat(lines, "\n")
    
    -- Auto-detect data file from hints
    if M.config.auto_detect.data_hints and not state.data_file then
      -- Get the directory of the current buffer
      local buf_path = vim.api.nvim_buf_get_name(bufnr)
      local buf_dir = nil
      if buf_path and buf_path ~= "" then
        buf_dir = vim.fn.fnamemodify(buf_path, ":h")
      end
      state.data_file = M.detect_data_hint(lines, buf_dir)
    end
    
    -- Auto-detect if current buffer is a CSV
    if M.config.auto_detect.csv_files and not state.data_file then
      local filename = vim.api.nvim_buf_get_name(bufnr)
      if filename:match("%.csv$") then
        state.data_file = filename
      end
    end
  end
  
  -- Save the query
  state.last_query = query
  
  -- Execute
  M.run_command(query, false)
end

-- Execute visual selection
function M.execute_selection()
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
  
  M.execute_query(query)
end

-- Run SQL CLI command
function M.run_command(query, show_plan)
  -- Create output window if needed
  if not M.is_output_window_valid() then
    M.create_output_window()
  end
  
  -- Clear output if configured
  if M.config.output.clear_on_run then
    vim.api.nvim_buf_set_lines(state.output_buf, 0, -1, false, {})
  end
  
  -- Build command
  local cmd = M.build_command(query, show_plan)
  
  -- Add header
  local header = {
    "-- SQL CLI Output --",
    "-- Command: " .. cmd,
    "-- " .. os.date("%Y-%m-%d %H:%M:%S"),
    string.rep("-", 60),
    "",
  }
  vim.api.nvim_buf_set_lines(state.output_buf, 0, 0, false, header)
  
  -- Run command asynchronously
  local output_lines = {}
  local csv_lines = {}  -- Store CSV format results
  local in_table = false  -- Track if we're in table output
  
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
        vim.api.nvim_buf_set_lines(state.output_buf, -1, -1, false, output_lines)
        
        -- Store results for saving (prefer CSV format if available)
        if #csv_lines > 0 then
          state.last_results = csv_lines
        else
          state.last_results = output_lines
        end
        
        -- Add footer
        local footer = {
          "",
          string.rep("-", 60),
          "-- Exit code: " .. exit_code,
        }
        vim.api.nvim_buf_set_lines(state.output_buf, -1, -1, false, footer)
        
        -- Focus output window if configured
        if M.config.output.focus_on_run and M.is_output_window_valid() then
          vim.api.nvim_set_current_win(state.output_win)
          -- Move cursor to end
          local line_count = vim.api.nvim_buf_line_count(state.output_buf)
          vim.api.nvim_win_set_cursor(state.output_win, {line_count, 0})
        end
        
        -- Notify completion
        if exit_code == 0 then
          vim.notify("Query executed successfully", vim.log.levels.INFO)
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

-- Build command line
function M.build_command(query, show_plan)
  local cmd_parts = { M.config.command }
  
  -- Add data file if set
  if state.data_file then
    table.insert(cmd_parts, vim.fn.shellescape(state.data_file))
  end
  
  -- Check if it's a script (has GO separator) or multi-line query
  local is_script = query:match("%sGO%s") or query:match("^GO%s") or query:match("%sGO$")
  local is_multiline = query:find("\n") ~= nil
  
  if is_script or is_multiline then
    -- Save to temp file for script execution or multi-line queries
    local temp_file = vim.fn.tempname() .. ".sql"
    local file = io.open(temp_file, "w")
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
  table.insert(cmd_parts, M.config.output_format)
  
  -- Add query plan flag if requested
  if show_plan then
    table.insert(cmd_parts, "--query-plan")
  end
  
  return table.concat(cmd_parts, " ")
end

-- Create output window
function M.create_output_window()
  -- Save current window
  local current_win = vim.api.nvim_get_current_win()
  
  -- Create split
  local split_cmd = M.config.split.direction == "vertical" and "vsplit" or "split"
  vim.cmd(split_cmd)
  
  -- Resize window
  local size
  if M.config.split.direction == "vertical" then
    size = math.floor(vim.o.columns * M.config.split.size)
    vim.api.nvim_win_set_width(0, size)
  else
    size = math.floor(vim.o.lines * M.config.split.size)
    vim.api.nvim_win_set_height(0, size)
  end
  
  -- Create or reuse buffer for output
  if not state.output_buf or not vim.api.nvim_buf_is_valid(state.output_buf) then
    state.output_buf = vim.api.nvim_create_buf(false, true)
    -- Try to set name, but handle if it already exists
    pcall(function()
      vim.api.nvim_buf_set_name(state.output_buf, "[SQL CLI Output]")
    end)
  end
  
  vim.api.nvim_win_set_buf(0, state.output_buf)
  state.output_win = vim.api.nvim_get_current_win()
  
  -- Set buffer options
  vim.bo[state.output_buf].buftype = "nofile"
  vim.bo[state.output_buf].bufhidden = "hide"
  vim.bo[state.output_buf].swapfile = false
  vim.bo[state.output_buf].filetype = "sql-cli-output"
  
  -- Apply syntax highlighting
  M.setup_output_highlighting()
  
  -- Set window options
  vim.wo[state.output_win].wrap = M.config.output.wrap
  vim.wo[state.output_win].number = M.config.output.number
  vim.wo[state.output_win].relativenumber = false
  vim.wo[state.output_win].signcolumn = "no"
  vim.wo[state.output_win].foldcolumn = "0"
  
  -- Return to original window
  vim.api.nvim_set_current_win(current_win)
end

-- Toggle output window
function M.toggle_output_window()
  if M.is_output_window_valid() then
    vim.api.nvim_win_close(state.output_win, false)
    state.output_win = nil
  else
    M.create_output_window()
    if state.last_query then
      M.execute_query(state.last_query)
    end
  end
end

-- Check if output window is valid
function M.is_output_window_valid()
  return state.output_win 
    and vim.api.nvim_win_is_valid(state.output_win)
    and state.output_buf
    and vim.api.nvim_buf_is_valid(state.output_buf)
end

-- Show query plan
function M.show_query_plan()
  if state.last_query then
    M.run_command(state.last_query, true)
  else
    M.execute_query(nil)
  end
end

-- Set data file
function M.set_data_file(file)
  if file and file ~= "" then
    -- Expand path
    file = vim.fn.expand(file)
    
    -- Check if file exists
    if vim.fn.filereadable(file) == 1 then
      state.data_file = file
      vim.notify("Data file set to: " .. file, vim.log.levels.INFO)
    else
      vim.notify("File not found: " .. file, vim.log.levels.ERROR)
    end
  end
end

-- Clear data file
function M.clear_data_file()
  state.data_file = nil
  vim.notify("Data file cleared", vim.log.levels.INFO)
end

-- Detect data hint from SQL comments
function M.detect_data_hint(lines, base_dir)
  for _, line in ipairs(lines) do
    -- Match various hint patterns
    local patterns = {
      "^%s*%-%-%s*#!data:%s*(.+)",
      "^%s*%-%-%s*#!datafile:%s*(.+)",
      "^%s*%-%-%s*#!%s+([%w%./%-_]+%.csv)",
      "^%s*%-%-%s*#!%s+([%w%./%-_]+%.json)",
    }
    
    for _, pattern in ipairs(patterns) do
      local hint = line:match(pattern)
      if hint then
        hint = vim.trim(hint)
        
        -- Resolve the path
        local resolved_path = hint
        
        -- Check if it's a relative path
        if not hint:match("^/") and not hint:match("^~") then
          if base_dir then
            -- Resolve relative to the buffer's directory
            resolved_path = base_dir .. "/" .. hint
          else
            -- Resolve relative to current working directory
            resolved_path = vim.fn.getcwd() .. "/" .. hint
          end
        end
        
        -- Expand ~ and normalize the path
        resolved_path = vim.fn.expand(resolved_path)
        resolved_path = vim.fn.fnamemodify(resolved_path, ":p")
        
        if vim.fn.filereadable(resolved_path) == 1 then
          vim.notify("Auto-detected data file: " .. resolved_path, vim.log.levels.INFO)
          return resolved_path
        else
          -- Try the original hint as-is (might be already correct)
          hint = vim.fn.expand(hint)
          if vim.fn.filereadable(hint) == 1 then
            vim.notify("Auto-detected data file: " .. hint, vim.log.levels.INFO)
            return hint
          end
        end
      end
    end
  end
  
  return nil
end

-- Get current data file
function M.get_data_file()
  return state.data_file
end

-- Status line component
function M.statusline()
  if state.data_file then
    local filename = vim.fn.fnamemodify(state.data_file, ":t")
    return string.format("SQL[📄%s]", filename)
  else
    return "SQL[∅]"
  end
end

-- Toggle split orientation
function M.toggle_split_orientation()
  -- Toggle the configuration
  if M.config.split.direction == "vertical" then
    M.config.split.direction = "horizontal"
    vim.notify("Split orientation: horizontal", vim.log.levels.INFO)
  else
    M.config.split.direction = "vertical"
    vim.notify("Split orientation: vertical", vim.log.levels.INFO)
  end
  
  -- If output window is open, recreate it with new orientation
  if M.is_output_window_valid() then
    -- Close current window
    vim.api.nvim_win_close(state.output_win, false)
    state.output_win = nil
    
    -- Recreate with new orientation
    M.create_output_window()
    
    -- Re-run last query if available
    if state.last_query then
      M.run_command(state.last_query, false)
    end
  end
end

-- Execute query at cursor position
function M.execute_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  
  -- Find the query boundaries (SELECT to GO/semicolon or next SELECT)
  local start_line = nil
  local end_line = nil
  
  -- Search backwards for SELECT or start of file
  for i = cursor_line, 1, -1 do
    if lines[i]:upper():match("^%s*SELECT") then
      start_line = i
      break
    end
  end
  
  -- If no SELECT found before cursor, search forward
  if not start_line then
    for i = cursor_line, #lines do
      if lines[i]:upper():match("^%s*SELECT") then
        start_line = i
        break
      end
    end
  end
  
  if not start_line then
    vim.notify("No SELECT statement found", vim.log.levels.WARN)
    return
  end
  
  -- Search forward for GO, semicolon, or next SELECT
  for i = start_line + 1, #lines do
    local line = lines[i]
    if line:match("^%s*GO%s*$") or line:match(";%s*$") then
      end_line = i
      break
    elseif i > start_line and line:upper():match("^%s*SELECT") then
      end_line = i - 1
      break
    end
  end
  
  -- If no end found, use end of file
  if not end_line then
    end_line = #lines
  end
  
  -- Extract the query
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end
  
  local query = table.concat(query_lines, "\n")
  
  -- Auto-detect data file if needed
  if M.config.auto_detect.data_hints and not state.data_file then
    local buf_path = vim.api.nvim_buf_get_name(bufnr)
    local buf_dir = nil
    if buf_path and buf_path ~= "" then
      buf_dir = vim.fn.fnamemodify(buf_path, ":h")
    end
    state.data_file = M.detect_data_hint(lines, buf_dir)
  end
  
  -- Execute the query
  M.execute_query(query)
end

-- Open data file in a new buffer
function M.open_data_file()
  local file = state.data_file
  
  -- If no data file is set, try to detect from current buffer
  if not file then
    local bufnr = vim.api.nvim_get_current_buf()
    local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    local buf_path = vim.api.nvim_buf_get_name(bufnr)
    local buf_dir = nil
    if buf_path and buf_path ~= "" then
      buf_dir = vim.fn.fnamemodify(buf_path, ":h")
    end
    file = M.detect_data_hint(lines, buf_dir)
  end
  
  if not file then
    vim.notify("No data file set or detected", vim.log.levels.WARN)
    return
  end
  
  -- Check if file exists
  if vim.fn.filereadable(file) ~= 1 then
    vim.notify("Data file not found: " .. file, vim.log.levels.ERROR)
    return
  end
  
  -- Open file in a new split
  vim.cmd("split " .. vim.fn.fnameescape(file))
  vim.notify("Opened data file: " .. file, vim.log.levels.INFO)
end

-- Jump to next query
function M.next_query()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  
  -- Find next SELECT statement
  for i = cursor_line + 1, #lines do
    if lines[i]:upper():match("^%s*SELECT") then
      vim.api.nvim_win_set_cursor(0, {i, 0})
      return
    end
  end
  
  -- Wrap around to beginning
  for i = 1, cursor_line do
    if lines[i]:upper():match("^%s*SELECT") then
      vim.api.nvim_win_set_cursor(0, {i, 0})
      vim.notify("Wrapped to first query", vim.log.levels.INFO)
      return
    end
  end
  
  vim.notify("No queries found", vim.log.levels.WARN)
end

-- Jump to previous query
function M.prev_query()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  
  -- Find previous SELECT statement
  for i = cursor_line - 1, 1, -1 do
    if lines[i]:upper():match("^%s*SELECT") then
      vim.api.nvim_win_set_cursor(0, {i, 0})
      return
    end
  end
  
  -- Wrap around to end
  for i = #lines, cursor_line, -1 do
    if lines[i]:upper():match("^%s*SELECT") then
      vim.api.nvim_win_set_cursor(0, {i, 0})
      vim.notify("Wrapped to last query", vim.log.levels.INFO)
      return
    end
  end
  
  vim.notify("No queries found", vim.log.levels.WARN)
end

-- Toggle comment for query at cursor
function M.toggle_comment_query()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  
  -- Find query boundaries
  local start_line = nil
  local end_line = nil
  
  -- Search backwards for SELECT
  for i = cursor_line, 1, -1 do
    if lines[i]:upper():match("^%s*SELECT") or lines[i]:upper():match("^%s*%-%-.*SELECT") then
      start_line = i
      break
    end
  end
  
  if not start_line then
    -- Search forward
    for i = cursor_line, #lines do
      if lines[i]:upper():match("^%s*SELECT") or lines[i]:upper():match("^%s*%-%-.*SELECT") then
        start_line = i
        break
      end
    end
  end
  
  if not start_line then
    vim.notify("No query found at cursor", vim.log.levels.WARN)
    return
  end
  
  -- Find end of query
  for i = start_line + 1, #lines do
    local line = lines[i]
    if line:match("^%s*GO%s*$") or line:match(";%s*$") then
      end_line = i
      break
    elseif i > start_line and (line:upper():match("^%s*SELECT") or line:upper():match("^%s*%-%-.*SELECT")) then
      end_line = i - 1
      break
    end
  end
  
  if not end_line then
    end_line = #lines
  end
  
  -- Check if query is commented
  local is_commented = lines[start_line]:match("^%s*%-%-")
  
  -- Toggle comments
  for i = start_line, end_line do
    local line = lines[i]
    if is_commented then
      -- Remove comment
      lines[i] = line:gsub("^%s*%-%-", "")
    else
      -- Add comment
      if line:match("^%s*$") then
        -- Don't comment empty lines
        lines[i] = line
      else
        lines[i] = "-- " .. line
      end
    end
  end
  
  vim.api.nvim_buf_set_lines(bufnr, start_line - 1, end_line, false, vim.list_slice(lines, start_line, end_line))
  
  if is_commented then
    vim.notify("Query uncommented", vim.log.levels.INFO)
  else
    vim.notify("Query commented", vim.log.levels.INFO)
  end
end

-- Save results to CSV file
function M.save_results_csv()
  if not state.last_results or #state.last_results == 0 then
    vim.notify("No results to save", vim.log.levels.WARN)
    return
  end
  
  vim.ui.input({ prompt = "Save results to: ", completion = "file", default = "results.csv" }, function(filename)
    if not filename then return end
    
    -- Ensure .csv extension
    if not filename:match("%.csv$") then
      filename = filename .. ".csv"
    end
    
    -- Write results to file
    local file = io.open(filename, "w")
    if not file then
      vim.notify("Failed to create file: " .. filename, vim.log.levels.ERROR)
      return
    end
    
    for _, line in ipairs(state.last_results) do
      file:write(line .. "\n")
    end
    file:close()
    
    vim.notify("Results saved to: " .. filename, vim.log.levels.INFO)
  end)
end

-- Open results in new buffer
function M.results_to_buffer()
  if not state.last_results or #state.last_results == 0 then
    vim.notify("No results to display", vim.log.levels.WARN)
    return
  end
  
  -- Create new buffer
  vim.cmd("new")
  local buf = vim.api.nvim_get_current_buf()
  
  -- Set buffer content
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, state.last_results)
  
  -- Set buffer options
  vim.bo[buf].filetype = "csv"
  vim.bo[buf].modified = false
  vim.api.nvim_buf_set_name(buf, "[SQL Results]")
  
  vim.notify("Results opened in new buffer", vim.log.levels.INFO)
end

-- Setup syntax highlighting for output buffer
function M.setup_output_highlighting()
  if not state.output_buf or not vim.api.nvim_buf_is_valid(state.output_buf) then
    return
  end
  
  -- Apply highlighting in the output buffer
  vim.api.nvim_buf_call(state.output_buf, function()
    -- Header highlighting
    vim.cmd([[syntax match SqlCliHeader /^--.*$/]])
    vim.cmd([[syntax match SqlCliSeparator /^-\+$/]])
    
    -- Table borders and structure
    vim.cmd([[syntax match SqlCliTableBorder /[│├┤┌┐└┘─┬┴┼]/]])
    vim.cmd([[syntax match SqlCliTablePipe /|/]])
    
    -- Numbers
    vim.cmd([[syntax match SqlCliNumber /\<\d\+\(\.\d\+\)\?\>/]])
    
    -- NULL values
    vim.cmd([[syntax match SqlCliNull /\<NULL\>/]])
    
    -- Booleans
    vim.cmd([[syntax match SqlCliBoolean /\<\(true\|false\|TRUE\|FALSE\)\>/]])
    
    -- Error messages
    vim.cmd([[syntax match SqlCliError /^ERROR:.*$/]])
    vim.cmd([[syntax match SqlCliError /Error:.*$/]])
    vim.cmd([[syntax match SqlCliError /failed.*$/]])
    
    -- Success messages (like "# Query completed: 10 rows in 906.909µs")
    vim.cmd([[syntax match SqlCliSuccess /^#.*Query completed:.*$/]])
    vim.cmd([[syntax match SqlCliSuccess /.*rows in.*µs$/]])
    vim.cmd([[syntax match SqlCliSuccess /^#.*$/]])
    
    -- Exit code line
    vim.cmd([[syntax match SqlCliExitCode /^-- Exit code:.*$/]])
    
    -- CSV values (quoted strings)
    vim.cmd([[syntax match SqlCliString /"[^"]*"/]])
    vim.cmd([[syntax match SqlCliString /'[^']*'/]])
    
    -- Set highlight colors
    vim.cmd([[highlight SqlCliHeader guifg=#6272a4 ctermfg=8]])
    vim.cmd([[highlight SqlCliSeparator guifg=#44475a ctermfg=8]])
    vim.cmd([[highlight SqlCliTableBorder guifg=#8be9fd ctermfg=14]])
    vim.cmd([[highlight SqlCliTablePipe guifg=#8be9fd ctermfg=14]])
    vim.cmd([[highlight SqlCliNumber guifg=#bd93f9 ctermfg=13]])
    vim.cmd([[highlight SqlCliNull guifg=#ff79c6 ctermfg=5]])
    vim.cmd([[highlight SqlCliBoolean guifg=#50fa7b ctermfg=10]])
    vim.cmd([[highlight SqlCliError guifg=#ff5555 ctermfg=9]])
    vim.cmd([[highlight SqlCliSuccess guifg=#50fa7b ctermfg=10]])  -- Green for success
    vim.cmd([[highlight SqlCliExitCode guifg=#f8f8f2 ctermfg=7]])
    vim.cmd([[highlight SqlCliString guifg=#f1fa8c ctermfg=11]])
  end)
end

-- Cleanup
function M.cleanup()
  if state.output_win and vim.api.nvim_win_is_valid(state.output_win) then
    vim.api.nvim_win_close(state.output_win, true)
  end
  if state.output_buf and vim.api.nvim_buf_is_valid(state.output_buf) then
    vim.api.nvim_buf_delete(state.output_buf, { force = true })
  end
end

return M