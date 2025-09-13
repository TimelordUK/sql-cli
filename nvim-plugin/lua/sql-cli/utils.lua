-- SQL CLI Utility Functions Module
-- Helper functions used across the plugin

local M = {}

-- Strip ANSI escape sequences from text
function M.strip_ansi_codes(text)
  -- Remove ANSI color codes and formatting codes
  -- \x1b is the escape character (octal 033, decimal 27)
  text = text:gsub("\x1b%[[%d;]*m", "")  -- Color codes like \x1b[38;5;12m
  text = text:gsub("\x1b%[%d*m", "")      -- Simple codes like \x1b[0m
  text = text:gsub("\x1b%[%d*;%d*m", "")  -- Codes like \x1b[1;32m
  text = text:gsub("\x1b%[K", "")         -- Clear to end of line
  text = text:gsub("\x1b%[[%d;]*[A-Za-z]", "") -- Other control sequences
  return text
end

-- Check if a line starts a SQL statement
function M.is_statement_start(line)
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

-- Check if a line is a query terminator
function M.is_terminator(line)
  if not line then return false end
  return line:match("^%s*GO%s*$") or        -- GO on its own line
         line:match(";%s*$") or              -- Semicolon at end of line
         line:match(";%s*%-%-") or           -- Semicolon followed by comment
         line:match(";%s*/")                 -- Semicolon followed by comment
end

-- Find query boundaries at cursor position
function M.find_query_at_cursor(lines, cursor_line)
  local start_line = 1
  local end_line = #lines

  -- STEP 1: Search backwards from cursor for previous terminator
  for i = cursor_line - 1, 1, -1 do
    if M.is_terminator(lines[i]) then
      start_line = i + 1
      break
    end
  end

  -- STEP 2: Search forwards from cursor for next terminator
  for i = cursor_line, #lines do
    if M.is_terminator(lines[i]) then
      end_line = i
      break
    end
  end

  -- Trim empty lines at start and end
  while start_line <= end_line and (not lines[start_line] or lines[start_line]:match("^%s*$")) do
    start_line = start_line + 1
  end

  if not M.is_terminator(lines[end_line]) then
    while end_line > start_line and (not lines[end_line] or lines[end_line]:match("^%s*$")) do
      end_line = end_line - 1
    end
  end

  return start_line, end_line
end

-- Get the full path to the sql-cli command
function M.get_command_path(command)
  command = command or "sql-cli"

  -- Try to find the full path if not absolute
  if not command:match("^/") then
    local which_cmd = vim.fn.exepath(command)
    if which_cmd and which_cmd ~= "" then
      command = which_cmd
    end
  end

  -- Check if command exists
  if vim.fn.executable(command) == 0 then
    return nil, "sql-cli not found at " .. command
  end

  return command, nil
end

-- Detect data file hint in SQL file
function M.detect_data_hint(lines, base_dir)
  -- Look for data hint in first 5 lines
  for i = 1, math.min(5, #lines) do
    local line = lines[i]
    if line then
      -- Pattern 1: -- #! path/to/data.csv
      local match = line:match("^%s*%-%-%s*#!%s*(.+)$")
      if match then
        local file_path = vim.trim(match)
        -- Handle relative paths
        if not file_path:match("^/") and not file_path:match("^~") then
          file_path = base_dir .. "/" .. file_path
        end
        -- Expand ~ to home directory
        file_path = vim.fn.expand(file_path)
        -- Check if file exists
        if vim.fn.filereadable(file_path) == 1 then
          return file_path
        end
      end

      -- Pattern 2: -- #!data: path/to/data.csv
      match = line:match("^%s*%-%-%s*#!data:%s*(.+)$")
      if match then
        local file_path = vim.trim(match)
        -- Handle relative paths
        if not file_path:match("^/") and not file_path:match("^~") then
          file_path = base_dir .. "/" .. file_path
        end
        -- Expand ~ to home directory
        file_path = vim.fn.expand(file_path)
        -- Check if file exists
        if vim.fn.filereadable(file_path) == 1 then
          return file_path
        end
      end

      -- Pattern 3: Simple comment with data file
      match = line:match("^%s*%-%-%s*data:%s*(.+%.csv)$")
      if match then
        local file_path = vim.trim(match)
        -- Handle relative paths
        if not file_path:match("^/") and not file_path:match("^~") then
          file_path = base_dir .. "/" .. file_path
        end
        -- Expand ~ to home directory
        file_path = vim.fn.expand(file_path)
        -- Check if file exists
        if vim.fn.filereadable(file_path) == 1 then
          return file_path
        end
      end
    end
  end

  return nil
end

-- Parse schema from csv header
function M.parse_csv_header(first_line)
  local columns = {}
  -- Handle quoted fields in CSV header
  local in_quotes = false
  local current_field = ""

  for i = 1, #first_line do
    local char = first_line:sub(i, i)
    if char == '"' then
      in_quotes = not in_quotes
    elseif char == ',' and not in_quotes then
      table.insert(columns, vim.trim(current_field:gsub('^"', ''):gsub('"$', '')))
      current_field = ""
    else
      current_field = current_field .. char
    end
  end

  -- Add the last field
  if current_field ~= "" then
    table.insert(columns, vim.trim(current_field:gsub('^"', ''):gsub('"$', '')))
  end

  return columns
end

-- Test if buffer is a SQL file
function M.is_sql_file(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  local filetype = vim.bo[bufnr].filetype
  return filetype == "sql" or filetype == "mysql" or filetype == "plsql"
end

-- Test if buffer is a CSV file
function M.is_csv_file(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  local filename = vim.api.nvim_buf_get_name(bufnr)
  return filename:match("%.csv$") ~= nil
end

return M