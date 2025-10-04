-- SQL CLI CTE Testing Module V3
-- Uses CLI analysis commands for robust CTE detection and extraction
-- Replaces complex regex parsing with structured CLI output

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

-- Get CLI analyzer module
local cli_analyzer = require('sql-cli.cli_analyzer')
local utils = require('sql-cli.utils')

--- Test CTE at cursor using CLI analysis
--- This is the main entry point called by \sC keybinding
function M.test_cte_at_cursor(config, state)
  local log = get_logger()

  if log then
    log.info('cte_test_v3', '=== test_cte_at_cursor called (V3 with CLI analysis) ===')
  end

  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  if log then
    log.debug('cte_test_v3', string.format('Buffer: %d, cursor line: %d, total lines: %d',
      bufnr, cursor_line, #lines))
  end

  -- Find query boundaries (using existing utils)
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    if log then
      log.error('cte_test_v3', string.format('No query found at cursor line %d', cursor_line))
    end
    vim.notify("No SQL query found at cursor", vim.log.levels.WARN)
    return
  end

  if log then
    log.info('cte_test_v3', string.format('Found query boundaries: lines %d-%d (%d lines)',
      start_line, end_line, end_line - start_line + 1))
  end

  -- Extract query text
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end
  local query = table.concat(query_lines, '\n')

  if log then
    log.debug('cte_test_v3', 'Query length: ' .. #query)
    log.debug('cte_test_v3', 'Query preview: ' .. query:sub(1, 200))
  end

  -- Analyze query using CLI
  vim.notify("Analyzing query with CLI...", vim.log.levels.INFO)
  local analysis, err = cli_analyzer.analyze_query(query)

  if err then
    if log then
      log.error('cte_test_v3', 'CLI analysis failed: ' .. err)
    end
    vim.notify("Failed to analyze query: " .. err, vim.log.levels.ERROR)
    return
  end

  if log then
    log.info('cte_test_v3', string.format('Analysis complete: %d CTEs found', #analysis.ctes))
  end

  -- Check if there are any CTEs
  if #analysis.ctes == 0 then
    if log then
      log.warn('cte_test_v3', 'No CTEs found in query')
    end
    vim.notify("No CTEs found in query", vim.log.levels.WARN)
    return
  end

  if log then
    log.info('cte_test_v3', 'CTEs: ' .. table.concat(cli_analyzer.get_cte_names(analysis), ', '))
  end

  -- Determine which CTE cursor is in
  local relative_cursor = cursor_line - start_line + 1
  local target_cte = M.find_cte_at_cursor(query_lines, analysis.ctes, relative_cursor, log)

  if not target_cte then
    vim.notify("Could not determine CTE at cursor", vim.log.levels.WARN)
    return
  end

  if log then
    log.info('cte_test_v3', string.format('Target CTE: %s (%s)', target_cte.name, target_cte.cte_type))
  end

  vim.notify(string.format("Testing CTE: %s (%s)", target_cte.name, target_cte.cte_type), vim.log.levels.INFO)

  -- Extract CTE using CLI
  vim.notify("Extracting CTE with CLI...", vim.log.levels.INFO)
  local cte_query, err = cli_analyzer.extract_cte(query, target_cte.name)

  if err then
    if log then
      log.error('cte_test_v3', 'CTE extraction failed: ' .. err)
    end
    vim.notify("Failed to extract CTE: " .. err, vim.log.levels.ERROR)
    return
  end

  if log then
    log.info('cte_test_v3', 'CTE extracted successfully (' .. #cte_query .. ' chars)')
    log.debug('cte_test_v3', 'Extracted query: ' .. cte_query)
  end

  -- Show query confirmation modal
  M.show_query_confirmation_modal(cte_query, target_cte, config, state, log)
end

--- Find which CTE the cursor is in
--- Uses simple heuristic: search for CTE names in order, find last one before cursor
function M.find_cte_at_cursor(query_lines, ctes, relative_cursor, log)
  if log then
    log.debug('cte_test_v3', string.format('Finding CTE at cursor (relative line %d)', relative_cursor))
  end

  -- Find start line for each CTE by searching for "name AS ("
  local cte_positions = {}

  for idx, cte in ipairs(ctes) do
    local cte_name = cte.name
    local upper_name = cte_name:upper()

    if log then
      log.debug('cte_test_v3', string.format('Searching for CTE #%d: %s', idx, cte_name))
    end

    -- Search for this CTE in the query
    for i, line in ipairs(query_lines) do
      local upper_line = line:upper()

      -- Match: "CTE_NAME AS (" (handles WITH, WITH WEB, comma-separated, etc.)
      if upper_line:match("%f[%w_]" .. upper_name .. "%s+AS%s*%(") then
        table.insert(cte_positions, {
          index = idx,
          line = i,
          name = cte_name,
          cte = cte
        })

        if log then
          log.info('cte_test_v3', string.format('Found CTE #%d "%s" at line %d', idx, cte_name, i))
        end
        break
      end
    end
  end

  if #cte_positions == 0 then
    if log then
      log.error('cte_test_v3', 'Could not find any CTE definitions in query text')
    end
    return nil
  end

  -- Find the last CTE whose start line is <= cursor line
  local target_cte = nil
  for _, pos in ipairs(cte_positions) do
    if pos.line <= relative_cursor then
      target_cte = pos.cte

      if log then
        log.debug('cte_test_v3', string.format('Cursor is at/after CTE "%s" (line %d <= %d)',
          pos.name, pos.line, relative_cursor))
      end
    end
  end

  if not target_cte then
    -- Cursor is before first CTE, use first one
    target_cte = cte_positions[1].cte

    if log then
      log.warn('cte_test_v3', 'Cursor before first CTE, using first CTE: ' .. target_cte.name)
    end
  end

  return target_cte
end

--- Show query confirmation modal with execute/yank options
function M.show_query_confirmation_modal(cte_query, target_cte, config, state, log)
  -- Create buffer for modal
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_option(buf, 'bufhidden', 'wipe')
  vim.api.nvim_buf_set_option(buf, 'filetype', 'sql')

  -- Prepare content
  local title = string.format(" CTE Test: %s (%s) ", target_cte.name, target_cte.cte_type)
  local separator = string.rep('─', #title)

  local lines = {
    title,
    separator,
    "",
  }

  -- Add query lines
  for line in cte_query:gmatch('[^\n]+') do
    table.insert(lines, line)
  end

  table.insert(lines, "")
  table.insert(lines, separator)
  table.insert(lines, " [Enter] Execute  [y] Yank  [q] Cancel ")

  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_buf_set_option(buf, 'modifiable', false)

  -- Calculate window size
  local width = 0
  for _, line in ipairs(lines) do
    width = math.max(width, #line)
  end
  width = math.min(width + 4, vim.o.columns - 10)
  local height = math.min(#lines + 2, vim.o.lines - 10)

  -- Open floating window
  local win_opts = {
    relative = 'editor',
    width = width,
    height = height,
    col = (vim.o.columns - width) / 2,
    row = (vim.o.lines - height) / 2,
    style = 'minimal',
    border = 'rounded',
  }

  local win = vim.api.nvim_open_win(buf, true, win_opts)

  -- Keybindings
  local function close_modal()
    vim.api.nvim_win_close(win, true)
  end

  local function execute_query()
    close_modal()

    if log then
      log.info('cte_test_v3', 'Executing CTE test query')
    end

    vim.notify(string.format("Executing CTE: %s", target_cte.name), vim.log.levels.INFO)

    -- Get data file
    local data_file = state:get_data_file()

    if log then
      if data_file then
        log.debug('cte_test_v3', 'Data file: ' .. data_file)
      else
        log.debug('cte_test_v3', 'No data file set')
      end
    end

    -- For WEB CTEs or queries with RANGE, don't use data file
    if target_cte.cte_type == 'WEB' or cte_query:match("FROM%s+RANGE%s*%(") then
      data_file = nil

      if log then
        log.info('cte_test_v3', 'Clearing data file for WEB CTE or RANGE query')
      end
    end

    -- Execute query
    local executor = require('sql-cli.executor')

    -- Temporarily set data file for execution
    local saved_data_file = state:get_data_file()
    if data_file ~= saved_data_file then
      state:set_data_file(data_file)
    end

    executor.execute_query(cte_query, config, state)

    -- Restore data file
    if saved_data_file then
      state:set_data_file(saved_data_file)
    end
  end

  local function yank_query()
    close_modal()
    vim.fn.setreg('+', cte_query)
    vim.notify(string.format("CTE %s yanked to clipboard", target_cte.name), vim.log.levels.INFO)

    if log then
      log.info('cte_test_v3', 'Query yanked to clipboard')
    end
  end

  -- Set keymaps
  local opts = { noremap = true, silent = true, buffer = buf }
  vim.keymap.set('n', '<CR>', execute_query, opts)
  vim.keymap.set('n', 'y', yank_query, opts)
  vim.keymap.set('n', 'q', close_modal, opts)
  vim.keymap.set('n', '<Esc>', close_modal, opts)
end

return M
