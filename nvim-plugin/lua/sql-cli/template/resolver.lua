-- Template Resolver Module
-- Resolves @{TYPE:args} patterns to their actual values

local M = {}

-- Resolution context
local context = {
  macros = {},
  config = {},
  cache = {},
  depth = 0,
  max_depth = 10
}

-- Initialize resolver with macros and config
function M.init(macros, config, templates)
  context.macros = macros or {}
  context.config = config or macros._CONFIG or {}
  context.templates = templates or {}
  context.cache = {}
  context.depth = 0
end

-- Resolve a single pattern
function M.resolve(pattern, interactive)
  local cache_key = pattern.type .. ":" .. (pattern.args or "")

  -- Check cache
  if context.cache[cache_key] then
    return context.cache[cache_key]
  end

  local result = nil

  if pattern.type == "VAR" then
    -- Variable lookup: CONFIG first, then environment
    local var_name = pattern.arg_list[1]
    if context.config[var_name] then
      result = context.config[var_name]
    else
      result = os.getenv(var_name) or vim.env[var_name] or ""
    end

    -- Handle @{ENV:name} references within the value
    if result:match("@{ENV:([^}]+)}") then
      local env_name = result:match("@{ENV:([^}]+)}")
      local env_value = os.getenv(env_name) or vim.env[env_name] or ""
      result = result:gsub("@{ENV:" .. env_name .. "}", env_value)
    end

  elseif pattern.type == "ENV" then
    -- Direct environment variable lookup
    local var_name = pattern.arg_list[1]
    result = os.getenv(var_name) or vim.env[var_name] or ""

  elseif pattern.type == "JVAR" then
    -- JSON-escaped variable
    local var_name = pattern.arg_list[1]
    local value
    if context.config[var_name] then
      value = context.config[var_name]
    else
      value = os.getenv(var_name) or vim.env[var_name] or ""
    end
    result = '\\"' .. value .. '\\"'

  elseif pattern.type == "INPUT" then
    if interactive then
      -- User input prompt
      local prompt = pattern.arg_list[1] or "Value"
      local default = pattern.arg_list[2] or ""
      local value = nil
      vim.ui.input({
        prompt = prompt .. ": ",
        default = default
      }, function(input)
        value = input
      end)
      -- Wait for input (synchronous simulation for now)
      vim.wait(10000, function() return value ~= nil end)
      result = value or ""
    else
      result = pattern.arg_list[2] or "DEFAULT_INPUT"
    end

  elseif pattern.type == "JINPUT" then
    if interactive then
      -- JSON-escaped user input
      local prompt = pattern.arg_list[1] or "Value"
      local default = pattern.arg_list[2] or ""
      local value = nil
      vim.ui.input({
        prompt = prompt .. " (JSON): ",
        default = default
      }, function(input)
        value = input
      end)
      vim.wait(10000, function() return value ~= nil end)
      result = '\\"' .. (value or "") .. '\\"'
    else
      result = '\\"' .. (pattern.arg_list[2] or "DEFAULT_INPUT") .. '\\"'
    end

  elseif pattern.type == "PICKER" then
    local macro_name = pattern.arg_list[1]
    local prompt = pattern.arg_list[2] or macro_name

    -- Get choices from macro
    local choices = M.parse_list_macro(context.macros[macro_name])

    if interactive and #choices > 0 then
      local selected = nil
      vim.ui.select(choices, {
        prompt = prompt .. ": "
      }, function(choice)
        selected = choice
      end)
      vim.wait(10000, function() return selected ~= nil end)
      result = selected or choices[1]
    else
      result = choices[1] or "DEFAULT_CHOICE"
    end

  elseif pattern.type == "JPICKER" then
    local macro_name = pattern.arg_list[1]
    local prompt = pattern.arg_list[2] or macro_name

    -- Get choices from macro
    local choices = M.parse_list_macro(context.macros[macro_name])

    if interactive and #choices > 0 then
      local selected = nil
      vim.ui.select(choices, {
        prompt = prompt .. " (JSON): "
      }, function(choice)
        selected = choice
      end)
      vim.wait(10000, function() return selected ~= nil end)
      result = '\\"' .. (selected or choices[1]) .. '\\"'
    else
      result = '\\"' .. (choices[1] or "DEFAULT_CHOICE") .. '\\"'
    end

  elseif pattern.type == "DATE" then
    if interactive then
      local prompt = pattern.arg_list[1] or "Date"
      local selected = nil

      -- Simple date picker
      local today = os.date("%Y-%m-%d")
      local dates = {
        today,
        os.date("%Y-%m-%d", os.time() - 86400),
        os.date("%Y-%m-%d", os.time() - 7 * 86400),
        "Custom..."
      }

      vim.ui.select(dates, {
        prompt = prompt .. ": "
      }, function(choice)
        if choice == "Custom..." then
          vim.ui.input({
            prompt = "Enter date (YYYY-MM-DD): ",
            default = today
          }, function(input)
            selected = input
          end)
        else
          selected = choice
        end
      end)

      vim.wait(10000, function() return selected ~= nil end)

      if selected then
        local year, month, day = selected:match("(%d+)-(%d+)-(%d+)")
        if year then
          result = string.format("DateTime(%s, %s, %s)", year, tonumber(month), tonumber(day))
        else
          result = selected
        end
      else
        result = "DateTime(2025, 1, 1)"
      end
    else
      result = "DateTime(2025, 1, 1)"
    end

  elseif pattern.type == "MACRO" then
    -- Macro expansion (watch for recursion)
    local macro_name = pattern.arg_list[1]
    if context.macros[macro_name] then
      result = context.macros[macro_name]
    else
      -- Keep original if not found
      result = "@{MACRO:" .. macro_name .. "}"
    end

  elseif pattern.type == "TEMPLATE" then
    -- Template expansion - look for templates in loaded files
    local template_name = pattern.arg_list[1]
    if context.templates and context.templates[template_name] then
      result = context.templates[template_name].content
    else
      -- Keep original if not found
      result = "@{TEMPLATE:" .. template_name .. "}"
    end

  else
    -- Unknown type, keep original
    result = pattern.match
  end

  -- Cache the result
  context.cache[cache_key] = result
  return result
