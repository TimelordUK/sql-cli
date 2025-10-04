-- SQL CLI Renderer
-- Renders DataModel to buffer using various themes
-- Only renders visible rows from viewport (virtual scrolling)

local M = {}

-- Get logger
local logger = nil
local function get_logger()
  if not logger then
    local ok, sql_cli = pcall(require, 'sql-cli')
    if ok and sql_cli.logger then
      logger = sql_cli.logger
    end
  end
  return logger
end

--- ASCII theme definition
local ASCII_THEME = {
  top_left = '+',
  top_right = '+',
  bottom_left = '+',
  bottom_right = '+',
  horizontal = '-',
  vertical = '|',
  cross = '+',
  t_down = '+',
  t_up = '+',
  t_left = '+',
  t_right = '+',
}

--- Renderer class
local Renderer = {}
Renderer.__index = Renderer

--- Create a new Renderer
--- @param viewport table Viewport instance
--- @param theme string Theme name ("ascii", "unicode", "markdown", "minimal")
--- @return table Renderer instance
function Renderer:new(viewport, theme)
  local instance = {
    viewport = viewport,
    theme = theme or "ascii",
    themes = {
      ascii = ASCII_THEME,
      -- Can add more themes later: unicode, markdown, minimal
    },
    namespace = vim.api.nvim_create_namespace('sql_cli_renderer'),
  }

  return setmetatable(instance, self)
end

--- Pad or truncate string to exact width
--- @param str string Input string
--- @param width number Target width
--- @param align string Alignment ("left", "right", "center")
--- @return string Padded/truncated string
local function pad_string(str, width, align)
  str = str or ""
  local len = #str

  if len > width then
    -- Truncate
    return str:sub(1, width - 3) .. "..."
  elseif len < width then
    -- Pad
    local padding = width - len
    if align == "right" then
      return string.rep(" ", padding) .. str
    elseif align == "center" then
      local left_pad = math.floor(padding / 2)
      local right_pad = padding - left_pad
      return string.rep(" ", left_pad) .. str .. string.rep(" ", right_pad)
    else
      -- left (default)
      return str .. string.rep(" ", padding)
    end
  else
    return str
  end
end

--- Build separator line
--- @param theme table Theme definition
--- @param widths table Array of column widths
--- @param left string Left character
--- @param middle string Middle character
--- @param right string Right character
--- @return string Separator line
local function build_separator(theme, widths, left, middle, right)
  local parts = {}
  table.insert(parts, left)
  for i, width in ipairs(widths) do
    table.insert(parts, string.rep(theme.horizontal, width + 2))  -- +2 for padding
    if i < #widths then
      table.insert(parts, middle)
    end
  end
  table.insert(parts, right)
  return table.concat(parts)
end

--- Build data row
--- @param theme table Theme definition
--- @param values table Array of cell values
--- @param widths table Array of column widths
--- @param alignments table Array of alignments
--- @return string Formatted row
local function build_row(theme, values, widths, alignments)
  local parts = {}
  table.insert(parts, theme.vertical)
  for i, value in ipairs(values) do
    local padded = pad_string(value, widths[i], alignments[i] or "left")
    table.insert(parts, " " .. padded .. " ")
    table.insert(parts, theme.vertical)
  end
  return table.concat(parts)
end

