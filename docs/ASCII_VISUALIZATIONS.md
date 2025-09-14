# SQL CLI ASCII Visualizations

## Overview

The SQL CLI nvim plugin now includes powerful ASCII chart rendering capabilities that work directly with your SQL query results. These visualizations help you quickly understand data distributions, trends, and patterns without leaving your editor.

## Available Chart Types

### 1. Bar Charts
Perfect for comparing categories or showing distributions.

```vim
:SqlBarChart SELECT price_bucket, COUNT(*) FROM prices GROUP BY price_bucket
```

Output:
```
Under $70  │████████████░░░░░░░░░░░░░░░░░░░ 85
$70-80     │████████████████████░░░░░░░░░░░ 134
$80-90     │██████████████████░░░░░░░░░░░░░ 124
$90-100    │████████████████████████░░░░░░░ 169
$100-110   │███████████████████████░░░░░░░░ 164
Over $110  │████████████████████████████████ 598
```

### 2. Pie Charts
Show proportional data with visual segments.

```vim
:SqlPieChart SELECT category, SUM(sales) FROM transactions GROUP BY category
```

Features:
- Circular representation with different fill characters
- Automatic legend with percentages
- Configurable radius

### 3. Histograms
Display frequency distributions of numeric data.

```vim
:SqlHistogram SELECT close_price FROM AAPL_data
```

Output:
```
 100 │      █ █
     │      █ █ █
     │    █ █ █ █ █
     │  █ █ █ █ █ █ █
   0 │█ █ █ █ █ █ █ █ █ █
     └────────────────────
      55.8         179.3
```

### 4. Box Plots
Statistical summaries showing quartiles and outliers.

```vim
:SqlBoxPlot SELECT MIN(price), PERCENTILE(price, 25), MEDIAN(price), PERCENTILE(price, 75), MAX(price), AVG(price) FROM data
```

Output:
```
       ▼ (mean)
├─────┌─────┼─────┐─────┤
Min:55.79  Q1:87.42  Median:109.01  Q3:131.89  Max:179.26
```

### 5. Sparklines
Compact time series visualization.

```vim
:SqlSparkline SELECT date, value FROM metrics ORDER BY date
```

Output:
```
▁▂▃▅▇█▇▅▃▂▁▂▄▆█▇▅▃▁
```

### 6. Scatter Plots
Show relationships between two variables.

```vim
:SqlScatter SELECT price, volume FROM trades
```

## Integration with Statistical Functions

The visualizations work perfectly with our new statistical functions:

```sql
-- Use MEDIAN, MODE, PERCENTILE for better analysis
SELECT
    MODE(category) as most_common,
    MEDIAN(value) as median_value,
    PERCENTILE(value, 75) as q3
FROM data
```

## Quick Statistical Analysis

Get instant statistical summaries with visualization:

```vim
:SqlStats table_name column_name
```

This shows:
- Count, Min, Max, Mean, Median
- Standard deviation and variance
- Histogram of distribution

## Configuration Options

### Bar Charts
```lua
{
  width = 60,           -- Chart width
  show_values = true,   -- Show numeric values
  bar_char = '█',       -- Character for bars
  empty_char = '░'      -- Character for empty space
}
```

### Histograms
```lua
{
  bins = 10,            -- Number of bins
  height = 15,          -- Chart height
  width = 60            -- Chart width
}
```

### Pie Charts
```lua
{
  radius = 10,          -- Circle radius
  show_legend = true    -- Display legend
}
```

## SQL Query Requirements

Different charts expect different query formats:

| Chart Type | Expected Columns | Example Query |
|------------|-----------------|---------------|
| Bar Chart | label, value | `SELECT name, count FROM groups` |
| Pie Chart | label, value | `SELECT category, sum FROM totals` |
| Histogram | value | `SELECT price FROM items` |
| Box Plot | min, q1, median, q3, max, [mean] | `SELECT MIN(x), PERCENTILE(x,25), ...` |
| Sparkline | value or date, value | `SELECT date, close FROM prices ORDER BY date` |
| Scatter | x, y | `SELECT price, volume FROM trades` |

## Usage Examples

### Distribution Analysis
```sql
-- Analyze price distribution
WITH buckets AS (
    SELECT
        CASE
            WHEN price < 100 THEN 'Low'
            WHEN price < 200 THEN 'Medium'
            ELSE 'High'
        END as bucket,
        price
    FROM products
)
SELECT bucket, COUNT(*) as frequency
FROM buckets
GROUP BY bucket
```

Then visualize: `:SqlBarChart`

### Time Series Analysis
```sql
-- Daily averages
SELECT
    date,
    AVG(value) as avg_value
FROM metrics
GROUP BY date
ORDER BY date
```

Then visualize: `:SqlSparkline`

### Statistical Summary
```sql
-- Complete statistical profile
SELECT
    MIN(value) as minimum,
    PERCENTILE(value, 25) as q1,
    MEDIAN(value) as median,
    PERCENTILE(value, 75) as q3,
    MAX(value) as maximum,
    AVG(value) as mean
FROM dataset
```

Then visualize: `:SqlBoxPlot`

## Tips and Tricks

1. **Use RANGE for testing**: Generate synthetic data for testing visualizations
   ```sql
   SELECT value, POWER(value, 2) as squared
   FROM RANGE(1, 100)
   ```

2. **Combine with CTEs**: Build complex analyses step by step
   ```sql
   WITH daily_stats AS (
       SELECT date, AVG(price) as avg_price
       FROM trades
       GROUP BY date
   )
   SELECT * FROM daily_stats
   ```

3. **Export visualizations**: Charts open in new buffers that can be saved
   - Use `:w chart.txt` to save
   - Copy to clipboard with visual selection

4. **Customize appearance**: Adjust width/height for your terminal size

## Performance Considerations

- Charts are rendered in pure Lua for speed
- Large datasets (>10,000 points) are automatically sampled
- Histograms automatically calculate optimal bin sizes
- Pie charts limit segments to maintain readability

## Future Enhancements

Planned features:
- [ ] Heatmaps for correlation matrices
- [ ] Time series with multiple lines
- [ ] Candlestick charts for financial data
- [ ] Network graphs for relationships
- [ ] Export to SVG/PNG via external tools

## Troubleshooting

**Chart doesn't appear**: Check that your query returns the expected columns
**Characters look wrong**: Ensure your terminal supports UTF-8
**Performance issues**: Try limiting data with `LIMIT` clause
**No data shown**: Verify numeric columns are actually numeric

## Integration with TUI

While these visualizations work in nvim, you can also:
- Use F5 in the TUI for debug views
- Export data and visualize externally
- Combine with the formatting functions for reports