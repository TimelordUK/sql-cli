#!/usr/bin/env lua

-- Test bar chart with actual SQL data
package.path = package.path .. ";../../nvim-plugin/lua/?.lua"

local charts = require('sql-cli.charts')

-- This is the exact data format the bar chart expects:
-- Column 1: Label (string) - becomes Y-axis labels
-- Column 2: Value (number) - determines bar length on X-axis

print("\n" .. string.rep("=", 70))
print("BAR CHART DATA FORMAT")
print(string.rep("=", 70))
print([[
The bar chart expects exactly 2 columns from your SQL query:

1. LABEL column (text)   → Y-axis labels (left side)
2. VALUE column (number) → X-axis bar length (horizontal bars)

SQL Query Structure:
SELECT label_column, numeric_column FROM table
]])

-- Example 1: Price Distribution
print("\n" .. string.rep("=", 70))
print("EXAMPLE 1: AAPL Price Distribution")
print(string.rep("=", 70))

local price_data = {
  {label = "$80-90", value = 38},
  {label = "$70-80", value = 137},
  {label = "Under $70", value = 153},
  {label = "$100-110", value = 164},
  {label = "$90-100", value = 169},
  {label = "Over $110", value = 598}
}

-- Sort by value for better visualization
table.sort(price_data, function(a, b) return a.value < b.value end)

local chart1 = charts.horizontal_bar_chart(price_data, {
  width = 40,
  show_values = true,
  bar_char = '█',
  empty_char = '░'
})

print("\nSQL Query:")
print([[
WITH price_buckets AS (
    SELECT CASE
        WHEN close < 70 THEN 'Under $70'
        WHEN close < 80 THEN '$70-80'
        WHEN close < 90 THEN '$80-90'
        WHEN close < 100 THEN '$90-100'
        WHEN close < 110 THEN '$100-110'
        ELSE 'Over $110'
    END as price_range
    FROM AAPL_data
)
SELECT
    price_range,      -- Column 1: Label (Y-axis)
    COUNT(*)          -- Column 2: Value (X-axis bar)
FROM price_buckets
GROUP BY price_range
ORDER BY COUNT(*)
]])

print("\nRendered Bar Chart:")
for _, line in ipairs(chart1) do
  print(line)
end

-- Example 2: Volatility Distribution
print("\n" .. string.rep("=", 70))
print("EXAMPLE 2: Volatility Distribution")
print(string.rep("=", 70))

local volatility_data = {
  {label = "Very Low (<0.5%)", value = 125},
  {label = "Low (0.5-1%)", value = 423},
  {label = "Medium (1-2%)", value = 512},
  {label = "High (2-3%)", value = 156},
  {label = "Very High (3-5%)", value = 38},
  {label = "Extreme (>5%)", value = 5}
}

local chart2 = charts.horizontal_bar_chart(volatility_data, {
  width = 35,
  show_values = true,
  bar_char = '▓',
  empty_char = '░'
})

print("\nSQL Query:")
print([[
SELECT
    CASE
        WHEN pct_change < 0.5 THEN 'Very Low (<0.5%)'
        WHEN pct_change < 1.0 THEN 'Low (0.5-1%)'
        WHEN pct_change < 2.0 THEN 'Medium (1-2%)'
        WHEN pct_change < 3.0 THEN 'High (2-3%)'
        WHEN pct_change < 5.0 THEN 'Very High (3-5%)'
        ELSE 'Extreme (>5%)'
    END as volatility_level,    -- Column 1: Label
    COUNT(*) as days            -- Column 2: Value
FROM daily_changes
GROUP BY volatility_level
]])

print("\nRendered Bar Chart:")
for _, line in ipairs(chart2) do
  print(line)
end

-- Example 3: Top 5 Trading Days
print("\n" .. string.rep("=", 70))
print("EXAMPLE 3: Top 5 Highest Volume Days")
print(string.rep("=", 70))

local volume_data = {
  {label = "2024-01-15", value = 266833581},
  {label = "2024-03-22", value = 198456732},
  {label = "2024-02-08", value = 187234561},
  {label = "2024-04-11", value = 176543218},
  {label = "2024-05-03", value = 165432109}
}

-- Scale values to millions for better display
for _, item in ipairs(volume_data) do
  item.value = item.value / 1000000  -- Convert to millions
end

local chart3 = charts.horizontal_bar_chart(volume_data, {
  width = 30,
  show_values = true,
  bar_char = '■'
})

print("\nSQL Query:")
print([[
SELECT
    date,              -- Column 1: Label (date)
    volume/1000000     -- Column 2: Value (millions)
FROM AAPL_data
ORDER BY volume DESC
LIMIT 5
]])

print("\nRendered Bar Chart (Volume in Millions):")
for _, line in ipairs(chart3) do
  print(line)
end

-- Show different bar styles
print("\n" .. string.rep("=", 70))
print("BAR CHART CUSTOMIZATION OPTIONS")
print(string.rep("=", 70))

local style_data = {
  {label = "Style 1", value = 75},
  {label = "Style 2", value = 100},
  {label = "Style 3", value = 50}
}

print("\nDefault Style (█░):")
local default_chart = charts.horizontal_bar_chart(style_data, {width = 20})
for _, line in ipairs(default_chart) do
  print(line)
end

print("\nBlock Style (■□):")
local block_chart = charts.horizontal_bar_chart(style_data, {
  width = 20,
  bar_char = '■',
  empty_char = '□'
})
for _, line in ipairs(block_chart) do
  print(line)
end

print("\nShaded Style (▓▒):")
local shade_chart = charts.horizontal_bar_chart(style_data, {
  width = 20,
  bar_char = '▓',
  empty_char = '▒'
})
for _, line in ipairs(shade_chart) do
  print(line)
end

print("\n" .. string.rep("=", 70))
print("KEY POINTS FOR BAR CHARTS")
print(string.rep("=", 70))
print([[
1. MUST have exactly 2 columns: label and value
2. First column becomes Y-axis labels (left side)
3. Second column must be numeric for bar length
4. Bars are drawn horizontally from left to right
5. Use ORDER BY to control bar order
6. Use LIMIT to show only top N items
7. Consider scaling large numbers (e.g., divide by 1M)
8. Keep labels short for better display

Common SQL patterns:
- COUNT(*) for frequency distributions
- AVG(column) for averages by category
- SUM(column) for totals by group
- MAX/MIN for extremes by category
]])