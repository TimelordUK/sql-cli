-- SQL CLI Table Navigation Module
-- Excel-like navigation for query result tables

local export = require('sql-cli.export')

local M = {}

-- Parse table structure within specific line boundaries
local function parse_table_at_position(lines, start_line, end_line)
  local table_info = {
    header_row = nil,
    separator_row = nil,
    data_start = nil,
    data_end = nil,
    columns = {},
    column_positions = {},
    style = nil
  }

  -- Look only within the specified range
  for i = start_line, end_line do
    local line = lines[i]
    if not line then break end

    -- ASCII table style
    if line:match("^%+%-") then
      table_info.style = "ascii"

      if not table_info.header_row and i < end_line then
        -- Check if next line is a header
        local next_line = lines[i + 1]
        if next_line and next_line:match("^|") then
          table_info.header_row = i + 1  -- Absolute line number
          -- Look for separator
          if i + 2 <= end_line and lines[i + 2] and lines[i + 2]:match("^%+%-") then
            table_info.separator_row = i + 2  -- Absolute line number
            table_info.data_start = i + 3     -- Absolute line number
          end
        end
      elseif table_info.data_start and not table_info.data_end and line:match("^%+%-") then
        -- This might be the end border
        table_info.data_end = i - 1  -- Absolute line number
      end
    -- Data rows
    elseif line:match("^|") and table_info.data_start and not table_info.data_end then
      -- Continue counting data rows
    end
  end

  -- Set data_end if not found (scan forward from data_start to find last data row)
  if table_info.data_start and not table_info.data_end then
    -- Find the last data row (before the end border)
    for i = table_info.data_start, end_line do
      local line = lines[i]
      if line and line:match("^%+%-") then
        -- Found end border
        table_info.data_end = i - 1
        break
      elseif i == end_line then
        -- Reached end of table range
        table_info.data_end = end_line
      end
    end
  end

  -- Parse column positions from header
  if table_info.header_row and lines[table_info.header_row] then
    local header_line = lines[table_info.header_row]
    local col_start = 1

    for pos = 1, #header_line do
      if header_line:sub(pos, pos) == "|" and pos > 1 then
        table.insert(table_info.column_positions, {start = col_start, stop = pos - 1})
        col_start = pos + 1
      end
    end
    -- Add the last column
    if col_start <= #header_line then
      table.insert(table_info.column_positions, {start = col_start, stop = #header_line})
    end

    -- Extract column names
    for _, col_pos in ipairs(table_info.column_positions) do
      local col_text = header_line:sub(col_pos.start, col_pos.stop):match("^%s*(.-)%s*$") or ""
      table.insert(table_info.columns, col_text)
    end
  end

  return table_info
end

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
function M.init_navigation(bufnr, window)
  nav_state.buffer = bufnr
  -- Use provided window or find the window containing the buffer
  if window then
    nav_state.window = window
  else
    -- Find the window that contains this buffer
    for _, win in ipairs(vim.api.nvim_list_wins()) do
      if vim.api.nvim_win_get_buf(win) == bufnr then
        nav_state.window = win
        break
      end
    end
    -- Fallback to current window if no window found
    if not nav_state.window then
      nav_state.window = vim.api.nvim_get_current_win()
    end
  end

  -- Use the lookup registry to find table at cursor position
  local multi_table_nav = require('sql-cli.multi_table_nav')
  local registry = multi_table_nav.get_table_registry(bufnr)

  local cursor_line = 1
  if nav_state.window and vim.api.nvim_win_is_valid(nav_state.window) then
    local cursor = vim.api.nvim_win_get_cursor(nav_state.window)
    cursor_line = cursor[1]
  end

  -- Find the table containing the cursor
  local current_registry_table = nil
  for _, tbl in ipairs(registry.tables) do
    if cursor_line >= tbl.start_line and cursor_line <= tbl.end_line then
      current_registry_table = tbl
      break
    end
  end

  -- Get buffer lines for fallback parsing or debug output
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  if current_registry_table then
    -- Convert registry format to nav_state format
    nav_state.table_info = {
      header_row = current_registry_table.header_line,
      separator_row = current_registry_table.separator_line,
      data_start = current_registry_table.data_start,
      data_end = current_registry_table.data_end,
      columns = current_registry_table.column_names,
      column_positions = current_registry_table.column_positions,
      style = current_registry_table.style or "ascii"
    }
  else
    -- Fallback to old behavior if no table found at cursor
    nav_state.table_info = parse_table_structure(lines)
  end

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
function M.disable_navigation(config)
  if nav_state.buffer and vim.api.nvim_buf_is_valid(nav_state.buffer) then
    -- Clear highlights
    vim.api.nvim_buf_clear_namespace(nav_state.buffer, nav_state.highlight_ns, 0, -1)

    -- Remove buffer variable
    vim.b[nav_state.buffer].sql_cli_table_nav_active = false

    -- Clear keymaps
    local opts = { buffer = nav_state.buffer }

    -- Get configuration or use defaults
    config = config or {}
    local table_config = config.table_navigation or {}
    local hijack_hjkl = table_config.hijack_hjkl ~= false

    if hijack_hjkl then
      pcall(vim.keymap.del, "n", "h", opts)
      pcall(vim.keymap.del, "n", "j", opts)
      pcall(vim.keymap.del, "n", "k", opts)
      pcall(vim.keymap.del, "n", "l", opts)
    else
      pcall(vim.keymap.del, "n", "<Left>", opts)
      pcall(vim.keymap.del, "n", "<Down>", opts)
      pcall(vim.keymap.del, "n", "<Up>", opts)
      pcall(vim.keymap.del, "n", "<Right>", opts)
    end

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

-- Clear navigation state (for multi-table navigation)
function M.clear_navigation()
  -- Don't remove keymaps, just clear the state data
  nav_state.current_row = 1
  nav_state.current_col = 1
  nav_state.table_info = nil
  nav_state.buffer = nil
  nav_state.window = nil

  -- Clear any existing highlights
  if nav_state.highlight_ns then
    for _, bufnr in ipairs(vim.api.nvim_list_bufs()) do
      if vim.api.nvim_buf_is_valid(bufnr) then
        vim.api.nvim_buf_clear_namespace(bufnr, nav_state.highlight_ns, 0, -1)
      end
    end
  end
end

-- Toggle navigation mode
function M.toggle_navigation(bufnr, config)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  config = config or {}

  if nav_state.buffer == bufnr and M.is_active() then
    M.disable_navigation(config)
    vim.notify("Table navigation disabled", vim.log.levels.INFO)
  else
    vim.notify("Looking for table patterns like +---, ┌, ├, │, or |", vim.log.levels.INFO)
    if M.init_navigation(bufnr) then
      M.setup_keymaps(bufnr, config)
      local nav_keys = config.table_navigation and config.table_navigation.hijack_hjkl == false and "arrow keys" or "h/j/k/l"
      vim.notify("Table navigation enabled (" .. nav_keys .. " to move) - " .. M.get_status(), vim.log.levels.INFO)
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

    -- Move cursor to the cell in the output window
    if nav_state.window and vim.api.nvim_win_is_valid(nav_state.window) then
      -- Set cursor position
      vim.api.nvim_win_set_cursor(nav_state.window, {line_num, col_pos.start})

      -- Ensure header remains visible by managing the viewport
      local header_line = nav_state.table_info.header_row
      if header_line then
        local win_height = vim.api.nvim_win_get_height(nav_state.window)
        local current_top = vim.fn.line('w0', nav_state.window)

        -- If header is above the viewport, scroll up to show it
        if header_line < current_top then
          -- Calculate how many lines to show above header (for context)
          local context_lines = 2
          local target_top = math.max(1, header_line - context_lines)

          -- Scroll window to show header with some context
          vim.api.nvim_win_call(nav_state.window, function()
            vim.cmd('normal! ' .. target_top .. 'zt')
            -- Then move cursor back to the data cell
            vim.api.nvim_win_set_cursor(nav_state.window, {line_num, col_pos.start})
          end)
        elseif line_num > current_top + win_height - 3 then
          -- If cursor would go below visible area, scroll but keep header visible
          local max_scroll = header_line + win_height - 5  -- Keep some buffer
          if line_num <= max_scroll then
            -- Normal scrolling - cursor in middle of window
            vim.api.nvim_win_call(nav_state.window, function()
              vim.cmd('normal! zz')
            end)
          else
            -- We're at the limit - ensure header stays visible
            vim.api.nvim_win_call(nav_state.window, function()
              vim.cmd('normal! ' .. header_line .. 'zt')
              -- Scroll down as much as possible while keeping header
              local lines_to_scroll = math.min(line_num - header_line - 2, win_height - 5)
              if lines_to_scroll > 0 then
                vim.cmd('normal! ' .. lines_to_scroll .. 'j')
                vim.cmd('normal! zt')
              end
              -- Set cursor to actual position
              vim.api.nvim_win_set_cursor(nav_state.window, {line_num, col_pos.start})
            end)
          end
        end
      end
    end
  end
end

-- Navigation functions
function M.move_left()
  -- Auto-initialize if table_info is nil (after multi-table navigation)
  if not nav_state.table_info then
    local bufnr = vim.api.nvim_get_current_buf()
    local win = vim.api.nvim_get_current_win()
    M.init_navigation(bufnr, win)
  end

  if nav_state.current_col > 1 then
    nav_state.current_col = nav_state.current_col - 1
    M.highlight_current_cell()
  end
end

function M.move_right()
  -- Auto-initialize if table_info is nil (after multi-table navigation)
  if not nav_state.table_info then
    local bufnr = vim.api.nvim_get_current_buf()
    local win = vim.api.nvim_get_current_win()
    M.init_navigation(bufnr, win)
  end

  if nav_state.table_info and nav_state.current_col < #nav_state.table_info.column_positions then
    nav_state.current_col = nav_state.current_col + 1
    M.highlight_current_cell()
  end
end

function M.move_up()
  -- Auto-initialize if table_info is nil (after multi-table navigation)
  if not nav_state.table_info then
    local bufnr = vim.api.nvim_get_current_buf()
    local win = vim.api.nvim_get_current_win()
    M.init_navigation(bufnr, win)
  end

  if nav_state.current_row > 1 then
    nav_state.current_row = nav_state.current_row - 1
    M.highlight_current_cell()
  end
end

function M.move_down()
  -- Auto-initialize if table_info is nil (after multi-table navigation)
  if not nav_state.table_info then
    local bufnr = vim.api.nvim_get_current_buf()
    local win = vim.api.nvim_get_current_win()
    M.init_navigation(bufnr, win)
  end

  if nav_state.table_info then
    local max_row = nav_state.table_info.data_end - nav_state.table_info.data_start + 1
    if nav_state.current_row < max_row then
      nav_state.current_row = nav_state.current_row + 1
      M.highlight_current_cell()
    end
  end
end

function M.go_to_first_column()
  if not nav_state.table_info then
    vim.notify("No table navigation active", vim.log.levels.WARN)
    return
  end
  nav_state.current_col = 1
  M.highlight_current_cell()
end

function M.go_to_last_column()
  if not nav_state.table_info or not nav_state.table_info.column_positions then
    vim.notify("No table navigation active", vim.log.levels.WARN)
    return
  end
  nav_state.current_col = #nav_state.table_info.column_positions
  M.highlight_current_cell()
end

function M.go_to_first_row()
  if not nav_state.table_info then
    vim.notify("No table navigation active", vim.log.levels.WARN)
    return
  end
  nav_state.current_row = 1
  M.highlight_current_cell()
end

function M.go_to_last_row()
  if not nav_state.table_info or not nav_state.table_info.data_end or not nav_state.table_info.data_start then
    vim.notify("No table navigation active", vim.log.levels.WARN)
    return
  end
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

-- Yank column as JSON array (for use in WEB CTE bodies)
function M.yank_column_as_json()
  if not nav_state.table_info or not nav_state.buffer then
    vim.notify("No table navigation active", vim.log.levels.WARN)
    return
  end

  local lines = vim.api.nvim_buf_get_lines(nav_state.buffer, 0, -1, false)
  local values = {}
  local max_row = nav_state.table_info.data_end - nav_state.table_info.data_start + 1

  -- Collect all values from the column
  for row = 1, max_row do
    local value = get_cell_value(lines, nav_state.table_info, row, nav_state.current_col)
    if value and value ~= "" then
      -- Detect if value is a number or string
      local num_value = tonumber(value)
      if num_value then
        -- It's a number, add without quotes
        table.insert(values, tostring(num_value))
      else
        -- It's a string, add with quotes and escape internal quotes
        local escaped_value = value:gsub('"', '\\"')
        table.insert(values, '"' .. escaped_value .. '"')
      end
    end
  end

  -- Format as JSON array
  local json_array = "[" .. table.concat(values, ", ") .. "]"

  -- Copy to clipboard
  vim.fn.setreg('"', json_array)
  vim.fn.setreg('+', json_array) -- System clipboard

  local col_name = nav_state.table_info.columns[nav_state.current_col] or ("Column " .. nav_state.current_col)
  vim.notify("Yanked column '" .. col_name .. "' as JSON array (" .. #values .. " values)", vim.log.levels.INFO)
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
function M.setup_keymaps(bufnr, config)
  local opts = { noremap = true, silent = true, buffer = bufnr }

  -- Store that we're in table nav mode
  vim.b[bufnr].sql_cli_table_nav_active = true

  -- Get configuration or use defaults
  config = config or {}
  local table_config = config.table_navigation or {}
  local hijack_hjkl = table_config.hijack_hjkl ~= false  -- Default to true for backward compatibility

  -- Navigation with visual feedback
  if hijack_hjkl then
    -- Traditional hjkl navigation
    vim.keymap.set("n", "h", function()
      M.move_left()
    end, opts)

    vim.keymap.set("n", "j", function()
      M.move_down()
    end, opts)

    vim.keymap.set("n", "k", function()
      M.move_up()
    end, opts)

    vim.keymap.set("n", "l", function()
      M.move_right()
    end, opts)
  else
    -- Alternative arrow-key style navigation (preserves hjkl for normal vim movement)
    vim.keymap.set("n", "<Left>", function()
      M.move_left()
    end, opts)

    vim.keymap.set("n", "<Down>", function()
      M.move_down()
    end, opts)

    vim.keymap.set("n", "<Up>", function()
      M.move_up()
    end, opts)

    vim.keymap.set("n", "<Right>", function()
      M.move_right()
    end, opts)
  end

  -- Jump to boundaries
  vim.keymap.set("n", "0", M.go_to_first_column, opts)
  vim.keymap.set("n", "$", M.go_to_last_column, opts)
  vim.keymap.set("n", "gg", M.go_to_first_row, opts)
  vim.keymap.set("n", "G", M.go_to_last_row, opts)

  -- Yank operations (basic cell operations)
  vim.keymap.set("n", "yy", M.yank_cell, opts)
  vim.keymap.set("n", "Y", M.yank_row, opts)
  vim.keymap.set("n", "yc", M.yank_column, opts)

  -- Yank column as JSON array (for WEB CTE bodies)
  vim.keymap.set("n", "<leader>sYj", M.yank_column_as_json, opts)

  -- Export operations with \s prefix
  vim.keymap.set("n", "<leader>se", function()
    export.show_export_menu(bufnr, nav_state.table_info)
  end, opts)

  vim.keymap.set("n", "<leader>sb", function()
    export.yank_as_html(bufnr, nav_state.table_info, true)   -- Open in browser
  end, opts)

  vim.keymap.set("n", "<leader>sH", function()
    export.yank_as_html(bufnr, nav_state.table_info, false)  -- Just yank HTML code
  end, opts)

  vim.keymap.set("n", "<leader>sm", function()
    export.yank_as_markdown(bufnr, nav_state.table_info)
  end, opts)

  vim.keymap.set("n", "<leader>st", function()
    export.yank_as_tsv(bufnr, nav_state.table_info)
  end, opts)

  vim.keymap.set("n", "<leader>si", function()
    vim.ui.input({ prompt = "Table name: ", default = "my_table" }, function(name)
      if name then
        export.yank_as_insert(bufnr, nav_state.table_info, name)
      end
    end)
  end, opts)

  vim.keymap.set("n", "<leader>sC", function()
    vim.ui.input({ prompt = "Table name: ", default = "my_table" }, function(name)
      if name then
        export.yank_create_table(bufnr, nav_state.table_info, name)
      end
    end)
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