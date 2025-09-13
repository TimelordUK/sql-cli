-- SQL CLI Results Handling Module
-- Functions for managing query results and expanding SELECT * statements

local utils = require('sql-cli.utils')
local state = require('sql-cli.state')

local M = {}

-- Save query results to CSV file
function M.save_results_csv()
  local last_results = state.get_last_results()
  if not last_results or #last_results == 0 then
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

    for _, line in ipairs(last_results) do
      file:write(line .. "\n")
    end
    file:close()

    vim.notify("Results saved to: " .. filename, vim.log.levels.INFO)
  end)
end

-- Open results in new buffer
function M.results_to_buffer()
  local last_results = state.get_last_results()
  if not last_results or #last_results == 0 then
    vim.notify("No results to display", vim.log.levels.WARN)
    return
  end

  -- Create new buffer
  vim.cmd("new")
  local buf = vim.api.nvim_get_current_buf()

  -- Set buffer content
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, last_results)

  -- Set buffer options
  vim.bo[buf].filetype = "csv"
  vim.bo[buf].modified = false
  vim.api.nvim_buf_set_name(buf, "[SQL Results]")

  vim.notify("Results opened in new buffer", vim.log.levels.INFO)
end

-- Expand SELECT * to column names at cursor
function M.expand_star_columns(config, load_schema_callback)
  -- First ensure we have schema information
  local data_file = state.get_data_file()
  if not data_file then
    -- Try to detect from current buffer
    local bufnr = vim.api.nvim_get_current_buf()
    local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    local buf_path = vim.api.nvim_buf_get_name(bufnr)
    local buf_dir = nil
    if buf_path and buf_path ~= "" then
      buf_dir = vim.fn.fnamemodify(buf_path, ":h")
    end
    data_file = utils.detect_data_hint(lines, buf_dir)
    if data_file then
      state.set_data_file(data_file)
    end

    -- Check if current buffer is a CSV
    if not data_file and buf_path:match("%.csv$") then
      data_file = buf_path
      state.set_data_file(data_file)
    end
  end

  if not data_file then
    vim.notify("No data file set. Use :SqlCliSetData or open a CSV file", vim.log.levels.WARN)
    return
  end

  -- Get schema if not already loaded
  local schema_columns = state.schema_columns
  if not schema_columns or #schema_columns == 0 then
    local command_path, err = utils.get_command_path(config.command)
    if not command_path then
      vim.notify(err, vim.log.levels.ERROR)
      return
    end

    local cmd = command_path .. " " .. vim.fn.shellescape(data_file) .. " --schema-json"
    local result = vim.fn.system(cmd)
    local exit_code = vim.v.shell_error

    if exit_code ~= 0 then
      vim.notify("Failed to get schema: " .. result, vim.log.levels.ERROR)
      return
    end

    -- Parse JSON schema
    local ok, schema = pcall(vim.json.decode, result)
    if ok and schema and schema.columns then
      schema_columns = {}
      for _, col in ipairs(schema.columns) do
        table.insert(schema_columns, {
          name = col.name,
          type = col.type
        })
      end
      state.schema_columns = schema_columns
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
  for _, col in ipairs(schema_columns) do
    local col_name = type(col) == "table" and col.name or col
    -- Quote column names that contain special characters or spaces
    if col_name:match("[%-%. ]") then
      table.insert(column_names, '"' .. col_name .. '"')
    else
      table.insert(column_names, col_name)
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
function M.expand_star_visual(config, load_schema_callback)
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
      M.expand_star_columns(config, load_schema_callback)
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

return M