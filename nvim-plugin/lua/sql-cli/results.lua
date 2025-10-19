-- SQL CLI Results Handling Module
-- Functions for managing query results and expanding SELECT * statements

local utils = require('sql-cli.utils')
local Job = require('plenary.job')

local M = {}

-- Get logger (lazy load to avoid circular dependencies)
local logger = nil
local function get_logger()
  if not logger then
    local ok, sql_cli = pcall(require, 'sql-cli')
    if ok and sql_cli.logger then
      logger = sql_cli.logger
    end
  end
  return logger
end

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
  local log = get_logger()
  local cursor = vim.api.nvim_win_get_cursor(0)
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)

  if log then
    log.debug('expand_star', string.format('get_query_for_expansion: cursor at line %d, total lines: %d',
      cursor[1], #lines))
  end

  -- Find query boundaries (GO statements or CTEs)
  local start_line = 1
  local end_line = #lines

  -- Look backward for WITH or previous GO
  for i = cursor[1] - 1, 1, -1 do
    if lines[i]:upper():match('^GO%s*$') or lines[i]:upper():match('^GO%s*;%s*$') then
      start_line = i + 1
      if log then
        log.debug('expand_star', string.format('Found GO at line %d, start_line set to %d', i, start_line))
      end
      break
    elseif lines[i]:upper():match('^%s*WITH%s+') then
      start_line = i
      if log then
        log.debug('expand_star', string.format('Found WITH at line %d, start_line set to %d', i, start_line))
      end
      break
    end
  end

  -- Look forward for GO
  for i = cursor[1], #lines do
    if lines[i]:upper():match('^GO%s*$') or lines[i]:upper():match('^GO%s*;%s*$') then
      end_line = i - 1
      if log then
        log.debug('expand_star', string.format('Found GO at line %d, end_line set to %d', i, end_line))
      end
      break
    end
  end

  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end

  local query = table.concat(query_lines, '\n')

  -- Extract and expand file-level variables (for @SET directives)
  local executor = require('sql-cli.executor')
  local file_vars = executor.extract_file_variables(lines)
  query = executor.expand_env_variables(query, file_vars)

  if log then
    log.info('expand_star', string.format('Extracted query from lines %d-%d (%d lines)',
      start_line, end_line, #query_lines))
    log.debug('expand_star', 'Query text: ' .. query:sub(1, 200) .. (  #query > 200 and '...' or ''))
  end

  return query
end

-- Replace * with column list
function M.replace_star_with_columns(columns)
  local log = get_logger()

  if log then
    log.info('expand_star', string.format('replace_star_with_columns called with %d columns', #columns))
  end

  local cursor_pos = vim.api.nvim_win_get_cursor(0)
  local current_line_num = cursor_pos[1]

  if log then
    log.debug('expand_star', string.format('Current cursor at line %d', current_line_num))
  end

  -- First, try to find SELECT * on current line
  local line = vim.api.nvim_get_current_line()
  local select_pattern = "SELECT%s+%*"
  local select_start, select_end = line:find(select_pattern)

  if not select_start then
    local line_lower = line:lower()
    select_start, select_end = line_lower:find("select%s+%*")
  end

  -- If not on current line, search nearby lines for standalone *
  local target_line_num = current_line_num
  local is_standalone_star = false

  if not select_start then
    if log then
      log.debug('expand_star', 'SELECT * not on current line, searching nearby lines')
    end

    -- Check if current line is just "*" or contains "*"
    if line:match("^%s*%*%s*$") then
      -- Current line is just "*"
      is_standalone_star = true
      if log then
        log.info('expand_star', 'Found standalone * on current line')
      end
      -- Look backwards for SELECT
      local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
      for i = current_line_num - 1, math.max(1, current_line_num - 5), -1 do
        if lines[i] and lines[i]:upper():match("SELECT") then
          if log then
            log.info('expand_star', string.format('Found SELECT on line %d', i))
          end
          -- Mark that we found it
          select_start = 1
          select_end = #line
          break
        end
      end
    else
      if log then
        log.warn('expand_star', 'No SELECT * pattern found on current line: ' .. line)
      end
      vim.notify("No SELECT * found on current line", vim.log.levels.WARN)
      return
    end
  end

  if not select_start then
    if log then
      log.error('expand_star', 'Could not find SELECT * pattern to replace')
    end
    vim.notify("No SELECT * found on current line", vim.log.levels.WARN)
    return
  end

  if log then
    log.info('expand_star', string.format('Found pattern at positions %d-%d on line: %s',
      select_start, select_end, line:sub(1, 50)))
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
  -- Special handling for standalone * on its own line
  if is_standalone_star then
    if log then
      log.info('expand_star', 'Handling standalone * replacement')
    end

    -- Replace the * line with column list (indented)
    local lines_to_insert = {}
    for i, col in ipairs(column_names) do
      local prefix = i == 1 and "    " or "  , "
      table.insert(lines_to_insert, prefix .. col)
    end

    local row = cursor_pos[1]

    if log then
      log.info('expand_star', string.format('Replacing line %d (standalone *) with %d column lines', row, #lines_to_insert))
      log.debug('expand_star', 'Column lines: ' .. vim.inspect(lines_to_insert))
    end

    vim.api.nvim_buf_set_lines(0, row - 1, row, false, lines_to_insert)

    if log then
      log.info('expand_star', string.format('Successfully expanded standalone * to %d columns', #column_names))
    end

    vim.notify("Expanded * to " .. #column_names .. " columns (smart expansion)",
               vim.log.levels.INFO)
    return
  end

  -- Standard case: SELECT * on same line or need full SELECT statement
  local after_star = line:sub(select_end + 1)
  local total_length = #("SELECT " .. table.concat(column_names, ", ")) + #after_star
  local use_multiline = #column_names > 5 or total_length > 100

  if log then
    log.debug('expand_star', string.format('After star text: "%s"', after_star))
    log.debug('expand_star', string.format('Total length: %d, using multiline: %s',
      total_length, use_multiline))
  end

  if use_multiline then
    if log then
      log.info('expand_star', 'Using multiline format')
    end

    local lines_to_insert = {"SELECT"}
    for i, col in ipairs(column_names) do
      local prefix = i == 1 and "    " or "  , "
      table.insert(lines_to_insert, prefix .. col)
    end

    -- Add FROM clause if present
    if after_star:match("^%s+FROM") or after_star:match("^%s+from") then
      table.insert(lines_to_insert, after_star:match("^%s*(.*)"))
      if log then
        log.debug('expand_star', 'Added FROM clause from after_star')
      end
    end

    local row = cursor_pos[1]

    if log then
      log.info('expand_star', string.format('Replacing line %d with %d lines', row, #lines_to_insert))
      log.debug('expand_star', 'New lines: ' .. vim.inspect(lines_to_insert))
    end

    vim.api.nvim_buf_set_lines(0, row - 1, row, false, lines_to_insert)

    if log then
      log.info('expand_star', string.format('Successfully expanded * to %d columns (multiline)', #column_names))
    end

    vim.notify("Expanded * to " .. #column_names .. " columns (smart expansion)",
               vim.log.levels.INFO)
  else
    if log then
      log.info('expand_star', 'Using single-line format')
    end

    local expanded = "SELECT " .. table.concat(column_names, ", ") .. after_star

    if log then
      log.debug('expand_star', 'Expanded line: ' .. expanded:sub(1, 100))
    end

    vim.api.nvim_set_current_line(expanded)

    if log then
      log.info('expand_star', string.format('Successfully expanded * to %d columns (single line)', #column_names))
    end

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
  local log = get_logger()

  if log then
    log.info('expand_star', '=== expand_star_smart called ===')
    log.debug('expand_star', 'smart_expansion config: ' .. vim.inspect(config.smart_expansion or {}))
  end

  if not config.smart_expansion or not config.smart_expansion.enabled then
    if log then
      log.warn('expand_star', 'Smart expansion not enabled, falling back to static expansion')
    end
    -- Fall back to old expansion
    M.expand_star_columns(config, state)
    return
  end

  -- Get current query (including CTEs, subqueries, etc.)
  local query = M.get_query_for_expansion()

  if not query or query == "" then
    if log then
      log.error('expand_star', 'get_query_for_expansion returned empty query')
    end
    vim.notify("Could not determine query context", vim.log.levels.WARN)
    return
  end

  if log then
    log.info('expand_star', string.format('Got query (length: %d)', #query))
  end

  -- Check if the query contains SELECT *
  local has_select_star = query:match("SELECT%s+%*") or query:lower():match("select%s+%*")
  if not has_select_star then
    if log then
      log.warn('expand_star', 'No SELECT * found in query')
      log.debug('expand_star', 'Query preview: ' .. query:sub(1, 300))
    end
    vim.notify("No SELECT * found in query", vim.log.levels.WARN)
    return
  end

  -- Execute a preview query to get column names (LIMIT 1 to get schema)
  -- Note: LIMIT 0 returns [] with no schema, so we need at least 1 row
  local preview_query = query:gsub("LIMIT%s+%d+", ""):gsub("limit%s+%d+", "") .. " LIMIT 1"

  if log then
    log.info('expand_star', string.format('Preview query (length: %d): %s...',
      #preview_query, preview_query:sub(1, 150)))
  end

  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    if log then
      log.error('expand_star', 'Failed to get command path: ' .. (err or 'unknown error'))
    end
    vim.notify(err, vim.log.levels.ERROR)
    return
  end

  if log then
    log.debug('expand_star', 'Command path: ' .. command_path)
    log.info('expand_star', 'Executing query to get schema...')
  end

  -- Show progress notification
  vim.notify("Executing query to get columns...", vim.log.levels.INFO)

  -- Build args with data file if set
  local args = {}

  -- Add data file first (positional argument)
  local data_file = state:get_data_file()
  if data_file then
    if log then
      log.info('expand_star', 'Using data file: ' .. data_file)
    end
    table.insert(args, data_file)
  else
    if log then
      log.warn('expand_star', 'No data file set - query may fail for table references')
    end
  end

  -- Add query and output format
  table.insert(args, '-q')
  table.insert(args, preview_query)
  table.insert(args, '-o')
  table.insert(args, 'json')

  if log then
    log.debug('expand_star', 'CLI args: ' .. vim.inspect(args))
  end

  -- Execute with -o json to get results including schema
  Job:new({
    command = command_path,
    args = args,
    on_exit = function(j, return_val)
      vim.schedule(function()
        local log = get_logger()

        if return_val == 0 then
          if log then
            log.info('expand_star', 'Query executed successfully (exit code 0)')
          end

          local result = table.concat(j:result(), '\n')

          if log then
            log.debug('expand_star', string.format('Raw result length: %d', #result))
            log.debug('expand_star', 'Raw result preview: ' .. result:sub(1, 500))
          end

          -- Filter out comment lines (starting with #)
          local json_lines = {}
          for line in result:gmatch("[^\r\n]+") do
            if not line:match("^%s*#") then
              table.insert(json_lines, line)
            end
          end
          local json_str = table.concat(json_lines, "\n")

          if log then
            log.debug('expand_star', string.format('JSON after filtering comments (length: %d): %s...',
              #json_str, json_str:sub(1, 300)))
          end

          local ok, data = pcall(vim.json.decode, json_str)

          if not ok then
            if log then
              log.error('expand_star', 'JSON decode failed: ' .. tostring(data))
              log.error('expand_star', 'JSON string: ' .. json_str:sub(1, 1000))
            end
          end

          if ok and data then
            if log then
              log.info('expand_star', 'Successfully parsed JSON result')
              log.debug('expand_star', 'Data type: ' .. type(data))
              if type(data) == "table" then
                log.debug('expand_star', 'Data has columns field: ' .. tostring(data.columns ~= nil))
                log.debug('expand_star', 'Data array length: ' .. tostring(#data))
              end
            end

            local columns = {}

            -- Handle two formats:
            -- 1. Array of objects: [{"col1": val, "col2": val}]
            -- 2. Object with columns: {columns: [{name: "col1"}, ...]}

            if type(data) == "table" then
              if data.columns then
                -- Format 2: {columns: [...]}
                if log then
                  log.info('expand_star', 'Using format 2 (columns field), count: ' .. #data.columns)
                end
                for _, col in ipairs(data.columns) do
                  local col_name = type(col) == "table" and col.name or col
                  table.insert(columns, col_name)
                end
              elseif #data > 0 and type(data[1]) == "table" then
                -- Format 1: [{col1: val, col2: val}, ...]
                -- Extract keys from first object
                if log then
                  log.info('expand_star', 'Using format 1 (array of objects)')
                end
                for key, _ in pairs(data[1]) do
                  table.insert(columns, key)
                end
                -- Sort columns alphabetically for consistency
                table.sort(columns)
              else
                if log then
                  log.warn('expand_star', 'Unrecognized data format')
                  log.debug('expand_star', 'Data: ' .. vim.inspect(data))
                end
              end
            end

            if #columns > 0 then
              if log then
                log.info('expand_star', string.format('Extracted %d columns: %s',
                  #columns, table.concat(columns, ', ')))
              end

              -- Store columns in state
              state:set_last_query_columns(columns)

              -- Perform the replacement
              M.replace_star_with_columns(columns)

              -- Add column hint comment
              M.add_column_hint_comment(columns, config)

              if log then
                log.info('expand_star', '=== expand_star_smart completed successfully ===')
              end
            else
              if log then
                log.error('expand_star', 'No columns extracted from result')
              end
              vim.notify("No columns found in query result", vim.log.levels.WARN)
            end
          else
            vim.notify("Could not parse query result", vim.log.levels.ERROR)
            if log then
              log.error('expand_star', 'Failed to parse JSON or invalid data structure')
            end
            -- Fall back to static expansion if enabled
            if config.smart_expansion.fallback_to_static then
              if log then
                log.info('expand_star', 'Falling back to static expansion')
              end
              vim.notify("Falling back to static schema...", vim.log.levels.INFO)
              M.expand_star_columns(config, state)
            end
          end
        else
          local err_msg = table.concat(j:stderr_result(), '\n')
          if log then
            log.error('expand_star', string.format('Query execution failed with exit code %d', return_val))
            log.error('expand_star', 'Error message: ' .. err_msg)
          end
          vim.notify("Query execution failed: " .. err_msg, vim.log.levels.ERROR)

          -- Fall back to static expansion if enabled
          if config.smart_expansion.fallback_to_static then
            if log then
              log.info('expand_star', 'Falling back to static expansion after execution failure')
            end
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

-- Expand * with dependency-aware execution for scripts with temp tables
-- Uses the CLI --get-columns-at flag to analyze dependencies and get columns
function M.expand_star_with_dependencies(config, state)
  local log = get_logger()

  if log then
    log.info('expand_star', '=== expand_star_with_dependencies called ===')
  end

  -- Get current buffer and cursor position
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Check if cursor line has SELECT *
  local current_line_text = lines[cursor_line]
  local has_select_star = current_line_text:match("SELECT%s+%*") or
                          current_line_text:lower():match("select%s+%*") or
                          current_line_text:match("^%s*%*%s*$")

  if not has_select_star then
    if log then
      log.warn('expand_star', 'No SELECT * found on current line')
    end
    vim.notify("No SELECT * found on current line", vim.log.levels.WARN)
    return
  end

  -- Get script text
  local script = table.concat(lines, "\n")

  -- Extract and expand file-level variables (for @SET directives)
  local executor = require('sql-cli.executor')
  local file_vars = executor.extract_file_variables(lines)
  script = executor.expand_env_variables(script, file_vars)

  -- Check if this is a script with GO separators
  local has_go_separator = script:match("%sGO%s") or
                          script:match("^GO%s") or
                          script:match("%sGO$") or
                          script:match("^GO$")

  if not has_go_separator then
    if log then
      log.info('expand_star', 'No GO separators detected, falling back to smart expansion')
    end
    -- Fall back to regular smart expansion
    M.expand_star_smart(config, state)
    return
  end

  if log then
    log.info('expand_star', string.format('Script with GO separators detected, using dependency-aware expansion at line %d', cursor_line))
    log.info('expand_star', string.format('Buffer has %d lines total, script length: %d chars', #lines, #script))
    log.debug('expand_star', string.format('Line %d content: "%s"', cursor_line, current_line_text))
  end

  vim.notify("Analyzing script dependencies...", vim.log.levels.INFO)

  -- Save script to temp file
  local temp_file = vim.fn.tempname() .. ".sql"
  local file = io.open(temp_file, "w")
  if not file then
    vim.notify("Failed to create temporary file", vim.log.levels.ERROR)
    return
  end
  file:write(script)
  file:close()

  if log then
    log.debug('expand_star', 'Saved script to temp file: ' .. temp_file)
    -- Log first and last 3 lines of script for debugging
    local script_lines = vim.split(script, '\n')
    local preview_lines = {}
    for i = 1, math.min(3, #script_lines) do
      table.insert(preview_lines, string.format('  [%d] %s', i, script_lines[i]))
    end
    if #script_lines > 6 then
      table.insert(preview_lines, '  ...')
    end
    for i = math.max(4, #script_lines - 2), #script_lines do
      table.insert(preview_lines, string.format('  [%d] %s', i, script_lines[i]))
    end
    log.debug('expand_star', 'Script preview:\n' .. table.concat(preview_lines, '\n'))
  end

  -- Build command
  local command_path, err = utils.get_command_path(config.command)
  if not command_path then
    if log then
      log.error('expand_star', 'Failed to get command path: ' .. (err or 'unknown error'))
    end
    vim.notify(err, vim.log.levels.ERROR)
    return
  end

  if log then
    log.info('expand_star', 'Command path: ' .. command_path)
  end

  local args = {}

  -- Add data file if set
  local data_file = state:get_data_file()
  if data_file then
    if log then
      log.info('expand_star', 'Using data file: ' .. data_file)
    end
    table.insert(args, data_file)
  else
    if log then
      log.warn('expand_star', 'No data file set')
    end
  end

  -- Add script file and --get-columns-at flag
  table.insert(args, '-f')
  table.insert(args, temp_file)
  table.insert(args, '--get-columns-at')
  table.insert(args, tostring(cursor_line))

  if log then
    log.debug('expand_star', 'CLI args: ' .. vim.inspect(args))
    local full_command = command_path .. ' ' .. table.concat(vim.tbl_map(function(arg)
      return vim.fn.shellescape(arg)
    end, args), ' ')
    log.info('expand_star', 'Full command: ' .. full_command)
    log.info('expand_star', 'Executing CLI to get columns at line ' .. cursor_line .. '...')
  end

  -- Execute CLI command
  Job:new({
    command = command_path,
    args = args,
    on_exit = function(j, return_val)
      vim.schedule(function()
        local log = get_logger()

        if log then
          log.debug('expand_star', string.format('Job completed with exit code %d', return_val))
        end

        -- Clean up temp file
        vim.fn.delete(temp_file)
        if log then
          log.debug('expand_star', 'Cleaned up temp file: ' .. temp_file)
        end

        if return_val == 0 then
          if log then
            log.info('expand_star', 'CLI execution successful (exit code 0)')
          end

          local stdout_lines = j:result()
          local result = table.concat(stdout_lines, '\n')

          if log then
            log.debug('expand_star', string.format('Stdout has %d lines, total length: %d', #stdout_lines, #result))
            log.debug('expand_star', 'Raw result: ' .. result)
          end

          -- Parse CSV result (columns should be on first non-empty line)
          local columns = {}
          local col_count = 0
          for col in result:gmatch("[^,]+") do
            -- Trim whitespace
            col = col:match("^%s*(.-)%s*$")
            if col and col ~= "" then
              col_count = col_count + 1
              table.insert(columns, col)
              if log and col_count <= 10 then
                log.debug('expand_star', string.format('Parsed column %d: "%s"', col_count, col))
              end
            end
          end

          if log and col_count > 10 then
            log.debug('expand_star', string.format('... and %d more columns', col_count - 10))
          end

          if #columns > 0 then
            if log then
              log.info('expand_star', string.format('Extracted %d columns: %s',
                #columns, table.concat(columns, ', ')))
            end

            -- Store columns in state
            state:set_last_query_columns(columns)

            -- Perform the replacement
            M.replace_star_with_columns(columns)

            -- Add column hint comment
            M.add_column_hint_comment(columns, config)

            if log then
              log.info('expand_star', '=== expand_star_with_dependencies completed successfully ===')
            end
          else
            if log then
              log.error('expand_star', 'No columns extracted from CLI output')
            end
            vim.notify("No columns returned from CLI", vim.log.levels.WARN)

            -- Fall back to smart expansion
            if log then
              log.info('expand_star', 'Falling back to smart expansion')
            end
            M.expand_star_smart(config, state)
          end
        else
          local stderr_lines = j:stderr_result()
          local stdout_lines = j:result()
          local err_msg = table.concat(stderr_lines, '\n')
          local stdout_msg = table.concat(stdout_lines, '\n')

          if log then
            log.error('expand_star', string.format('CLI execution failed with exit code %d', return_val))
            log.error('expand_star', string.format('Stderr (%d lines): %s', #stderr_lines, err_msg))
            if #stdout_lines > 0 then
              log.debug('expand_star', string.format('Stdout (%d lines): %s', #stdout_lines, stdout_msg))
            end
          end
          vim.notify("Column expansion failed: " .. err_msg, vim.log.levels.ERROR)

          -- Fall back to smart expansion
          if log then
            log.info('expand_star', 'Falling back to smart expansion after CLI failure')
          end
          vim.notify("Falling back to regular expansion...", vim.log.levels.INFO)
          M.expand_star_smart(config, state)
        end
      end)
    end
  }):start()
end

return M