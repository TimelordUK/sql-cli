-- Fuzzy filter module for SQL-CLI result buffers
-- Provides Telescope-style filtering of table data

local M = {}

-- Cache for parsed table data
local cache = {
  buffer = nil,
  data = nil,
  headers = nil,
  original_lines = nil,
  table_bounds = nil
}

-- Timer for debouncing
local debounce_timer = nil

-- Parse table data from buffer
local function parse_table_from_buffer(bufnr)
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find table boundaries
  local table_start = nil
  local table_end = nil
  local header_line = nil
  local separator_line = nil

  for i, line in ipairs(lines) do
    -- Look for ASCII table style
    if line:match("^%+%-") then
      if not table_start then
        table_start = i
        -- Next line should be header
        if i < #lines and lines[i + 1]:match("^|") then
          header_line = i + 1
          -- Check for separator
          if i + 2 <= #lines and lines[i + 2]:match("^%+%-") then
            separator_line = i + 2
          end
        end
      elseif separator_line and i > separator_line then
        table_end = i
        break
      end
    end
  end

  if not table_start or not header_line then
    return nil
  end

  -- Parse headers
  local header = lines[header_line]
  local headers = {}
  local col_positions = {}

  -- Find column boundaries based on | characters
  local pos = 1
  for col_end in header:gmatch("()%|") do
    if pos > 1 then  -- Skip first |
      local col_text = header:sub(pos, col_end - 1)
      col_text = col_text:match("^%s*(.-)%s*$") or ""
      table.insert(headers, col_text)
      table.insert(col_positions, {start = pos, stop = col_end - 1})
    end
    pos = col_end + 1
  end

  -- Parse data rows
  local data_rows = {}
  local data_start = separator_line and separator_line + 1 or header_line + 1
  local data_end = table_end and table_end - 1 or #lines

  for i = data_start, data_end do
    local line = lines[i]
    if line:match("^|") then
      local row = {}
      local raw_row = line

      -- Extract cell values based on column positions
      for j, col_pos in ipairs(col_positions) do
        local cell = line:sub(col_pos.start, col_pos.stop)
        cell = cell:match("^%s*(.-)%s*$") or ""
        row[j] = cell
      end

      table.insert(data_rows, {
        index = i,
        cells = row,
        raw = raw_row,
        visible = true
      })
    end
  end

  return {
    headers = headers,
    rows = data_rows,
    col_positions = col_positions,
    bounds = {
      table_start = table_start,
      table_end = table_end,
      header_line = header_line,
      separator_line = separator_line,
      data_start = data_start,
      data_end = data_end
    }
  }
end

