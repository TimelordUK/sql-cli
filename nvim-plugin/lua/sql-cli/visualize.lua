-- SQL CLI Visualization Commands Module
-- Integrates chart rendering with SQL query results

local M = {}
local charts = require('sql-cli.charts')

-- Parse CSV output from sql-cli
local function parse_csv(output)
  local lines = vim.split(output, '\n')
  local headers = {}
  local data = {}

  -- Parse headers
  if #lines > 0 then
    headers = vim.split(lines[1], ',')
  end

  -- Parse data rows
  for i = 2, #lines do
    if lines[i] and lines[i] ~= "" then
      local row = vim.split(lines[i], ',')
      local row_data = {}
      for j, header in ipairs(headers) do
        row_data[header] = tonumber(row[j]) or row[j]
      end
      table.insert(data, row_data)
    end
  end

  return headers, data
end

-- Execute SQL query and get results
local function execute_query(query)
  local cmd = string.format('sql-cli -q "%s" -o csv 2>/dev/null', query:gsub('"', '\\"'))
  local handle = io.popen(cmd)
  local result = handle:read("*a")
  handle:close()
  return result
end

-- ============================================================================
-- BAR CHART COMMAND
-- ============================================================================

-- Create bar chart from query results
-- Expected format: SELECT label, value FROM ...
function M.bar_chart(query, options)
  options = options or {}

  local output = execute_query(query)
  local headers, data = parse_csv(output)

  if #headers < 2 then
    vim.notify("Bar chart requires at least 2 columns (label, value)", vim.log.levels.ERROR)
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

  -- Generate chart
  local lines = charts.horizontal_bar_chart(chart_data, options)

  -- Create buffer with chart
  local title = "Bar Chart: " .. (options.title or query:sub(1, 50))
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, math.min(#lines + 5, 30))
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

  -- Generate chart
  local lines = charts.pie_chart(chart_data, options)

  -- Create buffer with chart
  local title = "Pie Chart: " .. (options.title or query:sub(1, 50))
  local bufnr = charts.create_chart_buffer(lines, title)

  -- Open in split
  vim.cmd('split')
  vim.api.nvim_win_set_buf(0, bufnr)
  vim.api.nvim_win_set_height(0, math.min(#lines + 5, 35))
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
end

return M