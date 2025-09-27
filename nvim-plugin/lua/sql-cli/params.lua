-- SQL CLI Parameter Resolution System
-- Handles runtime parameters in queries with history and quick selection

local M = {}

-- Storage for parameter history and sets
local param_data = {
  -- History of recent values per parameter
  history = {},  -- { ORDER_ID = {"ORDER_001", "ORDER_002", ...} }

  -- Named parameter sets
  sets = {},  -- { dev = {ORDER_ID = "ORDER_001", SOURCE = "Bloomberg"} }

  -- Last used values per parameter
  last_used = {},  -- { ORDER_ID = "ORDER_001" }

  -- Common sources for quick picker
  common_sources = {
    "Bloomberg",
    "Reuters",
    "TradeWeb",
    "MarketAxess",
    "ICE",
    "CME",
    "Eurex",
    "LSE",
    "NYSE"
  }
}

-- File to persist parameter data
local function get_data_file()
  local data_dir = vim.fn.stdpath('data') .. '/sql-cli'
  vim.fn.mkdir(data_dir, 'p')
  return data_dir .. '/params.json'
end

-- Load parameter data from disk
function M.load_data()
  local file = get_data_file()
  if vim.fn.filereadable(file) == 1 then
    local content = vim.fn.readfile(file)
    if #content > 0 then
      local ok, data = pcall(vim.json.decode, table.concat(content, '\n'))
      if ok and data then
        param_data = vim.tbl_deep_extend('force', param_data, data)
      end
    end
  end
end

-- Save parameter data to disk
function M.save_data()
  local file = get_data_file()
  local ok, json = pcall(vim.json.encode, param_data)
  if ok then
    vim.fn.writefile(vim.split(json, '\n'), file)
  end
end

-- Add value to history for a parameter
local function add_to_history(param_name, value)
  if not param_data.history[param_name] then
    param_data.history[param_name] = {}
  end

  local history = param_data.history[param_name]

  -- Remove if already exists (to move to front)
  for i, v in ipairs(history) do
    if v == value then
      table.remove(history, i)
      break
    end
  end

  -- Add to front
  table.insert(history, 1, value)

  -- Keep only last 10
  while #history > 10 do
    table.remove(history)
  end

  -- Update last used
  param_data.last_used[param_name] = value

  M.save_data()
end

-- Get parameter sets defined in buffer
local function get_buffer_param_sets(bufnr)
  local lines = vim.api.nvim_buf_get_lines(bufnr or 0, 0, -1, false)
  local sets = {}

  for _, line in ipairs(lines) do
    -- Look for: -- @PARAMS: name = { KEY: "value", ... }
    local name, params_str = line:match("^%s*%-%-%s*@PARAMS:%s*(%w+)%s*=%s*{(.-)}")
    if name and params_str then
      local params = {}
      -- Parse KEY: "value" pairs
      for key, value in params_str:gmatch('(%w+):%s*"([^"]*)"') do
        params[key] = value
      end
      if next(params) then
        sets[name] = params
      end
    end
  end

  return sets
end

-- Extract parameters from text
function M.extract_parameters(text)
  local params = {}

  -- Match {{PARAM_NAME}} or ${PARAM_NAME}
  for param in text:gmatch("{{([%w_]+)}}") do
    params[param] = true
  end
  for param in text:gmatch("${([%w_]+)}") do
    params[param] = true
  end

  return vim.tbl_keys(params)
end