-- Fuzzy match scoring
local function fuzzy_match(pattern, text)
  pattern = pattern:lower()
  text = text:lower()

  local pattern_idx = 1
  local score = 0
  local consecutive = 0

  for i = 1, #text do
    if pattern_idx <= #pattern then
      if text:sub(i, i) == pattern:sub(pattern_idx, pattern_idx) then
        score = score + 1 + consecutive
        consecutive = consecutive + 1
        pattern_idx = pattern_idx + 1
      else
        consecutive = 0
      end
    end
  end

  -- Return nil if not all pattern characters were found
  if pattern_idx <= #pattern then
    return nil
  end

  -- Bonus for exact match
  if text == pattern then
    score = score * 2
  end

  -- Bonus for pattern at start
  if text:sub(1, #pattern) == pattern then
    score = score * 1.5
  end

  return score
end

-- Apply filter to rows
local function apply_filter(data, pattern, column_filter)
  if not pattern or pattern == "" then
    -- Show all rows
    for _, row in ipairs(data.rows) do
      row.visible = true
      row.score = 0
    end
    return
  end

  -- Filter rows
  for _, row in ipairs(data.rows) do
    local best_score = 0
    local match_found = false

    if column_filter and column_filter > 0 and column_filter <= #row.cells then
      -- Filter specific column
      local score = fuzzy_match(pattern, row.cells[column_filter])
      if score then
        best_score = score
        match_found = true
      end
    else
      -- Filter all columns
      for _, cell in ipairs(row.cells) do
        local score = fuzzy_match(pattern, cell)
        if score and score > best_score then
          best_score = score
          match_found = true
        end
      end
    end

    row.visible = match_found
    row.score = best_score
  end

  -- Sort visible rows by score
  local visible_rows = {}
  for _, row in ipairs(data.rows) do
    if row.visible then
      table.insert(visible_rows, row)
    end
  end

  table.sort(visible_rows, function(a, b)
    if a.score == b.score then
      return a.index < b.index  -- Maintain original order for same score
    end
    return a.score > b.score
  end)
end

-- Update buffer with filtered results
local function update_buffer_display(bufnr, data, maintain_format)
  if maintain_format then
    -- Hide non-matching rows while maintaining table structure
    local lines = {}
    local original_lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

    -- Keep everything before the table
    for i = 1, data.bounds.table_start - 1 do
      table.insert(lines, original_lines[i] or "")
    end

    -- Add table top border
    table.insert(lines, original_lines[data.bounds.table_start])

    -- Add header
    table.insert(lines, original_lines[data.bounds.header_line])

    -- Add separator
    table.insert(lines, original_lines[data.bounds.separator_line])

    -- Add visible data rows
    for _, row in ipairs(data.rows) do
      if row.visible then
        table.insert(lines, row.raw)
      end
    end

    -- Add table bottom border
    if data.bounds.table_end then
      table.insert(lines, original_lines[data.bounds.table_end])
    end

    -- Keep everything after the table
    for i = (data.bounds.table_end or #original_lines) + 1, #original_lines do
      table.insert(lines, original_lines[i])
    end

    -- Update buffer
    vim.bo[bufnr].modifiable = true
    vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, lines)
    vim.bo[bufnr].modifiable = false
  else
    -- Just show matching rows in a simple format
    local lines = {}

    -- Add header
    local header_line = "| " .. table.concat(data.headers, " | ") .. " |"
    table.insert(lines, header_line)
    table.insert(lines, string.rep("-", #header_line))

    -- Add visible rows
    for _, row in ipairs(data.rows) do
      if row.visible then
        table.insert(lines, "| " .. table.concat(row.cells, " | ") .. " |")
      end
    end

    -- Update buffer
    vim.bo[bufnr].modifiable = true
    vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, lines)
    vim.bo[bufnr].modifiable = false
  end
end

-- Debounced filter function
local function debounced_filter(bufnr, pattern, column_filter, filter_win)
  if debounce_timer then
    vim.fn.timer_stop(debounce_timer)
  end

  debounce_timer = vim.fn.timer_start(150, function()
    if not cache.data or cache.buffer ~= bufnr then
      cache.data = parse_table_from_buffer(bufnr)
      cache.buffer = bufnr
      cache.original_lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)
    end

    if cache.data then
      apply_filter(cache.data, pattern, column_filter)
      update_buffer_display(bufnr, cache.data, true)

      -- Show match count
      local visible_count = 0
      for _, row in ipairs(cache.data.rows) do
        if row.visible then
          visible_count = visible_count + 1
        end
      end

      -- Update window title with match count
      if filter_win and vim.api.nvim_win_is_valid(filter_win) then
        local title
        if pattern and pattern ~= "" then
          if visible_count == 0 then
            title = string.format(' No matches (0/%d) ', #cache.data.rows)
          else
            title = string.format(' Showing %d/%d rows ', visible_count, #cache.data.rows)
          end
        else
          title = string.format(' Filter %d rows (ESC to close, C-l to clear) ', #cache.data.rows)
        end

        vim.api.nvim_win_set_config(filter_win, {
          title = title,
          title_pos = 'center'
        })
      end
    else
      vim.notify("No table found in buffer", vim.log.levels.WARN)
    end

    debounce_timer = nil
  end)
end

-- Open fuzzy finder interface
function M.open_fuzzy_finder(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()

  -- Parse table data
  local data = parse_table_from_buffer(bufnr)
  if not data then
    vim.notify("No table found in buffer", vim.log.levels.WARN)
    return
  end

  -- Store in cache
  cache.buffer = bufnr
  cache.data = data
  cache.original_lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Create filter window
  local width = 60
  local height = 1

  local filter_buf = vim.api.nvim_create_buf(false, true)

  -- Initial title showing total rows
  local initial_title = string.format(' Filter %d rows (ESC to close, C-l to clear) ', #data.rows)

  local filter_win = vim.api.nvim_open_win(filter_buf, true, {
    relative = 'editor',
    width = width,
    height = height,
    col = math.floor((vim.o.columns - width) / 2),
    row = 0,
    style = 'minimal',
    border = 'rounded',
    title = initial_title,
    title_pos = 'center'
  })

  -- Set up autocommands for live filtering
  vim.api.nvim_create_autocmd({"TextChangedI", "TextChanged"}, {
    buffer = filter_buf,
    callback = function()
      local pattern = vim.api.nvim_buf_get_lines(filter_buf, 0, 1, false)[1] or ""
      debounced_filter(bufnr, pattern, nil, filter_win)
    end
  })

  -- Keymaps for filter window
  local opts = { buffer = filter_buf, noremap = true, silent = true }

  -- Close filter
  vim.keymap.set('n', '<Esc>', function()
    vim.api.nvim_win_close(filter_win, true)
    -- Restore original content
    if cache.original_lines then
      vim.bo[bufnr].modifiable = true
      vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, cache.original_lines)
      vim.bo[bufnr].modifiable = false
    end
    cache = {}
  end, opts)

  -- Apply filter and close
  vim.keymap.set('n', '<CR>', function()
    vim.api.nvim_win_close(filter_win, true)
    -- Keep filtered content
    vim.notify("Filter applied", vim.log.levels.INFO)
  end, opts)

  -- Clear filter
  vim.keymap.set('n', '<C-l>', function()
    vim.api.nvim_buf_set_lines(filter_buf, 0, -1, false, {""})
    vim.cmd('startinsert!')
  end, opts)

  -- Column-specific filtering
  for i = 1, 9 do
    vim.keymap.set('n', '<C-' .. i .. '>', function()
      local pattern = vim.api.nvim_buf_get_lines(filter_buf, 0, 1, false)[1] or ""
      debounced_filter(bufnr, pattern, i, filter_win)
      vim.notify("Filtering column " .. i, vim.log.levels.INFO)
    end, opts)
  end

  -- Start in insert mode
  vim.cmd('startinsert!')

  -- Initial prompt
  vim.api.nvim_buf_set_lines(filter_buf, 0, -1, false, {""})
end

-- Reset filter
function M.reset_filter(bufnr)
  bufnr = bufnr or vim.api.nvim_get_current_buf()

  if cache.original_lines and cache.buffer == bufnr then
    vim.bo[bufnr].modifiable = true
    vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, cache.original_lines)
    vim.bo[bufnr].modifiable = false
    cache = {}
    vim.notify("Filter reset", vim.log.levels.INFO)
  end
end

-- Get filtered rows (for export/yank operations)
function M.get_filtered_rows()
  if not cache.data then
    return nil
  end

  local rows = {}
  for _, row in ipairs(cache.data.rows) do
    if row.visible then
      table.insert(rows, row.cells)
    end
  end

  return {
    headers = cache.data.headers,
    rows = rows
  }
end

return M