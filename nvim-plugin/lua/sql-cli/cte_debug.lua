-- SQL CLI CTE Debugging Module
-- Helps debug CTE parsing issues

local M = {}

-- Create a floating window with CTE analysis
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

  -- Analyze CTEs
  local cte_tester = require('sql-cli.cte_tester')
  local ctes = cte_tester.parse_ctes(query_lines)

  -- Prepare analysis content
  local content = {}
  table.insert(content, "═══════════════════════════════════════════")
  table.insert(content, "           CTE ANALYSIS REPORT")
  table.insert(content, "═══════════════════════════════════════════")
  table.insert(content, "")
  table.insert(content, string.format("Query Range: Lines %d-%d", start_line, end_line))
  table.insert(content, string.format("Total Lines: %d", #query_lines))
  table.insert(content, string.format("CTEs Found: %d", #ctes))
  table.insert(content, "")

  if #ctes > 0 then
    table.insert(content, "CTE STRUCTURE:")
    table.insert(content, "─────────────────────────────────")
    for i, cte in ipairs(ctes) do
      table.insert(content, string.format("%d. %s", i, cte.name))
      table.insert(content, string.format("   Lines: %d-%d (%d lines)",
        cte.start_line + start_line - 1,
        cte.end_line + start_line - 1,
        cte.end_line - cte.start_line + 1))
    end
    table.insert(content, "")
    table.insert(content, "CTE DEPENDENCY CHAIN:")
    table.insert(content, "─────────────────────────────────")
    for i, cte in ipairs(ctes) do
      if i == 1 then
        table.insert(content, string.format("  %s (base)", cte.name))
      else
        table.insert(content, string.format("  └─> %s", cte.name))
      end
    end
    table.insert(content, "")
    table.insert(content, "TEST COMMANDS:")
    table.insert(content, "─────────────────────────────────")
    for i, cte in ipairs(ctes) do
      table.insert(content, string.format("  Test %s: Move cursor to line %d and press <leader>sC",
        cte.name, cte.start_line + start_line - 1))
    end
  else
    table.insert(content, "❌ NO CTEs FOUND")
    table.insert(content, "")
    table.insert(content, "TROUBLESHOOTING:")
    table.insert(content, "─────────────────────────────────")
    table.insert(content, "• Ensure query starts with WITH keyword")
    table.insert(content, "• Check CTE syntax: name AS (...)")
    table.insert(content, "• Verify parentheses are balanced")
    table.insert(content, "")
    table.insert(content, "QUERY PREVIEW (first 10 lines):")
    table.insert(content, "─────────────────────────────────")
    for i = 1, math.min(10, #query_lines) do
      local line = query_lines[i]
      local marker = ""
      if line:upper():match("^%s*WITH%s") then
        marker = " <-- WITH found"
      elseif line:match("^%s*([%w_]+)%s+AS%s*%(") then
        marker = " <-- CTE definition"
      end
      table.insert(content, string.format("%2d: %s%s", i, line:sub(1, 45), marker))
    end

    -- Additional debugging
    table.insert(content, "")
    table.insert(content, "DEBUG INFO:")
    table.insert(content, "─────────────────────────────────")
    local clean_query = table.concat(query_lines, "\n"):gsub("%-%-[^\n]*", "")
    if clean_query:upper():match("WITH%s") then
      table.insert(content, "✓ WITH keyword found in query")
    else
      table.insert(content, "✗ WITH keyword NOT found")
    end

    -- Check for CTE pattern
    local cte_pattern = clean_query:match("([%w_]+)%s+AS%s*%(")
    if cte_pattern then
      table.insert(content, "✓ CTE pattern found: " .. cte_pattern)
    else
      table.insert(content, "✗ No CTE pattern found")
    end
  end

  table.insert(content, "")
  table.insert(content, "═══════════════════════════════════════════")
  table.insert(content, "Press 'y' to copy | 'q' to close | '?' for help")

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
    title = ' CTE Analysis ',
    title_pos = 'center',
  }

  -- Create window
  local win = vim.api.nvim_open_win(float_buf, true, opts)

  -- Set keymaps for the popup
  local function close_popup()
    vim.api.nvim_win_close(win, true)
  end

  local function copy_to_clipboard()
    local text = table.concat(content, "\n")
    vim.fn.setreg('+', text)
    vim.notify("CTE analysis copied to clipboard", vim.log.levels.INFO)
    close_popup()
  end

  -- Set buffer-local keymaps
  vim.keymap.set('n', 'q', close_popup, { buffer = float_buf, silent = true })
  vim.keymap.set('n', '<Esc>', close_popup, { buffer = float_buf, silent = true })
  vim.keymap.set('n', 'y', copy_to_clipboard, { buffer = float_buf, silent = true })
  vim.keymap.set('n', 'Y', copy_to_clipboard, { buffer = float_buf, silent = true })

  -- Help
  vim.keymap.set('n', '?', function()
    vim.notify([[
CTE Analysis Keybindings:
  y/Y - Copy analysis to clipboard
  q/Esc - Close window
  j/k - Scroll up/down]], vim.log.levels.INFO)
  end, { buffer = float_buf, silent = true })

  return ctes
end

-- Debug function to parse and display CTE structure
function M.debug_cte_parsing()
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

  vim.notify(string.format("Query boundaries: lines %d to %d", start_line, end_line), vim.log.levels.INFO)

  -- Extract query lines
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end

  vim.notify(string.format("Extracted %d lines", #query_lines), vim.log.levels.INFO)

  -- Show first 5 lines
  local preview = {}
  for i = 1, math.min(5, #query_lines) do
    table.insert(preview, string.format("Line %d: %s", i, query_lines[i]:sub(1, 50)))
  end
  vim.notify("First lines:\n" .. table.concat(preview, "\n"), vim.log.levels.INFO)

  -- Try to identify CTEs manually
  local ctes = {}
  local in_with = false
  local current_cte = nil
  local paren_depth = 0

  for i, line in ipairs(query_lines) do
    local upper = line:upper()
    local trimmed = vim.trim(line)

    -- Debug each line
    if i <= 10 then
      vim.notify(string.format("Line %d: in_with=%s, paren=%d, line=%s",
        i, tostring(in_with), paren_depth, trimmed:sub(1, 30)), vim.log.levels.DEBUG)
    end

    -- Check for WITH
    if upper:match("^%s*WITH%s*$") then
      vim.notify("Found WITH on line " .. i, vim.log.levels.INFO)
      in_with = true
    elseif upper:match("^%s*WITH%s+") then
      vim.notify("Found WITH with content on line " .. i, vim.log.levels.INFO)
      in_with = true
    end

    -- Look for CTE names
    if in_with then
      local name = trimmed:match("^(%w+)%s+AS%s*%(")
      if name then
        vim.notify(string.format("Found CTE '%s' on line %d", name, i), vim.log.levels.INFO)
        if current_cte then
          current_cte.end_line = i - 1
          table.insert(ctes, current_cte)
        end
        current_cte = {
          name = name,
          start_line = i,
          end_line = nil
        }
        paren_depth = 1
      end

      -- Count parentheses
      if current_cte then
        for char in line:gmatch(".") do
          if char == "(" then
            paren_depth = paren_depth + 1
          elseif char == ")" then
            paren_depth = paren_depth - 1
            if paren_depth == 0 then
              vim.notify(string.format("CTE '%s' closes on line %d", current_cte.name, i), vim.log.levels.INFO)
              current_cte.end_line = i
              table.insert(ctes, current_cte)
              current_cte = nil
              -- Check if more CTEs
              if not trimmed:match(",%s*$") then
                in_with = false
              end
              break
            end
          end
        end
      end
    end

    -- Check for SELECT (end of CTEs)
    if upper:match("^%s*SELECT%s") and not current_cte then
      vim.notify("Found SELECT on line " .. i .. ", CTEs end", vim.log.levels.INFO)
      break
    end
  end

  -- Report findings
  vim.notify(string.format("\nFound %d CTEs:", #ctes), vim.log.levels.INFO)
  for _, cte in ipairs(ctes) do
    vim.notify(string.format("  - %s (lines %d-%d)", cte.name, cte.start_line, cte.end_line), vim.log.levels.INFO)
  end

  return ctes
end

-- Use SQL CLI to parse CTEs
function M.parse_with_cli()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  local utils = require('sql-cli.utils')
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL query found", vim.log.levels.WARN)
    return
  end

  -- Extract query
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end
  local query = table.concat(query_lines, "\n")

  -- Write query to temp file
  local temp_file = vim.fn.tempname() .. ".sql"
  vim.fn.writefile(vim.split(query, "\n"), temp_file)

  -- Call sql-cli with --query-plan
  local cmd = "sql-cli --query-plan < " .. vim.fn.shellescape(temp_file)
  local output = vim.fn.system(cmd)

  -- Parse output to find CTEs
  vim.notify("AST Output:\n" .. output:sub(1, 500), vim.log.levels.INFO)

  -- Look for ctes array in AST
  if output:match("ctes: %[") then
    local ctes_section = output:match("ctes: %[(.-)%]")
    if ctes_section then
      vim.notify("CTEs section: " .. ctes_section, vim.log.levels.INFO)
    end
  else
    vim.notify("No CTEs found in AST", vim.log.levels.WARN)
  end

  -- Clean up
  vim.fn.delete(temp_file)
end

return M