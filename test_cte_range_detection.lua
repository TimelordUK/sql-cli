#!/usr/bin/env lua

-- Test CTE range detection logic
local function detect_cte_ranges(query_lines)
  local cte_line_ranges = {}
  local current_cte_idx = 0
  local paren_depth = 0
  local cte_start_line = nil

  for i, line in ipairs(query_lines) do
    local upper = line:upper()

    -- Check for CTE definition
    if line:match("^%s*([%w_]+)%s+AS%s*%(") then
      -- New CTE starts (not on WITH line)
      current_cte_idx = current_cte_idx + 1
      cte_start_line = i
      paren_depth = 0
      print(string.format("Line %d: Found CTE %d (pattern 1)", i, current_cte_idx))
    elseif upper:match("WITH%s+") and line:match("WITH%s+([%w_]+)%s+AS%s*%(") then
      -- CTE starts on WITH line
      if current_cte_idx == 0 then  -- Only count if we haven't already
        current_cte_idx = 1
        cte_start_line = i
        paren_depth = 0
        print(string.format("Line %d: Found CTE %d (WITH line)", i, current_cte_idx))
      end
    elseif upper:match("^%s*WITH%s*$") then
      -- WITH on its own line, CTE will follow
      print(string.format("Line %d: Found WITH keyword alone", i))
    end

    -- Track parentheses to find CTE end
    if cte_start_line then
      for char in line:gmatch(".") do
        if char == "(" then paren_depth = paren_depth + 1 end
        if char == ")" then paren_depth = paren_depth - 1 end
      end

      print(string.format("Line %d: paren_depth=%d, line=%s", i, paren_depth, line))

      -- If parentheses balanced, CTE ends here
      if paren_depth == 0 and line:match("%)") then
        table.insert(cte_line_ranges, {
          cte_index = current_cte_idx,
          start_line = cte_start_line,
          end_line = i
        })
        print(string.format("Line %d: CTE %d complete (lines %d-%d)", i, current_cte_idx, cte_start_line, i))
        cte_start_line = nil
      end
    end
  end

  return cte_line_ranges
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

print("\n=== Testing CTE Range Detection ===\n")
local ranges = detect_cte_ranges(query_lines)

print("\n--- Detected Ranges ---")
for _, range in ipairs(ranges) do
  print(string.format("CTE %d: lines %d-%d", range.cte_index, range.start_line, range.end_line))
end

-- Test cursor positions
print("\n--- Testing Cursor Positions ---")
local test_cursors = {7, 9, 13}  -- Lines in cte2
for _, cursor_line in ipairs(test_cursors) do
  local found = false
  for _, range in ipairs(ranges) do
    if cursor_line >= range.start_line and cursor_line <= range.end_line then
      print(string.format("Cursor at line %d -> CTE %d", cursor_line, range.cte_index))
      found = true
      break
    end
  end
  if not found then
    print(string.format("Cursor at line %d -> NOT FOUND (would default to last CTE!)", cursor_line))
  end
end
