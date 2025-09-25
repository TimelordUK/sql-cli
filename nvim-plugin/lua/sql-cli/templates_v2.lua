-- SQL-CLI Template System v2 - Clean rewrite
-- Supports @{TYPE:args} syntax only - no legacy ${VAR} support

local M = {}
local token_manager = require('sql-cli.token_manager')

-- Debug logging
local debug_enabled = false
local function debug_log(msg)
  if debug_enabled then
    vim.notify("[Templates] " .. msg, vim.log.levels.DEBUG)
  end
end

-- Core data structures
local expansion_depth = 0
local MAX_EXPANSION_DEPTH = 10
local function_cache = {}  -- Cache resolved values within single expansion

-- Parse file for macro definitions
function M.parse_file_macros()
  local macros = {}
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  local current_macro = nil
  local macro_lines = {}

  for _, line in ipairs(lines) do
    -- Check for macro start
    local macro_name = line:match("^%s*%-%-+%s*MACRO:%s*([%w_]+)")
    if macro_name then
      -- Save previous macro if exists
      if current_macro then
        macros[current_macro] = table.concat(macro_lines, "\n")
      end
      current_macro = macro_name
      macro_lines = {}
    elseif line:match("^%s*%-%-+%s*END%s+MACRO") then
      -- Save current macro
      if current_macro then
        macros[current_macro] = table.concat(macro_lines, "\n")
        current_macro = nil
        macro_lines = {}
      end
    elseif current_macro then
      -- Accumulate macro lines (skip comment prefix)
      local content = line:match("^%s*%-%-+%s*(.*)") or line
      table.insert(macro_lines, content)
    end
  end

  -- Parse CONFIG macro specially
  if macros.CONFIG then
    local config = {}
    for line in macros.CONFIG:gmatch("[^\n]+") do
      local key, value = line:match("^%s*([%w_]+)%s*=%s*(.-)%s*$")
      if key and value then
        config[key] = value
      end
    end
    macros._CONFIG = config
  end

  return macros
end

-- Pattern collection phase
function M.collect_patterns(text)
  local patterns = {}
  local index = 1
  local result_text = text

  -- Match @{TYPE:args} patterns, including those with surrounding quotes
  -- Pattern: optional quotes + @{TYPE:args} + optional quotes
  local pattern = '(\\"?)("?)@{([^:}]+):([^}]*)}("?)(\\"?)'

  for full_match in text:gmatch('[\\"]?["]?@{[^}]+}["]?[\\"]?') do
    -- Parse components
    local prefix_esc, prefix_quote, func_type, args, suffix_quote, suffix_esc =
      full_match:match('^(\\"?)("?)@{([^:}]+):([^}]*)}("?)(\\"?)$')

    if func_type and args then
      local placeholder = "__PATTERN_" .. index .. "__"

      -- Determine quote context
      local has_escaped_quotes = (prefix_esc ~= "" and suffix_esc ~= "")
      local has_quotes = (prefix_quote ~= "" and suffix_quote ~= "") and not has_escaped_quotes

      -- Split args by colon
      local arg_list = {}
      for arg in args:gmatch("[^:]+") do
        table.insert(arg_list, arg:match("^%s*(.-)%s*$"))
      end

      table.insert(patterns, {
        match = full_match,
        type = func_type,
        args = args,
        arg_list = arg_list,
        placeholder = placeholder,
        has_escaped_quotes = has_escaped_quotes,
        has_quotes = has_quotes,
        index = index
      })

      -- Replace with placeholder preserving quote structure
      local replacement
      if has_escaped_quotes then
        replacement = '\\"' .. placeholder .. '\\"'
      elseif has_quotes then
        replacement = '"' .. placeholder .. '"'
      else
        replacement = placeholder
      end

      result_text = result_text:gsub(full_match:gsub("([%^%$%(%)%%%.%[%]%*%+%-%?])", "%%%1"), replacement)
      index = index + 1
    end
  end

  return {
    patterns = patterns,
    text = result_text
  }
end

