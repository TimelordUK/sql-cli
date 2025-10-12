#!/usr/bin/env lua

-- Simple test to verify CTE termination fix
-- This simulates what the Lua plugin does

local function generate_simple_test_query(query_lines, target_position)
  local test_lines = {}
  local with_found = false
  local paren_depth = 0
  local current_cte_idx = 0

  for i, line in ipairs(query_lines) do
    local upper = line:upper()
    local trimmed = line:match("^%s*(.-)%s*$")  -- vim.trim equivalent

    -- Skip pure comment lines when looking for WITH
    if not with_found and trimmed:match("^%-%-") then
      goto continue
    end

    -- Look for WITH to start
    if not with_found then
      if upper:match("WITH%s") or upper:match("WITH$") then
        with_found = true
        print("Found WITH clause")
        table.insert(test_lines, line)
        -- Check if first CTE is on same line
        if line:match("WITH%s+([%w_]+)%s+AS%s*%(") then
          current_cte_idx = 1
          print("First CTE on same line as WITH")
        end
      end
      goto continue
    end

    -- Check for new CTE definitions (after WITH found)
    local cte_name = line:match("^%s*([%w_]+)%s+AS%s*%(")
    if cte_name then
      current_cte_idx = current_cte_idx + 1
      print(string.format("Found CTE %d: %s", current_cte_idx, cte_name))

      -- If we've gone past our target, don't include this line
      if current_cte_idx > target_position then
        -- Clean up trailing comma from previous line
        if #test_lines > 0 then
          test_lines[#test_lines] = test_lines[#test_lines]:gsub(",%s*$", "")
        end
        print("Stopping - past target CTE")
        break
      end
    end

    -- Add the current line
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
      print(string.format("Target CTE complete at line %d", i))
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
      print("Found main SELECT, stopping")
      break
    end

    ::continue::
  end

  return test_lines
end

-- Test with the sample query
local query_lines = {
  "-- Test CTE chain termination",
  "WITH cte1 AS (",
  "    SELECT value as n",
  "    FROM RANGE(1, 10)",
  "),",
  "cte2 AS (",
  "    SELECT n, n * 2 as doubled",
  "    FROM cte1",
  "    WHERE n > 3",
  "),",
  "cte3 AS (",
  "    SELECT doubled, doubled * 10 as final",
  "    FROM cte2",
  "    WHERE doubled < 15",
  ")",
  "SELECT * FROM cte3",
  "WHERE final > 50;"
}

print("\n=== Testing CTE 2 termination ===")
local result = generate_simple_test_query(query_lines, 2)
print("\n--- Result ---")
for _, line in ipairs(result) do
  print(line)
end

print("\n--- Should NOT contain cte3 or final SELECT ---")
local result_str = table.concat(result, "\n")
if result_str:match("cte3") then
  print("ERROR: Contains cte3!")
else
  print("PASS: Does not contain cte3")
end
if result_str:match("SELECT.*FROM.*cte3") then
  print("ERROR: Contains final SELECT!")
else
  print("PASS: Does not contain final SELECT")
end
