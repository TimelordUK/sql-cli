-- SQL CLI CTE Testing Module
-- Allows testing CTEs incrementally by converting to SELECT * FROM <cte_name>

local M = {}
local utils = require('sql-cli.utils')

-- Parse CTE structure from query
function M.parse_ctes(query_lines)
  local ctes = {}
  local in_with_block = false
  local current_cte = nil
  local paren_depth = 0

  -- Skip leading comments and empty lines to find actual query start
  local query_start = 1
  for i, line in ipairs(query_lines) do
    local trimmed = vim.trim(line)
    if trimmed ~= "" and not trimmed:match("^%-%-") then
      query_start = i
      break
    end
  end

  -- Join all lines to handle multi-line patterns better
  local full_query = table.concat(query_lines, "\n")

  -- Check if query has WITH clause (ignoring leading comments)
  if not full_query:upper():match("%sWITH%s") and not full_query:upper():match("^WITH%s") then
    -- Also check after removing comments
    local clean_query = full_query:gsub("%-%-[^\n]*", "")
    if not clean_query:upper():match("WITH%s") then
      return ctes
    end
  end

  for i, line in ipairs(query_lines) do
    local upper = line:upper()
    local trimmed = vim.trim(line)

    -- Skip comment lines and empty lines
    if trimmed == "" or trimmed:match("^%-%-") then
      goto continue
    end

    -- Check for WITH keyword (including WITH followed by CTE name)
    if upper:match("^%s*WITH%s+[%w_]") or upper:match("^%s*WITH%s*$") then
      in_with_block = true
      -- Check if WITH line has CTE definition
      local name = trimmed:match("^WITH%s+([%w_]+)%s+AS%s*%(")
      if name then
        current_cte = {
          name = name,
          start_line = i,
          end_line = nil
        }
        paren_depth = 1
      end
    end

    -- Look for CTE definitions
    if in_with_block then
      -- Match CTE name pattern (name AS ( )
      -- Use [%w_]+ to match alphanumeric and underscores
      -- Skip if we already found a CTE on the WITH line
      if not current_cte then
        local name = trimmed:match("^([%w_]+)%s+AS%s*%(")
        if name then
          -- Start new CTE
          current_cte = {
            name = name,
            start_line = i,
            end_line = nil
          }
          paren_depth = 1  -- We found the opening paren
        end
      end

      if current_cte then
        -- Track parentheses to find end of current CTE
        for char in line:gmatch(".") do
          if char == "(" then
            paren_depth = paren_depth + 1
          elseif char == ")" then
            paren_depth = paren_depth - 1

            -- Check if this closes the CTE
            if paren_depth == 0 then
              current_cte.end_line = i

              -- Check if there are more CTEs (line ends with comma)
              if trimmed:match("%)%s*,$") then
                -- More CTEs coming
                table.insert(ctes, current_cte)
                current_cte = nil
              elseif trimmed:match("%)%s*$") then
                -- Last CTE
                table.insert(ctes, current_cte)
                current_cte = nil
                in_with_block = false
              end
              break
            end
          end
        end
      end

      -- Check for SELECT (end of CTE block)
      if upper:match("^%s*SELECT%s") and paren_depth == 0 then
        if current_cte then
          current_cte.end_line = i - 1
          table.insert(ctes, current_cte)
          current_cte = nil
        end
        in_with_block = false
      end
    end

    ::continue::
  end

  -- Handle unclosed CTE
  if current_cte then
    current_cte.end_line = #query_lines
    table.insert(ctes, current_cte)
  end

  return ctes
end

-- Find CTE at cursor position
function M.find_cte_at_cursor(ctes, cursor_line)
  for i, cte in ipairs(ctes) do
    if cursor_line >= cte.start_line and cursor_line <= cte.end_line then
      return cte, i
    end
  end
  return nil, nil
end

-- Generate test query for CTE
function M.generate_test_query(query_lines, cte_index, ctes)
  local test_lines = {}

  -- Debug: Show what we're working with
  vim.notify(string.format("Generating test query for CTE %d of %d", cte_index, #ctes), vim.log.levels.INFO)

  -- Find the actual WITH line
  local with_line_found = false
  for i, line in ipairs(query_lines) do
    if line:upper():match("^%s*WITH%s") then
      table.insert(test_lines, line)
      with_line_found = true

      -- If WITH and first CTE are on same line (e.g., "WITH daily_ranges AS (")
      -- We need to handle this specially
      if line:match("WITH%s+[%w_]+%s+AS%s*%(") then
        -- The WITH and CTE name are on the same line
        -- Continue from next line
        local cte = ctes[1]
        for line_num = cte.start_line + 1, cte.end_line do
          if query_lines[line_num] then
            table.insert(test_lines, query_lines[line_num])
          end
        end

        -- If we only want the first CTE, we're done
        if cte_index == 1 then
          -- Remove trailing comma if present
          local last_line = test_lines[#test_lines]
          test_lines[#test_lines] = last_line:gsub(",%s*$", "")
        end
      else
        -- WITH is on its own line, copy CTEs normally
        for j = 1, cte_index do
          local cte = ctes[j]
          for line_num = cte.start_line, cte.end_line do
            local line = query_lines[line_num]

            -- For the last CTE, ensure it doesn't end with comma
            if j == cte_index and line_num == cte.end_line then
              line = line:gsub(",%s*$", "")
            end

            table.insert(test_lines, line)
          end

          -- Add comma after CTE if not the last one
          if j < cte_index then
            local last_line = test_lines[#test_lines]
            if not last_line:match(",%s*$") then
              test_lines[#test_lines] = last_line .. ","
            end
          end
        end
      end

      break
    end
  end

  -- If we didn't find WITH, add it
  if not with_line_found then
    table.insert(test_lines, "WITH")

    -- Add all CTEs up to and including the target CTE
    for i = 1, cte_index do
      local cte = ctes[i]

      -- Copy CTE lines
      for line_num = cte.start_line, cte.end_line do
        local line = query_lines[line_num]

        -- For the last CTE, ensure it doesn't end with comma
        if i == cte_index and line_num == cte.end_line then
          line = line:gsub(",%s*$", "")
        end

        table.insert(test_lines, line)
      end

      -- Add comma after CTE if not the last one
      if i < cte_index then
        local last_line = test_lines[#test_lines]
        if not last_line:match(",%s*$") then
          test_lines[#test_lines] = last_line .. ","
        end
      end
    end
  end

  -- Add SELECT * FROM last_cte
  local target_cte = ctes[cte_index]
  table.insert(test_lines, "SELECT *")
  table.insert(test_lines, "FROM " .. target_cte.name .. ";")

  local generated_query = table.concat(test_lines, "\n")

  -- Debug: Show generated query
  vim.notify("Generated test query:\n" .. generated_query:sub(1, 200) .. "...", vim.log.levels.DEBUG)

  return generated_query
end

-- Test CTE at cursor
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
  local query_lines = vim.list_slice(lines, start_line, end_line)

  -- Parse CTEs
  local ctes = M.parse_ctes(query_lines)

  if #ctes == 0 then
    vim.notify("No CTEs found in query", vim.log.levels.WARN)
    return
  end

  -- Find CTE at cursor (adjust cursor line to query-relative position)
  local relative_cursor = cursor_line - start_line + 1
  local cte, cte_index = M.find_cte_at_cursor(ctes, relative_cursor)

  if not cte then
    -- If not in a CTE, test up to the last CTE
    cte_index = #ctes
    cte = ctes[cte_index]
    vim.notify("Testing up to last CTE: " .. cte.name, vim.log.levels.INFO)
  else
    vim.notify("Testing up to CTE: " .. cte.name, vim.log.levels.INFO)
  end

  -- Generate test query
  local test_query = M.generate_test_query(query_lines, cte_index, ctes)

  -- Execute the test query
  local executor = require('sql-cli.executor')
  executor.execute_query(test_query, config, state, false)
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
  local query_lines = vim.list_slice(lines, start_line, end_line)

  -- Parse CTEs
  local ctes = M.parse_ctes(query_lines)

  if #ctes == 0 then
    vim.notify("No CTEs found in query", vim.log.levels.WARN)
    return
  end

  -- Find CTE at cursor
  local relative_cursor = cursor_line - start_line + 1
  local cte, cte_index = M.find_cte_at_cursor(ctes, relative_cursor)

  if not cte then
    cte_index = #ctes
    cte = ctes[cte_index]
  end

  -- Generate test query
  local test_query = M.generate_test_query(query_lines, cte_index, ctes)

  -- Create new buffer with test query
  vim.cmd("new")
  vim.bo.filetype = "sql"
  vim.api.nvim_buf_set_lines(0, 0, -1, false, vim.split(test_query, "\n"))

  vim.notify("Created test query for CTE: " .. cte.name, vim.log.levels.INFO)
end

return M