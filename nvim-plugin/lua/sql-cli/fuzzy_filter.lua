-- Fuzzy filter module for SQL-CLI result buffers
-- Provides Telescope-style filtering of table data

local M = {}

-- Get logger (lazy load to avoid circular dependencies)
local logger = nil
local function get_logger()
  if not logger then
    logger = require('sql-cli').logger
  end
  return logger
end

-- Configuration
M.config = {
  match_mode = 'exact'  -- 'exact' or 'fuzzy'
}

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
-- If table_bounds is provided, only parse that specific table
local function parse_table_from_buffer(bufnr, table_bounds)
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find table boundaries
  local table_start = nil
  local table_end = nil
  local header_line = nil
  local separator_line = nil

  -- If specific table bounds provided, use them directly
  if table_bounds then
    table_start = table_bounds.start_line
    table_end = table_bounds.end_line
    header_line = table_bounds.header_line
    separator_line = table_bounds.separator_line
  else
    -- Search for first table in buffer (old behavior)
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

-- Exact substring match (stricter, like fzf --exact)
local function exact_match(pattern, text)
  pattern = pattern:lower()
  text = text:lower()

  -- Check if pattern exists as exact substring
  local match_pos = text:find(pattern, 1, true)  -- plain text search
  if not match_pos then
    return nil
  end

  -- Score based on position (earlier = better)
  local score = 100 - match_pos

  -- Bonus for exact match
  if text == pattern then
    score = score + 100
  end

  -- Bonus for match at start
  if match_pos == 1 then
    score = score + 50
  end

  return score
end

