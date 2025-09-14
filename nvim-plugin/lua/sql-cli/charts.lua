-- SQL CLI ASCII Chart Rendering Module
-- Creates ASCII visualizations for query results

local M = {}

-- ============================================================================
-- BAR CHART
-- ============================================================================

-- Create a horizontal bar chart
-- @param data: table of {label, value} pairs
-- @param options: {width=80, show_values=true, bar_char='█'}
function M.horizontal_bar_chart(data, options)
  options = options or {}
  local width = options.width or 60
  local show_values = options.show_values ~= false
  local bar_char = options.bar_char or '█'
  local empty_char = options.empty_char or '░'

  local lines = {}

  -- Find max value and label width
  local max_value = 0
  local max_label_width = 0
  for _, item in ipairs(data) do
    max_value = math.max(max_value, item.value)
    max_label_width = math.max(max_label_width, #item.label)
  end

  -- Generate bars
  for _, item in ipairs(data) do
    local label = string.format("%-" .. max_label_width .. "s", item.label)
    local bar_width = math.floor((item.value / max_value) * width)
    local bar = string.rep(bar_char, bar_width)
    local empty = string.rep(empty_char, width - bar_width)

    local line = label .. " │" .. bar .. empty
    if show_values then
      line = line .. " " .. string.format("%.2f", item.value)
    end
    table.insert(lines, line)
  end

  return lines
end

-- ============================================================================
-- PIE CHART
-- ============================================================================

-- Create an ASCII pie chart using box drawing characters
-- @param data: table of {label, value} pairs
-- @param options: {radius=10, show_legend=true}
function M.pie_chart(data, options)
  options = options or {}
  local radius = options.radius or 10
  local show_legend = options.show_legend ~= false

  local lines = {}
  local chars = {'█', '▓', '▒', '░', '▄', '▀', '▌', '▐', '■', '□'}

  -- Calculate total and percentages
  local total = 0
  for _, item in ipairs(data) do
    total = total + item.value
  end

  -- Create circular representation
  local diameter = radius * 2 + 1
  local center = radius + 1

  -- Calculate angles for each segment
  local segments = {}
  local current_angle = 0
  for i, item in ipairs(data) do
    local percentage = item.value / total
    local angle_span = percentage * 2 * math.pi
    table.insert(segments, {
      start_angle = current_angle,
      end_angle = current_angle + angle_span,
      char = chars[(i - 1) % #chars + 1],
      label = item.label,
      percentage = percentage * 100,
      value = item.value
    })
    current_angle = current_angle + angle_span
  end

  -- Draw the circle
  for y = 1, diameter do
    local line = ""
    for x = 1, diameter do
      local dx = x - center
      local dy = y - center
      local distance = math.sqrt(dx * dx + dy * dy)

      if distance <= radius then
        -- Calculate angle from center
        local angle = math.atan2(dy, dx) + math.pi

        -- Find which segment this angle belongs to
        local char = ' '
        for _, segment in ipairs(segments) do
          if angle >= segment.start_angle and angle < segment.end_angle then
            char = segment.char
            break
          end
        end
        line = line .. char
      else
        line = line .. ' '
      end
    end
    table.insert(lines, line)
  end

  -- Add legend
  if show_legend then
    table.insert(lines, "")
    table.insert(lines, "Legend:")
    for _, segment in ipairs(segments) do
      local legend_line = string.format("%s %s: %.1f%% (%.2f)",
        segment.char,
        segment.label,
        segment.percentage,
        segment.value
      )
      table.insert(lines, legend_line)
    end
  end

  return lines
end

-- ============================================================================
-- HISTOGRAM
-- ============================================================================

-- Create a frequency histogram
-- @param data: array of numeric values
-- @param options: {bins=10, height=15, width=60}
function M.histogram(data, options)
  options = options or {}
  local bins = options.bins or 10
  local height = options.height or 15
  local width = options.width or 60

  local lines = {}

  -- Find min and max
  local min_val = math.huge
  local max_val = -math.huge
  for _, val in ipairs(data) do
    min_val = math.min(min_val, val)
    max_val = math.max(max_val, val)
  end

  -- Create bins
  local bin_width = (max_val - min_val) / bins
  local bin_counts = {}
  local bin_labels = {}

  for i = 1, bins do
    bin_counts[i] = 0
    local bin_start = min_val + (i - 1) * bin_width
    local bin_end = min_val + i * bin_width
    bin_labels[i] = string.format("%.1f-%.1f", bin_start, bin_end)
  end

  -- Count values in each bin
  for _, val in ipairs(data) do
    local bin_index = math.min(bins, math.max(1,
      math.floor((val - min_val) / bin_width) + 1))
    bin_counts[bin_index] = bin_counts[bin_index] + 1
  end

  -- Find max count for scaling
  local max_count = 0
  for _, count in ipairs(bin_counts) do
    max_count = math.max(max_count, count)
  end

  -- Draw histogram
  for y = height, 1, -1 do
    local line = ""
    for x = 1, bins do
      local bar_height = math.floor((bin_counts[x] / max_count) * height)
      if bar_height >= y then
        line = line .. "█"
      else
        line = line .. " "
      end
      line = line .. " "
    end

    -- Add y-axis label
    if y == height then
      line = string.format("%4d │%s", max_count, line)
    elseif y == 1 then
      line = string.format("%4d │%s", 0, line)
    else
      line = "     │" .. line
    end

    table.insert(lines, line)
  end

  -- Add x-axis
  table.insert(lines, "     └" .. string.rep("─", bins * 2))

  -- Add labels (simplified)
  local label_line = "      "
  for i = 1, bins do
    if i == 1 or i == bins or i == math.floor(bins/2) then
      label_line = label_line .. string.format("%.1f", min_val + (i-1) * bin_width)
      label_line = label_line .. string.rep(" ", math.max(0, 2 - #tostring(i)))
    else
      label_line = label_line .. "  "
    end
  end
  table.insert(lines, label_line)

  return lines
end

-- ============================================================================
-- SPARKLINE
-- ============================================================================

-- Create a sparkline (mini line chart)
-- @param data: array of numeric values
-- @param options: {width=40}
function M.sparkline(data, options)
  options = options or {}
  local width = options.width or math.min(#data, 40)

  local spark_chars = {'▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'}

  -- Find min and max
  local min_val = math.huge
  local max_val = -math.huge
  for _, val in ipairs(data) do
    min_val = math.min(min_val, val)
    max_val = math.max(max_val, val)
  end

  -- Normalize and create sparkline
  local sparkline = ""
  local step = math.max(1, math.floor(#data / width))

  for i = 1, #data, step do
    local val = data[i]
    local normalized = (val - min_val) / (max_val - min_val)
    local char_index = math.min(8, math.max(1, math.floor(normalized * 8) + 1))
    sparkline = sparkline .. spark_chars[char_index]
  end

  return {sparkline}
end

-- ============================================================================
-- BOX PLOT
-- ============================================================================

-- Create a box plot for statistical data
-- @param stats: {min, q1, median, q3, max, mean}
-- @param options: {width=60, show_labels=true}
function M.box_plot(stats, options)
  options = options or {}
  local width = options.width or 60
  local show_labels = options.show_labels ~= false

  local lines = {}

  -- Scale values to width
  local range = stats.max - stats.min
  local scale = function(val)
    return math.floor(((val - stats.min) / range) * width)
  end

  local min_pos = scale(stats.min)
  local q1_pos = scale(stats.q1)
  local median_pos = scale(stats.median)
  local q3_pos = scale(stats.q3)
  local max_pos = scale(stats.max)
  local mean_pos = scale(stats.mean or stats.median)

  -- Build the box plot line
  local plot_line = ""
  for i = 0, width do
    if i == min_pos then
      plot_line = plot_line .. "├"
    elseif i == max_pos then
      plot_line = plot_line .. "┤"
    elseif i > min_pos and i < q1_pos then
      plot_line = plot_line .. "─"
    elseif i == q1_pos then
      plot_line = plot_line .. "┌"
    elseif i > q1_pos and i < q3_pos then
      if i == median_pos then
        plot_line = plot_line .. "┼"
      else
        plot_line = plot_line .. "─"
      end
    elseif i == q3_pos then
      plot_line = plot_line .. "┐"
    elseif i > q3_pos and i < max_pos then
      plot_line = plot_line .. "─"
    else
      plot_line = plot_line .. " "
    end
  end

  -- Add mean marker on separate line
  local mean_line = string.rep(" ", mean_pos) .. "▼" .. " (mean)"

  table.insert(lines, mean_line)
  table.insert(lines, plot_line)

  -- Add labels
  if show_labels then
    local label_line = ""
    label_line = string.format("Min:%.2f  Q1:%.2f  Median:%.2f  Q3:%.2f  Max:%.2f",
      stats.min, stats.q1, stats.median, stats.q3, stats.max)
    table.insert(lines, label_line)
  end

  return lines
end

-- ============================================================================
-- SCATTER PLOT
-- ============================================================================

-- Create a scatter plot
-- @param data: array of {x, y} pairs
-- @param options: {width=60, height=20}
function M.scatter_plot(data, options)
  options = options or {}
  local width = options.width or 60
  local height = options.height or 20

  local lines = {}

  -- Find bounds
  local min_x, max_x = math.huge, -math.huge
  local min_y, max_y = math.huge, -math.huge

  for _, point in ipairs(data) do
    min_x = math.min(min_x, point.x)
    max_x = math.max(max_x, point.x)
    min_y = math.min(min_y, point.y)
    max_y = math.max(max_y, point.y)
  end

  -- Create grid
  local grid = {}
  for y = 1, height do
    grid[y] = {}
    for x = 1, width do
      grid[y][x] = ' '
    end
  end

  -- Plot points
  for _, point in ipairs(data) do
    local x_pos = math.floor(((point.x - min_x) / (max_x - min_x)) * (width - 1)) + 1
    local y_pos = height - math.floor(((point.y - min_y) / (max_y - min_y)) * (height - 1))

    if x_pos >= 1 and x_pos <= width and y_pos >= 1 and y_pos <= height then
      if grid[y_pos][x_pos] == ' ' then
        grid[y_pos][x_pos] = '●'
      elseif grid[y_pos][x_pos] == '●' then
        grid[y_pos][x_pos] = '◉'  -- Multiple points
      end
    end
  end

  -- Add axes
  for y = 1, height do
    local line = ""
    if y == height then
      line = string.format("%6.1f │", min_y)
    elseif y == 1 then
      line = string.format("%6.1f │", max_y)
    else
      line = "       │"
    end

    for x = 1, width do
      line = line .. grid[y][x]
    end

    table.insert(lines, line)
  end

  -- Add x-axis
  table.insert(lines, "       └" .. string.rep("─", width))
  table.insert(lines, string.format("       %.1f%s%.1f",
    min_x,
    string.rep(" ", width - 10),
    max_x))

  return lines
end

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Render chart lines to a buffer
function M.render_to_buffer(bufnr, lines, start_line)
  start_line = start_line or 0
  vim.api.nvim_buf_set_lines(bufnr, start_line, start_line + #lines, false, lines)
end

-- Create a new buffer with chart
function M.create_chart_buffer(lines, title)
  local bufnr = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_option(bufnr, 'buftype', 'nofile')
  vim.api.nvim_buf_set_option(bufnr, 'bufhidden', 'wipe')
  vim.api.nvim_buf_set_option(bufnr, 'modifiable', false)

  if title then
    table.insert(lines, 1, title)
    table.insert(lines, 2, string.rep("=", #title))
    table.insert(lines, 3, "")
  end

  M.render_to_buffer(bufnr, lines, 0)

  return bufnr
end

return M