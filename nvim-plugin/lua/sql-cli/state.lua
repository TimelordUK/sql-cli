-- SQL CLI State Management Module
-- Centralized state management for the plugin

local M = {}

-- State metatable for method access
local State = {}
State.__index = State

-- Create new state object
function M.new()
  local state = {
    data_file = nil,
    output_buf = nil,
    output_win = nil,
    last_query = nil,
    last_results = nil,  -- Store last query results for saving
    query_markers = {},  -- Track query positions in output
    schema_columns = {},  -- Store column schema information
    last_query_columns = {},  -- Store columns from last executed query
    query_bufnr = nil,  -- Track the buffer where queries are written
  }
  return setmetatable(state, State)
end

-- State methods
function State:get_data_file()
  return self.data_file
end

function State:set_data_file(file)
  self.data_file = file
end

function State:clear_data_file()
  self.data_file = nil
end

function State:get_output_buf()
  return self.output_buf
end

function State:set_output_buf(buf)
  self.output_buf = buf
end

function State:get_output_win()
  return self.output_win
end

function State:set_output_win(win)
  self.output_win = win
end

function State:get_last_query()
  return self.last_query
end

function State:set_last_query(query)
  self.last_query = query
end

function State:get_last_results()
  return self.last_results
end

function State:set_last_results(results)
  self.last_results = results
end

function State:get_query_markers()
  return self.query_markers
end

function State:set_query_markers(markers)
  self.query_markers = markers
end

function State:add_query_marker(marker)
  table.insert(self.query_markers, marker)
end

function State:clear_query_markers()
  self.query_markers = {}
end

function State:get_schema_columns()
  return self.schema_columns
end

function State:set_schema_columns(columns)
  self.schema_columns = columns or {}
end

function State:get_last_query_columns()
  return self.last_query_columns
end

function State:set_last_query_columns(columns)
  self.last_query_columns = columns or {}
end

function State:get_query_bufnr()
  return self.query_bufnr
end

function State:set_query_bufnr(bufnr)
  self.query_bufnr = bufnr
end

-- Cleanup function
function State:cleanup()
  if self.output_buf and vim.api.nvim_buf_is_valid(self.output_buf) then
    vim.api.nvim_buf_delete(self.output_buf, { force = true })
  end

  if self.output_win and vim.api.nvim_win_is_valid(self.output_win) then
    vim.api.nvim_win_close(self.output_win, true)
  end

  -- Reset state
  self.data_file = nil
  self.output_buf = nil
  self.output_win = nil
  self.last_query = nil
  self.last_results = nil
  self.query_markers = {}
  self.schema_columns = {}
  self.last_query_columns = {}
  self.query_bufnr = nil
end

return M