end

-- Async version of resolve for interactive mode
function M.resolve_async(pattern, interactive, callback, debug_callback)
  local cache_key = pattern.type .. ":" .. (pattern.args or "")

  -- Check cache
  if context.cache[cache_key] then
    vim.schedule(function() callback(context.cache[cache_key]) end)
    return
  end

  -- Handle different pattern types
  if pattern.type == "VAR" then
    -- Variable lookup
    local var_name = pattern.arg_list[1]
    local result
    if context.config[var_name] then
      result = context.config[var_name]
    else
      result = os.getenv(var_name) or vim.env[var_name] or ""
    end
    -- Handle @{ENV:name} references
    if result:match("@{ENV:([^}]+)}") then
      local env_name = result:match("@{ENV:([^}]+)}")
      local env_value = os.getenv(env_name) or vim.env[env_name] or ""
      result = result:gsub("@{ENV:" .. env_name .. "}", env_value)
    end
    context.cache[cache_key] = result
    callback(result)

  elseif pattern.type == "ENV" then
    local var_name = pattern.arg_list[1]
    local result = os.getenv(var_name) or vim.env[var_name] or ""
    context.cache[cache_key] = result
    callback(result)

  elseif pattern.type == "JVAR" then
    -- JSON-escaped variable
    local var_name = pattern.arg_list[1]
    local value
    if context.config[var_name] then
      value = context.config[var_name]
    else
      value = os.getenv(var_name) or vim.env[var_name] or ""
    end
    local result = '\\"' .. value .. '\\"'
    context.cache[cache_key] = result
    callback(result)

  elseif pattern.type == "INPUT" then
    if interactive then
      local prompt = pattern.arg_list[1] or "Value"
      local default = pattern.arg_list[2] or ""
      vim.ui.input({
        prompt = prompt .. ": ",
        default = default
      }, function(input)
        local result = input or ""
        context.cache[cache_key] = result
        callback(result)
      end)
    else
      local result = pattern.arg_list[2] or "DEFAULT_INPUT"
      callback(result)
    end

  elseif pattern.type == "JINPUT" then
    if interactive then
      local prompt = pattern.arg_list[1] or "Value"
      local default = pattern.arg_list[2] or ""
      vim.ui.input({
        prompt = prompt .. " (JSON): ",
        default = default
      }, function(input)
        local result = '\\"' .. (input or "") .. '\\"'
        context.cache[cache_key] = result
        callback(result)
      end)
    else
      local result = '\\"' .. (pattern.arg_list[2] or "DEFAULT_INPUT") .. '\\"'
      callback(result)
    end

  elseif pattern.type == "PICKER" or pattern.type == "JPICKER" then
    local macro_name = pattern.arg_list[1]
    local prompt = pattern.arg_list[2] or macro_name
    local choices = M.parse_list_macro(context.macros[macro_name])

    if interactive and #choices > 0 then
      vim.ui.select(choices, {
        prompt = prompt .. (pattern.type == "JPICKER" and " (JSON): " or ": ")
      }, function(choice)
        local result
        if choice then
          result = pattern.type == "JPICKER" and ('\\"' .. choice .. '\\"') or choice
        else
          result = pattern.type == "JPICKER" and '\\"\\"' or ""
        end
        context.cache[cache_key] = result
        callback(result)
      end)
    else
      local result = pattern.type == "JPICKER" and '\\"\\"' or (choices[1] or "")
      callback(result)
    end

  elseif pattern.type == "DATE" then
    if interactive then
      local prompt = pattern.arg_list[1] or "Date"
      -- Simple date picker
      local today = os.date("%Y-%m-%d")
      local dates = {
        today,
        os.date("%Y-%m-%d", os.time() - 86400),
        os.date("%Y-%m-%d", os.time() - 7 * 86400),
        "Custom..."
      }

      vim.ui.select(dates, {
        prompt = prompt .. ": "
      }, function(choice)
        if choice == "Custom..." then
          vim.ui.input({
            prompt = "Enter date (YYYY-MM-DD): ",
            default = today
          }, function(input)
            local date = input or today
            local year, month, day = date:match("(%d+)-(%d+)-(%d+)")
            local result
            if year then
              result = string.format("DateTime(%s, %s, %s)", year, tonumber(month), tonumber(day))
            else
              result = date
            end
            context.cache[cache_key] = result
            callback(result)
          end)
        else
          local date = choice or today
          local year, month, day = date:match("(%d+)-(%d+)-(%d+)")
          local result
          if year then
            result = string.format("DateTime(%s, %s, %s)", year, tonumber(month), tonumber(day))
          else
            result = date
          end
          context.cache[cache_key] = result
          callback(result)
        end
      end)
    else
      callback("DateTime(2025, 1, 1)")
    end

  elseif pattern.type == "MACRO" then
    local macro_name = pattern.arg_list[1]
    local result = context.macros[macro_name] or ("@{MACRO:" .. macro_name .. "}")
    context.cache[cache_key] = result
    callback(result)

  elseif pattern.type == "TEMPLATE" then
    local template_name = pattern.arg_list[1]
    local result
    if context.templates and context.templates[template_name] then
      result = context.templates[template_name].content
    else
      result = "@{TEMPLATE:" .. template_name .. "}"
    end
    context.cache[cache_key] = result
    callback(result)

  else
    -- Unknown type
    callback(pattern.match)
  end
