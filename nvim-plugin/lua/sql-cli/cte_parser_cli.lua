-- SQL CLI CTE Parser using CLI JSON output
-- Uses sql-cli --cte-info for robust parsing

local M = {}

-- Parse CTEs using SQL CLI
function M.parse_ctes_with_cli(query_lines)
  -- Write query to temp file
  local temp_file = vim.fn.tempname() .. ".sql"
  vim.fn.writefile(query_lines, temp_file)

  -- Get sql-cli command path
  local utils = require('sql-cli.utils')
  local cmd_path = utils.get_command_path("sql-cli")
  if not cmd_path then
    vim.notify("sql-cli not found", vim.log.levels.ERROR)
    return nil
  end

  -- Execute sql-cli with --cte-info
  local cmd = string.format("%s -f %s --cte-info -o csv 2>/dev/null",
    vim.fn.shellescape(cmd_path),
    vim.fn.shellescape(temp_file))

  local output = vim.fn.system(cmd)

  -- Clean up temp file
  vim.fn.delete(temp_file)

  -- Parse JSON output
  local ok, result = pcall(vim.json.decode, output)
  if not ok then
    vim.notify("Failed to parse CTE info: " .. output:sub(1, 100), vim.log.levels.ERROR)
    return nil
  end

  if not result.success then
    vim.notify("SQL parse error: " .. (result.error or "unknown"), vim.log.levels.ERROR)
    return nil
  end

  return result
end

-- Enhanced CTE analysis popup using CLI parser
function M.show_cte_analysis_popup()
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

  -- Parse CTEs using SQL CLI
  local result = M.parse_ctes_with_cli(query_lines)

  -- Prepare analysis content
  local content = {}
  table.insert(content, "═══════════════════════════════════════════")
  table.insert(content, "       CTE ANALYSIS REPORT (CLI Parser)")
  table.insert(content, "═══════════════════════════════════════════")
  table.insert(content, "")
  table.insert(content, string.format("Query Range: Lines %d-%d", start_line, end_line))
  table.insert(content, string.format("Total Lines: %d", #query_lines))

  if result then
    table.insert(content, string.format("CTEs Found: %d", result.total))
    table.insert(content, string.format("Has Final SELECT: %s", result.has_final_select and "Yes" or "No"))
    table.insert(content, "")

    if result.total > 0 then
      table.insert(content, "CTE STRUCTURE:")
      table.insert(content, "─────────────────────────────────")
      for _, cte in ipairs(result.ctes) do
        table.insert(content, string.format("%d. %s", cte.index + 1, cte.name))
        -- Handle null/nil columns from JSON
        if cte.columns and type(cte.columns) == "table" and #cte.columns > 0 then
          table.insert(content, "   Columns: " .. table.concat(cte.columns, ", "))
        end
        -- Handle dependencies
        if cte.dependencies and type(cte.dependencies) == "table" and #cte.dependencies > 0 then
          table.insert(content, "   Depends on: " .. table.concat(cte.dependencies, ", "))
        else
          table.insert(content, "   Depends on: (none - base CTE)")
        end
      end

      table.insert(content, "")
      table.insert(content, "CTE DEPENDENCY CHAIN:")
      table.insert(content, "─────────────────────────────────")
      for _, cte in ipairs(result.ctes) do
        local deps = "(base)"
        if cte.dependencies and type(cte.dependencies) == "table" and #cte.dependencies > 0 then
          deps = table.concat(cte.dependencies, ", ")
        end
        table.insert(content, string.format("  %s <- %s", cte.name, deps))
      end

      table.insert(content, "")
      table.insert(content, "TEST COMMANDS:")
      table.insert(content, "─────────────────────────────────")
      for _, cte in ipairs(result.ctes) do
        table.insert(content, string.format("  • Test %s with <leader>sC", cte.name))
      end
    else
      table.insert(content, "")
      table.insert(content, "❌ NO CTEs FOUND")
    end
  else
    table.insert(content, "")
    table.insert(content, "❌ PARSE ERROR")
    table.insert(content, "Check query syntax or run sql-cli --cte-info manually")
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
  vim.bo[float_buf].modifiable = false
  vim.bo[float_buf].filetype = 'markdown'

  -- Window options
  local opts = {
    relative = 'editor',
    width = width,
    height = height,
    col = col,
    row = row,
    style = 'minimal',
    border = 'rounded',
    title = ' CTE Analysis (CLI) ',
    title_pos = 'center',
  }

  -- Create window
  local win = vim.api.nvim_open_win(float_buf, true, opts)

  -- Set keymaps
  local function close_popup()
    vim.api.nvim_win_close(win, true)
  end

  local function copy_to_clipboard()
    local text = table.concat(content, "\n")
    vim.fn.setreg('+', text)
    vim.notify("CTE analysis copied to clipboard", vim.log.levels.INFO)
    close_popup()
  end

  vim.keymap.set('n', 'q', close_popup, { buffer = float_buf, silent = true })
  vim.keymap.set('n', '<Esc>', close_popup, { buffer = float_buf, silent = true })
  vim.keymap.set('n', 'y', copy_to_clipboard, { buffer = float_buf, silent = true })
  vim.keymap.set('n', 'Y', copy_to_clipboard, { buffer = float_buf, silent = true })

  return result
end

return M