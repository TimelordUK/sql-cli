-- SQL Template/Macro System for SQL-CLI
-- Allows quick reuse of common query patterns with variable substitution

local M = {}
local token_manager = require('sql-cli.token_manager')

-- Debug mode flag
M.debug = false  -- Set to true to enable debug output

-- Store user-defined templates
M.templates = {}

-- Debug logging function
local function debug_log(msg)
  if M.debug then
    if vim and vim.notify then
      vim.notify("[TEMPLATE DEBUG] " .. msg, vim.log.levels.INFO)
    else
      print("[TEMPLATE DEBUG] " .. msg)
    end
  end
end

-- Common variable definitions with quick selectors
M.common_variables = {
  -- Default sources - can be overridden by SOURCES macro in file
  sources = {
    "Bloomberg_FIX_FX",
    "Bloomberg_FIX_Equity",
    "Reuters_FX",
    "Internal_Trade_System",
    "Manual_Entry"
  },

  -- Quick date helpers
  date_helpers = {
    today = function() return os.date("%Y-%m-%d") end,
    yesterday = function()
      local t = os.time() - (24 * 60 * 60)
      return os.date("%Y-%m-%d", t)
    end,
    this_week_start = function()
      local t = os.time()
      local date = os.date("*t", t)
      local days_since_monday = (date.wday - 2) % 7
      local monday = t - (days_since_monday * 24 * 60 * 60)
      return os.date("%Y-%m-%d", monday)
    end,
    last_business_day = function()
      local t = os.time()
      local date = os.date("*t", t)
      -- If Monday, return Friday
      if date.wday == 2 then
        t = t - (3 * 24 * 60 * 60)
      -- If Sunday, return Friday
      elseif date.wday == 1 then
        t = t - (2 * 24 * 60 * 60)
      else
        t = t - (24 * 60 * 60)
      end
      return os.date("%Y-%m-%d", t)
    end
  }
}