-- Fuzzy match scoring (original, more permissive)
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

  -- Choose match function based on config
  local match_fn = M.config.match_mode == 'exact' and exact_match or fuzzy_match

  -- Filter rows
  for _, row in ipairs(data.rows) do
    local best_score = 0
    local match_found = false

    if column_filter and column_filter > 0 and column_filter <= #row.cells then
      -- Filter specific column
      local score = match_fn(pattern, row.cells[column_filter])
      if score then
        best_score = score
        match_found = true
      end
    else
      -- Filter all columns
      for _, cell in ipairs(row.cells) do
        local score = match_fn(pattern, cell)
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

      -- Update window title with match count and mode
      if filter_win and vim.api.nvim_win_is_valid(filter_win) then
        local mode_indicator = M.config.match_mode == 'exact' and '[EXACT]' or '[FUZZY]'
        local title
        if pattern and pattern ~= "" then
          if visible_count == 0 then
            title = string.format(' %s No matches (0/%d) ', mode_indicator, #cache.data.rows)
          else
            title = string.format(' %s Showing %d/%d rows ', mode_indicator, visible_count, #cache.data.rows)
          end
        else
          title = string.format(' %s Filter %d rows (ESC: close, C-l: clear, C-t: toggle) ',
            mode_indicator, #cache.data.rows)
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

  local log = get_logger()
  if log then
    log.debug('fuzzy_filter', 'Opening fuzzy finder for buffer ' .. bufnr)
  end

  -- Check if table navigation is active to determine which table to filter
  local table_bounds = nil
  local table_nav = require('sql-cli.table_nav')
  if table_nav.is_active and table_nav.is_active() then
    -- Get current table info from active navigation
    local table_info = table_nav.get_table_info and table_nav.get_table_info()
    if table_info then
      table_bounds = {
        start_line = table_info.header_row and (table_info.header_row - 1) or nil,
        end_line = table_info.data_end and (table_info.data_end + 1) or nil,
        header_line = table_info.header_row,
        separator_line = table_info.separator_row
      }
      if log then
        log.debug('fuzzy_filter', string.format('Using active table navigation bounds: lines %d-%d',
          table_bounds.start_line or 0, table_bounds.end_line or 0))
      end
    end
  end

  -- Parse table data (either specific table or first table in buffer)
  local data = parse_table_from_buffer(bufnr, table_bounds)
  if not data then
    vim.notify("No table found in buffer", vim.log.levels.WARN)
    if log then
      log.warn('fuzzy_filter', 'No table found in buffer ' .. bufnr)
    end
    return
  end

  if log then
    log.info('fuzzy_filter', string.format('Parsed table with %d rows, %d columns',
      #data.rows, #data.headers))
  end

  -- Store in cache
  cache.buffer = bufnr
  cache.data = data
  cache.original_lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Create filter window
  local width = 60
  local height = 1

  local filter_buf = vim.api.nvim_create_buf(false, true)

  -- Initial title showing total rows and match mode
  local mode_indicator = M.config.match_mode == 'exact' and '[EXACT]' or '[FUZZY]'
  local initial_title = string.format(' %s Filter %d rows (CR: lock, ESC: clear, C-j/k: scroll, C-t: toggle) ',
    mode_indicator, #data.rows)

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

  -- Lock filter (keep results, close input, enable navigation)
  vim.keymap.set('n', '<CR>', function()
    vim.api.nvim_win_close(filter_win, true)
    -- Keep filtered content but make it navigable
    local visible_count = 0
    for _, row in ipairs(data.rows) do
      if row.visible then
        visible_count = visible_count + 1
      end
    end
    vim.notify(string.format("Filter locked - %d/%d rows visible (/ to reopen, ESC to clear)",
      visible_count, #data.rows), vim.log.levels.INFO)

    -- Focus back on the results buffer
    if vim.api.nvim_win_is_valid(vim.fn.win_getid()) then
      vim.api.nvim_set_current_win(vim.fn.bufwinid(bufnr))
    end

    -- Set up buffer-local ESC handler to clear filter
    vim.keymap.set('n', '<Esc>', function()
      M.reset_filter(bufnr)
    end, { buffer = bufnr, noremap = true, silent = true })
  end, opts)

  vim.keymap.set('i', '<CR>', function()
    vim.api.nvim_win_close(filter_win, true)
    -- Keep filtered content but make it navigable
    local visible_count = 0
    for _, row in ipairs(data.rows) do
      if row.visible then
        visible_count = visible_count + 1
      end
    end
    vim.notify(string.format("Filter locked - %d/%d rows visible (/ to reopen, ESC to clear)",
      visible_count, #data.rows), vim.log.levels.INFO)

    -- Focus back on the results buffer and return to normal mode
    if vim.api.nvim_win_is_valid(vim.fn.win_getid()) then
      vim.api.nvim_set_current_win(vim.fn.bufwinid(bufnr))
      vim.cmd('stopinsert')  -- Exit insert mode
    end

    -- Set up buffer-local ESC handler to clear filter
    vim.keymap.set('n', '<Esc>', function()
      M.reset_filter(bufnr)
    end, { buffer = bufnr, noremap = true, silent = true })
  end, opts)

  -- Clear filter
  vim.keymap.set('n', '<C-l>', function()
    vim.api.nvim_buf_set_lines(filter_buf, 0, -1, false, {""})
    vim.cmd('startinsert!')
  end, opts)

  -- Toggle match mode (exact vs fuzzy)
  vim.keymap.set('n', '<C-t>', function()
    -- Toggle mode
    M.config.match_mode = M.config.match_mode == 'exact' and 'fuzzy' or 'exact'

    local log = get_logger()
    if log then
      log.info('fuzzy_filter', 'Match mode toggled to: ' .. M.config.match_mode)
    end

    -- Re-apply current filter with new mode
    local pattern = vim.api.nvim_buf_get_lines(filter_buf, 0, 1, false)[1] or ""
    debounced_filter(bufnr, pattern, nil, filter_win)

    -- Notify user
    local mode_name = M.config.match_mode == 'exact' and 'EXACT' or 'FUZZY'
    vim.notify("Match mode: " .. mode_name, vim.log.levels.INFO)
  end, opts)

  -- Toggle match mode in insert mode too
  vim.keymap.set('i', '<C-t>', function()
    -- Toggle mode
    M.config.match_mode = M.config.match_mode == 'exact' and 'fuzzy' or 'exact'

    -- Re-apply current filter with new mode
    local pattern = vim.api.nvim_buf_get_lines(filter_buf, 0, 1, false)[1] or ""
    debounced_filter(bufnr, pattern, nil, filter_win)

    -- Notify user
    local mode_name = M.config.match_mode == 'exact' and 'EXACT' or 'FUZZY'
    vim.notify("Match mode: " .. mode_name, vim.log.levels.INFO)
  end, opts)

  -- Column-specific filtering
  for i = 1, 9 do
    vim.keymap.set('n', '<C-' .. i .. '>', function()
      local pattern = vim.api.nvim_buf_get_lines(filter_buf, 0, 1, false)[1] or ""
      debounced_filter(bufnr, pattern, i, filter_win)
      vim.notify("Filtering column " .. i, vim.log.levels.INFO)
    end, opts)
  end

  -- Navigate results while filter is open (Ctrl+j/k)
  vim.keymap.set('i', '<C-j>', function()
    -- Move cursor down in results buffer
    local win = vim.fn.bufwinid(bufnr)
    if win ~= -1 then
      vim.api.nvim_win_call(win, function()
        vim.cmd('normal! j')
      end)
    end
  end, opts)

  vim.keymap.set('i', '<C-k>', function()
    -- Move cursor up in results buffer
    local win = vim.fn.bufwinid(bufnr)
    if win ~= -1 then
      vim.api.nvim_win_call(win, function()
        vim.cmd('normal! k')
      end)
    end
  end, opts)

  -- Page down/up in results while filter is open
  vim.keymap.set('i', '<C-d>', function()
    local win = vim.fn.bufwinid(bufnr)
    if win ~= -1 then
      vim.api.nvim_win_call(win, function()
        vim.cmd('normal! \\<C-d>')
      end)
    end
  end, opts)

  vim.keymap.set('i', '<C-u>', function()
    local win = vim.fn.bufwinid(bufnr)
    if win ~= -1 then
      vim.api.nvim_win_call(win, function()
        vim.cmd('normal! \\<C-u>')
      end)
    end
  end, opts)

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