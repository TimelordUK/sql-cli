-- CLI Analyzer - Bridge between Neovim plugin and SQL CLI analysis commands
-- This module calls sql-cli with analysis flags and returns parsed results
-- Replaces fragile text parsing with structured CLI output

local M = {}

-- Lazy-load logger to avoid circular dependencies
local logger = nil
local function get_logger()
  if not logger then
    logger = require('sql-cli').logger
  end
  return logger
end

-- Get sql-cli command path from config
local function get_cli_path()
  local config = require('sql-cli').config
  return config.command_path or 'sql-cli'
end

--- Analyze a SQL query and return structured information
--- @param query string The SQL query to analyze
--- @param data_file string|nil Optional data file path
--- @return table|nil analysis The query analysis, or nil on error
--- @return string|nil error Error message if failed
function M.analyze_query(query, data_file)
  local log = get_logger()

  if log then
    log.debug('cli_analyzer', 'analyze_query called with query length: ' .. #query)
    if data_file then
      log.debug('cli_analyzer', 'Data file: ' .. data_file)
    end
  end

  local cmd = {get_cli_path()}

  -- Add data file if provided
  if data_file then
    table.insert(cmd, data_file)
  end

  -- Add query and analysis flag
  table.insert(cmd, '-q')
  table.insert(cmd, query)
  table.insert(cmd, '--analyze-query')

  if log then
    log.debug('cli_analyzer', 'Executing: ' .. table.concat(cmd, ' '))
  end

  -- Execute command and capture output
  local output = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error

  if exit_code ~= 0 then
    local error_msg = 'CLI analysis failed (exit code ' .. exit_code .. '): ' .. output
    if log then
      log.error('cli_analyzer', error_msg)
    end
    return nil, error_msg
  end

  -- Parse JSON output
  local success, analysis = pcall(vim.json.decode, output)
  if not success then
    local error_msg = 'Failed to parse CLI output: ' .. tostring(analysis)
    if log then
      log.error('cli_analyzer', error_msg)
      log.debug('cli_analyzer', 'Raw output: ' .. output)
    end
    return nil, error_msg
  end

  if log then
    log.info('cli_analyzer', string.format(
      'Analysis successful: %d CTEs, has_star=%s, %d tables',
      #analysis.ctes,
      tostring(analysis.has_star),
      #analysis.tables
    ))
  end

  return analysis, nil
end

--- Expand SELECT * to actual column names
--- @param query string The SQL query with SELECT *
--- @param data_file string|nil Optional data file path
--- @return table|nil expansion The column expansion result, or nil on error
--- @return string|nil error Error message if failed
function M.expand_star(query, data_file)
  local log = get_logger()

  if log then
    log.debug('cli_analyzer', 'expand_star called')
  end

  local cmd = {get_cli_path()}

  -- Add data file if provided
  if data_file then
    table.insert(cmd, data_file)
  end

  -- Add query and expansion flag
  table.insert(cmd, '-q')
  table.insert(cmd, query)
  table.insert(cmd, '--expand-star')

  if log then
    log.debug('cli_analyzer', 'Executing: ' .. table.concat(cmd, ' '))
  end

  -- Execute command and capture output
  local output = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error

  if exit_code ~= 0 then
    local error_msg = 'Column expansion failed (exit code ' .. exit_code .. '): ' .. output
    if log then
      log.error('cli_analyzer', error_msg)
    end
    return nil, error_msg
  end

  -- Parse JSON output
  local success, expansion = pcall(vim.json.decode, output)
  if not success then
    local error_msg = 'Failed to parse expansion output: ' .. tostring(expansion)
    if log then
      log.error('cli_analyzer', error_msg)
    end
    return nil, error_msg
  end

  if log then
    log.info('cli_analyzer', string.format(
      'Expansion successful: %d columns',
      #expansion.columns
    ))
  end

  return expansion, nil
end

--- Extract a specific CTE as a standalone query
--- @param query string The SQL query containing CTEs
--- @param cte_name string Name of the CTE to extract
--- @return string|nil cte_query The extracted CTE query, or nil on error
--- @return string|nil error Error message if failed
function M.extract_cte(query, cte_name)
  local log = get_logger()

  if log then
    log.debug('cli_analyzer', 'extract_cte called for: ' .. cte_name)
  end

  local cmd = {
    get_cli_path(),
    '-q', query,
    '--extract-cte', cte_name
  }

  if log then
    log.debug('cli_analyzer', 'Executing: ' .. table.concat(cmd, ' '))
  end

  -- Execute command and capture output
  local output = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error

  if exit_code ~= 0 then
    local error_msg = 'CTE extraction failed (exit code ' .. exit_code .. '): ' .. output
    if log then
      log.error('cli_analyzer', error_msg)
    end
    return nil, error_msg
  end

  if log then
    log.info('cli_analyzer', 'CTE extracted successfully (' .. #output .. ' chars)')
  end

  return output, nil
end

--- Find query context at a specific line:column position
--- @param query string The SQL query
--- @param line number Line number (1-indexed)
--- @param column number Column number (1-indexed)
--- @return table|nil context The query context, or nil on error
--- @return string|nil error Error message if failed
function M.find_query_context(query, line, column)
  local log = get_logger()

  if log then
    log.debug('cli_analyzer', string.format('find_query_context at %d:%d', line, column))
  end

  local position = string.format('%d:%d', line, column)
  local cmd = {
    get_cli_path(),
    '-q', query,
    '--query-at-position', position
  }

  if log then
    log.debug('cli_analyzer', 'Executing: ' .. table.concat(cmd, ' '))
  end

  -- Execute command and capture output
  local output = vim.fn.system(cmd)
  local exit_code = vim.v.shell_error

  if exit_code ~= 0 then
    local error_msg = 'Context detection failed (exit code ' .. exit_code .. '): ' .. output
    if log then
      log.error('cli_analyzer', error_msg)
    end
    return nil, error_msg
  end

  -- Parse JSON output
  local success, context = pcall(vim.json.decode, output)
  if not success then
    local error_msg = 'Failed to parse context output: ' .. tostring(context)
    if log then
      log.error('cli_analyzer', error_msg)
    end
    return nil, error_msg
  end

  if log then
    log.info('cli_analyzer', string.format(
      'Context found: %s%s',
      context.context_type,
      context.cte_name and (' (' .. context.cte_name .. ')') or ''
    ))
  end

  return context, nil
end

--- Helper: Get all buffer text as a single string
--- @param bufnr number|nil Buffer number (defaults to current)
--- @return string The full buffer content
function M.get_buffer_text(bufnr)
  bufnr = bufnr or 0
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  return table.concat(lines, '\n')
end

--- Helper: Check if query has SELECT *
--- Quick check without full analysis
--- @param query string The SQL query
--- @return boolean true if query contains SELECT *
function M.has_star(query)
  -- Simple pattern match for SELECT *
  -- This is just a quick check, use analyze_query for accurate results
  return query:upper():match('SELECT%s+%*') ~= nil
end

--- Helper: Get CTE names from analysis
--- @param analysis table Analysis result from analyze_query
--- @return table List of CTE names
function M.get_cte_names(analysis)
  local names = {}
  for _, cte in ipairs(analysis.ctes or {}) do
    table.insert(names, cte.name)
  end
  return names
end

--- Helper: Find CTE by name in analysis
--- @param analysis table Analysis result from analyze_query
--- @param cte_name string Name of CTE to find
--- @return table|nil The CTE info, or nil if not found
function M.find_cte_by_name(analysis, cte_name)
  for _, cte in ipairs(analysis.ctes or {}) do
    if cte.name == cte_name then
      return cte
    end
  end
  return nil
end

--- Helper: Check if CTE is a WEB CTE
--- @param cte table CTE info from analysis
--- @return boolean true if WEB CTE
function M.is_web_cte(cte)
  return cte.cte_type == 'WEB'
end

--- Show raw JSON analysis in a modal popup
--- @param query string SQL query to analyze
function M.show_raw_analysis_popup(query)
  local log = get_logger()
  if log then
    log.info('cli_analyzer', 'show_raw_analysis_popup called')
  end

  -- Analyze query
  local analysis, err = M.analyze_query(query)

  if err then
    vim.notify("Failed to analyze query: " .. err, vim.log.levels.ERROR)
    return
  end

  -- Convert analysis to pretty JSON
  local json_str = vim.fn.json_encode(analysis)
  local ok, pretty_json = pcall(vim.fn.json_decode, json_str)
  if ok then
    json_str = vim.fn.json_encode(pretty_json)
  end

  -- Format JSON with indentation
  local lines = {}
  for line in json_str:gmatch('[^\n]+') do
    table.insert(lines, line)
  end

  -- Add header and footer
  table.insert(lines, 1, "")
  table.insert(lines, 1, "Raw CLI Analysis Output (--analyze-query)")
  table.insert(lines, 1, "═══════════════════════════════════════════")
  table.insert(lines, "")
  table.insert(lines, "═══════════════════════════════════════════")
  table.insert(lines, "Press 'y' to yank | 'q' to close")

  -- Create buffer
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_buf_set_option(buf, 'modifiable', false)
  vim.api.nvim_buf_set_option(buf, 'filetype', 'json')

  -- Calculate window size
  local width = math.min(80, vim.o.columns - 10)
  local height = math.min(#lines + 2, vim.o.lines - 5)

  -- Create window
  local win_opts = {
    relative = 'editor',
    width = width,
    height = height,
    col = math.floor((vim.o.columns - width) / 2),
    row = math.floor((vim.o.lines - height) / 2),
    style = 'minimal',
    border = 'rounded',
    title = ' Query Analysis (JSON) ',
    title_pos = 'center',
  }

  local win = vim.api.nvim_open_win(buf, true, win_opts)

  -- Set keymaps
  local opts = { noremap = true, silent = true, buffer = buf }

  vim.keymap.set('n', 'q', function()
    vim.api.nvim_win_close(win, true)
  end, opts)

  vim.keymap.set('n', '<Esc>', function()
    vim.api.nvim_win_close(win, true)
  end, opts)

  vim.keymap.set('n', 'y', function()
    vim.fn.setreg('+', json_str)
    vim.notify("Analysis JSON yanked to clipboard", vim.log.levels.INFO)
  end, opts)

  if log then
    log.info('cli_analyzer', 'Displayed raw analysis popup')
  end
end

return M
