-- SQL CLI Navigation Module
-- Functions for navigating between SQL queries and selecting them

local utils = require('sql-cli.utils')

local M = {}

-- Visually select query at cursor position
function M.select_query_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

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

-- Jump to next query
function M.next_query()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find next SQL statement
  for i = cursor_line + 1, #lines do
    if utils.is_statement_start(lines[i]) then
      vim.api.nvim_win_set_cursor(0, {i, 0})
      return
    end
  end

  -- Wrap around to beginning
  for i = 1, cursor_line do
    if utils.is_statement_start(lines[i]) then
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
    if utils.is_statement_start(lines[i]) then
      vim.api.nvim_win_set_cursor(0, {i, 0})
      return
    end
  end

  -- Wrap around to end
  for i = #lines, cursor_line, -1 do
    if utils.is_statement_start(lines[i]) then
      vim.api.nvim_win_set_cursor(0, {i, 0})
      vim.notify("Wrapped to last query", vim.log.levels.INFO)
      return
    end
  end

  vim.notify("No queries found", vim.log.levels.WARN)
end

-- Copy query at cursor to clipboard
function M.copy_query_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
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

  -- Copy to system clipboard (+ register)
  vim.fn.setreg('+', query)
  -- Also copy to unnamed register for convenience
  vim.fn.setreg('"', query)

  vim.notify("Query copied to clipboard", vim.log.levels.INFO)
end

-- Copy query at cursor in shell-friendly format
function M.copy_query_for_shell()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end

  -- Extract the query, skipping comments and terminators
  local query_lines = {}
  for i = start_line, end_line do
    local line = lines[i]
    -- Skip GO terminators and comment lines
    if not line:match("^%s*GO%s*$") and not line:match("^%s*%-%-") then
      -- Trim whitespace from each line
      local trimmed = line:match("^%s*(.-)%s*$")
      if trimmed and trimmed ~= "" then
        table.insert(query_lines, trimmed)
      end
    end
  end

  -- Join with spaces and escape for shell
  local query = table.concat(query_lines, " ")

  -- Escape double quotes and backslashes for shell
  query = query:gsub('\\', '\\\\')  -- Escape backslashes first
  query = query:gsub('"', '\\"')    -- Escape double quotes

  -- Format as a complete shell command
  local shell_command = string.format('sql-cli -q "%s"', query)

  -- Copy to system clipboard (+ register)
  vim.fn.setreg('+', shell_command)
  -- Also copy to unnamed register for convenience
  vim.fn.setreg('"', shell_command)

  vim.notify("Shell command copied to clipboard", vim.log.levels.INFO)
end

-- Toggle comment for query at cursor
function M.toggle_comment_query()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries using the same logic as execute_at_cursor
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

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
    elseif line and line:match("^%s*%-%-") and utils.is_statement_start(line:gsub("^%s*%-%-", "")) then
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
    if utils.is_statement_start(uncommented) then
      is_commented = true
    end
  end

  -- Toggle comments on SQL lines only (preserve documentation comments)
  for i = start_line, end_line do
    local line = lines[i]
    if line and not line:match("^%s*$") then
      -- Skip documentation comments that aren't SQL code
      local is_doc_comment = line:match("^%s*%-%-") and not utils.is_statement_start(line:gsub("^%s*%-%-", ""))

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

return M