end

-- Parse a list macro into array of choices
function M.parse_list_macro(macro_content)
  local choices = {}
  if not macro_content then
    return choices
  end

  for line in macro_content:gmatch("[^\n]+") do
    local choice = line:match("^%s*(.-)%s*$")
    if choice ~= "" and not choice:match("^%-%-") then
      table.insert(choices, choice)
    end
  end

  return choices
end

-- Test resolver (non-interactive)
function M.test_resolve(pattern, test_values)
  local key = pattern.type .. ":" .. (pattern.args or "")
  if test_values and test_values[key] then
    return test_values[key]
  end

  -- Generate default test values
  if pattern.type == "VAR" or pattern.type == "ENV" then
    return "TEST_VAR"
  elseif pattern.type == "JVAR" then
    return '\\"TEST_VAR\\"'
  elseif pattern.type == "INPUT" then
    return "TEST_INPUT"
  elseif pattern.type == "JINPUT" then
    return '\\"TEST_INPUT\\"'
  elseif pattern.type == "PICKER" then
    return "TEST_CHOICE"
  elseif pattern.type == "JPICKER" then
    return '\\"TEST_CHOICE\\"'
  elseif pattern.type == "DATE" then
    return "DateTime(2025, 1, 1)"
  elseif pattern.type == "MACRO" then
    return "TEST_MACRO_CONTENT"
  else
    return pattern.match
  end
end

return M