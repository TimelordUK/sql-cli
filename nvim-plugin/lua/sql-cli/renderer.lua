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
  table.insert(lines, self.viewport:get_status())

  -- Write to buffer
  vim.api.nvim_buf_set_option(bufnr, 'modifiable', true)
  vim.api.nvim_buf_set_lines(bufnr, 0, -1, false, lines)
  vim.api.nvim_buf_set_option(bufnr, 'modifiable', false)

  if log then
    log.info('renderer', string.format('Rendered %d lines', #lines))
  end
end

--- Update cell highlight using extmarks (no re-render needed)
--- @param bufnr number Buffer number
function Renderer:update_highlight(bufnr)
  -- Clear previous highlights
  vim.api.nvim_buf_clear_namespace(bufnr, self.namespace, 0, -1)

  local cursor_info = self.viewport:get_cursor_info()
  if not cursor_info.is_visible then
    return  -- Cursor not in visible range
  end

  -- Calculate buffer line for current row
  -- Line 0: top border
  -- Line 1: header
  -- Line 2: header separator
  -- Line 3+: data rows (starting from top_row)
  local visible_row_offset = cursor_info.row - self.viewport.top_row
  local buffer_line = 3 + visible_row_offset  -- 0-indexed

  -- Highlight the entire row
  vim.api.nvim_buf_add_highlight(
    bufnr,
    self.namespace,
    'Visual',  -- Use Visual highlight group
    buffer_line,
    0,
    -1
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
