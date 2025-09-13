-- SQL CLI UI/Window Management Module
-- Functions for creating and managing UI windows and output

local utils = require('sql-cli.utils')
local table_nav = require('sql-cli.table_nav')

local M = {}

-- Create output window for SQL CLI results
function M.create_output_window(config, state)
  -- Save current window
  local current_win = vim.api.nvim_get_current_win()

  -- Create split
  local split_cmd = config.split.direction == "vertical" and "vsplit" or "split"
  vim.cmd(split_cmd)

  -- Resize window
  local size
  if config.split.direction == "vertical" then
    size = math.floor(vim.o.columns * config.split.size)
    vim.api.nvim_win_set_width(0, size)
  else
    size = math.floor(vim.o.lines * config.split.size)
    vim.api.nvim_win_set_height(0, size)
  end

  -- Create or reuse buffer for output
  local output_buf = state:get_output_buf()
  if not output_buf or not vim.api.nvim_buf_is_valid(output_buf) then
    output_buf = vim.api.nvim_create_buf(false, true)
    state:set_output_buf(output_buf)
    -- Try to set name, but handle if it already exists
    pcall(function()
      vim.api.nvim_buf_set_name(output_buf, "[SQL CLI Output]")
    end)
  end

  vim.api.nvim_win_set_buf(0, output_buf)
  local output_win = vim.api.nvim_get_current_win()
  state:set_output_win(output_win)

  -- Set buffer options
  vim.bo[output_buf].buftype = "nofile"
  vim.bo[output_buf].bufhidden = "hide"
  vim.bo[output_buf].swapfile = false
  vim.bo[output_buf].filetype = ""  -- No filetype to prevent unwanted syntax
  vim.bo[output_buf].syntax = ""    -- Clear any syntax

  -- Apply our custom syntax highlighting
  M.setup_output_highlighting(state)

  -- Set window options
  vim.wo[output_win].wrap = config.output.wrap
  vim.wo[output_win].number = config.output.number
  vim.wo[output_win].relativenumber = false
  vim.wo[output_win].signcolumn = "no"
  vim.wo[output_win].foldcolumn = "0"

  -- Return to original window
  vim.api.nvim_set_current_win(current_win)
end

-- Toggle output window visibility
function M.toggle_output_window(config, state)
  if M.is_output_window_valid(state) then
    local output_win = state:get_output_win()
    vim.api.nvim_win_close(output_win, false)
    state:set_output_win(nil)
  else
    M.create_output_window(config, state)
  end
end

-- Check if output window is valid
function M.is_output_window_valid(state)
  local output_win = state:get_output_win()
  local output_buf = state:get_output_buf()
  return output_win
    and vim.api.nvim_win_is_valid(output_win)
    and output_buf
    and vim.api.nvim_buf_is_valid(output_buf)
end

-- Toggle split orientation
function M.toggle_split_orientation(config, state)
  -- Toggle the configuration
  if config.split.direction == "vertical" then
    config.split.direction = "horizontal"
    vim.notify("Split orientation: horizontal", vim.log.levels.INFO)
  else
    config.split.direction = "vertical"
    vim.notify("Split orientation: vertical", vim.log.levels.INFO)
  end

  -- If output window is open, recreate it with new orientation
  if M.is_output_window_valid(state) then
    -- Close current window
    local output_win = state:get_output_win()
    vim.api.nvim_win_close(output_win, false)
    state:set_output_win(nil)

    -- Recreate with new orientation
    M.create_output_window(config, state)
  end
end

