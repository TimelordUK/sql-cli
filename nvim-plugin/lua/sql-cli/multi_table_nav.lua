-- SQL CLI Multiple Table Navigation Module
-- Navigate between multiple result tables from scripts with GO statements

local M = {}

-- Global table registry - pre-processed table information
local table_registry = {
  buffer_id = nil,
  tables = {},
  last_updated = 0
}

-- Build complete table registry for a buffer
function M.build_table_registry(bufnr)
  if not bufnr or not vim.api.nvim_buf_is_valid(bufnr) then
    return {}
  end

  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local raw_tables = M.find_all_tables(bufnr)

  table_registry.buffer_id = bufnr
  table_registry.tables = {}
  table_registry.last_updated = vim.loop.now()

  for i, raw_table in ipairs(raw_tables) do
    local processed_table = {
      -- Basic boundaries
      start_line = raw_table.start_line,
      end_line = raw_table.end_line,
      style = raw_table.style,

      -- Header and data boundaries
      header_line = nil,
      separator_line = nil,
      data_start = nil,
      data_end = nil,

      -- Processed data
      rows = 0,
      columns = 0,
      column_positions = {},
      column_names = {},

      -- Navigation info
      first_data_cell = {line = nil, col = nil}  -- Top-left data cell coordinates
    }

    -- Find header, separator, and data boundaries
    for line_num = raw_table.start_line, raw_table.end_line do
      local line = lines[line_num]
      if not line then break end

      if line:match("^|") and not line:match("^|%-") then
        -- This is a data or header row
        if not processed_table.header_line then
          processed_table.header_line = line_num
        elseif not processed_table.data_start then
          -- First data row after header
          processed_table.data_start = line_num
        end
      elseif line:match("^%+%-+%+") and processed_table.header_line and not processed_table.separator_line then
        -- This is the separator after header
        processed_table.separator_line = line_num
        if line_num + 1 <= raw_table.end_line then
          processed_table.data_start = line_num + 1
        end
      elseif line:match("^%+%-+%+") and processed_table.data_start and not processed_table.data_end then
        -- This is the end border
        processed_table.data_end = line_num - 1
        break
      end
    end

    -- Set data_end if not found
    if processed_table.data_start and not processed_table.data_end then
      processed_table.data_end = raw_table.end_line
    end

    -- Calculate rows
    if processed_table.data_start and processed_table.data_end then
      processed_table.rows = processed_table.data_end - processed_table.data_start + 1
    end

    -- Parse column positions from header
    if processed_table.header_line and lines[processed_table.header_line] then
      local header_line = lines[processed_table.header_line]
      local col_start = 1

      for pos = 1, #header_line do
        if header_line:sub(pos, pos) == "|" and pos > 1 then
          table.insert(processed_table.column_positions, {start = col_start, stop = pos - 1})
          col_start = pos + 1
        end
      end
      -- Add the last column
      if col_start <= #header_line then
        table.insert(processed_table.column_positions, {start = col_start, stop = #header_line})
      end

      processed_table.columns = #processed_table.column_positions

      -- Extract column names
      for _, col_pos in ipairs(processed_table.column_positions) do
        local col_text = header_line:sub(col_pos.start, col_pos.stop):match("^%s*(.-)%s*$") or ""
        table.insert(processed_table.column_names, col_text)
      end
    end

    -- Calculate first data cell position
    if processed_table.data_start and #processed_table.column_positions > 0 then
      processed_table.first_data_cell.line = processed_table.data_start
      processed_table.first_data_cell.col = processed_table.column_positions[1].start
    end

    table_registry.tables[i] = processed_table
  end

  return table_registry
end

-- Get table registry (build if needed)
function M.get_table_registry(bufnr)
  if table_registry.buffer_id ~= bufnr or #table_registry.tables == 0 then
    M.build_table_registry(bufnr)
  end
  return table_registry
end

-- Find all table boundaries in the buffer
function M.find_all_tables(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local tables = {}
  local current_table = nil

  local i = 1
  while i <= #lines do
    local line = lines[i]

    -- Look for query headers first (most reliable boundary)
    if line:match("^%-%-+ Query %d+ %-%-+") then
      -- End the current table if one exists
      if current_table and not current_table.end_line then
        current_table.end_line = i - 1
        table.insert(tables, current_table)
      end
      -- Reset for next table
      current_table = nil
      i = i + 1

    -- Look for table start patterns
    elseif line:match("^%+%-+%+") then
      -- ASCII table border - this could be start or end
      if not current_table then
        -- Start a new table and find the first data row
        current_table = {
          start_line = i,
          end_line = nil,
          header_line = nil,
          style = "ascii"
        }

        -- Look ahead to find the first data row (not border, not header with column names)
        for j = i + 1, math.min(i + 6, #lines) do
          if lines[j] and lines[j]:match("^|") and not lines[j]:match("^|%-") then
            -- This is a data row (contains | but not |---)
            -- Check if it's after at least one separator
            local found_separator = false
            for k = i + 1, j - 1 do
              if lines[k] and lines[k]:match("^%+%-+%+") then
                found_separator = true
                break
              end
            end

            if found_separator then
              current_table.header_line = j
              break
            end
          end
        end

        -- Fallback if no clear data row found
        if not current_table.header_line then
          current_table.header_line = i + 3
        end
      elseif current_table and not current_table.end_line and i > current_table.start_line + 2 then
        -- End the current table
        current_table.end_line = i
        table.insert(tables, current_table)
        current_table = nil
      end
      i = i + 1

    -- Look for box table start
    elseif line:match("^┌") then
      current_table = {
        start_line = i,
        end_line = nil,
        header_line = i + 1,
        style = "box"
      }
      i = i + 1

    -- Look for box table end
    elseif line:match("^└") and current_table then
      current_table.end_line = i
      table.insert(tables, current_table)
      current_table = nil
      i = i + 1

    -- Look for pipe table
    elseif line:match("^%s*|") and not current_table then
      if i < #lines and lines[i+1]:match("^%s*|%-") then
        current_table = {
          start_line = i,
          end_line = nil,
          header_line = i,
          style = "pipe"
        }
      end
      i = i + 1

    -- Check for query completion messages (end current table)
    elseif line:match("^Query completed:") then
      if current_table and not current_table.end_line then
        current_table.end_line = i - 1
        table.insert(tables, current_table)
        current_table = nil
      end
      i = i + 1

    else
      i = i + 1
    end
  end

  -- Handle unclosed table at end of buffer
  if current_table and not current_table.end_line then
    current_table.end_line = #lines
    table.insert(tables, current_table)
  end

  return tables
end

-- Get the table index containing the given line
function M.get_table_at_line(tables, line_num)
  for i, tbl in ipairs(tables) do
    if line_num >= tbl.start_line and line_num <= tbl.end_line then
      return i, tbl
    end
  end
  return nil, nil
end

-- Navigate to the next table
function M.goto_next_table(state)
  local bufnr = state:get_output_buf()
  if not bufnr or not vim.api.nvim_buf_is_valid(bufnr) then
    vim.notify("No output buffer", vim.log.levels.WARN)
    return
  end

  local win = state:get_output_win()
  if not win or not vim.api.nvim_win_is_valid(win) then
    vim.notify("No output window", vim.log.levels.WARN)
    return
  end

  -- Get table registry
  local registry = M.get_table_registry(bufnr)
  if #registry.tables == 0 then
    vim.notify("No tables found", vim.log.levels.INFO)
    return
  end

  -- Get current cursor position
  local cursor = vim.api.nvim_win_get_cursor(win)
  local current_line = cursor[1]

  -- Find current table in registry
  local current_idx = nil
  for i, tbl in ipairs(registry.tables) do
    if current_line >= tbl.start_line and current_line <= tbl.end_line then
      current_idx = i
      break
    end
  end

  -- Determine next table
  local next_idx
  if not current_idx then
    -- Not in a table, go to first table
    next_idx = 1
  elseif current_idx < #registry.tables then
    -- Go to next table
    next_idx = current_idx + 1
  else
    -- Wrap to first table
    next_idx = 1
    vim.notify(string.format("Wrapped to table 1/%d", #registry.tables), vim.log.levels.INFO)
  end

  -- Use registry data to position cursor precisely
  local target_table = registry.tables[next_idx]
  local target_line = target_table.first_data_cell.line
  local target_col = target_table.first_data_cell.col

  if target_line and target_col then
    vim.api.nvim_win_set_cursor(win, {target_line, target_col - 1})  -- Vim uses 0-based column indexing
  else
    -- Fallback to start of first data line
    vim.api.nvim_win_set_cursor(win, {target_table.data_start or target_table.start_line + 2, 0})
  end

  -- Center the table in the viewport for better visibility
  vim.api.nvim_win_call(win, function()
    -- Calculate middle of table for centering
    local table_middle = math.floor((target_table.start_line + target_table.end_line) / 2)
    -- Use zz to center the view on the table
    vim.cmd('normal! zz')

    -- Alternatively, scroll so table header is visible near top
    local scroll_target = math.max(1, target_table.start_line - 3)  -- Show table with some context above
    vim.fn.winrestview({topline = scroll_target})
  end)

  -- Clear any existing single-table navigation state to prevent jumping back
  local table_nav = require('sql-cli.table_nav')
  if table_nav.clear_navigation then
    table_nav.clear_navigation()
  end

  -- Show notification
  if current_idx ~= next_idx then
    vim.notify(string.format("Table %d/%d", next_idx, #registry.tables), vim.log.levels.INFO)
  end
end

-- Navigate to the previous table
function M.goto_prev_table(state)
  local bufnr = state:get_output_buf()
  if not bufnr or not vim.api.nvim_buf_is_valid(bufnr) then
    vim.notify("No output buffer", vim.log.levels.WARN)
    return
  end

  local win = state:get_output_win()
  if not win or not vim.api.nvim_win_is_valid(win) then
    vim.notify("No output window", vim.log.levels.WARN)
    return
  end

  -- Find all tables
  local tables = M.find_all_tables(bufnr)
  if #tables == 0 then
    vim.notify("No tables found", vim.log.levels.INFO)
    return
  end

  -- Get current cursor position
  local cursor = vim.api.nvim_win_get_cursor(win)
  local current_line = cursor[1]

  -- Find current table
  local current_idx, _ = M.get_table_at_line(tables, current_line)

  -- Determine previous table
  local prev_idx
  if not current_idx then
    -- Not in a table, go to last table
    prev_idx = #tables
  elseif current_idx > 1 then
    -- Go to previous table
    prev_idx = current_idx - 1
  else
    -- Wrap to last table
    prev_idx = #tables
    vim.notify(string.format("Wrapped to table %d/%d", #tables, #tables), vim.log.levels.INFO)
  end

  -- Move cursor to first data row of previous table (same logic as next_table)
  local prev_table = tables[prev_idx]
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find the first actual data row in this table
  local data_line_num = nil
  for line_num = prev_table.start_line + 1, prev_table.end_line - 1 do
    if lines[line_num] and lines[line_num]:match("^|") and not lines[line_num]:match("^|%-") then
      local line_content = lines[line_num]
      if not line_content:match("^|%s*[A-Za-z_]+%s*|") or line_content:match("^|%s*%d") then
        data_line_num = line_num
        break
      end
    end
  end

  if not data_line_num then
    data_line_num = prev_table.header_line
  end

  if not data_line_num or data_line_num > #lines then
    data_line_num = prev_table.start_line + 2
  end

  -- Use registry data to position cursor precisely (same as goto_next_table)
  local registry = M.get_table_registry(bufnr)
  local target_table = registry.tables[prev_idx]
  local target_line = target_table.first_data_cell.line
  local target_col = target_table.first_data_cell.col

  if target_line and target_col then
    vim.api.nvim_win_set_cursor(win, {target_line, target_col - 1})  -- Vim uses 0-based column indexing
  else
    -- Fallback to start of first data line
    vim.api.nvim_win_set_cursor(win, {target_table.data_start or target_table.start_line + 2, 0})
  end

  -- Center the table in the viewport for better visibility
  vim.api.nvim_win_call(win, function()
    -- Scroll so table header is visible near top with some context
    local scroll_target = math.max(1, target_table.start_line - 3)
    vim.fn.winrestview({topline = scroll_target})
  end)

  -- Clear any existing single-table navigation state to prevent jumping back
  local table_nav = require('sql-cli.table_nav')
  if table_nav.clear_navigation then
    table_nav.clear_navigation()
  end

  -- Show notification
  if current_idx ~= prev_idx then
    vim.notify(string.format("Table %d/%d", prev_idx, #tables), vim.log.levels.INFO)
  end
end

-- Jump to specific table by index
function M.goto_table(state, table_num)
  local bufnr = state:get_output_buf()
  if not bufnr or not vim.api.nvim_buf_is_valid(bufnr) then
    vim.notify("No output buffer", vim.log.levels.WARN)
    return
  end

  local win = state:get_output_win()
  if not win or not vim.api.nvim_win_is_valid(win) then
    vim.notify("No output window", vim.log.levels.WARN)
    return
  end

  -- Find all tables
  local tables = M.find_all_tables(bufnr)
  if #tables == 0 then
    vim.notify("No tables found", vim.log.levels.INFO)
    return
  end

  -- Validate table number
  if table_num < 1 or table_num > #tables then
    vim.notify(string.format("Invalid table number. Available: 1-%d", #tables), vim.log.levels.WARN)
    return
  end

  -- Move to the requested table
  local target_table = tables[table_num]
  local target_line = target_table.header_line or target_table.start_line
  vim.api.nvim_win_set_cursor(win, {target_line, 0})

  vim.notify(string.format("Table %d/%d", table_num, #tables), vim.log.levels.INFO)
end

-- Comprehensive debug function with modal dialog
function M.debug_tables_modal(state)
  local bufnr = state:get_output_buf()
  if not bufnr or not vim.api.nvim_buf_is_valid(bufnr) then
    vim.notify("No output buffer", vim.log.levels.WARN)
    return
  end

  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
  local registry = M.get_table_registry(bufnr)

  -- Build detailed information
  local info_lines = {}
  local debug_data = {}  -- For clipboard export

  table.insert(info_lines, string.format("=== TABLE LOOKUP REGISTRY DEBUG ==="))
  table.insert(info_lines, string.format("Buffer: %d (%d lines)", bufnr, #lines))
  table.insert(info_lines, string.format("Tables found: %d", #registry.tables))
  table.insert(info_lines, string.format("Last updated: %d ms ago", vim.loop.now() - registry.last_updated))
  table.insert(info_lines, "")

  debug_data.buffer_id = bufnr
  debug_data.line_count = #lines
  debug_data.table_count = #registry.tables
  debug_data.tables = {}

  if #registry.tables == 0 then
    table.insert(info_lines, "❌ NO TABLES DETECTED")
    table.insert(info_lines, "")
    table.insert(info_lines, "Sample lines from buffer:")
    for i = 1, math.min(20, #lines) do
      local line = lines[i]
      local marker = ""
      if line:match("^%-%-+ Query %d+ %-%-+") then
        marker = " ← QUERY HEADER"
      elseif line:match("^%+%-+%+") then
        marker = " ← TABLE BORDER"
      elseif line:match("^|") then
        marker = " ← TABLE ROW"
      elseif line:match("^Query completed:") then
        marker = " ← COMPLETION MSG"
      end
      table.insert(info_lines, string.format("%3d: %s%s", i, line, marker))
    end
  else
    -- Show current position
    local win = state:get_output_win()
    local current_table_idx = nil
    if win and vim.api.nvim_win_is_valid(win) then
      local cursor = vim.api.nvim_win_get_cursor(win)
      local current_line = cursor[1]

      -- Find current table in registry
      for i, tbl in ipairs(registry.tables) do
        if current_line >= tbl.start_line and current_line <= tbl.end_line then
          current_table_idx = i
          break
        end
      end

      table.insert(info_lines, string.format("Current cursor: line %d", current_line))
      if current_table_idx then
        table.insert(info_lines, string.format("✅ Currently in table %d/%d", current_table_idx, #registry.tables))
      else
        table.insert(info_lines, "❌ Cursor not in any detected table")
      end
      table.insert(info_lines, "")
    end

    -- Show each table from registry
    for i, tbl in ipairs(registry.tables) do
      table.insert(info_lines, string.format("--- TABLE %d ---", i))
      table.insert(info_lines, string.format("  Boundaries: %d to %d (%s)", tbl.start_line, tbl.end_line, tbl.style))
      table.insert(info_lines, string.format("  Header: line %s", tbl.header_line or "nil"))
      table.insert(info_lines, string.format("  Data: lines %s to %s", tbl.data_start or "nil", tbl.data_end or "nil"))
      table.insert(info_lines, string.format("  Dimensions: %d rows × %d columns", tbl.rows, tbl.columns))
      table.insert(info_lines, string.format("  First cell: line %s, col %s",
        tbl.first_data_cell.line or "nil", tbl.first_data_cell.col or "nil"))

      if #tbl.column_names > 0 then
        table.insert(info_lines, string.format("  Columns: %s", table.concat(tbl.column_names, ", ")))
      end

      -- Add to clipboard data
      debug_data.tables[i] = {
        boundaries = {tbl.start_line, tbl.end_line},
        header_line = tbl.header_line,
        data_range = {tbl.data_start, tbl.data_end},
        dimensions = {tbl.rows, tbl.columns},
        first_cell = {tbl.first_data_cell.line, tbl.first_data_cell.col},
        column_names = tbl.column_names,
        column_positions = tbl.column_positions
      }

      table.insert(info_lines, "")
    end
  end

  table.insert(info_lines, "")
  table.insert(info_lines, "Press <Esc> to close, 'c' to copy to clipboard...")

  -- Create floating window
  local width = 100
  local height = math.min(40, #info_lines + 2)
  local row = math.floor((vim.o.lines - height) / 2)
  local col = math.floor((vim.o.columns - width) / 2)

  -- Create buffer for the modal
  local modal_buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(modal_buf, 0, -1, false, info_lines)
  vim.api.nvim_buf_set_option(modal_buf, 'modifiable', false)
  vim.api.nvim_buf_set_option(modal_buf, 'filetype', 'text')

  -- Create the window
  local modal_win = vim.api.nvim_open_win(modal_buf, true, {
    relative = 'editor',
    width = width,
    height = height,
    row = row,
    col = col,
    style = 'minimal',
    border = 'rounded',
    title = ' Table Lookup Registry Debug ',
    title_pos = 'center'
  })

  -- Close on Escape
  vim.keymap.set('n', '<Esc>', function()
    vim.api.nvim_win_close(modal_win, true)
  end, { buffer = modal_buf })

  -- Copy to clipboard on 'c'
  vim.keymap.set('n', 'c', function()
    -- Convert debug data to JSON for clipboard
    local json_str = vim.json.encode(debug_data)
    vim.fn.setreg('+', json_str)  -- Copy to system clipboard
    vim.fn.setreg('"', json_str)  -- Copy to default register
    vim.notify("Debug data copied to clipboard (JSON format)", vim.log.levels.INFO)
    vim.api.nvim_win_close(modal_win, true)
  end, { buffer = modal_buf })

  -- Copy pretty format on 'p'
  vim.keymap.set('n', 'p', function()
    -- Copy the readable format
    vim.fn.setreg('+', table.concat(info_lines, '\n'))
    vim.fn.setreg('"', table.concat(info_lines, '\n'))
    vim.notify("Debug info copied to clipboard (readable format)", vim.log.levels.INFO)
    vim.api.nvim_win_close(modal_win, true)
  end, { buffer = modal_buf })

  -- Close on Enter
  vim.keymap.set('n', '<CR>', function()
    vim.api.nvim_win_close(modal_win, true)
  end, { buffer = modal_buf })
end

-- Simple debug function (for backward compatibility)
function M.debug_tables(state)
  M.debug_tables_modal(state)
end

-- Show table index in status
function M.show_table_info(state)
  local bufnr = state:get_output_buf()
  if not bufnr or not vim.api.nvim_buf_is_valid(bufnr) then
    return
  end

  local win = state:get_output_win()
  if not win or not vim.api.nvim_win_is_valid(win) then
    return
  end

  -- Find all tables
  local tables = M.find_all_tables(bufnr)
  if #tables == 0 then
    vim.notify("No tables in output", vim.log.levels.INFO)
    return
  end

  -- Get current position
  local cursor = vim.api.nvim_win_get_cursor(win)
  local current_line = cursor[1]

  -- Find current table
  local current_idx, current_table = M.get_table_at_line(tables, current_line)

  if current_idx then
    local rows = current_table.end_line - current_table.start_line - 2 -- Exclude borders
    vim.notify(string.format("Table %d/%d (%d rows)", current_idx, #tables, rows), vim.log.levels.INFO)
  else
    vim.notify(string.format("Not in a table (%d tables available)", #tables), vim.log.levels.INFO)
  end
end

return M