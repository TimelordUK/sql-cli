#!/usr/bin/env lua

-- Test and demo ASCII chart capabilities
-- Run with: lua test_ascii_charts.lua

-- Add the nvim-plugin path to package path
package.path = package.path .. ";../../nvim-plugin/lua/?.lua"

local charts = require('sql-cli.charts')

-- Helper to print chart
local function print_chart(title, lines)
  print("\n" .. string.rep("=", 60))
  print(title)
  print(string.rep("=", 60))
  for _, line in ipairs(lines) do
    print(line)
  end
end

-- ============================================================================
-- TEST 1: Bar Chart - Price Distribution
-- ============================================================================

local price_distribution = {
  {label = "Under $70", value = 85},
  {label = "$70-80", value = 134},
  {label = "$80-90", value = 124},
  {label = "$90-100", value = 169},
  {label = "$100-110", value = 164},
  {label = "Over $110", value = 598}
}

local bar_lines = charts.horizontal_bar_chart(price_distribution, {
  width = 40,
  show_values = true,
  bar_char = '█'
})

print_chart("BAR CHART: AAPL Price Distribution", bar_lines)

-- ============================================================================
-- TEST 2: Pie Chart - Market Share
-- ============================================================================

local market_share = {
  {label = "AAPL", value = 45.2},
  {label = "GOOGL", value = 28.5},
  {label = "MSFT", value = 15.3},
  {label = "AMZN", value = 8.7},
  {label = "Other", value = 2.3}
}

local pie_lines = charts.pie_chart(market_share, {
  radius = 8,
  show_legend = true
})

print_chart("PIE CHART: Tech Market Share", pie_lines)

-- ============================================================================
-- TEST 3: Histogram - Returns Distribution
-- ============================================================================

-- Simulated returns data (normal distribution)
math.randomseed(os.time())
local returns = {}
for i = 1, 100 do
  -- Box-Muller transform for normal distribution
  local u1 = math.random()
  local u2 = math.random()
  local z = math.sqrt(-2 * math.log(u1)) * math.cos(2 * math.pi * u2)
  local return_val = z * 0.02 + 0.001  -- mean=0.001, std=0.02
  table.insert(returns, return_val)
end

local hist_lines = charts.histogram(returns, {
  bins = 12,
  height = 10,
  width = 50
})

print_chart("HISTOGRAM: Daily Returns Distribution", hist_lines)

-- ============================================================================
-- TEST 4: Sparkline - Time Series
-- ============================================================================

-- Simulated stock prices
local prices = {100}
for i = 2, 50 do
  local change = (math.random() - 0.5) * 5
  prices[i] = math.max(80, math.min(120, prices[i-1] + change))
end

local spark_lines = charts.sparkline(prices, {width = 40})
print_chart("SPARKLINE: 50-Day Price Trend", spark_lines)

-- ============================================================================
-- TEST 5: Box Plot - Statistical Summary
-- ============================================================================

local stats = {
  min = 55.79,
  q1 = 87.42,
  median = 109.01,
  q3 = 131.89,
  max = 179.26,
  mean = 109.07
}

local box_lines = charts.box_plot(stats, {
  width = 50,
  show_labels = true
})

print_chart("BOX PLOT: AAPL Price Statistics", box_lines)

-- ============================================================================
-- TEST 6: Scatter Plot - Price vs Volume
-- ============================================================================

-- Simulated price/volume relationship
local scatter_data = {}
for i = 1, 30 do
  local price = 80 + math.random() * 100
  local volume = 20 + (price - 80) * 0.5 + (math.random() - 0.5) * 20
  table.insert(scatter_data, {x = price, y = volume})
end

local scatter_lines = charts.scatter_plot(scatter_data, {
  width = 50,
  height = 15
})

print_chart("SCATTER PLOT: Price vs Volume", scatter_lines)

-- ============================================================================
-- TEST 7: Multi-Series Bar Chart
-- ============================================================================

print("\n" .. string.rep("=", 60))
print("MULTI-CHART: Volatility Analysis")
print(string.rep("=", 60))

local volatility_periods = {
  {label = "Low Vol (<1%)", value = 425},
  {label = "Med Vol (1-3%)", value = 678},
  {label = "High Vol (>3%)", value = 156}
}

local vol_bars = charts.horizontal_bar_chart(volatility_periods, {
  width = 30,
  bar_char = '▓',
  empty_char = '░'
})

print("\nVolatility Distribution:")
for _, line in ipairs(vol_bars) do
  print(line)
end

-- Also show as percentages
print("\nAs Percentages:")
local total = 0
for _, item in ipairs(volatility_periods) do
  total = total + item.value
end

for _, item in ipairs(volatility_periods) do
  local pct = (item.value / total) * 100
  local bar_width = math.floor(pct / 2)
  print(string.format("%-15s %s %.1f%%",
    item.label,
    string.rep("█", bar_width),
    pct))
end

-- ============================================================================
-- SUMMARY
-- ============================================================================

print("\n" .. string.rep("=", 60))
print("ASCII CHART CAPABILITIES SUMMARY")
print(string.rep("=", 60))
print([[
Available chart types:
1. Bar Charts       - Distribution analysis, comparisons
2. Pie Charts       - Proportional data, market share
3. Histograms       - Frequency distributions
4. Sparklines       - Compact time series
5. Box Plots        - Statistical summaries (min, Q1, median, Q3, max)
6. Scatter Plots    - Correlation analysis

Integration with SQL CLI:
- Use statistical functions: MEDIAN, MODE, PERCENTILE, STDDEV, VARIANCE
- Combine with GROUP BY for category analysis
- Use RANGE function for synthetic data generation
- Leverage window functions for time series analysis

Example SQL queries for visualization:
:SqlBarChart SELECT price_bucket, COUNT(*) FROM prices GROUP BY price_bucket
:SqlPieChart SELECT category, SUM(value) FROM sales GROUP BY category
:SqlHistogram SELECT close FROM AAPL_data
:SqlBoxPlot SELECT MIN(x), PERCENTILE(x,25), MEDIAN(x), PERCENTILE(x,75), MAX(x) FROM data
:SqlSparkline SELECT date, close FROM AAPL_data ORDER BY date
:SqlScatter SELECT price, volume FROM AAPL_data
]])

print("\nAll tests completed successfully!")