-- SQL CLI CTE Testing Module V2
-- Uses CLI parser for robust CTE detection

local M = {}
local utils = require('sql-cli.utils')

-- Test CTE at cursor using CLI parser
function M.test_cte_at_cursor(config, state)
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL query found at cursor", vim.log.levels.WARN)
    return
  end

  -- Extract query lines
  vim.notify(string.format("Extracting lines %d to %d from buffer", start_line, end_line), vim.log.levels.INFO)
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end
  vim.notify(string.format("Extracted %d lines", #query_lines), vim.log.levels.INFO)

  -- Use CLI to parse CTEs
  local cte_parser_cli = require('sql-cli.cte_parser_cli')
  local result = cte_parser_cli.parse_ctes_with_cli(query_lines)

  if not result or not result.success then
    vim.notify("Failed to parse CTEs", vim.log.levels.ERROR)
    return
  end

  if result.total == 0 then
    vim.notify("No CTEs found in query", vim.log.levels.WARN)
    return
  end

  -- Find which CTE we're in based on cursor position
  local relative_cursor = cursor_line - start_line + 1
  local target_cte = nil

  -- Simpler approach: Find all CTE start lines by name and determine which one cursor is after
  -- We know the CTE names from the CLI parser, so we can search for them in order
  local cte_start_lines = {}

  for idx, cte in ipairs(result.ctes) do
    local cte_name = cte.name
    -- Find the line where this CTE is defined
    for i, line in ipairs(query_lines) do
      -- Match: "name AS (" or "WITH name AS ("
      if line:match("%f[%w]" .. cte_name .. "%s+AS%s*%(") then
        table.insert(cte_start_lines, {
          index = idx,
          line = i,
          name = cte_name
        })
        vim.notify(string.format("[CTE PARSE] Found CTE #%d '%s' at line %d", idx, cte_name, i), vim.log.levels.WARN)
        break
      end
    end
  end

  -- Determine which CTE the cursor is in
  -- Logic: Cursor is in the CTE that starts at or before the cursor line,
  -- but after the previous CTE's start
  local target_index = 1  -- Default to first CTE

  vim.notify(string.format("[CTE DEBUG] Cursor at relative line %d (absolute line %d)", relative_cursor, cursor_line), vim.log.levels.WARN)
  vim.notify(string.format("[CTE DEBUG] Query starts at line %d, extracted %d lines", start_line, #query_lines), vim.log.levels.WARN)
  vim.notify(string.format("[CTE DEBUG] Found %d CTEs with start lines", #cte_start_lines), vim.log.levels.WARN)

  -- Find the last CTE whose start line is <= cursor line
  for _, cte_start in ipairs(cte_start_lines) do
    vim.notify(string.format("[CTE DEBUG] CTE %d (%s): starts at line %d", cte_start.index, cte_start.name, cte_start.line), vim.log.levels.WARN)
    if cte_start.line <= relative_cursor then
      target_index = cte_start.index
      vim.notify(string.format("[CTE DEBUG] → Cursor is at/after this CTE start (updating target to %d)", target_index), vim.log.levels.WARN)
    end
  end

  vim.notify(string.format("[CTE DEBUG] ✓ Final target_index: %d (CTE: %s)", target_index, result.ctes[target_index].name), vim.log.levels.WARN)

  -- Get the target CTE (result.ctes is 1-indexed array from JSON, target_index is 1-based)
  target_cte = result.ctes[target_index]

  if not target_cte then
    vim.notify(string.format("Could not find CTE at index %d (total: %d)", target_index, result.total), vim.log.levels.ERROR)
    return
  end

  vim.notify(string.format("Testing CTE: %s (CTE #%d of %d)", target_cte.name, target_index, result.total), vim.log.levels.INFO)
  vim.notify(string.format("Target position for query generation: %d", target_index), vim.log.levels.INFO)

  -- Generate test query
  local test_query = M.generate_simple_test_query(query_lines, target_cte, result.ctes, target_index)

  -- Show the query in a modal for confirmation
  M.show_query_confirmation_modal(test_query, function(action)
    if action == "execute" then
      -- Debug: show data file status
      local data_file = state:get_data_file()
      if data_file then
        vim.notify(string.format("Data file: %s", data_file), vim.log.levels.INFO)
      else
        vim.notify("No data file set", vim.log.levels.INFO)
      end

      -- For CTE testing with RANGE, temporarily clear the data file
      -- since RANGE doesn't need an external data source
      local saved_data_file = state:get_data_file()
      if test_query:match("FROM%s+RANGE%s*%(") then
        state:set_data_file(nil)
        vim.notify("Clearing data file for RANGE query", vim.log.levels.DEBUG)
      end

      -- Execute the test query
      local executor = require('sql-cli.executor')
      executor.execute_query(test_query, config, state)

      -- Restore the data file
      if saved_data_file then
        state:set_data_file(saved_data_file)
      end
    elseif action == "yank" then
      vim.fn.setreg('+', test_query)
      vim.notify("CTE test query yanked to clipboard", vim.log.levels.INFO)
    end
  end)
end

-- Generate a simple test query for a CTE
function M.generate_simple_test_query(query_lines, target_cte, all_ctes, target_position)
  -- Build test query by including all CTEs up to and including the target
  local test_lines = {}
  local with_found = false
  local paren_depth = 0
  local current_cte_idx = 0

  -- If target_position not provided, find it from the CTE name
  if not target_position then
    for idx, cte in ipairs(all_ctes) do
      if cte.name == target_cte.name then
        target_position = idx
        break
      end
    end
  end

  if not target_position then
    vim.notify(string.format("Target CTE %s not found in list", target_cte.name), vim.log.levels.ERROR)
    return ""
  end

  vim.notify(string.format("Building query for CTE '%s' at position %d", target_cte.name, target_position), vim.log.levels.INFO)

  -- Add debug header as SQL comments
  table.insert(test_lines, string.format("-- CTE Tester Debug Info"))
  table.insert(test_lines, string.format("-- Target CTE: %s", target_cte.name))
  table.insert(test_lines, string.format("-- Target Position: %d of %d total CTEs", target_position, #all_ctes))
  table.insert(test_lines, string.format("-- CTEs in query: %s", table.concat(vim.tbl_map(function(c) return c.name end, all_ctes), ", ")))
  table.insert(test_lines, "")

  -- Simple approach: Include everything from WITH up to and including target CTE
  for i, line in ipairs(query_lines) do
    local upper = line:upper()
    local trimmed = vim.trim(line)

    -- Skip pure comment lines when looking for WITH
    if not with_found and trimmed:match("^%-%-") then
      goto continue
    end

    -- Look for WITH to start (be more permissive)
    if not with_found then
      if upper:match("WITH%s") or upper:match("WITH$") then
        with_found = true
        vim.notify("Found WITH clause", vim.log.levels.DEBUG)
        -- Check if first CTE is on same line
        if line:match("WITH%s+([%w_]+)%s+AS%s*%(") then
          current_cte_idx = 1
          vim.notify("First CTE on same line as WITH", vim.log.levels.DEBUG)
        end
      end
    else
      -- Check for new CTE definitions (not on WITH line)
      local cte_name = line:match("^%s*([%w_]+)%s+AS%s*%(")
      if cte_name then
        current_cte_idx = current_cte_idx + 1
        vim.notify(string.format("Found CTE %d: %s (target=%d)", current_cte_idx, cte_name, target_position), vim.log.levels.INFO)
        -- Add debug comment in generated query
        table.insert(test_lines, string.format("-- [DEBUG] Found CTE #%d: %s (target=%d)", current_cte_idx, cte_name, target_position))

        -- If we've gone past our target, don't include this line
        if current_cte_idx > target_position then
          -- Clean up trailing comma from previous line
          if #test_lines > 0 then
            test_lines[#test_lines] = test_lines[#test_lines]:gsub(",%s*$", "")
          end
          vim.notify(string.format("Stopping - CTE %d > target %d", current_cte_idx, target_position), vim.log.levels.INFO)
          break
        end
      end
    end

    -- If we found WITH, start adding lines
    if with_found then
      table.insert(test_lines, line)

      -- Track parentheses to know when CTEs end
      for char in line:gmatch(".") do
        if char == "(" then paren_depth = paren_depth + 1 end
        if char == ")" then paren_depth = paren_depth - 1 end
      end

      -- If we've completed our target CTE (parens balanced and we're at target)
      if current_cte_idx == target_position and paren_depth == 0 and line:match("%)") then
        -- Remove trailing comma if present
        test_lines[#test_lines] = test_lines[#test_lines]:gsub(",%s*$", "")
        vim.notify(string.format("Target CTE %d complete at line %d (paren_depth=%d)", current_cte_idx, i, paren_depth), vim.log.levels.INFO)
        break
      end

      -- Also check if we hit a SELECT after CTEs (main query starts)
      if current_cte_idx > 0 and paren_depth == 0 and upper:match("^%s*SELECT%s") then
        -- We've hit the main SELECT, remove it and stop
        table.remove(test_lines, #test_lines)  -- Remove the SELECT line we just added
        -- Clean up trailing comma from previous line
        if #test_lines > 0 then
          test_lines[#test_lines] = test_lines[#test_lines]:gsub(",%s*$", "")
        end
        vim.notify("Found main SELECT, stopping", vim.log.levels.DEBUG)
        break
      end
    end

    ::continue::
  end

  vim.notify(string.format("Total lines collected: %d", #test_lines), vim.log.levels.DEBUG)

  -- Final cleanup: ensure no trailing comma
  if #test_lines > 0 then
    local last_line = test_lines[#test_lines]
    if last_line:match("%)%s*,$") then
      test_lines[#test_lines] = last_line:gsub("%)%s*,$", ")")
    end
  end

  -- Add SELECT from target CTE with debug comment
  table.insert(test_lines, "")
  table.insert(test_lines, string.format("-- Selecting from target CTE: %s (position %d)", target_cte.name, target_position))
  table.insert(test_lines, string.format("SELECT * FROM %s;", target_cte.name))

  return table.concat(test_lines, "\n")
end

-- Create test query in new buffer
function M.test_cte_in_new_buffer(config, state)
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL query found at cursor", vim.log.levels.WARN)
    return
  end

  -- Extract query lines
  vim.notify(string.format("Extracting lines %d to %d from buffer", start_line, end_line), vim.log.levels.INFO)
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end
  vim.notify(string.format("Extracted %d lines", #query_lines), vim.log.levels.INFO)

  -- Use CLI to parse CTEs
  local cte_parser_cli = require('sql-cli.cte_parser_cli')
  local result = cte_parser_cli.parse_ctes_with_cli(query_lines)

  if not result or not result.success or result.total == 0 then
    vim.notify("No CTEs found in query", vim.log.levels.WARN)
    return
  end

  -- Generate test query for last CTE
  local target_cte = result.ctes[result.total]
  local test_query = M.generate_simple_test_query(query_lines, target_cte, result.ctes)

  -- Create new buffer with test query
  vim.cmd("new")
  vim.bo.filetype = "sql"
  vim.api.nvim_buf_set_lines(0, 0, -1, false, vim.split(test_query, "\n"))

  vim.notify("Created test query for CTE: " .. target_cte.name, vim.log.levels.INFO)
end

-- Show query confirmation modal
function M.show_query_confirmation_modal(query, callback)
  -- Split query into lines for display
  local lines = vim.split(query, "\n")

  -- Add header and footer
  table.insert(lines, 1, "═══════════════════════════════════════════════════════")
  table.insert(lines, 2, "               CTE TEST QUERY PREVIEW")
  table.insert(lines, 3, "═══════════════════════════════════════════════════════")
  table.insert(lines, 4, "")
  table.insert(lines, "")
  table.insert(lines, "═══════════════════════════════════════════════════════")
  table.insert(lines, "  [Enter] Execute  |  [y] Yank to clipboard  |  [Esc] Cancel")
  table.insert(lines, "═══════════════════════════════════════════════════════")

  -- Calculate window size
  local width = 80
  local height = math.min(#lines + 2, 30)  -- Max 30 lines

  -- Find longest line for better width calculation
  for _, line in ipairs(lines) do
    width = math.max(width, #line + 4)
  end
  width = math.min(width, 120)  -- Cap at 120 chars wide

  -- Get editor dimensions
  local editor_width = vim.o.columns
  local editor_height = vim.o.lines

  -- Calculate centered position
  local col = math.floor((editor_width - width) / 2)
  local row = math.floor((editor_height - height) / 2)

  -- Create buffer
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.bo[buf].modifiable = false
  vim.bo[buf].filetype = 'sql'

  -- Window options
  local opts = {
    relative = 'editor',
    width = width,
    height = height,
    col = col,
    row = row,
    style = 'minimal',
    border = 'rounded',
    title = ' CTE Test Query ',
    title_pos = 'center',
  }

  -- Create window
  local win = vim.api.nvim_open_win(buf, true, opts)

  -- Set keymaps
  local function close_and_callback(action)
    vim.api.nvim_win_close(win, true)
    if callback then
      callback(action)
    end
  end

  -- Enter to execute
  vim.keymap.set('n', '<CR>', function()
    close_and_callback("execute")
  end, { buffer = buf, silent = true })

  -- y to yank
  vim.keymap.set('n', 'y', function()
    close_and_callback("yank")
  end, { buffer = buf, silent = true })
  vim.keymap.set('n', 'Y', function()
    close_and_callback("yank")
  end, { buffer = buf, silent = true })

  -- Escape or q to cancel
  vim.keymap.set('n', '<Esc>', function()
    close_and_callback("cancel")
  end, { buffer = buf, silent = true })
  vim.keymap.set('n', 'q', function()
    close_and_callback("cancel")
  end, { buffer = buf, silent = true })
end

return M