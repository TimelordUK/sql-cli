-- SQL CLI Viewport
-- Manages visible range and cursor position for large datasets
-- Supports virtual scrolling for efficient rendering

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

--- Viewport class
--- Tracks cursor position, visible range, and column state
local Viewport = {}
Viewport.__index = Viewport

--- Create a new Viewport
--- @param data_model table DataModel instance
--- @param config table Optional configuration {visible_rows, theme}
--- @return table Viewport instance
function Viewport:new(data_model, config)
  local log = get_logger()

  config = config or {}

  local instance = {
    data_model = data_model,

    -- Cursor position (1-indexed, logical coordinates)
    current_row = 1,
    current_col = 1,

    -- Visible range (for virtual scrolling with large datasets)
    top_row = 1,
    visible_rows = config.visible_rows or 50,  -- Render only this many rows

    -- Column state
    column_order = {},      -- Logical column indices [1, 2, 3, ...]
    hidden_columns = {},    -- Set of hidden column indices
    column_widths = {},     -- Custom widths or nil for auto

    -- Theme
    theme = config.theme or "ascii",  -- ascii, unicode, markdown, minimal
  }

  -- Initialize column order (all columns visible by default)
  for i = 1, data_model.total_cols do
    table.insert(instance.column_order, i)
  end

  if log then
    log.info('viewport', string.format(
      'Created viewport: %d visible rows, theme=%s',
      instance.visible_rows,
      instance.theme
    ))
  end

  return setmetatable(instance, self)
end

--- Move cursor with bounds checking
--- @param dr number Row delta (negative = up, positive = down)
--- @param dc number Column delta (negative = left, positive = right)
--- @return boolean True if cursor moved
function Viewport:move_cursor(dr, dc)
  local new_row = self.current_row + dr
  local new_col = self.current_col + dc

  -- Bounds checking
  new_row = math.max(1, math.min(new_row, self.data_model.total_rows))
  new_col = math.max(1, math.min(new_col, self.data_model.total_cols))

  if new_row == self.current_row and new_col == self.current_col then
    return false  -- No movement
  end

  self.current_row = new_row
  self.current_col = new_col

  -- Update visible range if cursor moved out of view
  self:ensure_cursor_visible()

  return true
end

--- Ensure cursor is within visible range, adjust top_row if needed
--- Scrolls one line at a time like Vim (smooth scrolling)
function Viewport:ensure_cursor_visible()
  local bottom_row = self.top_row + self.visible_rows - 1

  -- Cursor scrolled above visible range - scroll up one line
  if self.current_row < self.top_row then
    self.top_row = self.current_row
  end

  -- Cursor scrolled below visible range - scroll down one line
  if self.current_row > bottom_row then
    self.top_row = self.top_row + 1
  end

  -- Ensure top_row is valid
  local max_top_row = math.max(1, self.data_model.total_rows - self.visible_rows + 1)
  self.top_row = math.max(1, math.min(self.top_row, max_top_row))
end

--- Center viewport on current cursor position
function Viewport:center_on_cursor()
  local center_offset = math.floor(self.visible_rows / 2)
  self.top_row = math.max(1, self.current_row - center_offset)

  -- Ensure we don't scroll past the end
  local max_top_row = math.max(1, self.data_model.total_rows - self.visible_rows + 1)
  self.top_row = math.min(self.top_row, max_top_row)
end

--- Get all data (Neovim handles scrolling, we render everything)
--- @return table {rows, total_rows, columns}
function Viewport:get_visible_data()
  -- Get visible columns (respecting hidden columns and order)
  local visible_columns = {}
  for _, col_idx in ipairs(self.column_order) do
    if not self.hidden_columns[col_idx] then
      table.insert(visible_columns, col_idx)
    end
  end

  -- Extract ALL rows (no limit - if the CLI handled it, we can render it)
  local rows = {}
  for row = 1, self.data_model.total_rows do
    local row_data = {}
    for _, col_idx in ipairs(visible_columns) do
      table.insert(row_data, self.data_model:get_cell(row, col_idx))
    end
    table.insert(rows, row_data)
  end

  return {
    rows = rows,
    total_rows = self.data_model.total_rows,
    columns = visible_columns,
  }
end

--- Get column metadata for visible columns
--- @return table Array of column metadata
function Viewport:get_visible_column_meta()
  local meta = {}
  for _, col_idx in ipairs(self.column_order) do
    if not self.hidden_columns[col_idx] then
      table.insert(meta, self.data_model:get_column_meta(col_idx))
    end
  end
  return meta
end

--- Hide a column
--- @param col number Column index (1-based, logical)
function Viewport:hide_column(col)
  self.hidden_columns[col] = true
end

--- Show a hidden column
--- @param col number Column index (1-based, logical)
function Viewport:show_column(col)
  self.hidden_columns[col] = nil
end

--- Toggle column visibility
--- @param col number Column index (1-based, logical)
function Viewport:toggle_column(col)
  if self.hidden_columns[col] then
    self:show_column(col)
  else
    self:hide_column(col)
  end
end

--- Get cursor position info (in DATA coordinates)
--- @return table {row, col}
function Viewport:get_cursor_info()
  return {
    row = self.current_row,
    col = self.current_col,
  }
end

--- Jump to specific row
--- @param row number Target row (1-based)
function Viewport:goto_row(row)
  row = math.max(1, math.min(row, self.data_model.total_rows))
  self.current_row = row
  self:ensure_cursor_visible()
end

--- Jump to specific column
--- @param col number Target column (1-based)
function Viewport:goto_col(col)
  col = math.max(1, math.min(col, self.data_model.total_cols))
  self.current_col = col
end

--- Get status line info
--- @return string Status message
function Viewport:get_status()
  return string.format(
    "Row %d/%d, Col %d/%d (showing rows %d-%d)",
    self.current_row,
    self.data_model.total_rows,
    self.current_col,
    self.data_model.total_cols,
    self.top_row,
    math.min(self.top_row + self.visible_rows - 1, self.data_model.total_rows)
  )
end

-- Export
M.Viewport = Viewport

return M
