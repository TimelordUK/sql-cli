-- SQL CLI Query History System
-- Stores and recalls previously executed queries with deduplication

local M = {}

-- Storage for query history
local history = {
  queries = {},  -- Array of {query = "", checksum = "", timestamp = "", preview = ""}
  max_size = 100,  -- Maximum number of queries to keep
}

-- Calculate checksum for deduplication
local function calculate_checksum(query)
  -- Simple checksum using vim's sha256 if available
  if vim.fn.exists('*sha256') == 1 then
    return vim.fn.sha256(query)
  else
    -- Fallback to simple hash
    local hash = 0
    for i = 1, #query do
      hash = ((hash * 31) + string.byte(query, i)) % 2147483647
    end
    return tostring(hash)
  end
end

-- Get storage file path
local function get_history_file()
  local data_dir = vim.fn.stdpath('data') .. '/sql-cli'
  vim.fn.mkdir(data_dir, 'p')
  return data_dir .. '/query_history.json'
end

-- Load history from disk (for future persistence)
function M.load_history()
  local file = get_history_file()
  if vim.fn.filereadable(file) == 1 then
    local content = vim.fn.readfile(file)
    if #content > 0 then
      local ok, data = pcall(vim.json.decode, table.concat(content, '\n'))
      if ok and data then
        history = vim.tbl_deep_extend('force', history, data)
        -- Limit to max_size
        while #history.queries > history.max_size do
          table.remove(history.queries)
        end
      end
    end
  end
end

-- Save history to disk (for future persistence)
function M.save_history()
  local file = get_history_file()
  local ok, json = pcall(vim.json.encode, history)
  if ok then
    vim.fn.writefile(vim.split(json, '\n'), file)
  end
end

-- Add query to history with deduplication
function M.add_to_history(query)
  if not query or query == "" then
    return
  end

  -- Calculate checksum
  local checksum = calculate_checksum(query)

  -- Check if already exists and move to front
  for i, item in ipairs(history.queries) do
    if item.checksum == checksum then
      -- Move to front
      table.remove(history.queries, i)
      item.timestamp = os.date("%Y-%m-%d %H:%M:%S")
      table.insert(history.queries, 1, item)
      -- M.save_history()  -- Uncomment when ready for persistence
      return
    end
  end

  -- Create preview (first non-comment line or first 80 chars)
  local preview = ""
  for line in query:gmatch("[^\n]+") do
    if not line:match("^%s*%-%-") and line:match("%S") then
      preview = line:gsub("^%s+", ""):sub(1, 80)
      if #line > 80 then
        preview = preview .. "..."
      end
      break
    end
  end

  -- Add new query to front
  table.insert(history.queries, 1, {
    query = query,
    checksum = checksum,
    timestamp = os.date("%Y-%m-%d %H:%M:%S"),
    preview = preview
  })

  -- Limit size
  while #history.queries > history.max_size do
    table.remove(history.queries)
  end

  -- M.save_history()  -- Uncomment when ready for persistence
end

-- Show history picker
function M.show_history_picker(callback)
  if #history.queries == 0 then
    vim.notify("No query history available", vim.log.levels.INFO)
    return
  end

  local items = {}
  for i, item in ipairs(history.queries) do
    table.insert(items, {
      display = string.format("[%d] %s - %s", i, item.timestamp, item.preview),
      query = item.query,
      index = i
    })
  end

  vim.ui.select(items, {
    prompt = "Select query from history:",
    format_item = function(item)
      return item.display
    end,
    kind = "sql_history"
  }, function(item)
    if item and callback then
      callback(item.query)
    end
  end)
end

-- Get recent queries (for quick access)
function M.get_recent_queries(count)
  count = count or 10
  local recent = {}
  for i = 1, math.min(count, #history.queries) do
    table.insert(recent, history.queries[i])
  end
  return recent
end

-- Search history by pattern
function M.search_history(pattern)
  local results = {}
  pattern = pattern:lower()

  for _, item in ipairs(history.queries) do
    if item.query:lower():find(pattern, 1, true) or
       item.preview:lower():find(pattern, 1, true) then
      table.insert(results, item)
    end
  end

  return results
end

-- Clear history
function M.clear_history()
  history.queries = {}
  -- M.save_history()  -- Uncomment when ready for persistence
  vim.notify("Query history cleared", vim.log.levels.INFO)
end

-- Get history size
function M.get_history_size()
  return #history.queries
end

-- Initialize (load history if persistence is enabled)
-- M.load_history()  -- Uncomment when ready for persistence

return M