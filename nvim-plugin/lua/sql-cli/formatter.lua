-- SQL CLI Formatter Module
-- SQL query formatting using AST and fallback formatters

local M = {}
local utils = require('sql-cli.utils')

-- Format SQL query at cursor
function M.format_query_at_cursor(config, state)
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries using the same logic as execute_at_cursor
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end

  -- Preserve leading comments (like data hints)
  local preserved_comments = {}
  local actual_query_start = start_line

  -- Scan for leading comment lines to preserve
  for i = start_line, end_line do
    local line = lines[i]
    if line and line:match("^%s*%-%-") then
      -- This is a comment line
      table.insert(preserved_comments, line)
      actual_query_start = i + 1
    else
      -- Found first non-comment line, stop
      break
    end
  end

  -- Handle terminators (semicolon or GO)
  local terminator = ""
  local last_line = lines[end_line]
  if last_line then
    if last_line:match(";%s*$") then
      terminator = ";"
      lines[end_line] = last_line:gsub(";%s*$", "")
    elseif last_line:upper():match("^%s*GO%s*$") then
      terminator = "GO"
      end_line = end_line - 1  -- Exclude the GO line
    end
  end

  -- Extract the FULL query including comments (not from actual_query_start)
  -- We'll let the CLI handle comment preservation
  local query_lines = vim.list_slice(lines, start_line, end_line)
  local query = table.concat(query_lines, "\n")

  -- Debug output
  if config.debug_format then
    vim.notify("Original query: " .. query:sub(1, 100), vim.log.levels.INFO)
  end

  -- Format the query
  local formatted = M.format_sql(query, config)

  -- Handle empty or nil result
  if not formatted or formatted == "" then
    vim.notify("Formatting failed - no result", vim.log.levels.WARN)
    return
  end

  -- Debug formatted result
  if config.debug_format then
    vim.notify("Formatted result length: " .. #formatted, vim.log.levels.INFO)
    vim.notify("Formatted (first 100): " .. formatted:sub(1, 100), vim.log.levels.INFO)
  end

  -- Split formatted query into lines
  local new_lines = vim.split(formatted, "\n")

  -- Add terminator back if it was present
  if terminator == ";" then
    new_lines[#new_lines] = new_lines[#new_lines] .. ";"
  elseif terminator == "GO" then
    table.insert(new_lines, "GO")
  end

  -- Note: No need to manually prepend preserved comments anymore
  -- The CLI now handles comment preservation with --preserve-comments flag

  -- Replace the lines in the buffer
  vim.api.nvim_buf_set_lines(bufnr, start_line - 1, end_line, false, new_lines)

  vim.notify("Query formatted", vim.log.levels.INFO)
end

-- SQL Formatter function using AST-based formatter
function M.format_sql(query, config)
  -- Use the sql-cli AST formatter for accurate formatting
  local command, err = utils.get_command_path(config.command)

  if not command then
    vim.notify(err .. ", using fallback formatter", vim.log.levels.WARN)
    return M.format_sql_simple(query)
  end

  -- Build format command with options
  local format_cmd = {command, "--format"}

  -- Add comment preservation flag (enabled by default for better UX)
  table.insert(format_cmd, "--preserve-comments")

  -- Add user preferences
  if config.format and config.format.lowercase then
    table.insert(format_cmd, "--lowercase")
  end
  if config.format and config.format.compact then
    table.insert(format_cmd, "--compact")
  end
  if config.format and config.format.tabs then
    table.insert(format_cmd, "--tabs")
  end

  -- Execute formatter using vim.fn.systemlist for better handling
  local cmd_str = table.concat(format_cmd, " ")
  local result = vim.fn.system(cmd_str, query)

  -- Check for errors or empty result
  if vim.v.shell_error ~= 0 or not result or result == "" or result:match("^%s*$") then
    -- Check what went wrong
    if vim.v.shell_error ~= 0 then
      vim.notify("Formatter error (code " .. vim.v.shell_error .. "), using fallback", vim.log.levels.WARN)
    elseif not result or result == "" then
      vim.notify("Formatter returned empty result, using fallback", vim.log.levels.WARN)
    end
    -- Fall back to simple formatting if AST formatter fails
    return M.format_sql_simple(query)
  end

  -- Trim trailing newlines but preserve the formatted content
  result = result:gsub("%s+$", "")

  -- Final check - make sure we have content
  if not result or result == "" then
    vim.notify("Formatter result was empty after trimming, using fallback", vim.log.levels.WARN)
    return M.format_sql_simple(query)
  end

  return result
end

-- Simple fallback formatter (original regex-based)
function M.format_sql_simple(query)
  -- Separate comment lines from SQL to preserve them
  local lines = vim.split(query, "\n")
  local comment_lines = {}
  local sql_lines = {}

  for _, line in ipairs(lines) do
    if line:match("^%s*%-%-") then
      -- Preserve comment lines as-is
      table.insert(comment_lines, line)
    else
      table.insert(sql_lines, line)
    end
  end

  -- Join SQL lines and normalize whitespace
  local sql = table.concat(sql_lines, " ")
  sql = sql:gsub("%s+", " "):gsub("^%s+", ""):gsub("%s+$", "")

  -- Put major clauses on new lines
  sql = sql:gsub("%s+([Ww][Ii][Tt][Hh])%s+", "\nWITH ")
  sql = sql:gsub("%s+([Ss][Ee][Ll][Ee][Cc][Tt])%s+", "\nSELECT ")
  sql = sql:gsub("%s+([Ff][Rr][Oo][Mm])%s+", "\nFROM ")
  sql = sql:gsub("%s+([Ww][Hh][Ee][Rr][Ee])%s+", "\nWHERE ")
  sql = sql:gsub("%s+([Gg][Rr][Oo][Uu][Pp]%s+[Bb][Yy])%s+", "\nGROUP BY ")
  sql = sql:gsub("%s+([Hh][Aa][Vv][Ii][Nn][Gg])%s+", "\nHAVING ")
  sql = sql:gsub("%s+([Oo][Rr][Dd][Ee][Rr]%s+[Bb][Yy])%s+", "\nORDER BY ")
  sql = sql:gsub("%s+([Ll][Ii][Mm][Ii][Tt])%s+", "\nLIMIT ")
  sql = sql:gsub("%s+([Oo][Ff][Ff][Ss][Ee][Tt])%s+", "\nOFFSET ")

  -- Handle JOIN clauses (must be before single JOIN to avoid breaking compound joins)
  sql = sql:gsub("%s+([Ff][Uu][Ll][Ll]%s+[Oo][Uu][Tt][Ee][Rr]%s+[Jj][Oo][Ii][Nn])%s+", "\nFULL OUTER JOIN ")
  sql = sql:gsub("%s+([Ll][Ee][Ff][Tt]%s+[Jj][Oo][Ii][Nn])%s+", "\nLEFT JOIN ")
  sql = sql:gsub("%s+([Rr][Ii][Gg][Hh][Tt]%s+[Jj][Oo][Ii][Nn])%s+", "\nRIGHT JOIN ")
  sql = sql:gsub("%s+([Ii][Nn][Nn][Ee][Rr]%s+[Jj][Oo][Ii][Nn])%s+", "\nINNER JOIN ")
  sql = sql:gsub("%s+([Cc][Rr][Oo][Ss][Ss]%s+[Jj][Oo][Ii][Nn])%s+", "\nCROSS JOIN ")
  -- Only convert standalone JOIN if it wasn't already part of a compound JOIN
  sql = sql:gsub("([^%w])([Jj][Oo][Ii][Nn])%s+", "%1\nJOIN ")

  -- Indent ON clauses for JOINs
  sql = sql:gsub("%s+([Oo][Nn])%s+", "\n    ON ")

  -- Handle AND/OR in WHERE clause with proper indentation
  sql = sql:gsub("%s+([Aa][Nn][Dd])%s+", "\n    AND ")
  sql = sql:gsub("%s+([Oo][Rr])%s+", "\n    OR ")

  -- Handle CASE statements
  sql = sql:gsub("%s+([Cc][Aa][Ss][Ee])%s+", "\n    CASE ")
  sql = sql:gsub("%s+([Ww][Hh][Ee][Nn])%s+", "\n        WHEN ")
  sql = sql:gsub("%s+([Tt][Hh][Ee][Nn])%s+", " THEN ")
  sql = sql:gsub("%s+([Ee][Ll][Ss][Ee])%s+", "\n        ELSE ")
  sql = sql:gsub("%s+([Ee][Nn][Dd])%s+", "\n    END ")

  -- Clean up any double newlines
  sql = sql:gsub("\n\n+", "\n")

  -- Remove trailing semicolon or GO for reformatting
  sql = sql:gsub(";%s*$", "")
  sql = sql:gsub("%s+[Gg][Oo]%s*$", "")

  -- Trim final whitespace
  sql = sql:gsub("^%s+", ""):gsub("%s+$", "")

  -- Reassemble: comments first, then formatted SQL
  local result_lines = {}
  for _, comment in ipairs(comment_lines) do
    table.insert(result_lines, comment)
  end
  table.insert(result_lines, sql)

  return table.concat(result_lines, "\n")
end

-- Test the formatter directly (for debugging)
function M.test_formatter(config)
  local test_query = "SELECT a, b FROM table WHERE x = 1"
  vim.notify("Testing formatter with: " .. test_query, vim.log.levels.INFO)

  local result = M.format_sql(test_query, config)
  if result then
    vim.notify("Result: " .. result, vim.log.levels.INFO)
  else
    vim.notify("Formatter returned nil", vim.log.levels.ERROR)
  end
end

return M