-- Resolve a single pattern
function M.resolve_pattern(pattern, test_mode, test_values)
  local cache_key = pattern.type .. ":" .. pattern.args

  -- Check cache first
  if function_cache[cache_key] then
    debug_log("Using cached value for " .. cache_key)
    return function_cache[cache_key]
  end

  local result

  -- In test mode, use provided values or defaults
  if test_mode then
    if test_values and test_values[cache_key] then
      result = test_values[cache_key]
    else
      -- Generate default test values
      if pattern.type == "VAR" then
        result = "TEST_VAR"
      elseif pattern.type == "JVAR" then
        result = '\\"TEST_VAR\\"'
      elseif pattern.type == "INPUT" then
        result = "TEST_INPUT"
      elseif pattern.type == "JINPUT" then
        result = '\\"TEST_INPUT\\"'
      elseif pattern.type == "PICKER" then
        result = "TEST_CHOICE"
      elseif pattern.type == "JPICKER" then
        result = '\\"TEST_CHOICE\\"'
      elseif pattern.type == "DATE" then
        result = "DateTime(2025, 1, 1)"
      elseif pattern.type == "MACRO" then
        result = "TEST_MACRO_CONTENT"
      else
        result = pattern.match
      end
    end
    function_cache[cache_key] = result
    return result
  end

  -- Actual resolution for interactive mode
  local macros = M.parse_file_macros()

  if pattern.type == "VAR" then
    -- Variable lookup: CONFIG first, then environment
    local var_name = pattern.arg_list[1]
    if macros._CONFIG and macros._CONFIG[var_name] then
      result = macros._CONFIG[var_name]
    else
      result = os.getenv(var_name) or vim.env[var_name] or ""
    end

  elseif pattern.type == "JVAR" then
    -- JSON-escaped variable
    local var_name = pattern.arg_list[1]
    local value
    if macros._CONFIG and macros._CONFIG[var_name] then
      value = macros._CONFIG[var_name]
    else
      value = os.getenv(var_name) or vim.env[var_name] or ""
    end
    result = '\\"' .. value .. '\\"'

  elseif pattern.type == "INPUT" then
    -- User input prompt
    local prompt = pattern.arg_list[1] or "Value"
    vim.ui.input({prompt = prompt .. ": "}, function(value)
      result = value or ""
    end)

  elseif pattern.type == "JINPUT" then
    -- JSON-escaped user input
    local prompt = pattern.arg_list[1] or "Value"
    vim.ui.input({prompt = prompt .. " (JSON): "}, function(value)
      result = '\\"' .. (value or "") .. '\\"'
    end)

  elseif pattern.type == "PICKER" then
    -- Choice from macro list
    local macro_name = pattern.arg_list[1]
    local prompt = pattern.arg_list[2] or macro_name
    local choices = {}

    if macros[macro_name] then
      for line in macros[macro_name]:gmatch("[^\n]+") do
        local choice = line:match("^%s*(.-)%s*$")
        if choice ~= "" and not choice:match("^%-%-") then
          table.insert(choices, choice)
        end
      end
    end

    if #choices > 0 then
      vim.ui.select(choices, {prompt = prompt .. ": "}, function(choice)
        result = choice or ""
      end)
    else
      result = ""
    end

  elseif pattern.type == "JPICKER" then
    -- JSON-escaped picker
    local macro_name = pattern.arg_list[1]
    local prompt = pattern.arg_list[2] or macro_name
    local choices = {}

    if macros[macro_name] then
      for line in macros[macro_name]:gmatch("[^\n]+") do
        local choice = line:match("^%s*(.-)%s*$")
        if choice ~= "" and not choice:match("^%-%-") then
          table.insert(choices, choice)
        end
      end
    end

    if #choices > 0 then
      vim.ui.select(choices, {prompt = prompt .. " (JSON): "}, function(choice)
        result = '\\"' .. (choice or "") .. '\\"'
      end)
    else
      result = '\\"\\"'
    end

  elseif pattern.type == "DATE" then
    -- Date picker
    local prompt = pattern.arg_list[1] or "Date"
    M.quick_date_picker(function(date)
      local year, month, day = date:match("(%d+)-(%d+)-(%d+)")
      if year then
        result = string.format("DateTime(%s, %s, %s)", year, tonumber(month), tonumber(day))
      else
        result = date
      end
    end)

  elseif pattern.type == "MACRO" then
    -- Macro expansion (recursive)
    local macro_name = pattern.arg_list[1]
    if macros[macro_name] then
      result = macros[macro_name]
    else
      result = "@{MACRO:" .. macro_name .. "}"  -- Keep original if not found
    end

  else
    -- Unknown type, keep original
    result = pattern.match
  end

  -- Cache the result
  function_cache[cache_key] = result
  return result
end

