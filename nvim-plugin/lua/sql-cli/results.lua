-- SQL CLI Results Handling Module
-- Functions for managing query results and expanding SELECT * statements

local utils = require('sql-cli.utils')
local Job = require('plenary.job')

local M = {}

-- Save query results to CSV file
function M.save_results_csv(filename, state)
  local last_results = state:get_last_results()
  if not last_results or #last_results == 0 then
    vim.notify("No results to save", vim.log.levels.WARN)
    return
  end

  if not filename or filename == "" then
    -- Use file picker
    vim.ui.input({ prompt = "Save results to: ", completion = "file", default = "results.csv" }, function(input_filename)
      if not input_filename then return end

      -- Ensure .csv extension
      if not input_filename:match("%.csv$") then
        input_filename = input_filename .. ".csv"
      end

      -- Write results to file
      local file = io.open(input_filename, "w")
      if not file then
        vim.notify("Failed to create file: " .. input_filename, vim.log.levels.ERROR)
        return
      end

      for _, line in ipairs(last_results) do
        file:write(line .. "\n")
      end
      file:close()

      vim.notify("Results saved to: " .. input_filename, vim.log.levels.INFO)
    end)
  else
    -- Use provided filename
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
  end
end

-- Open results in new buffer
function M.results_to_buffer(state)
  local last_results = state:get_last_results()
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
function M.expand_star_columns(config, state)
  -- First ensure we have schema information
  local data_file = state:get_data_file()
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
      state:set_data_file(data_file)
    end

    -- Check if current buffer is a CSV
    if not data_file and buf_path:match("%.csv$") then
      data_file = buf_path
      state:set_data_file(data_file)
    end
  end

  if not data_file then
    vim.notify("No data file set. Use :SqlCliSetData or open a CSV file", vim.log.levels.WARN)
    return
  end

  -- Get schema if not already loaded
  local schema_columns = state:get_schema_columns()
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
      state:set_schema_columns(schema_columns)
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
function M.expand_star_visual(config, state)
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
      M.expand_star_columns(config, state)
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

-- Get query context for smart * expansion
-- Returns the complete query including CTEs, subqueries, etc.
function M.get_query_for_expansion()
  local cursor = vim.api.nvim_win_get_cursor(0)
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)

  -- Find query boundaries (GO statements or CTEs)
  local start_line = 1
  local end_line = #lines

  -- Look backward for WITH or previous GO
  for i = cursor[1] - 1, 1, -1 do
    if lines[i]:upper():match('^GO%s*$') or lines[i]:upper():match('^GO%s*;%s*$') then
      start_line = i + 1
      break
    elseif lines[i]:upper():match('^%s*WITH%s+') then
      start_line = i
      break
    end
  end

  -- Look forward for GO
  for i = cursor[1], #lines do
    if lines[i]:upper():match('^GO%s*$') or lines[i]:upper():match('^GO%s*;%s*$') then
      end_line = i - 1
      break
    end
  end

  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end

  return table.concat(query_lines, '\n')
end

