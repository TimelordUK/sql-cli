# SQL CLI Charting - Quick Start

## Overview

SQL CLI now includes a **standalone charting tool** (`sql-cli-chart`) for creating terminal-based visualizations of CSV/JSON data. This is perfect for quick data exploration and analysis without needing external tools.

## Features

- **Terminal-native**: Pure ratatui-based charts, no GUI dependencies
- **Time series support**: Automatic timestamp parsing and scaling  
- **Interactive controls**: Vim-like navigation (hjkl), pan, zoom
- **Smart auto-scaling**: Adapts Y-axis ranges for optimal data visibility
- **SQL engine integration**: Leverages the full SQL query capabilities

## Installation

The chart tool is built as a separate binary alongside the main sql-cli:

```bash
cargo build --release
# Creates both binaries:
# - ./target/release/sql-cli (main TUI)
# - ./target/release/sql-cli-chart (charting tool)
```

## Basic Usage

```bash
sql-cli-chart <file.csv> -q "SQL_QUERY" -x X_COLUMN -y Y_COLUMN -t "Chart Title"
```

### Parameters

- **file.csv**: Input CSV or JSON file
- **-q, --query**: SQL query to filter/transform data (required)
- **-x, --x-axis**: Column name for X-axis (required)
- **-y, --y-axis**: Column name for Y-axis (required)
- **-t, --title**: Chart title (optional, default: "SQL CLI Chart")
- **-c, --chart-type**: Chart type - line, scatter, bar (optional, default: line)

## Examples

### Time Series: VWAP Price Analysis

```bash
# Basic price over time
./target/release/sql-cli-chart data/production_vwap_final.csv \\
  -q "SELECT snapshot_time, average_price FROM production_vwap_final WHERE filled_quantity > 0" \\
  -x snapshot_time \\
  -y average_price \\
  -t "VWAP Average Price Over Time"

# Volume analysis
./target/release/sql-cli-chart data/production_vwap_final.csv \\
  -q "SELECT snapshot_time, filled_quantity FROM production_vwap_final WHERE order_type LIKE '%CLIENT%'" \\
  -x snapshot_time \\
  -y filled_quantity \\
  -t "Client Order Fill Volume"
```

### Scatter Plot: Price vs Volume Correlation

```bash
./target/release/sql-cli-chart data/production_vwap_final.csv \\
  -q "SELECT average_price, filled_quantity FROM production_vwap_final WHERE filled_quantity > 0" \\
  -x average_price \\
  -y filled_quantity \\
  -c scatter \\
  -t "Price vs Volume Correlation"
```

## Interactive Controls

Once the chart opens, use these vim-like controls:

- **hjkl**: Pan left/down/up/right
- **+/-**: Zoom in/out
- **r**: Reset view to auto-fit data
- **q/Esc**: Quit

## Smart Features

### Auto-Scaling
The tool automatically:
- Detects timestamp columns and converts to epoch time for plotting
- Applies smart Y-axis scaling with 10% padding
- Handles different numeric ranges (prices vs quantities)

### Time Series Support
Timestamps in RFC3339 format (like `2025-08-12T09:00:18.030000`) are:
- Automatically detected and parsed
- Converted to epoch seconds for plotting
- Displayed as HH:MM:SS on the X-axis

## Architecture

The charting system is designed as a **standalone subsystem**:

- **Non-invasive**: Separate binary, doesn't interfere with main TUI
- **Modular**: Clean separation in `src/chart/` module
- **Extensible**: Easy to add new chart types and features
- **Reusable**: Can be integrated into main TUI later

## Current Limitations

1. **No SQL query execution yet**: Currently works with raw data (query parameter ignored for now)
2. **Terminal compatibility**: May have issues in non-TTY environments
3. **Limited chart types**: Only line charts fully implemented

## Roadmap

- [ ] Integrate SQL query execution 
- [ ] Add candlestick charts for OHLC data
- [ ] Export chart data to CSV
- [ ] Multiple data series on same chart
- [ ] Integration with main TUI (press 'C' for chart mode)

## Technical Details

### Data Flow
```
CSV/JSON → DataTable → DataView → SQL Query → Chart Data → Ratatui Rendering
```

### Key Components
- `ChartEngine`: Data processing and query execution
- `LineRenderer`: Time series visualization  
- `ChartTui`: Interactive terminal interface
- `ChartViewport`: Pan/zoom state management

## Troubleshooting

**"No such device or address" error**: This occurs in environments without proper TTY support. The chart tool requires a real terminal for interactive display.

**Empty chart**: Check that your SQL query returns data and column names match exactly.

**Poor scaling**: Use the 'r' key to reset auto-scaling, or manually pan/zoom to find your data.