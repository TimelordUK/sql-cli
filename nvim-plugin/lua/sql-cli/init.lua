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
    toggle_output = "<leader>so",   -- Toggle output window
    set_data_file = "<leader>sd",   -- Set data file
    clear_data_file = "<leader>sc", -- Clear data file
    show_plan = "<leader>sp",       -- Show query plan
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
      state.data_file = M.detect_data_hint(lines)
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
  -- Get visual selection
  local start_pos = vim.fn.getpos("'<")
  local end_pos = vim.fn.getpos("'>")
  local lines = vim.api.nvim_buf_get_lines(
    0, start_pos[2] - 1, end_pos[2], false
  )
  
  if #lines == 0 then
    vim.notify("No selection", vim.log.levels.WARN)
    return
  end
  
  -- Handle partial line selection
  if #lines == 1 then
    lines[1] = string.sub(lines[1], start_pos[3], end_pos[3])
  else
    lines[1] = string.sub(lines[1], start_pos[3])
    lines[#lines] = string.sub(lines[#lines], 1, end_pos[3])
  end
  
  local query = table.concat(lines, "\n")
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
  local job_id = vim.fn.jobstart(cmd, {
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            table.insert(output_lines, line)
          end
        end
      end
    end,
    on_stderr = function(_, data)
      if data then
        for _, line in ipairs(data) do
          if line ~= "" then
            table.insert(output_lines, "ERROR: " .. line)
          end
        end
      end
    end,
    on_exit = function(_, exit_code)
      vim.schedule(function()
        -- Append output
        vim.api.nvim_buf_set_lines(state.output_buf, -1, -1, false, output_lines)
        
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
  
  -- Check if it's a script (has GO separator)
  local is_script = query:match("%sGO%s") or query:match("^GO%s") or query:match("%sGO$")
  
  if is_script then
    -- Save to temp file for script execution
    local temp_file = vim.fn.tempname() .. ".sql"
    local file = io.open(temp_file, "w")
    file:write(query)
    file:close()
    
    table.insert(cmd_parts, "-f")
    table.insert(cmd_parts, vim.fn.shellescape(temp_file))
  else
    -- Direct query
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
  
  -- Create buffer for output
  state.output_buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_name(state.output_buf, "[SQL CLI Output]")
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
function M.detect_data_hint(lines)
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
        -- Expand path
        hint = vim.fn.expand(hint)
        if vim.fn.filereadable(hint) == 1 then
          vim.notify("Auto-detected data file: " .. hint, vim.log.levels.INFO)
          return hint
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