-- Preview query at cursor position in floating window
function M.preview_query_at_cursor(config)
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end

  -- Extract the query lines, skipping comments
  local query_lines = {}
  for i = start_line, end_line do
    local line = lines[i]
    -- Skip SQL comment lines
    if not line:match("^%s*%-%-") then
      table.insert(query_lines, line)
    end
  end

  -- Create floating window for preview
  local width = math.min(80, vim.o.columns - 10)
  local height = math.min(#query_lines + 4, vim.o.lines - 10)

  -- Calculate centered position
  local row = math.floor((vim.o.lines - height) / 2)
  local col = math.floor((vim.o.columns - width) / 2)

  -- Create buffer for preview
  local preview_buf = vim.api.nvim_create_buf(false, true)

  -- Add header
  local header = "─── Query Preview (press q or <Esc> to close) ───"
  local padding = math.floor((width - #header) / 2)
  vim.api.nvim_buf_set_lines(preview_buf, 0, -1, false, {
    string.rep(" ", padding) .. header,
    "",
  })

  -- Add query lines
  vim.api.nvim_buf_set_lines(preview_buf, -1, -1, false, query_lines)

  -- Add footer with line info
  local footer = string.format("Lines %d-%d (%d lines)", start_line, end_line, end_line - start_line + 1)
  vim.api.nvim_buf_set_lines(preview_buf, -1, -1, false, {
    "",
    string.rep(" ", math.floor((width - #footer) / 2)) .. footer,
  })

  -- Create floating window
  local win_opts = {
    relative = "editor",
    row = row,
    col = col,
    width = width,
    height = height,
    style = "minimal",
    border = "rounded",
    title = " SQL Query Preview ",
    title_pos = "center",
  }

  local preview_win = vim.api.nvim_open_win(preview_buf, true, win_opts)

  -- Set buffer options
  vim.api.nvim_buf_set_option(preview_buf, "bufhidden", "delete")
  vim.api.nvim_buf_set_option(preview_buf, "filetype", "sql")
  vim.api.nvim_buf_set_option(preview_buf, "modifiable", false)

  -- Set window options
  vim.api.nvim_win_set_option(preview_win, "cursorline", true)
  vim.api.nvim_win_set_option(preview_win, "wrap", false)

  -- Add keymaps to close preview
  local close_preview = function()
    if vim.api.nvim_win_is_valid(preview_win) then
      vim.api.nvim_win_close(preview_win, true)
    end
  end

  vim.keymap.set("n", "q", close_preview, { buffer = preview_buf })
  vim.keymap.set("n", "<Esc>", close_preview, { buffer = preview_buf })
  vim.keymap.set("n", "<CR>", function()
    close_preview()
    vim.notify("Use :SqlCliExecuteAtCursor or keybind to execute", vim.log.levels.INFO)
  end, { buffer = preview_buf, desc = "Close preview" })
end

-- Get statusline information
function M.statusline(state)
  local data_file = state:get_data_file()
  if data_file then
    local filename = vim.fn.fnamemodify(data_file, ":t")
    return string.format("SQL[📄%s]", filename)
  else
    return "SQL[∅]"
  end
end

-- Setup output buffer syntax highlighting
function M.setup_output_highlighting(state, enable_nav)
  local output_buf = state:get_output_buf()
  if not output_buf or not vim.api.nvim_buf_is_valid(output_buf) then
    return
  end

  -- Apply highlighting in the output buffer
  vim.api.nvim_buf_call(output_buf, function()
    -- Clear any existing syntax first
    vim.cmd([[syntax clear]])

    -- Header and metadata
    vim.cmd([[syntax match SqlCliHeader /^--.*$/]])
    vim.cmd([[syntax match SqlCliSeparator /^[-─═]\+$/]])
    vim.cmd([[syntax match SqlCliExitCode /^-- Exit code:.*$/]])

    -- Table structure - must come before other patterns
    vim.cmd([[syntax match SqlCliTableBorder /[│├┤┌┐└┘─┬┴┼═╪╫╬]/]])
    vim.cmd([[syntax match SqlCliTablePipe /|/]])

    -- Success/Error messages - high priority
    vim.cmd([[syntax match SqlCliSuccess /^#.*Query completed:.*$/]])
    vim.cmd([[syntax match SqlCliError /^ERROR:.*$/]])
    vim.cmd([[syntax match SqlCliError /Error:.*$/]])

    -- Currency symbols and formatted money (high priority, before numbers)
    vim.cmd([[syntax match SqlCliCurrency /[$£€¥₹¤]\S*/]])
    vim.cmd([[syntax match SqlCliCurrency /\S*\s\+\(USD\|EUR\|GBP\|JPY\|CHF\|CNY\|INR\|AUD\|CAD\|SEK\|NOK\|DKK\)\>/]])

    -- Number patterns - order matters for precedence
    -- Use 'contained' and 'contains=NONE' to prevent overlapping matches

    -- Compact numbers (1.5k, 2M, etc) - highest priority
    vim.cmd([[syntax match SqlCliCompactNumber /\v\d+\.?\d*[kMBT]/ contains=NONE]])

    -- Numbers with thousand separators (1,234.56)
    vim.cmd([[syntax match SqlCliFormattedNumber /\v\d{1,3}([,.']\d{3})+\.?\d*/ contains=NONE]])

    -- Decimal numbers (must have decimal point)
    vim.cmd([[syntax match SqlCliDecimalNumber /\v\d+\.\d+/ contains=NONE]])

    -- Plain integers
    vim.cmd([[syntax match SqlCliNumber /\v\d+/ contains=NONE]])

    -- Special values
    vim.cmd([[syntax match SqlCliNull /\<NULL\>/]])
    vim.cmd([[syntax match SqlCliBoolean /\<\(true\|false\|TRUE\|FALSE\)\>/]])

    -- Quoted strings (lowest priority)
    vim.cmd([[syntax match SqlCliString /"[^"]*"/]])
    vim.cmd([[syntax match SqlCliString /'[^']*'/]])

    -- Define highlight colors with better contrast
    vim.cmd([[highlight SqlCliHeader guifg=#6272a4 ctermfg=60]])
    vim.cmd([[highlight SqlCliSeparator guifg=#44475a ctermfg=238]])
    vim.cmd([[highlight SqlCliTableBorder guifg=#6272a4 ctermfg=61]])
    vim.cmd([[highlight SqlCliTablePipe guifg=#6272a4 ctermfg=61]])

    -- Numbers and currency with distinct colors
    vim.cmd([[highlight SqlCliNumber guifg=#bd93f9 ctermfg=141]])
    vim.cmd([[highlight SqlCliDecimalNumber guifg=#bd93f9 ctermfg=141]])
    vim.cmd([[highlight SqlCliFormattedNumber guifg=#bd93f9 ctermfg=141]])
    vim.cmd([[highlight SqlCliCompactNumber guifg=#ff79c6 ctermfg=212]])
    vim.cmd([[highlight SqlCliCurrency guifg=#50fa7b ctermfg=84]])

    -- Special values
    vim.cmd([[highlight SqlCliNull guifg=#ff79c6 ctermfg=212 gui=italic cterm=italic]])
    vim.cmd([[highlight SqlCliBoolean guifg=#8be9fd ctermfg=117]])

    -- Status messages
    vim.cmd([[highlight SqlCliError guifg=#ff5555 ctermfg=203 gui=bold cterm=bold]])
    vim.cmd([[highlight SqlCliSuccess guifg=#50fa7b ctermfg=84]])
    vim.cmd([[highlight SqlCliExitCode guifg=#6272a4 ctermfg=60]])

    -- Strings with subdued color
    vim.cmd([[highlight SqlCliString guifg=#f1fa8c ctermfg=228]])
  end)

  -- Note: Table navigation is now enabled in executor.lua after query completes
  -- This ensures the buffer is fully populated with results
end

return M