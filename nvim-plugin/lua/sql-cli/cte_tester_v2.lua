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
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end

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

  -- Find which CTE we're in (or default to last)
  local relative_cursor = cursor_line - start_line + 1
  local target_cte = nil
  local target_index = result.total  -- Default to last CTE

  -- For simplicity, just test the first or last CTE for now
  -- Could enhance this to detect cursor position
  target_cte = result.ctes[target_index]

  vim.notify(string.format("Testing CTE: %s", target_cte.name), vim.log.levels.INFO)

  -- Generate test query
  local test_query = M.generate_simple_test_query(query_lines, target_cte, result.ctes)

  -- Show full debug info - always show what we're about to submit
  vim.notify(string.format("=== CTE Test Query ===\n%s\n=== End Query ===", test_query), vim.log.levels.INFO)

  -- Execute the test query
  local executor = require('sql-cli.executor')
  executor.execute_query(test_query, config, state, false)
end

-- Generate a simple test query for a CTE
function M.generate_simple_test_query(query_lines, target_cte, all_ctes)
  -- More robust approach: track parentheses carefully
  local test_lines = {}
  local with_found = false
  local paren_depth = 0
  local current_cte_count = 0
  local target_index = target_cte.index + 1  -- Convert 0-based to 1-based
  local inside_cte = false

  for i, line in ipairs(query_lines) do
    local upper = line:upper()

    -- Look for WITH
    if not with_found and (upper:match("^%s*WITH%s") or upper:match("^%s*%-%-.*WITH%s")) then
      with_found = true
      table.insert(test_lines, line)

      -- Check if CTE name is on same line (WITH name AS ())
      if line:match("WITH%s+([%w_]+)%s+AS%s*%(") then
        current_cte_count = 1
        inside_cte = true
        -- Count opening parens on this line
        for char in line:gmatch(".") do
          if char == "(" then paren_depth = paren_depth + 1 end
          if char == ")" then paren_depth = paren_depth - 1 end
        end
        -- Check if CTE closes on same line
        if paren_depth == 0 and current_cte_count >= target_index then
          -- Remove trailing comma if present
          local last_line = test_lines[#test_lines]
          test_lines[#test_lines] = last_line:gsub(",%s*$", "")
          break
        end
      end
    elseif with_found then
      -- Check for new CTE starting (name AS ())
      local cte_match = line:match("^%s*([%w_]+)%s+AS%s*%(")
      if cte_match then
        current_cte_count = current_cte_count + 1
        inside_cte = true
      end

      -- Track parentheses
      for char in line:gmatch(".") do
        if char == "(" then paren_depth = paren_depth + 1 end
        if char == ")" then paren_depth = paren_depth - 1 end
      end

      -- Add the line if we haven't exceeded our target
      if current_cte_count <= target_index then
        table.insert(test_lines, line)

        -- Check if current CTE closed
        if inside_cte and paren_depth == 0 then
          inside_cte = false
          -- If this was our target CTE, we're done
          if current_cte_count == target_index then
            -- Remove trailing comma from last line if present
            local last_line = test_lines[#test_lines]
            test_lines[#test_lines] = last_line:gsub(",%s*$", "")
            break
          end
        end
      else
        break  -- We've passed our target
      end
    end
  end

  -- Ensure no trailing comma on last CTE line
  if #test_lines > 0 then
    local last_line = test_lines[#test_lines]
    test_lines[#test_lines] = last_line:gsub("%)%s*,$", ")")
  end

  -- Add SELECT from target CTE
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
  local query_lines = {}
  for i = start_line, end_line do
    table.insert(query_lines, lines[i])
  end

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

return M