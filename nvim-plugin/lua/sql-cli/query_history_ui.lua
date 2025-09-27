-- SQL CLI Query History UI with Preview
-- Enhanced UI for browsing and previewing query history

local M = {}

-- Create a floating window with preview
function M.show_history_with_preview(history_items, callback)
  if #history_items == 0 then
    vim.notify("No query history available", vim.log.levels.INFO)
    return
  end

  -- Calculate window dimensions
  local width = math.min(120, math.floor(vim.o.columns * 0.8))
  local height = math.min(40, math.floor(vim.o.lines * 0.8))
  local preview_height = math.floor(height * 0.6)
  local list_height = height - preview_height - 3  -- Leave room for borders

  -- Create main buffer for list
  local list_buf = vim.api.nvim_create_buf(false, true)
  local preview_buf = vim.api.nvim_create_buf(false, true)

  -- Set buffer options
  vim.api.nvim_buf_set_option(list_buf, 'modifiable', false)
  vim.api.nvim_buf_set_option(list_buf, 'buftype', 'nofile')
  vim.api.nvim_buf_set_option(preview_buf, 'modifiable', false)
  vim.api.nvim_buf_set_option(preview_buf, 'buftype', 'nofile')
  vim.api.nvim_buf_set_option(preview_buf, 'filetype', 'sql')

  -- Prepare display items with better formatting
  local display_lines = {}
  local max_preview_len = width - 25  -- Leave room for timestamp and index

  for i, item in ipairs(history_items) do
    -- Create a more informative preview
    local first_line = item.query:match("^[^\n]+") or ""
    first_line = first_line:gsub("^%s+", ""):gsub("%s+$", "")

    -- Identify query type
    local query_type = "QUERY"
    if first_line:upper():match("^WITH%s+WEB") then
      query_type = "WEB CTE"
    elseif first_line:upper():match("^WITH%s") then
      query_type = "CTE"
    elseif first_line:upper():match("^SELECT") then
      query_type = "SELECT"
    elseif first_line:upper():match("^INSERT") then
      query_type = "INSERT"
    elseif first_line:upper():match("^UPDATE") then
      query_type = "UPDATE"
    elseif first_line:upper():match("^DELETE") then
      query_type = "DELETE"
    end

    -- Truncate preview if too long
    if #first_line > max_preview_len then
      first_line = first_line:sub(1, max_preview_len - 3) .. "..."
    end

    local display = string.format("[%2d] %s │ %-8s │ %s",
      i,
      item.timestamp:sub(12, 19),  -- Just show time
      query_type,
      first_line
    )
    table.insert(display_lines, display)
  end

  -- Write list to buffer
  vim.api.nvim_buf_set_option(list_buf, 'modifiable', true)
  vim.api.nvim_buf_set_lines(list_buf, 0, -1, false, display_lines)
  vim.api.nvim_buf_set_option(list_buf, 'modifiable', false)

  -- Calculate window positions
  local row = math.floor((vim.o.lines - height) / 2)
  local col = math.floor((vim.o.columns - width) / 2)

  -- Create list window
  local list_win = vim.api.nvim_open_win(list_buf, true, {
    relative = 'editor',
    width = width,
    height = list_height,
    row = row,
    col = col,
    style = 'minimal',
    border = 'rounded',
    title = ' Query History (↑↓ navigate, Enter select, / search, q quit) ',
    title_pos = 'center',
  })

  -- Create preview window below
  local preview_win = vim.api.nvim_open_win(preview_buf, false, {
    relative = 'editor',
    width = width,
    height = preview_height,
    row = row + list_height + 1,
    col = col,
    style = 'minimal',
    border = 'rounded',
    title = ' Preview ',
    title_pos = 'center',
  })

  -- Current selection
  local current_line = 1
  local search_mode = false
  local search_term = ""

  -- Function to update preview
  local function update_preview()
    if current_line > 0 and current_line <= #history_items then
      local item = history_items[current_line]
      local preview_lines = vim.split(item.query, '\n')

      vim.api.nvim_buf_set_option(preview_buf, 'modifiable', true)
      vim.api.nvim_buf_set_lines(preview_buf, 0, -1, false, preview_lines)
      vim.api.nvim_buf_set_option(preview_buf, 'modifiable', false)

      -- Update preview title with more info
      local line_count = #preview_lines
      local char_count = #item.query
      vim.api.nvim_win_set_config(preview_win, {
        title = string.format(' Preview - %s (%d lines, %d chars) ',
          item.timestamp, line_count, char_count),
      })
    end
  end

  -- Function to filter items
  local function filter_items(term)
    if not term or term == "" then
      -- Reset to all items
      vim.api.nvim_buf_set_option(list_buf, 'modifiable', true)
      vim.api.nvim_buf_set_lines(list_buf, 0, -1, false, display_lines)
      vim.api.nvim_buf_set_option(list_buf, 'modifiable', false)
      return
    end

    local filtered = {}
    local filtered_indices = {}
    term = term:lower()

    for i, item in ipairs(history_items) do
      if item.query:lower():find(term, 1, true) or
         display_lines[i]:lower():find(term, 1, true) then
        table.insert(filtered, display_lines[i])
        table.insert(filtered_indices, i)
      end
    end

    vim.api.nvim_buf_set_option(list_buf, 'modifiable', true)
    if #filtered > 0 then
      vim.api.nvim_buf_set_lines(list_buf, 0, -1, false, filtered)
    else
      vim.api.nvim_buf_set_lines(list_buf, 0, -1, false, {"No matches found for: " .. term})
    end
    vim.api.nvim_buf_set_option(list_buf, 'modifiable', false)
  end

  -- Initial preview
  update_preview()

  -- Set up keymaps
  local function setup_keymaps()
    -- Navigation
    vim.keymap.set('n', 'j', function()
      current_line = math.min(current_line + 1, #history_items)
      vim.api.nvim_win_set_cursor(list_win, {current_line, 0})
      update_preview()
    end, { buffer = list_buf })

    vim.keymap.set('n', 'k', function()
      current_line = math.max(current_line - 1, 1)
      vim.api.nvim_win_set_cursor(list_win, {current_line, 0})
      update_preview()
    end, { buffer = list_buf })

    vim.keymap.set('n', '<Down>', function()
      current_line = math.min(current_line + 1, #history_items)
      vim.api.nvim_win_set_cursor(list_win, {current_line, 0})
      update_preview()
    end, { buffer = list_buf })

    vim.keymap.set('n', '<Up>', function()
      current_line = math.max(current_line - 1, 1)
      vim.api.nvim_win_set_cursor(list_win, {current_line, 0})
      update_preview()
    end, { buffer = list_buf })

    -- Page navigation
    vim.keymap.set('n', '<C-f>', function()
      current_line = math.min(current_line + 10, #history_items)
      vim.api.nvim_win_set_cursor(list_win, {current_line, 0})
      update_preview()
    end, { buffer = list_buf })

    vim.keymap.set('n', '<C-b>', function()
      current_line = math.max(current_line - 10, 1)
      vim.api.nvim_win_set_cursor(list_win, {current_line, 0})
      update_preview()
    end, { buffer = list_buf })

    -- Selection
    vim.keymap.set('n', '<CR>', function()
      local selected = history_items[current_line]
      vim.api.nvim_win_close(preview_win, true)
      vim.api.nvim_win_close(list_win, true)
      if selected and callback then
        callback(selected.query)
      end
    end, { buffer = list_buf })

    -- Search
    vim.keymap.set('n', '/', function()
      vim.ui.input({ prompt = 'Search: ' }, function(input)
        if input then
          search_term = input
          filter_items(search_term)
        end
      end)
    end, { buffer = list_buf })

    -- Clear search
    vim.keymap.set('n', '<Esc>', function()
      if search_term ~= "" then
        search_term = ""
        filter_items("")
      else
        vim.api.nvim_win_close(preview_win, true)
        vim.api.nvim_win_close(list_win, true)
      end
    end, { buffer = list_buf })

    -- Quit
    vim.keymap.set('n', 'q', function()
      vim.api.nvim_win_close(preview_win, true)
      vim.api.nvim_win_close(list_win, true)
    end, { buffer = list_buf })

    -- Yank query
    vim.keymap.set('n', 'y', function()
      local selected = history_items[current_line]
      if selected then
        vim.fn.setreg('"', selected.query)
        vim.fn.setreg('+', selected.query)
        vim.notify("Query copied to clipboard", vim.log.levels.INFO)
      end
    end, { buffer = list_buf })

    -- Delete from history
    vim.keymap.set('n', 'd', function()
      if current_line > 0 and current_line <= #history_items then
        table.remove(history_items, current_line)
        table.remove(display_lines, current_line)

        vim.api.nvim_buf_set_option(list_buf, 'modifiable', true)
        vim.api.nvim_buf_set_lines(list_buf, 0, -1, false, display_lines)
        vim.api.nvim_buf_set_option(list_buf, 'modifiable', false)

        if current_line > #history_items and current_line > 1 then
          current_line = current_line - 1
        end
        update_preview()
      end
    end, { buffer = list_buf })
  end

  setup_keymaps()

  -- Auto-close on buffer leave
  vim.api.nvim_create_autocmd("BufLeave", {
    buffer = list_buf,
    once = true,
    callback = function()
      if vim.api.nvim_win_is_valid(preview_win) then
        vim.api.nvim_win_close(preview_win, true)
      end
      if vim.api.nvim_win_is_valid(list_win) then
        vim.api.nvim_win_close(list_win, true)
      end
    end
  })
end

return M