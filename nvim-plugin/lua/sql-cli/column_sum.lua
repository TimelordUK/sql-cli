-- Column sum calculator for SQL result tables
-- Provides quick sum/statistics for numeric columns

local M = {}

-- Parse a number from string, handling various formats
local function parse_number(str)
  if not str or str == "" or str == "NULL" then
    return nil
  end

  -- Remove common formatting
  local cleaned = str:gsub(",", "")  -- Remove thousand separators
  cleaned = cleaned:gsub("[$£€¥]", "")  -- Remove currency symbols
  cleaned = cleaned:gsub("^%s+", ""):gsub("%s+$", "")  -- Trim whitespace

  -- Try to convert to number
  local num = tonumber(cleaned)
  return num
end

-- Get column index from cursor position
local function get_column_at_cursor(line, col_pos)
  -- Count pipe characters before cursor position
  local column_idx = 0
  local current_col_start = 0

  for i = 1, col_pos do
    local char = line:sub(i, i)
    if char == "|" then
      column_idx = column_idx + 1
      current_col_start = i
    end
  end

  -- Find the end of current column
  local current_col_end = line:find("|", col_pos) or #line

  -- Extract column content
  local column_text = line:sub(current_col_start + 1, current_col_end - 1)
  column_text = column_text:gsub("^%s+", ""):gsub("%s+$", "")

  return column_idx, column_text
end

-- Parse table structure from buffer
local function parse_table_structure(bufnr)
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find table boundaries
  local header_line = nil
  local separator_line = nil
  local data_start = nil
  local data_end = nil

  for i, line in ipairs(lines) do
    if line:match("^%+%-") or line:match("^[├┌└│─┬┴┼═╪╫╬]+") then
      if not header_line then
        -- Look for header on next line
        if i < #lines and lines[i + 1]:match("^|") then
          header_line = i + 1
          separator_line = i
        end
      elseif header_line and i > header_line + 1 then
        -- Found the end separator
        data_end = i - 1
        break
      end
    elseif header_line and not data_start and line:match("^|") and i > header_line + 1 then
      data_start = i
    end
  end

  if not header_line then
    return nil
  end

  -- Parse headers
  local header = lines[header_line]
  local headers = {}
  local col_positions = {}

  -- Split by pipe and clean
  for col in header:gmatch("([^|]+)") do
    col = col:gsub("^%s+", ""):gsub("%s+$", "")
    if col ~= "" then
      table.insert(headers, col)
    end
  end

  return {
    headers = headers,
    header_line = header_line,
    data_start = data_start or header_line + 2,
    data_end = data_end or #lines,
    lines = lines
  }
end

-- Extract column data by index
local function extract_column_data(table_info, column_idx)
  local data = {}

  for i = table_info.data_start, table_info.data_end do
    local line = table_info.lines[i]
    if line:match("^|") then
      -- Split by pipe
      local cols = {}
      for col in line:gmatch("([^|]+)") do
        col = col:gsub("^%s+", ""):gsub("%s+$", "")
        table.insert(cols, col)
      end

      if cols[column_idx] then
        local num = parse_number(cols[column_idx])
        if num then
          table.insert(data, num)
        end
      end
    end
  end

  return data
end

