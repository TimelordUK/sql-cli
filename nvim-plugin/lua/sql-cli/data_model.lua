-- SQL CLI Data Model
-- Structured data representation for query results
-- Replaces fragile text-based table parsing

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

--- DataModel class
--- Stores structured query results with column metadata
local DataModel = {}
DataModel.__index = DataModel

--- Create a new DataModel from JSON structured output
--- @param json_data table Parsed JSON from sql-cli --output json-structured
--- @return table DataModel instance
function DataModel:new(json_data)
  local log = get_logger()

  if not json_data or not json_data.columns or not json_data.rows then
    if log then
      log.error('data_model', 'Invalid JSON data structure')
    end
    return nil
  end

  local instance = {
    columns = json_data.columns or {},
    rows = json_data.rows or {},
    metadata = json_data.metadata or {},

    -- Computed
    total_rows = #(json_data.rows or {}),
    total_cols = #(json_data.columns or {}),
  }

  if log then
    log.info('data_model', string.format(
      'Created DataModel: %d rows, %d columns',
      instance.total_rows,
      instance.total_cols
    ))
  end

  return setmetatable(instance, self)
end

--- Get cell value by logical position (1-indexed)
--- @param row number Row index (1-based)
--- @param col number Column index (1-based)
--- @return string|nil Cell value or nil if out of bounds
function DataModel:get_cell(row, col)
  if row < 1 or row > self.total_rows then
    return nil
  end
  if col < 1 or col > self.total_cols then
    return nil
  end

  local row_data = self.rows[row]
  if not row_data then
    return nil
  end

  return row_data[col] or ""
end

--- Get entire row (1-indexed)
--- @param row number Row index (1-based)
--- @return table|nil Row data as array of strings
function DataModel:get_row(row)
  if row < 1 or row > self.total_rows then
    return nil
  end
  return self.rows[row]
end

--- Get all values in a column (for statistics, export)
--- @param col number Column index (1-based)
--- @return table Array of column values
function DataModel:get_column_values(col)
  if col < 1 or col > self.total_cols then
    return {}
  end

  local values = {}
  for i = 1, self.total_rows do
    table.insert(values, self:get_cell(i, col))
  end
  return values
end

--- Get column metadata
--- @param col number Column index (1-based)
--- @return table|nil Column metadata {name, type, max_width, alignment}
function DataModel:get_column_meta(col)
  if col < 1 or col > self.total_cols then
    return nil
  end
  return self.columns[col]
end

--- Get column name
--- @param col number Column index (1-based)
--- @return string|nil Column name
function DataModel:get_column_name(col)
  local meta = self:get_column_meta(col)
  return meta and meta.name or nil
end

--- Get all column names
--- @return table Array of column names
function DataModel:get_column_names()
  local names = {}
  for _, col in ipairs(self.columns) do
    table.insert(names, col.name)
  end
  return names
end

--- Get query execution time
--- @return number Query time in milliseconds
function DataModel:get_query_time()
  return self.metadata.query_time_ms or 0
end

--- Check if position is valid
--- @param row number Row index (1-based)
--- @param col number Column index (1-based)
--- @return boolean True if position is within bounds
function DataModel:is_valid_position(row, col)
  return row >= 1 and row <= self.total_rows and
         col >= 1 and col <= self.total_cols
end

--- Get data summary for debugging
--- @return string Summary string
function DataModel:summary()
  return string.format(
    "DataModel: %d rows × %d columns, query_time=%.2fms",
    self.total_rows,
    self.total_cols,
    self:get_query_time()
  )
end

--- Parse JSON output from sql-cli
--- @param json_string string JSON output from sql-cli --output json-structured
--- @return table|nil DataModel instance or nil on error
function M.from_json(json_string)
  local log = get_logger()

  if log then
    log.debug('data_model', 'Parsing JSON (' .. #json_string .. ' chars)')
  end

  local success, json_data = pcall(vim.json.decode, json_string)
  if not success then
    if log then
      log.error('data_model', 'Failed to parse JSON: ' .. tostring(json_data))
    end
    return nil
  end

  return DataModel:new(json_data)
end

--- Create DataModel from sql-cli command
--- @param cmd table Command array {sql-cli, args...}
--- @return table|nil DataModel instance or nil on error
--- @return string|nil Error message if failed
function M.from_command(cmd)
  local log = get_logger()

  -- Add --output json-structured flag
  local cmd_with_format = vim.deepcopy(cmd)
  table.insert(cmd_with_format, '-o')
  table.insert(cmd_with_format, 'json-structured')

  if log then
    log.info('data_model', 'Executing: ' .. table.concat(cmd_with_format, ' '))
  end

  -- Execute command
  local output = vim.fn.system(cmd_with_format)
  local exit_code = vim.v.shell_error

  if exit_code ~= 0 then
    local error_msg = 'Command failed (exit code ' .. exit_code .. '): ' .. output
    if log then
      log.error('data_model', error_msg)
    end
    return nil, error_msg
  end

  -- Parse JSON
  local data_model = M.from_json(output)
  if not data_model then
    return nil, 'Failed to parse JSON output'
  end

  return data_model, nil
end

-- Export
M.DataModel = DataModel

return M
