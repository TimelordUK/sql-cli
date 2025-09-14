-- SQL CLI Visualization Commands Module
-- Integrates chart rendering with SQL query results

local M = {}
local charts = require('sql-cli.charts')
local utils = require('sql-cli.utils')

-- Parse CSV line handling quoted values
local function parse_csv_line(line)
  local values = {}
  local current = ""
  local in_quotes = false

  for i = 1, #line do
    local char = line:sub(i, i)
    if char == '"' then
      in_quotes = not in_quotes
    elseif char == ',' and not in_quotes then
      table.insert(values, current)
      current = ""
    else
      current = current .. char
    end
  end
  table.insert(values, current)

  return values
end

-- Parse CSV output from sql-cli
local function parse_csv(output)
  local lines = vim.split(output, '\n', {plain=true})
  local headers = {}
  local data = {}

  -- Skip any non-CSV lines (like error messages or comments that start with #)
  local csv_start = 1
  for i, line in ipairs(lines) do
    -- Skip comment lines that start with # and empty lines
    if line:match("^#") or line == "" then
      -- skip
    else
      csv_start = i
      break
    end
  end

  -- Parse headers
  if csv_start <= #lines and lines[csv_start] then
    headers = parse_csv_line(lines[csv_start])
    -- Trim whitespace from headers
    for i, h in ipairs(headers) do
      headers[i] = h:match("^%s*(.-)%s*$")
    end
  end

  -- Parse data rows
  for i = csv_start + 1, #lines do
    local line = lines[i]
    if line and line ~= "" and not line:match("^#") then
      local row = parse_csv_line(line)
      local row_data = {}
      for j, header in ipairs(headers) do
        -- Store both by header name and by index for reliability
        local value = row[j]
        if value then
          -- Trim whitespace
          value = value:match("^%s*(.-)%s*$")
          -- Try to parse as number for numeric columns, but keep as string otherwise
          -- Only convert to number if it's purely numeric
          if value:match("^%-?%d+%.?%d*$") then
            row_data[header] = tonumber(value)
          else
            row_data[header] = value
          end
        end
      end
      if next(row_data) then  -- Only add non-empty rows
        table.insert(data, row_data)
      end
    end
  end

  return headers, data
end

-- Execute SQL query and get results
local function execute_query(query, debug)
  local cmd = string.format('sql-cli -q "%s" -o csv 2>/dev/null', query:gsub('"', '\\"'))
  if debug then
    vim.notify("Executing: " .. cmd, vim.log.levels.INFO)
  end
  local handle = io.popen(cmd)
  local result = handle:read("*a")
  handle:close()
  if debug then
    vim.notify("Raw output (first 200 chars): " .. result:sub(1, 200), vim.log.levels.INFO)
  end
  return result
end

-- ============================================================================
-- BAR CHART COMMAND
-- ============================================================================

-- Create bar chart from query results
-- Expected format: SELECT label, value FROM ...
function M.bar_chart(query, options)
  options = options or {}
  local debug = options.debug
  local debug_info = {}

  if debug then
    table.insert(debug_info, "=== QUERY ===")
    table.insert(debug_info, query)
    table.insert(debug_info, "")
  end

  local output = execute_query(query, false)

  if debug then
    table.insert(debug_info, "=== RAW CSV OUTPUT ===")
    local lines = vim.split(output, '\n', {plain=true})
    for i = 1, math.min(10, #lines) do
      if lines[i] and lines[i] ~= "" then
        table.insert(debug_info, string.format("Line %d: [%s]", i, lines[i]))
      end
    end
    if #lines > 10 then
      table.insert(debug_info, string.format("... (%d more lines)", #lines - 10))
    end
    table.insert(debug_info, "")
  end

  local headers, data = parse_csv(output)

  if #headers < 2 then
    vim.notify("Bar chart requires at least 2 columns (label, value)", vim.log.levels.ERROR)
    return
  end

  -- Debug: Show parsed data
  if debug then
    table.insert(debug_info, "=== PARSED DATA ===")
    table.insert(debug_info, string.format("Headers: [%s]", table.concat(headers, ", ")))
    table.insert(debug_info, string.format("Total rows: %d", #data))
    table.insert(debug_info, "")
    table.insert(debug_info, "First 5 rows:")
    for i = 1, math.min(5, #data) do
      local row = data[i]
      local label_val = row[headers[1]]
      local num_val = row[headers[2]]
      table.insert(debug_info, string.format("  Row %d: [%s]='%s' (type: %s), [%s]=%s (type: %s)",
        i,
        headers[1], tostring(label_val), type(label_val),
        headers[2], tostring(num_val), type(num_val)))
    end
    table.insert(debug_info, "")
  end

  -- Transform data for chart
  local chart_data = {}
  for i, row in ipairs(data) do
    local label_val = row[headers[1]]
    local num_val = row[headers[2]]

    -- Ensure we have valid data
    if label_val ~= nil and num_val ~= nil then
      local label_str = tostring(label_val)
      local value_num = tonumber(num_val) or 0

      if debug and i <= 5 then
        table.insert(debug_info, string.format("Transform row %d: '%s' -> '%s', %s -> %.2f",
          i, tostring(label_val), label_str, tostring(num_val), value_num))
      end

      table.insert(chart_data, {
        label = label_str,
        value = value_num
      })
    end
  end

  if debug then
    table.insert(debug_info, "")
    table.insert(debug_info, "=== FINAL CHART DATA ===")
    table.insert(debug_info, string.format("Total items: %d", #chart_data))
    for i = 1, math.min(5, #chart_data) do
      table.insert(debug_info, string.format("  Item %d: label='%s', value=%.2f",
        i, chart_data[i].label, chart_data[i].value))
    end
  end

  if #chart_data == 0 then
    vim.notify("No valid data for bar chart", vim.log.levels.ERROR)
    return
  end

  -- Generate chart
  local lines = charts.horizontal_bar_chart(chart_data, options)

  -- If debug, append debug info below the chart
  if debug and #debug_info > 0 then
    table.insert(lines, "")
    table.insert(lines, "")
    table.insert(lines, "━━━━━━━━━━━━━━━━━━━━ DEBUG INFO ━━━━━━━━━━━━━━━━━━━━")
    for _, line in ipairs(debug_info) do
      table.insert(lines, line)
    end
  end

  -- Create buffer with chart
  local title = "Bar Chart: " .. (options.title or query:sub(1, 50))
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, math.min(#lines + 5, 40))
end

-- ============================================================================
-- PIE CHART COMMAND
-- ============================================================================

-- Create pie chart from query results
function M.pie_chart(query, options)
  options = options or {}

  local output = execute_query(query)
  local headers, data = parse_csv(output)

  if #headers < 2 then
    vim.notify("Pie chart requires at least 2 columns (label, value)", vim.log.levels.ERROR)
    return
  end

  -- Transform data for chart
  local chart_data = {}
  for _, row in ipairs(data) do
    table.insert(chart_data, {
      label = tostring(row[headers[1]]),
      value = tonumber(row[headers[2]]) or 0
    })
  end

  -- Generate chart with configurable radius (can be set via vim.g.sql_cli_pie_radius)
  options.radius = options.radius or vim.g.sql_cli_pie_radius or 15  -- Default to larger radius
  local lines = charts.pie_chart(chart_data, options)

  -- Create buffer with chart
  local title = "Pie Chart: " .. (options.title or query:sub(1, 50))
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split with more height for larger chart
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, math.min(#lines + 5, 45))  -- Increased from 35 to 45
end

-- ============================================================================
-- HISTOGRAM COMMAND
-- ============================================================================

-- Create histogram from query results
-- Expected format: SELECT value FROM ...
function M.histogram(query, options)
  options = options or {}

  local output = execute_query(query)
  local headers, data = parse_csv(output)

  if #headers < 1 then
    vim.notify("Histogram requires at least 1 numeric column", vim.log.levels.ERROR)
    return
  end

  -- Extract numeric values
  local values = {}
  for _, row in ipairs(data) do
    local val = tonumber(row[headers[1]])
    if val then
      table.insert(values, val)
    end
  end

  if #values == 0 then
    vim.notify("No numeric values found for histogram", vim.log.levels.ERROR)
    return
  end

  -- Generate chart
  local lines = charts.histogram(values, options)

  -- Create buffer with chart
  local title = "Histogram: " .. (options.title or headers[1])
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, math.min(#lines + 5, 25))
end

-- ============================================================================
-- BOX PLOT COMMAND
-- ============================================================================

-- Create box plot from statistical query
-- Expected: SELECT MIN(x), PERCENTILE(x, 25), MEDIAN(x), PERCENTILE(x, 75), MAX(x), AVG(x)
function M.box_plot(query, options)
  options = options or {}

  local output = execute_query(query)
  local headers, data = parse_csv(output)

  if #data == 0 or #headers < 5 then
    vim.notify("Box plot requires statistical aggregates (min, q1, median, q3, max)", vim.log.levels.ERROR)
    return
  end

  -- Extract statistics
  local row = data[1]
  local stats = {
    min = tonumber(row[headers[1]]) or 0,
    q1 = tonumber(row[headers[2]]) or 0,
    median = tonumber(row[headers[3]]) or 0,
    q3 = tonumber(row[headers[4]]) or 0,
    max = tonumber(row[headers[5]]) or 0,
    mean = headers[6] and tonumber(row[headers[6]]) or nil
  }

  -- Generate chart
  local lines = charts.box_plot(stats, options)

  -- Create buffer with chart
  local title = "Box Plot: " .. (options.title or "Statistical Distribution")
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, 10)
end

-- ============================================================================
-- SPARKLINE COMMAND
-- ============================================================================

-- Create sparkline from time series data
function M.sparkline(query, options)
  options = options or {}

  local output = execute_query(query)
  local headers, data = parse_csv(output)

  if #headers < 1 then
    vim.notify("Sparkline requires at least 1 numeric column", vim.log.levels.ERROR)
    return
  end

  -- Extract values (assume last column is the value)
  local values = {}
  local value_col = headers[#headers]

  for _, row in ipairs(data) do
    local val = tonumber(row[value_col])
    if val then
      table.insert(values, val)
    end
  end

  -- Generate sparkline
  local lines = charts.sparkline(values, options)

  -- Add context
  table.insert(lines, 1, string.format("Sparkline for %s (%d points):", value_col, #values))
  table.insert(lines, string.format("Min: %.2f  Max: %.2f  Latest: %.2f",
    math.min(table.unpack(values)),
    math.max(table.unpack(values)),
    values[#values]
  ))

  -- Show inline (don't create new buffer)
  for _, line in ipairs(lines) do
    print(line)
  end
end

-- ============================================================================
-- SCATTER PLOT COMMAND
-- ============================================================================

-- Create scatter plot from query results
-- Expected format: SELECT x, y FROM ...
function M.scatter_plot(query, options)
  options = options or {}

  local output = execute_query(query)
  local headers, data = parse_csv(output)

  if #headers < 2 then
    vim.notify("Scatter plot requires at least 2 numeric columns (x, y)", vim.log.levels.ERROR)
    return
  end

  -- Transform data for chart
  local chart_data = {}
  for _, row in ipairs(data) do
    local x = tonumber(row[headers[1]])
    local y = tonumber(row[headers[2]])
    if x and y then
      table.insert(chart_data, {x = x, y = y})
    end
  end

  if #chart_data == 0 then
    vim.notify("No valid numeric pairs found for scatter plot", vim.log.levels.ERROR)
    return
  end

  -- Generate chart
  local lines = charts.scatter_plot(chart_data, options)

  -- Create buffer with chart
  local title = string.format("Scatter Plot: %s vs %s", headers[1], headers[2])
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, math.min(#lines + 5, 30))
end

-- ============================================================================
-- AT CURSOR FUNCTIONS - Execute query at cursor and visualize
-- ============================================================================

-- Get query at cursor and data file
local function get_query_at_cursor()
  local bufnr = vim.api.nvim_get_current_buf()
  local cursor_line = vim.fn.line('.')
  local lines = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

  -- Find query boundaries
  local start_line, end_line = utils.find_query_at_cursor(lines, cursor_line)

  if not start_line then
    return nil, nil
  end

  -- Extract the query, skipping comments and terminators
  local query_lines = {}
  for i = start_line, end_line do
    local line = lines[i]
    -- Skip GO terminators and comment lines
    if not line:match("^%s*GO%s*$") and not line:match("^%s*%-%-") then
      table.insert(query_lines, line)
    end
  end

  local query = table.concat(query_lines, "\n")

  -- Auto-detect data file from hints
  local buf_path = vim.api.nvim_buf_get_name(bufnr)
  local buf_dir = nil
  if buf_path and buf_path ~= "" then
    buf_dir = vim.fn.fnamemodify(buf_path, ":h")
  end
  local data_file = utils.detect_data_hint(lines, buf_dir)

  -- If no hint found, check if current buffer is a CSV
  if not data_file then
    local filename = vim.api.nvim_buf_get_name(bufnr)
    if filename:match("%.csv$") then
      data_file = filename
    end
  end

  return query, data_file
end

-- Execute query with data file if needed
local function execute_query_with_file(query, data_file, debug)
  local cmd
  if data_file then
    cmd = string.format('sql-cli "%s" -q "%s" -o csv 2>/dev/null',
      data_file, query:gsub('"', '\\"'))
  else
    cmd = string.format('sql-cli -q "%s" -o csv 2>/dev/null',
      query:gsub('"', '\\"'))
  end

  if debug then
    vim.notify("Executing command: " .. cmd, vim.log.levels.INFO)
  end

  local handle = io.popen(cmd)
  local result = handle:read("*a")
  handle:close()

  if debug then
    vim.notify("Output length: " .. #result .. " chars", vim.log.levels.INFO)
    if #result > 0 then
      vim.notify("First 300 chars: " .. result:sub(1, 300), vim.log.levels.INFO)
    end
  end

  return result
end

-- Bar chart at cursor
function M.bar_chart_at_cursor(options)
  options = options or {}
  local debug = options.debug or vim.g.sql_cli_debug_charts
  local debug_info = {}

  local query, data_file = get_query_at_cursor()
  if not query then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end

  if debug then
    table.insert(debug_info, "=== QUERY DETECTION ===")
    table.insert(debug_info, "Query: " .. query)
    table.insert(debug_info, "Data file: " .. (data_file or "none"))
    table.insert(debug_info, "")
  end

  local output = execute_query_with_file(query, data_file, false)

  if debug then
    table.insert(debug_info, "=== RAW CSV OUTPUT ===")
    local lines = vim.split(output, '\n', {plain=true})
    for i = 1, math.min(10, #lines) do
      if lines[i] and lines[i] ~= "" then
        table.insert(debug_info, string.format("Line %d: [%s]", i, lines[i]))
      end
    end
    if #lines > 10 then
      table.insert(debug_info, string.format("... (%d more lines)", #lines - 10))
    end
    table.insert(debug_info, "")
  end

  local headers, data = parse_csv(output)

  if #headers < 2 then
    vim.notify("Bar chart requires at least 2 columns (label, value)", vim.log.levels.ERROR)
    return
  end

  -- Debug: Show parsed data
  if debug then
    table.insert(debug_info, "=== PARSED DATA ===")
    table.insert(debug_info, string.format("Headers: [%s]", table.concat(headers, ", ")))
    table.insert(debug_info, string.format("Total rows: %d", #data))
    table.insert(debug_info, "")
    table.insert(debug_info, "First 5 rows:")
    for i = 1, math.min(5, #data) do
      local row = data[i]
      local label_val = row[headers[1]]
      local num_val = row[headers[2]]
      table.insert(debug_info, string.format("  Row %d: [%s]='%s' (type: %s), [%s]=%s (type: %s)",
        i,
        headers[1], tostring(label_val), type(label_val),
        headers[2], tostring(num_val), type(num_val)))
    end
    table.insert(debug_info, "")
  end

  -- Transform data for chart
  local chart_data = {}
  for i, row in ipairs(data) do
    -- Ensure we're using the right columns
    local label_val = row[headers[1]]
    local numeric_val = row[headers[2]]

    -- Skip if no valid data
    if label_val and numeric_val then
      local label_str = tostring(label_val)
      local value_num = tonumber(numeric_val) or 0

      if debug and i <= 5 then
        table.insert(debug_info, string.format("Transform row %d: '%s' -> '%s', %s -> %.2f",
          i, tostring(label_val), label_str, tostring(numeric_val), value_num))
      end

      table.insert(chart_data, {
        label = label_str,
        value = value_num
      })
    end
  end

  if debug then
    table.insert(debug_info, "")
    table.insert(debug_info, "=== FINAL CHART DATA ===")
    table.insert(debug_info, string.format("Total items: %d", #chart_data))
    for i = 1, math.min(5, #chart_data) do
      table.insert(debug_info, string.format("  Item %d: label='%s', value=%.2f",
        i, chart_data[i].label, chart_data[i].value))
    end
  end

  -- Check if we have any data
  if #chart_data == 0 then
    vim.notify("No valid data found for bar chart", vim.log.levels.ERROR)
    return
  end

  -- Generate chart
  local lines = charts.horizontal_bar_chart(chart_data, options)

  -- If debug, append debug info below the chart
  if debug and #debug_info > 0 then
    table.insert(lines, "")
    table.insert(lines, "")
    table.insert(lines, "━━━━━━━━━━━━━━━━━━━━ DEBUG INFO ━━━━━━━━━━━━━━━━━━━━")
    for _, line in ipairs(debug_info) do
      table.insert(lines, line)
    end
  end

  -- Create buffer with chart
  local title = "Bar Chart: Query at line " .. vim.fn.line('.')
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, math.min(#lines + 5, 40))
end

-- Pie chart at cursor
function M.pie_chart_at_cursor(options)
  options = options or {}

  local query, data_file = get_query_at_cursor()
  if not query then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end

  local output = execute_query_with_file(query, data_file, false)
  local headers, data = parse_csv(output)

  if #headers < 2 then
    vim.notify("Pie chart requires at least 2 columns (label, value)", vim.log.levels.ERROR)
    return
  end

  -- Transform data for chart
  local chart_data = {}
  for _, row in ipairs(data) do
    table.insert(chart_data, {
      label = tostring(row[headers[1]]),
      value = tonumber(row[headers[2]]) or 0
    })
  end

  -- Generate chart with configurable radius (can be set via vim.g.sql_cli_pie_radius)
  options.radius = options.radius or vim.g.sql_cli_pie_radius or 15  -- Default to larger radius
  local lines = charts.pie_chart(chart_data, options)

  -- Create buffer with chart
  local title = "Pie Chart: Query at line " .. vim.fn.line('.')
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split with more height for larger chart
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, math.min(#lines + 5, 45))  -- Increased from 35 to 45
end

-- Histogram at cursor
function M.histogram_at_cursor(options)
  options = options or {}

  local query, data_file = get_query_at_cursor()
  if not query then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end

  local output = execute_query_with_file(query, data_file, false)
  local headers, data = parse_csv(output)

  if #headers < 1 then
    vim.notify("Histogram requires at least 1 numeric column", vim.log.levels.ERROR)
    return
  end

  -- Extract numeric values
  local values = {}
  for _, row in ipairs(data) do
    local val = tonumber(row[headers[1]])
    if val then
      table.insert(values, val)
    end
  end

  if #values == 0 then
    vim.notify("No numeric values found for histogram", vim.log.levels.ERROR)
    return
  end

  -- Generate chart
  local lines = charts.histogram(values, options)

  -- Create buffer with chart
  local title = "Histogram: " .. headers[1]
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, math.min(#lines + 5, 25))
end

-- Scatter plot at cursor
function M.scatter_plot_at_cursor(options)
  options = options or {}

  local query, data_file = get_query_at_cursor()
  if not query then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end

  local output = execute_query_with_file(query, data_file, false)
  local headers, data = parse_csv(output)

  if #headers < 2 then
    vim.notify("Scatter plot requires at least 2 numeric columns (x, y)", vim.log.levels.ERROR)
    return
  end

  -- Transform data for chart
  local chart_data = {}
  for _, row in ipairs(data) do
    local x = tonumber(row[headers[1]])
    local y = tonumber(row[headers[2]])
    if x and y then
      table.insert(chart_data, {x = x, y = y})
    end
  end

  if #chart_data == 0 then
    vim.notify("No valid numeric pairs found for scatter plot", vim.log.levels.ERROR)
    return
  end

  -- Generate chart
  local lines = charts.scatter_plot(chart_data, options)

  -- Create buffer with chart
  local title = string.format("Scatter Plot: %s vs %s", headers[1], headers[2])
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, math.min(#lines + 5, 30))
end

-- Sparkline at cursor (inline display)
function M.sparkline_at_cursor(options)
  options = options or {}

  local query, data_file = get_query_at_cursor()
  if not query then
    vim.notify("No SQL statement found at cursor", vim.log.levels.WARN)
    return
  end

  local output = execute_query_with_file(query, data_file, false)
  local headers, data = parse_csv(output)

  if #headers < 1 then
    vim.notify("Sparkline requires at least 1 numeric column", vim.log.levels.ERROR)
    return
  end

  -- Extract values (assume last column is the value)
  local values = {}
  local value_col = headers[#headers]

  for _, row in ipairs(data) do
    local val = tonumber(row[value_col])
    if val then
      table.insert(values, val)
    end
  end

  if #values == 0 then
    vim.notify("No numeric values found for sparkline", vim.log.levels.ERROR)
    return
  end

  -- Generate sparkline
  local lines = charts.sparkline(values, options)

  -- Add context
  local min_val = math.min(table.unpack(values))
  local max_val = math.max(table.unpack(values))
  local latest_val = values[#values]

  table.insert(lines, 1, string.format("Sparkline for %s (%d points):", value_col, #values))
  table.insert(lines, 2, "")
  table.insert(lines, "")
  table.insert(lines, string.format("Min: %.2f  Max: %.2f  Latest: %.2f",
    min_val, max_val, latest_val))

  -- Create small buffer with sparkline
  local title = "Sparkline: " .. value_col
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in smaller split
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, 8)
end

-- ============================================================================
-- COMMANDS REGISTRATION
-- ============================================================================

function M.setup()
  -- Bar Chart
  vim.api.nvim_create_user_command('SqlBarChart', function(opts)
    M.bar_chart(opts.args, {})
  end, {
    nargs = 1,
    desc = 'Create bar chart from SQL query'
  })

  -- Bar Chart with Debug
  vim.api.nvim_create_user_command('SqlBarChartDebug', function(opts)
    M.bar_chart(opts.args, {debug = true})
  end, {
    nargs = 1,
    desc = 'Create bar chart from SQL query with debug output'
  })

  -- Pie Chart
  vim.api.nvim_create_user_command('SqlPieChart', function(opts)
    M.pie_chart(opts.args, {})
  end, {
    nargs = 1,
    desc = 'Create pie chart from SQL query'
  })

  -- Histogram
  vim.api.nvim_create_user_command('SqlHistogram', function(opts)
    M.histogram(opts.args, {})
  end, {
    nargs = 1,
    desc = 'Create histogram from SQL query'
  })

  -- Box Plot
  vim.api.nvim_create_user_command('SqlBoxPlot', function(opts)
    M.box_plot(opts.args, {})
  end, {
    nargs = 1,
    desc = 'Create box plot from SQL statistical query'
  })

  -- Sparkline
  vim.api.nvim_create_user_command('SqlSparkline', function(opts)
    M.sparkline(opts.args, {})
  end, {
    nargs = 1,
    desc = 'Create sparkline from SQL query'
  })

  -- Scatter Plot
  vim.api.nvim_create_user_command('SqlScatter', function(opts)
    M.scatter_plot(opts.args, {})
  end, {
    nargs = 1,
    desc = 'Create scatter plot from SQL query'
  })

  -- Quick statistical visualization
  vim.api.nvim_create_user_command('SqlStats', function(opts)
    local table_name = opts.args
    local column = opts.fargs[2] or '*'

    -- Run statistical analysis
    local stats_query = string.format([[
      SELECT
        COUNT(%s) as count,
        MIN(%s) as min,
        MAX(%s) as max,
        AVG(%s) as mean,
        MEDIAN(%s) as median,
        STDDEV(%s) as stddev,
        VARIANCE(%s) as variance
      FROM %s
    ]], column, column, column, column, column, column, column, table_name)

    local output = execute_query(stats_query)
    print("Statistical Summary:")
    print(output)

    -- Also show distribution
    local dist_query = string.format("SELECT %s FROM %s", column, table_name)
    M.histogram(dist_query, {bins = 15, height = 10})
  end, {
    nargs = '+',
    desc = 'Show statistical summary and distribution'
  })

  -- Enable/disable chart debugging
  vim.api.nvim_create_user_command('SqlChartDebug', function(opts)
    local enable = opts.args == "on" or opts.args == "true" or opts.args == "1"
    vim.g.sql_cli_debug_charts = enable
    vim.notify("SQL chart debugging " .. (enable and "enabled" or "disabled"), vim.log.levels.INFO)
  end, {
    nargs = '?',
    desc = 'Enable/disable SQL chart debugging (on/off)'
  })

  -- Test bar chart with simple data
  vim.api.nvim_create_user_command('SqlTestBarChart', function()
    local test_query = [[SELECT 'Category A' as label, 100 as value UNION SELECT 'Category B', 200 UNION SELECT 'Category C', 150]]
    M.bar_chart(test_query, {debug = vim.g.sql_cli_debug_charts})
  end, {
    desc = 'Test bar chart with simple data'
  })

  -- Set pie chart radius
  vim.api.nvim_create_user_command('SqlPieRadius', function(opts)
    local radius = tonumber(opts.args)
    if radius and radius >= 5 and radius <= 30 then
      vim.g.sql_cli_pie_radius = radius
      vim.notify(string.format("Pie chart radius set to %d", radius), vim.log.levels.INFO)
    else
      vim.notify("Pie chart radius must be between 5 and 30", vim.log.levels.ERROR)
    end
  end, {
    nargs = 1,
    desc = 'Set pie chart radius (5-30)'
  })
end

return M