-- Calculate statistics for a column
local function calculate_stats(data)
  if #data == 0 then
    return nil
  end

  local sum = 0
  local min = data[1]
  local max = data[1]

  for _, val in ipairs(data) do
    sum = sum + val
    if val < min then min = val end
    if val > max then max = val end
  end

  local avg = sum / #data

  -- Calculate median
  local sorted = vim.deepcopy(data)
  table.sort(sorted)
  local median
  if #sorted % 2 == 0 then
    median = (sorted[#sorted / 2] + sorted[#sorted / 2 + 1]) / 2
  else
    median = sorted[math.ceil(#sorted / 2)]
  end

  return {
    count = #data,
    sum = sum,
    avg = avg,
    min = min,
    max = max,
    median = median
  }
end

-- Format number with thousand separators
local function format_number(num)
  -- Round to 2 decimal places if needed
  if num ~= math.floor(num) then
    num = string.format("%.2f", num)
  else
    num = tostring(math.floor(num))
  end

  -- Add thousand separators
  local formatted = tostring(num)
  local k
  while true do
    formatted, k = formatted:gsub("^(-?%d+)(%d%d%d)", "%1,%2")
    if k == 0 then break end
  end

  return formatted
end

-- Main function: Calculate sum for column at cursor
function M.sum_column_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor = vim.api.nvim_win_get_cursor(0)
  local line = cursor[1]
  local col = cursor[2]

  -- Get current line text
  local line_text = vim.api.nvim_buf_get_lines(bufnr, line - 1, line, false)[1]
  if not line_text or not line_text:match("^|") then
    vim.notify("Cursor must be on a table row", vim.log.levels.WARN)
    return
  end

  -- Get column index
  local column_idx, column_value = get_column_at_cursor(line_text, col + 1)

  -- Parse table structure
  local table_info = parse_table_structure(bufnr)
  if not table_info then
    vim.notify("Could not parse table structure", vim.log.levels.ERROR)
    return
  end

  -- Get column name
  local column_name = table_info.headers[column_idx] or ("Column " .. column_idx)

  -- Extract column data
  local data = extract_column_data(table_info, column_idx)

  if #data == 0 then
    vim.notify(string.format("Column '%s' contains no numeric values", column_name), vim.log.levels.INFO)
    return
  end

  -- Calculate statistics
  local stats = calculate_stats(data)

  -- Show results in a floating window
  M.show_stats_popup(column_name, stats)
end

-- Show statistics in a floating window
function M.show_stats_popup(column_name, stats)
  -- Create buffer for popup
  local buf = vim.api.nvim_create_buf(false, true)

  -- Format the statistics
  local lines = {
    string.format(" Column: %s ", column_name),
    " " .. string.rep("─", 30) .. " ",
    string.format(" Count:  %s rows", format_number(stats.count)),
    string.format(" Sum:    %s", format_number(stats.sum)),
    string.format(" Avg:    %s", format_number(stats.avg)),
    string.format(" Min:    %s", format_number(stats.min)),
    string.format(" Max:    %s", format_number(stats.max)),
    string.format(" Median: %s", format_number(stats.median)),
    " " .. string.rep("─", 30) .. " ",
    " Press 'q' or <Esc> to close ",
  }

  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

  -- Calculate window size
  local width = 34
  local height = #lines

  -- Create floating window
  local win = vim.api.nvim_open_win(buf, true, {
    relative = 'cursor',
    row = 1,
    col = 0,
    width = width,
    height = height,
    style = 'minimal',
    border = 'rounded',
    title = ' Column Statistics ',
    title_pos = 'center'
  })

  -- Set buffer options
  vim.api.nvim_buf_set_option(buf, 'modifiable', false)
  vim.api.nvim_buf_set_option(buf, 'bufhidden', 'delete')

  -- Add keymaps to close
  local opts = { buffer = buf, noremap = true, silent = true }
  vim.keymap.set('n', 'q', function()
    vim.api.nvim_win_close(win, true)
  end, opts)
  vim.keymap.set('n', '<Esc>', function()
    vim.api.nvim_win_close(win, true)
  end, opts)

  -- Optional: Copy sum to clipboard
  vim.keymap.set('n', 'y', function()
    vim.fn.setreg('+', tostring(stats.sum))
    vim.fn.setreg('"', tostring(stats.sum))
    vim.notify(string.format("Sum (%s) copied to clipboard", format_number(stats.sum)), vim.log.levels.INFO)
  end, vim.tbl_extend("force", opts, { desc = "Copy sum to clipboard" }))
end

-- Quick sum all numeric columns and show totals row
function M.show_all_totals()
  local bufnr = vim.api.nvim_get_current_buf()

  -- Parse table structure
  local table_info = parse_table_structure(bufnr)
  if not table_info then
    vim.notify("Could not parse table structure", vim.log.levels.ERROR)
    return
  end

  -- Calculate totals for each column
  local totals = {}
  local has_numeric = false

  for i, header in ipairs(table_info.headers) do
    local data = extract_column_data(table_info, i)
    if #data > 0 then
      local stats = calculate_stats(data)
      totals[i] = stats.sum
      has_numeric = true
    else
      totals[i] = nil
    end
  end

  if not has_numeric then
    vim.notify("No numeric columns found in table", vim.log.levels.INFO)
    return
  end

  -- Build totals row
  local total_cells = {}
  for i, header in ipairs(table_info.headers) do
    if totals[i] then
      table.insert(total_cells, format_number(totals[i]))
    else
      if i == 1 then
        table.insert(total_cells, "TOTAL")
      else
        table.insert(total_cells, "")
      end
    end
  end

  -- Create separator and total row
  local separator = "├" .. string.rep("─", vim.fn.strdisplaywidth(table_info.lines[table_info.header_line]) - 2) .. "┤"
  local total_row = "│ " .. table.concat(total_cells, " │ ") .. " │"

  -- Insert at end of table
  local insert_line = table_info.data_end + 1

  -- Make buffer modifiable temporarily
  vim.bo[bufnr].modifiable = true

  -- Insert the separator and total row
  vim.api.nvim_buf_set_lines(bufnr, insert_line, insert_line, false, {separator, total_row})

  -- Make buffer non-modifiable again
  vim.bo[bufnr].modifiable = false

  vim.notify("Totals row added to table", vim.log.levels.INFO)
end

return M