-- Replace * with column list
function M.replace_star_with_columns(columns)
  local line = vim.api.nvim_get_current_line()
  local cursor_pos = vim.api.nvim_win_get_cursor(0)

  -- Find SELECT *
  local select_pattern = "SELECT%s+%*"
  local select_start, select_end = line:find(select_pattern)

  if not select_start then
    local line_lower = line:lower()
    select_start, select_end = line_lower:find("select%s+%*")
  end

  if not select_start then
    vim.notify("No SELECT * found on current line", vim.log.levels.WARN)
    return
  end

  -- Quote columns if needed
  local column_names = {}
  for _, col in ipairs(columns) do
    if col:match("[%-%. ]") then
      table.insert(column_names, '"' .. col .. '"')
    else
      table.insert(column_names, col)
    end
  end

  -- Build expanded SELECT
  local after_star = line:sub(select_end + 1)
  local total_length = #("SELECT " .. table.concat(column_names, ", ")) + #after_star
  local use_multiline = #column_names > 5 or total_length > 100

  if use_multiline then
    local lines_to_insert = {"SELECT"}
    for i, col in ipairs(column_names) do
      local prefix = i == 1 and "    " or "  , "
      table.insert(lines_to_insert, prefix .. col)
    end

    -- Add FROM clause if present
    if after_star:match("^%s+FROM") or after_star:match("^%s+from") then
      table.insert(lines_to_insert, after_star:match("^%s*(.*)"))
    end

    local row = cursor_pos[1]
    vim.api.nvim_buf_set_lines(0, row - 1, row, false, lines_to_insert)

    vim.notify("Expanded * to " .. #column_names .. " columns (smart expansion)",
               vim.log.levels.INFO)
  else
    local expanded = "SELECT " .. table.concat(column_names, ", ") .. after_star
    vim.api.nvim_set_current_line(expanded)

    vim.notify("Expanded * to " .. #column_names .. " columns", vim.log.levels.INFO)
  end
end

-- Add column hint comment for nvim completion
function M.add_column_hint_comment(columns, config)
  -- Check if user wants this feature
  if not config or not config.smart_expansion or not config.smart_expansion.auto_insert_column_hints then
    return
  end

  -- Create hint comment
  local hint = "-- Columns: " .. table.concat(columns, ", ")

  -- Insert at top of buffer (after data file hint if present)
  local lines = vim.api.nvim_buf_get_lines(0, 0, 5, false)
  local insert_line = 0

  -- Skip past #! hint if present
  for i, line_text in ipairs(lines) do
    if line_text:match("^%s*$") or line_text:match("^%-%-%s*#!") then
      insert_line = i - 1
    else
      break
    end
  end

  -- Check if hint already exists
  local existing_hint = false
  for i = 0, math.min(insert_line + 2, #lines - 1) do
    local line_text = lines[i + 1]  -- Lua tables are 1-indexed
    if line_text and line_text:match("^%-%- Columns:") then
      existing_hint = true
      -- Update existing hint
      vim.api.nvim_buf_set_lines(0, i, i + 1, false, {hint})
      break
    end
  end

  if not existing_hint then
    vim.api.nvim_buf_set_lines(0, insert_line, insert_line, false, {hint, ""})
  end

  vim.notify("Column hint added (use Ctrl+N for completion)", vim.log.levels.INFO)
end

-- Smart * expansion with query execution
function M.expand_star_smart(config, state)
  if not config.smart_expansion or not config.smart_expansion.enabled then
    -- Fall back to old expansion
    M.expand_star_columns(config, state)
    return
  end

  -- Get current query (including CTEs, subqueries, etc.)
  local query = M.get_query_for_expansion()

  if not query or query == "" then
    vim.notify("Could not determine query context", vim.log.levels.WARN)
    return
  end

  -- Check if the query contains SELECT *
  if not query:match("SELECT%s+%*") and not query:lower():match("select%s+%*") then
    vim.notify("No SELECT * found in query", vim.log.levels.WARN)
    return
  end

  -- Execute a preview query to get column names (LIMIT 1 to get schema)
  -- Note: LIMIT 0 returns [] with no schema, so we need at least 1 row
  local preview_query = query:gsub("LIMIT%s+%d+", ""):gsub("limit%s+%d+", "") .. " LIMIT 1"

  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    vim.notify(err, vim.log.levels.ERROR)
    return
  end

  -- Show progress notification
  vim.notify("Executing query to get columns...", vim.log.levels.INFO)

  -- Execute with -o json to get results including schema
  Job:new({
    command = command_path,
    args = {
      '-q', preview_query,
      '-o', 'json'
    },
    on_exit = function(j, return_val)
      vim.schedule(function()
        if return_val == 0 then
          local result = table.concat(j:result(), '\n')

          -- Filter out comment lines (starting with #)
          local json_lines = {}
          for line in result:gmatch("[^\r\n]+") do
            if not line:match("^%s*#") then
              table.insert(json_lines, line)
            end
          end
          local json_str = table.concat(json_lines, "\n")

          local ok, data = pcall(vim.json.decode, json_str)

          if ok and data then
            local columns = {}

            -- Handle two formats:
            -- 1. Array of objects: [{"col1": val, "col2": val}]
            -- 2. Object with columns: {columns: [{name: "col1"}, ...]}

            if type(data) == "table" then
              if data.columns then
                -- Format 2: {columns: [...]}
                for _, col in ipairs(data.columns) do
                  local col_name = type(col) == "table" and col.name or col
                  table.insert(columns, col_name)
                end
              elseif #data > 0 and type(data[1]) == "table" then
                -- Format 1: [{col1: val, col2: val}, ...]
                -- Extract keys from first object
                for key, _ in pairs(data[1]) do
                  table.insert(columns, key)
                end
                -- Sort columns alphabetically for consistency
                table.sort(columns)
              end
            end

            if #columns > 0 then
              -- Store columns in state
              state:set_last_query_columns(columns)

              -- Perform the replacement
              M.replace_star_with_columns(columns)

              -- Add column hint comment
              M.add_column_hint_comment(columns, config)
            else
              vim.notify("No columns found in query result", vim.log.levels.WARN)
            end
          else
            vim.notify("Could not parse query result", vim.log.levels.ERROR)
            -- Fall back to static expansion if enabled
            if config.smart_expansion.fallback_to_static then
              vim.notify("Falling back to static schema...", vim.log.levels.INFO)
              M.expand_star_columns(config, state)
            end
          end
        else
          local err_msg = table.concat(j:stderr_result(), '\n')
          vim.notify("Query execution failed: " .. err_msg, vim.log.levels.ERROR)

          -- Fall back to static expansion if enabled
          if config.smart_expansion.fallback_to_static then
            vim.notify("Falling back to static schema...", vim.log.levels.INFO)
            M.expand_star_columns(config, state)
          end
        end
      end)
    end
  }):start()
end

-- Sync columns from results to query buffer
function M.sync_columns_to_query_buffer(query_bufnr, columns, config)
  if not config or not config.smart_expansion or not config.smart_expansion.auto_sync_column_hints then
    return
  end

  if not query_bufnr or not vim.api.nvim_buf_is_valid(query_bufnr) then
    return
  end

  -- Update column hint in the query buffer
  vim.api.nvim_buf_call(query_bufnr, function()
    M.add_column_hint_comment(columns, config)
  end)
end

return M