-- Show picker for parameter value
function M.pick_parameter_value(param_name, callback)
  local items = {}

  -- Special handling for DATE parameters
  if param_name == "DATE" or param_name:match("DATE") or param_name:match("_DATE$") then
    -- Add date options
    local today = os.date("%Y-%m-%d")
    local yesterday = os.date("%Y-%m-%d", os.time() - 86400)
    local week_ago = os.date("%Y-%m-%d", os.time() - 7 * 86400)
    local month_ago = os.date("%Y-%m-%d", os.time() - 30 * 86400)

    table.insert(items, {
      display = string.format("[Today] %s", today),
      value = today
    })
    table.insert(items, {
      display = string.format("[Yesterday] %s", yesterday),
      value = yesterday
    })
    table.insert(items, {
      display = string.format("[Week ago] %s", week_ago),
      value = week_ago
    })
    table.insert(items, {
      display = string.format("[Month ago] %s", month_ago),
      value = month_ago
    })

    -- For DateTime format (used in SQL queries)
    table.insert(items, {
      display = string.format("[DateTime Today] %s", today),
      value = string.format("%s, %s, %s", today:match("(%d+)-(%d+)-(%d+)"))
    })
    table.insert(items, {
      display = string.format("[DateTime Yesterday] %s", yesterday),
      value = string.format("%s, %s, %s", yesterday:match("(%d+)-(%d+)-(%d+)"))
    })
  end

  -- Add predefined sets that contain this parameter
  local buffer_sets = get_buffer_param_sets()
  for set_name, params in pairs(buffer_sets) do
    if params[param_name] then
      table.insert(items, {
        display = string.format("[Set: %s] %s", set_name, params[param_name]),
        value = params[param_name],
        is_set = true,
        set_name = set_name
      })
    end
  end

  -- Add global sets
  for set_name, params in pairs(param_data.sets) do
    if params[param_name] then
      table.insert(items, {
        display = string.format("[Global: %s] %s", set_name, params[param_name]),
        value = params[param_name],
        is_set = true,
        set_name = set_name
      })
    end
  end

  -- Special handling for SOURCE parameter
  if param_name == "SOURCE" or param_name:match("SOURCE") then
    for _, source in ipairs(param_data.common_sources) do
      local is_recent = false
      if param_data.history[param_name] then
        for _, hist in ipairs(param_data.history[param_name]) do
          if hist == source then
            is_recent = true
            break
          end
        end
      end
      if not is_recent then
        table.insert(items, {
          display = string.format("[Source] %s", source),
          value = source
        })
      end
    end
  end

  -- Add history
  if param_data.history[param_name] then
    for i, value in ipairs(param_data.history[param_name]) do
      table.insert(items, {
        display = string.format("[Recent %d] %s", i, value),
        value = value,
        is_history = true
      })
    end
  end

  -- Add last used from any parameter that looks similar
  for other_param, value in pairs(param_data.last_used) do
    if other_param ~= param_name and other_param:match(param_name:match("^(%w+)")) then
      table.insert(items, {
        display = string.format("[Similar: %s] %s", other_param, value),
        value = value
      })
    end
  end

  -- Add custom input option
  table.insert(items, {
    display = "[Enter custom value...]",
    is_custom = true
  })

  -- Create picker
  vim.ui.select(items, {
    prompt = string.format("Select value for {{%s}}:", param_name),
    format_item = function(item)
      return item.display
    end
  }, function(item)
    if not item then
      callback(nil)
      return
    end

    if item.is_custom then
      -- Prompt for custom value
      local default = param_data.last_used[param_name] or ""
      vim.ui.input({
        prompt = string.format("Enter value for {{%s}}: ", param_name),
        default = default
      }, function(value)
        if value then
          add_to_history(param_name, value)
          callback(value)
        else
          callback(nil)
        end
      end)
    else
      -- Use selected value
      add_to_history(param_name, item.value)

      -- If it's from a set, optionally apply all params from that set
      if item.is_set and item.set_name then
        vim.notify(string.format("Using parameter set: %s", item.set_name), vim.log.levels.INFO)
      end

      callback(item.value)
    end
  end)
end

-- Resolve all parameters in text
function M.resolve_parameters(text, callback)
  local params = M.extract_parameters(text)

  if #params == 0 then
    callback(text)
    return
  end

  local resolved = {}
  -- params is already a list from extract_parameters
  local remaining = {}
  for i, v in ipairs(params) do
    remaining[i] = v
  end

  local function resolve_next()
    if #remaining == 0 then
      -- All resolved, replace in text
      local result = text
      for param, value in pairs(resolved) do
        -- Check if parameter is inside JSON (within quotes)
        -- Look for patterns like "{{PARAM}}" which indicate JSON context
        local json_pattern = '"{{' .. param .. '}}"'
        local json_pattern_alt = "'{{" .. param .. "}}'"

        if result:find(json_pattern, 1, true) then
          -- Replace with escaped value for JSON
          local escaped_value = value:gsub('\\', '\\\\'):gsub('"', '\\"')
          result = result:gsub('"{{' .. param .. '}}"', '"' .. escaped_value .. '"')
        elseif result:find(json_pattern_alt, 1, true) then
          -- Replace with escaped value for JSON (single quotes)
          local escaped_value = value:gsub('\\', '\\\\'):gsub("'", "\\'")
          result = result:gsub("'{{" .. param .. "}}'", "'" .. escaped_value .. "'")
        else
          -- Regular replacement
          result = result:gsub("{{" .. param .. "}}", value)
          result = result:gsub("${" .. param .. "}", value)
        end
      end
      callback(result)
      return
    end

    local param = table.remove(remaining, 1)
    M.pick_parameter_value(param, function(value)
      if value then
        resolved[param] = value
        resolve_next()
      else
        -- Cancelled
        callback(nil)
      end
    end)
  end

  resolve_next()
