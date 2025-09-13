-- SQL CLI Table Navigation Module
-- Excel-like navigation for query result tables

local export = require('sql-cli.export')

local M = {}

-- Table parser to understand the structure
local function parse_table_structure(lines)
  local table_info = {
    header_row = nil,
    separator_row = nil,
    data_start = nil,
    data_end = nil,
    columns = {},
    column_positions = {},
    style = nil -- "ascii" for +---+ style, "box" for ┌─┐ style, "pipe" for | style
  }

  local found_table = false

  -- Detect table style and find header
  for i, line in ipairs(lines) do
    -- Skip empty lines and header comments
    if line:match("^%-%-") or line:match("^#") or line:match("^%s*$") then
      -- Skip comments and empty lines
    -- ASCII table style with +---+---+ borders (MOST COMMON)
    elseif line:match("^%+%-") then
      found_table = true
      table_info.style = "ascii"

      if not table_info.header_row then
        -- First +--- line is the top border
        -- Next line should be the header
        if i < #lines and lines[i+1]:match("^|") then
          table_info.header_row = i + 1
          -- Look for separator after header
          if i + 2 <= #lines and lines[i+2]:match("^%+%-") then
            table_info.separator_row = i + 2
            table_info.data_start = i + 3
          end
        end
      elseif table_info.separator_row and i == table_info.separator_row then
        -- This is the separator row, skip it
      elseif table_info.data_start and not table_info.data_end then
        -- This is the bottom border after we've started reading data
        table_info.data_end = i - 1
        break
      end
    -- Data rows in ASCII table
    elseif table_info.data_start and line:match("^|") then
      -- Keep tracking data rows (data_end will be set when we hit the bottom border)
    -- Box drawing style
    elseif line:match("^┌") or line:match("^├") or line:match("^└") then
      found_table = true
      table_info.style = "box"
      if line:match("^├") and not table_info.separator_row then
        table_info.separator_row = i
        table_info.header_row = i - 1
        table_info.data_start = i + 1
      elseif line:match("^└") then
        table_info.data_end = i - 1
        break
      end
    -- Simple pipe style (most common from sql-cli)
    elseif line:match("^%s*|") then
      found_table = true
      if not table_info.style then
        table_info.style = "pipe"
        -- Look for header separator line
        if i < #lines and lines[i+1]:match("^%s*|%-") then
          table_info.header_row = i
          table_info.separator_row = i + 1
          table_info.data_start = i + 2
        elseif not table_info.header_row then
          -- Assume first pipe line is header if no separator found yet
          table_info.header_row = i
        end
      end
      -- Track data rows
      if table_info.data_start and i >= table_info.data_start then
        -- Keep tracking until we find a non-table line
        if i == #lines or (lines[i + 1] and not lines[i + 1]:match("^%s*|")) then
          table_info.data_end = i
        end
      end
    -- Regular pipe style (no separator)
    elseif line:match("^│") then
      found_table = true
      if not table_info.style then
        table_info.style = "box"
      end
      if not table_info.header_row and i > 1 then
        -- This might be the header if previous line was ┌───┐
        if lines[i-1]:match("^┌") then
          table_info.header_row = i
        end
      end
    end
  end

  -- If we didn't find a proper data section, try to infer it
  if found_table and table_info.header_row and not table_info.data_start then
    table_info.data_start = table_info.separator_row and (table_info.separator_row + 1) or (table_info.header_row + 1)
    -- Find data end
    for i = table_info.data_start, #lines do
      if table_info.style == "pipe" and lines[i]:match("^%s*|") then
        table_info.data_end = i
      elseif table_info.style == "box" and lines[i]:match("^│") then
        table_info.data_end = i
      elseif lines[i]:match("^└") or lines[i]:match("^%-%-") then
        table_info.data_end = i - 1
        break
      end
    end
  end

  -- Parse column positions from header or separator
  if table_info.header_row then
    local header_line = lines[table_info.header_row]
    if table_info.style == "box" then
      -- Find column positions by │ separators
      local pos = 1
      for col_start, col_end in header_line:gmatch("()│()") do
        table.insert(table_info.column_positions, {start = pos, stop = col_start - 1})
        pos = col_end
      end
      -- Last column
      if pos < #header_line then
        table.insert(table_info.column_positions, {start = pos, stop = #header_line})
      end
    elseif table_info.style == "ascii" or table_info.style == "pipe" then
      -- ASCII/Pipe style - find columns by | separators
      local positions = {}
      for pos in header_line:gmatch("()|") do
        table.insert(positions, pos)
      end

      -- Create column ranges from pipe positions
      for i = 1, #positions - 1 do
        table.insert(table_info.column_positions, {
          start = positions[i] + 1,  -- Skip the | character
          stop = positions[i + 1] - 1 -- Stop before next |
        })
      end
    end

    -- Extract column names
    for _, col_pos in ipairs(table_info.column_positions) do
      local col_text = header_line:sub(col_pos.start, col_pos.stop)
      -- Clean up the column name
      col_text = col_text:gsub("^%s*│?%s*", ""):gsub("%s*│?%s*$", "")
      col_text = col_text:gsub("^|%s*", ""):gsub("%s*|$", "")
      table.insert(table_info.columns, col_text)
    end
  end

  return table_info
end

-- Get cell value at position
local function get_cell_value(lines, table_info, row, col)
  if not table_info.data_start or not table_info.data_end then
    return nil
  end

  local line_num = table_info.data_start + row - 1
  if line_num > table_info.data_end then
    return nil
  end

  local line = lines[line_num]
  if not line then
    return nil
  end

  local col_pos = table_info.column_positions[col]
  if not col_pos then
    return nil
  end

  local value = line:sub(col_pos.start, col_pos.stop)
  -- Clean up the value
  value = value:gsub("^%s*│?%s*", ""):gsub("%s*│?%s*$", "")
  value = value:gsub("^|%s*", ""):gsub("%s*|$", "")

  return value
end

-- State for current navigation
local nav_state = {
  current_row = 1,
  current_col = 1,
  table_info = nil,
  buffer = nil,
  window = nil,
  highlight_ns = vim.api.nvim_create_namespace("sql_cli_cell_highlight")
}

-- Initialize navigation for a buffer
function M.init_navigation(bufnr)
  nav_state.buffer = bufnr
  nav_state.window = vim.api.nvim_get_current_win()

  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  nav_state.table_info = parse_table_structure(lines)

  if nav_state.table_info.data_start then
    nav_state.current_row = 1
    nav_state.current_col = 1

    -- Make buffer non-modifiable to prevent accidental edits
    vim.bo[bufnr].modifiable = false
    vim.bo[bufnr].readonly = true

    M.highlight_current_cell()
  else
    -- Debug output to see what we're parsing
    vim.notify("No table found in buffer. Buffer has " .. #lines .. " lines", vim.log.levels.WARN)
    if #lines > 0 then
      -- Show first few non-empty lines
      local shown = 0
      for i = 1, math.min(#lines, 20) do
        if lines[i] and lines[i] ~= "" and not lines[i]:match("^%-%-") then
          vim.notify("Line " .. i .. ": " .. lines[i]:sub(1, 60), vim.log.levels.INFO)
          shown = shown + 1
          if shown >= 5 then break end
        end
      end
      vim.notify("Table info: header=" .. tostring(nav_state.table_info.header_row) ..
                 ", data_start=" .. tostring(nav_state.table_info.data_start) ..
                 ", data_end=" .. tostring(nav_state.table_info.data_end) ..
                 ", style=" .. tostring(nav_state.table_info.style) ..
                 ", columns=" .. #nav_state.table_info.columns, vim.log.levels.INFO)
    end
    return false
  end

  return true
end

-- Disable navigation
function M.disable_navigation()
  if nav_state.buffer and vim.api.nvim_buf_is_valid(nav_state.buffer) then
    -- Clear highlights
    vim.api.nvim_buf_clear_namespace(nav_state.buffer, nav_state.highlight_ns, 0, -1)

    -- Remove buffer variable
    vim.b[nav_state.buffer].sql_cli_table_nav_active = false

    -- Clear keymaps
    local opts = { buffer = nav_state.buffer }
    pcall(vim.keymap.del, "n", "h", opts)
    pcall(vim.keymap.del, "n", "j", opts)
    pcall(vim.keymap.del, "n", "k", opts)
    pcall(vim.keymap.del, "n", "l", opts)
    pcall(vim.keymap.del, "n", "yy", opts)
    pcall(vim.keymap.del, "n", "Y", opts)
    pcall(vim.keymap.del, "n", "yc", opts)
  end

  -- Reset state
  nav_state.buffer = nil
  nav_state.window = nil
  nav_state.table_info = nil
  nav_state.current_row = 1
  nav_state.current_col = 1
end

-- Toggle navigation mode
function M.toggle_navigation(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()

  if nav_state.buffer == bufnr and M.is_active() then
    M.disable_navigation()
    vim.notify("Table navigation disabled", vim.log.levels.INFO)
  else
    vim.notify("Looking for table patterns like +---, ┌, ├, │, or |", vim.log.levels.INFO)
    if M.init_navigation(bufnr) then
      M.setup_keymaps(bufnr)
      vim.notify("Table navigation enabled - " .. M.get_status(), vim.log.levels.INFO)
    else
      vim.notify("Could not find a table in the buffer. Tables should have ASCII borders (+---+) or box drawing characters.", vim.log.levels.WARN)
    end
  end
end

-- Highlight current cell
function M.highlight_current_cell()
  if not nav_state.table_info or not nav_state.buffer then
    return
  end

  -- Clear previous highlights
  vim.api.nvim_buf_clear_namespace(nav_state.buffer, nav_state.highlight_ns, 0, -1)

  local line_num = nav_state.table_info.data_start + nav_state.current_row - 1
  local col_pos = nav_state.table_info.column_positions[nav_state.current_col]

  if line_num <= nav_state.table_info.data_end and col_pos then
    -- Highlight the cell
    vim.api.nvim_buf_add_highlight(
      nav_state.buffer,
      nav_state.highlight_ns,
      "Visual",
      line_num - 1, -- 0-indexed
      col_pos.start - 1,
      col_pos.stop
    )

    -- Move cursor to the cell
    if nav_state.window and vim.api.nvim_win_is_valid(nav_state.window) then
      vim.api.nvim_win_set_cursor(nav_state.window, {line_num, col_pos.start})
    end
  end
end

-- Navigation functions
function M.move_left()
  if nav_state.current_col > 1 then
    nav_state.current_col = nav_state.current_col - 1
    M.highlight_current_cell()
  end
end

function M.move_right()
  if nav_state.current_col < #nav_state.table_info.column_positions then
    nav_state.current_col = nav_state.current_col + 1
    M.highlight_current_cell()
  end
end

function M.move_up()
  if nav_state.current_row > 1 then
    nav_state.current_row = nav_state.current_row - 1
    M.highlight_current_cell()
  end
end

function M.move_down()
  local max_row = nav_state.table_info.data_end - nav_state.table_info.data_start + 1
  if nav_state.current_row < max_row then
    nav_state.current_row = nav_state.current_row + 1
    M.highlight_current_cell()
  end
end

function M.go_to_first_column()
  nav_state.current_col = 1
  M.highlight_current_cell()
end

function M.go_to_last_column()
  nav_state.current_col = #nav_state.table_info.column_positions
  M.highlight_current_cell()
end

function M.go_to_first_row()
  nav_state.current_row = 1
  M.highlight_current_cell()
end

function M.go_to_last_row()
  nav_state.current_row = nav_state.table_info.data_end - nav_state.table_info.data_start + 1
  M.highlight_current_cell()
end

-- Yank current cell value
function M.yank_cell()
  if not nav_state.table_info or not nav_state.buffer then
    vim.notify("No table navigation active", vim.log.levels.WARN)
    return
  end

  local lines = vim.api.nvim_buf_get_lines(nav_state.buffer, 0, -1, false)
  local value = get_cell_value(lines, nav_state.table_info, nav_state.current_row, nav_state.current_col)

  if value then
    vim.fn.setreg('"', value)
    vim.fn.setreg('+', value) -- System clipboard
    vim.notify("Yanked: " .. value, vim.log.levels.INFO)
  else
    vim.notify("No value at current position", vim.log.levels.WARN)
  end
end

-- Yank entire row
function M.yank_row()
  if not nav_state.table_info or not nav_state.buffer then
    vim.notify("No table navigation active", vim.log.levels.WARN)
    return
  end

  local lines = vim.api.nvim_buf_get_lines(nav_state.buffer, 0, -1, false)
  local values = {}

  for col = 1, #nav_state.table_info.column_positions do
    local value = get_cell_value(lines, nav_state.table_info, nav_state.current_row, col)
    table.insert(values, value or "")
  end

  local csv_row = table.concat(values, ",")
  vim.fn.setreg('"', csv_row)
  vim.fn.setreg('+', csv_row) -- System clipboard
  vim.notify("Yanked row: " .. csv_row, vim.log.levels.INFO)
end

-- Yank column (all values in current column)
function M.yank_column()
  if not nav_state.table_info or not nav_state.buffer then
    vim.notify("No table navigation active", vim.log.levels.WARN)
    return
  end

  local lines = vim.api.nvim_buf_get_lines(nav_state.buffer, 0, -1, false)
  local values = {}
  local max_row = nav_state.table_info.data_end - nav_state.table_info.data_start + 1

  for row = 1, max_row do
    local value = get_cell_value(lines, nav_state.table_info, row, nav_state.current_col)
    table.insert(values, value or "")
  end

  local column_data = table.concat(values, "\n")
  vim.fn.setreg('"', column_data)
  vim.fn.setreg('+', column_data) -- System clipboard

  local col_name = nav_state.table_info.columns[nav_state.current_col] or ("Column " .. nav_state.current_col)
  vim.notify("Yanked column '" .. col_name .. "' (" .. #values .. " values)", vim.log.levels.INFO)
end

-- Get current cell info (for statusline)
function M.get_cell_info()
  if not nav_state.table_info or not nav_state.table_info.data_start then
    return ""
  end

  local col_name = nav_state.table_info.columns[nav_state.current_col] or "?"
  local max_row = nav_state.table_info.data_end - nav_state.table_info.data_start + 1

  return string.format("[%d,%d] %s (%d/%d rows)",
    nav_state.current_row,
    nav_state.current_col,
    col_name,
    nav_state.current_row,
    max_row
  )
end

-- Check if navigation is active
function M.is_active()
  return nav_state.buffer ~= nil and nav_state.table_info ~= nil
end

-- Get table info (for external use)
function M.get_table_info()
  return nav_state.table_info
end

-- Get navigation mode status
function M.get_status()
  if not M.is_active() then
    return ""
  end

  local col_name = nav_state.table_info.columns[nav_state.current_col] or ("Col " .. nav_state.current_col)
  local max_row = nav_state.table_info.data_end - nav_state.table_info.data_start + 1
  local max_col = #nav_state.table_info.column_positions

  return string.format("📊 TABLE NAV [%d/%d, %d/%d] %s",
    nav_state.current_row, max_row,
    nav_state.current_col, max_col,
    col_name
  )
end

-- Setup keymaps for navigation
function M.setup_keymaps(bufnr)
  local opts = { noremap = true, silent = true, buffer = bufnr }

  -- Store that we're in table nav mode
  vim.b[bufnr].sql_cli_table_nav_active = true

  -- Navigation with visual feedback
  vim.keymap.set("n", "h", function()
    M.move_left()
    vim.notify(M.get_status(), vim.log.levels.INFO)
  end, opts)

  vim.keymap.set("n", "j", function()
    M.move_down()
    vim.notify(M.get_status(), vim.log.levels.INFO)
  end, opts)

  vim.keymap.set("n", "k", function()
    M.move_up()
    vim.notify(M.get_status(), vim.log.levels.INFO)
  end, opts)

  vim.keymap.set("n", "l", function()
    M.move_right()
    vim.notify(M.get_status(), vim.log.levels.INFO)
  end, opts)

  -- Jump to boundaries
  vim.keymap.set("n", "0", M.go_to_first_column, opts)
  vim.keymap.set("n", "$", M.go_to_last_column, opts)
  vim.keymap.set("n", "gg", M.go_to_first_row, opts)
  vim.keymap.set("n", "G", M.go_to_last_row, opts)

  -- Yank operations
  vim.keymap.set("n", "yy", M.yank_cell, opts)
  vim.keymap.set("n", "Y", M.yank_row, opts)
  vim.keymap.set("n", "yc", M.yank_column, opts)

  -- Export operations
  vim.keymap.set("n", "ye", function()
    export.show_export_menu(bufnr, nav_state.table_info)
  end, opts)

  vim.keymap.set("n", "yh", function()
    export.yank_as_html(bufnr, nav_state.table_info)
  end, opts)

  vim.keymap.set("n", "ym", function()
    export.yank_as_markdown(bufnr, nav_state.table_info)
  end, opts)

  vim.keymap.set("n", "yt", function()
    export.yank_as_tsv(bufnr, nav_state.table_info)
  end, opts)

  -- Tab navigation
  vim.keymap.set("n", "<Tab>", function()
    M.move_right()
    if nav_state.current_col == 1 and nav_state.current_row < (nav_state.table_info.data_end - nav_state.table_info.data_start + 1) then
      M.move_down()
    end
  end, opts)

  vim.keymap.set("n", "<S-Tab>", function()
    M.move_left()
    if nav_state.current_col == #nav_state.table_info.column_positions and nav_state.current_row > 1 then
      M.move_up()
      M.go_to_last_column()
    end
  end, opts)
end

return M