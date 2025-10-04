-- SQL CLI CTE Parser using CLI analysis
-- Uses cli_analyzer module with --analyze-query for robust parsing

local M = {}

-- Get logger
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

-- Enhanced CTE analysis popup using CLI analyzer
function M.show_cte_analysis_popup()
  local log = get_logger()
  if log then
    log.info('cte_parser_cli', 'show_cte_analysis_popup called')
  end

  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries
  local utils = require('sql-cli.utils')
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL query found at cursor", vim.log.levels.WARN)
    return
  end

  -- Extract query lines
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end
  local query = table.concat(query_lines, '\n')

  if log then
    log.debug('cte_parser_cli', string.format('Query range: %d-%d (%d lines)', start_line, end_line, #query_lines))
  end

  -- Analyze query using CLI analyzer
  local cli_analyzer = require('sql-cli.cli_analyzer')
  local analysis, err = cli_analyzer.analyze_query(query)

  if err then
    vim.notify("Failed to analyze query: " .. err, vim.log.levels.ERROR)
    return
  end

  -- Prepare analysis content
  local content = {}
  table.insert(content, "═══════════════════════════════════════════")
  table.insert(content, "       CTE ANALYSIS REPORT")
  table.insert(content, "═══════════════════════════════════════════")
  table.insert(content, "")
  table.insert(content, string.format("Query Range: Lines %d-%d", start_line, end_line))
  table.insert(content, string.format("Total Lines: %d", #query_lines))
  table.insert(content, string.format("CTEs Found: %d", #analysis.ctes))
  table.insert(content, string.format("Has SELECT *: %s", analysis.has_star and "Yes" or "No"))
  table.insert(content, "")

  if #analysis.ctes > 0 then
    table.insert(content, "CTE STRUCTURE:")
    table.insert(content, "─────────────────────────────────")
    for i, cte in ipairs(analysis.ctes) do
      local cte_type_indicator = cte.cte_type == "WEB" and " [WEB]" or ""
      table.insert(content, string.format("%d. %s%s", i, cte.name, cte_type_indicator))

      -- Show columns if available
      if cte.columns and #cte.columns > 0 then
        local col_preview = table.concat(cte.columns, ", ")
        if #col_preview > 50 then
          col_preview = col_preview:sub(1, 47) .. "..."
        end
        table.insert(content, "   Columns: " .. col_preview)
      end

      -- Show dependencies
      if cte.references and #cte.references > 0 then
        table.insert(content, "   Depends on: " .. table.concat(cte.references, ", "))
      else
        table.insert(content, "   Depends on: (none - base CTE)")
      end
    end

    table.insert(content, "")
    table.insert(content, "CTE DEPENDENCY CHAIN:")
    table.insert(content, "─────────────────────────────────")
    for _, cte in ipairs(analysis.ctes) do
      local deps = "(base)"
      if cte.references and #cte.references > 0 then
        deps = table.concat(cte.references, ", ")
      end
      table.insert(content, string.format("  %s <- %s", cte.name, deps))
    end

    table.insert(content, "")
    table.insert(content, "TABLES REFERENCED:")
    table.insert(content, "─────────────────────────────────")
    if #analysis.tables > 0 then
      for _, table_name in ipairs(analysis.tables) do
        table.insert(content, "  • " .. table_name)
      end
    else
      table.insert(content, "  (no external tables)")
    end

    table.insert(content, "")
    table.insert(content, "TEST COMMANDS:")
    table.insert(content, "─────────────────────────────────")
    for _, cte in ipairs(analysis.ctes) do
      table.insert(content, string.format("  • Test %s with \\sC (cursor in CTE)", cte.name))
    end
  else
    table.insert(content, "")
    table.insert(content, "❌ NO CTEs FOUND")
    table.insert(content, "This query does not contain any WITH clauses")
  end

  table.insert(content, "")
  table.insert(content, "═══════════════════════════════════════════")
  table.insert(content, "Press 'y' to copy | 'q' to close")

  -- Create floating window
  local width = 60
  local height = math.min(#content + 2, 30)

  -- Get editor dimensions
  local editor_width = vim.o.columns
  local editor_height = vim.o.lines

  -- Calculate centered position
  local col = math.floor((editor_width - width) / 2)
  local row = math.floor((editor_height - height) / 2)

  -- Create buffer
  local float_buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(float_buf, 0, -1, false, content)
  vim.api.nvim_buf_set_option(float_buf, 'modifiable', false)
  vim.api.nvim_buf_set_option(float_buf, 'filetype', 'sql-cli-analysis')

  -- Create window
  local win_opts = {
    relative = 'editor',
    width = width,
    height = height,
    col = col,
    row = row,
    style = 'minimal',
    border = 'rounded',
    title = ' CTE Analysis ',
    title_pos = 'center',
  }

  local float_win = vim.api.nvim_open_win(float_buf, true, win_opts)

  -- Set up keymaps
  local opts = { noremap = true, silent = true, buffer = float_buf }

  vim.keymap.set('n', 'q', function()
    vim.api.nvim_win_close(float_win, true)
  end, opts)

  vim.keymap.set('n', '<Esc>', function()
    vim.api.nvim_win_close(float_win, true)
  end, opts)

  vim.keymap.set('n', 'y', function()
    vim.fn.setreg('+', table.concat(content, '\n'))
    vim.notify("Analysis copied to clipboard", vim.log.levels.INFO)
  end, opts)

  if log then
    log.info('cte_parser_cli', string.format('Displayed analysis popup with %d CTEs', #analysis.ctes))
  end
end

return M