--- Render viewport to buffer
--- @param bufnr number Buffer number
function Renderer:render_to_buffer(bufnr)
  local log = get_logger()

  if log then
    log.info('renderer', 'Rendering to buffer ' .. bufnr)
  end

  -- Get theme
  local theme = self.themes[self.theme] or self.themes.ascii

  -- Get visible data and metadata
  local visible_data = self.viewport:get_visible_data()
  local column_meta = self.viewport:get_visible_column_meta()

  -- Calculate column widths from metadata
  local widths = {}
  local alignments = {}
  for _, col in ipairs(column_meta) do
    table.insert(widths, col.max_width)
    table.insert(alignments, col.alignment)
  end

  -- Build lines
  local lines = {}

  -- Top border
  table.insert(lines, build_separator(
    theme, widths,
    theme.top_left, theme.t_down, theme.top_right
  ))

  -- Header row
  local header_values = {}
  for _, col in ipairs(column_meta) do
    table.insert(header_values, col.name)
  end
  table.insert(lines, build_row(theme, header_values, widths, alignments))

  -- Header separator
  table.insert(lines, build_separator(
    theme, widths,
    theme.t_right, theme.cross, theme.t_left
  ))

  -- Data rows
  for _, row in ipairs(visible_data.rows) do
    table.insert(lines, build_row(theme, row, widths, alignments))
  end

  -- Bottom border
  table.insert(lines, build_separator(
    theme, widths,
    theme.bottom_left, theme.t_up, theme.bottom_right
  ))

  -- Status line
  table.insert(lines, "")
  table.insert(lines, string.format(
    "%d rows × %d columns",
    self.viewport.data_model.total_rows,
    #column_meta
  ))

  -- Write to buffer
  vim.api.nvim_buf_set_option(bufnr, 'modifiable', true)
  vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, lines)
  vim.api.nvim_buf_set_option(bufnr, 'modifiable', false)

  if log then
    log.info('renderer', string.format('Rendered %d lines', #lines))
  end
end

--- Update cell highlight (maps data coords → buffer position)
--- Moves cursor so Neovim handles scrolling automatically
--- @param bufnr number Buffer number
function Renderer:update_highlight(bufnr)
  -- Clear ALL highlights from all possible namespaces
  -- This ensures we don't have stale highlights from previous renders
  vim.api.nvim_buf_clear_namespace(bufnr, -1, 0, -1)

  -- Also clear our specific namespace just to be sure
  vim.api.nvim_buf_clear_namespace(bufnr, self.namespace, 0, -1)

  local cursor_info = self.viewport:get_cursor_info()

  -- Calculate buffer line for current data row
  -- Line 0: top border
  -- Line 1: header
  -- Line 2: header separator
  -- Line 3+: data rows (row 1 = line 3, row 2 = line 4, etc.)
  local buffer_line = 3 + (cursor_info.row - 1)  -- 0-indexed for API, but we need 1-indexed for cursor

  -- Get column metadata to calculate cell position
  local column_meta = self.viewport:get_visible_column_meta()
  if not column_meta[cursor_info.col] then
    return  -- Column hidden
  end

  -- Calculate cell start position (sum of previous column widths + borders)
  local col_start = 1  -- Start after first '|'
  for i = 1, cursor_info.col - 1 do
    if column_meta[i] then
      col_start = col_start + column_meta[i].max_width + 3  -- width + ' | '
    end
  end

  local col_end = col_start + column_meta[cursor_info.col].max_width + 2  -- width + '  '

  -- Move Neovim's cursor to the cell (this makes Neovim scroll automatically)
  local win = vim.fn.bufwinid(bufnr)
  if win ~= -1 then
    -- Save current window and switch to target window
    local current_win = vim.api.nvim_get_current_win()
    vim.api.nvim_set_current_win(win)

    -- Set cursor position
    vim.api.nvim_win_set_cursor(win, {buffer_line + 1, col_start})  -- +1 because cursor is 1-indexed

    -- Reset the "goal column" by moving to the column explicitly
    -- This prevents Neovim from jumping back to a previous column position
    vim.cmd("normal! " .. col_start .. "|")

    -- Restore original window if needed
    if current_win ~= win and vim.api.nvim_win_is_valid(current_win) then
      vim.api.nvim_set_current_win(current_win)
    end
  end

  -- Highlight just the cell (not the whole row)
  vim.api.nvim_buf_add_highlight(
    bufnr,
    self.namespace,
    'Visual',
    buffer_line,
    col_start,
    col_end
  )
end

--- Get line number for a data row
--- @param data_row number Logical data row (1-indexed in dataset)
--- @return number|nil Buffer line number (0-indexed) or nil if not visible
function Renderer:get_buffer_line_for_row(data_row)
  if data_row < self.viewport.top_row then
    return nil
  end

  local visible_row_offset = data_row - self.viewport.top_row
  local max_visible = math.min(self.viewport.visible_rows, self.viewport.data_model.total_rows - self.viewport.top_row + 1)

  if visible_row_offset >= max_visible then
    return nil
  end

  return 3 + visible_row_offset  -- 0-indexed
end

-- Export
M.Renderer = Renderer

return M
