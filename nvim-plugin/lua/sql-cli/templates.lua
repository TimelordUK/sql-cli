-- SQL Template/Macro System for SQL-CLI
-- Allows quick reuse of common query patterns with variable substitution

local M = {}

-- Store user-defined templates
M.templates = {}

-- Common variable definitions with quick selectors
M.common_variables = {
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
      LIMIT = { prompt = "Limit", default = "100" },
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

      -- Build JSON body with proper formatting
      vars.WHERE_JSON = string.format('{\n        "Where": "%s",\n        "Limit": %s\n    }',
        where_clause,
        vars.LIMIT or "100"
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
        "Where": "Source = \\"${SOURCE}\\"",
        "Limit": ${LIMIT:100}
    }'
    FORMAT JSON
    JSON_PATH 'Result'
)
SELECT * FROM trades
ORDER BY TradeDate DESC]],
    variables = {
      SOURCE = { prompt = "Trade Source", default = "Bloomberg_FIX_FX", type = "select", options = "sources" },
      LIMIT = { prompt = "Limit", default = "100" },
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
        "Where": "TradeDate >= TradeDate(${START_YEAR}, ${START_MONTH}, ${START_DAY}) and TradeDate <= TradeDate(${END_YEAR}, ${END_MONTH}, ${END_DAY})"
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

  simple_filter = {
    name = "Simple Trade Filter",
    template = [[
WITH WEB trades AS (
    URL '${BASE_URL:http://localhost:5001}/trades'
    FORMAT JSON
    JSON_PATH '${JSON_PATH:Result}'
)
SELECT *
FROM trades
WHERE ${FILTER_COLUMN} = '${FILTER_VALUE}'
ORDER BY ${ORDER_BY:TradeDate} ${ORDER_DIR:DESC}]],
    variables = {
      BASE_URL = { prompt = "API Base URL", default = "http://localhost:5001" },
      JSON_PATH = { prompt = "JSON Path", default = "Result" },
      FILTER_COLUMN = { prompt = "Filter Column", default = "Source" },
      FILTER_VALUE = { prompt = "Filter Value", default = "" },
      ORDER_BY = { prompt = "Order By", default = "TradeDate" },
      ORDER_DIR = { prompt = "Order Direction", default = "DESC", type = "select", options = {"ASC", "DESC"} }
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
  vim.ui.select(M.common_variables.sources, {
    prompt = "Select Trade Source:"
  }, function(choice)
    if choice then
      callback(choice)
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

-- Quick variable substitution for current query
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

    -- Special handling for common variables
    if var == "SOURCE" then
      M.quick_source_picker(function(value)
        values[var] = value
        get_next_value()
      end)
    elseif var:match("DATE") or var:match("Day") then
      M.quick_date_picker(function(value)
        values[var] = value
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

-- Setup keymaps
function M.setup_keymaps()
  -- Template operations under \sT prefix (capital T for Templates)
  vim.keymap.set('n', '<leader>sT', M.select_template, { desc = 'Template: Select and apply' })
  vim.keymap.set('n', '<leader>sTq', M.quick_substitute, { desc = 'Template: Quick substitute variables' })
  vim.keymap.set({'n', 'v'}, '<leader>sTs', M.save_as_template, { desc = 'Template: Save current as template' })

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