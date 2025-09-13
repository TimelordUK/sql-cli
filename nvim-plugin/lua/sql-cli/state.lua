-- SQL CLI State Management Module
-- Centralized state management for the plugin

local M = {}

-- Create new state object
function M.new()
  return {
    data_file = nil,
    output_buf = nil,
    output_win = nil,
    last_query = nil,
    last_results = nil,  -- Store last query results for saving
    query_markers = {},  -- Track query positions in output
  }
end

-- Global state instance
M.state = M.new()

-- State accessors and mutators
function M.get_data_file()
  return M.state.data_file
end

function M.set_data_file(file)
  M.state.data_file = file
end

function M.clear_data_file()
  M.state.data_file = nil
end

function M.get_output_buf()
  return M.state.output_buf
end

function M.set_output_buf(buf)
  M.state.output_buf = buf
end

function M.get_output_win()
  return M.state.output_win
end

function M.set_output_win(win)
  M.state.output_win = win
end

function M.get_last_query()
  return M.state.last_query
end

function M.set_last_query(query)
  M.state.last_query = query
end

function M.get_last_results()
  return M.state.last_results
end

function M.set_last_results(results)
  M.state.last_results = results
end

function M.get_query_markers()
  return M.state.query_markers
end

function M.set_query_markers(markers)
  M.state.query_markers = markers
end

function M.add_query_marker(marker)
  table.insert(M.state.query_markers, marker)
end

function M.clear_query_markers()
  M.state.query_markers = {}
end

-- Cleanup function
function M.cleanup()
  if M.state.output_buf and vim.api.nvim_buf_is_valid(M.state.output_buf) then
    vim.api.nvim_buf_delete(M.state.output_buf, { force = true })
  end

  if M.state.output_win and vim.api.nvim_win_is_valid(M.state.output_win) then
    vim.api.nvim_win_close(M.state.output_win, true)
  end

  M.state = M.new()
end

return M