end

-- Quick parameter cycling (next/previous value from history)
function M.cycle_parameter(direction)
  local line = vim.api.nvim_get_current_line()
  local col = vim.api.nvim_win_get_cursor(0)[2]

  -- Find parameter under cursor
  local param_pattern = "{{([%w_]+)}}"
  local alt_pattern = "${([%w_]+)}"

  local param_name = nil
  local start_pos, end_pos = nil, nil
  local pattern_type = nil

  -- Check both patterns
  for match_start, name, match_end in line:gmatch("(){{([%w_]+)}}()") do
    if match_start <= col + 1 and match_end > col + 1 then
      param_name = name
      start_pos = match_start
      end_pos = match_end
      pattern_type = "{{}}"
      break
    end
  end

  if not param_name then
    for match_start, name, match_end in line:gmatch("()${([%w_]+)}()") do
      if match_start <= col + 1 and match_end > col + 1 then
        param_name = name
        start_pos = match_start
        end_pos = match_end
        pattern_type = "${}"
        break
      end
    end
  end

  if not param_name then
    vim.notify("No parameter under cursor", vim.log.levels.WARN)
    return
  end

  -- Get history for this parameter
  local history = param_data.history[param_name] or {}
  if #history == 0 then
    vim.notify(string.format("No history for parameter: %s", param_name), vim.log.levels.INFO)
    return
  end

  -- Find current value in line
  local current_value = line:sub(end_pos + 1):match('^"([^"]*)"')
  if not current_value then
    current_value = line:sub(end_pos + 1):match("^'([^']*)'")
  end

  -- Find position in history
  local current_index = 0
  if current_value then
    for i, v in ipairs(history) do
      if v == current_value then
        current_index = i
        break
      end
    end
  end

  -- Calculate next index
  local new_index
  if direction == "next" then
    new_index = current_index + 1
    if new_index > #history then new_index = 1 end
  else
    new_index = current_index - 1
    if new_index < 1 then new_index = #history end
  end

  local new_value = history[new_index]

  -- Replace in line
  local before = line:sub(1, end_pos)
  local after = line:sub(end_pos + 1)

  -- Update the value after the parameter
  after = after:gsub('^"[^"]*"', '"' .. new_value .. '"', 1)
  if after == line:sub(end_pos + 1) then
    after = after:gsub("^'[^']*'", "'" .. new_value .. "'", 1)
  end

  local new_line = before .. after
  vim.api.nvim_set_current_line(new_line)

  vim.notify(string.format("%s = %s [%d/%d]", param_name, new_value, new_index, #history), vim.log.levels.INFO)
end

-- Save current parameter values as a set
function M.save_parameter_set()
  local line = vim.api.nvim_get_current_line()
  local params = M.extract_parameters(line)

  if #params == 0 then
    vim.notify("No parameters found in current line", vim.log.levels.WARN)
    return
  end

  vim.ui.input({
    prompt = "Enter name for parameter set: "
  }, function(name)
    if not name or name == "" then
      return
    end

    local set = {}
    for param in pairs(params) do
      local value = param_data.last_used[param]
      if value then
        set[param] = value
      end
    end

    param_data.sets[name] = set
    M.save_data()

    vim.notify(string.format("Saved parameter set '%s' with %d parameters", name, vim.tbl_count(set)), vim.log.levels.INFO)
  end)
end

-- Initialize
M.load_data()

return M