-- Default templates
M.default_templates = {
  trade_fetch = {
    name = "Fetch Trades with Filters",
    template = [[
WITH WEB trades AS (
    URL '${BASE_URL:http://localhost:5001}/trades'
    METHOD POST
    BODY '${WHERE_JSON}'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer ${API_TOKEN}',
        'Content-Type': 'application/json'
    )
)
SELECT
    TradeId,
    Source,
    Symbol,
    Quantity,
    Price,
    TradeDate,
    Status
FROM trades
WHERE ${ADDITIONAL_WHERE:1=1}
ORDER BY TradeDate DESC]],
    variables = {
      SOURCE = { prompt = "Trade Source", default = "Bloomberg_FIX_FX", type = "select", options = "sources" },
      TRADE_DATE = { prompt = "Trade Date", type = "date_builder" },
      COLUMNS = { prompt = "Columns to select", default = "*", optional = true },
      API_TOKEN = { prompt = "API Token", default = "${JWT_TOKEN}", env = true },
      ADDITIONAL_WHERE = { prompt = "Additional WHERE clause", default = "1=1", optional = true },
      BASE_URL = { prompt = "API URL", default = "http://localhost:5001", optional = true }
    },
    -- Special builder function for complex JSON
    build = function(vars)
      -- Build WHERE clause automatically with proper escaping
      local where_parts = {}

      if vars.SOURCE and vars.SOURCE ~= "" then
        table.insert(where_parts, string.format('Source = \\"%s\\"', vars.SOURCE))
      end

      if vars.TRADE_DATE and vars.TRADE_DATE ~= "" then
        -- Parse date and build TradeDate function call
        local year, month, day = vars.TRADE_DATE:match("(%d+)-(%d+)-(%d+)")
        if year then
          table.insert(where_parts, string.format('TradeDate = TradeDate(%s, %s, %s)', year, month, day))
        end
      end

      local where_clause = table.concat(where_parts, " and ")

      -- Build select clause (columns)
      local select_clause = vars.COLUMNS or "*"

      -- Build JSON body with select and where (no limit)
      vars.WHERE_JSON = string.format('{\n        "select": "%s",\n        "where": "%s"\n    }',
        select_clause,
        where_clause
      )

      return vars
    end
  },

  trade_fetch_simple = {
    name = "Simple Trade Fetch (Source only)",
    template = [[
WITH WEB trades AS (
    URL '${BASE_URL:http://localhost:5001}/trades'
    METHOD POST
    BODY '{
        "select": "${COLUMNS:*}",
        "where": "Source = \\"${SOURCE}\\""
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT * FROM trades
ORDER BY TradeDate DESC]],
    variables = {
      SOURCE = { prompt = "Trade Source", default = "Bloomberg_FIX_FX", type = "select", options = "sources" },
      COLUMNS = { prompt = "Columns", default = "*", optional = true },
      BASE_URL = { prompt = "API URL", default = "http://localhost:5001", optional = true }
    }
  },

  trade_summary = {
    name = "Trade Summary by Source",
    template = [[
WITH WEB trades AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    BODY '{
        "select": "*",
        "where": "TradeDate >= TradeDate(${START_YEAR}, ${START_MONTH}, ${START_DAY}) and TradeDate <= TradeDate(${END_YEAR}, ${END_MONTH}, ${END_DAY})"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT
    Source,
    COUNT(*) as trade_count,
    SUM(Quantity * Price) as total_value,
    AVG(Price) as avg_price,
    MIN(TradeDate) as first_trade,
    MAX(TradeDate) as last_trade
FROM trades
GROUP BY Source
ORDER BY trade_count DESC]],
    variables = {
      START_YEAR = { prompt = "Start Year", default = function() return os.date("%Y") end },
      START_MONTH = { prompt = "Start Month", default = function() return os.date("%m") end },
      START_DAY = { prompt = "Start Day", default = "1" },
      END_YEAR = { prompt = "End Year", default = function() return os.date("%Y") end },
      END_MONTH = { prompt = "End Month", default = function() return os.date("%m") end },
      END_DAY = { prompt = "End Day", default = function() return os.date("%d") end }
    }
  },

  trade_fetch_broad = {
    name = "Broad Trade Fetch (All Columns)",
    template = [[
-- Fetch broad column set from trade server, then filter in SQL-CLI
WITH WEB raw_trades AS (
    URL '${BASE_URL:http://localhost:5001}/trades'
    METHOD POST
    BODY '{
        "select": "${COLUMNS:TradeId, Source, Symbol, Quantity, Price, TradeDate, Status, Account, Currency, ExecutionTime, Broker, Commission, SettlementDate}",
        "where": "${WHERE_CLAUSE}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
    HEADERS (
        'Authorization': 'Bearer ${API_TOKEN}',
        'Content-Type': 'application/json'
    )
)
-- Now use SQL-CLI's powerful features to filter and analyze
SELECT *
FROM raw_trades
${LOCAL_FILTER:-- Add your SQL-CLI filters here}]],
    variables = {
      WHERE_CLAUSE = {
        prompt = "Server WHERE clause (basic filter)",
        default = "TradeDate = TradeDate(2025, 9, 19)",
        type = "where_builder"
      },
      COLUMNS = {
        prompt = "Columns (or press Enter for default set)",
        default = "TradeId, Source, Symbol, Quantity, Price, TradeDate, Status, Account, Currency, ExecutionTime, Broker, Commission, SettlementDate",
        optional = true
      },
      API_TOKEN = { prompt = "API Token", default = "${JWT_TOKEN}", env = true },
      BASE_URL = { prompt = "API URL", default = "http://localhost:5001", optional = true },
      LOCAL_FILTER = {
        prompt = "SQL-CLI filter (optional, e.g., WHERE Symbol LIKE 'EUR%')",
        default = "-- Add your SQL-CLI filters here",
        optional = true
      }
    }
  },

  simple_filter = {
    name = "Quick Source/Date Filter",
    template = [[
WITH WEB trades AS (
    URL '${BASE_URL:http://localhost:5001}/trades'
    METHOD POST
    BODY '{
        "select": "*",
        "where": "Source = \\"${SOURCE}\\" and TradeDate = TradeDate(${YEAR}, ${MONTH}, ${DAY})"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT *
FROM trades
ORDER BY ExecutionTime DESC]],
    variables = {
      SOURCE = { prompt = "Trade Source", default = "Bloomberg_FIX_FX", type = "select", options = "sources" },
      YEAR = { prompt = "Year", default = function() return os.date("%Y") end },
      MONTH = { prompt = "Month", default = function() return os.date("%m") end },
      DAY = { prompt = "Day", default = function() return os.date("%d") end },
      BASE_URL = { prompt = "API URL", default = "http://localhost:5001", optional = true }
    }
  }
}

-- Initialize with defaults
function M.setup(config)
  M.config = config or {}

  -- Load default templates
  for key, template in pairs(M.default_templates) do
    M.templates[key] = template
  end

  -- Load user templates from config
  if M.config.templates then
    for key, template in pairs(M.config.templates) do
      M.templates[key] = template
    end
  end
end

-- Helper to escape quotes for JSON body
function M.escape_json_string(str)
  -- For JSON body, we need to escape quotes
  -- Convert single quotes to double quotes and escape them
  str = str:gsub("'", '"')
  str = str:gsub('"', '\\"')
  return str
end

-- Helper to build WHERE clause without manual escaping
function M.build_where_clause(conditions)
  local parts = {}
  for key, value in pairs(conditions) do
    if type(value) == "string" then
      -- Automatically quote string values
      table.insert(parts, string.format('%s = \\"%s\\"', key, value))
    elseif type(value) == "number" then
      table.insert(parts, string.format('%s = %d', key, value))
    else
      -- For complex values like TradeDate function calls
      table.insert(parts, string.format('%s = %s', key, value))
    end
  end
  return table.concat(parts, " and ")
end

-- Substitute variables in template
function M.substitute_variables(template_str, variables)
  local result = template_str

  -- Special handling for WHERE clause building
  if variables.WHERE_CONDITIONS then
    -- Parse WHERE_CONDITIONS as key=value pairs
    local where_clause = M.build_where_clause(variables.WHERE_CONDITIONS)
    variables.WHERE_CLAUSE = where_clause
  end

  -- Replace all ${VAR} or ${VAR:default} patterns
  result = result:gsub("${([^}:]+):?([^}]*)}", function(var, default)
    local value = variables[var]
    if value and value ~= "" then
      return value
    elseif default and default ~= "" then
      return default
    else
      return "${" .. var .. "}"  -- Keep placeholder if no value
    end
  end)

  -- Handle environment variables
  result = result:gsub("${([^}]+)}", function(var)
    -- Check if it's an environment variable
    local env_value = os.getenv(var)
    if env_value then
      return env_value
    else
      -- Check if we have a value for it
      local value = variables[var]
      if value then
        return value
      else
        return "${" .. var .. "}"
      end
    end
  end)

  return result
end

-- Quick template selector
function M.select_template()
  local template_names = {}
  local template_keys = {}

  for key, template in pairs(M.templates) do
    table.insert(template_names, template.name or key)
    table.insert(template_keys, key)
  end

  vim.ui.select(template_names, {
    prompt = "Select SQL Template:",
  }, function(choice, idx)
    if choice and idx then
      M.apply_template(template_keys[idx])
    end
  end)
end

-- Apply a template with variable substitution
function M.apply_template(template_key)
  local template = M.templates[template_key]
  if not template then
    vim.notify("Template not found: " .. template_key, vim.log.levels.ERROR)
    return
  end

  local variables = {}
  local var_definitions = template.variables or {}

  -- Collect variables in order
  local ordered_vars = {}
  for var_name, _ in pairs(var_definitions) do
    table.insert(ordered_vars, var_name)
  end
  table.sort(ordered_vars)

  -- Function to get next variable
  local var_index = 1
  local function get_next_variable()
    if var_index > #ordered_vars then
      -- All variables collected

      -- Run build function if it exists
      if template.build then
        variables = template.build(variables)
      end

      -- Apply template
      local result = M.substitute_variables(template.template, variables)
      M.insert_at_cursor(result)
      return
    end

    local var_name = ordered_vars[var_index]
    local var_def = var_definitions[var_name]
    var_index = var_index + 1

    -- Skip optional variables that have defaults
    if var_def.optional and var_def.default then
      variables[var_name] = var_def.default
      get_next_variable()
      return
    end

    -- Get default value
    local default_value = var_def.default
    if type(default_value) == "function" then
      default_value = default_value()
    end

    -- Check for environment variable if env flag is set
    if var_def.env then
      -- If default contains ${VAR_NAME}, extract and check that env var
      if default_value and type(default_value) == "string" and default_value:match("${([^}]+)}") then
        local env_var_name = default_value:match("${([^}]+)}")
        local env_value = vim.env[env_var_name]
        if env_value then
          variables[var_name] = env_value
          get_next_variable()
          return
        end
      else
        -- Check for env var with the same name as the variable
        local env_value = vim.env[var_name]
        if env_value then
          variables[var_name] = env_value
          get_next_variable()
          return
        end
      end
    end

    -- Handle different variable types
    if var_def.type == "select" then
      local options = var_def.options

      -- If options is a string, look it up in common_variables
      if type(options) == "string" then
        options = M.common_variables[options] or {}
      end

      vim.ui.select(options, {
        prompt = var_def.prompt .. ":",
        default = default_value
      }, function(choice)
        if choice then
          variables[var_name] = choice
          get_next_variable()
        end
      end)
    elseif var_def.type == "date_builder" then
      -- Special date picker
      M.quick_date_picker(function(date)
        variables[var_name] = date
        get_next_variable()
      end)
    elseif var_def.type == "where_builder" then
      -- Special WHERE clause builder
      M.where_clause_builder(function(where_clause)
        variables[var_name] = where_clause
        get_next_variable()
      end)
    else
      -- Regular text input
      vim.ui.input({
        prompt = var_def.prompt .. ": ",
        default = default_value or ""
      }, function(value)
        if value ~= nil then  -- Allow empty string
          variables[var_name] = value
          get_next_variable()
        end
      end)
    end
  end

  -- Start collecting variables
  get_next_variable()
end

-- Quick date picker
function M.quick_date_picker(callback)
  local date_options = {
    "Today - " .. M.common_variables.date_helpers.today(),
    "Yesterday - " .. M.common_variables.date_helpers.yesterday(),
    "Last Business Day - " .. M.common_variables.date_helpers.last_business_day(),
    "This Week Start - " .. M.common_variables.date_helpers.this_week_start(),
    "Custom Date..."
  }

  vim.ui.select(date_options, {
    prompt = "Select Date:"
  }, function(choice)
    if not choice then return end

    if choice:match("Custom Date") then
      vim.ui.input({
        prompt = "Enter date (YYYY-MM-DD): ",
        default = os.date("%Y-%m-%d")
      }, function(date)
        if date then
          callback(date)
        end
      end)
    else
      -- Extract date from choice
      local date = choice:match("([%d%-]+)$")
      callback(date)
    end
  end)
end

-- Quick source picker
function M.quick_source_picker(callback)
  -- First check if there's a SOURCES macro in the file
  local file_macros = M.parse_file_macros()
  local sources_list = M.common_variables.sources  -- Default

  if file_macros.SOURCES then
    -- Parse the SOURCES macro - expect one source per line
    sources_list = {}
    for line in file_macros.SOURCES:gmatch("[^\n]+") do
      local source = line:match("^%s*(.-)%s*$")  -- Trim whitespace
      if source and source ~= "" and not source:match("^%-%-") then  -- Skip empty lines and comments
        table.insert(sources_list, source)
      end
    end

    -- If no valid sources found, fall back to defaults
    if #sources_list == 0 then
      sources_list = M.common_variables.sources
    end
  end

  vim.ui.select(sources_list, {
    prompt = "Select Trade Source:"
  }, function(choice)
    if choice then
      callback(choice)
    end
  end)
end

-- Expand template to string (for nested template expansion)
function M.expand_template_to_string(template_def, callback)
  local template = template_def.template
  local var_defs = template_def.variables
  local values = {}
  local var_order = {}

  -- Collect variables in order
  for var_name, _ in pairs(var_defs) do
    table.insert(var_order, var_name)
  end
  table.sort(var_order)

  local var_index = 1

  local function collect_and_expand()
    if var_index > #var_order then
      -- All values collected, perform substitution
      local result = template:gsub("${([^}]+)}", function(var)
        return values[var] or "${" .. var .. "}"
      end)
      callback(result)
      return
    end

    local var_name = var_order[var_index]
    local var_def = var_defs[var_name]
    var_index = var_index + 1

    -- First check CONFIG for this variable
    local file_macros = M.parse_file_macros()
    if file_macros._CONFIG and file_macros._CONFIG[var_name] then
      values[var_name] = file_macros._CONFIG[var_name]
      collect_and_expand()
      return
    end

    -- Handle based on variable type
    if var_def.type == "choice" then
      -- Get choices from the specified source
      local choices = {}

      if var_def.source and file_macros[var_def.source] then
        -- Parse choices from macro
        for line in file_macros[var_def.source]:gmatch("[^\n]+") do
          local choice = line:match("^%s*(.-)%s*$")
          if choice and choice ~= "" and not choice:match("^%-%-") then
            table.insert(choices, choice)
          end
        end
      end

      if #choices > 0 then
        vim.ui.select(choices, {
          prompt = var_def.source or var_name .. ": "
        }, function(choice)
          if choice then
            values[var_name] = choice
            collect_and_expand()
          end
        end)
      else
        -- Fallback to text input if no choices found
        vim.ui.input({
          prompt = var_name .. ": ",
          default = var_def.default or ""
        }, function(value)
          if value ~= nil then
            values[var_name] = value
            collect_and_expand()
          end
        end)
      end

    elseif var_def.type == "date" then
      -- Handle date with optional format
      -- Format can be: date:Format Name:format_string
      -- e.g., date:DateTime:DateTime(YEAR, MONTH, DAY)
      -- or    date:ISO:YYYY-MM-DD
      local format_name = var_def.source
      local format_string = var_def.default

      M.quick_date_picker(function(date)
        -- Parse the selected date
        local year, month, day = date:match("(%d+)-(%d+)-(%d+)")

        if format_string and year then
          -- Apply custom format
          local formatted = format_string
          formatted = formatted:gsub("YYYY", year)
          formatted = formatted:gsub("YY", year:sub(3, 4))
          formatted = formatted:gsub("MM", string.format("%02d", month))
          formatted = formatted:gsub("M", tostring(tonumber(month)))
          formatted = formatted:gsub("DD", string.format("%02d", day))
          formatted = formatted:gsub("D", tostring(tonumber(day)))
          formatted = formatted:gsub("YEAR", year)
          formatted = formatted:gsub("MONTH", tostring(tonumber(month)))
          formatted = formatted:gsub("DAY", tostring(tonumber(day)))
          values[var_name] = formatted
        else
          -- Default to ISO format
          values[var_name] = date
        end
        collect_and_expand()
      end)

    elseif var_def.type == "config" then
      -- Get from CONFIG or use default
      local config_value = nil
      if file_macros._CONFIG then
        config_value = file_macros._CONFIG[var_def.source or var_name]
      end
      values[var_name] = config_value or var_def.default or ""
      collect_and_expand()

    else -- Default to "input" type
      vim.ui.input({
        prompt = var_def.source or var_name .. ": ",
        default = var_def.default or ""
      }, function(value)
        if value ~= nil then
          values[var_name] = value
          collect_and_expand()
        end
      end)
    end
  end

  -- Start collection
  collect_and_expand()
end

-- Expand functions in a string (used for macro expansion from file)
-- Can optionally provide test_values for headless operation
function M.expand_functions_in_string(text, callback, test_values)
  debug_log("expand_functions_in_string called with text length: " .. #text)

  -- Pass 1: Collect all function patterns and replace with placeholders
  local collection_result = M.collect_function_patterns(text)
  local functions_to_eval = collection_result.functions
  local expanded_text = collection_result.text

  debug_log("Pass 1 complete: Found " .. #functions_to_eval .. " functions")
  if #functions_to_eval > 0 then
    debug_log("Text with placeholders: " .. expanded_text:sub(1, 200))
  end

  -- If no functions found, just return the original text
  if #functions_to_eval == 0 then
    debug_log("No functions found, returning original text")
    callback(text)
    return
  end

  -- Pass 2: Resolve all functions (either from test_values or user input)
  debug_log("Pass 2: Starting function resolution")
  M.resolve_functions(functions_to_eval, test_values, function(resolved_values)
    if resolved_values == nil then
      -- User cancelled, don't insert anything
      debug_log("User cancelled, aborting expansion")
      callback(nil)
      return
    end
    local count = 0
    for _ in pairs(resolved_values) do count = count + 1 end
    debug_log("Pass 2 complete: Resolved " .. count .. " values")

    -- Pass 3: Replace all placeholders with resolved values
    debug_log("Pass 3: Substituting placeholders")
    local result = M.substitute_placeholders(expanded_text, functions_to_eval, resolved_values)
    debug_log("Pass 3 complete: Final result length: " .. #result)

    -- Check if the result has new @{...} patterns that need expansion (from MACRO expansion)
    if result:match("@{[^}]+}") then
      debug_log("Result contains new @{...} patterns, recursively expanding")
      -- Recursively expand any new patterns
      M.expand_functions_in_string(result, callback)
    else
      debug_log("Calling callback with result")
      callback(result)
    end
  end)
  debug_log("expand_functions_in_string setup complete, waiting for async resolution")
end

-- Pass 1: Collect all function patterns in text and replace with placeholders
function M.collect_function_patterns(text)
  debug_log("collect_function_patterns: Starting with text: " .. text:sub(1, 100))
  local functions_to_eval = {}
  local function_index = 1
  local expanded_text = text

  -- Check if we're in JSON context
  local is_json_context = text:match('\\\"') ~= nil or text:match("BODY%s*'")
  debug_log("JSON context detected: " .. tostring(is_json_context))

  -- NEW: Simple @{TYPE:prompt} syntax - much clearer!
  -- Handles @{INPUT:prompt}, @{PICKER:macro:prompt}, @{DATE:prompt}
  -- Pattern matches optional quotes before and after
  for match in text:gmatch('\\?"?@{[^}]+}\\?"?') do
    -- Parse the full match to extract components
    local prefix, func_type, args, suffix = match:match('^(\\?"?)@{([^:}]+):([^}]+)}(\\?"?)$')

    if func_type and args then
      local placeholder = "__FUNC_" .. function_index .. "__"
      local has_escaped_quotes = prefix:match('\\"') and suffix:match('\\"')
      local has_quotes = (prefix:match('"') and suffix:match('"')) and not has_escaped_quotes

      debug_log("Found @{} function: " .. func_type .. ":" .. args .. " -> " .. placeholder)
      debug_log("  Match: '" .. match .. "'")
      debug_log("  Quotes: prefix='" .. prefix .. "' suffix='" .. suffix .. "' escaped=" .. tostring(has_escaped_quotes))

      -- Parse the args (could be "prompt" or "macro:prompt" for PICKER)
      local arg_parts = {}
      for part in args:gmatch("[^:]+") do
        table.insert(arg_parts, part)
      end

      table.insert(functions_to_eval, {
        pattern = match,
        name = func_type,
        args = args,
        arg_list = arg_parts,
        placeholder = placeholder,
        has_escaped_quotes = has_escaped_quotes,
        has_quotes = has_quotes,
        is_json_context = is_json_context or has_escaped_quotes
      })

      -- Replace with placeholder, keeping quotes if they exist
      local replacement = has_escaped_quotes and ('\\"' .. placeholder .. '\\"') or
                         (has_quotes and ('"' .. placeholder .. '"')) or
                         placeholder

      local pattern_to_replace = match:gsub("([%(%)%.%[%]%+%-%*%?%^%$%%{}\\])", "%%%1")
      expanded_text = expanded_text:gsub(pattern_to_replace, replacement, 1)
      function_index = function_index + 1
    end
  end

  -- Find all function patterns and replace with placeholders
  -- First, find escaped quoted function calls like \"Input(...)\"
  for quoted_func in text:gmatch('\\"([%w_]+%b())\\"') do
    local func_name, args = quoted_func:match("([%w_]+)%((.*)%)")
    if func_name then
      local placeholder = "__FUNC_" .. function_index .. "__"
      debug_log("Found escaped quoted function: " .. func_name .. " -> " .. placeholder)
      table.insert(functions_to_eval, {
        pattern = '\\"' .. quoted_func .. '\\"',
        name = func_name,
        args = args,
        placeholder = placeholder,
        has_escaped_quotes = true,
        is_json_context = true
      })
      -- Important: Keep the escaped quotes in the replacement!
      local pattern_to_replace = ('\\"' .. quoted_func .. '\\"'):gsub("([%(%)%.%[%]%+%-%*%?%^%$%%])", "%%%1")
      expanded_text = expanded_text:gsub(pattern_to_replace, '\\"' .. placeholder .. '\\"', 1)
      function_index = function_index + 1
    end
  end

  -- Then regular quoted function calls like "Input(...)"
  for quoted_func in expanded_text:gmatch('"([%w_]+%b())"') do
    local func_name, args = quoted_func:match("([%w_]+)%((.*)%)")
    if func_name then
      local placeholder = "__FUNC_" .. function_index .. "__"
      debug_log("Found quoted function: " .. func_name .. " -> " .. placeholder)
      table.insert(functions_to_eval, {
        pattern = '"' .. quoted_func .. '"',
        name = func_name,
        args = args,
        placeholder = placeholder,
        has_quotes = true,
        is_json_context = is_json_context
      })
      expanded_text = expanded_text:gsub(('"' .. quoted_func .. '"'):gsub("([%(%)%.%[%]%+%-%*%?%^%$%%])", "%%%1"), placeholder, 1)
      function_index = function_index + 1
    end
  end

  -- Then unquoted function calls (but only specific known functions)
  local known_functions = {Input = true, Picker = true, DateTimePicker = true, DateTime = true, DatePicker = true}
  for func_call in expanded_text:gmatch("([%w_]+%b())") do
    local func_name, args = func_call:match("([%w_]+)%((.*)%)")
    if func_name and known_functions[func_name] then
      local placeholder = "__FUNC_" .. function_index .. "__"
      debug_log("Found unquoted function: " .. func_name .. " -> " .. placeholder)
      table.insert(functions_to_eval, {
        pattern = func_call,
        name = func_name,
        args = args,
        placeholder = placeholder,
        has_quotes = false,
        is_json_context = is_json_context
      })
      expanded_text = expanded_text:gsub(func_call:gsub("([%(%)%.%[%]%+%-%*%?%^%$%%])", "%%%1"), placeholder, 1)
      function_index = function_index + 1
    end
  end

  return {
    functions = functions_to_eval,
    text = expanded_text
  }
end

-- Pass 2: Resolve function values (from test_values or user input)
function M.resolve_functions(functions_to_eval, test_values, callback)
  debug_log("resolve_functions: Starting resolution for " .. #functions_to_eval .. " functions")
  local resolved_values = {}
  local func_idx = 1

  local function evaluate_next_func()
    if func_idx > #functions_to_eval then
      -- All functions resolved
      local count = 0
      for _ in pairs(resolved_values) do count = count + 1 end
      debug_log("All functions resolved, calling callback with " .. count .. " values")
      callback(resolved_values)
      return
    end

    local func_def = functions_to_eval[func_idx]
    debug_log("Resolving function " .. func_idx .. ": " .. func_def.name .. " (" .. func_def.placeholder .. ")")
    func_idx = func_idx + 1

    -- Check if we have a test value for this function
    if test_values and test_values[func_def.placeholder] then
      debug_log("Using test value for " .. func_def.placeholder)
      resolved_values[func_idx - 1] = test_values[func_def.placeholder]
      evaluate_next_func()
      return
    end

    -- Parse arguments
    local args = {}
    if func_def.args and func_def.args ~= "" then
      for arg in func_def.args:gmatch("([^,]+)") do
        arg = arg:match("^%s*(.-)%s*$")
        if arg:match("^['\"](.+)['\"]$") then
          arg = arg:match("^['\"](.+)['\"]$")
        end
        table.insert(args, arg)
      end
    end

    -- Handle different function types
    if func_def.name == "JINPUT" then
      -- JSON Input - always returns \"value\"
      local label = func_def.arg_list and func_def.arg_list[1] or "Value"
      local default = func_def.arg_list and func_def.arg_list[2] or ""
      debug_log("Prompting for JSON Input: '" .. label .. "'")
      vim.ui.input({
        prompt = label .. " (JSON): ",
        default = default
      }, function(value)
        if value ~= nil then
          -- Always escape for JSON
          resolved_values[func_idx - 1] = '\\"' .. value .. '\\"'
          evaluate_next_func()
        else
          callback(nil)
        end
      end)

    elseif func_def.name == "VAR" then
      -- Variable substitution - returns raw value from CONFIG or environment
      local var_name = func_def.arg_list and func_def.arg_list[1]
      if var_name then
        debug_log("Looking up variable: " .. var_name)

        -- First check CONFIG macro
        local file_macros = M.parse_file_macros()
        local value = nil

        if file_macros._CONFIG and file_macros._CONFIG[var_name] then
          value = file_macros._CONFIG[var_name]
          debug_log("  Found in CONFIG: " .. value)

          -- Special case: if value contains ${JWT_TOKEN}, resolve it
          if value and value:match("${JWT_TOKEN}") then
            local jwt_token = os.getenv("JWT_TOKEN") or vim.env.JWT_TOKEN
            if jwt_token then
              value = value:gsub("${JWT_TOKEN}", jwt_token)
              debug_log("  Resolved JWT_TOKEN: " .. value)
            end
          end
        else
          -- Check environment variable
          value = os.getenv(var_name)
          if value then
            debug_log("  Found in ENV: " .. value)
          end
        end

        resolved_values[func_idx - 1] = value or ("${" .. var_name .. "}")
        evaluate_next_func()
      else
        resolved_values[func_idx - 1] = "${VAR}"
        evaluate_next_func()
      end

    elseif func_def.name == "MACRO" then
      -- Macro expansion - returns the expanded macro content
      local macro_name = func_def.arg_list and func_def.arg_list[1]
      if macro_name then
        debug_log("Looking up macro: " .. macro_name)

        local file_macros = M.parse_file_macros()
        local macro_content = file_macros[macro_name]

        if macro_content and type(macro_content) == "string" then
          debug_log("  Found macro: " .. macro_name .. " (" .. #macro_content .. " chars)")
          -- The macro content itself might have more @{...} patterns that need expansion
          -- We'll just return it here and let the next pass handle any nested patterns
          resolved_values[func_idx - 1] = macro_content
        else
          debug_log("  Macro not found: " .. macro_name)
          resolved_values[func_idx - 1] = "@{MACRO:" .. macro_name .. "}"  -- Keep original if not found
        end
        evaluate_next_func()
      else
        resolved_values[func_idx - 1] = "@{MACRO}"
        evaluate_next_func()
      end

    elseif func_def.name == "JVAR" then
      -- Variable substitution for JSON - returns \"value\"
      local var_name = func_def.arg_list and func_def.arg_list[1]
      if var_name then
        debug_log("Looking up JSON variable: " .. var_name)

        -- First check CONFIG macro
        local file_macros = M.parse_file_macros()
        local value = nil

        if file_macros._CONFIG and file_macros._CONFIG[var_name] then
          value = file_macros._CONFIG[var_name]
          debug_log("  Found in CONFIG: " .. value)

          -- Special case: if value contains ${JWT_TOKEN}, resolve it
          if value and value:match("${JWT_TOKEN}") then
            local jwt_token = os.getenv("JWT_TOKEN") or vim.env.JWT_TOKEN
            if jwt_token then
              value = value:gsub("${JWT_TOKEN}", jwt_token)
              debug_log("  Resolved JWT_TOKEN: " .. value)
            end
          end
        else
          -- Check environment variable
          value = os.getenv(var_name)
          if value then
            debug_log("  Found in ENV: " .. value)
          end
        end

        if value then
          resolved_values[func_idx - 1] = '\\"' .. value .. '\\"'
        else
          resolved_values[func_idx - 1] = '\\"${' .. var_name .. '}\\"'
        end
        evaluate_next_func()
      else
        resolved_values[func_idx - 1] = '\\"${JVAR}\\"'
        evaluate_next_func()
      end

    elseif func_def.name == "INPUT" or func_def.name == "Input" then
      -- Regular input - returns raw value
      local label = func_def.arg_list and func_def.arg_list[1] or args[1] or "Value"
      local default = func_def.arg_list and func_def.arg_list[2] or args[2] or ""
      debug_log("Prompting for Input: '" .. label .. "'")
      vim.ui.input({
        prompt = label .. ": ",
        default = default
      }, function(value)
        if value ~= nil then
          -- Return raw value
          resolved_values[func_idx - 1] = value
          evaluate_next_func()
        else
          callback(nil)
        end
      end)

    elseif func_def.name == "JPICKER" then
      -- JSON Picker - always returns \"value\"
      local macro_name = func_def.arg_list and func_def.arg_list[1] or args[1]
      local label = func_def.arg_list and func_def.arg_list[2] or args[2] or macro_name or "Select"

      local file_macros = M.parse_file_macros()
      local choices = {}

      if macro_name and file_macros[macro_name] then
        for line in file_macros[macro_name]:gmatch("[^\n]+") do
          local choice = line:match("^%s*(.-)%s*$")
          if choice and choice ~= "" and not choice:match("^%-%-") then
            table.insert(choices, choice)
          end
        end
      end

      if #choices > 0 then
        vim.ui.select(choices, {
          prompt = label .. " (JSON): "
        }, function(choice)
          if choice then
            -- Always escape for JSON
            resolved_values[func_idx - 1] = '\\"' .. choice .. '\\"'
            evaluate_next_func()
          else
            callback(nil)
          end
        end)
      else
        vim.ui.input({
          prompt = label .. " (JSON): ",
          default = ""
        }, function(value)
          if value ~= nil then
            -- Always escape for JSON
            resolved_values[func_idx - 1] = '\\"' .. value .. '\\"'
            evaluate_next_func()
          else
            callback(nil)
          end
        end)
      end

    elseif func_def.name == "PICKER" or func_def.name == "Picker" then
      -- For @{PICKER:macro:prompt} syntax, arg_list[1] is macro, arg_list[2] is prompt
      local macro_name = func_def.arg_list and func_def.arg_list[1] or args[1]
      local label = func_def.arg_list and func_def.arg_list[2] or args[2] or macro_name or "Select"

      local file_macros = M.parse_file_macros()
      local choices = {}

      if macro_name and file_macros[macro_name] then
        for line in file_macros[macro_name]:gmatch("[^\n]+") do
          local choice = line:match("^%s*(.-)%s*$")
          if choice and choice ~= "" and not choice:match("^%-%-") then
            table.insert(choices, choice)
          end
        end
      end

      if #choices > 0 then
        vim.ui.select(choices, {
          prompt = label .. ": "
        }, function(choice)
          if choice then
            -- Return raw value - let the user be explicit with JPICKER if they need JSON escaping
            resolved_values[func_idx - 1] = choice
            evaluate_next_func()
          else
            callback(nil)
          end
        end)
      else
        vim.ui.input({
          prompt = label .. ": ",
          default = ""
        }, function(value)
          if value ~= nil then
            local formatted_value
            if func_def.has_escaped_quotes then
              -- Escaped quotes are already in the placeholder, just return the value
              formatted_value = value
            elseif func_def.has_quotes then
              if func_def.is_json_context then
                formatted_value = '\\"' .. value .. '\\"'
              else
                formatted_value = '"' .. value .. '"'
              end
            else
              formatted_value = value
            end
            resolved_values[func_idx - 1] = formatted_value
            evaluate_next_func()
          else
            -- User cancelled
            callback(nil)
          end
        end)
      end

    elseif func_def.name == "DATE" or func_def.name == "DateTimePicker" or func_def.name == "DateTime" then
      -- For @{DATE:prompt} syntax
      local label = func_def.arg_list and func_def.arg_list[1] or "Date"
      debug_log("Prompting for Date: '" .. label .. "'")
      M.quick_date_picker(function(date)
        local year, month, day = date:match("(%d+)-(%d+)-(%d+)")
        if year then
          resolved_values[func_idx - 1] = string.format("DateTime(%s, %s, %s)", year, tonumber(month), tonumber(day))
        else
          resolved_values[func_idx - 1] = date
        end
        evaluate_next_func()
      end)

    else
      -- Unknown function, keep as-is
      resolved_values[func_idx - 1] = func_def.pattern
      evaluate_next_func()
    end
  end

  evaluate_next_func()
end

-- Pass 3: Substitute placeholders with resolved values
function M.substitute_placeholders(text_with_placeholders, functions, resolved_values)
  local result = text_with_placeholders

  for i, func_def in ipairs(functions) do
    local replacement = resolved_values[i] or func_def.pattern

    debug_log("Substituting " .. func_def.placeholder .. " with: '" .. tostring(replacement) .. "'")
    debug_log("  In text: " .. (text_with_placeholders:match("(.{0,50}" .. func_def.placeholder .. ".{0,50})") or "not found"))

    -- For gsub, we need to escape % but NOT \ (backslash should pass through)
    local safe_replacement = replacement:gsub("%%", "%%%%")
    result = result:gsub(func_def.placeholder, safe_replacement)

    debug_log("  After substitution: " .. (result:match("(.{0,100}StartsWith.{0,50})") or "not found"))
  end

  debug_log("Final substitution result preview: " .. result:sub(1, 200))
  return result
end

-- Headless test function - expand a macro with test values
function M.test_macro_expansion(macro_text, test_values)
  -- Pass 1: Collect function patterns
  local collection_result = M.collect_function_patterns(macro_text)

  -- If no functions, return original
  if #collection_result.functions == 0 then
    return macro_text
  end

  -- Create test values map if needed
  local test_map = {}
  if test_values then
    -- Convert array of values to placeholder map
    for i, func_def in ipairs(collection_result.functions) do
      if test_values[i] then
        test_map[func_def.placeholder] = test_values[i]
      elseif test_values[func_def.name] then
        -- Also support mapping by function name
        test_map[func_def.placeholder] = test_values[func_def.name]
      end
    end
  end

  -- Pass 2: Use test values directly (no user interaction)
  local resolved_values = {}
  for i, func_def in ipairs(collection_result.functions) do
    if test_map[func_def.placeholder] then
      resolved_values[i] = test_map[func_def.placeholder]
    else
      -- Generate default test value based on function type
      if func_def.name == "Input" then
        resolved_values[i] = func_def.has_escaped_quotes and '\\"TEST_INPUT\\"' or
                           (func_def.has_quotes and '"TEST_INPUT"' or "TEST_INPUT")
      elseif func_def.name == "Picker" then
        resolved_values[i] = func_def.has_escaped_quotes and '\\"TEST_CHOICE\\"' or
                           (func_def.has_quotes and '"TEST_CHOICE"' or "TEST_CHOICE")
      elseif func_def.name == "DateTimePicker" or func_def.name == "DateTime" then
        resolved_values[i] = "DateTime(2025, 1, 1)"
      else
        resolved_values[i] = func_def.pattern
      end
    end
  end

  -- Pass 3: Substitute and return
  return M.substitute_placeholders(collection_result.text, collection_result.functions, resolved_values)
end

-- Debug function to show what functions would be resolved
function M.debug_show_functions(macro_text)
  local collection_result = M.collect_function_patterns(macro_text)

  if #collection_result.functions == 0 then
    print("No functions found in macro")
    return
  end

  print("Functions found in macro:")
  for i, func_def in ipairs(collection_result.functions) do
    print(string.format("%d. %s: %s -> %s",
      i,
      func_def.placeholder,
      func_def.pattern,
      func_def.name .. "(" .. (func_def.args or "") .. ")"
    ))
  end

  print("\nText with placeholders:")
  print(collection_result.text)
end

-- Expand simple macro (without function patterns)
function M.expand_simple_macro(template, row, start_pos, end_pos, is_json_context)
  -- Find function patterns in the template
  local functions_to_eval = {}
  local function_index = 1
  local expanded_template = template

  -- Check if we're in a JSON body context
  if is_json_context == nil then
    -- Check if we're inside a BODY block by looking at surrounding context
    local lines = vim.api.nvim_buf_get_lines(0, math.max(0, row - 10), math.min(vim.api.nvim_buf_line_count(0), row + 10), false)
    local context_text = table.concat(lines, "\n")

    -- We're in JSON context if we're between BODY and FORMAT or if escaped quotes are present
    is_json_context = (context_text:match("BODY%s*'") and context_text:match("FORMAT%s+JSON")) or template:match('\\\"') ~= nil
  end

  -- Pattern to match function calls like SourcePicker(), DateTimePicker(label), Input(label, default)
  -- Also check if they're wrapped in quotes like "Picker(DEAL_TYPES)" or escaped quotes \"Picker(DEAL_TYPES)\"

  -- First, find escaped quoted function calls like \"Picker(DEAL_TYPES)\"
  for quoted_func in template:gmatch('\\"([%w_]+%b())\\"') do
    local func_name, args = quoted_func:match("([%w_]+)%((.*)%)")
    table.insert(functions_to_eval, {
      pattern = '\\"' .. quoted_func .. '\\"',
      func_pattern = quoted_func,
      name = func_name,
      args = args,
      placeholder = "__FUNC_" .. function_index .. "__",
      is_json_context = true, -- Escaped quotes mean JSON context
      has_escaped_quotes = true
    })
    -- Replace with placeholder
    local pattern_to_replace = ('\\"' .. quoted_func .. '\\"'):gsub("([%(%)%.%[%]%+%-%*%?%^%$%%])", "%%%1")
    expanded_template = expanded_template:gsub(pattern_to_replace, "__FUNC_" .. function_index .. "__", 1)
    function_index = function_index + 1
  end

  -- Then find regular quoted function calls like "Picker(DEAL_TYPES)"
  for quoted_func in template:gmatch('"([%w_]+%b())"') do
    local func_name, args = quoted_func:match("([%w_]+)%((.*)%)")
    table.insert(functions_to_eval, {
      pattern = '"' .. quoted_func .. '"',
      func_pattern = quoted_func,
      name = func_name,
      args = args,
      placeholder = "__FUNC_" .. function_index .. "__",
      is_json_context = is_json_context,
      has_quotes = true
    })
    -- Replace with placeholder
    local pattern_to_replace = ('"' .. quoted_func .. '"'):gsub("([%(%)%.%[%]%+%-%*%?%^%$%%])", "%%%1")
    expanded_template = expanded_template:gsub(pattern_to_replace, "__FUNC_" .. function_index .. "__", 1)
    function_index = function_index + 1
  end

  -- Then find unquoted function calls
  for func_call in expanded_template:gmatch("([%w_]+%b())") do
    local func_name, args = func_call:match("([%w_]+)%((.*)%)")
    table.insert(functions_to_eval, {
      pattern = func_call,
      func_pattern = func_call,
      name = func_name,
      args = args,
      placeholder = "__FUNC_" .. function_index .. "__",
      is_json_context = is_json_context,
      has_quotes = false
    })
    -- Replace with placeholder
    expanded_template = expanded_template:gsub(func_call:gsub("([%(%)%.])", "%%%1"), "__FUNC_" .. function_index .. "__", 1)
    function_index = function_index + 1
  end

  -- If no functions found, just replace inline
  if #functions_to_eval == 0 then
    M.replace_macro_inline(row, start_pos, end_pos, template)
    return
  end

  -- Evaluate functions
  local func_values = {}
  local func_idx = 1

  local function evaluate_next()
    if func_idx > #functions_to_eval then
      -- All functions evaluated, substitute back
      local result = expanded_template
      for i, func_def in ipairs(functions_to_eval) do
        local replacement = func_values[i] or func_def.pattern
        -- Escape special characters in replacement string
        replacement = replacement:gsub("%%", "%%%%")
        result = result:gsub(func_def.placeholder, replacement)
      end
      M.replace_macro_inline(row, start_pos, end_pos, result)
      return
    end

    local func_def = functions_to_eval[func_idx]
    func_idx = func_idx + 1

    -- Parse arguments (simple CSV parsing)
    local args = {}
    if func_def.args and func_def.args ~= "" then
      for arg in func_def.args:gmatch("([^,]+)") do
        -- Trim whitespace and quotes
        arg = arg:match("^%s*(.-)%s*$")
        if arg:match("^['\"](.+)['\"]$") then
          arg = arg:match("^['\"](.+)['\"]$")
        end
        table.insert(args, arg)
      end
    end

    -- Handle different function types
    if func_def.name == "Picker" then
      -- Generic picker that uses any macro as source
      local macro_name = args[1]
      local label = args[2] or macro_name or "Select"

      local file_macros = M.parse_file_macros()
      local choices = {}

      if macro_name and file_macros[macro_name] then
        -- Parse choices from the specified macro
        for line in file_macros[macro_name]:gmatch("[^\n]+") do
          local choice = line:match("^%s*(.-)%s*$")
          if choice and choice ~= "" and not choice:match("^%-%-") then
            table.insert(choices, choice)
          end
        end
      end

      if #choices > 0 then
        vim.ui.select(choices, {
          prompt = label .. ": "
        }, function(choice)
          if choice then
            -- Handle quotes based on context
            if func_def.has_escaped_quotes then
              -- Template had escaped quotes, keep them escaped
              func_values[func_idx - 1] = '\\"' .. choice .. '\\"'
            elseif func_def.has_quotes then
              -- Template had regular quotes, add them back (escaped for JSON if needed)
              if func_def.is_json_context then
                func_values[func_idx - 1] = '\\"' .. choice .. '\\"'
              else
                func_values[func_idx - 1] = '"' .. choice .. '"'
              end
            else
              -- No quotes in template, return raw value
              func_values[func_idx - 1] = choice
            end
            evaluate_next()
          end
        end)
      else
        -- Fallback to text input if macro not found
        vim.ui.input({
          prompt = label .. ": ",
          default = ""
        }, function(value)
          if value ~= nil then
            -- Handle quotes based on context
            if func_def.has_escaped_quotes then
              -- Template had escaped quotes, keep them escaped
              func_values[func_idx - 1] = '\\"' .. value .. '\\"'
            elseif func_def.has_quotes then
              -- Template had regular quotes, add them back (escaped for JSON if needed)
              if func_def.is_json_context then
                func_values[func_idx - 1] = '\\"' .. value .. '\\"'
              else
                func_values[func_idx - 1] = '"' .. value .. '"'
              end
            else
              -- No quotes in template, return raw value
              func_values[func_idx - 1] = value
            end
            evaluate_next()
          end
        end)
      end

    elseif func_def.name == "SourcePicker" then
      -- Kept for backward compatibility - just delegates to Picker
      local label = args[1] or "Source"
      M.quick_source_picker(function(source)
        func_values[func_idx - 1] = source
        evaluate_next()
      end)

    elseif func_def.name == "DateTimePicker" then
      local label = args[1] or "Date"
      M.quick_date_picker(function(date)
        local year, month, day = date:match("(%d+)-(%d+)-(%d+)")
        if year then
          func_values[func_idx - 1] = string.format("DateTime(%s, %s, %s)", year, tonumber(month), tonumber(day))
        else
          func_values[func_idx - 1] = date
        end
        evaluate_next()
      end)

    elseif func_def.name == "DatePicker" then
      local label = args[1] or "Date"
      M.quick_date_picker(function(date)
        func_values[func_idx - 1] = date
        evaluate_next()
      end)

    elseif func_def.name == "Input" then
      local label = args[1] or "Value"
      local default = args[2] or ""
      vim.ui.input({
        prompt = label .. ": ",
        default = default
      }, function(value)
        if value ~= nil then
          -- Handle quotes based on context (same as Picker)
          if func_def.has_escaped_quotes then
            -- Template had escaped quotes around Input(), keep them escaped
            func_values[func_idx - 1] = '\\"' .. value .. '\\"'
          elseif func_def.has_quotes then
            -- Template had regular quotes, add them back (escaped for JSON if needed)
            if func_def.is_json_context then
              func_values[func_idx - 1] = '\\"' .. value .. '\\"'
            else
              func_values[func_idx - 1] = '"' .. value .. '"'
            end
          else
            -- No quotes in template, return raw value
            func_values[func_idx - 1] = value
          end
          evaluate_next()
        end
      end)

    elseif func_def.name == "ConfigValue" then
      local key = args[1]
      local default = args[2] or ""
      local file_macros = M.parse_file_macros()
      local value = nil
      if file_macros._CONFIG and key then
        value = file_macros._CONFIG[key]
      end
      func_values[func_idx - 1] = value or default
      evaluate_next()

    else
      -- Unknown function, keep as-is
      func_values[func_idx - 1] = func_def.pattern
      evaluate_next()
    end
  end

  evaluate_next()
end

-- Generic template-based macro expander
function M.expand_template_macro(template_def, row, start_pos, end_pos)
  local template = template_def.template
  local var_defs = template_def.variables
  local values = {}
  local var_order = {}

  -- Collect variables in order
  for var_name, _ in pairs(var_defs) do
    table.insert(var_order, var_name)
  end
  table.sort(var_order)

  local var_index = 1

  local function collect_and_expand()
    if var_index > #var_order then
      -- All values collected, perform substitution
      local result = template:gsub("${([^}]+)}", function(var)
        return values[var] or "${" .. var .. "}"
      end)
      M.replace_macro_inline(row, start_pos, end_pos, result)
      return
    end

    local var_name = var_order[var_index]
    local var_def = var_defs[var_name]
    var_index = var_index + 1

    -- First check CONFIG for this variable
    local file_macros = M.parse_file_macros()
    if file_macros._CONFIG and file_macros._CONFIG[var_name] then
      values[var_name] = file_macros._CONFIG[var_name]
      collect_and_expand()
      return
    end

    -- Handle based on variable type
    if var_def.type == "choice" then
      -- Get choices from the specified source
      local choices = {}

      if var_def.source and file_macros[var_def.source] then
        -- Parse choices from macro
        for line in file_macros[var_def.source]:gmatch("[^\n]+") do
          local choice = line:match("^%s*(.-)%s*$")
          if choice and choice ~= "" and not choice:match("^%-%-") then
            table.insert(choices, choice)
          end
        end
      end

      if #choices > 0 then
        vim.ui.select(choices, {
          prompt = var_def.source or var_name .. ": "
        }, function(choice)
          if choice then
            values[var_name] = choice
            collect_and_expand()
          end
        end)
      else
        -- Fallback to text input if no choices found
        vim.ui.input({
          prompt = var_name .. ": ",
          default = var_def.default or ""
        }, function(value)
          if value ~= nil then
            values[var_name] = value
            collect_and_expand()
          end
        end)
      end

    elseif var_def.type == "date" then
      -- Handle date with optional format
      -- Format can be: date:Format Name:format_string
      -- e.g., date:DateTime:DateTime(YEAR, MONTH, DAY)
      -- or    date:ISO:YYYY-MM-DD
      local format_name = var_def.source
      local format_string = var_def.default

      M.quick_date_picker(function(date)
        -- Parse the selected date
        local year, month, day = date:match("(%d+)-(%d+)-(%d+)")

        if format_string and year then
          -- Apply custom format
          local formatted = format_string
          formatted = formatted:gsub("YYYY", year)
          formatted = formatted:gsub("YY", year:sub(3, 4))
          formatted = formatted:gsub("MM", string.format("%02d", month))
          formatted = formatted:gsub("M", tostring(tonumber(month)))
          formatted = formatted:gsub("DD", string.format("%02d", day))
          formatted = formatted:gsub("D", tostring(tonumber(day)))
          formatted = formatted:gsub("YEAR", year)
          formatted = formatted:gsub("MONTH", tostring(tonumber(month)))
          formatted = formatted:gsub("DAY", tostring(tonumber(day)))
          values[var_name] = formatted
        else
          -- Default to ISO format
          values[var_name] = date
        end
        collect_and_expand()
      end)

    elseif var_def.type == "config" then
      -- Get from CONFIG or use default
      local config_value = nil
      if file_macros._CONFIG then
        config_value = file_macros._CONFIG[var_def.source or var_name]
      end
      values[var_name] = config_value or var_def.default or ""
      collect_and_expand()

    else -- Default to "input" type
      vim.ui.input({
        prompt = var_def.source or var_name .. ": ",
        default = var_def.default or ""
      }, function(value)
        if value ~= nil then
          values[var_name] = value
          collect_and_expand()
        end
      end)
    end
  end

  -- Start collection
  collect_and_expand()
end

-- WHERE clause builder (kept for backward compatibility)
function M.where_clause_builder(callback)
  local options = {
    "Source and Date",
    "Date Range",
    "Source Only",
    "Today's Trades",
    "Custom WHERE"
  }

  vim.ui.select(options, {
    prompt = "Select WHERE clause type:"
  }, function(choice)
    if not choice then return end

    if choice == "Source and Date" then
      M.quick_source_picker(function(source)
        M.quick_date_picker(function(date)
          local year, month, day = date:match("(%d+)-(%d+)-(%d+)")
          local where = string.format('Source = \\"%s\\" and TradeDate = TradeDate(%s, %s, %s)',
            source, year, month, day)
          callback(where)
        end)
      end)
    elseif choice == "Date Range" then
      vim.ui.input({
        prompt = "Start date (YYYY-MM-DD): ",
        default = M.common_variables.date_helpers.this_week_start()
      }, function(start_date)
        if not start_date then return end
        vim.ui.input({
          prompt = "End date (YYYY-MM-DD): ",
          default = M.common_variables.date_helpers.today()
        }, function(end_date)
          if not end_date then return end
          local sy, sm, sd = start_date:match("(%d+)-(%d+)-(%d+)")
          local ey, em, ed = end_date:match("(%d+)-(%d+)-(%d+)")
          local where = string.format('TradeDate >= TradeDate(%s, %s, %s) and TradeDate <= TradeDate(%s, %s, %s)',
            sy, sm, sd, ey, em, ed)
          callback(where)
        end)
      end)
    elseif choice == "Source Only" then
      M.quick_source_picker(function(source)
        local where = string.format('Source = \\"%s\\"', source)
        callback(where)
      end)
    elseif choice == "Today's Trades" then
      local today = M.common_variables.date_helpers.today()
      local year, month, day = today:match("(%d+)-(%d+)-(%d+)")
      local where = string.format('TradeDate = TradeDate(%s, %s, %s)', year, month, day)
      callback(where)
    elseif choice == "Custom WHERE" then
      vim.ui.input({
        prompt = "Enter WHERE clause: ",
        default = 'Source = \\"Bloomberg_FIX_FX\\"'
      }, function(where)
        if where then
          callback(where)
        end
      end)
    end
  end)
end

-- Insert text at cursor position
function M.insert_at_cursor(text)
  local row, col = unpack(vim.api.nvim_win_get_cursor(0))
  local lines = vim.split(text, "\n")

  if #lines == 1 then
    -- Single line insertion
    local line = vim.api.nvim_get_current_line()
    local new_line = line:sub(1, col) .. text .. line:sub(col + 1)
    vim.api.nvim_set_current_line(new_line)
    vim.api.nvim_win_set_cursor(0, {row, col + #text})
  else
    -- Multi-line insertion
    vim.api.nvim_put(lines, 'c', true, true)
  end
end

-- Save current query as template
function M.save_as_template()
  -- Get current query (either selection or query at cursor)
  local query = M.get_current_query()
  if not query then
    vim.notify("No query found at cursor", vim.log.levels.WARN)
    return
  end

  vim.ui.input({
    prompt = "Template name: "
  }, function(name)
    if not name or name == "" then return end

    -- Extract variables from query (look for ${VAR} patterns)
    local variables = {}
    for var in query:gmatch("${([^}:]+)") do
      if not variables[var] then
        variables[var] = { prompt = var, default = "" }
      end
    end

    -- Save template
    M.templates[name] = {
      name = name,
      template = query,
      variables = variables
    }

    vim.notify("Template saved: " .. name, vim.log.levels.INFO)
  end)
end

-- Get current query (from selection or at cursor)
function M.get_current_query()
  local mode = vim.api.nvim_get_mode().mode

  if mode == 'v' or mode == 'V' then
    -- Visual mode - get selection
    local start_pos = vim.fn.getpos("'<")
    local end_pos = vim.fn.getpos("'>")
    local lines = vim.api.nvim_buf_get_lines(0, start_pos[2] - 1, end_pos[2], false)
    return table.concat(lines, "\n")
  else
    -- Normal mode - get query at cursor
    local navigation = require('sql-cli.navigation')
    return navigation.get_query_at_cursor()
  end
end

-- Quick variable substitution for current query (in-place expansion)
function M.quick_substitute()
  local query = M.get_current_query()
  if not query then
    vim.notify("No query found at cursor", vim.log.levels.WARN)
    return
  end

  -- Find all variables in the query
  local variables = {}
  local var_order = {}

  for var, default in query:gmatch("${([^}:]+):?([^}]*)}") do
    if not variables[var] then
      variables[var] = default or ""
      table.insert(var_order, var)
    end
  end

  if #var_order == 0 then
    vim.notify("No template variables found in query", vim.log.levels.INFO)
    return
  end

  -- Collect values for each variable
  local values = {}
  local var_index = 1

  local function get_next_value()
    if var_index > #var_order then
      -- All values collected, perform substitution
      local result = M.substitute_variables(query, values)

      -- Replace the current query with substituted version
      M.replace_current_query(result)
      return
    end

    local var = var_order[var_index]
    local default = variables[var]
    var_index = var_index + 1

    -- Special handling for common variables with smart defaults
    if var == "SOURCE" or var:match("^SOURCE") then
      M.quick_source_picker(function(value)
        values[var] = value
        get_next_value()
      end)
    elseif var == "BASE_URL" then
      -- Check if CONFIG macro defines BASE_URL
      local file_macros = M.parse_file_macros()
      local default_url = "http://localhost:5001"
      if file_macros._CONFIG and file_macros._CONFIG.BASE_URL then
        default_url = file_macros._CONFIG.BASE_URL
      end
      vim.ui.input({
        prompt = "API Base URL: ",
        default = default_url
      }, function(value)
        if value ~= nil then
          values[var] = value
          get_next_value()
        end
      end)
    elseif var == "TRADE_DATE" or var:match("^DATE") then
      M.quick_date_picker(function(date)
        -- Auto-convert to TradeDate() function format
        local year, month, day = date:match("(%d+)-(%d+)-(%d+)")
        if year then
          values[var] = string.format("TradeDate(%s, %s, %s)", year, month, day)
        else
          values[var] = date
        end
        get_next_value()
      end)
    elseif var == "YEAR" then
      vim.ui.input({
        prompt = "Year: ",
        default = os.date("%Y")
      }, function(value)
        if value ~= nil then
          values[var] = value
          get_next_value()
        end
      end)
    elseif var == "MONTH" then
      vim.ui.input({
        prompt = "Month: ",
        default = os.date("%m")
      }, function(value)
        if value ~= nil then
          values[var] = value
          get_next_value()
        end
      end)
    elseif var == "DAY" then
      vim.ui.input({
        prompt = "Day: ",
        default = os.date("%d")
      }, function(value)
        if value ~= nil then
          values[var] = value
          get_next_value()
        end
      end)
    elseif var:match("WHERE") then
      M.where_clause_builder(function(where_clause)
        values[var] = where_clause
        get_next_value()
      end)
    else
      vim.ui.input({
        prompt = var .. ": ",
        default = default
      }, function(value)
        if value ~= nil then
          values[var] = value
          get_next_value()
        end
      end)
    end
  end

  get_next_value()
end

-- Parse macro definitions from comment blocks in the file
function M.parse_file_macros()
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  local macros = {}
  local in_macro_block = false
  local current_macro_name = nil
  local current_macro_lines = {}

  for _, line in ipairs(lines) do
    -- Check for macro definition start: -- MACRO: NAME
    local macro_start = line:match("^%s*%-%-+%s*MACRO:%s*([%w_]+)")
    if macro_start then
      -- Save previous macro if exists
      if current_macro_name then
        macros[current_macro_name] = table.concat(current_macro_lines, "\n")
      end
      -- Start new macro
      in_macro_block = true
      current_macro_name = macro_start
      current_macro_lines = {}
    -- Check for macro end: -- END MACRO
    elseif line:match("^%s*%-%-+%s*END%s+MACRO") then
      if current_macro_name then
        macros[current_macro_name] = table.concat(current_macro_lines, "\n")
      end
      in_macro_block = false
      current_macro_name = nil
      current_macro_lines = {}
    -- Collect macro lines (skip comment markers)
    elseif in_macro_block then
      -- Remove leading comment markers but preserve indentation
      local content = line:gsub("^%s*%-%-+%s?", "")
      table.insert(current_macro_lines, content)
    end
  end

  -- Save last macro if not closed
  if current_macro_name then
    macros[current_macro_name] = table.concat(current_macro_lines, "\n")
  end

  -- Parse CONFIG macro if present for key-value settings
  if macros.CONFIG then
    local config = {}
    for line in macros.CONFIG:gmatch("[^\n]+") do
      local key, value = line:match("^%s*([%w_]+)%s*=%s*(.-)%s*$")
      if key and value then
        -- Resolve any ${ENV_VAR} references in the value
        local resolved_value = value:gsub("${([^}]+)}", function(env_var_name)
          -- Check if this is the token variable and use fresh token
          if env_var_name == token_manager.config.token_var_name then
            local token = token_manager.get_token()
            if token then
              return token
            end
          end

          -- Otherwise check environment
          local env_value = vim.env[env_var_name] or os.getenv(env_var_name)
          if env_value then
            return env_value
          else
            -- Keep the original ${ENV_VAR} if not found
            return "${" .. env_var_name .. "}"
          end
        end)
        config[key] = resolved_value
      end
    end
    macros._CONFIG = config  -- Store parsed config separately
  end

  -- Parse macros that have TEMPLATE and VARIABLES structure
  for macro_name, macro_content in pairs(macros) do
    -- Skip special macros and non-string content
    if macro_name ~= "CONFIG" and
       macro_name ~= "_CONFIG" and
       not macro_name:match("^_TEMPLATE_") and
       type(macro_content) == "string" then
      local lines = vim.split(macro_content, "\n")
      local template = nil
      local variables = {}
      local in_variables = false

      for _, line in ipairs(lines) do
        if line:match("^TEMPLATE:") then
          template = line:match("^TEMPLATE:%s*(.+)$")
        elseif line:match("^VARIABLES:") then
          in_variables = true
        elseif in_variables and line:match("^%s+(%w+):") then
          -- Parse variable definition: VAR_NAME: type:source:default
          local var_name, rest = line:match("^%s+([%w_]+):%s*(.+)$")
          if var_name and rest then
            -- Split only on first two colons to preserve format strings
            -- Trim whitespace from rest first
            rest = rest:match("^%s*(.-)%s*$")

            local first_colon = rest:find(":")
            local var_type = "input"
            local var_source = nil
            local var_default = nil

            if first_colon then
              var_type = rest:sub(1, first_colon - 1):match("^%s*(.-)%s*$")
              local remaining = rest:sub(first_colon + 1)
              local second_colon = remaining:find(":")

              if second_colon then
                var_source = remaining:sub(1, second_colon - 1):match("^%s*(.-)%s*$")
                var_default = remaining:sub(second_colon + 1):match("^%s*(.-)%s*$")
              else
                var_source = remaining:match("^%s*(.-)%s*$")
              end
            else
              var_type = rest
            end

            variables[var_name] = {
              type = var_type,
              source = var_source,
              default = var_default
            }
          end
        end
      end

      -- If this macro has a template structure, store it specially
      if template and next(variables) then
        -- Debug output
        -- vim.notify("Template macro " .. macro_name .. ": " .. vim.inspect(variables), vim.log.levels.INFO)
        macros["_TEMPLATE_" .. macro_name] = {
          template = template,
          variables = variables
        }
      end
    end
  end

  return macros
end

-- Expand all macros from file definitions
function M.expand_file_macros()
  local macros = M.parse_file_macros()

  if vim.tbl_isempty(macros) then
    vim.notify("No macro definitions found in file. Use -- MACRO: NAME ... -- END MACRO", vim.log.levels.INFO)
    return
  end

  -- Show available macros
  local macro_names = vim.tbl_keys(macros)
  table.sort(macro_names)

  vim.ui.select(macro_names, {
    prompt = "Select macro to expand:",
    format_item = function(name)
      local preview = macros[name]:gsub("\n.*", "...")
      if #preview > 50 then
        preview = preview:sub(1, 47) .. "..."
      end
      return string.format("%s: %s", name, preview)
    end
  }, function(choice)
    if choice then
      local expansion = macros[choice]

      -- Check if macro has function patterns (like Input(), Picker(), etc.)
      if expansion:match("[%w_]+%b()") then
        -- Expand functions in string and insert the result
        M.expand_functions_in_string(expansion, function(result)
          if result then
            -- Debug output
            -- vim.notify("Expanded result: " .. vim.inspect(result), vim.log.levels.INFO)
            M.insert_at_cursor(result)
          else
            vim.notify("Macro expansion cancelled", vim.log.levels.INFO)
          end
        end)
      -- Check if macro has variables
      elseif expansion:match("${[^}]+}") then
        -- Store the macro content temporarily
        local temp_query = expansion
        -- Use quick_substitute on the macro content
        M.replace_current_query = function(result)
          M.insert_at_cursor(result)
        end
        M.get_current_query = function()
          return temp_query
        end
        M.quick_substitute()
      else
        -- No variables or functions, just insert
        M.insert_at_cursor(expansion)
      end
    end
  end)
end

-- Inline macro expansion - expand ${MACRO:...} definitions in-place
function M.expand_inline_macro()
  local line = vim.api.nvim_get_current_line()
  local row, col = unpack(vim.api.nvim_win_get_cursor(0))

  -- Look for macro usage like ${TRADE_FETCH} or @{MACRO_NAME}
  local patterns = {
    { pattern = "${([^}:]+):?([^}]*)}", prefix = "${", suffix = "}" },
    { pattern = "@{([^}:]+):?([^}]*)}", prefix = "@{", suffix = "}" },
    { pattern = "@([%w_]+)", prefix = "@", suffix = "" }  -- Fixed: added underscore to pattern
  }

  local start_pos, end_pos, macro_name, macro_args

  -- Find the macro at or near cursor position
  for _, p in ipairs(patterns) do
    -- Search for all occurrences of the pattern in the line
    local search_start = 1
    while true do
      local s, e, name, args = line:find(p.pattern, search_start)
      if not s then break end

      -- Check if cursor is within or near this match (col is 0-based, positions are 1-based)
      if col + 1 >= s and col + 1 <= e then
        start_pos = s
        end_pos = e
        macro_name = name
        macro_args = args
        break
      end

      search_start = e + 1
    end

    if start_pos then break end
  end

  if not start_pos then
    vim.notify("No macro found at cursor position. Use ${MACRO}, @{MACRO} or @MACRO", vim.log.levels.INFO)
    return
  end

  -- First check file-defined macros
  local file_macros = M.parse_file_macros()

  -- Then check built-in macros
  local inline_macros = {
    TRADE_FETCH = [[
WITH WEB trades AS (
    URL 'http://localhost:5001/trades'
    METHOD POST
    BODY '{
        "select": "*",
        "where": "${WHERE}"
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)]],
    WHERE_SOURCE = 'Source = \\"${SOURCE}\\"',
    WHERE_DATE = 'TradeDate = TradeDate(${YEAR}, ${MONTH}, ${DAY})',
    WHERE_TODAY = function()
      local today = os.date("%Y-%m-%d")
      local y, m, d = today:match("(%d+)-(%d+)-(%d+)")
      return string.format('TradeDate = TradeDate(%s, %s, %s)', y, m, d)
    end,
    COLUMNS_BASIC = 'TradeId, Source, Symbol, Quantity, Price, TradeDate',
    COLUMNS_FULL = 'TradeId, Source, Symbol, Quantity, Price, TradeDate, Status, Account, Currency, ExecutionTime, Broker, Commission, SettlementDate'
  }

  -- Look for the macro (file macros take precedence)
  local expansion = file_macros[macro_name] or inline_macros[macro_name]
  local template_def = file_macros["_TEMPLATE_" .. macro_name]

  debug_log("expand_inline_macro: Found macro '" .. macro_name .. "', expansion type: " .. type(expansion))
  if expansion and type(expansion) == "string" then
    debug_log("Macro content preview: " .. expansion:sub(1, 100))
  end

  -- Check if it's a simple macro with function patterns (old or new syntax)
  if expansion and type(expansion) == "string" and (expansion:match("[%w_]+%b()") or expansion:match("@{[^}]+}")) then
    debug_log("Macro has function patterns, using expand_functions_in_string")
    -- Has function patterns, expand them properly
    M.expand_functions_in_string(expansion, function(result)
      debug_log("expand_functions_in_string callback called, result: " .. tostring(result and #result or "nil"))
      if result then
        -- First check if result has nested macros (@MACRO_NAME patterns)
        if result:match("@[%w_]+") then
          debug_log("Result has nested macros, recursively expanding")
          debug_log("Expanding nested macros...")

          -- Recursively expand nested macros
          local max_depth = 10
          local depth = 0
          local file_macros = M.parse_file_macros()

          while depth < max_depth do
            local found = false
            result = result:gsub("@([%w_]+)", function(nested_macro)
              debug_log("Found nested macro: @" .. nested_macro)
              local nested_expansion = file_macros[nested_macro]
              if nested_expansion and type(nested_expansion) == "string" then
                found = true
                debug_log("  Expanding @" .. nested_macro)
                return nested_expansion
              else
                debug_log("  Macro @" .. nested_macro .. " not found")
                return "@" .. nested_macro  -- Keep unchanged if not found
              end
            end)

            if not found then
              break
            end
            depth = depth + 1
          end

          -- After expanding nested macros, check if we have new function patterns
          if result:match("@{[^}]+}") or result:match("[%w_]+%b()") then
            debug_log("Nested macros introduced new function patterns, re-expanding")
            -- Recursively call expand_functions_in_string for the nested patterns
            M.expand_functions_in_string(result, function(nested_result)
              if nested_result then
                M.replace_macro_inline(row, start_pos, end_pos, nested_result)
              else
                vim.notify("Nested macro expansion cancelled", vim.log.levels.INFO)
              end
            end)
            return  -- Exit early since we're handling it recursively
          end
        end

        -- Check if result still has variables that need substitution
        if result:match("${[^}]+}") then
          debug_log("Result has variables, performing substitution")
          -- Load CONFIG values
          local file_macros = M.parse_file_macros()
          local values = {}

          -- Add CONFIG values
          if file_macros._CONFIG then
            debug_log("Found CONFIG with " .. vim.tbl_count(file_macros._CONFIG) .. " values")
            for k, v in pairs(file_macros._CONFIG) do
              values[k] = v
              debug_log("  CONFIG: " .. k .. " = " .. v)
            end
          else
            debug_log("No CONFIG macro found")
          end

          -- Also check environment variables
          for var in result:gmatch("${([^}:]+):?[^}]*}") do
            debug_log("Checking variable: " .. var)
            if not values[var] then
              local env_val = os.getenv(var)
              if env_val then
                values[var] = env_val
                debug_log("  ENV: " .. var .. " = " .. env_val)
              else
                debug_log("  No value found for: " .. var)
              end
            end
          end

          debug_log("Substituting with " .. vim.tbl_count(values) .. " values")
          result = M.substitute_variables(result, values)

          -- Check if substitution worked
          if result:match("${BASE_URL}") then
            debug_log("WARNING: BASE_URL still not substituted!")
          end
        else
          debug_log("No variables found in result")
        end
        debug_log("Replacing inline macro at pos " .. start_pos .. "-" .. end_pos)
        M.replace_macro_inline(row, start_pos, end_pos, result)
      else
        vim.notify("Macro expansion cancelled", vim.log.levels.INFO)
      end
    end)
    debug_log("Returning from expand_inline_macro (async expansion started)")
    return
  end

  -- If this is a template-based macro, handle it specially
  if template_def then
    -- vim.notify("Expanding template macro: " .. macro_name, vim.log.levels.INFO)
    M.expand_template_macro(template_def, row, start_pos, end_pos)
    return
  end

  if not expansion then
    vim.notify("Unknown macro: " .. macro_name .. ". Define it with -- MACRO: " .. macro_name, vim.log.levels.WARN)
    return
  end

  -- Handle function macros
  if type(expansion) == "function" then
    expansion = expansion()
  end

  -- Recursively expand any macros within the expansion
  local function expand_nested_macros(text)
    local expanded = text
    local max_depth = 10  -- Prevent infinite recursion
    local depth = 0

    while depth < max_depth do
      local found = false
      -- Look for @MACRO_NAME patterns
      expanded = expanded:gsub("@([%w_]+)", function(nested_macro)
        local nested_expansion = file_macros[nested_macro] or inline_macros[nested_macro]
        if nested_expansion then
          found = true
          if type(nested_expansion) == "function" then
            return nested_expansion()
          else
            return nested_expansion
          end
        else
          return "@" .. nested_macro  -- Keep unchanged if not found
        end
      end)

      if not found then
        break  -- No more macros to expand
      end
      depth = depth + 1
    end

    return expanded
  end

  -- Expand nested macros
  expansion = expand_nested_macros(expansion)

  -- After expanding nested macros, check if the result has function patterns that need evaluation
  if expansion:match("[%w_]+%b()") then
    -- Has function patterns from nested macro expansion, use simple expansion
    -- Check if we're in JSON context (looking for escaped quotes or BODY context)
    local is_json_context = expansion:match('\\\"') ~= nil
    M.expand_simple_macro(expansion, row, start_pos, end_pos, is_json_context)
    return
  end

  -- Check if expansion has variables that need substitution
  if expansion:match("${[^}]+}") then
    -- Inline variable substitution without replacing entire buffer
    -- Collect all variables first
    local variables = {}
    local var_order = {}

    for var, default in expansion:gmatch("${([^}:]+):?([^}]*)}") do
      if not variables[var] then
        variables[var] = default or ""
        table.insert(var_order, var)
      end
    end

    if #var_order == 0 then
      -- No variables found, just replace inline
      M.replace_macro_inline(row, start_pos, end_pos, expansion)
      return
    end

    -- Collect values for each variable
    local values = {}
    local var_index = 1

    local function collect_and_expand()
      if var_index > #var_order then
        -- All values collected, perform substitution
        local result = M.substitute_variables(expansion, values)
        -- Replace only the macro location, not the entire buffer
        M.replace_macro_inline(row, start_pos, end_pos, result)
        return
      end

      local var = var_order[var_index]
      local default = variables[var]
      var_index = var_index + 1

      -- First check if CONFIG macro defines a value for this variable
      local file_macros = M.parse_file_macros()

      -- Check if CONFIG macro defines a value for this variable
      -- Also check for special mappings like DEFAULT_COLUMNS -> COLUMNS
      local config_value = nil
      if file_macros._CONFIG then
        config_value = file_macros._CONFIG[var]

        -- Special mapping: DEFAULT_COLUMNS can be used for COLUMNS variable
        if not config_value and var == "COLUMNS" and file_macros._CONFIG.DEFAULT_COLUMNS then
          config_value = file_macros._CONFIG.DEFAULT_COLUMNS
        end
      end

      if config_value then
        -- Use the config value (already resolved env vars during parsing)
        values[var] = config_value
        -- Debug output
        vim.notify(string.format("Using CONFIG value for %s: %s", var, config_value), vim.log.levels.INFO)
        collect_and_expand()
        return
      end

      -- Special handling for common variables
      if var == "SOURCE" or var:match("^SOURCE") then
        M.quick_source_picker(function(value)
          values[var] = value
          collect_and_expand()
        end)
      elseif var == "BASE_URL" then
        -- This should not be reached if CONFIG defines BASE_URL
        -- Add debug to see why we got here
        vim.notify("BASE_URL not found in CONFIG, prompting user", vim.log.levels.WARN)
        vim.ui.input({
          prompt = "API Base URL: ",
          default = default or "http://localhost:5001"
        }, function(value)
          if value ~= nil then
            values[var] = value
            collect_and_expand()
          end
        end)
      elseif var == "TRADE_DATE" or var:match("^DATE") then
        M.quick_date_picker(function(date)
          -- Auto-convert to TradeDate() function format
          local year, month, day = date:match("(%d+)-(%d+)-(%d+)")
          if year then
            values[var] = string.format("TradeDate(%s, %s, %s)", year, month, day)
          else
            values[var] = date
          end
          collect_and_expand()
        end)
      elseif var == "YEAR" then
        vim.ui.input({
          prompt = "Year: ",
          default = os.date("%Y")
        }, function(value)
          if value ~= nil then
            values[var] = value
            collect_and_expand()
          end
        end)
      elseif var == "MONTH" then
        vim.ui.input({
          prompt = "Month: ",
          default = os.date("%m")
        }, function(value)
          if value ~= nil then
            values[var] = value
            collect_and_expand()
          end
        end)
      elseif var == "DAY" then
        vim.ui.input({
          prompt = "Day: ",
          default = os.date("%d")
        }, function(value)
          if value ~= nil then
            values[var] = value
            collect_and_expand()
          end
        end)
      elseif var == "WHERE_CLAUSE" or var == "WHERE" then
        -- Check if there are WHERE template macros available
        local where_templates = {}
        for macro_name, _ in pairs(file_macros) do
          if macro_name:match("^WHERE_") and file_macros["_TEMPLATE_" .. macro_name] then
            table.insert(where_templates, macro_name)
          end
        end

        if #where_templates > 0 then
          -- Add custom option
          table.insert(where_templates, "Custom WHERE")

          vim.ui.select(where_templates, {
            prompt = "Select WHERE template: "
          }, function(choice)
            if not choice then return end

            if choice == "Custom WHERE" then
              vim.ui.input({
                prompt = "Enter WHERE clause: ",
                default = default or ""
              }, function(where)
                if where ~= nil then
                  values[var] = where
                  collect_and_expand()
                end
              end)
            else
              -- Expand the selected WHERE template
              local template_def = file_macros["_TEMPLATE_" .. choice]
              if template_def then
                -- Recursively collect values for the WHERE template
                M.expand_template_to_string(template_def, function(expanded_where)
                  values[var] = expanded_where
                  collect_and_expand()
                end)
              end
            end
          end)
        else
          -- Fall back to original WHERE builder
          M.where_clause_builder(function(where)
            values[var] = where
            collect_and_expand()
          end)
        end
      elseif var == "COLUMNS" then
        -- This should not be reached if CONFIG defines DEFAULT_COLUMNS
        -- Use the default from the template or prompt for columns
        local default_cols = default
        if not default_cols or default_cols == "" then
          default_cols = "*"
        end
        vim.ui.input({
          prompt = "Columns to select: ",
          default = default_cols
        }, function(value)
          if value ~= nil then
            values[var] = value
            collect_and_expand()
          end
        end)
      else
        -- Check for environment variable
        local env_value = nil

        -- Special handling for API_TOKEN with ${JWT_TOKEN} default
        if var == "API_TOKEN" and default and default:match("${JWT_TOKEN}") then
          local jwt_token = vim.env.JWT_TOKEN
          if jwt_token then
            -- Use JWT_TOKEN from environment
            values[var] = jwt_token
            collect_and_expand()
            return
          end
        else
          -- Check for direct environment variable matching the var name
          env_value = vim.env[var]
        end

        if env_value then
          -- Use environment variable
          values[var] = env_value
          collect_and_expand()
        else
          -- Generic variable input
          vim.ui.input({
            prompt = var .. ": ",
            default = default
          }, function(value)
            if value ~= nil then
              values[var] = value
              collect_and_expand()
            end
          end)
        end
      end
    end

    -- Start collecting values
    collect_and_expand()
  else
    -- No variables, just replace inline
    M.replace_macro_inline(row, start_pos, end_pos, expansion)
  end
end

-- Replace current query with new text
function M.replace_current_query(new_text)
  local mode = vim.api.nvim_get_mode().mode

  if mode == 'v' or mode == 'V' then
    -- Visual mode - replace selection
    vim.cmd('normal! gv')
    vim.cmd('normal! "_d')
    M.insert_at_cursor(new_text)
  else
    -- Normal mode - replace query at cursor
    local navigation = require('sql-cli.navigation')
    navigation.select_query_at_cursor()
    vim.cmd('normal! "_d')
    M.insert_at_cursor(new_text)
  end
end

-- Replace macro inline (used during expand_inline_macro)
function M.replace_macro_inline(row, start_pos, end_pos, new_text)
  debug_log("replace_macro_inline called: row=" .. row .. ", start=" .. start_pos .. ", end=" .. end_pos)
  debug_log("Replacement text preview: " .. new_text:sub(1, 100))

  local line = vim.api.nvim_buf_get_lines(0, row - 1, row, false)[1]
  local prefix = line:sub(1, start_pos - 1)
  local suffix = line:sub(end_pos + 1)

  -- Handle multi-line replacements
  local lines = vim.split(new_text, "\n")
  if #lines == 1 then
    -- Single line - simple replacement
    local new_line = prefix .. new_text .. suffix
    vim.api.nvim_buf_set_lines(0, row - 1, row, false, {new_line})
    vim.api.nvim_win_set_cursor(0, {row, start_pos - 1 + #new_text})
  else
    -- Multiple lines
    lines[1] = prefix .. lines[1]
    lines[#lines] = lines[#lines] .. suffix

    vim.api.nvim_buf_set_lines(0, row - 1, row, false, lines)
    vim.api.nvim_win_set_cursor(0, {row + #lines - 1, #lines[#lines] - #suffix})
  end
end

-- Setup keymaps
function M.setup_keymaps()
  -- Template operations under \sT prefix (capital T for Templates)
  vim.keymap.set('n', '<leader>sT', M.select_template, { desc = 'Template: Select and apply' })
  vim.keymap.set('n', '<leader>sTq', M.quick_substitute, { desc = 'Template: Quick substitute variables' })
  vim.keymap.set({'n', 'v'}, '<leader>sTs', M.save_as_template, { desc = 'Template: Save current as template' })
  vim.keymap.set('n', '<leader>sTe', M.expand_inline_macro, { desc = 'Template: Expand macro at cursor' })
  vim.keymap.set('n', '<leader>sTm', M.expand_file_macros, { desc = 'Template: Expand all macros from definitions' })

  -- Quick pickers
  vim.keymap.set('n', '<leader>sTd', function()
    M.quick_date_picker(function(date)
      M.insert_at_cursor(date)
    end)
  end, { desc = 'Template: Insert quick date' })

  vim.keymap.set('n', '<leader>sTS', function()
    M.quick_source_picker(function(source)
      M.insert_at_cursor('"' .. source .. '"')
    end)
  end, { desc = 'Template: Insert trade source' })
end

return M