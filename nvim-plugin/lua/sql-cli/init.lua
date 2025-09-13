-- SQL CLI Neovim Plugin
-- A companion plugin for the SQL CLI tool

local M = {}

-- Helper function: Strip ANSI escape sequences from text
local function strip_ansi_codes(text)
  -- Remove ANSI color codes and formatting codes
  -- \x1b is the escape character (octal 033, decimal 27)
  text = text:gsub("\x1b%[[%d;]*m", "")  -- Color codes like \x1b[38;5;12m
  text = text:gsub("\x1b%[%d*m", "")      -- Simple codes like \x1b[0m
  text = text:gsub("\x1b%[%d*;%d*m", "")  -- Codes like \x1b[1;32m
  text = text:gsub("\x1b%[K", "")         -- Clear to end of line
  text = text:gsub("\x1b%[[%d;]*[A-Za-z]", "") -- Other control sequences
  return text
end

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
    execute_with_plan = "<leader>sX", -- Execute query at cursor with execution plan
    select_query = "<leader>sv",    -- Visually select query at cursor
    preview_query = "<leader>sP",   -- Preview query in floating window
    toggle_output = "<leader>so",   -- Toggle output window
    toggle_orientation = "<leader>st", -- Toggle split orientation
    set_data_file = "<leader>sd",   -- Set data file
    clear_data_file = "<leader>sc", -- Clear data file
    show_plan = "<leader>sp",       -- Show query plan
    open_data_file = "<leader>sV",  -- View data file (capital V to avoid conflict)
    next_query = "]q",              -- Jump to next query
    prev_query = "[q",              -- Jump to previous query
    toggle_comment = "<leader>s/",  -- Toggle comment for query at cursor
    save_results_csv = "<leader>sw", -- Write results to CSV file
    results_to_buffer = "<leader>sb", -- Results to new buffer
    function_help = "K",            -- Show function help at cursor
    list_functions = "<leader>sf",  -- List all SQL functions
    search_functions = "<leader>sF", -- Search SQL functions
    show_schema = "<leader>sh",     -- Show table schema
    column_help = "<leader>sk",     -- Smart column/function detection at cursor
    expand_star = "<leader>se",     -- Expand SELECT * to column names
    copy_query = "<leader>sy",       -- Copy query at cursor to clipboard (y for yank)
    format_query = "<leader>s=",     -- Format/prettify query at cursor
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

-- Helper function to check if a line starts a SQL statement
local function is_statement_start(line)
  if not line then return false end
  local upper = line:upper()
  return upper:match("^%s*WITH%s+") or        -- CTE
         upper:match("^%s*SELECT%s+") or      -- SELECT
         upper:match("^%s*INSERT%s+") or      -- INSERT
         upper:match("^%s*UPDATE%s+") or      -- UPDATE
         upper:match("^%s*DELETE%s+") or      -- DELETE
         upper:match("^%s*CREATE%s+") or      -- CREATE
         upper:match("^%s*DROP%s+") or        -- DROP
         upper:match("^%s*ALTER%s+")          -- ALTER
end

-- Helper function to find query boundaries at cursor position
local function find_query_at_cursor(lines, cursor_line)
  -- Helper function to check if a line is a query terminator
  local function is_terminator(line)
    if not line then return false end
    return line:match("^%s*GO%s*$") or        -- GO on its own line
           line:match(";%s*$") or              -- Semicolon at end of line
           line:match(";%s*%-%-") or           -- Semicolon followed by comment
           line:match(";%s*/")                 -- Semicolon followed by comment
  end
  
  local start_line = 1
  local end_line = #lines
  
  -- STEP 1: Search backwards from cursor for previous terminator
  for i = cursor_line - 1, 1, -1 do
    if is_terminator(lines[i]) then
      start_line = i + 1
      break
    end
  end
  
  -- STEP 2: Search forwards from cursor for next terminator  
  for i = cursor_line, #lines do
    if is_terminator(lines[i]) then
      end_line = i
      break
    end
  end
  
  -- Trim empty lines at start and end
  while start_line <= end_line and (not lines[start_line] or lines[start_line]:match("^%s*$")) do
    start_line = start_line + 1
  end
  
  if not is_terminator(lines[end_line]) then
    while end_line > start_line and (not lines[end_line] or lines[end_line]:match("^%s*$")) do
      end_line = end_line - 1
    end
  end
  
  return start_line, end_line
end

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
  
  -- Setup completion
  M.setup_completion()
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
  
  vim.api.nvim_create_user_command("SqlCliCopyQuery", function()
    M.copy_query_at_cursor()
  end, { desc = "Copy query at cursor to clipboard" })
  
  vim.api.nvim_create_user_command("SqlCliSelectQuery", function()
    M.select_query_at_cursor()
  end, { desc = "Visually select SQL query at cursor" })
  
  vim.api.nvim_create_user_command("SqlCliPreviewQuery", function()
    M.preview_query_at_cursor()
  end, { desc = "Preview SQL query at cursor in floating window" })
  
  vim.api.nvim_create_user_command("SqlCliFormatQuery", function()
    M.format_query_at_cursor()
  end, { desc = "Format SQL query at cursor" })
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
  
  if keymaps.execute_with_plan then
    vim.keymap.set("n", keymaps.execute_with_plan, M.execute_at_cursor_with_plan,
      { desc = "Execute SQL query at cursor with execution plan", silent = true })
  end
  
  if keymaps.select_query then
    vim.keymap.set("n", keymaps.select_query, M.select_query_at_cursor,
      { desc = "Visually select SQL query at cursor", silent = true })
  end
  
  if keymaps.preview_query then
    vim.keymap.set("n", keymaps.preview_query, M.preview_query_at_cursor,
      { desc = "Preview SQL query at cursor in floating window", silent = true })
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
  
  if keymaps.function_help then
    vim.keymap.set("n", keymaps.function_help, M.show_function_help,
      { desc = "Show SQL function help", silent = true })
  end
  
  if keymaps.list_functions then
    vim.keymap.set("n", keymaps.list_functions, M.list_functions,
      { desc = "List all SQL functions", silent = true })
  end
  
  if keymaps.search_functions then
    vim.keymap.set("n", keymaps.search_functions, M.search_functions,
      { desc = "Search SQL functions", silent = true })
  end
  
  if keymaps.show_schema then
    vim.keymap.set("n", keymaps.show_schema, M.show_schema,
      { desc = "Show table schema", silent = true })
  end
  
  if keymaps.column_help then
    vim.keymap.set("n", keymaps.column_help, M.get_column_at_cursor,
      { desc = "Smart column/function detection at cursor", silent = true })
  end
  
  if keymaps.expand_star then
    vim.keymap.set("n", keymaps.expand_star, M.expand_star,
      { desc = "Expand SELECT * to column names", silent = true })
    vim.keymap.set("v", keymaps.expand_star, M.expand_star_visual,
      { desc = "Expand SELECT * in selection", silent = true })
  end
  
  if keymaps.copy_query then
    vim.keymap.set("n", keymaps.copy_query, M.copy_query_at_cursor,
      { desc = "Copy SQL query at cursor to clipboard", silent = true })
  end
  
  if keymaps.format_query then
    vim.keymap.set("n", keymaps.format_query, M.format_query_at_cursor,
      { desc = "Format SQL query at cursor", silent = true })
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
          -- Load schema for completion
          M.load_schema_for_completion()
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
      if state.data_file then
        -- Load schema for completion
        M.load_schema_for_completion()
      end
    end
    
    -- Auto-detect if current buffer is a CSV
    if M.config.auto_detect.csv_files and not state.data_file then
      local filename = vim.api.nvim_buf_get_name(bufnr)
      if filename:match("%.csv$") then
        state.data_file = filename
        -- Load schema for completion
        M.load_schema_for_completion()
      end
    end
  end
  
  -- Save the query
  state.last_query = query
  
  -- Execute
  M.run_command(query, false)
end

-- Execute query with execution plan
function M.execute_query_with_plan(query)
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
      if state.data_file then
        -- Load schema for completion
        M.load_schema_for_completion()
      end
    end
    
    -- Auto-detect if current buffer is a CSV
    if M.config.auto_detect.csv_files and not state.data_file then
      local filename = vim.api.nvim_buf_get_name(bufnr)
      if filename:match("%.csv$") then
        state.data_file = filename
        -- Load schema for completion
        M.load_schema_for_completion()
      end
    end
  end
  
  -- Save the query
  state.last_query = query
  
  -- Execute with execution plan
  M.run_command(query, true)
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
    table.insert(cmd_parts, "--execution-plan")
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
      
      -- Load schema for completion
      M.load_schema_for_completion()
    else
      vim.notify("File not found: " .. file, vim.log.levels.ERROR)
    end
  end
end

-- Clear data file
function M.clear_data_file()
  state.data_file = nil
  state.schema_columns = nil  -- Clear cached schema
  vim.notify("Data file cleared", vim.log.levels.INFO)
end

-- Load schema for completion (called when data file is set)
function M.load_schema_for_completion()
  if not state.data_file then
    return
  end
  
  -- Run schema-json command to get clean JSON output
  local cmd = M.config.command .. " " .. vim.fn.shellescape(state.data_file) .. " --schema-json"
  local result = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error
  
  if exit_code == 0 then
    -- Parse JSON schema
    local ok, schema = pcall(vim.json.decode, result)
    if ok and schema and schema.columns then
      state.schema_columns = {}
      for _, col in ipairs(schema.columns) do
        table.insert(state.schema_columns, {
          name = col.name,
          type = col.type
        })
      end
      
      if #state.schema_columns > 0 then
        vim.notify(string.format("Loaded schema: %d columns from %s", #state.schema_columns, schema.table), vim.log.levels.INFO)
      end
    else
      -- Fallback to old method if JSON parsing fails
      state.schema_columns = {}
      local clean_result = strip_ansi_codes(result)
      for line in clean_result:gmatch("[^\n]+") do
        -- Match lines like "1. column_name TYPE"
        local num, col_name, col_type = line:match("%s*(%d+)%.%s+([%w_]+)%s+([%w]+)")
        if col_name and col_type then
          table.insert(state.schema_columns, {
            name = col_name,
            type = col_type
          })
        end
      end
      
      if #state.schema_columns > 0 then
        vim.notify(string.format("Loaded schema: %d columns", #state.schema_columns), vim.log.levels.INFO)
      end
    end
  end
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

-- Visually select query at cursor position
function M.select_query_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  
  -- Find query boundaries
  local start_line, end_line = find_query_at_cursor(lines, cursor_line)
  
  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end
  
  -- Enter visual line mode and select the query
  vim.cmd('normal! ' .. start_line .. 'G')  -- Go to start line
  vim.cmd('normal! V')                      -- Enter visual line mode
  vim.cmd('normal! ' .. end_line .. 'G')    -- Extend selection to end line
  
  -- Show a notification about what was selected
  local query_type = "Query"
  if lines[start_line]:upper():match("^%s*WITH%s+") then
    query_type = "CTE"
  elseif lines[start_line]:upper():match("^%s*SELECT%s+") then
    query_type = "SELECT"
  elseif lines[start_line]:upper():match("^%s*INSERT%s+") then
    query_type = "INSERT"
  elseif lines[start_line]:upper():match("^%s*UPDATE%s+") then
    query_type = "UPDATE"
  elseif lines[start_line]:upper():match("^%s*DELETE%s+") then
    query_type = "DELETE"
  end
  
  vim.notify(string.format("%s selected (lines %d-%d)", query_type, start_line, end_line), vim.log.levels.INFO)
end

-- Preview query at cursor with highlighting (shows in floating window)
function M.preview_query_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  
  -- Find query boundaries
  local start_line, end_line = find_query_at_cursor(lines, cursor_line)
  
  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end
  
  -- Extract the query lines
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end
  
  -- Create floating window for preview
  local width = math.min(80, vim.o.columns - 10)
  local height = math.min(#query_lines + 4, vim.o.lines - 10)
  
  -- Calculate centered position
  local row = math.floor((vim.o.lines - height) / 2)
  local col = math.floor((vim.o.columns - width) / 2)
  
  -- Create buffer for preview
  local preview_buf = vim.api.nvim_create_buf(false, true)
  
  -- Add header
  local header = "─── Query Preview (press q or <Esc> to close) ───"
  local padding = math.floor((width - #header) / 2)
  vim.api.nvim_buf_set_lines(preview_buf, 0, -1, false, {
    string.rep(" ", padding) .. header,
    "",
  })
  
  -- Add query lines
  vim.api.nvim_buf_set_lines(preview_buf, -1, -1, false, query_lines)
  
  -- Add footer with line info
  local footer = string.format("Lines %d-%d (%d lines)", start_line, end_line, end_line - start_line + 1)
  vim.api.nvim_buf_set_lines(preview_buf, -1, -1, false, {
    "",
    string.rep(" ", math.floor((width - #footer) / 2)) .. footer,
  })
  
  -- Create floating window
  local win_opts = {
    relative = "editor",
    row = row,
    col = col,
    width = width,
    height = height,
    style = "minimal",
    border = "rounded",
    title = " SQL Query Preview ",
    title_pos = "center",
  }
  
  local preview_win = vim.api.nvim_open_win(preview_buf, true, win_opts)
  
  -- Set buffer options
  vim.api.nvim_buf_set_option(preview_buf, "bufhidden", "delete")
  vim.api.nvim_buf_set_option(preview_buf, "filetype", "sql")
  vim.api.nvim_buf_set_option(preview_buf, "modifiable", false)
  
  -- Set window options
  vim.api.nvim_win_set_option(preview_win, "cursorline", true)
  vim.api.nvim_win_set_option(preview_win, "wrap", false)
  
  -- Add keymaps to close preview
  local close_preview = function()
    if vim.api.nvim_win_is_valid(preview_win) then
      vim.api.nvim_win_close(preview_win, true)
    end
  end
  
  vim.keymap.set("n", "q", close_preview, { buffer = preview_buf })
  vim.keymap.set("n", "<Esc>", close_preview, { buffer = preview_buf })
  vim.keymap.set("n", "<CR>", function()
    close_preview()
    M.execute_at_cursor()
  end, { buffer = preview_buf, desc = "Execute query and close preview" })
end

-- Execute query at cursor position
function M.execute_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  
  -- Find query boundaries
  local start_line, end_line = find_query_at_cursor(lines, cursor_line)
  
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
  
  -- Extract the query
  local query_lines = {}
  for i = start_line, end_line do
    -- Skip GO terminators
    if not lines[i]:match("^%s*GO%s*$") then
      table.insert(query_lines, lines[i])
    end
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

-- Execute query at cursor position with execution plan
function M.execute_at_cursor_with_plan()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  
  -- Find query boundaries
  local start_line, end_line = find_query_at_cursor(lines, cursor_line)
  
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
  
  -- Extract the query
  local query_lines = {}
  for i = start_line, end_line do
    -- Skip GO terminators
    if not lines[i]:match("^%s*GO%s*$") then
      table.insert(query_lines, lines[i])
    end
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
  
  -- Execute the query with execution plan
  M.execute_query_with_plan(query)
end

-- Copy query at cursor to clipboard
function M.copy_query_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  
  -- Find query boundaries
  local start_line, end_line = find_query_at_cursor(lines, cursor_line)
  
  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end
  
  -- Extract the query
  local query_lines = {}
  for i = start_line, end_line do
    -- Skip GO terminators
    if not lines[i]:match("^%s*GO%s*$") then
      table.insert(query_lines, lines[i])
    end
  end
  
  local query = table.concat(query_lines, "\n")
  
  -- Copy to system clipboard (+ register)
  vim.fn.setreg('+', query)
  -- Also copy to unnamed register for convenience
  vim.fn.setreg('"', query)
  
  vim.notify("Query copied to clipboard", vim.log.levels.INFO)
end

-- Format SQL query at cursor
function M.format_query_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  
  -- Find query boundaries using the same logic as execute_at_cursor
  local start_line, end_line = find_query_at_cursor(lines, cursor_line)
  
  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end
  
  -- Extract the query
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end
  
  local query = table.concat(query_lines, " ")
  
  -- Format the query
  local formatted = M.format_sql(query)
  
  -- Split formatted query into lines
  local formatted_lines = vim.split(formatted, "\n")
  
  -- Replace the original query with formatted version
  vim.api.nvim_buf_set_lines(bufnr, start_line - 1, end_line, false, formatted_lines)
  
  vim.notify("Query formatted", vim.log.levels.INFO)
end

-- SQL Formatter function
function M.format_sql(query)
  -- Remove extra whitespace and normalize
  query = query:gsub("%s+", " "):gsub("^%s+", ""):gsub("%s+$", "")
  
  -- SQL keywords that should be on new lines
  local keywords = {
    "SELECT", "FROM", "WHERE", "GROUP BY", "HAVING", "ORDER BY", 
    "LIMIT", "OFFSET", "JOIN", "LEFT JOIN", "RIGHT JOIN", "INNER JOIN",
    "OUTER JOIN", "CROSS JOIN", "ON", "AND", "OR", "CASE", "WHEN", 
    "THEN", "ELSE", "END", "WITH", "AS", "UNION", "UNION ALL", "EXCEPT", 
    "INTERSECT"
  }
  
  -- Create a pattern for keywords (case-insensitive)
  local function make_keyword_pattern(keyword)
    local pattern = ""
    for i = 1, #keyword do
      local char = keyword:sub(i, i)
      if char == " " then
        pattern = pattern .. "%s+"
      else
        pattern = pattern .. "[" .. char:upper() .. char:lower() .. "]"
      end
    end
    return pattern
  end
  
  -- First, handle SELECT clause - put columns on separate lines
  query = query:gsub("([Ss][Ee][Ll][Ee][Cc][Tt])%s+", "%1\n    ")
  
  -- Handle commas in SELECT clause (before FROM)
  local select_part, rest = query:match("^(.-)%s+([Ff][Rr][Oo][Mm].*)$")
  if select_part then
    -- Add newline after commas in SELECT, maintaining indentation
    select_part = select_part:gsub(",%s*", ",\n    ")
    query = select_part .. "\n" .. rest
  end
  
  -- Put major clauses on new lines
  query = query:gsub("%s+([Ff][Rr][Oo][Mm])%s+", "\nFROM ")
  query = query:gsub("%s+([Ww][Hh][Ee][Rr][Ee])%s+", "\nWHERE ")
  query = query:gsub("%s+([Gg][Rr][Oo][Uu][Pp]%s+[Bb][Yy])%s+", "\nGROUP BY ")
  query = query:gsub("%s+([Hh][Aa][Vv][Ii][Nn][Gg])%s+", "\nHAVING ")
  query = query:gsub("%s+([Oo][Rr][Dd][Ee][Rr]%s+[Bb][Yy])%s+", "\nORDER BY ")
  query = query:gsub("%s+([Ll][Ii][Mm][Ii][Tt])%s+", "\nLIMIT ")
  query = query:gsub("%s+([Oo][Ff][Ff][Ss][Ee][Tt])%s+", "\nOFFSET ")
  
  -- Handle JOIN clauses (must be before single JOIN to avoid breaking compound joins)
  query = query:gsub("%s+([Ff][Uu][Ll][Ll]%s+[Oo][Uu][Tt][Ee][Rr]%s+[Jj][Oo][Ii][Nn])%s+", "\nFULL OUTER JOIN ")
  query = query:gsub("%s+([Ll][Ee][Ff][Tt]%s+[Jj][Oo][Ii][Nn])%s+", "\nLEFT JOIN ")
  query = query:gsub("%s+([Rr][Ii][Gg][Hh][Tt]%s+[Jj][Oo][Ii][Nn])%s+", "\nRIGHT JOIN ")
  query = query:gsub("%s+([Ii][Nn][Nn][Ee][Rr]%s+[Jj][Oo][Ii][Nn])%s+", "\nINNER JOIN ")
  query = query:gsub("%s+([Cc][Rr][Oo][Ss][Ss]%s+[Jj][Oo][Ii][Nn])%s+", "\nCROSS JOIN ")
  -- Only convert standalone JOIN if it wasn't already part of a compound JOIN
  query = query:gsub("([^%w])([Jj][Oo][Ii][Nn])%s+", "%1\nJOIN ")
  
  -- Indent ON clauses for JOINs
  query = query:gsub("%s+([Oo][Nn])%s+", "\n    ON ")
  
  -- Handle AND/OR in WHERE clause with proper indentation
  query = query:gsub("%s+([Aa][Nn][Dd])%s+", "\n    AND ")
  query = query:gsub("%s+([Oo][Rr])%s+", "\n    OR ")
  
  -- Handle CASE statements
  query = query:gsub("%s+([Cc][Aa][Ss][Ee])%s+", "\n    CASE ")
  query = query:gsub("%s+([Ww][Hh][Ee][Nn])%s+", "\n        WHEN ")
  query = query:gsub("%s+([Tt][Hh][Ee][Nn])%s+", " THEN ")
  query = query:gsub("%s+([Ee][Ll][Ss][Ee])%s+", "\n        ELSE ")
  query = query:gsub("%s+([Ee][Nn][Dd])%s+", "\n    END ")
  
  -- Clean up any double newlines
  query = query:gsub("\n\n+", "\n")
  
  -- Remove trailing semicolon or GO for reformatting
  query = query:gsub(";%s*$", "")
  query = query:gsub("%s*GO%s*$", "")
  
  -- Add semicolon at the end
  query = query .. ";"
  
  return query
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
  
  -- Find next SQL statement
  for i = cursor_line + 1, #lines do
    if is_statement_start(lines[i]) then
      vim.api.nvim_win_set_cursor(0, {i, 0})
      return
    end
  end
  
  -- Wrap around to beginning
  for i = 1, cursor_line do
    if is_statement_start(lines[i]) then
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
  
  -- Find previous SQL statement
  for i = cursor_line - 1, 1, -1 do
    if is_statement_start(lines[i]) then
      vim.api.nvim_win_set_cursor(0, {i, 0})
      return
    end
  end
  
  -- Wrap around to end
  for i = #lines, cursor_line, -1 do
    if is_statement_start(lines[i]) then
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
  
  -- Find query boundaries using the same logic as execute_at_cursor
  local start_line, end_line = find_query_at_cursor(lines, cursor_line)
  
  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end
  
  -- Find the first actual SQL statement line (skip documentation comments)
  local sql_start_line = start_line
  for i = start_line, end_line do
    local line = lines[i]
    if line and not line:match("^%s*$") and not line:match("^%s*%-%-") then
      sql_start_line = i
      break
    elseif line and line:match("^%s*%-%-") and is_statement_start(line:gsub("^%s*%-%-", "")) then
      -- This is a commented-out SQL statement
      sql_start_line = i
      break
    end
  end
  
  -- Check if the SQL code is commented (not just documentation comments)
  local is_commented = false
  local first_sql_line = lines[sql_start_line]
  if first_sql_line and first_sql_line:match("^%s*%-%-") then
    local uncommented = first_sql_line:gsub("^%s*%-%-", "")
    -- Only consider it "commented" if the uncommented line is SQL code
    if is_statement_start(uncommented) then
      is_commented = true
    end
  end
  
  -- Toggle comments on SQL lines only (preserve documentation comments)
  for i = start_line, end_line do
    local line = lines[i]
    if line and not line:match("^%s*$") then
      -- Skip documentation comments that aren't SQL code
      local is_doc_comment = line:match("^%s*%-%-") and not is_statement_start(line:gsub("^%s*%-%-", ""))
      
      if not is_doc_comment then
        if is_commented then
          -- Remove comment from SQL lines
          lines[i] = line:gsub("^(%s*)%-%-(%s?)", "%1")
        else
          -- Add comment to SQL lines
          local indent = line:match("^(%s*)")
          lines[i] = indent .. "-- " .. line:sub(#indent + 1)
        end
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

-- Show function help for word under cursor
function M.show_function_help()
  -- Get word under cursor
  local word = vim.fn.expand("<cword>"):upper()
  
  if word == "" then
    vim.notify("No word under cursor", vim.log.levels.WARN)
    return
  end
  
  -- Build command to get function help
  local cmd = M.config.command .. " --function-help " .. vim.fn.shellescape(word)
  
  -- Execute command
  local result = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error
  
  if exit_code ~= 0 then
    -- Try searching for functions containing this word
    local search_cmd = M.config.command .. " --list-functions"
    local all_functions = vim.fn.system(search_cmd)
    
    -- Check if any function contains this word
    local matches = {}
    for line in all_functions:gmatch("[^\n]+") do
      if line:upper():find(word) then
        table.insert(matches, line)
      end
    end
    
    if #matches > 0 then
      vim.notify("Function '" .. word .. "' not found. Similar functions:\n" .. table.concat(matches, "\n"), vim.log.levels.WARN)
    else
      vim.notify("Function '" .. word .. "' not found", vim.log.levels.WARN)
    end
    return
  end
  
  -- Create a floating window to show help
  M.show_help_in_float(word, result)
end


-- Show help text in a floating window
function M.show_help_in_float(title, content)
  -- Strip ANSI codes from content
  content = strip_ansi_codes(content)
  
  -- Split content into lines
  local lines = {}
  for line in content:gmatch("[^\n]+") do
    table.insert(lines, line)
  end
  
  -- Create buffer
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  
  -- Set buffer options
  vim.bo[buf].modifiable = false
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].filetype = ""  -- No filetype to avoid unwanted highlighting
  
  -- Apply custom syntax highlighting for schema display
  vim.api.nvim_buf_call(buf, function()
    -- Highlight column numbers consistently
    vim.cmd([[syntax match SqlSchemaNumber /^\s*\d\+\./]])
    vim.cmd([[syntax match SqlSchemaColumnName /\d\+\.\s\+\zs\S\+/]])
    vim.cmd([[syntax match SqlSchemaType /\s\+\zs\(String\|Integer\|Float\|Boolean\|DateTime\|Mixed\|Null\)\ze/]])
    vim.cmd([[syntax match SqlSchemaNullPercent /(\d\+% NULL)/]])
    vim.cmd([[syntax match SqlSchemaHeader /^Table:\|^Rows:\|^Columns:/]])
    vim.cmd([[syntax match SqlSchemaSeparator /^-\+$/]])
    
    -- Define colors
    vim.cmd([[hi def link SqlSchemaNumber Number]])
    vim.cmd([[hi def link SqlSchemaColumnName Identifier]])
    vim.cmd([[hi def link SqlSchemaType Type]])
    vim.cmd([[hi def link SqlSchemaNullPercent Comment]])
    vim.cmd([[hi def link SqlSchemaHeader Title]])
    vim.cmd([[hi def link SqlSchemaSeparator NonText]])
  end)
  
  -- Calculate window size
  local width = 80
  local height = math.min(#lines, 30)
  
  -- Get editor dimensions
  local ui = vim.api.nvim_list_uis()[1]
  local row = math.floor((ui.height - height) / 2)
  local col = math.floor((ui.width - width) / 2)
  
  -- Create floating window
  local opts = {
    relative = "editor",
    width = width,
    height = height,
    row = row,
    col = col,
    style = "minimal",
    border = "rounded",
    title = " " .. title .. " Function Help ",
    title_pos = "center",
  }
  
  local win = vim.api.nvim_open_win(buf, true, opts)
  
  -- Set window options
  vim.wo[win].wrap = true
  vim.wo[win].linebreak = true
  vim.wo[win].cursorline = true
  
  -- Set up keymaps to close window
  vim.keymap.set("n", "q", function()
    vim.api.nvim_win_close(win, true)
  end, { buffer = buf, silent = true })
  
  vim.keymap.set("n", "<Esc>", function()
    vim.api.nvim_win_close(win, true)
  end, { buffer = buf, silent = true })
  
  -- Add syntax highlighting for SQL code blocks
  vim.cmd([[
    syntax match SqlFunctionName /^[A-Z_]\+/
    syntax match SqlFunctionCategory /^Category:.*/
    syntax match SqlFunctionArgs /^Arguments:.*/
    syntax match SqlFunctionReturns /^Returns:.*/
    syntax match SqlFunctionExample /^Example:/
    syntax region SqlCodeBlock start=/^  / end=/$/
    
    highlight SqlFunctionName guifg=#8be9fd ctermfg=14
    highlight SqlFunctionCategory guifg=#50fa7b ctermfg=10
    highlight SqlFunctionArgs guifg=#f1fa8c ctermfg=11
    highlight SqlFunctionReturns guifg=#ff79c6 ctermfg=13
    highlight SqlFunctionExample guifg=#bd93f9 ctermfg=5
    highlight SqlCodeBlock guifg=#f8f8f2 ctermfg=7
  ]])
end

-- List all available SQL functions
function M.list_functions()
  -- Build command
  local cmd = M.config.command .. " --list-functions"
  
  -- Execute command
  local result = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error
  
  if exit_code ~= 0 then
    vim.notify("Failed to list functions", vim.log.levels.ERROR)
    return
  end
  
  -- Parse functions into categories
  local categories = {}
  local current_category = nil
  
  for line in result:gmatch("[^\n]+") do
    if line:match("^%s*$") then
      -- Skip empty lines
    elseif line:match("^Available SQL Functions:") then
      -- Skip header
    elseif line:match("^[A-Z].*:$") then
      -- Category header (e.g., "Aggregate Functions:", "Mathematical Functions:")
      current_category = line:gsub(":$", "")
      categories[current_category] = {}
    elseif current_category and line:match("^%s+") then
      -- Function in current category (indented lines)
      table.insert(categories[current_category], line)
    end
  end
  
  -- Create a buffer to show functions
  vim.cmd("new")
  local buf = vim.api.nvim_get_current_buf()
  vim.api.nvim_buf_set_name(buf, "[SQL Functions]")
  
  -- Build content
  local lines = {"# SQL CLI Functions", "", "Press <CR> on any function to see detailed help", ""}
  
  -- Keep categories in order they appear in output
  local category_order = {}
  for line in result:gmatch("[^\n]+") do
    if line:match("^[A-Z].*:$") then
      local category = line:gsub(":$", "")
      if categories[category] then
        table.insert(category_order, category)
      end
    end
  end
  
  -- Add functions by category in order
  for _, category in ipairs(category_order) do
    local functions = categories[category]
    if functions and #functions > 0 then
      table.insert(lines, "## " .. category)
      table.insert(lines, "")
      for _, func in ipairs(functions) do
        table.insert(lines, func)
      end
      table.insert(lines, "")
    end
  end
  
  -- Set buffer content
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  
  -- Set buffer options
  vim.bo[buf].modifiable = false
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].filetype = "markdown"
  
  -- Add keymap to get help for function under cursor
  vim.keymap.set("n", "<CR>", function()
    local word = vim.fn.expand("<cword>")
    if word and word ~= "" then
      M.show_function_help()
    end
  end, { buffer = buf, desc = "Show help for function under cursor" })
  
  vim.notify("Press <CR> on any function to see its help", vim.log.levels.INFO)
end

-- Search for SQL functions
function M.search_functions()
  vim.ui.input({ prompt = "Search functions: " }, function(query)
    if not query or query == "" then return end
    
    -- Build command
    local cmd = M.config.command .. " --list-functions"
    
    -- Execute command
    local result = vim.fn.system(cmd)
    local exit_code = vim.v.shell_error
    
    if exit_code ~= 0 then
      vim.notify("Failed to list functions", vim.log.levels.ERROR)
      return
    end
    
    -- Search for matching functions
    local matches = {}
    local query_upper = query:upper()
    
    for line in result:gmatch("[^\n]+") do
      if line:upper():find(query_upper) then
        table.insert(matches, line)
      end
    end
    
    if #matches == 0 then
      vim.notify("No functions found matching: " .. query, vim.log.levels.WARN)
      return
    end
    
    -- If only one match, show its help directly
    if #matches == 1 then
      local func_name = matches[1]:match("^%s*([A-Z_]+)")
      if func_name then
        local help_cmd = M.config.command .. " --function-help " .. vim.fn.shellescape(func_name)
        local help_result = vim.fn.system(help_cmd)
        if vim.v.shell_error == 0 then
          M.show_help_in_float(func_name, help_result)
          return
        end
      end
    end
    
    -- Show matches in a buffer
    vim.cmd("new")
    local buf = vim.api.nvim_get_current_buf()
    vim.api.nvim_buf_set_name(buf, "[Search Results: " .. query .. "]")
    
    local lines = {
      "# Function Search Results",
      "## Query: " .. query,
      "",
      "Found " .. #matches .. " matching functions:",
      ""
    }
    
    for _, match in ipairs(matches) do
      table.insert(lines, match)
    end
    
    vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
    
    -- Set buffer options
    vim.bo[buf].modifiable = false
    vim.bo[buf].buftype = "nofile"
    vim.bo[buf].filetype = "markdown"
    
    -- Add keymap to get help for function under cursor
    vim.keymap.set("n", "<CR>", function()
      local word = vim.fn.expand("<cword>")
      if word and word ~= "" then
        M.show_function_help()
      end
    end, { buffer = buf, desc = "Show help for function under cursor" })
  end)
end

-- Show table schema
function M.show_schema()
  if not state.data_file then
    -- Try to detect from current buffer
    local bufnr = vim.api.nvim_get_current_buf()
    local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    local buf_path = vim.api.nvim_buf_get_name(bufnr)
    local buf_dir = nil
    if buf_path and buf_path ~= "" then
      buf_dir = vim.fn.fnamemodify(buf_path, ":h")
    end
    state.data_file = M.detect_data_hint(lines, buf_dir)
    
    -- Check if current buffer is a CSV
    if not state.data_file and buf_path:match("%.csv$") then
      state.data_file = buf_path
    end
  end
  
  if not state.data_file then
    vim.notify("No data file set or detected", vim.log.levels.WARN)
    return
  end
  
  -- Run schema-json command
  local cmd = M.config.command .. " " .. vim.fn.shellescape(state.data_file) .. " --schema-json"
  local result = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error
  
  if exit_code ~= 0 then
    vim.notify("Failed to get schema: " .. result, vim.log.levels.ERROR)
    return
  end
  
  -- Parse JSON schema
  local ok, schema = pcall(vim.json.decode, result)
  if ok and schema and schema.columns then
    state.schema_columns = {}
    local display_lines = {}
    table.insert(display_lines, "Table: " .. schema.table)
    table.insert(display_lines, "Rows: " .. schema.rows)
    table.insert(display_lines, "Columns: " .. #schema.columns)
    table.insert(display_lines, "")
    table.insert(display_lines, "Column Information:")
    table.insert(display_lines, string.rep("-", 60))
    
    for i, col in ipairs(schema.columns) do
      table.insert(state.schema_columns, {
        name = col.name,
        type = col.type
      })
      
      local nullable_str = ""
      if col.nullable and col.null_percentage > 0 then
        nullable_str = string.format(" (%d%% NULL)", col.null_percentage)
      end
      
      table.insert(display_lines, string.format("  %3d. %-30s %-10s%s", 
        i, col.name, col.type, nullable_str))
    end
    
    -- Show in floating window
    M.show_help_in_float("Schema: " .. vim.fn.fnamemodify(state.data_file, ":t"), table.concat(display_lines, "\n"))
  else
    vim.notify("Failed to parse schema JSON", vim.log.levels.ERROR)
  end
end

-- Expand SELECT * to column names
function M.expand_star()
  -- First ensure we have schema information
  if not state.data_file then
    -- Try to detect from current buffer
    local bufnr = vim.api.nvim_get_current_buf()
    local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    local buf_path = vim.api.nvim_buf_get_name(bufnr)
    local buf_dir = nil
    if buf_path and buf_path ~= "" then
      buf_dir = vim.fn.fnamemodify(buf_path, ":h")
    end
    state.data_file = M.detect_data_hint(lines, buf_dir)
    
    -- Check if current buffer is a CSV
    if not state.data_file and buf_path:match("%.csv$") then
      state.data_file = buf_path
    end
  end
  
  if not state.data_file then
    vim.notify("No data file set. Use :SqlCliSetData or open a CSV file", vim.log.levels.WARN)
    return
  end
  
  -- Get schema if not already loaded
  if not state.schema_columns or #state.schema_columns == 0 then
    local cmd = M.config.command .. " " .. vim.fn.shellescape(state.data_file) .. " --schema-json"
    local result = vim.fn.system(cmd)
    local exit_code = vim.v.shell_error
    
    if exit_code ~= 0 then
      vim.notify("Failed to get schema: " .. result, vim.log.levels.ERROR)
      return
    end
    
    -- Parse JSON schema
    local ok, schema = pcall(vim.json.decode, result)
    if ok and schema and schema.columns then
      state.schema_columns = {}
      for _, col in ipairs(schema.columns) do
        table.insert(state.schema_columns, {
          name = col.name,
          type = col.type
        })
      end
    else
      vim.notify("Failed to parse schema", vim.log.levels.ERROR)
      return
    end
  end
  
  -- Get current line
  local line = vim.api.nvim_get_current_line()
  local cursor_pos = vim.api.nvim_win_get_cursor(0)
  
  -- Check if line contains SELECT *
  local select_pattern = "SELECT%s+%*"
  local select_start, select_end = line:find(select_pattern)
  
  if not select_start then
    -- Try case-insensitive match
    local line_lower = line:lower()
    select_start, select_end = line_lower:find("select%s+%*")
    
    if not select_start then
      vim.notify("No SELECT * found on current line", vim.log.levels.INFO)
      return
    end
  end
  
  -- Build column list
  local column_names = {}
  for _, col in ipairs(state.schema_columns) do
    -- Quote column names that contain special characters or spaces
    if col.name:match("[%-%. ]") then
      table.insert(column_names, '"' .. col.name .. '"')
    else
      table.insert(column_names, col.name)
    end
  end
  
  -- Join columns with appropriate formatting
  local expanded_inline = "SELECT " .. table.concat(column_names, ", ")
  
  -- Check if there's more after the * (like FROM clause)
  local after_star = line:sub(select_end + 1)
  local from_clause = ""
  if after_star:match("^%s+FROM") or after_star:match("^%s+from") then
    from_clause = " " .. after_star:match("^%s+(.*)")
  end
  
  -- Determine if we should use multi-line format (if too many columns or too long)
  local total_length = #expanded_inline + #from_clause
  local use_multiline = #column_names > 5 or total_length > 100
  
  if use_multiline then
    -- Multi-line format with nice indentation
    local lines = {"SELECT"}
    for i, col in ipairs(column_names) do
      local prefix = i == 1 and "    " or "  , "
      table.insert(lines, prefix .. col)
    end
    if from_clause ~= "" then
      table.insert(lines, from_clause:match("^%s*(.*)"))
    end
    
    -- Get current line number
    local row = cursor_pos[1]
    
    -- Delete current line and insert new lines
    vim.api.nvim_buf_set_lines(0, row - 1, row, false, lines)
    
    vim.notify("Expanded * to " .. #column_names .. " columns (multi-line format)", vim.log.levels.INFO)
  else
    -- Single line format
    local expanded = expanded_inline .. from_clause
    vim.api.nvim_set_current_line(expanded)
    
    vim.notify("Expanded * to " .. #column_names .. " columns", vim.log.levels.INFO)
  end
end

-- Expand SELECT * in visual selection
function M.expand_star_visual()
  -- Get visual selection range
  local start_pos = vim.fn.getpos("'<")
  local end_pos = vim.fn.getpos("'>")
  local start_line = start_pos[2]
  local end_line = end_pos[2]
  
  -- Process each line in the selection
  local expanded_count = 0
  for line_num = start_line, end_line do
    -- Set cursor to this line
    vim.api.nvim_win_set_cursor(0, {line_num, 0})
    
    -- Get the line
    local line = vim.api.nvim_buf_get_lines(0, line_num - 1, line_num, false)[1]
    
    -- Check if it contains SELECT *
    if line:match("SELECT%s+%*") or line:lower():match("select%s+%*") then
      -- Call the normal expand function
      M.expand_star()
      expanded_count = expanded_count + 1
      
      -- Adjust end_line if we inserted multiple lines
      local new_line_count = vim.api.nvim_buf_line_count(0)
      local lines_added = new_line_count - (end_line - start_line + 1)
      if lines_added > 0 then
        end_line = end_line + lines_added
      end
    end
  end
  
  if expanded_count > 0 then
    vim.notify("Expanded " .. expanded_count .. " SELECT * statements", vim.log.levels.INFO)
  else
    vim.notify("No SELECT * found in selection", vim.log.levels.INFO)
  end
end

-- Show column help at cursor
function M.show_column_help()
  -- Get word under cursor
  local word = vim.fn.expand("<cword>")
  
  if word == "" then
    vim.notify("No word under cursor", vim.log.levels.WARN)
    return
  end
  
  -- First, ensure we have schema information
  if not state.schema_columns or #state.schema_columns == 0 then
    -- Try to load schema
    if state.data_file then
      local cmd = M.config.command .. " " .. vim.fn.shellescape(state.data_file) .. " --schema"
      local result = vim.fn.system(cmd)
      
      if vim.v.shell_error == 0 then
        state.schema_columns = {}
        for line in result:gmatch("[^\n]+") do
          local col_name = line:match("%d+%.%s+([%w_]+)")
          if col_name then
            table.insert(state.schema_columns, col_name)
          end
        end
      end
    end
  end
  
  -- Check if word is a column
  if state.schema_columns then
    local found_column = nil
    local word_lower = word:lower()
    
    for _, col in ipairs(state.schema_columns) do
      if col:lower() == word_lower then
        found_column = col
        break
      end
    end
    
    if found_column then
      -- Get schema info and highlight the specific column
      if state.data_file then
        local cmd = M.config.command .. " " .. vim.fn.shellescape(state.data_file) .. " --schema"
        local result = vim.fn.system(cmd)
        
        if vim.v.shell_error == 0 then
          -- Extract just the column info
          local column_info = {}
          local found = false
          for line in result:gmatch("[^\n]+") do
            if line:match(found_column) then
              table.insert(column_info, line)
              found = true
            elseif found and line:match("^%s*%d+%.") then
              -- Stop at next column
              break
            elseif found then
              table.insert(column_info, line)
            end
          end
          
          if #column_info > 0 then
            M.show_help_in_float("Column: " .. found_column, table.concat(column_info, "\n"))
            return
          end
        end
      end
    else
      -- Try function help as fallback
      M.show_function_help()
    end
  else
    -- No schema loaded, try function help
    M.show_function_help()
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

-- Get column info at cursor (smart detection)
function M.get_column_at_cursor()
  local word = vim.fn.expand('<cword>')
  if not word or word == "" then
    vim.notify("No word under cursor", vim.log.levels.WARN)
    return
  end
  
  -- First try column info from cached schema
  if state.schema_columns then
    for _, col in ipairs(state.schema_columns) do
      if col.name:lower() == word:lower() then
        local content = string.format("Column: %s\nType: %s", col.name, col.type or "Unknown")
        M.show_help_in_float(col.name, content)
        return
      end
    end
  end
  
  -- If not a column, try function help
  M.show_function_help()
end

-- Trigger column-specific completion with better UI
function M.trigger_column_completion()
  -- Get current word under cursor
  local line = vim.api.nvim_get_current_line()
  local col = vim.api.nvim_win_get_cursor(0)[2]
  
  -- Find word start
  local word_start = col
  while word_start > 0 and line:sub(word_start, word_start):match('[%w_]') do
    word_start = word_start - 1
  end
  
  local prefix = line:sub(word_start + 1, col)
  
  -- Build completion items
  local items = {}
  
  -- Add columns from schema if available
  if state.schema_columns then
    for _, column in ipairs(state.schema_columns) do
      if prefix == "" or column.name:lower():sub(1, #prefix) == prefix:lower() then
        table.insert(items, {
          word = column.name,
          abbr = column.name,
          menu = '[' .. column.type .. ']',
          kind = 'Column',
          info = 'Column: ' .. column.name .. '\nType: ' .. column.type
        })
      end
    end
  end
  
  -- If we have items, show them using built-in completion
  if #items > 0 then
    -- Use vim.fn.complete() to show the menu
    vim.fn.complete(word_start + 1, items)
  else
    -- Fall back to regular omnifunc completion
    vim.api.nvim_feedkeys(vim.api.nvim_replace_termcodes('<C-x><C-o>', true, false, true), 'n', false)
  end
end

-- Column name completion function for omnifunc
function M.complete_columns(findstart, base)
  if findstart == 1 then
    -- Find the start of the word to complete
    local line = vim.api.nvim_get_current_line()
    local col = vim.api.nvim_win_get_cursor(0)[2]  -- 1-based column
    
    -- Move backwards to find word start
    local start_col = col
    while start_col > 0 do
      local char = line:sub(start_col, start_col)
      if not char:match('[%w_]') then
        break
      end
      start_col = start_col - 1
    end
    
    -- Debug: Show what we found (uncomment for debugging)
    -- vim.notify(string.format("Completion: col=%d, start_col=%d, text='%s'", col, start_col, line:sub(start_col+1, col)), vim.log.levels.INFO)
    
    -- Return 0-based column for vim (start_col is now at the character before the word)
    return start_col
  else
    -- Debug: Show base string (uncomment for debugging)
    -- vim.notify(string.format("Completing base='%s', schema_loaded=%s", base, state.schema_columns and "yes" or "no"), vim.log.levels.INFO)
    -- Return completions based on the partial word
    local completions = {}
    
    -- Add column names from cached schema
    if state.schema_columns then
      for _, col in ipairs(state.schema_columns) do
        -- Use simple pattern matching without vim.pesc to avoid issues
        local base_lower = base:lower()
        local col_lower = col.name:lower()
        if base == "" or col_lower:sub(1, #base_lower) == base_lower then
          table.insert(completions, {
            word = col.name,
            menu = '[Col: ' .. (col.type or "?") .. ']',
            kind = 'v',  -- Variable kind
            info = string.format("Column: %s\nType: %s", col.name, col.type or "Unknown")
          })
        end
      end
    end
    
    -- Add common SQL functions
    local sql_functions = {
      {name = "COUNT", desc = "Count rows"},
      {name = "SUM", desc = "Sum values"},
      {name = "AVG", desc = "Average values"},
      {name = "MIN", desc = "Minimum value"},
      {name = "MAX", desc = "Maximum value"},
      {name = "ROUND", desc = "Round number"},
      {name = "ABS", desc = "Absolute value"},
      {name = "UPPER", desc = "Uppercase string"},
      {name = "LOWER", desc = "Lowercase string"},
      {name = "LENGTH", desc = "String length"},
      {name = "TRIM", desc = "Trim whitespace"},
      {name = "SUBSTR", desc = "Substring"},
      {name = "REPLACE", desc = "Replace string"},
      {name = "CONCAT", desc = "Concatenate strings"},
      {name = "COALESCE", desc = "First non-null value"},
      {name = "CAST", desc = "Type conversion"},
      {name = "DATE", desc = "Extract date"},
      {name = "NOW", desc = "Current timestamp"},
      {name = "RANK", desc = "Window rank"},
      {name = "ROW_NUMBER", desc = "Row number"},
      {name = "LEAD", desc = "Next row value"},
      {name = "LAG", desc = "Previous row value"},
      {name = "CONVERT", desc = "Unit conversion"},
      {name = "RANGE", desc = "Generate range"},
      {name = "RANDOM", desc = "Random number"},
      {name = "SQRT", desc = "Square root"},
      {name = "POW", desc = "Power"},
      {name = "LOG", desc = "Logarithm"},
      {name = "EXP", desc = "Exponential"},
      {name = "SIN", desc = "Sine"},
      {name = "COS", desc = "Cosine"},
      {name = "TAN", desc = "Tangent"},
    }
    
    for _, func in ipairs(sql_functions) do
      local base_lower = base:lower()
      local func_lower = func.name:lower()
      if base == "" or func_lower:sub(1, #base_lower) == base_lower then
        table.insert(completions, {
          word = func.name .. '(',
          menu = '[Func]',
          kind = 'f',  -- Function kind
          info = func.desc
        })
      end
    end
    
    -- Add SQL keywords
    local keywords = {
      'SELECT', 'FROM', 'WHERE', 'GROUP', 'BY', 'ORDER', 'HAVING',
      'LIMIT', 'OFFSET', 'AS', 'WITH', 'DISTINCT', 'ALL', 'AND', 'OR',
      'NOT', 'IN', 'EXISTS', 'BETWEEN', 'LIKE', 'IS', 'NULL', 'ASC', 'DESC',
      'CASE', 'WHEN', 'THEN', 'ELSE', 'END', 'OVER', 'PARTITION',
      'INNER', 'LEFT', 'RIGHT', 'OUTER', 'JOIN', 'ON', 'USING'
    }
    
    for _, kw in ipairs(keywords) do
      local base_lower = base:lower()
      local kw_lower = kw:lower()
      if base == "" or kw_lower:sub(1, #base_lower) == base_lower then
        table.insert(completions, {
          word = kw,
          menu = '[SQL]',
          kind = 'k',  -- Keyword kind
        })
      end
    end
    
    -- Sort completions: columns first, then functions, then keywords
    table.sort(completions, function(a, b)
      if a.kind ~= b.kind then
        local order = {v = 1, f = 2, k = 3}
        return (order[a.kind] or 4) < (order[b.kind] or 4)
      end
      return a.word < b.word
    end)
    
    return completions
  end
end

-- Enable column completion for SQL files
function M.setup_completion()
  -- Set omnifunc for SQL files
  vim.api.nvim_create_autocmd("FileType", {
    pattern = "sql",
    callback = function()
      vim.bo.omnifunc = 'v:lua.require("sql-cli").complete_columns'
      -- Set completion options - menuone shows menu even with single match
      -- noselect doesn't auto-select first item
      vim.opt_local.completeopt = 'menu,menuone,noselect'
      
      -- Add instructions in buffer-local variable
      vim.b.sql_cli_completion = true
      
      -- Map <C-Space> to trigger omni completion in insert mode
      vim.keymap.set('i', '<C-Space>', '<C-x><C-o>', 
        { buffer = true, desc = 'Trigger SQL completion' })
      
      -- Also map <C-S-Space> for compatibility
      vim.keymap.set('i', '<C-S-Space>', '<C-x><C-o>', 
        { buffer = true, desc = 'Trigger SQL completion' })
      
      -- Map M-; (Alt+semicolon) to trigger SQL column completion
      vim.keymap.set('i', '<M-;>', function()
        -- Trigger column-specific completion
        M.trigger_column_completion()
      end, { buffer = true, desc = 'Trigger SQL column completion' })
      
      -- Also map Alt+. as an alternative
      vim.keymap.set('i', '<M-.>', function()
        M.trigger_column_completion()
      end, { buffer = true, desc = 'Trigger SQL column completion' })
      
      -- Map Tab to accept completion when popup menu is visible
      vim.keymap.set('i', '<Tab>', function()
        if vim.fn.pumvisible() == 1 then
          return vim.api.nvim_replace_termcodes('<C-n>', true, false, true)
        else
          return vim.api.nvim_replace_termcodes('<Tab>', true, false, true)
        end
      end, { buffer = true, expr = true, desc = 'Tab through completions' })
      
      -- Map Shift-Tab to go backwards in completion
      vim.keymap.set('i', '<S-Tab>', function()
        if vim.fn.pumvisible() == 1 then
          return vim.api.nvim_replace_termcodes('<C-p>', true, false, true)
        else
          return vim.api.nvim_replace_termcodes('<S-Tab>', true, false, true)
        end
      end, { buffer = true, expr = true, desc = 'Reverse tab through completions' })
      
      -- Map Enter to accept selected completion
      vim.keymap.set('i', '<CR>', function()
        if vim.fn.pumvisible() == 1 then
          return vim.api.nvim_replace_termcodes('<C-y>', true, false, true)
        else
          return vim.api.nvim_replace_termcodes('<CR>', true, false, true)
        end
      end, { buffer = true, expr = true, desc = 'Accept completion' })
      
      -- Map numbers 1-9 to quickly select completion items
      for i = 1, 9 do
        vim.keymap.set('i', tostring(i), function()
          if vim.fn.pumvisible() == 1 then
            -- Select the i-th item and accept it
            local keys = string.rep(vim.api.nvim_replace_termcodes('<C-n>', true, false, true), i - 1)
            keys = keys .. vim.api.nvim_replace_termcodes('<C-y>', true, false, true)
            return keys
          else
            return tostring(i)
          end
        end, { buffer = true, expr = true, desc = 'Quick select completion item ' .. i })
      end
      
      -- Check for data hints and load schema if found
      local bufnr = vim.api.nvim_get_current_buf()
      local lines = vim.api.nvim_buf_get_lines(bufnr, 0, math.min(20, vim.api.nvim_buf_line_count(bufnr)), false)
      local buf_path = vim.api.nvim_buf_get_name(bufnr)
      local buf_dir = nil
      if buf_path and buf_path ~= "" then
        buf_dir = vim.fn.fnamemodify(buf_path, ":h")
      end
      
      local detected_file = M.detect_data_hint(lines, buf_dir)
      if detected_file and not state.data_file then
        state.data_file = detected_file
        M.load_schema_for_completion()
        vim.notify("SQL CLI: Auto-detected data file from hint", vim.log.levels.INFO)
      end
      
      -- Show a hint about completion once per session
      if not state.completion_hint_shown then
        vim.defer_fn(function()
          vim.notify("SQL completion enabled. Use <C-Space> or <M-;> to trigger.", vim.log.levels.INFO)
          state.completion_hint_shown = true
        end, 100)
      end
    end
  })
  
  -- Also enable for buffers that are already SQL
  if vim.bo.filetype == "sql" then
    vim.bo.omnifunc = 'v:lua.require("sql-cli").complete_columns'
    vim.opt_local.completeopt = 'menu,menuone,noselect'
    
    -- Map <C-Space> for current buffer
    vim.keymap.set('i', '<C-Space>', '<C-x><C-o>', 
      { buffer = true, desc = 'Trigger SQL completion' })
    vim.keymap.set('i', '<C-S-Space>', '<C-x><C-o>', 
      { buffer = true, desc = 'Trigger SQL completion' })
    vim.keymap.set('i', '<M-;>', function()
      M.trigger_column_completion()
    end, { buffer = true, desc = 'Trigger SQL column completion' })
    vim.keymap.set('i', '<M-.>', function()
      M.trigger_column_completion()
    end, { buffer = true, desc = 'Trigger SQL column completion' })
    
    -- Add Tab/Enter/Number selection mappings
    vim.keymap.set('i', '<Tab>', function()
      if vim.fn.pumvisible() == 1 then
        return vim.api.nvim_replace_termcodes('<C-n>', true, false, true)
      else
        return vim.api.nvim_replace_termcodes('<Tab>', true, false, true)
      end
    end, { buffer = true, expr = true, desc = 'Tab through completions' })
    
    vim.keymap.set('i', '<S-Tab>', function()
      if vim.fn.pumvisible() == 1 then
        return vim.api.nvim_replace_termcodes('<C-p>', true, false, true)
      else
        return vim.api.nvim_replace_termcodes('<S-Tab>', true, false, true)
      end
    end, { buffer = true, expr = true, desc = 'Reverse tab through completions' })
    
    vim.keymap.set('i', '<CR>', function()
      if vim.fn.pumvisible() == 1 then
        return vim.api.nvim_replace_termcodes('<C-y>', true, false, true)
      else
        return vim.api.nvim_replace_termcodes('<CR>', true, false, true)
      end
    end, { buffer = true, expr = true, desc = 'Accept completion' })
    
    for i = 1, 9 do
      vim.keymap.set('i', tostring(i), function()
        if vim.fn.pumvisible() == 1 then
          local keys = string.rep(vim.api.nvim_replace_termcodes('<C-n>', true, false, true), i - 1)
          keys = keys .. vim.api.nvim_replace_termcodes('<C-y>', true, false, true)
          return keys
        else
          return tostring(i)
        end
      end, { buffer = true, expr = true, desc = 'Quick select completion item ' .. i })
    end
  end
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