-- Main expansion function
function M.expand(text, callback, test_mode, test_values)
  -- Check recursion depth
  expansion_depth = expansion_depth + 1
  if expansion_depth > MAX_EXPANSION_DEPTH then
    vim.notify("Maximum expansion depth reached. Check for circular macro references.", vim.log.levels.WARN)
    expansion_depth = 0
    callback(text)
    return
  end

  -- Clear cache at start of new expansion
  if expansion_depth == 1 then
    function_cache = {}
  end

  -- Collect patterns
  local collection = M.collect_patterns(text)

  -- If no patterns, return original
  if #collection.patterns == 0 then
    expansion_depth = expansion_depth - 1
    callback(text)
    return
  end

  -- Resolve all patterns
  local resolved = {}
  local async_count = 0
  local async_complete = 0

  -- Count async operations
  for _, pattern in ipairs(collection.patterns) do
    if not test_mode and (pattern.type == "INPUT" or pattern.type == "JINPUT" or
        pattern.type == "PICKER" or pattern.type == "JPICKER" or pattern.type == "DATE") then
      async_count = async_count + 1
    end
  end

  -- Resolve each pattern
  for i, pattern in ipairs(collection.patterns) do
    if test_mode or (pattern.type ~= "INPUT" and pattern.type ~= "JINPUT" and
        pattern.type ~= "PICKER" and pattern.type ~= "JPICKER" and pattern.type ~= "DATE") then
      -- Synchronous resolution
      resolved[i] = M.resolve_pattern(pattern, test_mode, test_values)
    else
      -- Asynchronous resolution
      M.resolve_pattern(pattern, test_mode, test_values, function(value)
        resolved[i] = value
        async_complete = async_complete + 1

        -- Check if all async operations complete
        if async_complete >= async_count then
          -- Perform substitution
          local result = collection.text
          for j, p in ipairs(collection.patterns) do
            local replacement = resolved[j] or p.match
            -- Escape special characters for gsub
            local safe_replacement = replacement:gsub("%%", "%%%%")
            result = result:gsub(p.placeholder, safe_replacement)
          end

          -- Check for new patterns (recursive expansion)
          if result:match("@{[^}]+}") then
            M.expand(result, callback, test_mode, test_values)
          else
            expansion_depth = expansion_depth - 1
            callback(result)
          end
        end
      end)
    end
  end

  -- If all synchronous (test mode)
  if test_mode then
    local result = collection.text
    for i, pattern in ipairs(collection.patterns) do
      local replacement = resolved[i] or pattern.match
      local safe_replacement = replacement:gsub("%%", "%%%%")
      result = result:gsub(pattern.placeholder, safe_replacement)
    end

    -- Check for new patterns
    if result:match("@{[^}]+}") then
      M.expand(result, callback, test_mode, test_values)
    else
      expansion_depth = expansion_depth - 1
      callback(result)
    end
  end
end

-- Test function for headless expansion
function M.test_expand(text, test_values)
  local result = nil
  M.expand(text, function(expanded)
    result = expanded
  end, true, test_values)
  return result
end

-- Date picker helper (simplified)
function M.quick_date_picker(callback)
  local today = os.date("%Y-%m-%d")
  local dates = {
    today,
    os.date("%Y-%m-%d", os.time() - 86400),  -- Yesterday
    os.date("%Y-%m-%d", os.time() - 7 * 86400),  -- Week ago
    os.date("%Y-01-01"),  -- Start of year
    "Custom..."
  }

  vim.ui.select(dates, {prompt = "Select date: "}, function(choice)
    if choice == "Custom..." then
      vim.ui.input({prompt = "Enter date (YYYY-MM-DD): ", default = today}, callback)
    else
      callback(choice or today)
    end
  end)
end

-- Inline macro expansion at cursor
function M.expand_at_cursor()
  local line = vim.api.nvim_get_current_line()
  local row, col = unpack(vim.api.nvim_win_get_cursor(0))

  -- Find macro pattern at cursor
  local patterns = {
    { pattern = "@{([^}]+)}", prefix = "@{", suffix = "}" },
    { pattern = "@([%w_]+)", prefix = "@", suffix = "" }
  }

  local start_pos, end_pos, macro_content

  for _, p in ipairs(patterns) do
    local pos = 1
    while pos <= #line do
      local s, e, content = line:find(p.pattern, pos)
      if s and s <= col + 1 and e >= col + 1 then
        start_pos = s
        end_pos = e
        macro_content = p.prefix .. content .. p.suffix
        break
      end
      pos = (e or pos) + 1
    end
    if start_pos then break end
  end

  if not start_pos then
    vim.notify("No macro found at cursor position", vim.log.levels.INFO)
    return
  end

  -- Expand the macro
  M.expand(macro_content, function(expanded)
    if expanded then
      -- Replace inline
      local new_line = line:sub(1, start_pos - 1) .. expanded .. line:sub(end_pos + 1)
      vim.api.nvim_set_current_line(new_line)
      -- Move cursor to end of replacement
      vim.api.nvim_win_set_cursor(0, {row, start_pos + #expanded - 1})
    end
  end, false)
end

-- Enable/disable debug logging
function M.set_debug(enabled)
  debug_enabled = enabled
end

return M