-- SQL CLI Query History System
-- Stores and recalls previously executed queries with deduplication

local M = {}

-- Storage for query history
local history = {
  queries = {},  -- Array of {query = "", checksum = "", timestamp = "", preview = ""}
  max_size = 100,  -- Maximum number of queries to keep
}

-- Configuration (will be set by init function)
local config = nil

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

  -- Trim leading and trailing empty lines
  local lines = vim.split(query, '\n')

  -- Remove leading empty lines
  while #lines > 0 and lines[1]:match("^%s*$") do
    table.remove(lines, 1)
  end

  -- Remove trailing empty lines
  while #lines > 0 and lines[#lines]:match("^%s*$") do
    table.remove(lines)
  end

  -- Reconstruct the trimmed query
  query = table.concat(lines, '\n')

  if query == "" then
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

  -- Create preview (first meaningful SQL line)
  local preview = ""
  for line in query:gmatch("[^\n]+") do
    local trimmed = line:gsub("^%s+", ""):gsub("%s+$", "")
    -- Skip comments and empty lines
    if not trimmed:match("^%-%-") and trimmed ~= "" then
      -- Get the first significant SQL keyword
      if trimmed:upper():match("^WITH%s+WEB") or
         trimmed:upper():match("^SELECT") or
         trimmed:upper():match("^INSERT") or
         trimmed:upper():match("^UPDATE") or
         trimmed:upper():match("^DELETE") or
         trimmed:upper():match("^WITH%s") then
        preview = trimmed:sub(1, 100)
        if #trimmed > 100 then
          preview = preview .. "..."
        end
        break
      elseif preview == "" then
        -- If no SQL keyword found yet, use first non-comment line
        preview = trimmed:sub(1, 100)
        if #trimmed > 100 then
          preview = preview .. "..."
        end
      end
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

  -- Save if auto-save is enabled
  if config and config.query_history and config.query_history.auto_save then
    M.save_history()
  end
end

-- Show history picker (simple version)
function M.show_history_picker_simple(callback)
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

-- Show history picker with enhanced UI and preview
function M.show_history_picker(callback, execute_callback)
  local ui = require('sql-cli.query_history_ui')
  ui.show_history_with_preview(history.queries, callback, execute_callback)
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
  if config and config.query_history and config.query_history.auto_save then
    M.save_history()
  end
  vim.notify("Query history cleared", vim.log.levels.INFO)
end

-- Get history size
function M.get_history_size()
  return #history.queries
end

-- Export history to a specific file
function M.export_history(filepath)
  if not filepath then
    -- Prompt for file path
    vim.ui.input({ prompt = 'Export to file: ', default = vim.fn.expand('~/sql_history.json') }, function(input)
      if input then
        M.export_history(input)
      end
    end)
    return
  end

  -- Expand the path
  filepath = vim.fn.expand(filepath)

  local ok, json = pcall(vim.json.encode, history)
  if ok then
    vim.fn.writefile(vim.split(json, '\n'), filepath)
    vim.notify("Exported " .. #history.queries .. " queries to " .. filepath, vim.log.levels.INFO)
  else
    vim.notify("Failed to export history: " .. tostring(json), vim.log.levels.ERROR)
  end
end

-- Import history from a specific file
function M.import_history(filepath, merge)
  if not filepath then
    -- Prompt for file path
    vim.ui.input({ prompt = 'Import from file: ', default = vim.fn.expand('~/sql_history.json') }, function(input)
      if input then
        M.import_history(input, false)
      end
    end)
    return
  end

  -- Expand the path
  filepath = vim.fn.expand(filepath)

  if vim.fn.filereadable(filepath) == 0 then
    vim.notify("File not found: " .. filepath, vim.log.levels.ERROR)
    return
  end

  local content = vim.fn.readfile(filepath)
  if #content > 0 then
    local ok, data = pcall(vim.json.decode, table.concat(content, '\n'))
    if ok and data and data.queries then
      if merge then
        -- Merge with existing history
        local existing_checksums = {}
        for _, item in ipairs(history.queries) do
          existing_checksums[item.checksum] = true
        end

        local added = 0
        for _, item in ipairs(data.queries) do
          if not existing_checksums[item.checksum] then
            table.insert(history.queries, item)
            added = added + 1
          end
        end

        -- Sort by timestamp (newest first)
        table.sort(history.queries, function(a, b)
          return a.timestamp > b.timestamp
        end)

        -- Limit size
        while #history.queries > history.max_size do
          table.remove(history.queries)
        end

        vim.notify("Imported " .. added .. " new queries (merged)", vim.log.levels.INFO)
      else
        -- Replace existing history
        history = data
        vim.notify("Imported " .. #data.queries .. " queries (replaced)", vim.log.levels.INFO)
      end

      -- Save the updated history
      if config and config.query_history and config.query_history.auto_save then
        M.save_history()
      end
    else
      vim.notify("Invalid history file format", vim.log.levels.ERROR)
    end
  end
end

-- Initialize with configuration
function M.init(user_config)
  config = user_config

  -- Set max size from config
  if config and config.query_history and config.query_history.max_items then
    history.max_size = config.query_history.max_items
  end

  -- Load history if persistence is enabled
  if config and config.query_history and config.query_history.persist then
    M.load_history()
  end
end

-- Get current configuration
function M.get_config()
  return config
end

return M